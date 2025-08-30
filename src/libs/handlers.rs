use actix_web::{web, HttpRequest, HttpResponse, Result};
use tera::{Tera, Context};
use std::path::Path;
use chrono::Datelike;

use super::client::{detect_client_info, parse_screen_info, is_bot_request, generate_screen_detection_html};
use super::translations::Translations;
use super::config::Config;

/// Generic page handler with language from URL
pub async fn render_page_with_lang(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    translations: web::Data<Translations>,
    config: web::Data<Config>,
    template_path: &str,
    current_page: &str,
    lang: &str,
) -> Result<HttpResponse> {
    render_with_lang(req, tmpl, translations, config, template_path, current_page, lang).await
}

/// Generic page handler - DRY principle (deprecated, kept for compatibility)
pub async fn render_page(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    translations: web::Data<Translations>,
    config: web::Data<Config>,
    template_path: &str,
    current_page: &str,
) -> Result<HttpResponse> {
    render_with_client_detection(req, tmpl, translations, config, template_path, current_page).await
}

/// Render page with client detection
pub async fn render_with_client_detection(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    translations: web::Data<Translations>,
    config: web::Data<Config>,
    template_name: &str,
    current_page: &str,
) -> Result<HttpResponse> {
    // Skip detection for bots
    if !is_bot_request(&req) && parse_screen_info(&req).is_none() {
        return Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(generate_screen_detection_html()));
    }
    
    let mut client = detect_client_info(&req);
    
    // Check for language cookie first
    let mut cookie_lang = None;
    if let Some(cookie_header) = req.headers().get("cookie") {
        if let Ok(cookies_str) = cookie_header.to_str() {
            for cookie in cookies_str.split(';') {
                let trimmed = cookie.trim();
                if let Some(value) = trimmed.strip_prefix("lang=") {
                    let lang = value.to_lowercase();
                    if config.languages.available.contains(&lang) {
                        cookie_lang = Some(lang);
                        break;
                    }
                }
            }
        }
    }
    
    // Check for language override in query parameters (overrides cookie)
    let query_string = req.query_string();
    let mut selected_lang = None;
    
    // Parse query parameters manually
    for param in query_string.split('&') {
        if let Some(value) = param.strip_prefix("lang=") {
            let lang = value.to_lowercase();
            if config.languages.available.contains(&lang) {
                selected_lang = Some(lang);
                break;
            }
        }
    }
    
    // Priority: 1. Query param, 2. Cookie, 3. Browser detection
    client.lang = if let Some(lang) = selected_lang.clone() {
        lang
    } else if let Some(lang) = cookie_lang {
        lang
    } else {
        let lang_code = client.language[..2].to_lowercase();
        if config.languages.available.contains(&lang_code) {
            lang_code
        } else {
            config.languages.available.first().unwrap_or(&"en".to_string()).clone()
        }
    };
    
    // Get latest update date from git log or use current date as fallback
    let latest_update = std::process::Command::new("git")
        .args(&["log", "-1", "--format=%cd", "--date=short"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    
    // Create page info object
    let page_info = serde_json::json!({
        "languages": config.languages.available.clone(),
        "latest_update": latest_update
    });
    
    let mut context = Context::new();
    context.insert("current_year", &chrono::Local::now().year());
    context.insert("client", &client);
    context.insert("page", &page_info);
    context.insert("current_page", &current_page);
    
    // Create a simple translation map for the detected locale
    let locale_key = if client.lang == "de" { "de-DE" } else { "en-EN" };
    let t = translations.strings.get(locale_key)
        .or_else(|| translations.strings.get("en-EN"))
        .cloned()
        .unwrap_or_default();
    context.insert("t", &t);

    let rendered = tmpl
        .render(template_name, &context)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Build response with optional language cookie
    let mut response = HttpResponse::Ok();
    
    // If language was explicitly selected via query param, set a cookie
    if selected_lang.is_some() {
        response.cookie(
            actix_web::cookie::Cookie::build("lang", client.lang.clone())
                .path("/")
                .max_age(actix_web::cookie::time::Duration::days(365))
                .same_site(actix_web::cookie::SameSite::Lax)
                .finish()
        );
    }
    
    Ok(response.content_type("text/html").body(rendered))
}

/// Render page with language from URL
pub async fn render_with_lang(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    translations: web::Data<Translations>,
    config: web::Data<Config>,
    template_name: &str,
    current_page: &str,
    lang: &str,
) -> Result<HttpResponse> {
    // Skip detection for bots
    if !is_bot_request(&req) && parse_screen_info(&req).is_none() {
        return Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(generate_screen_detection_html()));
    }
    
    let mut client = detect_client_info(&req);
    
    // Use language from URL
    client.lang = lang.to_string();
    
    // Get latest update date from git log or use current date as fallback
    let latest_update = std::process::Command::new("git")
        .args(&["log", "-1", "--format=%cd", "--date=short"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    
    // Create page info object
    let page_info = serde_json::json!({
        "languages": config.languages.available.clone(),
        "latest_update": latest_update
    });
    
    let mut context = Context::new();
    context.insert("current_year", &chrono::Local::now().year());
    context.insert("client", &client);
    context.insert("page", &page_info);
    context.insert("current_page", &current_page);
    context.insert("current_lang", &lang);
    
    // Create a simple translation map for the detected locale
    let locale_key = if lang == "de" { "de-DE" } else { "en-EN" };
    let t = translations.strings.get(locale_key)
        .or_else(|| translations.strings.get("en-EN"))
        .cloned()
        .unwrap_or_default();
    context.insert("t", &t);

    let rendered = tmpl
        .render(template_name, &context)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Set language cookie
    Ok(HttpResponse::Ok()
        .cookie(
            actix_web::cookie::Cookie::build("lang", lang.to_string())
                .path("/")
                .max_age(actix_web::cookie::time::Duration::days(365))
                .same_site(actix_web::cookie::SameSite::Lax)
                .finish()
        )
        .content_type("text/html")
        .body(rendered))
}

/// Serve static files (CSS, JS, images, fonts)
pub async fn static_files(path: web::Path<String>) -> Result<HttpResponse> {
    let file_path = format!("static/{}", path);
    
    // Determine content type based on extension
    let content_type = match Path::new(&file_path).extension().and_then(|s| s.to_str()) {
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        _ => "application/octet-stream",
    };

    match std::fs::read(&file_path) {
        Ok(contents) => Ok(HttpResponse::Ok()
            .content_type(content_type)
            .body(contents)),
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

// Redirect to language-specific URL
pub async fn redirect_to_language(
    req: HttpRequest,
    config: web::Data<Config>,
) -> Result<HttpResponse> {
    // Check for language cookie
    let mut lang = "en".to_string(); // Default
    
    if let Some(cookie_header) = req.headers().get("cookie") {
        if let Ok(cookies_str) = cookie_header.to_str() {
            for cookie in cookies_str.split(';') {
                let trimmed = cookie.trim();
                if let Some(value) = trimmed.strip_prefix("lang=") {
                    let cookie_lang = value.to_lowercase();
                    if config.languages.available.contains(&cookie_lang) {
                        lang = cookie_lang;
                        break;
                    }
                }
            }
        }
    }
    
    // If no cookie, detect from browser
    if lang == "en" {
        if let Some(accept_lang) = req.headers().get("accept-language") {
            if let Ok(lang_str) = accept_lang.to_str() {
                for lang_part in lang_str.split(',') {
                    let detected = lang_part.split(';').next().unwrap_or("").trim();
                    let lang_code = detected[..2.min(detected.len())].to_lowercase();
                    if config.languages.available.contains(&lang_code) {
                        lang = lang_code;
                        break;
                    }
                }
            }
        }
    }
    
    // Get the current path
    let path = req.path();
    let redirect_url = format!("/{}{}", lang, path);
    
    Ok(HttpResponse::Found()
        .append_header(("Location", redirect_url))
        .finish())
}

// Route definitions as a macro for DRY
#[macro_export]
macro_rules! page_handler {
    ($name:ident, $template:expr, $page_name:expr) => {
        pub async fn $name(
            req: HttpRequest,
            lang: web::Path<String>,
            tmpl: web::Data<Tera>,
            translations: web::Data<Translations>,
            config: web::Data<Config>,
        ) -> Result<HttpResponse> {
            // Validate language
            let lang_str = lang.into_inner();
            if !config.languages.available.contains(&lang_str) {
                return Ok(HttpResponse::NotFound().finish());
            }
            
            render_page_with_lang(req, tmpl, translations, config, $template, $page_name, &lang_str).await
        }
    };
}

// Generate all page handlers
page_handler!(index, "content/index.tera", "index");
page_handler!(portfolio, "content/portfolio.tera", "portfolio");
page_handler!(knowledge, "content/knowledge.tera", "knowledge");
page_handler!(impressum, "content/impressum.tera", "impressum");
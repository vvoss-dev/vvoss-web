use actix_web::{web, HttpRequest, HttpResponse, Result};
use tera::{Tera, Context};
use std::path::Path;
use chrono::Datelike;

use super::client::{detect_client_info, parse_screen_info, is_bot_request, generate_screen_detection_html};
use super::translations::Translations;
use super::config::Config;

/// Generic page handler - DRY principle
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
    
    // Add language info to client struct
    let lang_code = client.language[..2].to_lowercase();
    client.lang = if config.languages.available.contains(&lang_code) {
        lang_code
    } else {
        config.languages.available.first().unwrap_or(&"en".to_string()).clone()
    };
    
    // Create page info object
    let page_info = serde_json::json!({
        "languages": config.languages.available.clone()
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

    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
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

// Route definitions as a macro for DRY
#[macro_export]
macro_rules! page_handler {
    ($name:ident, $template:expr, $page_name:expr) => {
        pub async fn $name(
            req: HttpRequest,
            tmpl: web::Data<Tera>,
            translations: web::Data<Translations>,
            config: web::Data<Config>,
        ) -> Result<HttpResponse> {
            render_page(req, tmpl, translations, config, $template, $page_name).await
        }
    };
}

// Generate all page handlers
page_handler!(index, "partials/index.tera", "index");
page_handler!(portfolio, "partials/portfolio.tera", "portfolio");
page_handler!(knowledge, "partials/knowledge.tera", "knowledge");
page_handler!(impressum, "partials/impressum.tera", "impressum");
use actix_web::{web, HttpRequest, HttpResponse, Result};
use tera::{Tera, Context};
use std::path::Path;
use chrono::Datelike;

use super::client::{detect_client_info, parse_screen_info, is_bot_request, generate_screen_detection_html};
use super::translations::Translations;

/// Generic page handler - DRY principle
pub async fn render_page(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    translations: web::Data<Translations>,
    template_path: &str,
) -> Result<HttpResponse> {
    render_with_client_detection(req, tmpl, translations, template_path).await
}

/// Render page with client detection
pub async fn render_with_client_detection(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    translations: web::Data<Translations>,
    template_name: &str,
) -> Result<HttpResponse> {
    // Skip detection for bots
    if !is_bot_request(&req) && parse_screen_info(&req).is_none() {
        return Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(generate_screen_detection_html()));
    }
    
    let client = detect_client_info(&req);
    let mut context = Context::new();
    context.insert("current_year", &chrono::Local::now().year());
    context.insert("client", &client);
    
    // Create a simple translation map for the detected locale
    let t = translations.strings.get(&client.language)
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
    ($name:ident, $template:expr) => {
        pub async fn $name(
            req: HttpRequest,
            tmpl: web::Data<Tera>,
            translations: web::Data<Translations>,
        ) -> Result<HttpResponse> {
            render_page(req, tmpl, translations, $template).await
        }
    };
}

// Generate all page handlers
page_handler!(index, "partials/index.tera");
page_handler!(portfolio, "partials/portfolio.tera");
page_handler!(impressum, "partials/impressum.tera");
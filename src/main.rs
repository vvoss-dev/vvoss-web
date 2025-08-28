use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result};
use std::os::unix::net::UnixListener;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tera::Tera;
use serde::Serialize;
use log::info;
use chrono::Datelike;

#[derive(Serialize)]
struct PageContext {
    title: String,
    description: String,
    current_year: i32,
}

/// Serve the homepage
async fn index(tmpl: web::Data<Tera>) -> Result<HttpResponse> {
    let context = PageContext {
        title: "vvoss.dev".to_string(),
        description: "Personal website of V. Voss".to_string(),
        current_year: chrono::Local::now().year(),
    };

    let rendered = tmpl
        .render("partials/index.tera", &tera::Context::from_serialize(&context).unwrap())
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

/// Serve the portfolio page
async fn portfolio(tmpl: web::Data<Tera>) -> Result<HttpResponse> {
    let context = PageContext {
        title: "Portfolio - vvoss.dev".to_string(),
        description: "Portfolio and projects".to_string(),
        current_year: chrono::Local::now().year(),
    };

    let rendered = tmpl
        .render("partials/portfolio.tera", &tera::Context::from_serialize(&context).unwrap())
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

/// Serve the imprint page
async fn impressum(tmpl: web::Data<Tera>) -> Result<HttpResponse> {
    let context = PageContext {
        title: "Impressum - vvoss.dev".to_string(),
        description: "Legal information".to_string(),
        current_year: chrono::Local::now().year(),
    };

    let rendered = tmpl
        .render("partials/impressum.tera", &tera::Context::from_serialize(&context).unwrap())
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

/// Serve static files (CSS, JS, images, fonts)
async fn static_files(path: web::Path<String>) -> Result<HttpResponse> {
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    info!("Loading templates...");
    let tera = Tera::new("templates/**/*.tera").expect("Failed to load templates");

    // Socket path - same as nginx expects
    let socket_path = "/var/run/sockets/vvoss_www.sock";
    
    // Remove socket if it exists
    if Path::new(socket_path).exists() {
        std::fs::remove_file(socket_path)?;
    }

    info!("Starting server on Unix socket: {}", socket_path);

    // Create Unix listener
    let listener = UnixListener::bind(socket_path)?;
    
    // Set socket permissions for nginx access
    std::fs::set_permissions(
        socket_path,
        std::fs::Permissions::from_mode(0o666)
    )?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(index))
            .route("/portfolio", web::get().to(portfolio))
            .route("/impressum", web::get().to(impressum))
            .route("/static/{filename:.*}", web::get().to(static_files))
    })
    .listen_uds(listener)?
    .run()
    .await
}
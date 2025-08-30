use actix_web::{middleware, web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use std::os::unix::net::UnixListener;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tera::Tera;
use log::info;

mod libs;

use libs::auth::validator;
use libs::config::Config;
use libs::translations::Translations;
use libs::handlers::{index, portfolio, knowledge, impressum, static_files, redirect_to_language};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Load configuration
    let config = Config::from_file("config.toml")
        .expect("Failed to load config.toml");
    
    let socket_path = config.server.socket_path.clone();

    // Load translations
    let translations = Translations::from_csv("templates/translations/strings.csv")
        .expect("Failed to load translations");

    info!("Loading templates...");
    let tera = Tera::new("templates/**/*.tera")
        .expect("Failed to load templates");
    
    // Remove socket if it exists
    if Path::new(&socket_path).exists() {
        std::fs::remove_file(&socket_path)?;
    }

    info!("Starting server on Unix socket: {}", socket_path);

    // Create Unix listener
    let listener = UnixListener::bind(&socket_path)?;
    
    // Set socket permissions for nginx access
    std::fs::set_permissions(
        &socket_path,
        std::fs::Permissions::from_mode(0o666)
    )?;

    HttpServer::new(move || {
        let auth = HttpAuthentication::basic(validator);
        
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(translations.clone()))
            .wrap(middleware::Logger::default())
            .wrap(auth)
            // Redirect root to default language
            .route("/", web::get().to(redirect_to_language))
            .route("/portfolio", web::get().to(redirect_to_language))
            .route("/knowledge", web::get().to(redirect_to_language))
            .route("/impressum", web::get().to(redirect_to_language))
            
            // Language-specific routes
            .route("/{lang}/", web::get().to(index))
            .route("/{lang}/portfolio", web::get().to(portfolio))
            .route("/{lang}/knowledge", web::get().to(knowledge))
            .route("/{lang}/impressum", web::get().to(impressum))
            
            // Static files (no language prefix)
            .route("/static/{filename:.*}", web::get().to(static_files))
    })
    .listen_uds(listener)?
    .run()
    .await
}
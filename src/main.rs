use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result, dev::ServiceRequest, Error, HttpRequest};
use actix_web_httpauth::{extractors::basic::BasicAuth, middleware::HttpAuthentication};
use std::os::unix::net::UnixListener;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::collections::HashMap;
use tera::{Tera, Context};
use serde::{Serialize, Deserialize};
use serde_json;
use log::info;
use chrono::Datelike;

#[derive(Serialize)]
struct PageContext {
    current_year: i32,
    locale: String,
}

#[derive(Serialize, Clone)]
struct ClientInfo {
    language: String,
    screen_width: Option<u32>,
    screen_height: Option<u32>,
    viewport_width: Option<u32>,
    viewport_height: Option<u32>,
    dpr: Option<f32>,      // device pixel ratio
    device_type: String,   // mobile, tablet, desktop, wide
    breakpoint: String,    // phone, tablet, screen, wide
}

#[derive(Deserialize)]
struct ScreenInfo {
    width: u32,
    height: u32,
    dpr: f32,
    viewport_width: u32,
    viewport_height: u32,
}

#[derive(Deserialize, Clone)]
struct Config {
    auth: AuthConfig,
    server: ServerConfig,
}

#[derive(Clone)]
struct Translations {
    strings: HashMap<String, HashMap<String, String>>,
}

#[derive(Deserialize, Clone)]
struct AuthConfig {
    enabled: bool,
    username: String,
    password: String,
}

#[derive(Deserialize, Clone)]
struct ServerConfig {
    socket_path: String,
}

/// Parse screen info from cookie
fn parse_screen_info(req: &HttpRequest) -> Option<ScreenInfo> {
    if let Some(cookie_header) = req.headers().get("cookie") {
        if let Ok(cookies_str) = cookie_header.to_str() {
            for cookie_part in cookies_str.split(';') {
                let trimmed = cookie_part.trim();
                if trimmed.starts_with("screen_info=") {
                    let value = &trimmed[12..];
                    if let Ok(decoded) = urlencoding::decode(value) {
                        if let Ok(screen_info) = serde_json::from_str::<ScreenInfo>(&decoded) {
                            return Some(screen_info);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Detect client information from headers and cookies
fn detect_client_info(req: &HttpRequest) -> ClientInfo {
    // Try to get screen info from cookie
    let screen_info = parse_screen_info(req);
    
    // Detect language from Accept-Language header
    let mut language = "en-EN".to_string();
    
    if let Some(accept_lang) = req.headers().get("accept-language") {
        if let Ok(lang_str) = accept_lang.to_str() {
            // Parse Accept-Language header (e.g., "de-DE,de;q=0.9,en;q=0.8")
            for lang_part in lang_str.split(',') {
                let lang = lang_part.split(';').next().unwrap_or("").trim();
                
                // Check if we support this language
                if lang.starts_with("de") {
                    language = "de-DE".to_string();
                    break;
                } else if lang.starts_with("en") {
                    language = "en-EN".to_string();
                    break;
                }
            }
        }
    }
    
    ClientInfo {
        language,
        screen_width: screen_info.as_ref().map(|s| s.width),
        screen_height: screen_info.as_ref().map(|s| s.height),
        viewport_width: screen_info.as_ref().map(|s| s.viewport_width),
        viewport_height: screen_info.as_ref().map(|s| s.viewport_height),
        dpr: screen_info.as_ref().map(|s| s.dpr),
        device_type: detect_device_type(req, &screen_info),
        breakpoint: detect_breakpoint(req, &screen_info),
    }
}

/// Detect device type from User-Agent and screen info
fn detect_device_type(req: &HttpRequest, screen_info: &Option<ScreenInfo>) -> String {
    // If we have screen info, use it for better detection
    if let Some(info) = screen_info {
        // Use viewport width for more accurate detection
        if info.viewport_width <= 559 {
            return "mobile".to_string();
        } else if info.viewport_width <= 959 {
            return "tablet".to_string();
        } else if info.viewport_width > 1920 {
            return "wide".to_string();
        }
    }
    
    // Fallback to User-Agent detection
    if let Some(user_agent) = req.headers().get("user-agent") {
        if let Ok(ua_str) = user_agent.to_str() {
            let ua_lower = ua_str.to_lowercase();
            
            // Bot/Crawler detection
            if ua_lower.contains("bot") || 
               ua_lower.contains("crawler") ||
               ua_lower.contains("spider") ||
               ua_lower.contains("googlebot") {
                return "bot".to_string();
            }
            
            // Mobile detection (phones)
            if ua_lower.contains("mobile") || 
               ua_lower.contains("android") || 
               ua_lower.contains("iphone") ||
               ua_lower.contains("windows phone") ||
               ua_lower.contains("blackberry") {
                return "mobile".to_string();
            }
            
            // Tablet detection
            if ua_lower.contains("ipad") || 
               ua_lower.contains("tablet") ||
               ua_lower.contains("kindle") ||
               ua_lower.contains("silk") {
                return "tablet".to_string();
            }
            
            // Smart TV detection
            if ua_lower.contains("smart-tv") ||
               ua_lower.contains("smarttv") ||
               ua_lower.contains("googletv") ||
               ua_lower.contains("appletv") {
                return "tv".to_string();
            }
            
            // Game console detection
            if ua_lower.contains("playstation") ||
               ua_lower.contains("xbox") ||
               ua_lower.contains("nintendo") {
                return "console".to_string();
            }
        }
    }
    
    "desktop".to_string()
}

/// Determine CSS breakpoint based on device type and screen info
fn detect_breakpoint(req: &HttpRequest, screen_info: &Option<ScreenInfo>) -> String {
    // If we have actual screen measurements, use them
    if let Some(info) = screen_info {
        if info.viewport_width <= 559 {
            return "phone".to_string();
        } else if info.viewport_width <= 959 {
            return "tablet".to_string();
        } else if info.viewport_width <= 1259 {
            return "screen".to_string();
        } else {
            return "wide".to_string();
        }
    }
    
    // Fallback to device type detection
    let device = detect_device_type(req, screen_info);
    match device.as_str() {
        "mobile" => "phone".to_string(),
        "tablet" => "tablet".to_string(),
        "wide" | "tv" | "console" => "wide".to_string(),
        "bot" => "screen".to_string(),
        _ => "screen".to_string(),
    }
}

/// Generate screen detection HTML
fn generate_screen_detection_html() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>body{margin:0;padding:0;}</style>
    <script>
    (function(){
        var d={
            width:screen.width,
            height:screen.height,
            dpr:window.devicePixelRatio||1,
            viewport_width:window.innerWidth||document.documentElement.clientWidth,
            viewport_height:window.innerHeight||document.documentElement.clientHeight
        };
        document.cookie='screen_info='+encodeURIComponent(JSON.stringify(d))+';path=/;max-age=31536000;SameSite=Lax';
        location.reload();
    })();
    </script>
</head>
<body></body>
</html>"#.to_string()
}

/// Helper to check if request is from a bot
fn is_bot_request(req: &HttpRequest) -> bool {
    req.headers().get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| {
            let lower = s.to_lowercase();
            lower.contains("bot") || lower.contains("crawler") || lower.contains("spider")
        })
        .unwrap_or(false)
}

/// Render page with client detection
async fn render_with_client_detection(
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

/// Serve the homepage
async fn index(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    translations: web::Data<Translations>,
) -> Result<HttpResponse> {
    render_with_client_detection(req, tmpl, translations, "partials/index.tera").await
}

/// Serve the portfolio page
async fn portfolio(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    translations: web::Data<Translations>,
) -> Result<HttpResponse> {
    render_with_client_detection(req, tmpl, translations, "partials/portfolio.tera").await
}

/// Serve the imprint page
async fn impressum(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    translations: web::Data<Translations>,
) -> Result<HttpResponse> {
    render_with_client_detection(req, tmpl, translations, "partials/impressum.tera").await
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

/// Basic auth validator
async fn validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // Load config from app data
    let config = req.app_data::<web::Data<Config>>().unwrap();
    
    if credentials.user_id() == config.auth.username {
        if let Some(password) = credentials.password() {
            if password == config.auth.password {
                return Ok(req);
            }
        }
    }
    
    Err((actix_web::error::ErrorUnauthorized("Unauthorized"), req))
}

fn load_translations() -> Translations {
    let csv_content = std::fs::read_to_string("templates/translations/strings.csv")
        .expect("Failed to read translations file");
    
    let mut strings: HashMap<String, HashMap<String, String>> = HashMap::new();
    
    for (i, line) in csv_content.lines().enumerate() {
        if i == 0 { continue; } // Skip header
        
        let parts: Vec<&str> = line.split(';').collect();
        if parts.len() == 3 {
            let key = parts[0];
            let text = parts[1];
            let locale = parts[2];
            
            strings.entry(locale.to_string())
                .or_insert_with(HashMap::new)
                .insert(key.to_string(), text.to_string());
        }
    }
    
    Translations { strings }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Load configuration
    let config_str = std::fs::read_to_string("config.toml")
        .expect("Failed to read config.toml");
    let config: Config = toml::from_str(&config_str)
        .expect("Failed to parse config.toml");
    
    let socket_path = config.server.socket_path.clone();

    // Load translations
    let translations = load_translations();

    info!("Loading templates...");
    let mut tera = Tera::new("templates/**/*.tera").expect("Failed to load templates");
    
    // No filter needed - we'll pass a translation helper function to context
    
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

    let config_clone = config.clone();
    HttpServer::new(move || {
        let auth = HttpAuthentication::basic(validator);
        
        App::new()
            .app_data(web::Data::new(config_clone.clone()))
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(translations.clone()))
            .wrap(middleware::Logger::default())
            .wrap(auth)
            .route("/", web::get().to(index))
            .route("/portfolio", web::get().to(portfolio))
            .route("/impressum", web::get().to(impressum))
            .route("/static/{filename:.*}", web::get().to(static_files))
    })
    .listen_uds(listener)?
    .run()
    .await
}
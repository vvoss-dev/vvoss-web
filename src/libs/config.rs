use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    #[allow(dead_code)]
    pub auth: AuthConfig,
    pub server: ServerConfig,
    pub languages: LanguagesConfig,
}

#[allow(dead_code)]
#[derive(Deserialize, Clone)]
pub struct AuthConfig {
    pub enabled: bool,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Clone)]
pub struct ServerConfig {
    pub socket_path: String,
}

#[derive(Deserialize, Clone)]
pub struct LanguagesConfig {
    pub available: Vec<String>,
}

impl Config {
    /// Load config from file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
}
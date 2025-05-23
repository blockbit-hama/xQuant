/**
* filename : lib
* author : HAMA
* date: 2025. 5. 8.
* description: 
**/

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::error::TradingError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub exchange: ExchangeConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub name: String,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub base_url: Option<String>,
    pub use_mock: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
}

impl Config {
    /// Load configuration from a file
    pub fn load() -> Result<Self, TradingError> {
        // Try to load from config.json
        let config_path = Path::new("config.json");

        if config_path.exists() {
            let mut file = File::open(config_path)
                .map_err(|e| TradingError::ConfigError(format!("Failed to open config file: {}", e)))?;

            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map_err(|e| TradingError::ConfigError(format!("Failed to read config file: {}", e)))?;

            serde_json::from_str(&contents)
                .map_err(|e| TradingError::ConfigError(format!("Failed to parse config file: {}", e)))
        } else {
            // Return default configuration
            Ok(Config::default())
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3030,
            },
            exchange: ExchangeConfig {
                name: "Mock".to_string(),
                api_key: None,
                api_secret: None,
                base_url: None,
                use_mock: true,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: None,
            },
        }
    }
}
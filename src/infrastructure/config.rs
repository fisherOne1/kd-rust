use crate::domain::error::KdError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_paging")]
    pub paging: bool,
    #[serde(default = "default_pager_command")]
    pub pager_command: String,
    #[serde(default)]
    pub english_only: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    pub http_proxy: Option<String>,
    #[serde(default)]
    pub clear_screen: bool,
    #[serde(default = "default_enable_emoji")]
    pub enable_emoji: bool,
    #[serde(default)]
    pub freq_alert: bool,
    #[serde(default)]
    pub logging: Logging,
    #[serde(default)]
    pub youdao: YoudaoConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Logging {
    #[serde(default = "default_enable")]
    pub enable: bool,
    pub path: Option<String>,
    #[serde(default = "default_log_level")]
    pub level: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct YoudaoConfig {
    pub api_id: Option<String>,
    pub api_key: Option<String>,
}

impl Default for Logging {
    fn default() -> Self {
        Self {
            enable: true,
            path: None,
            level: "WARN".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            paging: true,
            pager_command: default_pager_command(),
            english_only: false,
            theme: default_theme(),
            http_proxy: None,
            clear_screen: false,
            enable_emoji: true,
            freq_alert: false,
            logging: Logging::default(),
            youdao: YoudaoConfig::default(),
        }
    }
}

// Defaults
fn default_paging() -> bool {
    true
}
fn default_pager_command() -> String {
    // Windows 使用 more，Unix 系统使用 less
    if cfg!(target_os = "windows") {
        "more".to_string()
    } else {
        "less -RF".to_string()
    }
}
fn default_theme() -> String {
    "temp".to_string()
}
fn default_enable_emoji() -> bool {
    true
}
fn default_enable() -> bool {
    true
}
fn default_log_level() -> String {
    "WARN".to_string()
}

pub fn get_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("kd").join("config.toml"))
}

/// Get database path (uses config directory by default)
pub fn get_database_path(_config: &Config) -> PathBuf {
    // Use config directory: ~/.config/kd/kd.db (Linux)
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("kd")
        .join("kd.db")
}

pub fn load_config() -> Result<Config, KdError> {
    let config_path = get_config_path();

    if let Some(path) = config_path {
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            match toml::from_str::<Config>(&content) {
                Ok(config) => return Ok(config),
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse config file: {}. Using defaults.",
                        e
                    );
                }
            }
        }
    }

    Ok(Config::default())
}

pub fn generate_config_sample() -> Result<(), KdError> {
    let config_path = get_config_path();

    if let Some(path) = config_path {
        if path.exists() {
            eprintln!("Config file already exists at: {}", path.display());
            return Ok(());
        }

        // Create directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Generate sample config
        let sample = Config::default();
        let toml_content = toml::to_string_pretty(&sample)
            .map_err(|e| KdError::Config(format!("Failed to serialize config: {}", e)))?;
        fs::write(&path, toml_content)
            .map_err(|e| KdError::Config(format!("Failed to write config file: {}", e)))?;
        println!("Generated config file at: {}", path.display());
    } else {
        return Err(KdError::Config(
            "Cannot determine config directory".to_string(),
        ));
    }

    Ok(())
}

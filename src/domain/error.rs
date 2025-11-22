use thiserror::Error;

#[derive(Error, Debug)]
pub enum KdError {
    #[error("Database error: {0}")]
    Database(#[from] tokio_rusqlite::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Compression error: {0}")]
    #[allow(dead_code)]
    Compression(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Start-up error: {0}")]
    #[allow(dead_code)]
    Init(String),

    #[error("API Error: {0}")]
    Api(String),

    #[error("Time error: {0}")]
    Time(#[from] std::time::SystemTimeError),
}

use crate::domain::error::KdError;
use crate::domain::model::QueryResult;
use crate::infrastructure::config::Config;
use dashmap::DashMap;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_rusqlite::Connection;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Connection>,
    pub cache: Arc<DashMap<String, QueryResult>>,
    pub config: Arc<RwLock<Config>>,
    pub http_client: Client,
}

impl AppState {
    pub fn new(db: Connection, config: Config) -> Result<Self, KdError> {
        let mut client_builder = Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(std::time::Duration::from_secs(30))
            .user_agent("kd/0.1.0");

        // Configure HTTP/HTTPS proxy if specified
        if let Some(proxy_url) = &config.http_proxy {
            if !proxy_url.is_empty() {
                // Support both http:// and https:// proxy URLs
                let proxy = if proxy_url.starts_with("https://") {
                    reqwest::Proxy::https(proxy_url)
                        .map_err(|e| KdError::Config(format!("Invalid HTTPS proxy URL: {}", e)))?
                } else {
                    reqwest::Proxy::http(proxy_url)
                        .map_err(|e| KdError::Config(format!("Invalid HTTP proxy URL: {}", e)))?
                };
                client_builder = client_builder.proxy(proxy);
            }
        }

        let http_client = client_builder.build()?;

        Ok(Self {
            db: Arc::new(db),
            cache: Arc::new(DashMap::new()),
            config: Arc::new(RwLock::new(config)),
            http_client,
        })
    }
}

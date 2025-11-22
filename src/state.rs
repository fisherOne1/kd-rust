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
        let http_client = Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(std::time::Duration::from_secs(30))
            .user_agent("kd/0.1.0")
            .build()?;

        Ok(Self {
            db: Arc::new(db),
            cache: Arc::new(DashMap::new()),
            config: Arc::new(RwLock::new(config)),
            http_client,
        })
    }
}

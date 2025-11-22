// HTTP client utilities
use crate::domain::error::KdError;
use reqwest::Client;

/// Create a default HTTP client with appropriate settings
///
/// Reserved for future use. Currently creating client directly in AppState.
#[allow(dead_code)]
pub fn create_client() -> Result<Client, KdError> {
    Ok(Client::builder()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("kd/0.1.0")
        .build()?)
}

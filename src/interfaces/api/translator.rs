use crate::domain::error::KdError;
use crate::domain::model::QueryResult;
use async_trait::async_trait;

/// Trait for translation services (reserved for future multi-provider support)
#[async_trait]
#[allow(dead_code)]
pub trait Translator {
    async fn translate(&self, query: &str) -> Result<QueryResult, KdError>;
}

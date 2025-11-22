use crate::domain::error::KdError;
use crate::domain::model::QueryResult;
use async_trait::async_trait;

/// Trait for translation services
///
/// This trait provides an abstraction for different translation providers.
/// Implementations can be swapped without changing the calling code.
/// Reserved for future multi-provider support and testing.
#[async_trait]
#[allow(dead_code)]
pub trait Translator {
    /// Translate a query string
    async fn translate(&self, query: &str) -> Result<QueryResult, KdError>;
}

/// Trait for database operations
///
/// This trait abstracts database operations, allowing different database
/// implementations to be used (SQLite, PostgreSQL, etc.)
/// Reserved for future database abstraction and testing.
#[async_trait]
#[allow(dead_code)]
pub trait Database {
    /// Query cache by query string
    async fn query_cache(&self, query: &str) -> Result<Option<QueryResult>, KdError>;

    /// Insert a query result into cache
    async fn insert_cache(&self, query: &str, result: &QueryResult) -> Result<(), KdError>;

    /// Batch insert multiple query results (for migration)
    async fn batch_insert_cache(&self, items: Vec<(String, QueryResult)>)
        -> Result<usize, KdError>;
}

/// Trait for cache operations
///
/// This trait abstracts in-memory cache operations.
/// Reserved for future cache abstraction and testing.
#[allow(dead_code)]
pub trait Cache {
    /// Get a cached result by key
    fn get(&self, key: &str) -> Option<QueryResult>;

    /// Insert a result into cache
    fn insert(&self, key: String, value: QueryResult);
}

// In-memory cache implementation using DashMap
use crate::domain::model::QueryResult;
use dashmap::DashMap;

/// Thread-safe in-memory cache
///
/// Reserved for future use with Cache trait abstraction.
/// Currently using DashMap directly in AppState for simplicity.
#[allow(dead_code)]
pub struct MemoryCache {
    map: DashMap<String, QueryResult>,
}

impl MemoryCache {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<QueryResult> {
        self.map.get(key).map(|entry| entry.value().clone())
    }

    #[allow(dead_code)]
    pub fn insert(&self, key: String, value: QueryResult) {
        self.map.insert(key, value);
    }

    #[allow(dead_code)]
    pub fn clear(&self) {
        self.map.clear();
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.map.len()
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

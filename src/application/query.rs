use crate::domain::error::KdError;
use crate::domain::model::{QueryResult, QuerySource};
use crate::infrastructure::network::client::query_youdao;
use crate::infrastructure::storage::db::{insert_cache, query_cache};
use crate::state::AppState;
use chrono::Utc;

pub async fn query_word(
    state: &AppState,
    query: &str,
    no_cache: bool,
    is_long_text: bool,
) -> Result<QueryResult, KdError> {
    // 1. Memory Cache
    if !no_cache {
        if let Some(cached) = state.cache.get(query) {
            let mut res = cached.clone();
            res.source = QuerySource::LocalCache;
            return Ok(res);
        }
    }

    // 2. Database Cache
    if !no_cache {
        if let Some(cached) = query_cache(&state.db, query).await? {
            // Update memory cache
            state.cache.insert(query.to_string(), cached.clone());

            let mut res = cached;
            // Keep original source if it was from online, otherwise mark as offline
            if matches!(res.source, QuerySource::Online(_)) {
                // Keep online source for display
            } else {
                res.source = QuerySource::OfflineDb;
            }
            return Ok(res);
        }
    }

    // 3. Online Query
    // Use a read lock for config, but don't hold it across await if possible or safe
    let mut result = {
        let config = state.config.read().await;
        query_youdao(&state.http_client, &config, query).await?
    };

    // If needed, update is_long_text if not set by API
    if is_long_text {
        result.is_long_text = true;
    }

    // 4. Write back to cache
    // Only cache if found
    if !no_cache && result.found {
        result.cached_at = Some(Utc::now().timestamp());

        // Update memory cache
        state.cache.insert(query.to_string(), result.clone());

        // Update DB cache
        let db_result = result.clone();
        let db = state.db.clone();
        let q = query.to_string();

        insert_cache(&db, &q, &db_result).await?;
    }

    Ok(result)
}

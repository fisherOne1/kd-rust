use crate::domain::error::KdError;
use crate::domain::model::QueryResult;
use crate::domain::traits::Database;
use async_trait::async_trait;
use std::path::Path;
use tokio_rusqlite::Connection;

pub async fn init_database(db_path: &Path) -> Result<Connection, KdError> {
    let db = Connection::open(db_path.to_path_buf()).await?;

    db.call(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cache (
                query TEXT PRIMARY KEY,
                data BLOB NOT NULL,
                compressed_size INTEGER NOT NULL,
                original_size INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cache_updated ON cache(updated_at)",
            [],
        )?;

        Ok(())
    })
    .await?;

    Ok(db)
}

/// SQLite database implementation
///
/// Reserved for future use with Database trait abstraction.
#[allow(dead_code)]
pub struct SqliteDatabase {
    conn: Connection,
}

impl SqliteDatabase {
    #[allow(dead_code)]
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl Database for SqliteDatabase {
    async fn query_cache(&self, query: &str) -> Result<Option<QueryResult>, KdError> {
        query_cache_impl(&self.conn, query).await
    }

    async fn insert_cache(&self, query: &str, result: &QueryResult) -> Result<(), KdError> {
        insert_cache_impl(&self.conn, query, result).await
    }

    async fn batch_insert_cache(
        &self,
        items: Vec<(String, QueryResult)>,
    ) -> Result<usize, KdError> {
        batch_insert_cache_impl(&self.conn, items).await
    }
}

// Internal implementation functions (kept for backward compatibility)
pub async fn query_cache(db: &Connection, query: &str) -> Result<Option<QueryResult>, KdError> {
    query_cache_impl(db, query).await
}

pub async fn insert_cache(
    db: &Connection,
    query: &str,
    result: &QueryResult,
) -> Result<(), KdError> {
    insert_cache_impl(db, query, result).await
}

pub async fn batch_insert_cache(
    db: &Connection,
    items: Vec<(String, QueryResult)>,
) -> Result<usize, KdError> {
    batch_insert_cache_impl(db, items).await
}

// Internal implementation
async fn query_cache_impl(db: &Connection, query: &str) -> Result<Option<QueryResult>, KdError> {
    use rusqlite::OptionalExtension;
    use std::io::Cursor;
    use tokio_rusqlite::params;
    use zstd::stream::decode_all;

    let query_string = query.to_string();
    let result = db
        .call(move |conn| {
            conn.query_row(
                "SELECT data FROM cache WHERE query = ?",
                params![query_string],
                |row| {
                    let compressed_data: Vec<u8> = row.get(0)?;
                    let decompressed = decode_all(Cursor::new(&compressed_data)).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Blob,
                            Box::new(e),
                        )
                    })?;
                    let result: QueryResult =
                        serde_json::from_slice(&decompressed).map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                0,
                                rusqlite::types::Type::Blob,
                                Box::new(e),
                            )
                        })?;
                    Ok(result)
                },
            )
            .optional()
        })
        .await?;

    Ok(result)
}

async fn insert_cache_impl(
    db: &Connection,
    query: &str,
    result: &QueryResult,
) -> Result<(), KdError> {
    use std::io::Cursor;
    use tokio_rusqlite::params;
    use zstd::stream::encode_all;

    let serialized = serde_json::to_vec(result)?;
    let compressed = encode_all(Cursor::new(&serialized), 0)?;
    let now = chrono::Utc::now().timestamp();

    let query_string = query.to_string();
    let compressed_len = compressed.len();
    let original_len = serialized.len();

    db.call(move |conn| {
        conn.execute(
            "INSERT OR REPLACE INTO cache (query, data, compressed_size, original_size, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![
                query_string,
                compressed,
                compressed_len,
                original_len,
                now,
                now
            ],
        )
    }).await?;

    Ok(())
}

async fn batch_insert_cache_impl(
    db: &Connection,
    items: Vec<(String, QueryResult)>,
) -> Result<usize, KdError> {
    use std::io::Cursor;
    use tokio_rusqlite::params;
    use zstd::stream::encode_all;

    let now = chrono::Utc::now().timestamp();

    let prepared_items: Vec<_> = items
        .into_iter()
        .filter_map(|(query, result)| {
            let serialized = serde_json::to_vec(&result).ok()?;
            let compressed = encode_all(Cursor::new(&serialized), 0).ok()?;
            let compressed_len = compressed.len();
            let original_len = serialized.len();
            Some((query, compressed, compressed_len, original_len))
        })
        .collect();

    if prepared_items.is_empty() {
        return Ok(0);
    }

    let success_count = db.call(move |conn| {
        let tx = conn.transaction()?;
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO cache (query, data, compressed_size, original_size, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)"
        )?;

        let mut count = 0;
        for (query, compressed, compressed_len, original_len) in prepared_items {
            if stmt.execute(params![query, compressed, compressed_len, original_len, now, now]).is_ok() {
                count += 1;
            }
        }

        stmt.finalize()?;
        tx.commit()?;
        Ok(count)
    }).await?;

    Ok(success_count)
}

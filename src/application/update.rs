use crate::domain::error::KdError;
use crate::domain::model::QueryResult;
use crate::infrastructure::storage::db::batch_insert_cache;
use crate::migration::legacy::LegacyResult;
use crate::state::AppState;
use flate2::read::ZlibDecoder;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use rusqlite::Connection;
use serde::Deserialize;
use std::io::Read;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

const DATA_ZIP_URL_CN: &str = "https://gitee.com/void_kmz/kd/releases/download/v0.0.1/kd_data.zip";
const DATA_ZIP_URL_GLOBAL: &str =
    "https://raw.githubusercontent.com/Karmenzind/static/main/kd/kd_data.zip";

#[derive(Debug, Deserialize)]
struct IPInfo {
    country: String,
}

impl IPInfo {
    fn is_cn(&self) -> bool {
        self.country.to_uppercase() == "CN"
    }
}

/// Get download URL based on IP location
async fn get_download_url(client: &Client) -> &'static str {
    // Try to detect IP location, default to CN if detection fails
    match detect_ip_location(client).await {
        Ok(info) => {
            if !info.is_cn() {
                println!("Detected non-CN IP, using global download source.");
                return DATA_ZIP_URL_GLOBAL;
            }
        }
        Err(e) => {
            println!(
                "Failed to detect IP location ({}), using CN source as fallback.",
                e
            );
        }
    }
    DATA_ZIP_URL_CN
}

/// Detect IP location using ipinfo.io
async fn detect_ip_location(client: &Client) -> Result<IPInfo, KdError> {
    let response = client
        .get("https://ipinfo.io/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    let info: IPInfo = response.json().await?;
    Ok(info)
}

pub async fn update_dict(state: &AppState) -> Result<(), KdError> {
    let config_guard = state.config.read().await;
    let db_path = crate::infrastructure::config::get_database_path(&config_guard);
    let data_dir = db_path.parent().unwrap();
    drop(config_guard);

    let zip_path = data_dir.join("kd_data.zip");

    // Auto download and extract
    if !zip_path.exists() {
        let download_url = get_download_url(&state.http_client).await;
        let source_name = if download_url == DATA_ZIP_URL_CN {
            "Gitee (CN)"
        } else {
            "GitHub (Global)"
        };
        println!("Downloading dictionary data from {}...", source_name);
        download_file(&state.http_client, download_url, &zip_path).await?;
    } else {
        println!("Zip file already exists, skipping download.");
    }

    println!("Extracting...");
    extract_zip(&zip_path, data_dir).await?;

    // Find extracted DB file
    let source_db_path = find_db_file(data_dir)
        .await
        .ok_or_else(|| KdError::Io(std::io::Error::other("No DB file found in extracted zip")))?;

    println!("Found database file: {:?}", source_db_path);

    println!("Migrating data from {:?}...", source_db_path);
    migrate_data(&source_db_path, &state.db).await?;

    // Cleanup
    if zip_path.exists() {
        tokio::fs::remove_file(&zip_path).await?;
        println!("Cleaned up zip file.");
    }
    if source_db_path.parent() == Some(data_dir) {
        tokio::fs::remove_file(&source_db_path).await?;
        println!("Cleaned up extracted DB file.");
    }

    println!("Dictionary update complete!");
    Ok(())
}

async fn download_file(client: &Client, url: &str, path: &Path) -> Result<(), KdError> {
    let res = client.get(url).send().await?;
    let total_size = res.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    let mut file = File::create(path).await?;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(KdError::Http)?;
        file.write_all(&chunk).await?;
        pb.inc(chunk.len() as u64);
    }

    pb.finish_with_message("Downloaded");
    Ok(())
}

async fn extract_zip(zip_path: &Path, dest: &Path) -> Result<(), KdError> {
    // Use spawn_blocking for CPU-intensive zip extraction
    let zip_path = zip_path.to_path_buf();
    let dest = dest.to_path_buf();

    tokio::task::spawn_blocking(move || -> Result<(), KdError> {
        use std::fs::File as StdFile;

        let file = StdFile::open(&zip_path)?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| KdError::Io(std::io::Error::other(e)))?;

        println!("Extracting {} files...", archive.len());
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| KdError::Io(std::io::Error::other(e)))?;
            let outpath = dest.join(file.mangled_name());

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = StdFile::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }
        Ok(())
    })
    .await
    .map_err(|e| KdError::Io(std::io::Error::other(format!("Task join error: {}", e))))?
}

async fn find_db_file(dir: &Path) -> Option<PathBuf> {
    let dir = dir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let entries = std::fs::read_dir(&dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("db")
                && path.file_name().unwrap() != "kd.db"
            {
                return Some(path);
            }
        }
        None
    })
    .await
    .ok()?
}

type MigrationRow = (String, Vec<u8>);
type MigrationData = (i64, Vec<MigrationRow>);

async fn migrate_data(
    source_db_path: &Path,
    target_conn: &tokio_rusqlite::Connection,
) -> Result<(), KdError> {
    let source_db_path = source_db_path.to_path_buf();
    let tables = vec!["en", "ch"];

    for table in tables {
        // Read all data from source database in a single blocking task
        let table_clone = table.to_string();
        let source_path = source_db_path.clone();
        let (total, rows): MigrationData = tokio::task::spawn_blocking(
            move || -> Result<MigrationData, KdError> {
                let src_conn = Connection::open(&source_path).map_err(KdError::Sqlite)?;

                // Check table existence
                let table_exists: bool = src_conn
                    .query_row(
                        "SELECT exists(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?)",
                        [&table_clone],
                        |row| row.get(0),
                    )
                    .unwrap_or(false);

                if !table_exists {
                    return Ok((0, Vec::new()));
                }

                // Get total count
                let total: i64 = src_conn
                    .query_row(
                        &format!("SELECT count(*) FROM {}", table_clone),
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);

                // Read all rows
                let mut stmt = src_conn
                    .prepare(&format!("SELECT query, detail FROM {}", table_clone))
                    .map_err(KdError::Sqlite)?;

                let rows = stmt
                    .query_map([], |row| {
                        let query: String = row.get(0)?;
                        let detail: Vec<u8> = row.get(1)?;
                        Ok((query, detail))
                    })
                    .map_err(KdError::Sqlite)?;

                let rows_vec = rows
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(KdError::Sqlite)?;
                Ok((total, rows_vec))
            },
        )
        .await
        .map_err(|e| KdError::Io(std::io::Error::other(format!("Task join error: {}", e))))??;

        if rows.is_empty() {
            println!("Table {} not found in source DB or empty, skipping.", table);
            continue;
        }

        println!("Table {} has {} records", table, total);
        println!("Starting migration...");

        println!("Processing records...");
        let pb = ProgressBar::new(total as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
            .unwrap()
            .progress_chars("#>-"));
        pb.set_message(format!("Migrating {}", table));

        let mut count = 0;
        let mut error_count = 0;
        let mut batch = Vec::new();
        const BATCH_SIZE: usize = 100;

        for (i, (query, detail_bytes)) in rows.iter().enumerate() {
            let mut decoder = ZlibDecoder::new(&detail_bytes[..]);
            let mut decompressed = Vec::new();

            let final_bytes = if decoder.read_to_end(&mut decompressed).is_ok() {
                decompressed
            } else {
                detail_bytes.clone()
            };

            match serde_json::from_slice::<LegacyResult>(&final_bytes) {
                Ok(legacy) => {
                    let new_result = convert_legacy(legacy);
                    batch.push((query.clone(), new_result));

                    // Batch insert using transaction
                    if batch.len() >= BATCH_SIZE {
                        match batch_insert_cache(target_conn, std::mem::take(&mut batch)).await {
                            Ok(inserted) => {
                                count += inserted;
                                pb.set_message(format!("Migrating {} - {} inserted", table, count));
                            }
                            Err(e) => {
                                error_count += BATCH_SIZE;
                                if error_count <= 10 {
                                    eprintln!("Batch insert error: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    error_count += 1;
                }
            }

            // Print status every 100 records
            if i > 0 && i % 100 == 0 {
                println!("Processed {} records, inserted {} so far...", i, count);
            }

            // Update progress bar
            if i % 100 == 0 {
                pb.set_position(i as u64);
            }
        }

        // Insert remaining batch
        if !batch.is_empty() {
            match batch_insert_cache(target_conn, batch).await {
                Ok(inserted) => count += inserted,
                Err(e) => {
                    eprintln!("Final batch insert error: {}", e);
                    error_count += 1;
                }
            }
        }

        pb.finish_with_message(format!(
            "Table {} done. Inserted {} records, {} errors.",
            table, count, error_count
        ));
        println!(
            "\nTable {} completed: {} records inserted, {} errors",
            table, count, error_count
        );
    }

    Ok(())
}

fn convert_legacy(legacy: LegacyResult) -> QueryResult {
    let keyword = legacy.keyword.unwrap_or_else(|| "unknown".to_string());
    let mut result = QueryResult::new(keyword.clone(), false);
    result.found = true;

    // Handle pronunciation - support both Chinese keys (美/英) and English keys (us/uk)
    if let Some(pron) = legacy.pronounce {
        // Store both US and UK pronunciations separately
        if let Some(us) = pron.get("美") {
            result.pronunciation_us = Some(us.clone());
            result.pronunciation = Some(us.clone()); // Keep for backward compatibility
        } else if let Some(us) = pron.get("us") {
            result.pronunciation_us = Some(us.clone());
            result.pronunciation = Some(us.clone());
        }

        if let Some(uk) = pron.get("英") {
            result.pronunciation_uk = Some(uk.clone());
            if result.pronunciation.is_none() {
                result.pronunciation = Some(uk.clone());
            }
        } else if let Some(uk) = pron.get("uk") {
            result.pronunciation_uk = Some(uk.clone());
            if result.pronunciation.is_none() {
                result.pronunciation = Some(uk.clone());
            }
        }
    }

    if let Some(para) = legacy.paraphrase {
        result.translations = para;
    }

    // Handle examples from top-level eg field
    if let Some(egs) = legacy.examples {
        for (_, list) in egs {
            for pair in list {
                if pair.len() >= 2 {
                    result.examples.push((pair[0].clone(), pair[1].clone()));
                }
            }
        }
    }

    // Handle Collins dictionary (co.li[].eg) - preserve full structure
    if let Some(collins) = legacy.collins {
        // Store rank information
        if let Some(rank) = collins.rank {
            result.collins_rank = Some(rank);
        }

        // Store Collins items with full structure
        if let Some(items) = collins.items {
            for item in items {
                let mut collins_item = crate::domain::model::CollinsDisplayItem {
                    additional: item.additional.clone(),
                    major_trans: item.major_trans.clone(),
                    examples: Vec::new(),
                };

                if let Some(egs) = item.examples {
                    for pair in egs {
                        if pair.len() >= 2 {
                            collins_item
                                .examples
                                .push((pair[0].clone(), pair[1].clone()));
                        }
                    }
                }

                // Only add if it has content
                if collins_item.major_trans.is_some() || !collins_item.examples.is_empty() {
                    result.collins_items.push(collins_item);
                }
            }
        }
    }

    result.source = crate::domain::model::QuerySource::OfflineDb;
    result
}

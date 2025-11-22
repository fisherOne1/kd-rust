// Main entry point
mod application;
mod domain;
mod infrastructure;
mod interfaces;
mod migration;
mod presentation;
mod state;

use clap::Parser;
use colored::Colorize;
use infrastructure::config::load_config;
use interfaces::cli::Cli;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup graceful shutdown handler
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    // Spawn signal handler task
    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            eprintln!("Failed to listen for shutdown signal: {}", e);
        } else {
            eprintln!("\n收到中断信号，正在优雅关闭...");
            let _ = shutdown_tx.send(());
        }
    });

    let cli = Cli::parse();
    let config = load_config()?;

    // Initialize logging
    if config.logging.enable {
        tracing_subscriber::fmt::init();
    }

    // Setup database path (from config or default)
    let db_path = infrastructure::config::get_database_path(&config);
    if let Some(parent) = db_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Initialize AppState
    let db_conn = infrastructure::storage::db::init_database(&db_path).await?;
    let state = AppState::new(db_conn, config.clone())?;

    // Handle commands (flags)
    if cli.update_dict {
        // Use select! to handle shutdown during update
        tokio::select! {
            result = application::update::update_dict(&state) => {
                result?;
            }
            _ = shutdown_rx => {
                eprintln!("更新操作被中断");
                return Ok(());
            }
        }
        return Ok(());
    }
    if cli.generate_config {
        infrastructure::config::generate_config_sample()?;
        return Ok(());
    }
    if cli.edit_config {
        if let Some(config_path) = infrastructure::config::get_config_path() {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            let config_path_clone = config_path.clone();
            // Run editor in blocking task
            tokio::task::spawn_blocking(move || {
                std::process::Command::new(editor)
                    .arg(&config_path_clone)
                    .status()
            })
            .await??;
        } else {
            eprintln!("{}", "Config file not found".red());
        }
        return Ok(());
    }
    if cli.status {
        print_status(&state).await?;
        return Ok(());
    }

    // Handle query
    if cli.query.is_empty() {
        eprintln!("{}", "Please provide a query word".red());
        std::process::exit(1);
    }

    let query = cli.query.join(" ");
    let result = application::query::query_word(&state, &query, cli.nocache, cli.text).await?;

    // Load theme
    let theme_name = cli.theme.as_deref().unwrap_or(config.theme.as_str());
    let theme = presentation::theme::Theme::from_name(theme_name);

    // Output result
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_result(&result, &theme, config.english_only);
    }

    Ok(())
}

/// Check if a query string is English (only contains letters, numbers, spaces, hyphens, dots, question marks)
fn is_english_query(query: &str) -> bool {
    // Match pattern: ^[A-Za-z0-9 -.?]+$
    // Only contains: letters, numbers, spaces, hyphens, dots, question marks
    query
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == ' ' || c == '-' || c == '.' || c == '?')
        && !query.is_empty()
}

/// Filter translations to only English ones (for english_only mode)
fn filter_english_translations(translations: &[String]) -> Vec<String> {
    translations
        .iter()
        .filter(|trans| {
            // Keep translations that start with English (like "n. word" or "v. to do")
            // or are pure English sentences
            let trimmed = trans.trim();
            if trimmed.is_empty() {
                return false;
            }
            // Check if starts with English letter or is a pure English sentence
            trimmed
                .chars()
                .next()
                .map(|c| c.is_ascii_alphabetic())
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}

fn print_result(
    result: &domain::model::QueryResult,
    theme: &presentation::theme::Theme,
    english_only: bool,
) {
    // Check if query is English (used for english_only mode)
    let is_english = is_english_query(&result.query);

    // Query word with source indicator
    let source_indicator = match &result.source {
        domain::model::QuerySource::OfflineDb => "[离线]",
        domain::model::QuerySource::LocalCache => "[缓存]",
        domain::model::QuerySource::Online(_) => "[在线]",
    };
    println!(
        "{} {}",
        (theme.title)(&result.query),
        source_indicator.cyan()
    );

    // Pronunciation (US/UK)
    // In english_only mode, use EN/US instead of 美/英
    if let Some(pron_us) = &result.pronunciation_us {
        let label = if english_only && is_english {
            "US"
        } else {
            "美"
        };
        println!("  {} {}", label.cyan(), (theme.pron)(pron_us));
    }
    if let Some(pron_uk) = &result.pronunciation_uk {
        let label = if english_only && is_english {
            "EN"
        } else {
            "英"
        };
        println!("  {} {}", label.cyan(), (theme.pron)(pron_uk));
    }
    // Fallback to single pronunciation
    if result.pronunciation_us.is_none() && result.pronunciation_uk.is_none() {
        if let Some(pron) = &result.pronunciation {
            println!("  {}", (theme.pron)(pron));
        }
    }

    // Translations - skip Chinese translations in english_only mode for English queries
    if !result.translations.is_empty() {
        let translations_to_show = if english_only && is_english {
            // In english_only mode, only show English translations
            filter_english_translations(&result.translations)
        } else {
            // Show all translations
            result.translations.clone()
        };

        if !translations_to_show.is_empty() {
            println!();
            for trans in &translations_to_show {
                println!("  {}", (theme.para)(trans));
            }
        }
    }

    // Collins rank
    if let Some(rank) = &result.collins_rank {
        println!("  {}", (theme.rank)(rank));
    }

    // Collins dictionary items - format like Go version
    if !result.collins_items.is_empty() {
        println!();
        let cutoff = "⸺".repeat(40);
        println!("  {}", (theme.line)(&cutoff));

        for (i, item) in result.collins_items.iter().enumerate() {
            // Build the item header: number. [additional] major_trans
            // Format like Go version: if starts with [, use as is, otherwise wrap in ()
            let mut item_header = format!("  {}. ", (theme.idx)(&(i + 1).to_string()));

            if let Some(additional) = &item.additional {
                let formatted_additional =
                    if additional.starts_with('[') && additional.ends_with(']') {
                        additional.clone()
                    } else {
                        format!("({})", additional)
                    };
                item_header.push_str(&format!("{} ", (theme.addi)(&formatted_additional)));
            }

            if let Some(major_trans) = &item.major_trans {
                // In english_only mode, extract English part from Collins translation
                // Collins format: "English part 中文翻译" or "English part"
                // Go version uses regex: ^([^\u4e00-\u9fa5]+) ([^ ]*[\u4e00-\u9fa5]+.*)$
                let trans_to_show = if english_only && is_english {
                    // Find the first Chinese character position
                    let mut english_end = 0;
                    for (idx, c) in major_trans.char_indices() {
                        if ('\u{4e00}'..='\u{9fff}').contains(&c) {
                            english_end = idx;
                            break;
                        }
                    }
                    if english_end > 0 {
                        // Found Chinese, extract English part (trim whitespace)
                        major_trans[..english_end].trim().to_string()
                    } else {
                        // No Chinese found, use as is
                        major_trans.clone()
                    }
                } else {
                    major_trans.clone()
                };
                item_header.push_str(&(theme.collins_para)(&trans_to_show));
            }

            println!("{}", item_header);

            // Print examples with ≫ prefix
            // In english_only mode for English queries, only show English part (orig)
            for (orig, trans) in &item.examples {
                if english_only && is_english {
                    // Only show English sentence, skip Chinese translation
                    println!("    ≫   {}", orig);
                } else {
                    let eg_line = format!("≫   {}  {}", orig, (theme.eg)(trans));
                    println!("    {}", eg_line);
                }
            }
        }
    } else if !result.examples.is_empty() {
        // Fallback to simple examples if no Collins items
        println!();
        let cutoff = "⸺".repeat(40);
        println!("  {}", (theme.line)(&cutoff));

        for (i, (orig, trans)) in result.examples.iter().enumerate() {
            if english_only && is_english {
                // Only show English sentence, skip Chinese translation
                println!("  {}. ≫   {}", (theme.idx)(&(i + 1).to_string()), orig);
            } else {
                let eg_line = format!("≫   {}  {}", orig, (theme.eg)(trans));
                println!("  {}. {}", (theme.idx)(&(i + 1).to_string()), eg_line);
            }
        }
    }

    println!();
}

async fn print_status(state: &AppState) -> anyhow::Result<()> {
    println!("{}", "kd Status".green().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Database status
    let config = state.config.read().await;
    let db_path = infrastructure::config::get_database_path(&config);
    drop(config);

    if db_path.exists() {
        let count = sqlite3_count(&state.db).await?;
        println!("Database: {} ({} records)", db_path.display(), count);
    } else {
        println!("Database: Not initialized");
    }

    // Cache status
    println!("Memory Cache: {} entries", state.cache.len());

    // Config status
    let config = state.config.read().await;
    println!(
        "Config: {}",
        infrastructure::config::get_config_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "Not found".to_string())
    );

    if config.youdao.api_id.is_some() {
        println!("Youdao API: Configured");
    } else {
        println!("Youdao API: Not configured");
    }

    Ok(())
}

async fn sqlite3_count(db: &tokio_rusqlite::Connection) -> anyhow::Result<usize> {
    use tokio_rusqlite::params;

    let count: i64 = db
        .call(|conn| conn.query_row("SELECT COUNT(*) FROM cache", params![], |row| row.get(0)))
        .await?;

    Ok(count as usize)
}

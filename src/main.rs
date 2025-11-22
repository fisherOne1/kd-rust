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
        init_logging(&config.logging)?;
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

    // Clear screen if configured
    if config.clear_screen {
        clear_screen();
    }

    // Check frequency alert if configured
    if config.freq_alert {
        check_frequency_alert(&state).await?;
    }

    // Output result
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        let output = format_result(&result, &theme, config.english_only, config.enable_emoji);

        // Use pager if configured
        if config.paging {
            print_with_pager(&output, &config.pager_command)?;
        } else {
            print!("{}", output);
        }
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

/// Clear the terminal screen
fn clear_screen() {
    // ANSI escape sequence: clear screen and move cursor to top-left
    print!("\x1B[2J\x1B[1;1H");
    std::io::Write::flush(&mut std::io::stdout()).ok();
}

/// Initialize logging with path and level configuration
fn init_logging(logging: &infrastructure::config::Logging) -> anyhow::Result<()> {
    use tracing_subscriber::EnvFilter;

    let level = match logging.level.as_str() {
        "DEBUG" => "debug",
        "INFO" => "info",
        "WARN" => "warn",
        "ERROR" => "error",
        _ => "warn",
    };

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    if let Some(path) = &logging.path {
        if !path.is_empty() {
            // Log to file
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_writer(file)
                .init();
            return Ok(());
        }
    }

    // Log to stderr (default)
    tracing_subscriber::fmt().with_env_filter(filter).init();

    Ok(())
}

/// Check query frequency and alert if too high
async fn check_frequency_alert(_state: &AppState) -> anyhow::Result<()> {
    use once_cell::sync::Lazy;
    use std::collections::VecDeque;
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    // Get or create query history in state
    // For simplicity, we'll use a static approach with a mutex
    // In a real implementation, this should be part of AppState
    static QUERY_HISTORY: Lazy<Mutex<VecDeque<Instant>>> =
        Lazy::new(|| Mutex::new(VecDeque::new()));

    let now = Instant::now();
    let mut history = QUERY_HISTORY.lock().unwrap();

    // Remove queries older than 1 minute
    history.retain(|&time| now.duration_since(time) < Duration::from_secs(60));

    // Check if frequency is too high (more than 30 queries per minute)
    if history.len() >= 30 {
        eprintln!("{}", "⚠️  查询频率过高，请稍后再试".yellow());
        return Ok(());
    }

    // Add current query to history
    history.push_back(now);

    Ok(())
}

/// Format result as string (for pager support)
fn format_result(
    result: &domain::model::QueryResult,
    theme: &presentation::theme::Theme,
    english_only: bool,
    enable_emoji: bool,
) -> String {
    use std::fmt::Write;

    let mut output = String::new();
    // Check if query is English (used for english_only mode)
    let is_english = is_english_query(&result.query);

    // Query word with source indicator
    let source_indicator = match &result.source {
        domain::model::QuerySource::OfflineDb => {
            if enable_emoji {
                "📚 [离线]"
            } else {
                "[离线]"
            }
        }
        domain::model::QuerySource::LocalCache => {
            if enable_emoji {
                "💾 [缓存]"
            } else {
                "[缓存]"
            }
        }
        domain::model::QuerySource::Online(_) => {
            if enable_emoji {
                "🌐 [在线]"
            } else {
                "[在线]"
            }
        }
    };
    writeln!(
        output,
        "{} {}",
        (theme.title)(&result.query),
        source_indicator.cyan()
    )
    .ok();

    // Pronunciation (US/UK)
    // In english_only mode, use EN/US instead of 美/英
    if let Some(pron_us) = &result.pronunciation_us {
        let label = if english_only && is_english {
            "US"
        } else {
            "美"
        };
        writeln!(output, "  {} {}", label.cyan(), (theme.pron)(pron_us)).ok();
    }
    if let Some(pron_uk) = &result.pronunciation_uk {
        let label = if english_only && is_english {
            "EN"
        } else {
            "英"
        };
        writeln!(output, "  {} {}", label.cyan(), (theme.pron)(pron_uk)).ok();
    }
    // Fallback to single pronunciation
    if result.pronunciation_us.is_none() && result.pronunciation_uk.is_none() {
        if let Some(pron) = &result.pronunciation {
            writeln!(output, "  {}", (theme.pron)(pron)).ok();
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
            writeln!(output).ok();
            for trans in &translations_to_show {
                writeln!(output, "  {}", (theme.para)(trans)).ok();
            }
        }
    }

    // Collins rank
    if let Some(rank) = &result.collins_rank {
        writeln!(output, "  {}", (theme.rank)(rank)).ok();
    }

    // Collins dictionary items - format like Go version
    if !result.collins_items.is_empty() {
        writeln!(output).ok();
        let cutoff = "⸺".repeat(40);
        writeln!(output, "  {}", (theme.line)(&cutoff)).ok();

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

            writeln!(output, "{}", item_header).ok();

            // Print examples with ≫ prefix
            // In english_only mode for English queries, only show English part (orig)
            let prefix = if enable_emoji { "≫" } else { ">" };
            for (orig, trans) in &item.examples {
                if english_only && is_english {
                    // Only show English sentence, skip Chinese translation
                    writeln!(output, "    {}   {}", prefix, orig).ok();
                } else {
                    let eg_line = format!("{}   {}  {}", prefix, orig, (theme.eg)(trans));
                    writeln!(output, "    {}", eg_line).ok();
                }
            }
        }
    } else if !result.examples.is_empty() {
        // Fallback to simple examples if no Collins items
        writeln!(output).ok();
        let cutoff = "⸺".repeat(40);
        writeln!(output, "  {}", (theme.line)(&cutoff)).ok();

        let prefix = if enable_emoji { "≫" } else { ">" };
        for (i, (orig, trans)) in result.examples.iter().enumerate() {
            if english_only && is_english {
                // Only show English sentence, skip Chinese translation
                writeln!(
                    output,
                    "  {}. {}   {}",
                    (theme.idx)(&(i + 1).to_string()),
                    prefix,
                    orig
                )
                .ok();
            } else {
                let eg_line = format!("{}   {}  {}", prefix, orig, (theme.eg)(trans));
                writeln!(
                    output,
                    "  {}. {}",
                    (theme.idx)(&(i + 1).to_string()),
                    eg_line
                )
                .ok();
            }
        }
    }

    writeln!(output).ok();
    output
}

/// Print output with pager if configured
fn print_with_pager(output: &str, pager_command: &str) -> anyhow::Result<()> {
    use std::process::{Command, Stdio};

    // Parse pager command (e.g., "less -RF" -> ["less", "-RF"])
    let parts: Vec<&str> = pager_command.split_whitespace().collect();
    if parts.is_empty() {
        // Fallback to direct print if no command specified
        print!("{}", output);
        return Ok(());
    }

    let mut cmd = Command::new(parts[0]);
    if parts.len() > 1 {
        cmd.args(&parts[1..]);
    }

    // Set up stdin to receive output
    let mut child = match cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            // If pager command not found, fallback to direct print
            eprintln!(
                "Warning: Pager '{}' not found: {}. Printing directly.",
                parts[0], e
            );
            print!("{}", output);
            return Ok(());
        }
    };

    // Write output to pager's stdin
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(output.as_bytes())?;
        stdin.flush()?;
    }

    // Wait for pager to finish
    child.wait()?;

    Ok(())
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

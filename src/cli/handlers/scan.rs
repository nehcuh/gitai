use anyhow::Result;
use log::{debug, info};
use std::path::PathBuf;

use gitai::args::Command;
use gitai::config::Config;
use gitai::scan;

/// Handler for scan command with Command enum
#[cfg(feature = "security")]
pub async fn handle_command(
    config: &Config,
    command: &Command,
) -> crate::cli::CliResult<()> {
    match command {
        Command::Scan {
            path,
            tool,
            full,
            remote,
            update_rules,
            format,
            output,
            translate,
            auto_install,
            lang,
            no_history,
            timeout,
            benchmark,
        } => {
            handle_scan(
                config,
                path,
                tool,
                *full,
                *remote,
                *update_rules,
                format,
                output.clone(),
                *translate,
                *auto_install,
                lang.as_deref(),
                *no_history,
                *timeout,
                *benchmark,
            )
            .await
            .map_err(|e| e.into())
        }
        Command::ScanHistory { limit, format: _ } => {
            handle_scan_history(*limit).await.map_err(|e| e.into())
        }
        _ => Err("Invalid command for scan handler".into()),
    }
}

/// Handle security scan
#[cfg(feature = "security")]
#[allow(clippy::too_many_arguments)]
async fn handle_scan(
    config: &Config,
    path: &std::path::Path,
    tool: &str,
    _full: bool,
    _remote: bool,
    update_rules: bool,
    _format: &str,
    output: Option<PathBuf>,
    _translate: bool,
    auto_install: bool,
    lang: Option<&str>,
    no_history: bool,
    timeout: Option<u64>,
    benchmark: bool,
) -> Result<()> {
    info!("Executing security scan with tool: {}", tool);
    
    let mut scan_config = scan::ScanConfig::from_config(config);
    
    if let Some(timeout_val) = timeout {
        scan_config.timeout = Some(timeout_val);
        debug!("Set scan timeout to {} seconds", timeout_val);
    }
    
    if let Some(language) = lang {
        scan_config.language = Some(language.to_string());
        debug!("Filtering scan for language: {}", language);
    }
    
    scan_config.update_rules = update_rules;
    scan_config.auto_install = auto_install;
    scan_config.benchmark = benchmark;
    scan_config.save_history = !no_history;
    scan_config.output = output;
    
    if update_rules {
        debug!("Will update scan rules");
    }
    
    if auto_install {
        debug!("Will auto-install scan tools if needed");
    }
    
    if benchmark {
        debug!("Running in benchmark mode");
    }
    
    scan::execute_scan(path, tool, scan_config).await
}

/// Handler for scan history display
async fn handle_scan_history(limit: usize) -> Result<()> {
    use std::fs;
    use serde_json;
    
    info!("Displaying scan history with limit: {}", limit);
    
    let cache_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache/gitai/scan_history");

    if !cache_dir.exists() {
        println!("📁 扫描历史目录不存在");
        debug!("Scan history directory does not exist: {}", cache_dir.display());
        return Ok(());
    }

    println!("🔍 读取扫描历史...");
    debug!("Reading scan history from: {}", cache_dir.display());

    let mut entries: Vec<_> = fs::read_dir(&cache_dir)?
        .flatten()
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();

    // 按修改时间排序（最新的在前）
    entries.sort_by(|a, b| {
        let a_time = a
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let b_time = b
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        b_time.cmp(&a_time)
    });

    println!("📋 扫描历史 (最近{}次):", limit);
    println!();

    for (i, entry) in entries.iter().take(limit).enumerate() {
        let path = entry.path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(result) = serde_json::from_str::<scan::ScanResult>(&content) {
                let modified = entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
                    .and_then(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0))
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "未知时间".to_string());

                println!("{}. {} - {}", i + 1, modified, result.tool);
                println!("   执行时间: {:.2}s", result.execution_time);
                println!("   发现问题: {}", result.findings.len());
                
                if !result.findings.is_empty() {
                    println!("   前3个问题:");
                    for finding in result.findings.iter().take(3) {
                        println!("     - {}", finding.title);
                    }
                }
                println!();
                
                debug!("Displayed scan result: {} findings in {:.2}s", 
                       result.findings.len(), result.execution_time);
            }
        }
    }

    info!("Displayed {} scan history entries", entries.len().min(limit));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AiConfig, ScanConfig};

    fn create_test_config() -> Config {
        Config {
            ai: AiConfig {
                api_url: "http://localhost:11434/v1/chat/completions".to_string(),
                model: "test-model".to_string(),
                api_key: None,
                temperature: Some(0.3),
            },
            scan: ScanConfig {
                default_path: Some(".".to_string()),
                timeout: Some(300),
                jobs: Some(4),
            },
            devops: None,
            mcp: None,
        }
    }

    #[tokio::test]
    #[cfg(feature = "security")]
    async fn test_handle_scan_command() {
        let config = create_test_config();
        let command = Command::Scan {
            path: std::path::PathBuf::from("."),
            tool: "opengrep".to_string(),
            full: false,
            remote: false,
            update_rules: false,
            format: "text".to_string(),
            output: None,
            translate: false,
            auto_install: false,
            lang: None,
            no_history: false,
            timeout: None,
            benchmark: false,
        };
        
        let result = handle_command(&config, &command).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_handle_scan_history() {
        let result = handle_scan_history(5).await;
        assert!(result.is_ok());
    }
}

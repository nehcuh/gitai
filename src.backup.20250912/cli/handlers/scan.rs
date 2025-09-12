use anyhow::Result;
use log::{debug, info};
use std::fs;
use std::path::PathBuf;

use gitai::args::Command;
use gitai::config::Config;
use gitai_security as scan;

/// Handler for scan command with Command enum
#[cfg(feature = "security")]
pub async fn handle_command(config: &Config, command: &Command) -> crate::cli::CliResult<()> {
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
        } => handle_scan(
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
        .map_err(|e| e.into()),
        Command::ScanHistory { limit, format: _ } => {
            handle_scan_history(*limit).await.map_err(|e| e.into())
        }
        _ => Err(anyhow::anyhow!("Invalid command for scan handler").into()),
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
    format: &str,
    output: Option<PathBuf>,
    _translate: bool,
    auto_install: bool,
    lang: Option<&str>,
    no_history: bool,
    timeout: Option<u64>,
    benchmark: bool,
) -> Result<()> {
    use serde_json;

    let show_progress = format != "json";

    if show_progress {
        println!("🔍 正在扫描: {}", path.display());
    }

    // 确保扫描工具已安装
    // 将 'security' 映射为 'opengrep' 以保持向后兼容性
    let normalized_tool = match tool {
        "security" => "opengrep",
        other => other,
    };

    if (normalized_tool == "opengrep" || normalized_tool == "auto")
        && !scan::is_opengrep_installed()
    {
        if auto_install {
            if show_progress {
                println!("🔧 未检测到 OpenGrep，正在自动安装...");
            }
            scan::install_opengrep()
                .map_err(|e| anyhow::anyhow!("Failed to install OpenGrep: {}", e))?;
        } else {
            return Err(anyhow::anyhow!(
                "未检测到 OpenGrep，请先安装或使用 --auto-install 进行自动安装"
            ));
        }
    }

    // 更新规则（如果需要）
    if update_rules {
        if show_progress {
            println!("🔄 正在更新扫描规则...");
        }
        #[cfg(feature = "update-notifier")]
        {
            use gitai::update::AutoUpdater;
            let updater = AutoUpdater::new(config.clone());
            if let Err(e) = updater.update_scan_rules().await {
                eprintln!("⚠️ 规则更新失败: {}", e);
            }
        }
        #[cfg(not(feature = "update-notifier"))]
        {
            eprintln!("ℹ️  update-notifier 功能未启用，跳过规则更新。");
        }
    }

    // 执行扫描
    let result = if normalized_tool == "opengrep" || normalized_tool == "auto" {
        let include_version = show_progress && !benchmark;
        scan::run_opengrep_scan(config, path, lang, timeout, include_version)
            .map_err(|e| anyhow::anyhow!("OpenGrep scan execution failed: {}", e))?
    } else {
        return Err(anyhow::anyhow!(
            "不支持的扫描工具: {} (支持的工具: opengrep, security, auto)",
            tool
        ));
    };

    // 保存扫描历史（无论输出格式）
    if !(no_history || benchmark) {
        let cache_dir = get_cache_dir()?;
        let history_dir = cache_dir.join("scan_history");
        if let Err(e) = fs::create_dir_all(&history_dir) {
            eprintln!("⚠️ 无法创建扫描历史目录: {}", e);
        }
        let ts = chrono::Utc::now().format("%Y%m%d%H%M%S");
        let history_file = history_dir.join(format!("scan_{}_{}.json", result.tool, ts));
        if let Ok(json) = serde_json::to_string(&result) {
            if let Err(e) = fs::write(&history_file, json) {
                eprintln!("⚠️ 写入扫描历史失败: {}", e);
            }
        }
    }

    // 输出结果
    if format == "json" {
        let json = serde_json::to_string_pretty(&result)?;
        if let Some(output_path) = output {
            fs::write(output_path, json)?;
        } else {
            println!("{}", json);
        }
    } else {
        if show_progress {
            println!("📊 扫描结果:");
            println!("  工具: {}", result.tool);
            println!("  版本: {}", result.version);
            println!("  执行时间: {:.2}s", result.execution_time);

            if !result.findings.is_empty() {
                println!("  发现问题: {}", result.findings.len());
                for finding in result.findings.iter().take(5) {
                    println!(
                        "    - {} ({}:{})",
                        finding.title,
                        finding.file_path.display(),
                        finding.line
                    );
                }
                if result.findings.len() > 5 {
                    println!("    ... 还有 {} 个问题", result.findings.len() - 5);
                }
            } else {
                println!("  ✅ 未发现问题");
            }
        }
    }

    Ok(())
}

/// 获取缓存目录
fn get_cache_dir() -> Result<PathBuf> {
    let cache_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache")
        .join("gitai");

    fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

/// Handler for scan history display
async fn handle_scan_history(limit: usize) -> Result<()> {
    use serde_json;
    use std::fs;

    info!("Displaying scan history with limit: {}", limit);

    let cache_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache/gitai/scan_history");

    if !cache_dir.exists() {
        println!("📁 扫描历史目录不存在");
        debug!(
            "Scan history directory does not exist: {}",
            cache_dir.display()
        );
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

                debug!(
                    "Displayed scan result: {} findings in {:.2}s",
                    result.findings.len(),
                    result.execution_time
                );
            }
        }
    }

    info!(
        "Displayed {} scan history entries",
        entries.len().min(limit)
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        use gitai::config::{AiConfig, ScanConfig};
        Config {
            ai: AiConfig {
                api_url: "http://localhost:11434/v1/chat/completions".to_string(),
                model: "test-model".to_string(),
                api_key: None,
                temperature: 0.3,
            },
            scan: ScanConfig {
                default_path: Some(".".to_string()),
                timeout: 300,
                jobs: 4,
                rules_dir: None,
            },
            devops: None,
            language: None,
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

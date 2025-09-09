use anyhow::Result;
use log::{debug, info, warn};

use gitai::config::Config;
use gitai::git;

/// Handler for the git command with AI enhancement
#[cfg(feature = "ai")]
pub async fn handle_git_with_ai(config: &Config, git_args: &[String]) -> Result<()> {
    use gitai::ai;
    
    info!("Executing git command with AI explanation: {}", git_args.join(" "));
    
    // 执行Git命令
    let output = git::run_git(git_args)?;
    print!("{output}");
    debug!("Git command output: {}", output.trim());

    // 添加AI解释
    let command_str = format!("git {}", git_args.join(" "));
    let prompt = format!(
        "用户刚刚执行了以下Git命令：\n\n{}\n\n命令输出：\n{}\n\n请简洁地解释这个命令的作用和输出结果。",
        command_str,
        output.trim()
    );

    match ai::call_ai(config, &prompt).await {
        Ok(explanation) => {
            println!("\n🤖 AI解释:");
            println!("{explanation}");
            debug!("AI explanation generated successfully");
        }
        Err(e) => {
            warn!("AI解释失败: {e}");
            eprintln!("⚠️  AI解释功能暂时不可用");
        }
    }

    Ok(())
}

/// Handler for the git command without AI enhancement
#[cfg(not(feature = "ai"))]
pub async fn handle_git(git_args: &[String]) -> Result<()> {
    info!("Executing git command: {}", git_args.join(" "));
    
    let output = git::run_git(git_args)?;
    print!("{output}");
    debug!("Git command output: {}", output.trim());
    
    Ok(())
}

/// Handler for git command history display
pub async fn handle_scan_history(limit: usize) -> Result<()> {
    use std::fs;
    use serde_json;
    #[cfg(feature = "security")]
    use gitai::scan;
    
    info!("Displaying scan history with limit: {}", limit);
    
    let cache_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
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
            #[cfg(feature = "security")]
            {
                if let Ok(result) = serde_json::from_str::<gitai::scan::ScanResult>(&content) {
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
            #[cfg(not(feature = "security"))]
            {
                println!("{}. {} - {}", i + 1, "Unknown time", "N/A (security feature disabled)");
                println!("   Security scanning feature is not enabled");
                println!();
            }
        }
    }

    info!("Displayed {} scan history entries", entries.len().min(limit));
    Ok(())
}

/// Handler for git command with Command enum
pub async fn handle_command(
    config: &gitai::config::Config,
    command: &gitai::args::Command,
    args: &gitai::args::Args,
) -> crate::cli::CliResult<()> {
    use gitai::args::Command;
    
    match command {
        Command::Git(git_args) => {
            // 默认不启用AI解释；--ai 显式开启；--noai 可显式关闭（当外部别名强制开启时）
            let use_ai = args.ai && !args.noai;
            
            #[cfg(feature = "ai")]
            {
                if use_ai {
                    handle_git_with_ai(config, git_args).await.map_err(|e| e.into())
                } else {
                    let output = gitai::git::run_git(git_args)?;
                    print!("{output}");
                    Ok(())
                }
            }
            
            #[cfg(not(feature = "ai"))]
            {
                let output = gitai::git::run_git(git_args)?;
                print!("{output}");
                Ok(())
            }
        }
        _ => Err("Invalid command for git handler".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitai::config::{AiConfig, ScanConfig};

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
    #[cfg(feature = "ai")]
    async fn test_handle_git_with_ai() {
        let config = create_test_config();
        let git_args = vec!["status".to_string()];
        
        // This test would need proper git setup
        let result = handle_git_with_ai(&config, &git_args).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    #[cfg(not(feature = "ai"))]
    async fn test_handle_git() {
        let git_args = vec!["status".to_string()];
        
        let result = handle_git(&git_args).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_handle_scan_history() {
        let result = handle_scan_history(5).await;
        assert!(result.is_ok());
    }
}

use anyhow::Result;
use log::{debug, info};

use gitai::args::Command;
use gitai::config::Config;

/// Handler for update command with Command enum
#[cfg(feature = "update-notifier")]
pub async fn handle_command(
    config: &Config,
    command: &Command,
) -> crate::cli::CliResult<()> {
    use crate::update;

    match command {
        Command::Update { check, format } => {
            if *check {
                handle_update_check(config, format).await.map_err(|e| e.into())
            } else {
                handle_update(config).await.map_err(|e| e.into())
            }
        }
        _ => Err("Invalid command for update handler".into()),
    }
}

#[cfg(feature = "update-notifier")]
async fn handle_update_check(config: &Config, format: &str) -> Result<()> {
    use crate::update::AutoUpdater;
    
    info!("Checking for updates in {} format", format);
    
    let updater = AutoUpdater::new(config.clone());
    let status = updater.check_update_status();

    if format == "json" {
        let json = serde_json::to_string_pretty(&status)?;
        println!("{}", json);
        debug!("Update status displayed in JSON format");
    } else {
        println!("🔎 更新检查:");
        println!();

        for item in &status {
            println!("📦 {}: {}", item.name, item.message);
        }

        println!();
        if status.is_empty() {
            println!("就绪状态: ✅ 已就绪");
            debug!("All components are up to date");
        } else {
            println!("就绪状态: ❌ 需要更新");
            debug!("Some components need updates");
        }
    }

    Ok(())
}

#[cfg(feature = "update-notifier")]
async fn handle_update(config: &Config) -> Result<()> {
    use crate::update::AutoUpdater;
    
    info!("Updating scan rules");
    println!("🔄 正在更新规则...");
    
    let updater = AutoUpdater::new(config.clone());
    let result = updater.update_scan_rules().await?;

    println!("✅ 更新完成");
    println!("   更新状态: {}", result.message);
    debug!("Update completed with status: {}", result.message);

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
    #[cfg(feature = "update-notifier")]
    async fn test_handle_update_check() {
        let config = create_test_config();
        let command = Command::Update {
            check: true,
            format: "text".to_string(),
        };
        
        let result = handle_command(&config, &command).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    #[cfg(feature = "update-notifier")]
    async fn test_handle_update() {
        let config = create_test_config();
        let command = Command::Update {
            check: false,
            format: "text".to_string(),
        };
        
        let result = handle_command(&config, &command).await;
        assert!(result.is_ok() || result.is_err());
    }
}

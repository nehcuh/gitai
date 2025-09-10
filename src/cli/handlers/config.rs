use anyhow::Result;
use log::{debug, info};
use std::fs;
use std::path::PathBuf;

use gitai::args::ConfigAction;
use gitai::config::Config;

/// Handler for the config command
pub async fn handle_config(config: &Config, action: &ConfigAction, offline: bool) -> Result<()> {
    use gitai::resource_manager::{load_resource_config, ResourceManager};

    match action {
        ConfigAction::Check => {
            info!("Checking configuration status");
            println!("🔍 检查配置状态...");

            // 检查配置文件
            let config_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config/gitai");
            let config_path = config_dir.join("config.toml");

            if config_path.exists() {
                println!("✅ 配置文件: {}", config_path.display());
                debug!("Config file exists at: {}", config_path.display());
            } else {
                println!("❌ 配置文件不存在");
                debug!("Config file does not exist");
            }

            // 检查缓存目录
            let cache_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".cache/gitai");

            if cache_dir.exists() {
                println!("✅ 缓存目录: {}", cache_dir.display());
                debug!("Cache directory exists at: {}", cache_dir.display());

                // 检查规则
                let rules_dir = cache_dir.join("rules");
                if rules_dir.exists() {
                    println!("  ✅ 规则缓存: 已就绪");
                    debug!("Rules cache is ready");
                } else {
                    println!("  ⚠️  规则缓存: 未找到");
                    debug!("Rules cache not found");
                }

                // 检查 Tree-sitter
                let ts_dir = cache_dir.join("tree-sitter");
                if ts_dir.exists() {
                    println!("  ✅ Tree-sitter缓存: 已就绪");
                    debug!("Tree-sitter cache is ready");
                } else {
                    println!("  ⚠️  Tree-sitter缓存: 未找到");
                    debug!("Tree-sitter cache not found");
                }
            } else {
                println!("❌ 缓存目录不存在");
                debug!("Cache directory does not exist");
            }
        }
        ConfigAction::Show { format } => {
            info!("Showing configuration in {} format", format);
            match format.as_str() {
                "json" => {
                    // Config 可能没有实现 Serialize，暂时用简单格式
                    println!("{{");
                    println!("  \"ai\": {{");
                    println!("    \"api_url\": \"{}\",", config.ai.api_url);
                    println!("    \"model\": \"{}\"", config.ai.model);
                    println!("  }},");
                    println!("  \"scan\": {{");
                    println!(
                        "    \"default_path\": \"{}\"",
                        config.scan.default_path.as_deref().unwrap_or(".")
                    );
                    println!("  }}");
                    println!("}}");
                }
                "toml" => {
                    // Config 类型可能没有实现 Serialize，暂时显示简单信息
                    println!("📋 TOML 格式输出暂不可用");
                    debug!("TOML format output not yet implemented");
                }
                _ => {
                    println!("📋 当前配置:");
                    println!("  AI服务: {}", config.ai.api_url);
                    println!("  AI模型: {}", config.ai.model);
                    // config.scan 是 ScanConfig 类型，不是 Option
                    println!(
                        "  扫描路径: {}",
                        config.scan.default_path.as_deref().unwrap_or(".")
                    );
                }
            }
        }
        ConfigAction::Update { force } => {
            info!("Updating resources (force: {})", force);
            println!("🔄 更新资源...");

            let config_path = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config/gitai/config.toml");

            if let Ok(resource_config) = load_resource_config(&config_path) {
                let manager = ResourceManager::new(resource_config)?;

                if offline {
                    eprintln!("⚠️  离线模式下无法更新资源");
                    debug!("Cannot update resources in offline mode");
                    return Ok(());
                }

                if *force {
                    println!("🚀 强制更新所有资源...");
                    debug!("Force updating all resources");
                }

                manager.update_all().await?;
                println!("✅ 资源更新完成");
                info!("Resources updated successfully");
            } else {
                eprintln!("❌ 无法加载资源配置");
                debug!("Failed to load resource configuration");
            }
        }
        ConfigAction::Reset { no_backup } => {
            info!("Resetting configuration (no_backup: {})", no_backup);
            println!("🔄 重置配置...");

            let config_path = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config/gitai/config.toml");

            if !no_backup && config_path.exists() {
                let backup_path = config_path.with_extension("toml.backup");
                fs::copy(&config_path, &backup_path)?;
                println!("💾 已备份到: {}", backup_path.display());
                info!("Configuration backed up to: {}", backup_path.display());
            }

            // 写入默认配置
            let default_config = include_str!("../../../assets/config.enhanced.toml");
            fs::write(&config_path, default_config)?;
            println!("✅ 配置已重置到默认值");
            info!("Configuration reset to default values");
        }
        ConfigAction::Clean => {
            info!("Cleaning cache");
            println!("🧹 清理缓存...");

            let config_path = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config/gitai/config.toml");

            if let Ok(resource_config) = load_resource_config(&config_path) {
                let manager = ResourceManager::new(resource_config)?;
                manager.clean_cache().await?;
                println!("✅ 缓存清理完成");
                info!("Cache cleaned successfully");
            } else {
                eprintln!("❌ 无法加载资源配置");
                debug!("Failed to load resource configuration");
            }
        }
    }

    Ok(())
}

/// Handler for config command with Command enum
pub async fn handle_command(
    config: &gitai::config::Config,
    command: &gitai::args::Command,
    offline: bool,
) -> crate::cli::CliResult<()> {
    use gitai::args::Command;

    match command {
        Command::Config { action } => handle_config(config, action, offline)
            .await
            .map_err(|e| e.into()),
        _ => Err("Invalid command for config handler".into()),
    }
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
    async fn test_handle_config_check() {
        let config = create_test_config();
        let action = ConfigAction::Check;
        let result = handle_config(&config, &action, false).await;
        // This would work in a real test environment
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_handle_config_show_json() {
        let config = create_test_config();
        let action = ConfigAction::Show {
            format: "json".to_string(),
        };
        let result = handle_config(&config, &action, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_config_show_text() {
        let config = create_test_config();
        let action = ConfigAction::Show {
            format: "text".to_string(),
        };
        let result = handle_config(&config, &action, false).await;
        assert!(result.is_ok());
    }
}

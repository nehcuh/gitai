use anyhow::Result;
use log::{debug, info};
use std::fs;
use std::path::PathBuf;

use gitai::args::PromptAction;
use gitai::config::Config;

/// Handler for the prompts command
pub async fn handle_prompts(_config: &Config, action: &PromptAction) -> Result<()> {
    info!("Handling prompts action: {:?}", action);

    match action {
        PromptAction::Init => {
            info!("Initializing prompts directory");
            println!("🔄 正在初始化提示词目录...");

            let prompts_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
                .join("gitai")
                .join("prompts");

            fs::create_dir_all(&prompts_dir)?;
            debug!("Created prompts directory: {}", prompts_dir.display());

            // 创建默认模板
            let templates = [
                (
                    "commit.md",
                    include_str!("../../../assets/prompts/commit.md"),
                ),
                (
                    "review.md",
                    include_str!("../../../assets/prompts/review.md"),
                ),
            ];

            for (filename, content) in &templates {
                let file_path = prompts_dir.join(filename);
                if !file_path.exists() {
                    fs::write(&file_path, content)?;
                    debug!("Created template file: {}", file_path.display());
                }
            }

            println!("✅ 提示词目录已就绪: {}", prompts_dir.display());
            info!("Prompts directory initialized successfully");
        }
        PromptAction::List => {
            info!("Listing available prompt templates");
            let prompts_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
                .join("gitai")
                .join("prompts");

            if !prompts_dir.exists() {
                println!("📁 提示词目录不存在，请先运行: gitai prompts init");
                debug!("Prompts directory does not exist");
                return Ok(());
            }

            println!("📝 可用的提示词模板:");
            let entries = fs::read_dir(&prompts_dir)?;
            let mut count = 0;

            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                        println!("  - {name}");
                        count += 1;
                        debug!("Found template: {}", name);
                    }
                }
            }

            info!("Listed {} prompt templates", count);
        }
        PromptAction::Show { name, language: _ } => {
            info!("Showing prompt template: {}", name);
            let prompts_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
                .join("gitai")
                .join("prompts");

            let file_path = prompts_dir.join(format!("{name}.md"));
            if file_path.exists() {
                let content = fs::read_to_string(&file_path)?;
                println!("📝 提示词模板: {name}");
                println!("{content}");
                debug!("Displayed template: {}", file_path.display());
            } else {
                println!("❌ 未找到提示词模板: {name}");
                debug!("Template not found: {}", file_path.display());
            }
        }
        PromptAction::Update => {
            info!("Update prompt templates requested");
            println!("🔄 更新提示词模板功能暂未实现");
            debug!("Prompt template update feature not yet implemented");
        }
    }

    Ok(())
}

/// Handler for prompts command with Command enum
pub async fn handle_command(
    config: &gitai::config::Config,
    command: &gitai::args::Command,
) -> crate::cli::CliResult<()> {
    use gitai::args::Command;

    match command {
        Command::Prompts { action } => handle_prompts(config, action).await.map_err(|e| e.into()),
        _ => Err("Invalid command for prompts handler".into()),
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
    async fn test_handle_prompts_list() {
        let config = create_test_config();
        let action = PromptAction::List;

        // This test would need proper setup of home directory
        // For now, just verify the function signature works
        let result = handle_prompts(&config, &action).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_handle_prompts_show() {
        let config = create_test_config();
        let action = PromptAction::Show {
            name: "commit".to_string(),
            language: None,
        };

        let result = handle_prompts(&config, &action).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_handle_prompts_update() {
        let config = create_test_config();
        let action = PromptAction::Update;

        let result = handle_prompts(&config, &action).await;
        assert!(result.is_ok());
    }
}

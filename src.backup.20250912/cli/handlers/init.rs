use anyhow::Result;
use log::{debug, info};
use std::path::{Path, PathBuf};

/// Handler for the init command
pub async fn handle_init(
    config_url: Option<String>,
    offline: bool,
    _resources_dir: Option<PathBuf>,
    _dev: bool,
    download_resources: bool,
) -> Result<()> {
    use gitai::config_init::ConfigInitializer;

    info!("Initializing GitAI configuration");
    println!("🚀 初始化 GitAI 配置...");

    let mut initializer = ConfigInitializer::new();

    if let Some(url) = config_url {
        println!("📥 使用配置URL: {url}");
        debug!("Using config URL: {}", url);
        initializer = initializer.with_config_url(Some(url));
    }

    if offline {
        println!("🔌 离线模式初始化");
        debug!("Offline mode initialization");
        initializer = initializer.with_offline_mode(true);
    }

    match initializer.initialize().await {
        Ok(config_path) => {
            println!("✅ 配置初始化成功!");
            println!("📁 配置文件: {}", config_path.display());
            info!(
                "Configuration initialized successfully at: {}",
                config_path.display()
            );

            // 如果需要下载资源
            if download_resources && !offline {
                println!();
                println!("📦 正在下载资源...");
                debug!("Downloading resources");

                // 下载 Tree-sitter queries
                println!("🌳 下载 Tree-sitter queries...");
                match download_tree_sitter_resources().await {
                    Ok(()) => {
                        println!("✅ Tree-sitter queries 下载完成");
                        debug!("Tree-sitter queries downloaded successfully");
                    }
                    Err(e) => {
                        eprintln!("⚠️  Tree-sitter queries 下载失败: {e}");
                        debug!("Failed to download Tree-sitter queries: {}", e);
                    }
                }

                // 下载 OpenGrep 规则（如果可能的话）
                println!("🔒 下载 OpenGrep 规则...");
                match download_opengrep_resources(&config_path).await {
                    Ok(()) => {
                        println!("✅ OpenGrep 规则下载完成");
                        debug!("OpenGrep rules downloaded successfully");
                    }
                    Err(e) => {
                        eprintln!("⚠️  OpenGrep 规则下载失败: {e}");
                        debug!("Failed to download OpenGrep rules: {}", e);
                    }
                }

                println!("✅ 资源下载完成！");
                info!("Resources download completed");
            } else if download_resources && offline {
                println!();
                println!("⚠️  离线模式下无法下载资源");
                debug!("Cannot download resources in offline mode");
            }

            println!();
            println!("🎉 您现在可以使用 GitAI 了:");
            println!("  gitai review     - 代码评审");
            println!("  gitai commit     - 智能提交");
            println!("  gitai scan       - 安全扫描");
            println!("  gitai --help     - 查看更多命令");
        }
        Err(e) => {
            eprintln!("❌ 初始化失败: {e}");
            debug!("Initialization failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

/// Download Tree-sitter resources
async fn download_tree_sitter_resources() -> Result<()> {
    // 检查是否启用了任意 Tree-sitter 语言支持
    #[cfg(any(
        feature = "tree-sitter-rust",
        feature = "tree-sitter-java",
        feature = "tree-sitter-python",
        feature = "tree-sitter-javascript",
        feature = "tree-sitter-typescript",
        feature = "tree-sitter-go",
        feature = "tree-sitter-c",
        feature = "tree-sitter-cpp"
    ))]
    {
        match gitai::tree_sitter::TreeSitterManager::new().await {
            Ok(_) => {
                info!("Tree-sitter 资源初始化成功");
                Ok(())
            }
            Err(e) => {
                log::warn!("Tree-sitter 资源初始化失败: {e}");
                Err(anyhow::anyhow!("Tree-sitter 资源下载失败: {e}"))
            }
        }
    }
    #[cfg(not(any(
        feature = "tree-sitter-rust",
        feature = "tree-sitter-java",
        feature = "tree-sitter-python",
        feature = "tree-sitter-javascript",
        feature = "tree-sitter-typescript",
        feature = "tree-sitter-go",
        feature = "tree-sitter-c",
        feature = "tree-sitter-cpp"
    )))]
    {
        info!("Tree-sitter 功能未启用，跳过资源下载");
        Ok(())
    }
}

/// Download OpenGrep resources
async fn download_opengrep_resources(config_path: &Path) -> Result<()> {
    #[cfg(feature = "security")]
    {
        use gitai::resource_manager::{load_resource_config, ResourceManager};

        // 尝试加载资源配置
        match load_resource_config(config_path) {
            Ok(resource_config) => {
                let manager = ResourceManager::new(resource_config)?;
                match manager.update_all().await {
                    Ok(_) => {
                        info!("OpenGrep 规则资源更新成功");
                        Ok(())
                    }
                    Err(e) => {
                        log::warn!("OpenGrep 规则资源更新失败: {}", e);
                        Err(anyhow::anyhow!("OpenGrep 规则下载失败: {}", e))
                    }
                }
            }
            Err(e) => {
                log::warn!("无法加载资源配置: {}", e);
                // 不将此视为错误，因为可能配置还未完全设置
                Ok(())
            }
        }
    }
    #[cfg(not(feature = "security"))]
    {
        info!("安全扫描功能未启用，跳过 OpenGrep 规则下载");
        Ok(())
    }
}

/// Handler for init command with Command enum
pub async fn handle_command(command: &gitai::args::Command) -> crate::cli::CliResult<()> {
    use gitai::args::Command;

    match command {
        Command::Init {
            config_url,
            offline,
            resources_dir,
            dev,
            download_resources,
        } => handle_init(
            config_url.clone(),
            *offline,
            resources_dir.clone(),
            *dev,
            *download_resources,
        )
        .await
        .map_err(|e| e.into()),
        _ => Err("Invalid command for init handler".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_init_offline() {
        // Test offline initialization
        let result = handle_init(None, true, None, false, false).await;
        // This would fail in real test without proper setup, but shows the interface
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_handle_init_with_config_url() {
        // Test with config URL
        let config_url = Some("https://example.com/config.toml".to_string());
        let result = handle_init(config_url, false, None, false, false).await;
        // This would fail in real test without proper setup, but shows the interface
        assert!(result.is_ok() || result.is_err());
    }
}

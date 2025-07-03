use crate::{
    ast_grep_analyzer::rule_manager::RuleManager,
    config::AppConfig,
    errors::AppError,
    types::git::UpdateRulesArgs,
};
use colored::Colorize;
use std::time::Instant;
use tracing::{error, info, warn};

/// Handle the update-rules command
pub async fn handle_update_rules(
    _config: &AppConfig,
    args: &UpdateRulesArgs,
) -> Result<(), AppError> {
    let start_time = Instant::now();

    info!("🔄 开始更新 AST-Grep 规则");

    // Create rule manager
    let mut rule_manager = RuleManager::new(args.target_dir.as_ref().map(|s| s.into()))
        .map_err(|e| AppError::Analysis(e))?;

    // Handle list sources command
    if args.list_sources {
        return handle_list_sources(&rule_manager).await;
    }

    // Check for updates first
    if !args.force {
        if let Err(e) = check_and_display_updates(&rule_manager, args).await {
            warn!("检查更新失败: {}", e);
        }
    }

    // Perform the update
    match rule_manager.update_rules(args).await {
        Ok(metadata) => {
            let duration = start_time.elapsed();

            println!("✅ {}", "规则更新成功!".bright_green());
            println!("📊 更新统计:");
            println!("  版本: {}", metadata.version.bright_blue());
            println!(
                "  规则数量: {}",
                metadata.rule_count.to_string().bright_yellow()
            );
            println!("  源: {}", metadata.source.bright_cyan());
            println!("  耗时: {:.2}s", duration.as_secs_f64());

            if args.verbose {
                println!("\n📄 更新的文件:");
                for file in &metadata.files {
                    println!("  • {}", file);
                }
            }

            // Show disk usage if verbose
            if args.verbose {
                if let Ok(disk_usage) = rule_manager.get_disk_usage() {
                    println!("💾 磁盘使用: {}", format_bytes(disk_usage));
                }
            }

            info!("规则更新完成: {} 个规则", metadata.rule_count);
        }
        Err(e) => {
            error!("规则更新失败: {}", e);
            eprintln!("❌ {}: {}", "更新失败".bright_red(), e);
            return Err(AppError::Analysis(e));
        }
    }

    // Cleanup old backups if requested
    if args.backup {
        match rule_manager.cleanup_backups(5) {
            Ok(removed) => {
                if removed > 0 {
                    info!("清理了 {} 个旧备份", removed);
                }
            }
            Err(e) => {
                warn!("清理备份失败: {}", e);
            }
        }
    }

    Ok(())
}

/// Handle listing available rule sources
async fn handle_list_sources(rule_manager: &RuleManager) -> Result<(), AppError> {
    println!("📚 {}", "可用的规则源:".bright_blue().bold());
    println!();

    let sources = rule_manager.list_sources();

    if sources.is_empty() {
        println!("  没有配置规则源");
        return Ok(());
    }

    for (name, source) in sources {
        let status = if source.enabled { "✅" } else { "❌" };
        let priority = source.priority;

        println!(
            "{} {} {} (优先级: {})",
            status,
            name.bright_cyan().bold(),
            format!("({})", source.source_type).bright_black(),
            priority.to_string().bright_yellow()
        );

        println!("   📍 位置: {}", source.location);
        println!("   📝 描述: {}", source.description);
        println!("   🔀 引用: {}", source.reference);

        if let Some(last_updated) = source.last_updated {
            let datetime =
                chrono::DateTime::from_timestamp(last_updated as i64, 0).unwrap_or_default();
            println!("   🕒 最后更新: {}", datetime.format("%Y-%m-%d %H:%M:%S"));
        } else {
            println!("   🕒 最后更新: {}", "从未更新".bright_black());
        }

        println!();
    }

    // Show installed sources
    match rule_manager.get_installed_sources() {
        Ok(installed) => {
            if !installed.is_empty() {
                println!("💾 {}", "已安装的规则:".bright_green().bold());
                println!();

                for (source_name, metadata) in installed {
                    println!("📦 {}", source_name.bright_cyan());
                    println!("   版本: {}", metadata.version);
                    println!("   规则数: {}", metadata.rule_count);
                    println!("   校验和: {}", metadata.checksum);

                    let datetime =
                        chrono::DateTime::from_timestamp(metadata.downloaded_at as i64, 0)
                            .unwrap_or_default();
                    println!("   下载时间: {}", datetime.format("%Y-%m-%d %H:%M:%S"));
                    println!();
                }
            }
        }
        Err(e) => {
            warn!("获取已安装规则失败: {}", e);
        }
    }

    Ok(())
}

/// Check and display available updates
async fn check_and_display_updates(
    rule_manager: &RuleManager,
    args: &UpdateRulesArgs,
) -> Result<(), AppError> {
    println!("🔍 {}", "检查更新...".bright_blue());

    let source_name = if args.repository.is_some() {
        Some("custom")
    } else {
        Some("official")
    };

    match rule_manager.check_updates(source_name).await {
        Ok(updates) => {
            let mut has_updates = false;

            for (name, update_info) in updates {
                if update_info.update_available {
                    has_updates = true;

                    println!("📈 {} 有可用更新:", name.bright_cyan());

                    if let Some(current) = &update_info.current_version {
                        println!("   当前版本: {}", current.bright_red());
                    }

                    println!(
                        "   最新版本: {}",
                        update_info.available_version.bright_green()
                    );

                    if let Some(changelog) = &update_info.changelog {
                        println!("   更新说明: {}", changelog);
                    }

                    if let Some(size) = update_info.download_size {
                        println!("   下载大小: {}", format_bytes(size));
                    }

                    println!();
                } else if args.verbose {
                    println!("✅ {} 已是最新版本", name.bright_green());
                }
            }

            if !has_updates {
                println!("✅ {}", "所有规则都是最新版本".bright_green());
            }
        }
        Err(e) => {
            return Err(AppError::Analysis(e));
        }
    }

    Ok(())
}

/// Format bytes into human readable format
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Display update progress (placeholder for future implementation)
pub fn display_update_progress(current: usize, total: usize, message: &str) {
    let percentage = if total > 0 {
        (current as f64 / total as f64 * 100.0) as u8
    } else {
        0
    };

    let bar_length = 40;
    let filled_length = (bar_length as f64 * current as f64 / total as f64) as usize;
    let bar = "=".repeat(filled_length) + &"-".repeat(bar_length - filled_length);

    print!("\r[{}] {}% - {}", bar, percentage, message);

    if current >= total {
        println!(); // New line when complete
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::git::UpdateRulesArgs;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
        assert_eq!(format_bytes(1073741824), "1.0 GB");
    }

    #[tokio::test]
    async fn test_handle_list_sources() {
        let rule_manager = RuleManager::new(None).expect("Failed to create rule manager");
        let result = handle_list_sources(&rule_manager).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_update_progress() {
        // Test that progress display doesn't panic
        display_update_progress(50, 100, "Testing progress");
        display_update_progress(100, 100, "Complete");
    }

    #[tokio::test]
    async fn test_update_rules_args_defaults() {
        let args = UpdateRulesArgs {
            source: "github".to_string(),
            repository: None,
            reference: "main".to_string(),
            target_dir: None,
            force: false,
            backup: false,
            verify: false,
            list_sources: false,
            verbose: false,
        };

        assert_eq!(args.source, "github");
        assert_eq!(args.reference, "main");
        assert!(!args.force);
        assert!(!args.backup);
    }
}

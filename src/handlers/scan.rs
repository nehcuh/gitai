use crate::config::AppConfig;
use crate::errors::AppError;
use crate::rule_manager::RuleManager;
use crate::scanner::LocalScanner;
use crate::types::git::ScanArgs;
use colored::Colorize;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn handle_scan(
    config: &AppConfig,
    args: ScanArgs,
    language: Option<&str>,
) -> Result<(), AppError> {
    println!("Starting scan...");
    
    // Initialize rule manager and get rules
    let mut rule_manager = RuleManager::new(config.scan.rule_manager.clone())?;
    let rule_paths = get_language_specific_rules(&mut rule_manager, &args, language, config).await?;
    
    if rule_paths.is_empty() {
        println!("No scan rules found. Make sure rules are available or run with --update-rules");
        return Ok(());
    }
    
    println!("Found {} rule files.", rule_paths.len());
    
    // Initialize scanner
    let mut scanner = LocalScanner::new(config.clone())?;
    
    // Perform scan
    println!("Scanning {} files...", if args.full { "all" } else { "changed" });
    let scan_result = scanner.scan(&args, &rule_paths).await?;
    
    // Display results summary
    println!("\n=== Scan Results ===");
    println!("Repository: {}", scan_result.repository);
    println!("Commit ID: {}", scan_result.commit_id);
    println!("Scan Type: {}", scan_result.scan_type);
    println!("Files Scanned: {}", scan_result.files_scanned);
    println!("Rules Applied: {}", scan_result.rules_count);
    println!("Total Matches: {}", scan_result.summary.total_matches);
    
    if !scan_result.summary.by_severity.is_empty() {
        println!("\nBy Severity:");
        for (severity, count) in &scan_result.summary.by_severity {
            println!("  {}: {}", severity, count);
        }
    }
    
    if scan_result.summary.total_matches > 0 {
        println!("\n=== Match Details ===");
        for (i, match_item) in scan_result.matches.iter().enumerate() {
            if i >= 10 {
                println!("... and {} more matches", scan_result.matches.len() - 10);
                break;
            }
            println!("{}. {} ({}:{})", 
                     i + 1,
                     match_item.message,
                     match_item.file_path,
                     match_item.line_number);
            println!("   Rule: {} | Severity: {}", 
                     match_item.rule_id, 
                     match_item.severity);
            println!("   Match: {}", match_item.matched_text.trim());
            println!();
        }
    }
    
    // Save results
    scanner.save_results(&scan_result, args.output.as_deref())?;
    
    println!("Scan completed successfully!");
    Ok(())
}

/// Get language-specific rules based on the language parameter
async fn get_language_specific_rules(
    rule_manager: &mut RuleManager,
    args: &ScanArgs,
    language: Option<&str>,
    config: &AppConfig,
) -> Result<Vec<PathBuf>, AppError> {
    // Get effective language
    let effective_language = match language {
        Some(lang) => {
            let lang_str = lang.to_string();
            config.get_output_language(Some(&lang_str))
        }
        None => config.get_output_language(None)
    };
    
    // If language-specific language is specified, try to use translated rules
    if effective_language == "cn" || effective_language == "us" {
        match find_translated_rules(rule_manager, args, &effective_language).await {
            Ok(translated_paths) if !translated_paths.is_empty() => {
                let language_name = if effective_language == "cn" { "中文" } else { "英文" };
                println!("🌐 使用{}翻译的扫描规则 ({}个规则文件)", language_name, translated_paths.len());
                return Ok(translated_paths);
            }
            Ok(_) => {
                let language_name = if effective_language == "cn" { "中文" } else { "英文" };
                println!("🌐 未找到{}翻译规则，使用默认规则", language_name);
            }
            Err(e) => {
                let language_name = if effective_language == "cn" { "中文" } else { "英文" };
                tracing::warn!("查找{}翻译规则时出错: {}，使用默认规则", language_name, e);
            }
        }
    }
    
    // Fall back to default rules
    let default_paths = rule_manager.get_rule_paths(args.update_rules).await?;
    println!("📋 使用默认扫描规则 ({}个规则文件)", default_paths.len());
    Ok(default_paths)
}

/// Find translated rules in the language-specific directory
async fn find_translated_rules(
    rule_manager: &mut RuleManager,
    args: &ScanArgs,
    language: &str,
) -> Result<Vec<PathBuf>, AppError> {
    // First ensure we have the latest rules
    let default_rule_paths = rule_manager.get_rule_paths(args.update_rules).await?;
    
    if default_rule_paths.is_empty() {
        return Ok(Vec::new());
    }
    
    // Get the rules directory from the first rule path
    let first_rule_path = &default_rule_paths[0];
    let rules_base_dir = first_rule_path
        .parent()
        .and_then(|p| p.parent()) // Go up from rules/language/ to rules/
        .ok_or_else(|| AppError::Generic("无法确定规则基础目录".to_string()))?;
    
    // Look for language-specific translated directory
    // For "cn" language, look for "cn" directory, for "us" look for "us", etc.
    let translated_dir = rules_base_dir.join(language);
    
    if !translated_dir.exists() {
        tracing::debug!("{}语言翻译目录不存在: {:?}", language, translated_dir);
        return Ok(Vec::new());
    }
    
    // Scan for YAML files in the translated directory
    let mut translated_paths = Vec::new();
    scan_translated_directory(&translated_dir, &mut translated_paths)?;
    
    if translated_paths.is_empty() {
        tracing::debug!("{}语言翻译目录中未找到规则文件: {:?}", language, translated_dir);
    } else {
        tracing::info!("在{}语言翻译目录中找到{}个规则文件", language, translated_paths.len());
    }
    
    Ok(translated_paths)
}

/// Recursively scan translated directory for YAML rule files
fn scan_translated_directory(dir: &PathBuf, paths: &mut Vec<PathBuf>) -> Result<(), AppError> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| AppError::Generic(format!("读取翻译目录失败 {:?}: {}", dir, e)))?;
    
    for entry in entries {
        let entry = entry
            .map_err(|e| AppError::Generic(format!("读取目录项失败: {}", e)))?;
        let path = entry.path();
        
        if path.is_dir() {
            // Recursively scan subdirectories
            scan_translated_directory(&path, paths)?;
        } else if let Some(extension) = path.extension() {
            if extension == "yml" || extension == "yaml" {
                paths.push(path);
            }
        }
    }
    
    Ok(())
}

/// Handle scan rules update command
pub async fn handle_update_scan_rules(config: &AppConfig) -> Result<(), AppError> {
    println!("{}", "🔄 开始更新代码扫描规则...".blue());
    
    // Initialize rule manager
    let mut rule_manager = RuleManager::new(config.scan.rule_manager.clone())
        .map_err(|e| AppError::Generic(format!("规则管理器初始化失败: {}", e)))?;
    
    // Check current version info
    if let Some(version_info) = rule_manager.get_version_info() {
        println!("{}", format!("📋 当前规则版本: {}", version_info.commit_hash).cyan());
        println!("{}", format!("📅 最后更新时间: {}", format_system_time(&version_info.last_updated)).cyan());
        println!("{}", format!("📊 规则数量: {}", version_info.rule_count).cyan());
    } else {
        println!("{}", "📋 尚未安装扫描规则".yellow());
    }
    
    // Force update rules
    println!("{}", "🚀 强制更新规则...".yellow());
    rule_manager.force_update().await
        .map_err(|e| AppError::Generic(format!("更新规则失败: {}", e)))?;
    
    // Get updated rule paths to verify
    let rule_paths = rule_manager.get_rule_paths(false).await
        .map_err(|e| AppError::Generic(format!("获取更新后规则路径失败: {}", e)))?;
    
    println!("{}", format!("✅ 规则更新完成！共找到 {} 个规则文件", rule_paths.len()).green());
    
    // Display updated version info
    if let Some(version_info) = rule_manager.get_version_info() {
        println!("{}", "📋 更新后版本信息:".cyan());
        println!("{}", format!("  - 版本: {}", version_info.commit_hash).cyan());
        println!("{}", format!("  - 更新时间: {}", format_system_time(&version_info.last_updated)).cyan());
        println!("{}", format!("  - 规则数量: {}", version_info.rule_count).cyan());
        if let Some(performance) = &version_info.performance_metrics {
            println!("{}", format!("  - 平均执行时间: {:.2}ms", performance.avg_execution_time_ms).cyan());
        }
    }
    
    Ok(())
}

/// Helper function to format SystemTime for display
fn format_system_time(system_time: &SystemTime) -> String {
    match system_time.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let timestamp = duration.as_secs();
            // Simple formatting - you could use chrono for more sophisticated formatting
            let datetime = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp);
            format!("{:?}", datetime)
        }
        Err(_) => "无效时间".to_string()
    }
}

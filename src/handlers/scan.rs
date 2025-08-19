use crate::config::AppConfig;
use crate::errors::AppError;
use crate::rule_manager::RuleManager;
use crate::scanner::LocalScanner;
use crate::types::git::ScanArgs;
use std::path::PathBuf;

pub async fn handle_scan(
    config: &AppConfig,
    args: ScanArgs,
) -> Result<(), AppError> {
    let _result = handle_scan_with_output(config, args).await?;
    Ok(())
}

pub async fn handle_scan_with_output(
    config: &AppConfig,
    args: ScanArgs,
) -> Result<String, AppError> {
    let mut output = String::new();
    output.push_str("🔍 开始代码扫描...\n\n");
    
    // Initialize rule manager and get rules
    let mut rule_manager = RuleManager::new(config.scan.rule_manager.clone())?;
    let rule_paths = get_language_specific_rules(&mut rule_manager, &args, config).await?;
    
    if rule_paths.is_empty() {
        output.push_str("❌ 未找到扫描规则。请确保规则可用或使用 --update-rules 参数\n");
        return Ok(output);
    }
    
    output.push_str(&format!("📋 找到 {} 个规则文件\n", rule_paths.len()));
    
    // Clone config for scanner initialization
    let scanner_config = config.clone();
    
    // Perform scan in a separate scope to avoid Send issues
    let scan_result = {
        // Initialize scanner
        let mut scanner = LocalScanner::new(scanner_config)?;
        
        // Perform scan
        output.push_str(&format!("🔍 扫描{}文件...\n", if args.full { "所有" } else { "变更的" }));
        scanner.scan(&args, &rule_paths).await?
    };
    
    // Display results summary
    output.push_str("\n=== 扫描结果 ===\n");
    output.push_str(&format!("📁 仓库: {}\n", scan_result.repository));
    output.push_str(&format!("🔖 提交ID: {}\n", scan_result.commit_id));
    output.push_str(&format!("📊 扫描类型: {}\n", scan_result.scan_type));
    output.push_str(&format!("📄 扫描文件数: {}\n", scan_result.files_scanned));
    output.push_str(&format!("📋 应用规则数: {}\n", scan_result.rules_count));
    output.push_str(&format!("🎯 匹配总数: {}\n", scan_result.summary.total_matches));
    
    if !scan_result.summary.by_severity.is_empty() {
        output.push_str("\n📊 按严重性分类:\n");
        for (severity, count) in &scan_result.summary.by_severity {
            output.push_str(&format!("  {}: {}\n", severity, count));
        }
    }
    
    if scan_result.summary.total_matches > 0 {
        output.push_str("\n=== 匹配详情 ===\n");
        
        for (i, match_item) in scan_result.matches.iter().enumerate() {
            if i >= 10 {
                let remaining_text = format!("... 还有 {} 个匹配项", scan_result.matches.len() - 10);
                output.push_str(&format!("{}\n", remaining_text));
                break;
            }
            let match_line = format!("{}. {} ({}:{})", 
                     i + 1,
                     match_item.message,
                     match_item.file_path,
                     match_item.line_number);
            let rule_line = format!("   规则: {} | 严重性: {}", 
                     match_item.rule_id, 
                     match_item.severity);
            let match_text_line = format!("   匹配: {}", match_item.matched_text.trim());
            
            output.push_str(&format!("{}\n{}\n{}\n\n", match_line, rule_line, match_text_line));
        }
        
    }
    
    // Save results (create a new scanner instance for saving)
    {
        let save_scanner = LocalScanner::new(config.clone())?;
        save_scanner.save_results(&scan_result, args.output.as_deref())?;
    }
    
    output.push_str("\n✅ 扫描完成！\n");
    Ok(output)
}

/// Update scan rules
pub async fn handle_update_scan_rules(
    config: &AppConfig,
) -> Result<(), AppError> {
    tracing::info!("🔄 更新代码扫描规则");
    
    let mut rule_manager = RuleManager::new(config.scan.rule_manager.clone())?;
    rule_manager.force_update().await?;
    
    println!("✅ 扫描规则更新完成");
    Ok(())
}

/// Get rules for scanning
async fn get_language_specific_rules(
    rule_manager: &mut RuleManager,
    args: &ScanArgs,
    _config: &AppConfig,
) -> Result<Vec<PathBuf>, AppError> {
    // Get default rules
    let default_paths = rule_manager.get_rule_paths(args.update_rules).await?;
    println!("📋 使用默认扫描规则 ({}个规则文件)", default_paths.len());
    Ok(default_paths)
}


use crate::config::AppConfig;
use crate::errors::AppError;
use crate::rule_manager::RuleManager;
use crate::scanner::LocalScanner;
use crate::types::git::ScanArgs;
use crate::handlers::ai::execute_translation_request;
use colored::Colorize;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn handle_scan(
    config: &AppConfig,
    args: ScanArgs,
    language: Option<&str>,
) -> Result<(), AppError> {
    let _result = handle_scan_with_output(config, args, language).await?;
    Ok(())
}

pub async fn handle_scan_with_output(
    config: &AppConfig,
    args: ScanArgs,
    language: Option<&str>,
) -> Result<String, AppError> {
    let mut output = String::new();
    output.push_str("🔍 开始代码扫描...\n\n");
    
    // Initialize rule manager and get rules
    let mut rule_manager = RuleManager::new(config.scan.rule_manager.clone())?;
    let rule_paths = get_language_specific_rules(&mut rule_manager, &args, language, config).await?;
    
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
        
        // Check if AI translation is needed
        let effective_language = match language {
            Some(lang) => {
                let lang_str = lang.to_string();
                config.get_output_language(Some(&lang_str))
            }
            None => config.get_output_language(None)
        };
        
        // If AI translation is explicitly enabled, and specified language doesn't have cached translated rules, use AI to translate output
        if args.translate && 
           (effective_language == "cn" || effective_language == "us") && 
           !has_translated_rules_cache(&mut rule_manager, &effective_language).await {
            
            output.push_str("\n🤖 使用 AI 翻译扫描结果...\n");
            match execute_translation_request(config, &output, &effective_language).await {
                Ok(translated_content) => {
                    output.push_str("\n=== 翻译结果 ===\n");
                    output.push_str(&translated_content);
                }
                Err(e) => {
                    tracing::warn!("AI translation failed: {}", e);
                    output.push_str("⚠️ AI 翻译失败，显示原始结果\n");
                }
            }
        }
    }
    
    // Save results (create a new scanner instance for saving)
    {
        let mut save_scanner = LocalScanner::new(config.clone())?;
        save_scanner.save_results(&scan_result, args.output.as_deref())?;
    }
    
    output.push_str("\n✅ 扫描完成！\n");
    Ok(output)
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

/// Check if translated rules cache exists for the specified language
async fn has_translated_rules_cache(rule_manager: &mut RuleManager, language: &str) -> bool {
    // Try to find any rule paths for the language
    // This is a simplified check - in a more sophisticated implementation,
    // you might want to check specific translated rule files
    let default_paths = match rule_manager.get_rule_paths(false).await {
        Ok(paths) => paths,
        Err(_) => return false,
    };
    
    if default_paths.is_empty() {
        return false;
    }
    
    // Get the rules directory from the first rule path
    let rules_base_dir = match default_paths[0]
        .parent()
        .and_then(|p| p.parent()) {
        Some(dir) => dir,
        None => return false,
    };
    
    // Look for language-specific translated directory
    let translated_dir = rules_base_dir.join(language);
    translated_dir.exists() && translated_dir.is_dir()
}

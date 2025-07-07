use crate::config::AppConfig;
use crate::errors::AppError;
use crate::rule_manager::RuleManager;
use crate::scanner::LocalScanner;
use crate::types::git::ScanArgs;
use std::path::PathBuf;

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
    
    // If Chinese language is specified, try to use translated rules
    if effective_language == "cn" {
        match find_translated_rules(rule_manager, args).await {
            Ok(translated_paths) if !translated_paths.is_empty() => {
                println!("🌐 使用中文翻译的扫描规则 ({}个规则文件)", translated_paths.len());
                return Ok(translated_paths);
            }
            Ok(_) => {
                println!("🌐 未找到中文翻译规则，使用默认规则");
            }
            Err(e) => {
                tracing::warn!("查找中文翻译规则时出错: {}，使用默认规则", e);
            }
        }
    }
    
    // Fall back to default rules
    let default_paths = rule_manager.get_rule_paths(args.update_rules).await?;
    println!("📋 使用默认扫描规则 ({}个规则文件)", default_paths.len());
    Ok(default_paths)
}

/// Find translated rules in the translated directory
async fn find_translated_rules(
    rule_manager: &mut RuleManager,
    args: &ScanArgs,
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
    
    // Look for translated directory
    let translated_dir = rules_base_dir.join("translated");
    
    if !translated_dir.exists() {
        tracing::debug!("翻译目录不存在: {:?}", translated_dir);
        return Ok(Vec::new());
    }
    
    // Scan for YAML files in the translated directory
    let mut translated_paths = Vec::new();
    scan_translated_directory(&translated_dir, &mut translated_paths)?;
    
    if translated_paths.is_empty() {
        tracing::debug!("翻译目录中未找到规则文件: {:?}", translated_dir);
    } else {
        tracing::info!("在翻译目录中找到{}个规则文件", translated_paths.len());
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

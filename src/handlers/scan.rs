use crate::config::AppConfig;
use crate::errors::AppError;
use crate::rule_manager::RuleManager;
use crate::scanner::LocalScanner;
use crate::types::git::ScanArgs;

pub async fn handle_scan(
    config: &AppConfig,
    args: ScanArgs,
) -> Result<(), AppError> {
    println!("Starting scan...");
    
    // Initialize rule manager and get rules
    let rule_manager = RuleManager::new(config.scan.rule_manager.clone())?;
    let rule_paths = rule_manager.get_rule_paths(args.update_rules).await?;
    
    if rule_paths.is_empty() {
        println!("No scan rules found. Make sure rules are available or run with --update-rules");
        return Ok(());
    }
    
    println!("Found {} rule files.", rule_paths.len());
    
    // Initialize scanner
    let scanner = LocalScanner::new(config.clone())?;
    
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

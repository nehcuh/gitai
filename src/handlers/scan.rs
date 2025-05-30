use crate::config::AppConfig;
use crate::errors::AppError;
use crate::types::general::ScanArgs;
use crate::tree_sitter_analyzer::security::{
    TreeSitterSecurityScanner, SecurityScanResults, SecuritySeverity, SecurityFinding
};
use crate::tree_sitter_analyzer::core::create_language_registry;
use crate::handlers::ai::execute_ai_request_generic;
use crate::types::ai::ChatMessage;
use colored::*;
use serde_json::Value;
use tokio::process::Command as AsyncCommand;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use chrono;
use serde::{Deserialize, Serialize};

/// Semgrep finding structure
#[derive(Debug, Deserialize, Serialize)]
struct SemgrepFinding {
    check_id: String,
    path: String,
    start: SemgrepPosition,
    end: SemgrepPosition,
    extra: SemgrepExtra,
}

#[derive(Debug, Deserialize, Serialize)]
struct SemgrepPosition {
    line: u32,
    col: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct SemgrepExtra {
    message: String,
    severity: String,
    metadata: Option<HashMap<String, Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SemgrepResults {
    results: Vec<SemgrepFinding>,
}

/// Handle the scan command using Tree-sitter AST analysis with Semgrep integration
pub async fn handle_scan(config: &AppConfig, args: ScanArgs) -> Result<(), AppError> {
    println!("{}", "🔍 Starting security scan with Tree-sitter and Semgrep...".cyan().bold());

    let scan_config = &config.scan;
    
    // Collect files to scan
    let scan_path = args.path.as_deref().unwrap_or(".");
    let files = collect_source_files(scan_path, &args.exclude)?;
    
    if files.is_empty() {
        println!("{}", "⚠️  No source files found to scan".yellow());
        return Ok(());
    }

    let mut combined_results = SecurityScanResults::new();

    // Perform Tree-sitter security scan if enabled
    if scan_config.treesitter_enabled {
        println!("📁 Scanning {} files with Tree-sitter AST analysis...", files.len());
        
        let language_registry = create_language_registry();
        let scanner = TreeSitterSecurityScanner::new(language_registry);
        
        let tree_sitter_results = scanner.scan_files(files.clone())
            .map_err(|e| AppError::Tool(format!("Tree-sitter scan failed: {}", e)))?;
        
        combined_results = tree_sitter_results;
    }

    // Run Semgrep scan if enabled
    if scan_config.semgrep_enabled {
        println!("🔍 Running Semgrep analysis...");
        let semgrep_results = run_semgrep_scan_with_config(scan_path, scan_config).await?;
        merge_semgrep_results(&mut combined_results, semgrep_results);
    }

    // Display combined results
    display_tree_sitter_results(&combined_results, &args).await?;

    // AI analysis if requested
    if args.ai_analysis {
        analyze_scan_with_ai(config, &combined_results).await?;
    }

    // Save results if auto_save is enabled or output is specified
    if scan_config.auto_save || args.output.is_some() {
        let output_file = generate_output_filename(&args, scan_config, scan_path)?;
        let output_format = if args.format.is_empty() { 
            &scan_config.output_format 
        } else { 
            &args.format 
        };
        
        let output_content = if output_format == "json" {
            combined_results.to_json()
                .map_err(|e| AppError::Tool(format!("Failed to serialize results to JSON: {}", e)))?
        } else {
            combined_results.to_markdown()
        };

        // Ensure output directory exists
        if let Some(parent) = Path::new(&output_file).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::Tool(format!("Failed to create output directory: {}", e)))?;
        }

        fs::write(&output_file, &output_content)
            .map_err(|e| AppError::Tool(format!("Failed to write output file: {}", e)))?;
        
        println!("{} {} ({})", "📄 Results saved to:".green(), output_file, output_format.to_uppercase());
    }

    Ok(())
}

/// Collect source files for scanning
fn collect_source_files(scan_path: &str, exclude_patterns: &Option<Vec<String>>) -> Result<Vec<(PathBuf, String, String)>, AppError> {
    let mut files = Vec::new();
    let path = Path::new(scan_path);

    if path.is_file() {
        if let Some((content, language)) = read_and_detect_language(path)? {
            files.push((path.to_path_buf(), content, language));
        }
    } else if path.is_dir() {
        collect_files_recursive(path, &mut files, exclude_patterns)?;
    }

    Ok(files)
}

/// Recursively collect source files from directory
fn collect_files_recursive(
    dir: &Path, 
    files: &mut Vec<(PathBuf, String, String)>, 
    exclude_patterns: &Option<Vec<String>>
) -> Result<(), AppError> {
    let entries = fs::read_dir(dir)
        .map_err(|e| AppError::Tool(format!("Failed to read directory {}: {}", dir.display(), e)))?;

    for entry in entries {
        let entry = entry.map_err(|e| AppError::Tool(format!("Failed to read directory entry: {}", e)))?;
        let path = entry.path();

        // Skip excluded patterns
        if let Some(patterns) = exclude_patterns {
            let path_str = path.to_string_lossy();
            if patterns.iter().any(|pattern| path_str.contains(pattern)) {
                continue;
            }
        }

        // Skip hidden files and common non-source directories
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') || 
               name == "node_modules" || 
               name == "target" || 
               name == "build" || 
               name == "dist" ||
               name == "__pycache__" {
                continue;
            }
        }

        if path.is_dir() {
            collect_files_recursive(&path, files, exclude_patterns)?;
        } else if let Some((content, language)) = read_and_detect_language(&path)? {
            files.push((path, content, language));
        }
    }

    Ok(())
}

/// Read file and detect programming language
fn read_and_detect_language(path: &Path) -> Result<Option<(String, String)>, AppError> {
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    let language = match extension {
        "rs" => "rust",
        "py" => "python",
        "js" | "jsx" | "ts" | "tsx" => "js", // Use "js" to match language registry
        "java" => "java",
        "cpp" | "cc" | "cxx" => "cpp",
        "c" => "c",
        "go" => "go",
        _ => return Ok(None), // Skip unsupported file types
    };

    let content = fs::read_to_string(path)
        .map_err(|e| AppError::Tool(format!("Failed to read file {}: {}", path.display(), e)))?;

    Ok(Some((content, language.to_string())))
}

/// Display Tree-sitter scan results
async fn display_tree_sitter_results(results: &SecurityScanResults, args: &ScanArgs) -> Result<(), AppError> {
    println!("\n{}", "📊 Security Scan Results".cyan().bold());
    println!("{}", "=".repeat(50).cyan());

    if results.findings.is_empty() {
        println!("{}", "✅ No security vulnerabilities found!".green().bold());
        return Ok(());
    }

    // Display summary
    let summary = &results.summary;
    println!("\n{}", "Summary:".yellow().bold());
    println!("  📁 Files scanned: {}", summary.total_files_scanned);
    println!("  🔍 Total findings: {}", summary.total_findings);
    
    if summary.critical_count > 0 {
        println!("  {} Critical: {}", "🔴".red(), summary.critical_count.to_string().red().bold());
    }
    if summary.high_count > 0 {
        println!("  {} High: {}", "🟠".yellow(), summary.high_count.to_string().yellow().bold());
    }
    if summary.medium_count > 0 {
        println!("  {} Medium: {}", "🟡".yellow(), summary.medium_count.to_string().yellow());
    }
    if summary.low_count > 0 {
        println!("  {} Low: {}", "🔵".blue(), summary.low_count.to_string().blue());
    }
    if summary.info_count > 0 {
        println!("  {} Info: {}", "⚪".white(), summary.info_count.to_string().white());
    }

    // Filter by severity if specified
    let filtered_findings: Vec<&SecurityFinding> = if let Some(severity_filter) = &args.severity {
        let min_severity = match severity_filter.to_uppercase().as_str() {
            "CRITICAL" => SecuritySeverity::Critical,
            "HIGH" => SecuritySeverity::High,
            "MEDIUM" => SecuritySeverity::Medium,
            "LOW" => SecuritySeverity::Low,
            _ => SecuritySeverity::Info,
        };
        results.filter_by_severity(min_severity)
    } else {
        results.findings.iter().collect()
    };

    // Display detailed findings if requested
    if args.detailed {
        display_detailed_tree_sitter_findings(&filtered_findings, args.show_low_severity);
    }

    Ok(())
}

/// Display detailed Tree-sitter findings
fn display_detailed_tree_sitter_findings(findings: &[&SecurityFinding], show_low: bool) {
    // Group by severity
    let mut by_severity: HashMap<SecuritySeverity, Vec<&SecurityFinding>> = HashMap::new();
    for finding in findings {
        by_severity.entry(finding.severity).or_insert_with(Vec::new).push(finding);
    }

    // Display in severity order
    let severities = if show_low {
        vec![SecuritySeverity::Critical, SecuritySeverity::High, SecuritySeverity::Medium, SecuritySeverity::Low, SecuritySeverity::Info]
    } else {
        vec![SecuritySeverity::Critical, SecuritySeverity::High, SecuritySeverity::Medium]
    };

    for severity in severities {
        if let Some(findings) = by_severity.get(&severity) {
            if findings.is_empty() {
                continue;
            }

            let (icon, _color) = match severity {
                SecuritySeverity::Critical => ("🔴", "red"),
                SecuritySeverity::High => ("🟠", "yellow"),
                SecuritySeverity::Medium => ("🟡", "yellow"),
                SecuritySeverity::Low => ("🔵", "blue"),
                SecuritySeverity::Info => ("⚪", "white"),
            };

            println!("\n{} {} {} Issues:", icon, severity, findings.len());
            println!("{}", "-".repeat(40));

            for (i, finding) in findings.iter().enumerate() {
                println!("\n{}. {} ({}:{})", 
                    i + 1, 
                    finding.title.yellow().bold(), 
                    finding.file_path.display().to_string().cyan(), 
                    finding.line_start
                );
                println!("   {}", finding.description);
                
                if let Some(cwe) = &finding.cwe_id {
                    println!("   {} {}", "CWE:".green().bold(), cwe);
                }
                
                if let Some(owasp) = &finding.owasp_category {
                    println!("   {} {}", "OWASP:".green().bold(), owasp);
                }
                
                println!("   {} {}", "Recommendation:".green().bold(), finding.recommendation);
                
                if !finding.code_snippet.is_empty() {
                    println!("   {} {}", "Code:".blue().bold(), finding.code_snippet.trim());
                }
            }
        }
    }
}

/// Analyze scan results with AI
async fn analyze_scan_with_ai(config: &AppConfig, results: &SecurityScanResults) -> Result<(), AppError> {
    println!("\n{}", "🤖 AI Security Analysis".cyan().bold());
    println!("{}", "=".repeat(50).cyan());

    let findings_summary = if results.findings.is_empty() {
        "No security vulnerabilities were found in the scanned code.".to_string()
    } else {
        let mut summary = format!("Found {} security findings:\n", results.findings.len());
        
        // Group findings by severity for summary
        let mut severity_counts = HashMap::new();
        for finding in &results.findings {
            *severity_counts.entry(finding.severity).or_insert(0) += 1;
        }
        
        for (severity, count) in severity_counts {
            summary.push_str(&format!("- {}: {} issues\n", severity, count));
        }
        
        // Add top findings details
        summary.push_str("\nTop findings:\n");
        for (i, finding) in results.findings.iter().take(5).enumerate() {
            summary.push_str(&format!("{}. {} in {} (Line {}): {}\n", 
                i + 1, finding.title, finding.file_path.display(), finding.line_start, finding.description));
        }
        
        summary
    };

    let user_message = format!(
        "Analyze the following security scan results from Tree-sitter AST analysis:\n\n\
        {}\n\n\
        Please provide:\n\
        1. Risk assessment and prioritization\n\
        2. Security recommendations\n\
        3. Patterns in vulnerabilities\n\
        4. Remediation strategies\n\
        5. Best practices to prevent similar issues",
        findings_summary
    );

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "You are a cybersecurity expert analyzing code security scan results. Provide actionable insights, risk assessments, and specific remediation guidance.".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: user_message,
        },
    ];

    match execute_ai_request_generic(config, messages, "Security Analysis", true).await {
        Ok(analysis) => {
            println!("{}", analysis);
        }
        Err(e) => {
            println!("{} {}", "⚠️  AI analysis failed:".yellow(), e);
        }
    }

    Ok(())
}

/// Generate output filename based on configuration and arguments
fn generate_output_filename(args: &ScanArgs, scan_config: &crate::config::ScanConfig, scan_path: &str) -> Result<String, AppError> {
    if let Some(output) = &args.output {
        return Ok(output.clone());
    }

    // Get repository name from scan path
    let repo_name = if scan_path == "." {
        std::env::current_dir()
            .map_err(|e| AppError::Tool(format!("Failed to get current directory: {}", e)))?
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    } else {
        Path::new(scan_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    };

    // Get current commit ID if in a git repository
    let commit_id = get_current_commit_id().unwrap_or_else(|| {
        chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string()
    });

    // Expand storage path
    let storage_path = shellexpand::tilde(&scan_config.storage_path);
    let extension = if scan_config.output_format == "json" { "json" } else { "md" };
    
    let filename = format!("scan_{}.{}", commit_id, extension);
    let full_path = Path::new(storage_path.as_ref())
        .join(&repo_name)
        .join(filename);

    Ok(full_path.to_string_lossy().to_string())
}

/// Get current git commit ID
fn get_current_commit_id() -> Option<String> {
    use std::process::Command;
    
    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;
    
    if output.status.success() {
        let commit_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !commit_id.is_empty() {
            return Some(commit_id);
        }
    }
    
    None
}

/// Run Semgrep scan with configuration and return results
async fn run_semgrep_scan_with_config(scan_path: &str, scan_config: &crate::config::ScanConfig) -> Result<Vec<SemgrepFinding>, AppError> {
    // Check if Semgrep is available
    let version_check = AsyncCommand::new("semgrep")
        .arg("--version")
        .output()
        .await;
    
    if version_check.is_err() || !version_check.unwrap().status.success() {
        println!("{}", "⚠️  Semgrep not available, skipping Semgrep scan".yellow());
        return Ok(Vec::new());
    }

    let mut cmd = AsyncCommand::new("semgrep");
    
    // Use custom config file if specified, otherwise use auto
    if let Some(config_file) = &scan_config.semgrep.config_file {
        let config_path = shellexpand::tilde(config_file);
        if Path::new(config_path.as_ref()).exists() {
            cmd.arg(format!("--config={}", config_path));
        } else {
            println!("{} {}", "⚠️  Semgrep config file not found, using auto:".yellow(), config_file);
            cmd.arg("--config=auto");
        }
    } else {
        // Check for default config file in scan rules directory
        let default_config = Path::new(&shellexpand::tilde(&scan_config.rules_path).as_ref()).join("semgrep.yml");
        if default_config.exists() {
            cmd.arg(format!("--config={}", default_config.display()));
        } else {
            cmd.arg("--config=auto");
        }
    }
    
    // Add custom rules if specified
    for rule in &scan_config.semgrep.rules {
        cmd.arg("--config").arg(rule);
    }
    
    cmd.arg("--json")
       .arg("--quiet")
       .arg("--no-git-ignore");
    
    // Add extra arguments
    for arg in &scan_config.semgrep.extra_args {
        cmd.arg(arg);
    }
    
    // Set timeout
    cmd.arg("--timeout").arg(scan_config.semgrep.timeout.to_string());
    
    cmd.arg(scan_path);

    let output = cmd.output().await
        .map_err(|e| AppError::Tool(format!("Failed to execute Semgrep: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("{} {}", "⚠️  Semgrep scan failed:".yellow(), stderr);
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        return Ok(Vec::new());
    }

    let results: SemgrepResults = serde_json::from_str(&stdout)
        .map_err(|e| AppError::Tool(format!("Failed to parse Semgrep output: {}", e)))?;

    Ok(results.results)
}

/// Run Semgrep scan and return results
async fn run_semgrep_scan(scan_path: &str) -> Result<Vec<SemgrepFinding>, AppError> {
    // Check if Semgrep is available
    let version_check = AsyncCommand::new("semgrep")
        .arg("--version")
        .output()
        .await;
    
    if version_check.is_err() || !version_check.unwrap().status.success() {
        println!("{}", "⚠️  Semgrep not available, skipping Semgrep scan".yellow());
        return Ok(Vec::new());
    }

    let mut cmd = AsyncCommand::new("semgrep");
    cmd.arg("--config=auto")
       .arg("--json")
       .arg("--quiet")
       .arg("--no-git-ignore")
       .arg(scan_path);

    let output = cmd.output().await
        .map_err(|e| AppError::Tool(format!("Failed to execute Semgrep: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("{} {}", "⚠️  Semgrep scan failed:".yellow(), stderr);
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        return Ok(Vec::new());
    }

    let results: SemgrepResults = serde_json::from_str(&stdout)
        .map_err(|e| AppError::Tool(format!("Failed to parse Semgrep output: {}", e)))?;

    Ok(results.results)
}

/// Merge Semgrep results into Tree-sitter results
fn merge_semgrep_results(tree_sitter_results: &mut SecurityScanResults, semgrep_findings: Vec<SemgrepFinding>) {
    for semgrep_finding in semgrep_findings {
        // Convert Semgrep finding to SecurityFinding
        let severity = match semgrep_finding.extra.severity.to_lowercase().as_str() {
            "error" => SecuritySeverity::Critical,
            "warning" => SecuritySeverity::High,
            "info" => SecuritySeverity::Medium,
            _ => SecuritySeverity::Low,
        };

        let finding = SecurityFinding {
            id: format!("semgrep-{}", semgrep_finding.check_id),
            title: semgrep_finding.check_id.clone(),
            description: semgrep_finding.extra.message.clone(),
            severity,
            file_path: PathBuf::from(&semgrep_finding.path),
            line_start: semgrep_finding.start.line as usize,
            line_end: semgrep_finding.end.line as usize,
            column_start: semgrep_finding.start.col as usize,
            column_end: semgrep_finding.end.col as usize,
            code_snippet: "".to_string(), // Semgrep doesn't provide code snippets in JSON output
            recommendation: format!("Review and fix the issue identified by Semgrep rule: {}", semgrep_finding.check_id),
            cwe_id: None,
            owasp_category: None,
        };

        tree_sitter_results.findings.push(finding);
    }

    // Update summary counts
    tree_sitter_results.summary.total_findings = tree_sitter_results.findings.len();
    
    // Recalculate severity counts
    tree_sitter_results.summary.critical_count = tree_sitter_results.findings.iter()
        .filter(|f| matches!(f.severity, SecuritySeverity::Critical)).count();
    tree_sitter_results.summary.high_count = tree_sitter_results.findings.iter()
        .filter(|f| matches!(f.severity, SecuritySeverity::High)).count();
    tree_sitter_results.summary.medium_count = tree_sitter_results.findings.iter()
        .filter(|f| matches!(f.severity, SecuritySeverity::Medium)).count();
    tree_sitter_results.summary.low_count = tree_sitter_results.findings.iter()
        .filter(|f| matches!(f.severity, SecuritySeverity::Low)).count();
    tree_sitter_results.summary.info_count = tree_sitter_results.findings.iter()
        .filter(|f| matches!(f.severity, SecuritySeverity::Info)).count();
}

/// Check if Semgrep is available
async fn is_semgrep_available() -> bool {
    AsyncCommand::new("semgrep")
        .arg("--version")
        .output()
        .await
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Run Semgrep as fallback scanner (legacy function, kept for compatibility)
async fn run_semgrep_fallback(args: &ScanArgs) -> Result<(), AppError> {
    println!("{}", "Running Semgrep fallback scan...".cyan());

    let mut cmd = AsyncCommand::new("semgrep");
    cmd.arg("--config=auto")
       .arg("--json")
       .arg("--quiet");

    // Add custom rules if specified
    if let Some(rules) = &args.rules {
        cmd.arg("--config").arg(rules);
    }

    // Add severity filter
    if let Some(severity) = &args.severity {
        cmd.arg("--severity").arg(severity);
    }

    // Add exclude patterns
    if let Some(exclude) = &args.exclude {
        for pattern in exclude {
            cmd.arg("--exclude").arg(pattern);
        }
    }

    // Set scan path
    let scan_path = args.path.as_deref().unwrap_or(".");
    cmd.arg(scan_path);

    // Execute Semgrep
    let output = cmd.output().await
        .map_err(|e| AppError::Tool(format!("Failed to execute Semgrep: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("{} {}", "⚠️  Semgrep scan failed:".yellow(), stderr);
        return Ok(());
    }

    // Parse results
    let stdout = String::from_utf8_lossy(&output.stdout);
    let results: Value = serde_json::from_str(&stdout)
        .map_err(|e| AppError::Tool(format!("Failed to parse Semgrep output: {}", e)))?;

    let findings = results["results"].as_array()
        .ok_or_else(|| AppError::Tool("Invalid Semgrep output format".to_string()))?;

    if findings.is_empty() {
        println!("{}", "✅ Semgrep found no additional issues".green());
    } else {
        println!("{} Semgrep found {} additional findings", "📋".cyan(), findings.len());
        
        if args.detailed {
            for (i, finding) in findings.iter().enumerate() {
                let rule_id = finding["check_id"].as_str().unwrap_or("unknown");
                let message = finding["extra"]["message"].as_str().unwrap_or("No message");
                let file_path = finding["path"].as_str().unwrap_or("unknown");
                let start_line = finding["start"]["line"].as_u64().unwrap_or(0);

                println!("\n{}. {} ({}:{})", i + 1, rule_id.yellow().bold(), file_path.cyan(), start_line);
                println!("   {}", message);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_collect_source_files() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");
        fs::write(&rust_file, "fn main() {}").unwrap();

        let files = collect_source_files(temp_dir.path().to_str().unwrap(), &None).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].2, "rust");
    }

    #[test]
    fn test_language_detection() {
        let path = Path::new("test.rs");
        let result = read_and_detect_language(path);
        // This will fail because the file doesn't exist, but we can test the logic
        assert!(result.is_err());
    }
}
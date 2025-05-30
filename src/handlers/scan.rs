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

/// Handle the scan command using Tree-sitter AST analysis with Semgrep fallback
pub async fn handle_scan(config: &AppConfig, args: ScanArgs) -> Result<(), AppError> {
    println!("{}", "🔍 Starting Tree-sitter security scan...".cyan().bold());

    // Initialize tree-sitter security scanner
    let language_registry = create_language_registry();
    let scanner = TreeSitterSecurityScanner::new(language_registry);

    // Collect files to scan
    let scan_path = args.path.as_deref().unwrap_or(".");
    let files = collect_source_files(scan_path, &args.exclude)?;
    
    if files.is_empty() {
        println!("{}", "⚠️  No source files found to scan".yellow());
        return Ok(());
    }

    println!("📁 Scanning {} files with Tree-sitter AST analysis...", files.len());

    // Perform security scan
    let scan_results = scanner.scan_files(files)
        .map_err(|e| AppError::Tool(format!("Tree-sitter scan failed: {}", e)))?;

    // Display results
    display_tree_sitter_results(&scan_results, &args).await?;

    // AI analysis if requested
    if args.ai_analysis {
        analyze_scan_with_ai(config, &scan_results).await?;
    }

    // Save to file if specified
    if let Some(output_file) = &args.output {
        let json_output = scan_results.to_json()
            .map_err(|e| AppError::Tool(format!("Failed to serialize results: {}", e)))?;
        fs::write(output_file, &json_output)
            .map_err(|e| AppError::Tool(format!("Failed to write output file: {}", e)))?;
        println!("{} {}", "📄 Results saved to:".green(), output_file);
    }

    // Fallback to Semgrep if requested or if no findings and Semgrep is available
    if (scan_results.findings.is_empty() || args.rules.is_some()) && is_semgrep_available().await {
        println!("{}", "🔄 Running Semgrep as additional check...".cyan());
        run_semgrep_fallback(&args).await?;
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
        "js" | "jsx" => "javascript",
        "ts" | "tsx" => "typescript",
        "java" => "java",
        "cpp" | "cc" | "cxx" => "cpp",
        "c" => "c",
        "go" => "go",
        "php" => "php",
        "rb" => "ruby",
        "cs" => "csharp",
        "swift" => "swift",
        "kt" => "kotlin",
        "scala" => "scala",
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

/// Check if Semgrep is available
async fn is_semgrep_available() -> bool {
    AsyncCommand::new("semgrep")
        .arg("--version")
        .output()
        .await
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Run Semgrep as fallback scanner
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
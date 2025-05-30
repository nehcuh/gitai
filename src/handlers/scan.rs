use crate::config::AppConfig;
use crate::errors::AppError;
use crate::types::general::ScanArgs;
use colored::*;
use serde_json::Value;
use tokio::process::Command as AsyncCommand;

/// Handle the scan command using Semgrep
pub async fn handle_scan(config: &AppConfig, args: ScanArgs) -> Result<(), AppError> {
    println!("{}", "🔍 Starting Semgrep code scan...".cyan().bold());

    // Check if Semgrep is installed
    if !is_semgrep_installed().await? {
        return Err(AppError::Tool("Semgrep is not installed. Please install it first: pip install semgrep".to_string()));
    }

    // Build Semgrep command
    let mut cmd = AsyncCommand::new("semgrep");
    
    // Add basic arguments
    cmd.arg("--config=auto")  // Use Semgrep's default rules
        .arg("--json")        // Output in JSON format for parsing
        .arg("--verbose");    // Verbose output

    // Add custom rules if specified
    if let Some(rules) = &args.rules {
        cmd.arg("--config").arg(rules);
    }

    // Add severity filter if specified
    if let Some(severity) = &args.severity {
        cmd.arg("--severity").arg(severity);
    }

    // Add exclude patterns if specified
    if let Some(exclude) = &args.exclude {
        for pattern in exclude {
            cmd.arg("--exclude").arg(pattern);
        }
    }

    // Set target path (default to current directory)
    let target_path = args.path.as_deref().unwrap_or(".");
    cmd.arg(target_path);

    println!("{} {}", "Running:".green(), format!("semgrep --config=auto --json --verbose {}", target_path).yellow());

    // Execute Semgrep
    let output = cmd.output().await.map_err(|e| {
        AppError::Tool(format!("Failed to execute Semgrep: {}", e))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Tool(format!("Semgrep failed: {}", stderr)));
    }

    // Parse and display results
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_and_display_results(&stdout, &args).await?;

    // If AI analysis is enabled, provide AI insights
    if args.ai_analysis {
        provide_ai_analysis(config, &stdout).await?;
    }

    Ok(())
}

/// Check if Semgrep is installed
async fn is_semgrep_installed() -> Result<bool, AppError> {
    let output = AsyncCommand::new("semgrep")
        .arg("--version")
        .output()
        .await;

    match output {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Ok(false),
    }
}

/// Parse Semgrep JSON output and display results
async fn parse_and_display_results(json_output: &str, args: &ScanArgs) -> Result<(), AppError> {
    let results: Value = serde_json::from_str(json_output)
        .map_err(|e| AppError::Tool(format!("Failed to parse Semgrep output: {}", e)))?;

    let findings = results["results"].as_array()
        .ok_or_else(|| AppError::Tool("Invalid Semgrep output format".to_string()))?;

    println!("\n{}", "📊 Scan Results".cyan().bold());
    println!("{}", "=".repeat(50).cyan());

    if findings.is_empty() {
        println!("{}", "✅ No security issues found!".green().bold());
        return Ok(());
    }

    // Group findings by severity
    let mut critical = Vec::new();
    let mut high = Vec::new();
    let mut medium = Vec::new();
    let mut low = Vec::new();
    let mut info = Vec::new();

    for finding in findings {
        let severity = finding["extra"]["severity"].as_str().unwrap_or("INFO");
        match severity.to_uppercase().as_str() {
            "CRITICAL" => critical.push(finding),
            "HIGH" => high.push(finding),
            "MEDIUM" => medium.push(finding),
            "LOW" => low.push(finding),
            _ => info.push(finding),
        }
    }

    // Display summary
    println!("\n{}", "Summary:".yellow().bold());
    if !critical.is_empty() { println!("  {} Critical: {}", "🔴".red(), critical.len().to_string().red().bold()); }
    if !high.is_empty() { println!("  {} High: {}", "🟠".yellow(), high.len().to_string().yellow().bold()); }
    if !medium.is_empty() { println!("  {} Medium: {}", "🟡".yellow(), medium.len().to_string().yellow()); }
    if !low.is_empty() { println!("  {} Low: {}", "🔵".blue(), low.len().to_string().blue()); }
    if !info.is_empty() { println!("  {} Info: {}", "⚪".white(), info.len().to_string().white()); }

    // Display detailed findings if requested
    if args.detailed {
        display_detailed_findings(&critical, "CRITICAL", "🔴");
        display_detailed_findings(&high, "HIGH", "🟠");
        display_detailed_findings(&medium, "MEDIUM", "🟡");
        if args.show_low_severity {
            display_detailed_findings(&low, "LOW", "🔵");
            display_detailed_findings(&info, "INFO", "⚪");
        }
    }

    // Save results to file if specified
    if let Some(output_file) = &args.output {
        tokio::fs::write(output_file, json_output).await
            .map_err(|e| AppError::Tool(format!("Failed to write output file: {}", e)))?;
        println!("\n{} {}", "💾 Results saved to:".green(), output_file.yellow());
    }

    Ok(())
}

/// Display detailed findings for a specific severity level
fn display_detailed_findings(findings: &[&Value], severity: &str, icon: &str) {
    if findings.is_empty() {
        return;
    }

    println!("\n{} {} {} Issues:", icon, severity, findings.len());
    println!("{}", "-".repeat(40));

    for (i, finding) in findings.iter().enumerate() {
        let rule_id = finding["check_id"].as_str().unwrap_or("unknown");
        let message = finding["extra"]["message"].as_str().unwrap_or("No message");
        let file_path = finding["path"].as_str().unwrap_or("unknown");
        let start_line = finding["start"]["line"].as_u64().unwrap_or(0);

        println!("\n{}. {} ({}:{})", i + 1, rule_id.yellow().bold(), file_path.cyan(), start_line);
        println!("   {}", message);
        
        if let Some(fix) = finding["extra"]["fix"].as_str() {
            println!("   {} {}", "Fix:".green().bold(), fix);
        }
    }
}

/// Provide AI analysis of scan results
async fn provide_ai_analysis(config: &AppConfig, scan_results: &str) -> Result<(), AppError> {
    use crate::handlers::ai::execute_ai_request_generic;

    println!("\n{}", "🤖 AI Analysis".cyan().bold());
    println!("{}", "=".repeat(50).cyan());

    use crate::types::ai::ChatMessage;

    let user_message = format!(
        "Analyze the following Semgrep security scan results and provide insights:\n\
        1. Summarize the most critical security issues\n\
        2. Suggest prioritization for fixing issues\n\
        3. Provide general security recommendations\n\
        4. Identify patterns in the vulnerabilities\n\n\
        Scan Results:\n{}",
        scan_results
    );

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "You are a security expert analyzing code scan results. Provide actionable insights and recommendations.".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: user_message,
        },
    ];

    match execute_ai_request_generic(config, messages, "Security Scan Analysis", true).await {
        Ok(analysis) => {
            println!("{}", analysis);
        }
        Err(e) => {
            println!("{} {}", "⚠️  AI analysis failed:".yellow(), e);
        }
    }

    Ok(())
}

/// Install Semgrep if not present
pub async fn install_semgrep() -> Result<(), AppError> {
    println!("{}", "📦 Installing Semgrep...".cyan().bold());

    let output = AsyncCommand::new("pip")
        .args(&["install", "semgrep"])
        .output()
        .await
        .map_err(|e| AppError::Tool(format!("Failed to install Semgrep: {}", e)))?;

    if output.status.success() {
        println!("{}", "✅ Semgrep installed successfully!".green().bold());
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::Tool(format!("Failed to install Semgrep: {}", stderr)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_semgrep_installation_check() {
        // This test will pass if Semgrep is installed, otherwise it will show it's not installed
        let result = is_semgrep_installed().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_empty_results() {
        let empty_results = r#"{"results": []}"#;
        let args = ScanArgs::default();
        let result = parse_and_display_results(empty_results, &args).await;
        assert!(result.is_ok());
    }
}
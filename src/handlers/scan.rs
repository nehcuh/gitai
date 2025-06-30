use crate::{
    ast_grep_analyzer::{
        core::{CodeIssue, IssueSeverity, create_analysis_engine},
        language_support::{LanguageStats, LanguageSupport},
        rule_manager::RuleManager,
    },
    config::AppConfig,
    errors::AppError,
    types::git::ScanArgs,
};
use colored::Colorize;

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};
use tracing::{debug, info, warn};
use walkdir::WalkDir;

/// Comprehensive scan results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResults {
    /// Total files scanned
    pub files_scanned: usize,
    /// Total issues found
    pub total_issues: usize,
    /// Issues by severity
    pub issues_by_severity: HashMap<String, usize>,
    /// Issues by language
    pub issues_by_language: HashMap<String, usize>,
    /// Issues by rule
    pub issues_by_rule: HashMap<String, usize>,
    /// Scan duration in milliseconds
    pub scan_duration_ms: u64,
    /// Detailed file results
    pub file_results: Vec<FileResult>,
    /// Language statistics
    pub language_stats: Option<LanguageStats>,
    /// Configuration used for scan
    pub scan_config: ScanConfig,
}

/// Result for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResult {
    /// File path
    pub file_path: String,
    /// Detected language
    pub language: String,
    /// Issues found in this file
    pub issues: Vec<CodeIssue>,
    /// File size in bytes
    pub file_size: u64,
    /// Lines of code
    pub lines_of_code: usize,
    /// Analysis duration for this file
    pub analysis_duration_ms: u64,
}

/// Scan configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// Target path
    pub target: String,
    /// Languages to scan
    pub languages: Vec<String>,
    /// Rules to use
    pub rules: Vec<String>,
    /// Severity levels
    pub severity_levels: Vec<String>,
    /// Include patterns
    pub include_patterns: Vec<String>,
    /// Exclude patterns
    pub exclude_patterns: Vec<String>,
    /// Parallel processing enabled
    pub parallel: bool,
    /// Maximum issues to report
    pub max_issues: usize,
}

/// Statistics collector for parallel scanning
#[derive(Debug, Default)]
struct ScanStats {
    files_scanned: usize,
    total_issues: usize,
    issues_by_severity: HashMap<String, usize>,
    issues_by_language: HashMap<String, usize>,
    issues_by_rule: HashMap<String, usize>,
}

impl ScanStats {
    fn add_result(&mut self, result: &FileResult) {
        self.files_scanned += 1;
        self.total_issues += result.issues.len();

        // Count by severity
        for issue in &result.issues {
            let severity = format!("{:?}", issue.severity).to_lowercase();
            *self.issues_by_severity.entry(severity).or_insert(0) += 1;
        }

        // Count by language
        *self
            .issues_by_language
            .entry(result.language.clone())
            .or_insert(0) += result.issues.len();

        // Count by rule
        for issue in &result.issues {
            *self
                .issues_by_rule
                .entry(issue.rule_id.clone())
                .or_insert(0) += 1;
        }
    }
}

/// Main scan handler
pub async fn handle_scan(_config: &AppConfig, args: &ScanArgs) -> Result<(), AppError> {
    let start_time = Instant::now();

    info!("🔍 开始代码扫描");
    info!("目标路径: {}", args.target);

    // Initialize components
    let language_support = LanguageSupport::new();
    let _rule_manager = RuleManager::new(None).map_err(|e| AppError::Analysis(e))?;

    // Parse configuration
    let scan_config = parse_scan_config(args, &language_support)?;

    if args.verbose {
        print_scan_config(&scan_config);
    }

    // Discover files to scan
    let files_to_scan = discover_files(&scan_config, &language_support)?;

    if files_to_scan.is_empty() {
        warn!("⚠️  未找到符合条件的文件");
        return Ok(());
    }

    info!("📂 发现 {} 个文件需要扫描", files_to_scan.len());

    // Perform scanning
    let results = if args.parallel {
        perform_parallel_scan(&files_to_scan, &scan_config, &language_support).await?
    } else {
        perform_sequential_scan(&files_to_scan, &scan_config, &language_support).await?
    };

    let scan_duration = start_time.elapsed();

    // Create final results
    let final_results = ScanResults {
        files_scanned: results.files_scanned,
        total_issues: results.total_issues,
        issues_by_severity: results.issues_by_severity,
        issues_by_language: results.issues_by_language,
        issues_by_rule: results.issues_by_rule,
        scan_duration_ms: scan_duration.as_millis() as u64,
        file_results: results.file_results,
        language_stats: Some(language_support.get_language_stats()),
        scan_config,
    };

    // Output results
    output_results(&final_results, args).await?;

    // Print statistics if requested
    if args.stats {
        print_statistics(&final_results);
    }

    // Exit with error code if issues found and fail_on_error is set
    if args.fail_on_error && final_results.total_issues > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Parse scan configuration from arguments
fn parse_scan_config(
    args: &ScanArgs,
    language_support: &LanguageSupport,
) -> Result<ScanConfig, AppError> {
    // Parse languages
    let languages = if let Some(lang_str) = &args.languages {
        lang_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|lang| {
                if !language_support.is_language_supported(lang) {
                    warn!("⚠️  不支持的语言: {}", lang);
                    false
                } else {
                    true
                }
            })
            .collect()
    } else {
        language_support.get_enabled_languages()
    };

    // Parse rules
    let rules = if let Some(rules_str) = &args.rules {
        rules_str.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        vec![] // Use all rules
    };

    // Parse severity levels
    let severity_levels: Vec<String> = args
        .severity
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    // Parse include/exclude patterns
    let include_patterns = if let Some(pattern) = &args.include {
        vec![pattern.clone()]
    } else {
        vec![]
    };

    let exclude_patterns = if let Some(pattern) = &args.exclude {
        vec![pattern.clone()]
    } else {
        vec![]
    };

    Ok(ScanConfig {
        target: args.target.clone(),
        languages,
        rules,
        severity_levels,
        include_patterns,
        exclude_patterns,
        parallel: args.parallel,
        max_issues: args.max_issues,
    })
}

/// Print scan configuration
fn print_scan_config(config: &ScanConfig) {
    println!("📋 扫描配置:");
    println!("  目标: {}", config.target);
    println!("  语言: {:?}", config.languages);
    println!("  规则: {:?}", config.rules);
    println!("  严重级别: {:?}", config.severity_levels);
    if !config.include_patterns.is_empty() {
        println!("  包含模式: {:?}", config.include_patterns);
    }
    if !config.exclude_patterns.is_empty() {
        println!("  排除模式: {:?}", config.exclude_patterns);
    }
    println!("  并行处理: {}", config.parallel);
    if config.max_issues > 0 {
        println!("  最大问题数: {}", config.max_issues);
    }
    println!();
}

/// Discover files to scan
fn discover_files(
    config: &ScanConfig,
    language_support: &LanguageSupport,
) -> Result<Vec<(PathBuf, String)>, AppError> {
    let target_path = Path::new(&config.target);

    if !target_path.exists() {
        return Err(AppError::Generic(format!(
            "目标路径不存在: {}",
            config.target
        )));
    }

    let mut files = Vec::new();

    if target_path.is_file() {
        // Single file
        if let Some(language) = language_support.detect_language_from_path(target_path) {
            if config.languages.is_empty() || config.languages.contains(&language) {
                files.push((target_path.to_path_buf(), language));
            }
        }
    } else {
        // Directory - walk recursively
        for entry in WalkDir::new(target_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            // Check include/exclude patterns
            if !should_include_file(path, &config.include_patterns, &config.exclude_patterns) {
                continue;
            }

            // Check language support
            if let Some(language) = language_support.detect_language_from_path(path) {
                if config.languages.is_empty() || config.languages.contains(&language) {
                    files.push((path.to_path_buf(), language));
                }
            }
        }
    }

    Ok(files)
}

/// Check if file should be included based on patterns
fn should_include_file(
    path: &Path,
    include_patterns: &[String],
    exclude_patterns: &[String],
) -> bool {
    let path_str = path.to_string_lossy();

    // Check exclude patterns first
    for pattern in exclude_patterns {
        if let Ok(regex) = Regex::new(pattern) {
            if regex.is_match(&path_str) {
                return false;
            }
        }
    }

    // If no include patterns, include by default
    if include_patterns.is_empty() {
        return true;
    }

    // Check include patterns
    for pattern in include_patterns {
        if let Ok(regex) = Regex::new(pattern) {
            if regex.is_match(&path_str) {
                return true;
            }
        }
    }

    false
}

/// Perform parallel scanning
async fn perform_parallel_scan(
    files: &[(PathBuf, String)],
    config: &ScanConfig,
    language_support: &LanguageSupport,
) -> Result<ScanResults, AppError> {
    info!("🚀 使用并行处理扫描 {} 个文件", files.len());

    let stats = Arc::new(Mutex::new(ScanStats::default()));
    let file_results = Arc::new(Mutex::new(Vec::new()));

    // Process files in parallel
    files
        .iter()
        .try_for_each(|(path, language)| -> Result<(), AppError> {
            match scan_single_file(path, language, config, language_support) {
                Ok(result) => {
                    // Update statistics
                    {
                        let mut stats_guard = stats.lock().unwrap();
                        stats_guard.add_result(&result);
                    }

                    // Store result
                    {
                        let mut results_guard = file_results.lock().unwrap();
                        results_guard.push(result);
                    }

                    Ok(())
                }
                Err(e) => {
                    warn!("扫描文件失败 {}: {}", path.display(), e);
                    Ok(()) // Continue with other files
                }
            }
        })?;

    let final_stats = Arc::try_unwrap(stats).unwrap().into_inner().unwrap();
    let final_results = Arc::try_unwrap(file_results).unwrap().into_inner().unwrap();

    Ok(ScanResults {
        files_scanned: final_stats.files_scanned,
        total_issues: final_stats.total_issues,
        issues_by_severity: final_stats.issues_by_severity,
        issues_by_language: final_stats.issues_by_language,
        issues_by_rule: final_stats.issues_by_rule,
        scan_duration_ms: 0, // Will be set by caller
        file_results: final_results,
        language_stats: None, // Will be set by caller
        scan_config: config.clone(),
    })
}

/// Perform sequential scanning
async fn perform_sequential_scan(
    files: &[(PathBuf, String)],
    config: &ScanConfig,
    language_support: &LanguageSupport,
) -> Result<ScanResults, AppError> {
    info!("📝 使用顺序处理扫描 {} 个文件", files.len());

    let mut stats = ScanStats::default();
    let mut file_results = Vec::new();

    for (i, (path, language)) in files.iter().enumerate() {
        if config.max_issues > 0 && stats.total_issues >= config.max_issues {
            info!("达到最大问题数限制 ({}), 停止扫描", config.max_issues);
            break;
        }

        debug!("扫描文件 [{}/{}]: {}", i + 1, files.len(), path.display());

        match scan_single_file(path, language, config, language_support) {
            Ok(result) => {
                stats.add_result(&result);
                file_results.push(result);
            }
            Err(e) => {
                warn!("扫描文件失败 {}: {}", path.display(), e);
            }
        }
    }

    Ok(ScanResults {
        files_scanned: stats.files_scanned,
        total_issues: stats.total_issues,
        issues_by_severity: stats.issues_by_severity,
        issues_by_language: stats.issues_by_language,
        issues_by_rule: stats.issues_by_rule,
        scan_duration_ms: 0, // Will be set by caller
        file_results,
        language_stats: None, // Will be set by caller
        scan_config: config.clone(),
    })
}

/// Scan a single file
fn scan_single_file(
    path: &Path,
    language: &str,
    config: &ScanConfig,
    _language_support: &LanguageSupport,
) -> Result<FileResult, AppError> {
    let start_time = Instant::now();

    // Read file content
    let content = fs::read_to_string(path)
        .map_err(|e| AppError::IO(format!("读取文件失败: {}", path.display()), e))?;

    let file_size = content.len() as u64;
    let lines_of_code = content.lines().count();

    // Create analyzer
    let analysis_engine = create_analysis_engine();

    // Analyze file
    let issues = analysis_engine
        .analyze_file_content(&content, language, path)
        .map_err(|e| AppError::Generic(format!("Analysis error: {}", e)))?
        .into_iter()
        .filter(|issue| should_include_issue(issue, config))
        .collect();

    let analysis_duration = start_time.elapsed();

    Ok(FileResult {
        file_path: path.to_string_lossy().to_string(),
        language: language.to_string(),
        issues,
        file_size,
        lines_of_code,
        analysis_duration_ms: analysis_duration.as_millis() as u64,
    })
}

/// Check if issue should be included based on configuration
fn should_include_issue(issue: &CodeIssue, config: &ScanConfig) -> bool {
    // Check severity filter
    let severity_str = format!("{:?}", issue.severity).to_lowercase();
    if !config.severity_levels.contains(&severity_str) {
        return false;
    }

    // Check rule filter
    if !config.rules.is_empty() && !config.rules.contains(&issue.rule_id) {
        return false;
    }

    true
}

/// Output scan results in the specified format
async fn output_results(results: &ScanResults, args: &ScanArgs) -> Result<(), AppError> {
    let output_content = match args.format.as_str() {
        "json" => format_json_output(results)?,
        "sarif" => format_sarif_output(results)?,
        "csv" => format_csv_output(results)?,
        "text" | _ => format_text_output(results, args.verbose),
    };

    if let Some(output_file) = &args.output {
        fs::write(output_file, &output_content)
            .map_err(|e| AppError::IO(format!("写入输出文件失败: {}", output_file), e))?;
        info!("📄 结果已保存到: {}", output_file);
    } else {
        println!("{}", output_content);
    }

    Ok(())
}

/// Format results as JSON
fn format_json_output(results: &ScanResults) -> Result<String, AppError> {
    serde_json::to_string_pretty(results)
        .map_err(|e| AppError::Generic(format!("JSON序列化失败: {}", e)))
}

/// Format results as SARIF (Static Analysis Results Interchange Format)
fn format_sarif_output(results: &ScanResults) -> Result<String, AppError> {
    // SARIF is a standardized format for static analysis results
    let mut sarif = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "GitAI AST-Grep Scanner",
                    "version": "1.0.0",
                    "informationUri": "https://github.com/gitai-project/gitai"
                }
            },
            "results": []
        }]
    });

    let mut sarif_results = Vec::new();

    for file_result in &results.file_results {
        for issue in &file_result.issues {
            let sarif_result = serde_json::json!({
                "ruleId": issue.rule_id,
                "level": match issue.severity {
                    IssueSeverity::Error => "error",
                    IssueSeverity::Warning => "warning",
                    IssueSeverity::Info => "note",
                    IssueSeverity::Hint => "note",
                },
                "message": {
                    "text": issue.message
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": file_result.file_path
                        },
                        "region": {
                            "startLine": issue.line,
                            "startColumn": issue.column,
                            "endLine": issue.end_line.unwrap_or(issue.line),
                            "endColumn": issue.end_column.unwrap_or(issue.column)
                        }
                    }
                }]
            });
            sarif_results.push(sarif_result);
        }
    }

    sarif["runs"][0]["results"] = serde_json::Value::Array(sarif_results);

    serde_json::to_string_pretty(&sarif)
        .map_err(|e| AppError::Generic(format!("SARIF序列化失败: {}", e)))
}

/// Format results as CSV
fn format_csv_output(results: &ScanResults) -> Result<String, AppError> {
    let mut csv = String::new();
    csv.push_str("File,Language,Rule,Severity,Line,Column,Message,Suggestion\n");

    for file_result in &results.file_results {
        for issue in &file_result.issues {
            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{:?}\",{},{},\"{}\",\"{}\"\n",
                file_result.file_path,
                file_result.language,
                issue.rule_id,
                issue.severity,
                issue.line,
                issue.column,
                issue.message.replace("\"", "\"\""),
                issue
                    .suggestion
                    .as_ref()
                    .unwrap_or(&"".to_string())
                    .replace("\"", "\"\"")
            ));
        }
    }

    Ok(csv)
}

/// Format results as text
fn format_text_output(results: &ScanResults, verbose: bool) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!("🔍 {} 扫描完成\n", "AST-Grep".bright_blue()));
    output.push_str(&format!(
        "📂 扫描了 {} 个文件\n",
        results.files_scanned.to_string().bright_green()
    ));
    output.push_str(&format!(
        "⚠️  发现 {} 个问题\n",
        results.total_issues.to_string().bright_yellow()
    ));
    output.push_str(&format!(
        "⏱️  耗时 {} ms\n\n",
        results.scan_duration_ms.to_string().bright_blue()
    ));

    // Issues by severity
    if !results.issues_by_severity.is_empty() {
        output.push_str("📊 问题分布:\n");
        for (severity, count) in &results.issues_by_severity {
            let color = match severity.as_str() {
                "error" => "red",
                "warning" => "yellow",
                "info" => "blue",
                "hint" => "green",
                _ => "white",
            };
            output.push_str(&format!(
                "  {}: {}\n",
                match color {
                    "red" => severity.bright_red(),
                    "yellow" => severity.bright_yellow(),
                    "blue" => severity.bright_blue(),
                    "green" => severity.bright_green(),
                    _ => severity.bright_white(),
                },
                count
            ));
        }
        output.push('\n');
    }

    // File results
    if verbose || results.total_issues > 0 {
        for file_result in &results.file_results {
            if file_result.issues.is_empty() && !verbose {
                continue;
            }

            output.push_str(&format!(
                "📄 {} ({})\n",
                file_result.file_path.bright_cyan(),
                file_result.language.bright_magenta()
            ));

            if file_result.issues.is_empty() {
                output.push_str("  ✅ 未发现问题\n");
            } else {
                for issue in &file_result.issues {
                    let severity_color = match issue.severity {
                        IssueSeverity::Error => "red",
                        IssueSeverity::Warning => "yellow",
                        IssueSeverity::Info => "blue",
                        IssueSeverity::Hint => "green",
                    };

                    output.push_str(&format!(
                        "  {}:{} {} [{}] {}\n",
                        issue.line.to_string().bright_white(),
                        issue.column.to_string().bright_white(),
                        match issue.severity {
                            IssueSeverity::Error => format!("{:?}", issue.severity).bright_red(),
                            IssueSeverity::Warning =>
                                format!("{:?}", issue.severity).bright_yellow(),
                            IssueSeverity::Info => format!("{:?}", issue.severity).bright_blue(),
                            IssueSeverity::Hint => format!("{:?}", issue.severity).bright_green(),
                        },
                        issue.rule_id.bright_cyan(),
                        issue.message
                    ));

                    if let Some(suggestion) = &issue.suggestion {
                        output.push_str(&format!("    💡 {}\n", suggestion.bright_blue()));
                    }
                }
            }
            output.push('\n');
        }
    }

    output
}

/// Print scan statistics
fn print_statistics(results: &ScanResults) {
    println!("📈 详细统计:");
    println!("  文件总数: {}", results.files_scanned);
    println!("  问题总数: {}", results.total_issues);
    println!("  扫描耗时: {} ms", results.scan_duration_ms);

    if let Some(stats) = &results.language_stats {
        println!("  支持语言: {}", stats.total_languages);
        println!("  启用语言: {}", stats.enabled_languages);
    }

    if !results.issues_by_language.is_empty() {
        println!("\n📊 按语言分布:");
        let mut lang_issues: Vec<_> = results.issues_by_language.iter().collect();
        lang_issues.sort_by(|a, b| b.1.cmp(a.1));
        for (language, count) in lang_issues {
            println!("  {}: {}", language, count);
        }
    }

    if !results.issues_by_rule.is_empty() {
        println!("\n🔍 按规则分布 (前10):");
        let mut rule_issues: Vec<_> = results.issues_by_rule.iter().collect();
        rule_issues.sort_by(|a, b| b.1.cmp(a.1));
        for (rule, count) in rule_issues.iter().take(10) {
            println!("  {}: {}", rule, count);
        }
    }
}

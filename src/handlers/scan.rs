use crate::{
    ast_grep_analyzer::{
        core::{CodeIssue, IssueSeverity, create_analysis_engine},
        language_support::{LanguageStats, LanguageSupport},
        rule_manager::RuleManager,
        translation::{SupportedLanguage, TranslationManager},
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
pub async fn handle_scan(config: &AppConfig, args: &ScanArgs) -> Result<(), AppError> {
    let start_time = Instant::now();

    // Initialize translation manager
    let mut translation_manager = TranslationManager::new(config.translation.clone())
        .map_err(|e| AppError::Generic(format!("Failed to create translation manager: {}", e)))?;

    translation_manager.initialize().await.map_err(|e| {
        AppError::Generic(format!("Failed to initialize translation manager: {}", e))
    })?;

    let is_chinese = translation_manager.is_enabled()
        && translation_manager.target_language() == &SupportedLanguage::Chinese;

    let scan_start_msg = if is_chinese {
        "🔍 开始代码扫描"
    } else {
        "🔍 Starting code scan"
    };

    let target_msg = if is_chinese {
        format!("目标路径: {}", args.target)
    } else {
        format!("Target path: {}", args.target)
    };

    info!("{}", scan_start_msg);
    info!("{}", target_msg);

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
        let no_files_msg = if is_chinese {
            "⚠️  未找到符合条件的文件"
        } else {
            "⚠️  No matching files found"
        };
        warn!("{}", no_files_msg);
        return Ok(());
    }

    let files_found_msg = if is_chinese {
        format!("📂 发现 {} 个文件需要扫描", files_to_scan.len())
    } else {
        format!("📂 Found {} files to scan", files_to_scan.len())
    };
    info!("{}", files_found_msg);

    // Perform scanning
    let results = if args.parallel {
        perform_parallel_scan(&files_to_scan, &scan_config, &language_support, is_chinese).await?
    } else {
        perform_sequential_scan(&files_to_scan, &scan_config, &language_support, is_chinese).await?
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
    output_results(&final_results, args, &translation_manager).await?;

    // Print statistics if requested
    if args.stats {
        print_statistics(&final_results, &translation_manager);
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
    is_chinese: bool,
) -> Result<ScanResults, AppError> {
    let scan_msg = if is_chinese {
        format!("📝 使用并行处理扫描 {} 个文件", files.len())
    } else {
        format!("📝 Scanning {} files in parallel", files.len())
    };
    info!("{}", scan_msg);

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
    is_chinese: bool,
) -> Result<ScanResults, AppError> {
    let scan_msg = if is_chinese {
        format!("📝 使用顺序处理扫描 {} 个文件", files.len())
    } else {
        format!("📝 Scanning {} files sequentially", files.len())
    };
    info!("{}", scan_msg);

    let mut stats = ScanStats::default();
    let mut file_results = Vec::new();

    for (i, (path, language)) in files.iter().enumerate() {
        if config.max_issues > 0 && stats.total_issues >= config.max_issues {
            let limit_msg = if is_chinese {
                format!("达到最大问题数限制 ({}), 停止扫描", config.max_issues)
            } else {
                format!(
                    "Reached maximum issue limit ({}), stopping scan",
                    config.max_issues
                )
            };
            info!("{}", limit_msg);
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
async fn output_results(
    results: &ScanResults,
    args: &ScanArgs,
    translation_manager: &TranslationManager,
) -> Result<(), AppError> {
    let output_content = match args.format.as_str() {
        "json" => format_json_output(results)?,
        "sarif" => format_sarif_output(results)?,
        "csv" => format_csv_output(results)?,
        "text" | _ => format_text_output(results, args.verbose, translation_manager),
    };

    if let Some(output_file) = &args.output {
        let is_chinese = translation_manager.is_enabled()
            && translation_manager.target_language() == &SupportedLanguage::Chinese;
        let write_error_msg = if is_chinese {
            format!("写入输出文件失败: {}", output_file)
        } else {
            format!("Failed to write output file: {}", output_file)
        };
        fs::write(output_file, &output_content).map_err(|e| AppError::IO(write_error_msg, e))?;
        let saved_msg = if is_chinese {
            format!("📄 结果已保存到: {}", output_file)
        } else {
            format!("📄 Results saved to: {}", output_file)
        };
        info!("{}", saved_msg);
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
fn format_text_output(
    results: &ScanResults,
    verbose: bool,
    translation_manager: &TranslationManager,
) -> String {
    let mut output = String::new();
    let is_chinese = translation_manager.is_enabled()
        && translation_manager.target_language() == &SupportedLanguage::Chinese;

    // Header
    let scan_complete_msg = if is_chinese {
        format!("🔍 {} 扫描完成\n", "AST-Grep".bright_blue())
    } else {
        format!("🔍 {} Scan Complete\n", "AST-Grep".bright_blue())
    };
    output.push_str(&scan_complete_msg);

    let files_scanned_msg = if is_chinese { "扫描了" } else { "Scanned" };
    let files_text = if is_chinese { "个文件" } else { "files" };
    output.push_str(&format!(
        "📂 {} {} {}\n",
        files_scanned_msg,
        results.files_scanned.to_string().bright_green(),
        files_text
    ));

    let issues_found_msg = if is_chinese { "发现" } else { "Found" };
    let issues_text = if is_chinese { "个问题" } else { "issues" };
    output.push_str(&format!(
        "⚠️  {} {} {}\n",
        issues_found_msg,
        results.total_issues.to_string().bright_yellow(),
        issues_text
    ));

    let duration_msg = if is_chinese { "耗时" } else { "Duration" };
    output.push_str(&format!(
        "⏱️  {} {} ms\n\n",
        duration_msg,
        results.scan_duration_ms.to_string().bright_blue()
    ));

    // Issues by severity
    if !results.issues_by_severity.is_empty() {
        let distribution_msg = if is_chinese {
            "📊 问题分布:"
        } else {
            "📊 Issue Distribution:"
        };
        output.push_str(&format!("{}\n", distribution_msg));
        for (severity, count) in &results.issues_by_severity {
            let severity_display = if is_chinese {
                match severity.as_str() {
                    "error" => "错误",
                    "warning" => "警告",
                    "info" => "信息",
                    "hint" => "提示",
                    _ => severity,
                }
            } else {
                severity
            };

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
                    "red" => severity_display.bright_red(),
                    "yellow" => severity_display.bright_yellow(),
                    "blue" => severity_display.bright_blue(),
                    "green" => severity_display.bright_green(),
                    _ => severity_display.bright_white(),
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
                let no_issues_msg = if is_chinese {
                    "  ✅ 未发现问题\n"
                } else {
                    "  ✅ No issues found\n"
                };
                output.push_str(no_issues_msg);
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
fn print_statistics(results: &ScanResults, translation_manager: &TranslationManager) {
    let is_chinese = translation_manager.is_enabled()
        && translation_manager.target_language() == &SupportedLanguage::Chinese;

    let stats_header = if is_chinese {
        "📈 详细统计:"
    } else {
        "📈 Detailed Statistics:"
    };
    println!("{}", stats_header);

    let files_label = if is_chinese {
        "文件总数"
    } else {
        "Total Files"
    };
    println!("  {}: {}", files_label, results.files_scanned);

    let issues_label = if is_chinese {
        "问题总数"
    } else {
        "Total Issues"
    };
    println!("  {}: {}", issues_label, results.total_issues);

    let duration_label = if is_chinese {
        "扫描耗时"
    } else {
        "Scan Duration"
    };
    println!("  {}: {} ms", duration_label, results.scan_duration_ms);

    if let Some(stats) = &results.language_stats {
        let supported_label = if is_chinese {
            "支持语言"
        } else {
            "Supported Languages"
        };
        println!("  {}: {}", supported_label, stats.total_languages);

        let enabled_label = if is_chinese {
            "启用语言"
        } else {
            "Enabled Languages"
        };
        println!("  {}: {}", enabled_label, stats.enabled_languages);
    }

    if !results.issues_by_language.is_empty() {
        let lang_dist_label = if is_chinese {
            "\n📊 按语言分布:"
        } else {
            "\n📊 Distribution by Language:"
        };
        println!("{}", lang_dist_label);
        let mut lang_issues: Vec<_> = results.issues_by_language.iter().collect();
        lang_issues.sort_by(|a, b| b.1.cmp(a.1));
        for (language, count) in lang_issues {
            println!("  {}: {}", language, count);
        }
    }

    if !results.issues_by_rule.is_empty() {
        let rule_dist_label = if is_chinese {
            "\n🔍 按规则分布 (前10):"
        } else {
            "\n🔍 Distribution by Rule (Top 10):"
        };
        println!("{}", rule_dist_label);
        let mut rule_issues: Vec<_> = results.issues_by_rule.iter().collect();
        rule_issues.sort_by(|a, b| b.1.cmp(a.1));
        for (rule, count) in rule_issues.iter().take(10) {
            println!("  {}: {}", rule, count);
        }
    }
}

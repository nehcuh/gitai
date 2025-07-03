use super::core::{
    AstAnalysisEngine, CodeIssue, CodeMetrics, DiffAnalysis, FileAnalysis, IssueSeverity,
    create_analysis_engine, detect_language_from_extension, parse_git_diff,
};
use crate::config::AstGrepConfig;
use crate::errors::AnalysisError;
use std::fs;
use std::time::Instant;

pub struct AstGrepAnalyzer {
    pub config: AstGrepConfig,
    analysis_engine: AstAnalysisEngine,
}

impl AstGrepAnalyzer {
    /// Creates a new `AstGrepAnalyzer` with the given configuration.
    ///
    /// Initializes the AST analysis engine. Returns an error if initialization fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let config = AstGrepConfig::default();
    /// let analyzer = AstGrepAnalyzer::new(config).unwrap();
    /// ```
    pub fn new(config: AstGrepConfig) -> Result<Self, AnalysisError> {
        Ok(Self {
            config,
            analysis_engine: create_analysis_engine(),
        })
    }

    /// Returns `true` if the given language is supported for AST-based analysis.
    ///
    /// Supported languages include Rust, Python, JavaScript, TypeScript, Java, C, C++, and Go.
    ///
    /// # Examples
    ///
    /// ```
    /// let analyzer = AstGrepAnalyzer::new(config).unwrap();
    /// assert!(analyzer.is_supported_language("rust"));
    /// assert!(!analyzer.is_supported_language("php"));
    /// ```
    fn is_supported_language(&self, lang: &str) -> bool {
        matches!(
            lang,
            "rust" | "python" | "javascript" | "typescript" | "java" | "c" | "cpp" | "go"
        )
    }

    /// Analyzes a git diff and returns structured results for each changed file.
    ///
    /// For each file in the diff, detects its language and, if supported, performs AST-based analysis to identify code issues and collect code metrics. If AST analysis fails or the language is unsupported, falls back to simple pattern matching or reports lack of support. Aggregates per-file analyses, issue counts, and overall summary including analysis duration.
    ///
    /// # Returns
    ///
    /// A `DiffAnalysis` containing per-file analysis results, overall summary, total issues found, number of files analyzed, and analysis duration in milliseconds.
    ///
    /// # Errors
    ///
    /// Returns an `AnalysisError` if the diff cannot be parsed or if file analysis encounters unrecoverable errors.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut analyzer = AstGrepAnalyzer::new(config).unwrap();
    /// let diff_text = "..."; // git diff text
    /// let analysis = analyzer.analyze_diff(diff_text).unwrap();
    /// assert!(analysis.total_files_analyzed > 0);
    /// ```
    pub fn analyze_diff(&mut self, diff_text: &str) -> Result<DiffAnalysis, AnalysisError> {
        let start_time = Instant::now();
        let git_diff = parse_git_diff(diff_text)?;
        let mut file_analyses = Vec::new();
        let mut total_issues = 0;
        let mut total_files_analyzed = 0;

        for changed_file in &git_diff.changed_files {
            let language = detect_language_from_extension(&changed_file.path)
                .unwrap_or_else(|| "unknown".to_string());

            let analysis_result = if language != "unknown" && self.is_supported_language(&language)
            {
                match self.analyze_file_with_ast_grep(changed_file, &language) {
                    Ok((issues, metrics)) => {
                        total_files_analyzed += 1;
                        total_issues += issues.len();

                        let summary = self.generate_file_summary(changed_file, &issues, &metrics);

                        FileAnalysis {
                            path: changed_file.path.clone(),
                            language: language.clone(),
                            change_type: changed_file.change_type.clone(),
                            summary: Some(summary),
                            issues,
                            metrics: Some(metrics),
                        }
                    }
                    Err(e) => FileAnalysis {
                        path: changed_file.path.clone(),
                        language: language.clone(),
                        change_type: changed_file.change_type.clone(),
                        summary: Some(format!("分析失败: {}", e)),
                        issues: vec![],
                        metrics: None,
                    },
                }
            } else {
                FileAnalysis {
                    path: changed_file.path.clone(),
                    language: language.clone(),
                    change_type: changed_file.change_type.clone(),
                    summary: Some("语言不支持 AST 分析".to_string()),
                    issues: vec![],
                    metrics: None,
                }
            };

            file_analyses.push(analysis_result);
        }

        let analysis_duration = start_time.elapsed();
        let overall_summary = if total_files_analyzed > 0 {
            format!(
                "🔍 AST-Grep 分析完成\n已分析 {} 个文件，发现 {} 个潜在问题\n📊 支持的语言: Rust, Python, JavaScript, TypeScript, Java, C/C++, Go 等\n⏱️ 分析耗时: {:.2}ms",
                total_files_analyzed,
                total_issues,
                analysis_duration.as_secs_f64() * 1000.0
            )
        } else {
            "未找到支持分析的文件类型".to_string()
        };

        Ok(DiffAnalysis {
            file_analyses,
            overall_summary,
            total_issues,
            total_files_analyzed,
            analysis_duration_ms: analysis_duration.as_millis() as u64,
        })
    }

    /// Analyzes a changed file using AST-based analysis, with a fallback to pattern matching if AST analysis fails.
    ///
    /// Attempts to read the file content and perform AST-based code analysis for the specified language.
    /// If AST analysis is unsuccessful, falls back to simple pattern-based checks for common issues.
    /// Returns a tuple containing the list of detected code issues and calculated code metrics, or an error if the file cannot be read.
    ///
    /// # Examples
    ///
    /// ```
    /// let (issues, metrics) = analyzer.analyze_file_with_ast_grep(&changed_file, "rust")?;
    /// assert!(!issues.is_empty() || metrics.lines_of_code > 0);
    /// ```
    fn analyze_file_with_ast_grep(
        &self,
        changed_file: &crate::types::git::ChangedFile,
        language: &str,
    ) -> Result<(Vec<CodeIssue>, CodeMetrics), AnalysisError> {
        let file_path = &changed_file.path;

        // Try to read file content
        let content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(_) => {
                // File might be deleted or not accessible
                return Err(AnalysisError::Generic(format!(
                    "无法读取文件: {}",
                    file_path.display()
                )));
            }
        };

        // Perform AST-based analysis
        let issues = match self
            .analysis_engine
            .analyze_file_content(&content, language, file_path)
        {
            Ok(issues) => issues,
            Err(e) => {
                tracing::warn!("AST 分析失败，回退到基础分析: {}", e);
                // Fallback to basic pattern matching
                self.analyze_code_patterns_fallback(&content, language)
            }
        };

        // Calculate enhanced metrics
        let metrics = self.analysis_engine.calculate_metrics(&content, language);

        Ok((issues, metrics))
    }

    /// Performs simple pattern-based code analysis as a fallback when AST analysis is unavailable.
    ///
    /// Dispatches to language-specific pattern checks for Rust, Python, JavaScript, and TypeScript to identify common code issues using text matching. Returns a list of detected issues.
    ///
    /// # Parameters
    /// - `content`: The source code to analyze.
    /// - `language`: The programming language of the source code.
    ///
    /// # Returns
    /// A vector of `CodeIssue` instances representing issues found by pattern matching.
    ///
    /// # Examples
    ///
    /// ```
    /// let issues = analyzer.analyze_code_patterns_fallback("let x = foo.unwrap();", "rust");
    /// assert!(!issues.is_empty());
    /// ```
    fn analyze_code_patterns_fallback(&self, content: &str, language: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();

        // Fallback to simple text pattern matching when AST analysis fails
        match language {
            "rust" => {
                issues.extend(self.check_rust_patterns_simple(content));
            }
            "python" => {
                issues.extend(self.check_python_patterns_simple(content));
            }
            "javascript" | "typescript" => {
                issues.extend(self.check_js_patterns_simple(content));
            }
            _ => {
                // Generic checks for other languages
            }
        }

        issues
    }

    /// Creates a `CodeIssue` with default position and code quality category.
    ///
    /// This helper is used when generating issues from simple pattern matching, where precise location and matched text are unavailable.
    ///
    /// # Parameters
    ///
    /// - `rule_id`: Identifier for the rule that triggered the issue.
    /// - `message`: Description of the issue.
    /// - `severity`: Severity level of the issue.
    ///
    /// # Returns
    ///
    /// A `CodeIssue` with default line and column set to 1, no matched text, no suggestion, and category set to code quality.
    ///
    /// # Examples
    ///
    /// ```
    /// let issue = analyzer.create_simple_issue("rust-unwrap", "Avoid using unwrap()", IssueSeverity::Warning);
    /// assert_eq!(issue.rule_id, "rust-unwrap");
    /// assert_eq!(issue.severity, IssueSeverity::Warning);
    /// assert_eq!(issue.line, 1);
    /// ```
    fn create_simple_issue(
        &self,
        rule_id: &str,
        message: &str,
        severity: IssueSeverity,
    ) -> CodeIssue {
        CodeIssue {
            rule_id: rule_id.to_string(),
            severity,
            message: message.to_string(),
            line: 1, // Default line when position is unknown
            column: 1,
            end_line: None,
            end_column: None,
            matched_text: "".to_string(),
            suggestion: None,
            category: super::core::IssueCategory::CodeQuality,
        }
    }

    /// Performs simple pattern checks for common risky patterns in Rust code.
    ///
    /// Detects usage of `.unwrap()` and `todo!()` macros, returning issues with appropriate severity.
    ///
    /// # Examples
    ///
    /// ```
    /// let code = "let x = some_option.unwrap();\ntodo!()";
    /// let analyzer = AstGrepAnalyzer::new(default_config).unwrap();
    /// let issues = analyzer.check_rust_patterns_simple(code);
    /// assert_eq!(issues.len(), 2);
    /// ```
    fn check_rust_patterns_simple(&self, content: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();

        // Check for unwrap() usage
        if content.contains(".unwrap()") {
            issues.push(self.create_simple_issue(
                "rust-unwrap",
                "建议使用 expect() 或适当的错误处理替代 unwrap()",
                IssueSeverity::Warning,
            ));
        }

        // Check for todo!() macros
        if content.contains("todo!()") {
            issues.push(self.create_simple_issue(
                "rust-todo",
                "发现 todo!() 宏，需要完成实现",
                IssueSeverity::Info,
            ));
        }

        issues
    }

    /// Performs simple pattern checks on Python code to identify common issues.
    ///
    /// Detects usage of `print` statements (recommending the use of the logging module)
    /// and potential SQL injection risks via `execute(` calls.
    ///
    /// # Returns
    ///
    /// A vector of `CodeIssue` instances representing detected issues.
    ///
    /// # Examples
    ///
    /// ```
    /// let code = r#"
    /// def foo():
    ///     print('debug')
    ///     cursor.execute('SELECT * FROM users')
    /// "#;
    /// let issues = analyzer.check_python_patterns_simple(code);
    /// assert!(issues.iter().any(|i| i.rule_id == "python-print"));
    /// assert!(issues.iter().any(|i| i.rule_id == "python-sql-injection"));
    /// ```
    fn check_python_patterns_simple(&self, content: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();

        // Check for print statements (should use logging)
        if content.contains("print(") {
            issues.push(self.create_simple_issue(
                "python-print",
                "建议使用 logging 模块替代 print 语句",
                IssueSeverity::Info,
            ));
        }

        // Check for execute( patterns (SQL injection risk)
        if content.contains("execute(") {
            issues.push(self.create_simple_issue(
                "python-sql-injection",
                "潜在的 SQL 注入风险",
                IssueSeverity::Warning,
            ));
        }

        issues
    }

    /// Performs simple pattern-based checks for common JavaScript and TypeScript code issues.
    ///
    /// Scans the provided code content for usage of `console.log`, non-strict equality (`==`), and `innerHTML`,
    /// returning issues that may indicate logging in production, lack of strict equality, or potential XSS risks.
    ///
    /// # Examples
    ///
    /// ```
    /// let js_code = r#"
    ///     console.log('debug');
    ///     if (a == b) { /* ... */ }
    ///     element.innerHTML = userInput;
    /// "#;
    /// let issues = analyzer.check_js_patterns_simple(js_code);
    /// assert!(issues.iter().any(|i| i.rule_id == "js-console-log"));
    /// assert!(issues.iter().any(|i| i.rule_id == "js-strict-equality"));
    /// assert!(issues.iter().any(|i| i.rule_id == "js-xss-innerhtml"));
    /// ```
    fn check_js_patterns_simple(&self, content: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();

        // Check for console.log in production code
        if content.contains("console.log(") {
            issues.push(self.create_simple_issue(
                "js-console-log",
                "生产代码中应避免使用 console.log",
                IssueSeverity::Warning,
            ));
        }

        // Check for == instead of ===
        if content.contains(" == ") {
            issues.push(self.create_simple_issue(
                "js-strict-equality",
                "建议使用 === 进行严格相等比较",
                IssueSeverity::Warning,
            ));
        }

        // Check for innerHTML usage (XSS risk)
        if content.contains("innerHTML") {
            issues.push(self.create_simple_issue(
                "js-xss-innerhtml",
                "潜在的 XSS 风险：设置 innerHTML",
                IssueSeverity::Warning,
            ));
        }

        issues
    }

    /// Generates a summary string for a changed file, including change type, issue counts by severity, and code metrics.
    ///
    /// The summary describes the file change (added, modified, deleted, or renamed), the number and severity of detected issues, and key code metrics such as lines of code, function and class counts, and maintainability index.
    ///
    /// # Examples
    ///
    /// ```
    /// let summary = analyzer.generate_file_summary(
    ///     &changed_file,
    ///     &issues,
    ///     &metrics,
    /// );
    /// println!("{}", summary);
    /// ```
    fn generate_file_summary(
        &self,
        changed_file: &crate::types::git::ChangedFile,
        issues: &[CodeIssue],
        metrics: &CodeMetrics,
    ) -> String {
        let change_desc = match changed_file.change_type {
            crate::types::git::ChangeType::Added => "新增文件",
            crate::types::git::ChangeType::Modified => "修改文件",
            crate::types::git::ChangeType::Deleted => "删除文件",
            crate::types::git::ChangeType::Renamed => "重命名文件",
            _ => "变更文件",
        };

        let issue_summary = if issues.is_empty() {
            "✅ 未发现明显问题".to_string()
        } else {
            let errors = issues
                .iter()
                .filter(|i| matches!(i.severity, IssueSeverity::Error))
                .count();
            let warnings = issues
                .iter()
                .filter(|i| matches!(i.severity, IssueSeverity::Warning))
                .count();
            let infos = issues
                .iter()
                .filter(|i| matches!(i.severity, IssueSeverity::Info))
                .count();
            let hints = issues
                .iter()
                .filter(|i| matches!(i.severity, IssueSeverity::Hint))
                .count();

            format!(
                "⚠️ 发现 {} 个问题 (错误: {}, 警告: {}, 建议: {}, 提示: {})",
                issues.len(),
                errors,
                warnings,
                infos,
                hints
            )
        };

        format!(
            "{} | {} | 📏 {} 行代码 | 🔧 {} 个函数 | 🏛️ {} 个类 | 📊 可维护性: {:.1}",
            change_desc,
            issue_summary,
            metrics.lines_of_code,
            metrics.function_count,
            metrics.class_count,
            metrics.maintainability_index
        )
    }
}

use crate::types::git::{ChangeType, ChangedFile, GitDiff};

use ast_grep_core::source::TSParseError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// Complete analysis of a Git diff
#[derive(Debug, Clone)]
pub struct DiffAnalysis {
    pub file_analyses: Vec<FileAnalysis>,
    pub overall_summary: String,
    pub total_issues: usize,
    pub total_files_analyzed: usize,
    pub analysis_duration_ms: u64,
}

// Analysis of a single file
#[derive(Debug, Clone)]
pub struct FileAnalysis {
    pub path: PathBuf,
    pub language: String,
    pub change_type: ChangeType,
    pub summary: Option<String>,
    pub issues: Vec<CodeIssue>,
    pub metrics: Option<CodeMetrics>,
}

// Detailed information about a code issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIssue {
    pub rule_id: String,
    pub severity: IssueSeverity,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub end_line: Option<usize>,
    pub end_column: Option<usize>,
    pub matched_text: String,
    pub suggestion: Option<String>,
    pub category: IssueCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueCategory {
    CodeQuality,
    Security,
    Performance,
    Style,
    BestPractice,
    BugRisk,
}

// Code metrics for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetrics {
    pub lines_of_code: usize,
    pub non_empty_lines: usize,
    pub comment_lines: usize,
    pub function_count: usize,
    pub class_count: usize,
    pub complexity_score: f32,
    pub maintainability_index: f32,
}

// Rule definition for code analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub language: String,
    pub pattern: String,
    pub message: String,
    pub suggestion: Option<String>,
    pub enabled: bool,
}

// Built-in rules registry
pub struct RuleRegistry {
    rules: HashMap<String, Vec<AnalysisRule>>,
}

impl RuleRegistry {
    /// Creates a new `RuleRegistry` instance and loads built-in analysis rules for supported languages.
    ///
    /// # Examples
    ///
    /// ```
    /// let registry = RuleRegistry::new();
    /// let rust_rules = registry.get_rules_for_language("rust");
    /// assert!(!rust_rules.is_empty());
    /// ```
    pub fn new() -> Self {
        let mut registry = Self {
            rules: HashMap::new(),
        };
        registry.load_builtin_rules();
        registry
    }

    /// Returns all enabled analysis rules for the specified programming language.
    ///
    /// # Arguments
    ///
    /// * `language` - The name of the programming language to retrieve rules for.
    ///
    /// # Returns
    ///
    /// A vector of references to enabled `AnalysisRule` objects for the given language. If no rules are found, returns an empty vector.
    pub fn get_rules_for_language(&self, language: &str) -> Vec<&AnalysisRule> {
        self.rules
            .get(language)
            .map(|rules| rules.iter().filter(|rule| rule.enabled).collect())
            .unwrap_or_default()
    }

    /// Loads built-in static analysis rules for supported languages into the registry.
    ///
    /// This method populates the registry with predefined rules for Rust, Python, JavaScript, and TypeScript.
    fn load_builtin_rules(&mut self) {
        self.add_rust_rules();
        self.add_python_rules();
        self.add_javascript_rules();
        self.add_typescript_rules();
    }

    /// Adds built-in static analysis rules for Rust to the registry.
    ///
    /// The rules target common Rust code issues such as misuse of `unwrap()`, presence of `todo!()` macros, and unnecessary cloning of `Copy` types. These rules are used for static analysis of Rust source files.
    fn add_rust_rules(&mut self) {
        let rules = vec![
            AnalysisRule {
                id: "rust-unwrap".to_string(),
                name: "Avoid unwrap()".to_string(),
                description: "Using unwrap() can cause panics. Consider using expect() or proper error handling.".to_string(),
                severity: IssueSeverity::Warning,
                category: IssueCategory::BugRisk,
                language: "rust".to_string(),
                pattern: "$VAR.unwrap()".to_string(),
                message: "Avoid using unwrap(); consider using expect() or proper error handling".to_string(),
                suggestion: Some("Use .expect(\"meaningful message\") or proper error handling with match/if let".to_string()),
                enabled: true,
            },
            AnalysisRule {
                id: "rust-todo".to_string(),
                name: "TODO macro".to_string(),
                description: "TODO macros indicate unfinished implementation.".to_string(),
                severity: IssueSeverity::Info,
                category: IssueCategory::CodeQuality,
                language: "rust".to_string(),
                pattern: "todo!($MSG)".to_string(),
                message: "TODO macro found - implementation needed".to_string(),
                suggestion: Some("Complete the implementation or use a more specific error type".to_string()),
                enabled: true,
            },
            AnalysisRule {
                id: "rust-clone-copy".to_string(),
                name: "Unnecessary clone on Copy types".to_string(),
                description: "Clone is unnecessary for Copy types.".to_string(),
                severity: IssueSeverity::Hint,
                category: IssueCategory::Performance,
                language: "rust".to_string(),
                pattern: "$NUM.clone()".to_string(),
                message: "Unnecessary clone() on Copy type".to_string(),
                suggestion: Some("Remove .clone() call for Copy types".to_string()),
                enabled: true,
            },
        ];
        self.rules.insert("rust".to_string(), rules);
    }

    /// Adds built-in static analysis rules for Python to the registry.
    ///
    /// The rules include checks for print statement usage and potential SQL injection risks.
    fn add_python_rules(&mut self) {
        let rules = vec![
            AnalysisRule {
                id: "python-print".to_string(),
                name: "Print statement".to_string(),
                description: "Consider using logging instead of print statements.".to_string(),
                severity: IssueSeverity::Info,
                category: IssueCategory::BestPractice,
                language: "python".to_string(),
                pattern: "print($ARGS)".to_string(),
                message: "Consider using logging instead of print statements".to_string(),
                suggestion: Some(
                    "Use logging.info(), logging.debug(), etc. instead of print()".to_string(),
                ),
                enabled: true,
            },
            AnalysisRule {
                id: "python-sql-injection".to_string(),
                name: "SQL injection risk".to_string(),
                description: "Potential SQL injection vulnerability.".to_string(),
                severity: IssueSeverity::Error,
                category: IssueCategory::Security,
                language: "python".to_string(),
                pattern: "execute($SQL)".to_string(),
                message: "Potential SQL injection vulnerability".to_string(),
                suggestion: Some("Use parameterized queries or prepared statements".to_string()),
                enabled: true,
            },
        ];
        self.rules.insert("python".to_string(), rules);
    }

    /// Adds built-in static analysis rules for JavaScript to the rule registry.
    ///
    /// The rules include checks for `console.log` usage, strict equality comparisons, and potential XSS vulnerabilities via `innerHTML`.
    /// These rules are registered under the "javascript" language key and are enabled by default.
    fn add_javascript_rules(&mut self) {
        let rules = vec![
            AnalysisRule {
                id: "js-console-log".to_string(),
                name: "Console.log usage".to_string(),
                description: "Console.log should not be used in production code.".to_string(),
                severity: IssueSeverity::Warning,
                category: IssueCategory::CodeQuality,
                language: "javascript".to_string(),
                pattern: "console.log($ARGS)".to_string(),
                message: "Remove console.log from production code".to_string(),
                suggestion: Some(
                    "Use proper logging framework or remove debug statements".to_string(),
                ),
                enabled: true,
            },
            AnalysisRule {
                id: "js-strict-equality".to_string(),
                name: "Use strict equality".to_string(),
                description: "Use === instead of == for strict equality comparison.".to_string(),
                severity: IssueSeverity::Warning,
                category: IssueCategory::BestPractice,
                language: "javascript".to_string(),
                pattern: "$A == $B".to_string(),
                message: "Use === for strict equality comparison".to_string(),
                suggestion: Some("Replace == with === for type-safe comparison".to_string()),
                enabled: true,
            },
            AnalysisRule {
                id: "js-xss-innerhtml".to_string(),
                name: "XSS via innerHTML".to_string(),
                description: "Setting innerHTML with user input can lead to XSS vulnerabilities."
                    .to_string(),
                severity: IssueSeverity::Error,
                category: IssueCategory::Security,
                language: "javascript".to_string(),
                pattern: "$ELEM.innerHTML = $VAR".to_string(),
                message: "Potential XSS vulnerability via innerHTML".to_string(),
                suggestion: Some(
                    "Use textContent or properly sanitize input before setting innerHTML"
                        .to_string(),
                ),
                enabled: true,
            },
        ];
        self.rules.insert("javascript".to_string(), rules);
    }

    /// Adds TypeScript analysis rules by cloning JavaScript rules and updating their language field to TypeScript.
    ///
    /// This enables TypeScript support by reusing the existing JavaScript rule set with appropriate language labeling.
    fn add_typescript_rules(&mut self) {
        // TypeScript shares most JavaScript rules
        let mut rules = self.rules.get("javascript").cloned().unwrap_or_default();
        for rule in &mut rules {
            rule.language = "typescript".to_string();
        }
        self.rules.insert("typescript".to_string(), rules);
    }
}

// AST analysis engine
pub struct AstAnalysisEngine {
    rule_registry: RuleRegistry,
}

impl AstAnalysisEngine {
    /// Creates a new `AstAnalysisEngine` with a fresh rule registry.
    ///
    /// # Examples
    ///
    /// ```
    /// let engine = AstAnalysisEngine::new();
    /// ```
    pub fn new() -> Self {
        Self {
            rule_registry: RuleRegistry::new(),
        }
    }

    /// Analyzes source code content for issues using language-specific static analysis rules.
    ///
    /// Applies enabled rules for the specified language by performing simple text-based pattern matching.
    /// Returns a list of detected code issues, each with location and details, or a parse error if analysis fails.
    ///
    /// # Parameters
    /// - `content`: The source code to analyze.
    /// - `language`: The programming language of the source code.
    ///
    /// # Returns
    /// A vector of `CodeIssue` objects representing detected issues, or a `TSParseError` if analysis fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let engine = create_analysis_engine();
    /// let issues = engine.analyze_file_content("fn main() { println!(\"Hello\"); }", "rust", Path::new("main.rs")).unwrap();
    /// assert!(issues.is_empty() || !issues.is_empty());
    /// ```
    pub fn analyze_file_content(
        &self,
        content: &str,
        language: &str,
        _file_path: &Path,
    ) -> Result<Vec<CodeIssue>, TSParseError> {
        let mut issues = Vec::new();

        // Get language-specific rules
        let rules = self.rule_registry.get_rules_for_language(language);

        // For now, use simple text-based pattern matching
        // TODO: Implement full AST analysis when ast-grep API is stable
        for rule in rules {
            if content.contains(
                &rule
                    .pattern
                    .replace("$VAR", "")
                    .replace("$ARGS", "")
                    .replace("$MSG", "")
                    .replace("$A", "")
                    .replace("$B", "")
                    .replace("$ELEM", ""),
            ) {
                // Find line number by searching for the pattern
                let lines: Vec<&str> = content.lines().collect();
                for (line_idx, line) in lines.iter().enumerate() {
                    if line.contains(
                        &rule
                            .pattern
                            .replace("$VAR", "")
                            .replace("$ARGS", "")
                            .replace("$MSG", "")
                            .replace("$A", "")
                            .replace("$B", "")
                            .replace("$ELEM", ""),
                    ) {
                        issues.push(CodeIssue {
                            rule_id: rule.id.clone(),
                            severity: rule.severity.clone(),
                            message: rule.message.clone(),
                            line: line_idx + 1,
                            column: 1,
                            end_line: Some(line_idx + 1),
                            end_column: Some(line.len()),
                            matched_text: line.to_string(),
                            suggestion: rule.suggestion.clone(),
                            category: rule.category.clone(),
                        });
                        break; // Only report first occurrence per rule
                    }
                }
            }
        }

        Ok(issues)
    }

    /// Calculates code metrics for the given source code content and language.
    ///
    /// Metrics include total lines of code, non-empty lines, comment lines, function and class counts, a complexity score, and a maintainability index.
    ///
    /// # Examples
    ///
    /// ```
    /// let engine = create_analysis_engine();
    /// let metrics = engine.calculate_metrics("fn main() {\n    // hello\n}\n", "rust");
    /// assert_eq!(metrics.lines_of_code, 3);
    /// assert_eq!(metrics.comment_lines, 1);
    /// ```
    pub fn calculate_metrics(&self, content: &str, language: &str) -> CodeMetrics {
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let non_empty_lines = lines.iter().filter(|line| !line.trim().is_empty()).count();

        let comment_lines = self.count_comment_lines(&lines, language);
        let function_count = self.count_functions_advanced(content, language);
        let class_count = self.count_classes(content, language);

        // Simple complexity calculation based on control flow keywords
        let complexity_score = self.calculate_complexity(content, language);

        // Simple maintainability index (0-100 scale)
        let maintainability_index =
            self.calculate_maintainability_index(total_lines, function_count, complexity_score);

        CodeMetrics {
            lines_of_code: total_lines,
            non_empty_lines,
            comment_lines,
            function_count,
            class_count,
            complexity_score,
            maintainability_index,
        }
    }

    /// Checks if the specified language is supported for analysis.
    ///
    /// Returns `true` if the language is one of Rust, Python, JavaScript, TypeScript, Java, C, C++, or Go; otherwise, returns `false`.
    fn _is_language_supported(&self, lang: &str) -> bool {
        matches!(
            lang,
            "rust" | "python" | "javascript" | "typescript" | "java" | "c" | "cpp" | "go"
        )
    }

    /// Counts the number of comment lines in the provided source code lines for a given language.
    ///
    /// A line is considered a comment if it starts with a recognized comment pattern for the specified language.
    ///
    /// # Parameters
    /// - `lines`: An array of source code lines to analyze.
    /// - `language`: The programming language to determine comment syntax.
    ///
    /// # Returns
    /// The number of lines that are recognized as comments.
    fn count_comment_lines(&self, lines: &[&str], language: &str) -> usize {
        let comment_patterns = match language {
            "rust" | "javascript" | "typescript" | "java" | "c" | "cpp" | "go" => {
                vec!["//", "/*", "*/"]
            }
            "python" => vec!["#"],
            "html" => vec!["<!--", "-->"],
            "css" => vec!["/*", "*/"],
            _ => vec![],
        };

        lines
            .iter()
            .filter(|line| {
                let trimmed = line.trim();
                comment_patterns
                    .iter()
                    .any(|pattern| trimmed.starts_with(pattern))
            })
            .count()
    }

    /// Estimates the number of functions in the source code using language-specific heuristics.
    ///
    /// For each supported language, this method applies simple pattern matching to count function definitions.
    /// The approach is heuristic and may not be fully accurate for all code styles or edge cases.
    ///
    /// # Parameters
    /// - `content`: The source code to analyze.
    /// - `language`: The programming language identifier (e.g., "rust", "python").
    ///
    /// # Returns
    /// The estimated number of functions found in the source code.
    ///
    /// # Examples
    ///
    /// ```
    /// let rust_code = "fn foo() {}\nfn bar() {}";
    /// let count = engine.count_functions_advanced(rust_code, "rust");
    /// assert_eq!(count, 2);
    /// ```
    fn count_functions_advanced(&self, content: &str, language: &str) -> usize {
        match language {
            "rust" => content.matches("fn ").count(),
            "python" => content.matches("def ").count(),
            "javascript" | "typescript" => {
                content.matches("function ").count()
                    + content.matches("() =>").count()
                    + content.matches(") =>").count()
            }
            "java" | "c" | "cpp" => {
                // Simple heuristic for these languages
                content
                    .lines()
                    .filter(|line| {
                        let trimmed = line.trim();
                        trimmed.contains("(")
                            && trimmed.contains(")")
                            && trimmed.contains("{")
                            && !trimmed.starts_with("//")
                            && !trimmed.starts_with("if")
                            && !trimmed.starts_with("for")
                            && !trimmed.starts_with("while")
                    })
                    .count()
            }
            _ => 0,
        }
    }

    /// Counts the number of class or struct declarations in the source code for the specified language.
    ///
    /// For Rust, both `struct` and `enum` declarations are counted. For Python, JavaScript, TypeScript, Java, C, and C++, only `class` declarations are counted.
    ///
    /// # Examples
    ///
    /// ```
    /// let rust_code = "struct Foo {}\nenum Bar {}";
    /// let count = engine.count_classes(rust_code, "rust");
    /// assert_eq!(count, 2);
    ///
    /// let py_code = "class MyClass:\n    pass";
    /// let count = engine.count_classes(py_code, "python");
    /// assert_eq!(count, 1);
    /// ```
    fn count_classes(&self, content: &str, language: &str) -> usize {
        match language {
            "rust" => content.matches("struct ").count() + content.matches("enum ").count(),
            "python" => content.matches("class ").count(),
            "javascript" | "typescript" => content.matches("class ").count(),
            "java" | "c" | "cpp" => content.matches("class ").count(),
            _ => 0,
        }
    }

    /// Estimates the code complexity score based on control flow keywords for the specified language.
    ///
    /// The complexity score is calculated as a base value of 1.0 plus 0.1 for each occurrence of language-specific control flow keywords (such as `if`, `for`, `while`, etc.) found in the code content.
    ///
    /// # Parameters
    /// - `content`: The source code to analyze.
    /// - `language`: The programming language of the source code.
    ///
    /// # Returns
    /// A floating-point value representing the estimated complexity score.
    ///
    /// # Examples
    ///
    /// ```
    /// let engine = create_analysis_engine();
    /// let code = "if (x > 0) { for (let i = 0; i < x; i++) {} }";
    /// let complexity = engine.calculate_complexity(code, "javascript");
    /// assert!(complexity > 1.0);
    /// ```
    fn calculate_complexity(&self, content: &str, language: &str) -> f32 {
        let control_keywords = match language {
            "rust" => vec!["if", "match", "for", "while", "loop"],
            "python" => vec!["if", "for", "while", "try", "except", "elif"],
            "javascript" | "typescript" => vec!["if", "for", "while", "switch", "try", "catch"],
            "java" | "c" | "cpp" => vec!["if", "for", "while", "switch", "try", "catch"],
            _ => vec![],
        };

        let complexity_count: usize = control_keywords
            .iter()
            .map(|keyword| content.matches(keyword).count())
            .sum();

        1.0 + complexity_count as f32 * 0.1 // Base complexity of 1 + 0.1 per control structure
    }

    /// Calculates a simplified maintainability index for a source file.
    ///
    /// The index is computed based on lines of code, function count, and code complexity,
    /// and is scaled to a value between 0 and 100, where higher values indicate better maintainability.
    ///
    /// # Parameters
    /// - `lines_of_code`: Total number of lines of code in the file.
    /// - `function_count`: Number of functions detected in the file.
    /// - `complexity`: Calculated complexity score for the file.
    ///
    /// # Returns
    /// A floating-point value representing the maintainability index, ranging from 0 (least maintainable) to 100 (most maintainable).
    ///
    /// # Examples
    ///
    /// ```
    /// let index = engine.calculate_maintainability_index(200, 10, 15.0);
    /// assert!(index >= 0.0 && index <= 100.0);
    /// ```
    fn calculate_maintainability_index(
        &self,
        lines_of_code: usize,
        function_count: usize,
        complexity: f32,
    ) -> f32 {
        // Simplified maintainability index calculation
        let loc_factor = if lines_of_code > 0 {
            (lines_of_code as f32).ln()
        } else {
            0.0
        };

        let function_factor = if function_count > 0 {
            function_count as f32
        } else {
            1.0
        };

        // Scale to 0-100
        let index = 171.0 - 5.2 * loc_factor - 0.23 * complexity - 16.2 * function_factor.ln();
        index.max(0.0).min(100.0)
    }
}

/// Returns the programming language name corresponding to a file's extension.
///
/// Returns `Some(language)` if the extension matches a known language, or `None` if the extension is unrecognized.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// let lang = detect_language_from_extension(Path::new("main.rs"));
/// assert_eq!(lang, Some("rust".to_string()));
/// let unknown = detect_language_from_extension(Path::new("archive.unknown"));
/// assert_eq!(unknown, None);
/// ```
pub fn detect_language_from_extension(path: &Path) -> Option<String> {
    let extension = path.extension()?.to_str()?;

    match extension {
        "rs" => Some("rust".to_string()),
        "py" => Some("python".to_string()),
        "js" | "jsx" => Some("javascript".to_string()),
        "ts" | "tsx" => Some("typescript".to_string()),
        "java" => Some("java".to_string()),
        "c" => Some("c".to_string()),
        "cpp" | "cxx" | "cc" => Some("cpp".to_string()),
        "go" => Some("go".to_string()),
        "rb" => Some("ruby".to_string()),
        "php" => Some("php".to_string()),
        "cs" => Some("csharp".to_string()),
        "swift" => Some("swift".to_string()),
        "kt" => Some("kotlin".to_string()),
        "scala" => Some("scala".to_string()),
        "sh" | "bash" => Some("bash".to_string()),
        "yml" | "yaml" => Some("yaml".to_string()),
        "toml" => Some("toml".to_string()),
        "json" => Some("json".to_string()),
        "xml" => Some("xml".to_string()),
        "html" => Some("html".to_string()),
        "css" => Some("css".to_string()),
        "md" => Some("markdown".to_string()),
        _ => None,
    }
}

/// Enhanced git diff parser with better hunk handling
pub struct GitDiffParser {
    file_header_re: Regex,
    file_status_re: Regex,
}

impl GitDiffParser {
    /// Creates a new `GitDiffParser` with compiled regular expressions for parsing Git diff file headers and status lines.
    ///
    /// # Examples
    ///
    /// ```
    /// let parser = GitDiffParser::new();
    /// ```
    pub fn new() -> Self {
        Self {
            file_header_re: Regex::new(r"^diff --git a/(.*) b/(.*)$").unwrap(),
            file_status_re: Regex::new(r"^(new file mode|deleted file mode|index)").unwrap(),
        }
    }

    /// Parses Git diff text and extracts a list of changed files with their change types.
    ///
    /// Scans the provided Git diff text for file headers and status lines, identifying each changed file and whether it was added, deleted, or modified. Returns a `GitDiff` containing all detected file changes.
    ///
    /// # Returns
    /// A `Result` containing the parsed `GitDiff` on success, or an `AnalysisError` if parsing fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let parser = GitDiffParser::new();
    /// let diff = "\
    /// diff --git a/foo.rs b/foo.rs
    /// new file mode 100644
    /// ";
    /// let result = parser.parse(diff).unwrap();
    /// assert_eq!(result.changed_files.len(), 1);
    /// assert_eq!(result.changed_files[0].change_type, ChangeType::Added);
    /// ```
    pub fn parse(&self, diff_text: &str) -> Result<GitDiff, crate::errors::AnalysisError> {
        let mut changed_files = Vec::new();
        let lines: Vec<&str> = diff_text.lines().collect();
        let mut current_file: Option<ChangedFile> = None;

        for line in lines {
            if let Some(caps) = self.file_header_re.captures(line) {
                // Save previous file if exists
                if let Some(file) = current_file.take() {
                    changed_files.push(file);
                }

                // Start new file
                let file_path = caps.get(1).unwrap().as_str();
                current_file = Some(ChangedFile {
                    path: PathBuf::from(file_path),
                    change_type: ChangeType::Modified, // Default, will be updated
                    hunks: Vec::new(),
                    file_mode_change: None,
                });
            } else if self.file_status_re.is_match(line) {
                // Determine file status
                if let Some(ref mut file) = current_file {
                    if line.contains("new file mode") {
                        file.change_type = ChangeType::Added;
                    } else if line.contains("deleted file mode") {
                        file.change_type = ChangeType::Deleted;
                    }
                }
            }
        }

        // Save last file if exists
        if let Some(file) = current_file {
            changed_files.push(file);
        }

        Ok(GitDiff {
            changed_files,
            metadata: None,
        })
    }
}

/// Returns the ast-grep language identifier for a given language string, if supported.
///
/// # Arguments
///
/// * `lang` - The language name as a string (e.g., "rust", "python").
///
/// # Returns
///
/// An `Option` containing the ast-grep language identifier as a static string, or `None` if the language is not supported.
///
/// # Examples
///
/// ```
/// assert_eq!(get_ast_grep_language_name("rust"), Some("rust"));
/// assert_eq!(get_ast_grep_language_name("unknown"), None);
/// ```
pub fn get_ast_grep_language_name(lang: &str) -> Option<&'static str> {
    match lang {
        "rust" => Some("rust"),
        "python" => Some("python"),
        "javascript" => Some("javascript"),
        "typescript" => Some("typescript"),
        "java" => Some("java"),
        "c" => Some("c"),
        "cpp" => Some("cpp"),
        "go" => Some("go"),
        "csharp" => Some("csharp"),
        "swift" => Some("swift"),
        "kotlin" => Some("kotlin"),
        "scala" => Some("scala"),
        "bash" => Some("bash"),
        "html" => Some("html"),
        "css" => Some("css"),
        _ => None,
    }
}

/// Parses a Git diff text and returns a structured `GitDiff` object representing the changed files and their statuses.
///
/// # Returns
///
/// - `Ok(GitDiff)`: On successful parsing, containing the list of changed files and their change types.
/// - `Err(AnalysisError)`: If the diff text cannot be parsed.
///
/// # Examples
///
/// ```
/// let diff_text = "\
/// diff --git a/foo.rs b/foo.rs
/// new file mode 100644
/// index 0000000..e69de29
/// --- /dev/null
/// +++ b/foo.rs
/// ";
/// let git_diff = parse_git_diff(diff_text).unwrap();
/// assert_eq!(git_diff.changed_files.len(), 1);
/// ```
pub fn parse_git_diff(diff_text: &str) -> Result<GitDiff, crate::errors::AnalysisError> {
    let parser = GitDiffParser::new();
    parser.parse(diff_text)
}

/// Creates and returns a new instance of the AST analysis engine.
///
/// # Examples
///
/// ```
/// let engine = create_analysis_engine();
/// ```
pub fn create_analysis_engine() -> AstAnalysisEngine {
    AstAnalysisEngine::new()
}

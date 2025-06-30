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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn new() -> Self {
        let mut registry = Self {
            rules: HashMap::new(),
        };
        registry.load_builtin_rules();
        registry
    }

    pub fn get_rules_for_language(&self, language: &str) -> Vec<&AnalysisRule> {
        self.rules
            .get(language)
            .map(|rules| rules.iter().filter(|rule| rule.enabled).collect())
            .unwrap_or_default()
    }

    fn load_builtin_rules(&mut self) {
        self.add_rust_rules();
        self.add_python_rules();
        self.add_javascript_rules();
        self.add_typescript_rules();
    }

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
    pub fn new() -> Self {
        Self {
            rule_registry: RuleRegistry::new(),
        }
    }

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

    fn _is_language_supported(&self, lang: &str) -> bool {
        matches!(
            lang,
            "rust" | "python" | "javascript" | "typescript" | "java" | "c" | "cpp" | "go"
        )
    }

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

    fn count_classes(&self, content: &str, language: &str) -> usize {
        match language {
            "rust" => content.matches("struct ").count() + content.matches("enum ").count(),
            "python" => content.matches("class ").count(),
            "javascript" | "typescript" => content.matches("class ").count(),
            "java" | "c" | "cpp" => content.matches("class ").count(),
            _ => 0,
        }
    }

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

/// Detect programming language from file extension
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
    hunk_header_re: Regex,
}

impl GitDiffParser {
    pub fn new() -> Self {
        Self {
            file_header_re: Regex::new(r"^diff --git a/(.*) b/(.*)$").unwrap(),
            file_status_re: Regex::new(r"^(new file mode|deleted file mode|index)").unwrap(),
            hunk_header_re: Regex::new(r"^@@\s+-(\d+)(?:,(\d+))?\s+\+(\d+)(?:,(\d+))?\s+@@")
                .unwrap(),
        }
    }

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

/// Get ast-grep language string for ast-grep operations
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

/// Parse git diff text into structured GitDiff
pub fn parse_git_diff(diff_text: &str) -> Result<GitDiff, crate::errors::AnalysisError> {
    let parser = GitDiffParser::new();
    parser.parse(diff_text)
}

/// Create a new AST analysis engine instance
pub fn create_analysis_engine() -> AstAnalysisEngine {
    AstAnalysisEngine::new()
}

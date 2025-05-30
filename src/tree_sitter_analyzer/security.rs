use std::collections::HashMap;
use std::path::PathBuf;
use tree_sitter::{Query, QueryCursor, Tree};
use serde::{Deserialize, Serialize};
use streaming_iterator::StreamingIterator;
use chrono;

use crate::errors::TreeSitterError;
use super::core::LanguageRegistry;

/// Security vulnerability severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for SecuritySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecuritySeverity::Info => write!(f, "INFO"),
            SecuritySeverity::Low => write!(f, "LOW"),
            SecuritySeverity::Medium => write!(f, "MEDIUM"),
            SecuritySeverity::High => write!(f, "HIGH"),
            SecuritySeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Security vulnerability finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub file_path: PathBuf,
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub code_snippet: String,
    pub recommendation: String,
    pub cwe_id: Option<String>,
    pub owasp_category: Option<String>,
}

/// Security scan results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanResults {
    pub findings: Vec<SecurityFinding>,
    pub summary: SecuritySummary,
    pub scan_time: std::time::SystemTime,
}

/// Security scan summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySummary {
    pub total_files_scanned: usize,
    pub total_findings: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub info_count: usize,
}

/// Security rule definition
#[derive(Debug, Clone)]
pub struct SecurityRule {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub query: String,
    pub languages: Vec<String>,
    pub cwe_id: Option<String>,
    pub owasp_category: Option<String>,
    pub recommendation: String,
}

/// Tree-sitter based security scanner
pub struct TreeSitterSecurityScanner {
    language_registry: LanguageRegistry,
    security_rules: HashMap<String, Vec<SecurityRule>>,
}

impl TreeSitterSecurityScanner {
    pub fn new(language_registry: LanguageRegistry) -> Self {
        let mut scanner = Self {
            language_registry,
            security_rules: HashMap::new(),
        };
        scanner.load_default_rules();
        scanner
    }

    /// Load default security rules for various languages
    fn load_default_rules(&mut self) {
        // SQL Injection rules
        self.add_rule(SecurityRule {
            id: "sql-injection-1".to_string(),
            title: "Potential SQL Injection".to_string(),
            description: "Direct string concatenation in SQL queries can lead to SQL injection vulnerabilities".to_string(),
            severity: SecuritySeverity::High,
            query: r#"
                (call_expression
                  function: (identifier) @func
                  arguments: (arguments
                    (binary_expression
                      left: (string) @sql_string
                      right: (_) @user_input)))
                (#match? @func "(query|execute|exec)")
                (#match? @sql_string "(SELECT|INSERT|UPDATE|DELETE|CREATE|DROP|ALTER)")
            "#.to_string(),
            languages: vec!["js".to_string(), "java".to_string()],
            cwe_id: Some("CWE-89".to_string()),
            owasp_category: Some("A03:2021 – Injection".to_string()),
            recommendation: "Use parameterized queries or prepared statements instead of string concatenation".to_string(),
        });

        // XSS rules
        self.add_rule(SecurityRule {
            id: "xss-innerHTML".to_string(),
            title: "Potential XSS via innerHTML".to_string(),
            description: "Setting innerHTML with user input can lead to XSS vulnerabilities".to_string(),
            severity: SecuritySeverity::High,
            query: r#"
                (assignment_expression
                  left: (member_expression
                    property: (property_identifier) @prop)
                  right: (_) @value)
                (#eq? @prop "innerHTML")
            "#.to_string(),
            languages: vec!["js".to_string()],
            cwe_id: Some("CWE-79".to_string()),
            owasp_category: Some("A03:2021 – Injection".to_string()),
            recommendation: "Use textContent instead of innerHTML, or sanitize user input properly".to_string(),
        });

        // Command Injection rules
        self.add_rule(SecurityRule {
            id: "command-injection-1".to_string(),
            title: "Potential Command Injection".to_string(),
            description: "Executing system commands with user input can lead to command injection".to_string(),
            severity: SecuritySeverity::Critical,
            query: r#"
                (call_expression
                  function: (identifier) @func
                  arguments: (arguments
                    (binary_expression
                      left: (string)
                      right: (_) @user_input)))
                (#match? @func "(exec|system|spawn|execSync)")
            "#.to_string(),
            languages: vec!["js".to_string(), "python".to_string()],
            cwe_id: Some("CWE-78".to_string()),
            owasp_category: Some("A03:2021 – Injection".to_string()),
            recommendation: "Use parameterized command execution or input validation/sanitization".to_string(),
        });

        // Hardcoded secrets
        self.add_rule(SecurityRule {
            id: "hardcoded-secret-1".to_string(),
            title: "Hardcoded Secret".to_string(),
            description: "Hardcoded secrets in source code pose security risks".to_string(),
            severity: SecuritySeverity::High,
            query: r#"
                (variable_declarator
                  name: (identifier) @var_name
                  value: (string) @secret_value)
                (#match? @var_name "(?i)(password|secret|key|token|api_key|private_key)")
                (#match? @secret_value "[A-Za-z0-9+/]{20,}")
            "#.to_string(),
            languages: vec!["js".to_string(), "python".to_string(), "java".to_string()],
            cwe_id: Some("CWE-798".to_string()),
            owasp_category: Some("A02:2021 – Cryptographic Failures".to_string()),
            recommendation: "Use environment variables or secure configuration management for secrets".to_string(),
        });

        // Weak cryptography
        self.add_rule(SecurityRule {
            id: "weak-crypto-1".to_string(),
            title: "Weak Cryptographic Algorithm".to_string(),
            description: "Use of weak or deprecated cryptographic algorithms".to_string(),
            severity: SecuritySeverity::Medium,
            query: r#"
                (call_expression
                  function: (member_expression
                    object: (identifier) @crypto_obj
                    property: (property_identifier) @method))
                (#match? @crypto_obj "(crypto|Crypto)")
                (#match? @method "(md5|sha1|des|rc4)")
            "#.to_string(),
            languages: vec!["js".to_string(), "python".to_string()],
            cwe_id: Some("CWE-327".to_string()),
            owasp_category: Some("A02:2021 – Cryptographic Failures".to_string()),
            recommendation: "Use strong cryptographic algorithms like SHA-256, AES, or modern alternatives".to_string(),
        });

        // Rust-specific rules
        self.add_rule(SecurityRule {
            id: "rust-unsafe-1".to_string(),
            title: "Unsafe Rust Code".to_string(),
            description: "Unsafe blocks bypass Rust's safety guarantees and should be carefully reviewed".to_string(),
            severity: SecuritySeverity::Medium,
            query: r#"
                (unsafe_block) @unsafe_block
            "#.to_string(),
            languages: vec!["rust".to_string()],
            cwe_id: Some("CWE-119".to_string()),
            owasp_category: None,
            recommendation: "Ensure unsafe code is necessary and properly reviewed for memory safety".to_string(),
        });

        // Path traversal
        self.add_rule(SecurityRule {
            id: "path-traversal-1".to_string(),
            title: "Potential Path Traversal".to_string(),
            description: "File operations with user input may allow path traversal attacks".to_string(),
            severity: SecuritySeverity::High,
            query: r#"
                (call_expression
                  function: (member_expression
                    object: (identifier) @fs_obj
                    property: (property_identifier) @method)
                  arguments: (arguments
                    (binary_expression
                      left: (_)
                      right: (_) @user_path)))
                (#match? @fs_obj "(fs|path|File)")
                (#match? @method "(readFile|writeFile|open|join)")
            "#.to_string(),
            languages: vec!["js".to_string(), "python".to_string()],
            cwe_id: Some("CWE-22".to_string()),
            owasp_category: Some("A01:2021 – Broken Access Control".to_string()),
            recommendation: "Validate and sanitize file paths, use allowlists for permitted directories".to_string(),
        });
    }

    fn add_rule(&mut self, rule: SecurityRule) {
        for language in &rule.languages {
            self.security_rules
                .entry(language.clone())
                .or_insert_with(Vec::new)
                .push(rule.clone());
        }
    }

    /// Scan a file for security vulnerabilities
    pub fn scan_file(&self, file_path: &PathBuf, content: &str, language: &str) -> Result<Vec<SecurityFinding>, TreeSitterError> {
        let mut findings = Vec::new();

        // Get language parser
        let language_config = self.language_registry.get_config(language)
            .ok_or_else(|| TreeSitterError::UnsupportedLanguage(language.to_string()))?;
        let language_def = language_config.get_language();

        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language_def)
            .map_err(|e| TreeSitterError::ParseError(format!("Failed to set language: {}", e)))?;

        // Parse the file
        let tree = parser.parse(content, None)
            .ok_or_else(|| TreeSitterError::ParseError("Failed to parse file".to_string()))?;

        // Get security rules for this language
        if let Some(rules) = self.security_rules.get(language) {
            for rule in rules {
                let rule_findings = self.apply_rule(rule, &tree, content, file_path)?;
                findings.extend(rule_findings);
            }
        }

        Ok(findings)
    }

    /// Apply a security rule to a parsed tree
    fn apply_rule(&self, rule: &SecurityRule, tree: &Tree, content: &str, file_path: &PathBuf) -> Result<Vec<SecurityFinding>, TreeSitterError> {
        let mut findings = Vec::new();

        // Get language for the query
        let language = &rule.languages[0]; // Use first language as primary
        let language_config = self.language_registry.get_config(language)
            .ok_or_else(|| TreeSitterError::UnsupportedLanguage(language.to_string()))?;
        let language_def = language_config.get_language();

        // Create and execute query
        let query = Query::new(&language_def, &rule.query)
            .map_err(|e| TreeSitterError::QueryError(format!("Invalid query: {}", e)))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let start_position = node.start_position();
                let end_position = node.end_position();

                // Extract code snippet
                let start_byte = node.start_byte();
                let end_byte = node.end_byte();
                let code_snippet = content.get(start_byte..end_byte)
                    .unwrap_or("")
                    .to_string();

                let finding = SecurityFinding {
                    id: rule.id.clone(),
                    title: rule.title.clone(),
                    description: rule.description.clone(),
                    severity: rule.severity,
                    file_path: file_path.clone(),
                    line_start: start_position.row + 1,
                    line_end: end_position.row + 1,
                    column_start: start_position.column + 1,
                    column_end: end_position.column + 1,
                    code_snippet,
                    recommendation: rule.recommendation.clone(),
                    cwe_id: rule.cwe_id.clone(),
                    owasp_category: rule.owasp_category.clone(),
                };

                findings.push(finding);
            }
        }

        Ok(findings)
    }

    /// Scan multiple files and generate a comprehensive report
    pub fn scan_files(&self, files: Vec<(PathBuf, String, String)>) -> Result<SecurityScanResults, TreeSitterError> {
        let mut all_findings = Vec::new();
        let scan_time = std::time::SystemTime::now();

        for (file_path, content, language) in &files {
            let findings = self.scan_file(file_path, content, language)?;
            all_findings.extend(findings);
        }

        // Generate summary
        let summary = self.generate_summary(&all_findings, files.len());

        Ok(SecurityScanResults {
            findings: all_findings,
            summary,
            scan_time,
        })
    }

    fn generate_summary(&self, findings: &[SecurityFinding], files_scanned: usize) -> SecuritySummary {
        let mut critical_count = 0;
        let mut high_count = 0;
        let mut medium_count = 0;
        let mut low_count = 0;
        let mut info_count = 0;

        for finding in findings {
            match finding.severity {
                SecuritySeverity::Critical => critical_count += 1,
                SecuritySeverity::High => high_count += 1,
                SecuritySeverity::Medium => medium_count += 1,
                SecuritySeverity::Low => low_count += 1,
                SecuritySeverity::Info => info_count += 1,
            }
        }

        SecuritySummary {
            total_files_scanned: files_scanned,
            total_findings: findings.len(),
            critical_count,
            high_count,
            medium_count,
            low_count,
            info_count,
        }
    }

    /// Add custom security rule
    pub fn add_custom_rule(&mut self, rule: SecurityRule) {
        self.add_rule(rule);
    }

    /// Get available languages
    pub fn get_supported_languages(&self) -> Vec<String> {
        self.security_rules.keys().cloned().collect()
    }
}

impl SecurityScanResults {
    /// Create a new empty SecurityScanResults
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
            summary: SecuritySummary {
                total_files_scanned: 0,
                total_findings: 0,
                critical_count: 0,
                high_count: 0,
                medium_count: 0,
                low_count: 0,
                info_count: 0,
            },
            scan_time: std::time::SystemTime::now(),
        }
    }

    /// Filter findings by severity
    pub fn filter_by_severity(&self, min_severity: SecuritySeverity) -> Vec<&SecurityFinding> {
        self.findings
            .iter()
            .filter(|finding| finding.severity >= min_severity)
            .collect()
    }

    /// Group findings by file
    pub fn group_by_file(&self) -> HashMap<PathBuf, Vec<&SecurityFinding>> {
        let mut grouped = HashMap::new();
        for finding in &self.findings {
            grouped
                .entry(finding.file_path.clone())
                .or_insert_with(Vec::new)
                .push(finding);
        }
        grouped
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Convert scan results to markdown format
    pub fn to_markdown(&self) -> String {
        let mut markdown = String::new();
        
        // Header
        markdown.push_str("# Security Scan Results\n\n");
        
        // Summary
        markdown.push_str(&format!("**Scan Summary:**\n"));
        markdown.push_str(&format!("- Files scanned: {}\n", self.summary.total_files_scanned));
        markdown.push_str(&format!("- Total findings: {}\n", self.findings.len()));
        
        // Calculate scan duration from scan_time
        let duration = self.scan_time.elapsed().unwrap_or(std::time::Duration::from_secs(0));
        markdown.push_str(&format!("- Scan time: {}\n\n", 
            chrono::DateTime::<chrono::Utc>::from(self.scan_time).format("%Y-%m-%d %H:%M:%S UTC")));
        
        if self.findings.is_empty() {
            markdown.push_str("✅ **No security issues found!**\n");
            return markdown;
        }
        
        // Severity breakdown
        let mut severity_counts = HashMap::new();
        for finding in &self.findings {
            *severity_counts.entry(finding.severity).or_insert(0) += 1;
        }
        
        markdown.push_str("**Findings by Severity:**\n");
        for severity in [SecuritySeverity::Critical, SecuritySeverity::High, SecuritySeverity::Medium, SecuritySeverity::Low, SecuritySeverity::Info] {
            if let Some(count) = severity_counts.get(&severity) {
                let emoji = match severity {
                    SecuritySeverity::Critical => "🔴",
                    SecuritySeverity::High => "🟠",
                    SecuritySeverity::Medium => "🟡",
                    SecuritySeverity::Low => "🔵",
                    SecuritySeverity::Info => "ℹ️",
                };
                markdown.push_str(&format!("- {} {}: {}\n", emoji, severity, count));
            }
        }
        markdown.push_str("\n");
        
        // Group findings by file
        let grouped = self.group_by_file();
        
        markdown.push_str("## Detailed Findings\n\n");
        
        for (file_path, findings) in grouped {
            markdown.push_str(&format!("### 📁 {}\n\n", file_path.display()));
            
            for finding in findings {
                let severity_emoji = match finding.severity {
                    SecuritySeverity::Critical => "🔴",
                    SecuritySeverity::High => "🟠",
                    SecuritySeverity::Medium => "🟡",
                    SecuritySeverity::Low => "🔵",
                    SecuritySeverity::Info => "ℹ️",
                };
                
                markdown.push_str(&format!("#### {} {} {}\n\n", severity_emoji, finding.severity, finding.title));
                markdown.push_str(&format!("**Description:** {}\n\n", finding.description));
                markdown.push_str(&format!("**Location:** Lines {}-{}, Columns {}-{}\n\n", 
                    finding.line_start, finding.line_end, finding.column_start, finding.column_end));
                
                if !finding.code_snippet.is_empty() {
                    markdown.push_str("**Code:**\n");
                    markdown.push_str("```\n");
                    markdown.push_str(&finding.code_snippet);
                    markdown.push_str("\n```\n\n");
                }
                
                markdown.push_str(&format!("**Recommendation:** {}\n\n", finding.recommendation));
                
                if let Some(cwe) = &finding.cwe_id {
                    markdown.push_str(&format!("**CWE:** {}\n\n", cwe));
                }
                
                if let Some(owasp) = &finding.owasp_category {
                    markdown.push_str(&format!("**OWASP Category:** {}\n\n", owasp));
                }
                
                markdown.push_str("---\n\n");
            }
        }
        
        // Footer
        markdown.push_str(&format!("*Generated on {}*\n", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        
        markdown
    }
}
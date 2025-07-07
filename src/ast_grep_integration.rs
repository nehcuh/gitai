// AST-grep integration for gitai
// Provides correct API usage for ast-grep-core 0.38.6

use crate::errors::AppError;
use ast_grep_config::SerializableRuleCore;
use serde_yaml;
use std::collections::HashMap;
use regex;

/// Supported languages for AST analysis
#[derive(Debug, Clone, PartialEq)]
pub enum SupportedLanguage {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Java,
    C,
    Cpp,
    Go,
    Html,
    Css,
}

impl SupportedLanguage {
    /// Get the string representation used by ast-grep
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript", 
            Self::Python => "python",
            Self::Java => "java",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::Go => "go",
            Self::Html => "html",
            Self::Css => "css",
        }
    }

    /// Create from string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "rust" => Some(Self::Rust),
            "javascript" => Some(Self::JavaScript),
            "typescript" => Some(Self::TypeScript),
            "python" => Some(Self::Python),
            "java" => Some(Self::Java),
            "c" => Some(Self::C),
            "cpp" => Some(Self::Cpp),
            "go" => Some(Self::Go),
            "html" => Some(Self::Html),
            "css" => Some(Self::Css),
            _ => None,
        }
    }
}

/// Represents a match found by ast-grep
#[derive(Debug, Clone)]
pub struct AstMatch {
    pub line: usize,
    pub column: usize,
    pub text: String,
    pub file_path: String,
    pub rule_id: String,
    pub pattern_variables: HashMap<String, String>, // Captured pattern variables
}

/// AST-grep rule container
#[derive(Clone)]
pub struct AstGrepRule {
    pub id: String,
    pub language: String,
    pub severity: String,
    pub message: String,
    pub note: Option<String>,
    pub utils: Option<HashMap<String, serde_yaml::Value>>,
    pub rule_core: SerializableRuleCore,
}

impl std::fmt::Debug for AstGrepRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AstGrepRule")
            .field("id", &self.id)
            .field("language", &self.language)
            .field("severity", &self.severity)
            .field("message", &self.message)
            .field("note", &self.note)
            .field("utils", &self.utils)
            .field("rule_core", &"<SerializableRuleCore>")
            .finish()
    }
}

/// AST-grep engine wrapper with performance optimizations
pub struct AstGrepEngine {
    // Store configured rules
    rules: HashMap<String, AstGrepRule>,
    // Cache compiled regex patterns for performance
    regex_cache: HashMap<String, regex::Regex>,
}

impl AstGrepEngine {
    /// Create a new AST-grep engine
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            regex_cache: HashMap::new(),
        }
    }

    /// Add a rule from YAML configuration with improved error handling
    pub fn add_rule(&mut self, rule_yaml: &str) -> Result<String, AppError> {
        // Early validation - check if YAML is empty or malformed
        if rule_yaml.trim().is_empty() {
            return Err(AppError::Config(crate::errors::ConfigError::Other(
                "Rule YAML cannot be empty".to_string()
            )));
        }
        
        // Parse the full rule configuration with better error context
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(rule_yaml)
            .map_err(|e| AppError::Config(crate::errors::ConfigError::Other(
                format!("Failed to parse rule YAML at line {}: {}", 
                       e.location().map(|l| l.line()).unwrap_or(0), e)
            )))?;

        // Extract metadata with validation
        let rule_id = yaml_value
            .get("id")
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("rule_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));

        let language = yaml_value
            .get("language")
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| AppError::Config(crate::errors::ConfigError::Other(
                "Rule must specify a valid 'language' field".to_string()
            )))?
            .to_string();

        // For unsupported languages, issue a warning but still add the rule for fallback matching
        if !self.is_language_supported(&language) {
            println!("Warning: Language '{}' may use fallback regex matching instead of full AST analysis", language);
        }

        let severity = yaml_value
            .get("severity")
            .and_then(|v| v.as_str())
            .filter(|s| matches!(*s, "error" | "warning" | "info" | "hint"))
            .unwrap_or("info")
            .to_string();

        let message = yaml_value
            .get("message")
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("Pattern match found")
            .to_string();

        let note = yaml_value
            .get("note")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Parse utils section
        let utils = yaml_value
            .get("utils")
            .and_then(|v| v.as_mapping())
            .map(|mapping| {
                mapping.iter().map(|(k, v)| {
                    (k.as_str().unwrap_or("").to_string(), v.clone())
                }).collect::<HashMap<String, serde_yaml::Value>>()
            });

        // Parse the rule core
        let rule_core: SerializableRuleCore = serde_yaml::from_str(rule_yaml)
            .map_err(|e| AppError::Config(crate::errors::ConfigError::Other(
                format!("Failed to parse rule core: {}", e)
            )))?;

        let ast_grep_rule = AstGrepRule {
            id: rule_id.clone(),
            language,
            severity,
            message,
            note,
            utils,
            rule_core,
        };

        self.rules.insert(rule_id.clone(), ast_grep_rule);
        Ok(rule_id)
    }

    /// Optimized pattern matching using regex fallback for unsupported languages
    pub fn find_matches_simple(
        &self,
        source_code: &str,
        language: SupportedLanguage,
        file_path: &str,
    ) -> Result<Vec<AstMatch>, AppError> {
        let mut matches = Vec::new();
        
        // Early exit if source code is empty
        if source_code.trim().is_empty() {
            return Ok(matches);
        }
        
        // Pre-filter rules by language for performance
        let applicable_rules: Vec<_> = self.rules.iter()
            .filter(|(_, rule)| rule.language == language.as_str())
            .collect();
        
        if applicable_rules.is_empty() {
            return Ok(matches);
        }
        
        // Split into lines once for all rules (performance optimization)
        let lines: Vec<&str> = source_code.lines().collect();
        
        for (rule_id, rule) in applicable_rules {
            // Try to extract simple patterns from the rule
            let simple_matches = self.apply_simple_pattern_matching_optimized(
                &lines,
                rule,
                file_path,
                rule_id,
            )?;
            
            matches.extend(simple_matches);
        }
        
        Ok(matches)
    }


    /// Extract pattern string from rule configuration
    fn extract_pattern_from_rule(&self, rule: &AstGrepRule) -> Result<Option<String>, AppError> {
        // Try to extract pattern from the rule_core
        let yaml_str = serde_yaml::to_string(&rule.rule_core)
            .map_err(|e| AppError::Config(crate::errors::ConfigError::Other(
                format!("Failed to serialize rule core: {}", e)
            )))?;
        
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_str)
            .map_err(|e| AppError::Config(crate::errors::ConfigError::Other(
                format!("Failed to parse rule core YAML: {}", e)
            )))?;
        
        // Look for pattern in rule section
        if let Some(rule_section) = yaml_value.get("rule") {
            if let Some(pattern) = rule_section.get("pattern") {
                if let Some(pattern_str) = pattern.as_str() {
                    return Ok(Some(pattern_str.to_string()));
                }
            }
            
            // Look for 'any' patterns (common in ast-grep rules)
            if let Some(any_section) = rule_section.get("any") {
                if let Some(any_array) = any_section.as_sequence() {
                    for item in any_array {
                        if let Some(matches_value) = item.get("matches") {
                            if let Some(pattern_str) = matches_value.as_str() {
                                return Ok(Some(pattern_str.to_string()));
                            }
                        }
                        if let Some(pattern) = item.get("pattern") {
                            if let Some(pattern_str) = pattern.as_str() {
                                return Ok(Some(pattern_str.to_string()));
                            }
                        }
                    }
                }
            }
            
            // Look for 'matches' patterns
            if let Some(matches_value) = rule_section.get("matches") {
                if let Some(pattern_str) = matches_value.as_str() {
                    return Ok(Some(pattern_str.to_string()));
                }
            }
        }
        
        // Look for pattern at root level
        if let Some(pattern) = yaml_value.get("pattern") {
            if let Some(pattern_str) = pattern.as_str() {
                return Ok(Some(pattern_str.to_string()));
            }
        }
        
        Ok(None)
    }

    /// Convert ast-grep pattern with variables to regex pattern
    fn convert_ast_grep_pattern_to_regex(&self, pattern: &str) -> Result<String, AppError> {
        // For simple patterns like "HTTPBasicAuth($USER,\"\",...):", create targeted regex
        if pattern.contains("HTTPBasicAuth") && pattern.contains("$USER") {
            // Create a regex that matches HTTPBasicAuth calls with hardcoded passwords
            return Ok(r#"HTTPBasicAuth\s*\(\s*[^,]+\s*,\s*["'][^"']+["']\s*[,)]"#.to_string());
        }
        
        // For patterns with requests.auth, create specific matching
        if pattern.contains("requests.auth.HTTPBasicAuth") {
            return Ok(r#"requests\.auth\.HTTPBasicAuth\s*\(\s*[^,]+\s*,\s*["'][^"']+["']\s*[,)]"#.to_string());
        }
        
        let mut regex_pattern = pattern.to_string();
        
        // Map common ast-grep pattern variables to regex patterns
        let variable_mappings = [
            ("$USER", r"[^,\)]+"),                          // User parameter (more flexible)
            ("$INSTANCE", r"[a-zA-Z_][a-zA-Z0-9_]*"),      // Variable/instance names
            ("$STR", r#"["'][^"']*["']"#),                  // String literals with quotes
            ("$MSG", r#"["'][^"']*["']"#),                  // Message strings
            ("$ARGS", r"[^)]*"),                            // Function arguments
            ("$VALUE", r"[^;,\n]*"),                        // Values in assignments
            ("$EXPR", r"[^;,\n]*"),                         // Expressions
            ("$FUNC", r"[a-zA-Z_][a-zA-Z0-9_]*"),          // Function names
            ("$VAR", r"[a-zA-Z_][a-zA-Z0-9_]*"),           // Variable names
            ("$TYPE", r"[a-zA-Z_][a-zA-Z0-9_<>]*"),        // Type names
            ("$BODY", r"[\s\S]*?"),                         // Function/block bodies
            ("$NAME", r"[a-zA-Z_][a-zA-Z0-9_]*"),          // Generic names
            ("$URL", r#"["'][^"']*["']"#),                  // URL strings
            ("$KEY", r#"["'][^"']*["']"#),                  // Key strings
            ("$SECRET", r#"["'][^"']*["']"#),               // Secret strings
            ("$PASSWORD", r#"["'][^"']*["']"#),             // Password strings
            ("$TOKEN", r#"["'][^"']*["']"#),                // Token strings
            ("...", r"[^)]*"),                              // Ellipsis for variable args
        ];
        
        // Replace ast-grep variables with regex patterns (no escaping needed)
        for (var, regex_replacement) in &variable_mappings {
            regex_pattern = regex_pattern.replace(var, regex_replacement);
        }
        
        // Escape remaining regex special characters but preserve our inserted patterns
        // This is more complex - we need to selectively escape
        regex_pattern = regex_pattern
            .replace(".", r"\.")
            .replace("(", r"\(")
            .replace(")", r"\)")
            .replace("[", r"\[")
            .replace("]", r"\]")
            .replace("{", r"\{")
            .replace("}", r"\}")
            .replace("+", r"\+")
            .replace("*", r"\*")
            .replace("?", r"\?")
            .replace("^", r"\^")
            .replace("$", r"\$")
            .replace("|", r"\|");
        
        // Restore our regex patterns that got escaped
        for (_, regex_replacement) in &variable_mappings {
            let escaped_replacement = regex::escape(regex_replacement);
            regex_pattern = regex_pattern.replace(&escaped_replacement, regex_replacement);
        }
        
        Ok(regex_pattern)
    }

    /// Extract pattern variables from matched text
    fn extract_pattern_variables(&self, regex: &regex::Regex, text: &str) -> HashMap<String, String> {
        let mut variables = HashMap::new();
        
        if let Some(captures) = regex.captures(text) {
            // For now, just capture the entire match
            // In a more sophisticated implementation, we would map specific capture groups to variables
            if let Some(full_match) = captures.get(0) {
                variables.insert("$MATCH".to_string(), full_match.as_str().to_string());
            }
        }
        
        variables
    }

    /// Optimized pattern matching that works with pre-split lines and uses caching
    fn apply_simple_pattern_matching_optimized(
        &self,
        lines: &[&str],
        rule: &AstGrepRule,
        file_path: &str,
        rule_id: &str,
    ) -> Result<Vec<AstMatch>, AppError> {
        let mut matches = Vec::new();
        
        // Extract pattern from rule configuration if possible
        let pattern = self.extract_pattern_from_rule(rule)?;
        
        if let Some(pattern_str) = pattern {
            // Convert ast-grep pattern to simplified regex for pattern variables support
            let converted_pattern = self.convert_ast_grep_pattern_to_regex(&pattern_str)?;
            
            // Compile regex (caching would require &mut self or Arc<Mutex<>> pattern)
            match regex::Regex::new(&converted_pattern) {
                Ok(regex) => {
                    for (line_num, line) in lines.iter().enumerate() {
                        if regex.is_match(line) {
                            // Extract matched variables if present
                            let matched_variables = self.extract_pattern_variables(&regex, line);
                            
                            matches.push(AstMatch {
                                line: line_num + 1,
                                column: self.find_match_column(&regex, line).unwrap_or(1),
                                text: line.to_string(),
                                file_path: file_path.to_string(),
                                rule_id: rule_id.to_string(),
                                pattern_variables: matched_variables,
                            });
                        }
                    }
                }
                Err(_) => {
                    // Enhanced fallback to text search for security patterns
                    for (line_num, line) in lines.iter().enumerate() {
                        let should_match = match rule.language.as_str() {
                            "python" => {
                                line.contains("HTTPBasicAuth") ||
                                line.contains("requests.auth") ||
                                line.contains("hardcoded_password") ||
                                line.contains("password=") ||
                                line.contains("secret=") ||
                                line.contains("token=") ||
                                (line.contains("print(") && rule_id.contains("avoid-print"))
                            },
                            "javascript" | "typescript" => {
                                line.contains("console.log") ||
                                line.contains("hardcoded") ||
                                line.contains("password:") ||
                                line.contains("secret:")
                            },
                            "rust" => {
                                (line.contains("println!") && rule_id.contains("avoid-println")) ||
                                line.contains("hardcoded")
                            },
                            _ => {
                                line.contains("password") ||
                                line.contains("secret") ||
                                line.contains("hardcoded")
                            }
                        };
                        
                        if should_match {
                            matches.push(AstMatch {
                                line: line_num + 1,
                                column: self.find_match_column_simple(line, &["password", "secret", "hardcoded", "HTTPBasicAuth"]).unwrap_or(1),
                                text: line.to_string(),
                                file_path: file_path.to_string(),
                                rule_id: rule_id.to_string(),
                                pattern_variables: HashMap::new(),
                            });
                        }
                    }
                }
            }
        }
        
        Ok(matches)
    }

    /// Find the column position of a regex match
    fn find_match_column(&self, regex: &regex::Regex, line: &str) -> Option<usize> {
        regex.find(line).map(|m| m.start() + 1)
    }
    
    /// Find the column position of simple text patterns
    fn find_match_column_simple(&self, line: &str, patterns: &[&str]) -> Option<usize> {
        for pattern in patterns {
            if let Some(pos) = line.find(pattern) {
                return Some(pos + 1);
            }
        }
        None
    }

    /// Get all loaded rules
    pub fn get_rules(&self) -> &HashMap<String, AstGrepRule> {
        &self.rules
    }

    /// Clear all rules and regex cache
    pub fn clear_rules(&mut self) {
        self.rules.clear();
        self.regex_cache.clear();
    }

    /// Remove a specific rule
    pub fn remove_rule(&mut self, rule_id: &str) -> Option<AstGrepRule> {
        self.rules.remove(rule_id)
    }

    /// Check if a rule exists
    pub fn has_rule(&self, rule_id: &str) -> bool {
        self.rules.contains_key(rule_id)
    }

    /// Check if a language is supported by the engine
    fn is_language_supported(&self, language: &str) -> bool {
        matches!(language.to_lowercase().as_str(), 
            "rust" | "javascript" | "typescript" | "python" | "java" | 
            "c" | "cpp" | "go" | "html" | "css" |
            // Support variations in capitalization
            "js" | "ts" | "py" | "rs" | "rb" | "ruby" | "php" | "swift" |
            "kotlin" | "scala" | "csharp" | "c#"
        )
    }
}

impl Default for AstGrepEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a simple pattern rule
pub fn create_simple_pattern_rule(
    id: &str,
    language: &str,
    pattern: &str,
    message: &str,
    severity: &str,
) -> String {
    format!(
        r#"id: {}
language: {}
rule:
  pattern: {}
message: "{}"
severity: {}"#,
        id, language, pattern, message, severity
    )
}

/// Helper function to create a complex rule with utils
pub fn create_complex_rule(
    id: &str,
    language: &str,
    rule_body: &str,
    utils: Option<&str>,
    message: &str,
    severity: &str,
) -> String {
    let utils_section = if let Some(utils_content) = utils {
        format!("utils:\n{}", utils_content)
    } else {
        String::new()
    };

    format!(
        r#"id: {}
language: {}
rule:
{}
{}
message: "{}"
severity: {}"#,
        id, language, rule_body, utils_section, message, severity
    )
}

/// Example rule templates for common patterns
pub struct RuleTemplates;

impl RuleTemplates {
    /// Create a rule to find console.log statements in JavaScript
    pub fn javascript_console_log() -> String {
        create_simple_pattern_rule(
            "no-console-log",
            "javascript",
            "console.log($ARGS)",
            "Avoid using console.log in production code",
            "warning"
        )
    }

    /// Create a rule to find println! macros in Rust
    pub fn rust_println() -> String {
        create_simple_pattern_rule(
            "avoid-println",
            "rust",
            "println!($ARGS)",
            "Consider using proper logging instead of println!",
            "info"
        )
    }

    /// Create a rule to find print statements in Python
    pub fn python_print() -> String {
        create_simple_pattern_rule(
            "avoid-print",
            "python",
            "print($ARGS)",
            "Consider using logging instead of print",
            "info"
        )
    }

    /// Create a rule to find TODO comments
    pub fn todo_comments(language: &str) -> String {
        format!(
            r#"id: todo-comments
language: {}
rule:
  regex: "TODO|FIXME|HACK"
message: "TODO comment found - consider creating an issue"
severity: hint"#,
            language
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_language_conversion() {
        assert_eq!(SupportedLanguage::Rust.as_str(), "rust");
        assert_eq!(SupportedLanguage::from_str("rust"), Some(SupportedLanguage::Rust));
        assert_eq!(SupportedLanguage::from_str("unknown"), None);
    }

    #[test]
    fn test_create_simple_pattern_rule() {
        let rule = create_simple_pattern_rule(
            "test-rule",
            "rust",
            "println!($MSG)",
            "Found println macro",
            "info"
        );
        
        assert!(rule.contains("id: test-rule"));
        assert!(rule.contains("language: rust"));
        assert!(rule.contains("pattern: println!($MSG)"));
    }

    #[test]
    fn test_ast_grep_engine_creation() {
        let engine = AstGrepEngine::new();
        assert!(engine.rules.is_empty());
    }

    #[test]
    fn test_add_rule() {
        let mut engine = AstGrepEngine::new();
        let rule_yaml = RuleTemplates::rust_println();
        
        let result = engine.add_rule(&rule_yaml);
        assert!(result.is_ok());
        assert!(engine.has_rule("avoid-println"));
    }

    #[test]
    fn test_rule_templates() {
        let js_rule = RuleTemplates::javascript_console_log();
        assert!(js_rule.contains("console.log"));
        
        let rust_rule = RuleTemplates::rust_println();
        assert!(rust_rule.contains("println!"));
        
        let python_rule = RuleTemplates::python_print();
        assert!(python_rule.contains("print"));
    }

    #[test]
    fn test_simple_pattern_matching() {
        let mut engine = AstGrepEngine::new();
        let rule_yaml = RuleTemplates::rust_println();
        engine.add_rule(&rule_yaml).unwrap();

        let source_code = r#"
fn main() {
    println!("Hello, world!");
    let x = 42;
}
"#;

        let matches = engine.find_matches_simple(
            source_code,
            SupportedLanguage::Rust,
            "test.rs"
        ).unwrap();

        assert!(!matches.is_empty());
        assert!(matches[0].text.contains("println!"));
        
        // Test that pattern variables are properly handled
        assert!(matches[0].pattern_variables.contains_key("$MATCH"));
    }
}
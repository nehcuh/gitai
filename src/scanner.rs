use crate::config::AppConfig;
use crate::errors::AppError;
use crate::types::git::ScanArgs;
use crate::ast_grep_integration::{AstGrepEngine, SupportedLanguage};
use serde_yaml::Value;
use regex::Regex;
use git2::{Repository, DiffOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AstGrepRule {
    pub id: String,
    pub language: String,
    pub severity: String,
    pub message: String,
    pub note: Option<String>,
    pub utils: Option<HashMap<String, serde_yaml::Value>>,
    pub rule: AstGrepRuleConfig,
}

#[derive(Clone)]
pub enum AstGrepRuleConfig {
    Pattern(String),
    Complex(serde_yaml::Value),
    // TODO: Re-enable when ast-grep integration is fixed
    // Parsed(SerializableRuleCore),
}

impl std::fmt::Debug for AstGrepRuleConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstGrepRuleConfig::Pattern(p) => write!(f, "Pattern({})", p),
            AstGrepRuleConfig::Complex(_) => write!(f, "Complex(yaml)"),
            // TODO: Re-enable when ast-grep integration is fixed
            // AstGrepRuleConfig::Parsed(_) => write!(f, "Parsed(rule)"),
        }
    }
}

// Keep the simple rule for backward compatibility
#[derive(Debug, Clone)]
pub struct SimpleRule {
    pub id: String,
    pub pattern: String,
    pub message: String,
    pub severity: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMatch {
    pub file_path: String,
    pub line_number: usize,
    pub column_number: usize,
    pub rule_id: String,
    pub rule_name: String,
    pub message: String,
    pub severity: String,
    pub matched_text: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub scan_id: String,
    pub repository: String,
    pub commit_id: String,
    pub scan_type: String, // "incremental" or "full"
    pub scan_time: String,
    pub rules_count: usize,
    pub files_scanned: usize,
    pub matches: Vec<ScanMatch>,
    pub summary: ScanSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    pub total_matches: usize,
    pub by_severity: HashMap<String, usize>,
    pub by_rule: HashMap<String, usize>,
    pub by_file: HashMap<String, usize>,
}

pub struct LocalScanner {
    config: AppConfig,
    repo: Option<Repository>,
    ast_engine: AstGrepEngine,
}

impl LocalScanner {
    pub fn new(config: AppConfig) -> Result<Self, AppError> {
        let repo = Repository::open_from_env().ok();
        let ast_engine = AstGrepEngine::new();
        Ok(Self { config, repo, ast_engine })
    }

    pub async fn scan(&mut self, args: &ScanArgs, rule_paths: &[PathBuf]) -> Result<ScanResult, AppError> {
        let files_to_scan = if args.full {
            self.get_all_files(&args.path)?
        } else {
            self.get_incremental_files(&args.path)?
        };

        let rules = self.load_rules(rule_paths)?;
        let mut matches = Vec::new();

        for file_path in &files_to_scan {
            if let Ok(file_content) = fs::read_to_string(file_path) {
                let file_matches = self.scan_file(&file_content, file_path, &rules)?;
                matches.extend(file_matches);
            }
        }

        let scan_result = self.build_scan_result(args, &matches, &files_to_scan, rule_paths.len())?;
        Ok(scan_result)
    }

    fn get_all_files(&self, path: &Option<String>) -> Result<Vec<PathBuf>, AppError> {
        let scan_path = if let Some(path) = path {
            PathBuf::from(path)
        } else {
            std::env::current_dir()
                .map_err(|e| AppError::FileRead("current directory".to_string(), e))?
        };

        let mut files = Vec::new();
        self.collect_files(&scan_path, &mut files)?;
        
        // Filter out files that should be ignored
        let filtered_files = self.filter_ignored_files(files)?;
        Ok(filtered_files)
    }

    fn get_incremental_files(&self, path: &Option<String>) -> Result<Vec<PathBuf>, AppError> {
        let repo = self.repo.as_ref().ok_or_else(|| {
            AppError::Git(crate::errors::GitError::NotARepository)
        })?;

        let mut diff_options = DiffOptions::new();
        diff_options.include_untracked(true);
        
        let head_tree = repo.head().map_err(|e| AppError::Git(crate::errors::GitError::Other(e.to_string())))?.peel_to_tree().map_err(|e| AppError::Git(crate::errors::GitError::Other(e.to_string())))?;
        let diff = repo.diff_tree_to_workdir_with_index(Some(&head_tree), Some(&mut diff_options)).map_err(|e| AppError::Git(crate::errors::GitError::Other(e.to_string())))?;

        let mut files = Vec::new();
        diff.foreach(
            &mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    files.push(path.to_path_buf());
                }
                true
            },
            None,
            None,
            None,
        ).map_err(|e| AppError::Git(crate::errors::GitError::Other(e.to_string())))?;

        // Filter by scan path if specified
        if let Some(scan_path) = path {
            let scan_path_buf = PathBuf::from(scan_path);
            files.retain(|f| f.starts_with(&scan_path_buf));
        }

        // Convert to absolute paths
        let current_dir = std::env::current_dir()
            .map_err(|e| AppError::FileRead("current directory".to_string(), e))?;
        let absolute_files: Vec<PathBuf> = files.into_iter()
            .map(|f| current_dir.join(f))
            .collect();

        Ok(absolute_files)
    }

    fn collect_files(&self, path: &Path, files: &mut Vec<PathBuf>) -> Result<(), AppError> {
        if path.is_file() {
            files.push(path.to_path_buf());
        } else if path.is_dir() {
            let entries = fs::read_dir(path)
                .map_err(|e| AppError::FileRead(path.to_string_lossy().to_string(), e))?;
            
            for entry in entries {
                let entry = entry.map_err(|e| AppError::FileRead("directory entry".to_string(), e))?;
                let entry_path = entry.path();
                
                // Skip hidden directories and common ignore patterns
                if let Some(file_name) = entry_path.file_name() {
                    let file_name_str = file_name.to_string_lossy();
                    if file_name_str.starts_with('.') || 
                       file_name_str == "node_modules" || 
                       file_name_str == "target" ||
                       file_name_str == "__pycache__" {
                        continue;
                    }
                }
                
                self.collect_files(&entry_path, files)?;
            }
        }
        Ok(())
    }

    fn filter_ignored_files(&self, files: Vec<PathBuf>) -> Result<Vec<PathBuf>, AppError> {
        // For now, implement basic filtering
        // In a more complete implementation, we would parse .gitignore files
        let filtered: Vec<PathBuf> = files.into_iter()
            .filter(|path| {
                // Only scan text files that are likely to contain code
                if let Some(extension) = path.extension() {
                    matches!(extension.to_str(), Some("rs") | Some("py") | Some("js") | Some("ts") | 
                            Some("java") | Some("c") | Some("cpp") | Some("go") | Some("rb") | 
                            Some("php") | Some("cs") | Some("swift") | Some("kt") | Some("scala") |
                            Some("html") | Some("css") | Some("scss") | Some("less") | Some("vue") |
                            Some("jsx") | Some("tsx") | Some("json") | Some("yaml") | Some("yml") |
                            Some("xml") | Some("sql") | Some("sh") | Some("bash") | Some("zsh") |
                            Some("fish") | Some("ps1") | Some("dockerfile") | Some("tf") | Some("hcl"))
                } else {
                    // Check if it's a common file without extension
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|name| matches!(name.to_lowercase().as_str(), 
                                           "dockerfile" | "makefile" | "rakefile" | "gemfile" | 
                                           "pipfile" | "requirements" | "setup" | "configure"))
                        .unwrap_or(false)
                }
            })
            .collect();
            
        Ok(filtered)
    }

    fn load_rules(&mut self, rule_paths: &[PathBuf]) -> Result<Vec<AstGrepRule>, AppError> {
        let mut rules = Vec::new();
        
        // Clear existing rules in AST engine
        self.ast_engine.clear_rules();
        
        for rule_path in rule_paths {
            let content = fs::read_to_string(rule_path)
                .map_err(|e| AppError::FileRead(rule_path.to_string_lossy().to_string(), e))?;
            
            // Try to add rule to AST engine
            match self.ast_engine.add_rule(&content) {
                Ok(rule_id) => {
                    println!("Successfully loaded rule: {}", rule_id);
                }
                Err(e) => {
                    println!("Warning: Failed to load rule from {}: {}", rule_path.display(), e);
                }
            }
            
            let yaml_value: Value = serde_yaml::from_str(&content)
                .map_err(|e| AppError::Config(crate::errors::ConfigError::Other(format!("Failed to parse rule file {}: {}", 
                                                    rule_path.display(), e))))?;
            
            // Parse ast-grep rule format for compatibility
            if let Some(rule) = self.parse_ast_grep_rule(&yaml_value, rule_path)? {
                rules.push(rule);
            }
        }
        
        println!("Loaded {} rules into AST engine", self.ast_engine.get_rules().len());
        Ok(rules)
    }

    fn parse_ast_grep_rule(&self, yaml: &Value, rule_path: &Path) -> Result<Option<AstGrepRule>, AppError> {
        if let Some(rule_map) = yaml.as_mapping() {
            // Required fields
            let id = rule_map.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| {
                    rule_path.file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or("unknown")
                })
                .to_string();
            
            let language = rule_map.get("language")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AppError::Config(crate::errors::ConfigError::Other(
                    format!("Rule {} missing required 'language' field", id)
                )))?
                .to_string();
            
            let severity = rule_map.get("severity")
                .and_then(|v| v.as_str())
                .unwrap_or("info")
                .to_string();
            
            let message = rule_map.get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Pattern match found")
                .to_string();
            
            // Optional fields
            let note = rule_map.get("note")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            
            // Parse utils section
            let utils = rule_map.get("utils")
                .and_then(|v| v.as_mapping())
                .map(|mapping| {
                    mapping.iter().map(|(k, v)| {
                        (k.as_str().unwrap_or("").to_string(), v.clone())
                    }).collect::<HashMap<String, Value>>()
                });
            
            // Parse rule section
            let rule_config = if let Some(rule_value) = rule_map.get("rule") {
                if let Some(pattern_str) = rule_value.get("pattern").and_then(|v| v.as_str()) {
                    AstGrepRuleConfig::Pattern(pattern_str.to_string())
                } else {
                    // Complex rules with utils, any, matches, etc.
                    AstGrepRuleConfig::Complex(rule_value.clone())
                }
            } else {
                // Fallback: look for a simple pattern field at root level
                if let Some(pattern_str) = rule_map.get("pattern").and_then(|v| v.as_str()) {
                    AstGrepRuleConfig::Pattern(pattern_str.to_string())
                } else {
                    return Err(AppError::Config(crate::errors::ConfigError::Other(
                        format!("Rule {} missing 'rule' section", id)
                    )));
                }
            };
            
            Ok(Some(AstGrepRule {
                id,
                language,
                severity,
                message,
                note,
                utils,
                rule: rule_config,
            }))
        } else {
            Ok(None)
        }
    }

    fn scan_file(&self, content: &str, file_path: &Path, rules: &[AstGrepRule]) -> Result<Vec<ScanMatch>, AppError> {
        let mut matches = Vec::new();
        
        // Determine language from file extension
        let language = self.detect_language(file_path);
        
        if let Some(lang) = &language {
            for rule in rules {
                // Check if rule applies to this language
                if rule.language != *lang {
                    continue;
                }
                
                // Use enhanced AST-based matching when possible
                let rule_matches = if let Some(supported_lang) = self.get_supported_language(lang) {
                    self.apply_ast_grep_matching(content, file_path, &supported_lang)?
                } else if self.supports_ast_grep(lang) {
                    self.apply_enhanced_rule_matching(content, file_path, rule)?
                } else {
                    self.apply_regex_fallback(content, file_path, rule)?
                };
                matches.extend(rule_matches);
            }
        }
        
        Ok(matches)
    }

    fn detect_language(&self, file_path: &Path) -> Option<String> {
        file_path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext {
                "rs" => Some("rust"),
                "py" => Some("python"),
                "js" => Some("javascript"),
                "ts" => Some("typescript"),
                "java" => Some("java"),
                "c" => Some("c"),
                "cpp" | "cc" | "cxx" => Some("cpp"),
                "go" => Some("go"),
                "rb" => Some("ruby"),
                "php" => Some("php"),
                "cs" => Some("csharp"),
                "swift" => Some("swift"),
                "kt" => Some("kotlin"),
                "scala" => Some("scala"),
                "html" => Some("html"),
                "css" => Some("css"),
                "scss" => Some("scss"),
                "less" => Some("less"),
                "vue" => Some("vue"),
                "jsx" => Some("jsx"),
                "tsx" => Some("tsx"),
                "json" => Some("json"),
                "yaml" | "yml" => Some("yaml"),
                "xml" => Some("xml"),
                "sql" => Some("sql"),
                "sh" | "bash" => Some("bash"),
                _ => None,
            })
            .map(|s| s.to_string())
    }

    fn get_supported_language(&self, language: &str) -> Option<SupportedLanguage> {
        SupportedLanguage::from_str(language)
    }

    fn get_context(&self, content: &str, line_number: usize, context_lines: usize) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let start = line_number.saturating_sub(context_lines);
        let end = (line_number + context_lines + 1).min(lines.len());
        
        lines[start..end].join("\n")
    }

    fn supports_ast_grep(&self, language: &str) -> bool {
        matches!(language, "rust" | "javascript" | "typescript" | "python" | "java" | "c" | "cpp" | "go" | "html" | "css" | "json" | "yaml")
    }

    fn apply_ast_grep_matching(
        &self,
        content: &str,
        file_path: &Path,
        language: &SupportedLanguage,
    ) -> Result<Vec<ScanMatch>, AppError> {
        let mut matches = Vec::new();
        
        // Use the AST engine for pattern matching
        let ast_matches = self.ast_engine.find_matches_simple(
            content,
            language.clone(),
            &file_path.to_string_lossy(),
        )?;
        
        // Convert AstMatch to ScanMatch
        for ast_match in ast_matches {
            // Create a more informative message if pattern variables were captured
            let message = if !ast_match.pattern_variables.is_empty() {
                let var_info: Vec<String> = ast_match.pattern_variables.iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                format!("Pattern found by AST analysis (variables: {})", var_info.join(", "))
            } else {
                "Pattern found by AST analysis".to_string()
            };
            
            let scan_match = ScanMatch {
                file_path: ast_match.file_path,
                line_number: ast_match.line,
                column_number: ast_match.column,
                rule_id: ast_match.rule_id,
                rule_name: "ast-grep-pattern".to_string(),
                message,
                severity: "info".to_string(),
                matched_text: ast_match.text,
                context: Some(self.get_context(content, ast_match.line.saturating_sub(1), 2)),
            };
            matches.push(scan_match);
        }
        
        Ok(matches)
    }

    fn apply_enhanced_rule_matching(
        &self,
        content: &str,
        file_path: &Path,
        rule: &AstGrepRule,
    ) -> Result<Vec<ScanMatch>, AppError> {
        let mut matches = Vec::new();

        match &rule.rule {
            AstGrepRuleConfig::Pattern(pattern) => {
                // For simple patterns, use regex for now
                // TODO: Implement true ast-grep pattern matching
                matches.extend(self.apply_pattern_matching(content, file_path, rule, pattern)?);
            }
            AstGrepRuleConfig::Complex(_yaml_value) => {
                // Complex rules with utils are not yet supported
                // TODO: Implement complex rule matching with utils support
                println!("Complex rule {} with utils not yet supported, skipping", rule.id);
            }
            // TODO: Re-enable when ast-grep integration is fixed
            // AstGrepRuleConfig::Parsed(_rule_config) => {
            //     // Parsed rules are not yet supported
            //     // TODO: Implement parsed rule matching
            //     println!("Parsed rule {} not yet supported, skipping", rule.id);
            // }
        }

        Ok(matches)
    }

    fn apply_regex_fallback(
        &self,
        content: &str,
        file_path: &Path,
        rule: &AstGrepRule,
    ) -> Result<Vec<ScanMatch>, AppError> {
        let mut matches = Vec::new();

        if let AstGrepRuleConfig::Pattern(pattern) = &rule.rule {
            // Simple pattern matching for testing
            if let Ok(regex) = Regex::new(pattern) {
                let lines: Vec<&str> = content.lines().collect();
                for (line_num, line) in lines.iter().enumerate() {
                    if regex.is_match(line) {
                        let scan_match = ScanMatch {
                            file_path: file_path.to_string_lossy().to_string(),
                            line_number: line_num + 1,
                            column_number: 1,
                            rule_id: rule.id.clone(),
                            rule_name: pattern.clone(),
                            message: rule.message.clone(),
                            severity: rule.severity.clone(),
                            matched_text: line.to_string(),
                            context: Some(self.get_context(content, line_num, 2)),
                        };
                        matches.push(scan_match);
                    }
                }
            }
        }

        Ok(matches)
    }

    fn apply_pattern_matching(
        &self,
        content: &str,
        file_path: &Path,
        rule: &AstGrepRule,
        pattern: &str,
    ) -> Result<Vec<ScanMatch>, AppError> {
        let mut matches = Vec::new();
        
        // For now, use regex matching as a working implementation
        // TODO: Implement true ast-grep pattern matching
        if let Ok(regex) = Regex::new(pattern) {
            let lines: Vec<&str> = content.lines().collect();
            for (line_num, line) in lines.iter().enumerate() {
                if regex.is_match(line) {
                    let scan_match = ScanMatch {
                        file_path: file_path.to_string_lossy().to_string(),
                        line_number: line_num + 1,
                        column_number: 1,
                        rule_id: rule.id.clone(),
                        rule_name: pattern.to_string(),
                        message: rule.message.clone(),
                        severity: rule.severity.clone(),
                        matched_text: line.to_string(),
                        context: Some(self.get_context(content, line_num, 2)),
                    };
                    matches.push(scan_match);
                }
            }
        }
        
        Ok(matches)
    }

    // TODO: Fix ast-grep API usage - currently disabled due to compilation errors
    /*
    fn try_ast_grep_pattern(
        &self,
        content: &str,
        language: &str,
        pattern: &str,
    ) -> Result<Option<Vec<SimpleAstMatch>>, AppError> {
        // Get the appropriate language
        let ts_lang = match self.get_ts_language(language) {
            Some(lang) => lang,
            None => return Ok(None), // Language not supported
        };

        // Create ast-grep source
        let source = ts_lang.ast_grep(content).map_err(|e| {
            AppError::Config(crate::errors::ConfigError::Other(
                format!("Failed to parse source with ast-grep: {:?}", e)
            ))
        })?;

        // Create a simple pattern rule
        let pattern_rule = format!("pattern: |\n  {}", pattern);
        let rule_core: SerializableRuleCore = serde_yaml::from_str(&pattern_rule)
            .map_err(|e| AppError::Config(crate::errors::ConfigError::Other(
                format!("Failed to parse pattern into SerializableRuleCore: {}", e)
            )))?;

        // Create rule config
        let rule_config = ast_grep_config::RuleConfig::try_from(rule_core)
            .map_err(|e| AppError::Config(crate::errors::ConfigError::Other(
                format!("Failed to create RuleConfig: {:?}", e)
            )))?;

        // Find matches
        let node_matches = source.root().find_all(&rule_config);
        
        let mut ast_matches = Vec::new();
        for node_match in node_matches {
            let node = node_match.get_node();
            let start_pos = node.start_position();
            let matched_text = node.utf8_text(content.as_bytes())
                .unwrap_or("<unable to extract text>")
                .to_string();

            ast_matches.push(SimpleAstMatch {
                line: start_pos.row + 1,
                column: start_pos.column + 1,
                text: matched_text,
            });
        }
        
        Ok(Some(ast_matches))
    }

    fn get_ts_language(&self, language: &str) -> Option<TSLanguage> {
        match language {
            "rust" => Some(TSLanguage::Rust),
            "javascript" => Some(TSLanguage::JavaScript),
            "typescript" => Some(TSLanguage::TypeScript),
            "python" => Some(TSLanguage::Python),
            "java" => Some(TSLanguage::Java),
            "c" => Some(TSLanguage::C),
            "cpp" => Some(TSLanguage::Cpp),
            "go" => Some(TSLanguage::Go),
            "html" => Some(TSLanguage::Html),
            "css" => Some(TSLanguage::Css),
            _ => None,
        }
    }
    */

    fn build_scan_result(&self, args: &ScanArgs, matches: &[ScanMatch], files_scanned: &[PathBuf], rules_count: usize) -> Result<ScanResult, AppError> {
        let repo_name = self.get_repository_name()?;
        let commit_id = self.get_current_commit_id()?;
        let scan_type = if args.full { "full" } else { "incremental" };
        
        let mut by_severity = HashMap::new();
        let mut by_rule = HashMap::new();
        let mut by_file = HashMap::new();
        
        for match_item in matches {
            *by_severity.entry(match_item.severity.clone()).or_insert(0) += 1;
            *by_rule.entry(match_item.rule_id.clone()).or_insert(0) += 1;
            *by_file.entry(match_item.file_path.clone()).or_insert(0) += 1;
        }
        
        let summary = ScanSummary {
            total_matches: matches.len(),
            by_severity,
            by_rule,
            by_file,
        };
        
        let scan_result = ScanResult {
            scan_id: format!("{}_{}", commit_id, chrono::Utc::now().timestamp()),
            repository: repo_name,
            commit_id,
            scan_type: scan_type.to_string(),
            scan_time: chrono::Utc::now().to_rfc3339(),
            rules_count,
            files_scanned: files_scanned.len(),
            matches: matches.to_vec(),
            summary,
        };
        
        Ok(scan_result)
    }

    fn get_repository_name(&self) -> Result<String, AppError> {
        if let Some(repo) = &self.repo {
            if let Ok(remote) = repo.find_remote("origin") {
                if let Some(url) = remote.url() {
                    // Extract repo name from URL
                    let repo_name = url.split('/').last()
                        .unwrap_or("unknown")
                        .trim_end_matches(".git");
                    return Ok(repo_name.to_string());
                }
            }
        }
        
        // Fallback to current directory name
        std::env::current_dir()
            .map_err(|e| AppError::FileRead("current directory".to_string(), e))?
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .ok_or_else(|| AppError::Config(crate::errors::ConfigError::Other("Could not determine repository name".to_string())))
    }

    fn get_current_commit_id(&self) -> Result<String, AppError> {
        if let Some(repo) = &self.repo {
            let head = repo.head().map_err(|e| AppError::Git(crate::errors::GitError::Other(e.to_string())))?;
            if let Some(oid) = head.target() {
                return Ok(oid.to_string());
            }
        }
        
        // Fallback to timestamp if not in a git repository
        Ok(format!("no-git-{}", chrono::Utc::now().timestamp()))
    }
    pub fn save_results(&self, result: &ScanResult, output_path: Option<&str>) -> Result<(), AppError> {
        let results_path = if let Some(path) = output_path {
            PathBuf::from(path)
        } else {
            let base_path = shellexpand::tilde(&self.config.scan.results_path);
            let mut path = PathBuf::from(base_path.as_ref());
            path.push(&result.repository);
            
            // Create directory if it doesn't exist
            if !path.exists() {
                std::fs::create_dir_all(&path)
                    .map_err(|e| AppError::FileWrite(path.to_string_lossy().to_string(), e))?;
            }
            
            path.push(format!("{}.json", result.commit_id));
            path
        };
        
        let json_content = serde_json::to_string_pretty(result)
            .map_err(|e| AppError::Config(crate::errors::ConfigError::Other(format!("Failed to serialize scan result: {}", e))))?;
        
        std::fs::write(&results_path, json_content)
            .map_err(|e| AppError::FileWrite(results_path.to_string_lossy().to_string(), e))?;
        
        println!("Scan results saved to: {}", results_path.display());
        Ok(())
    }
}

// Remote Scanner Framework
// This provides an extensible framework for remote scanning services
pub trait RemoteScanner {
    /// Perform a remote scan
    async fn scan_remote(&self, args: &ScanArgs, rules: &[AstGrepRule]) -> Result<ScanResult, AppError>;
    
    /// Upload files to remote scanning service
    async fn upload_files(&self, files: &[PathBuf]) -> Result<String, AppError>;
    
    /// Get scan results from remote service
    async fn get_results(&self, scan_id: &str) -> Result<ScanResult, AppError>;
    
    /// Check if remote service is available
    async fn is_available(&self) -> Result<bool, AppError>;
}

// Default placeholder implementation for future extension
pub struct DefaultRemoteScanner {
    config: AppConfig,
}

impl DefaultRemoteScanner {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
}

impl RemoteScanner for DefaultRemoteScanner {
    async fn scan_remote(&self, _args: &ScanArgs, _rules: &[AstGrepRule]) -> Result<ScanResult, AppError> {
        Err(AppError::Config(crate::errors::ConfigError::Other(
            "Remote scanning is not yet implemented".to_string()
        )))
    }
    
    async fn upload_files(&self, _files: &[PathBuf]) -> Result<String, AppError> {
        Err(AppError::Config(crate::errors::ConfigError::Other(
            "Remote file upload is not yet implemented".to_string()
        )))
    }
    
    async fn get_results(&self, _scan_id: &str) -> Result<ScanResult, AppError> {
        Err(AppError::Config(crate::errors::ConfigError::Other(
            "Remote result retrieval is not yet implemented".to_string()
        )))
    }
    
    async fn is_available(&self) -> Result<bool, AppError> {
        Ok(false) // Remote scanning not available by default
    }
}


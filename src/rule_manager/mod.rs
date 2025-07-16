use crate::config::RuleManagerConfig;
use crate::errors::AppError;
use shellexpand;
use std::path::{PathBuf};
use std::time::{Duration, SystemTime};
use std::process::Command;
use serde::{Deserialize, Serialize};
use std::fs;

/// Rule version information for tracking updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleVersion {
    pub commit_hash: String,
    pub last_updated: SystemTime,
    pub rule_count: usize,
    pub validation_passed: bool,
    pub performance_metrics: Option<RulePerformanceMetrics>,
}

/// Performance metrics for rule execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulePerformanceMetrics {
    pub avg_execution_time_ms: f64,
    pub max_execution_time_ms: f64,
    pub total_matches: usize,
    pub benchmark_timestamp: SystemTime,
}

/// Rule validation result
#[derive(Debug, Clone)]
pub struct RuleValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub performance_score: Option<f64>,
}

/// Enhanced rule manager with auto-update capabilities
pub struct RuleManager {
    config: RuleManagerConfig,
    cache_path: PathBuf,
    version_info: Option<RuleVersion>,
}

impl RuleManager {
    pub fn new(config: RuleManagerConfig) -> Result<Self, AppError> {
        // Use the rules path directly since build.rs already handles downloading
        let cache_path = Self::resolve_cache_path(&config.path)?;
        if !cache_path.exists() {
            std::fs::create_dir_all(&cache_path)
                .map_err(|e| AppError::FileWrite(cache_path.to_string_lossy().to_string(), e))?;
        }
        
        // Load existing version info if available
        let version_info = Self::load_version_info(&cache_path).ok();
        
        Ok(Self { config, cache_path, version_info })
    }

    /// Enhanced update with version tracking and validation
    pub async fn update_rules(&mut self) -> Result<(), AppError> {
        // Check if update is needed
        if !self.should_update().await? {
            println!("Rules are up to date, skipping update");
            return Ok(());
        }
        
        println!("Updating rules...");
        
        // Download rules with atomic operation
        let mut temp_version = self.download_rules_atomic().await?;
        
        // Validate new rules before applying
        let validation_result = self.validate_rules().await?;
        
        if !validation_result.is_valid {
            println!("⚠️  Rule validation failed:");
            for error in &validation_result.errors {
                println!("  - {}", error);
            }
            return Err(AppError::Config(crate::errors::ConfigError::Other(
                "Rule validation failed, update aborted".to_string()
            )));
        }
        
        if !validation_result.warnings.is_empty() {
            println!("⚠️  Rule validation warnings:");
            for warning in &validation_result.warnings {
                println!("  - {}", warning);
            }
        }
        
        // Mark validation as passed
        temp_version.validation_passed = true;
        
        // Apply update and save version info
        self.version_info = Some(temp_version);
        self.save_version_info()?;
        
        println!("✅ Rules updated successfully!");
        Ok(())
    }

    pub async fn get_rule_paths(&mut self, force_update: bool) -> Result<Vec<PathBuf>, AppError> {
        // Since build.rs handles downloading, we just need to read the existing rules
        // force_update is ignored since build.rs handles updates

        let mut rule_paths = Vec::new();
        
        // Look for rules in the rules directory (build.rs already downloaded them)
        let rules_dir = self.cache_path.join("rules");
        
        if rules_dir.exists() {
            // Recursively find all .yml files in the rules directory
            self.collect_rule_files(&rules_dir, &mut rule_paths)?;
        } else {
            // If rules directory doesn't exist, the build.rs download might have failed
            tracing::warn!("Rules directory not found at: {:?}. Rules may not have been downloaded properly during build.", rules_dir);
        }
        
        // Also check for any .yml files in the cache root (for backward compatibility)
        for entry in std::fs::read_dir(&self.cache_path)
            .map_err(|e| AppError::FileRead(self.cache_path.to_string_lossy().to_string(), e))? {
            let entry = entry.map_err(|e| AppError::FileRead("directory entry".to_string(), e))?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "yml") {
                rule_paths.push(path);
            }
        }
        Ok(rule_paths)
    }

    /// Check if rules need to be updated based on remote version
    async fn should_update(&self) -> Result<bool, AppError> {
        match self.get_remote_commit_hash().await {
            Ok(remote_hash) => {
                if let Some(version) = &self.version_info {
                    Ok(version.commit_hash != remote_hash)
                } else {
                    Ok(true) // No version info, needs update
                }
            }
            Err(_) => {
                // If we can't check remote, only update if we have no local rules
                let rules_dir = self.cache_path.join("rules");
                Ok(!rules_dir.exists())
            }
        }
    }
    
    /// Get current commit hash from remote repository
    async fn get_remote_commit_hash(&self) -> Result<String, AppError> {
        let git_url = self.config.url.replace("/contents", ".git");
        let output = Command::new("git")
            .args(&["ls-remote", &git_url, "HEAD"])
            .output()
            .map_err(|e| AppError::Git(crate::errors::GitError::Other(format!("Failed to get remote commit hash: {}", e))))?;
        
        if !output.status.success() {
            return Err(AppError::Git(crate::errors::GitError::Other(format!(
                "Failed to get remote commit hash: {}", 
                String::from_utf8_lossy(&output.stderr)
            ))));
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let commit_hash = output_str
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().next())
            .ok_or_else(|| AppError::Git(crate::errors::GitError::Other(
                "Failed to parse remote commit hash".to_string()
            )))?;
        
        Ok(commit_hash.to_string())
    }
    
    /// Download rules with atomic operation and version tracking
    async fn download_rules_atomic(&self) -> Result<RuleVersion, AppError> {
        let rules_repo_path = self.cache_path.join("scan-rules");
        let backup_path = self.cache_path.join("scan-rules.backup");
        
        // Create backup if repository exists
        if rules_repo_path.exists() {
            if backup_path.exists() {
                std::fs::remove_dir_all(&backup_path)
                    .map_err(|e| AppError::FileWrite(backup_path.to_string_lossy().to_string(), e))?;
            }
            std::fs::rename(&rules_repo_path, &backup_path)
                .map_err(|e| AppError::FileWrite(rules_repo_path.to_string_lossy().to_string(), e))?;
        }
        
        let result = self.download_rules_impl().await;
        
        match result {
            Ok(version) => {
                // Success - remove backup
                if backup_path.exists() {
                    std::fs::remove_dir_all(&backup_path)
                        .map_err(|e| AppError::FileWrite(backup_path.to_string_lossy().to_string(), e))?;
                }
                Ok(version)
            }
            Err(e) => {
                // Failure - restore backup
                if backup_path.exists() {
                    if rules_repo_path.exists() {
                        std::fs::remove_dir_all(&rules_repo_path)
                            .map_err(|e| AppError::FileWrite(rules_repo_path.to_string_lossy().to_string(), e))?;
                    }
                    std::fs::rename(&backup_path, &rules_repo_path)
                        .map_err(|e| AppError::FileWrite(rules_repo_path.to_string_lossy().to_string(), e))?;
                }
                Err(e)
            }
        }
    }
    
    /// Internal implementation of rule download
    async fn download_rules_impl(&self) -> Result<RuleVersion, AppError> {
        let rules_repo_path = self.cache_path.join("scan-rules");
        
        if rules_repo_path.exists() && rules_repo_path.join(".git").exists() {
            // Update existing repository
            println!("Updating existing scan rules repository...");
            let output = Command::new("git")
                .args(&["pull", "origin", "main"])
                .current_dir(&rules_repo_path)
                .output()
                .map_err(|e| AppError::Git(crate::errors::GitError::Other(format!("Failed to execute git pull: {}", e))))?;
            
            if !output.status.success() {
                return Err(AppError::Git(crate::errors::GitError::Other(format!(
                    "Git pull failed: {}", 
                    String::from_utf8_lossy(&output.stderr)
                ))));
            }
            println!("Successfully updated scan rules repository");
        } else {
            // Clone repository for the first time
            println!("Cloning scan rules repository...");
            
            // Remove existing directory if it exists but isn't a git repo
            if rules_repo_path.exists() {
                std::fs::remove_dir_all(&rules_repo_path)
                    .map_err(|e| AppError::FileWrite(rules_repo_path.to_string_lossy().to_string(), e))?;
            }
            
            let git_url = self.config.url.replace("/contents", ".git");
            let output = Command::new("git")
                .args(&["clone", &git_url, "scan-rules"])
                .current_dir(&self.cache_path)
                .output()
                .map_err(|e| AppError::Git(crate::errors::GitError::Other(format!("Failed to execute git clone: {}", e))))?;
            
            if !output.status.success() {
                return Err(AppError::Git(crate::errors::GitError::Other(format!(
                    "Git clone failed: {}", 
                    String::from_utf8_lossy(&output.stderr)
                ))));
            }
            println!("Successfully cloned scan rules repository");
        }
        
        // Get current commit hash and count rules
        let commit_hash = self.get_local_commit_hash(&rules_repo_path)?;
        let rule_count = self.count_rules(&rules_repo_path)?;
        
        Ok(RuleVersion {
            commit_hash,
            last_updated: SystemTime::now(),
            rule_count,
            validation_passed: false, // Will be set during validation
            performance_metrics: None,
        })
    }

    fn collect_rule_files(&self, dir: &PathBuf, rule_paths: &mut Vec<PathBuf>) -> Result<(), AppError> {
        for entry in std::fs::read_dir(dir)
            .map_err(|e| AppError::FileRead(dir.to_string_lossy().to_string(), e))? {
            let entry = entry.map_err(|e| AppError::FileRead("directory entry".to_string(), e))?;
            let path = entry.path();
            
            if path.is_dir() {
                // Recursively search subdirectories
                self.collect_rule_files(&path, rule_paths)?;
            } else if path.is_file() && path.extension().map_or(false, |ext| ext == "yml" || ext == "yaml") {
                rule_paths.push(path);
            }
        }
        Ok(())
    }

    /// Get current commit hash from local repository
    fn get_local_commit_hash(&self, repo_path: &PathBuf) -> Result<String, AppError> {
        let output = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .current_dir(repo_path)
            .output()
            .map_err(|e| AppError::Git(crate::errors::GitError::Other(format!("Failed to get local commit hash: {}", e))))?;
        
        if !output.status.success() {
            return Err(AppError::Git(crate::errors::GitError::Other(format!(
                "Failed to get local commit hash: {}", 
                String::from_utf8_lossy(&output.stderr)
            ))));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
    
    /// Count total number of rule files
    fn count_rules(&self, repo_path: &PathBuf) -> Result<usize, AppError> {
        let mut count = 0;
        let rules_dir = repo_path.join("rules");
        
        if rules_dir.exists() {
            let mut paths = Vec::new();
            self.collect_rule_files(&rules_dir, &mut paths)?;
            count = paths.len();
        }
        
        Ok(count)
    }
    
    /// Validate downloaded rules
    async fn validate_rules(&self) -> Result<RuleValidationResult, AppError> {
        let mut result = RuleValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            performance_score: None,
        };
        
        let rules_repo_path = self.cache_path.join("scan-rules");
        let rules_dir = rules_repo_path.join("rules");
        
        if !rules_dir.exists() {
            result.is_valid = false;
            result.errors.push("Rules directory not found".to_string());
            return Ok(result);
        }
        
        let mut rule_paths = Vec::new();
        self.collect_rule_files(&rules_dir, &mut rule_paths)?;
        
        if rule_paths.is_empty() {
            result.is_valid = false;
            result.errors.push("No rule files found".to_string());
            return Ok(result);
        }
        
        // Validate each rule file
        let mut valid_count = 0;
        for rule_path in &rule_paths {
            match self.validate_single_rule(rule_path) {
                Ok(is_valid) => {
                    if is_valid {
                        valid_count += 1;
                    } else {
                        result.warnings.push(format!("Invalid rule file: {}", rule_path.display()));
                    }
                }
                Err(e) => {
                    result.errors.push(format!("Failed to validate {}: {}", rule_path.display(), e));
                }
            }
        }
        
        // Require at least 80% of rules to be valid
        let validation_threshold = 0.8;
        let validation_rate = valid_count as f64 / rule_paths.len() as f64;
        
        if validation_rate < validation_threshold {
            result.is_valid = false;
            result.errors.push(format!(
                "Validation rate {:.1}% is below threshold {:.1}%", 
                validation_rate * 100.0, 
                validation_threshold * 100.0
            ));
        }
        
        result.performance_score = Some(validation_rate);
        
        println!("Rule validation: {}/{} rules valid ({:.1}%)", 
                valid_count, rule_paths.len(), validation_rate * 100.0);
        
        Ok(result)
    }
    
    /// Validate a single rule file
    fn validate_single_rule(&self, rule_path: &PathBuf) -> Result<bool, AppError> {
        let content = fs::read_to_string(rule_path)
            .map_err(|e| AppError::FileRead(rule_path.to_string_lossy().to_string(), e))?;
        
        // Basic YAML validation
        match serde_yaml::from_str::<serde_yaml::Value>(&content) {
            Ok(yaml_value) => {
                // Check required fields
                if let Some(mapping) = yaml_value.as_mapping() {
                    let has_id = mapping.contains_key(&serde_yaml::Value::String("id".to_string()));
                    let has_language = mapping.contains_key(&serde_yaml::Value::String("language".to_string()));
                    let has_rule = mapping.contains_key(&serde_yaml::Value::String("rule".to_string()));
                    
                    return Ok(has_id && has_language && has_rule);
                }
                Ok(false)
            }
            Err(_) => Ok(false),
        }
    }
    
    /// Load version information from cache
    fn load_version_info(cache_path: &PathBuf) -> Result<RuleVersion, AppError> {
        let version_path = cache_path.join(".version.json");
        let content = fs::read_to_string(&version_path)
            .map_err(|e| AppError::FileRead(version_path.to_string_lossy().to_string(), e))?;
        
        serde_json::from_str(&content)
            .map_err(|e| AppError::Config(crate::errors::ConfigError::Other(format!("Failed to parse version info: {}", e))))
    }
    
    /// Save version information to cache
    fn save_version_info(&self) -> Result<(), AppError> {
        if let Some(version) = &self.version_info {
            let version_path = self.cache_path.join(".version.json");
            let content = serde_json::to_string_pretty(version)
                .map_err(|e| AppError::Config(crate::errors::ConfigError::Other(format!("Failed to serialize version info: {}", e))))?;
            
            fs::write(&version_path, content)
                .map_err(|e| AppError::FileWrite(version_path.to_string_lossy().to_string(), e))?;
        }
        Ok(())
    }
    
    /// Get current rule version information
    pub fn get_version_info(&self) -> Option<&RuleVersion> {
        self.version_info.as_ref()
    }
    
    /// Force update rules regardless of version
    pub async fn force_update(&mut self) -> Result<(), AppError> {
        self.version_info = None; // Clear version info to force update
        self.update_rules().await
    }
    
    fn resolve_cache_path(path_str: &str) -> Result<PathBuf, AppError> {
        let expanded_path = shellexpand::tilde(path_str);
        Ok(PathBuf::from(expanded_path.into_owned()))
    }
}
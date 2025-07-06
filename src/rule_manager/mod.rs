use crate::config::RuleManagerConfig;
use crate::errors::AppError;
use shellexpand;
use std::path::{PathBuf};
use std::time::{Duration, SystemTime};
use std::process::Command;

pub struct RuleManager {
    config: RuleManagerConfig,
    cache_path: PathBuf,
}

impl RuleManager {
    pub fn new(config: RuleManagerConfig) -> Result<Self, AppError> {
        let cache_path = Self::resolve_cache_path(&config.cache_path)?;
        if !cache_path.exists() {
            std::fs::create_dir_all(&cache_path)
                .map_err(|e| AppError::FileWrite(cache_path.to_string_lossy().to_string(), e))?;
        }
        Ok(Self { config, cache_path })
    }

    pub async fn update_rules(&self) -> Result<(), AppError> {
        self.download_rules().await
    }

    pub async fn get_rule_paths(&self, force_update: bool) -> Result<Vec<PathBuf>, AppError> {
        let ttl = Duration::from_secs(self.config.ttl_hours as u64 * 3600);
        let metadata_path = self.cache_path.join(".metadata");

        let needs_update = if force_update || !metadata_path.exists() {
            true
        } else {
            let metadata = std::fs::metadata(&metadata_path)
                .map_err(|e| AppError::FileRead(metadata_path.to_string_lossy().to_string(), e))?;
            let last_updated = metadata.modified()
                .map_err(|e| AppError::FileRead(metadata_path.to_string_lossy().to_string(), e))?;
            SystemTime::now().duration_since(last_updated).unwrap_or_default() > ttl
        };

        if needs_update {
            self.update_rules().await?;
            std::fs::write(&metadata_path, "")
                .map_err(|e| AppError::FileWrite(metadata_path.to_string_lossy().to_string(), e))?;
        }

        let mut rule_paths = Vec::new();
        
        // Look for rules in the cloned repository
        let rules_repo_path = self.cache_path.join("ast-grep-essentials");
        let rules_dir = rules_repo_path.join("rules");
        
        if rules_dir.exists() {
            // Recursively find all .yml files in the rules directory
            self.collect_rule_files(&rules_dir, &mut rule_paths)?;
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

    async fn download_rules(&self) -> Result<(), AppError> {
        // Check if rules directory already exists and is a git repository
        let rules_repo_path = self.cache_path.join("ast-grep-essentials");
        
        if rules_repo_path.exists() && rules_repo_path.join(".git").exists() {
            // Update existing repository
            println!("Updating existing ast-grep rules repository...");
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
            println!("Successfully updated ast-grep rules repository");
        } else {
            // Clone repository for the first time
            println!("Cloning ast-grep rules repository...");
            
            // Remove existing directory if it exists but isn't a git repo
            if rules_repo_path.exists() {
                std::fs::remove_dir_all(&rules_repo_path)
                    .map_err(|e| AppError::FileWrite(rules_repo_path.to_string_lossy().to_string(), e))?;
            }
            
            let git_url = self.config.url.replace("/contents", ".git");
            let output = Command::new("git")
                .args(&["clone", &git_url, "ast-grep-essentials"])
                .current_dir(&self.cache_path)
                .output()
                .map_err(|e| AppError::Git(crate::errors::GitError::Other(format!("Failed to execute git clone: {}", e))))?;
            
            if !output.status.success() {
                return Err(AppError::Git(crate::errors::GitError::Other(format!(
                    "Git clone failed: {}", 
                    String::from_utf8_lossy(&output.stderr)
                ))));
            }
            println!("Successfully cloned ast-grep rules repository");
        }

        Ok(())
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

    fn resolve_cache_path(path_str: &str) -> Result<PathBuf, AppError> {
        let expanded_path = shellexpand::tilde(path_str);
        Ok(PathBuf::from(expanded_path.into_owned()))
    }
}

use crate::errors::AnalysisError;
use crate::types::git::UpdateRulesArgs;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio;
use tracing::{debug, error, info, warn};

/// Rule manager for handling ast-grep rules updates and management
#[derive(Debug)]
pub struct RuleManager {
    /// Base directory for storing rules
    pub rules_dir: PathBuf,
    /// Configuration for rule sources
    pub sources: HashMap<String, RuleSource>,
    /// HTTP client for downloading rules
    client: reqwest::Client,
}

/// Configuration for a rule source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSource {
    /// Source type (github, url, local)
    pub source_type: String,
    /// Repository or URL
    pub location: String,
    /// Branch or tag reference
    pub reference: String,
    /// Description of the source
    pub description: String,
    /// Whether this source is enabled
    pub enabled: bool,
    /// Last update timestamp
    pub last_updated: Option<u64>,
    /// Source priority (higher = preferred)
    pub priority: u32,
}

/// Metadata for downloaded rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMetadata {
    /// Version of the rules
    pub version: String,
    /// Download timestamp
    pub downloaded_at: u64,
    /// Source information
    pub source: String,
    /// Number of rules
    pub rule_count: usize,
    /// Checksum for verification
    pub checksum: String,
    /// List of included rule files
    pub files: Vec<String>,
}

/// Information about available rule updates
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// Current version
    pub current_version: Option<String>,
    /// Available version
    pub available_version: String,
    /// Whether an update is available
    pub update_available: bool,
    /// Release notes or changelog
    pub changelog: Option<String>,
    /// Download size in bytes
    pub download_size: Option<u64>,
}

impl RuleManager {
    /// Create a new rule manager
    pub fn new(rules_dir: Option<PathBuf>) -> Result<Self, AnalysisError> {
        let rules_dir = rules_dir.unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("gitai")
                .join("rules")
        });

        // Ensure rules directory exists
        if !rules_dir.exists() {
            fs::create_dir_all(&rules_dir).map_err(|e| AnalysisError::IOError(e))?;
        }

        let client = reqwest::Client::builder()
            .user_agent("GitAI-Rule-Manager/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AnalysisError::Generic(format!("Failed to create HTTP client: {}", e)))?;

        let mut manager = Self {
            rules_dir,
            sources: HashMap::new(),
            client,
        };

        manager.initialize_default_sources();
        Ok(manager)
    }

    /// Initialize default rule sources
    fn initialize_default_sources(&mut self) {
        // Official ast-grep rules repository
        self.sources.insert(
            "official".to_string(),
            RuleSource {
                source_type: "github".to_string(),
                location: "coderabbitai/ast-grep-essentials".to_string(),
                reference: "main".to_string(),
                description: "Official ast-grep rules repository".to_string(),
                enabled: true,
                last_updated: None,
                priority: 100,
            },
        );

        // Community rules
        self.sources.insert(
            "community".to_string(),
            RuleSource {
                source_type: "github".to_string(),
                location: "gitai-project/ast-grep-rules".to_string(),
                reference: "main".to_string(),
                description: "GitAI community rules".to_string(),
                enabled: true,
                last_updated: None,
                priority: 80,
            },
        );

        // Security-focused rules
        self.sources.insert(
            "security".to_string(),
            RuleSource {
                source_type: "github".to_string(),
                location: "security-rules/ast-grep-security".to_string(),
                reference: "main".to_string(),
                description: "Security-focused ast-grep rules".to_string(),
                enabled: false,
                last_updated: None,
                priority: 90,
            },
        );
    }

    /// List available rule sources
    pub fn list_sources(&self) -> Vec<(&String, &RuleSource)> {
        let mut sources: Vec<_> = self.sources.iter().collect();
        sources.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));
        sources
    }

    /// Add a new rule source
    pub fn add_source(&mut self, name: String, source: RuleSource) {
        self.sources.insert(name, source);
    }

    /// Remove a rule source
    pub fn remove_source(&mut self, name: &str) -> bool {
        self.sources.remove(name).is_some()
    }

    /// Check for available updates
    pub async fn check_updates(
        &self,
        source_name: Option<&str>,
    ) -> Result<Vec<(String, UpdateInfo)>, AnalysisError> {
        let mut updates = Vec::new();

        let sources_to_check = if let Some(name) = source_name {
            if let Some(source) = self.sources.get(name) {
                vec![(name, source)]
            } else {
                return Err(AnalysisError::Generic(format!(
                    "Source '{}' not found",
                    name
                )));
            }
        } else {
            self.sources
                .iter()
                .filter(|(_, source)| source.enabled)
                .map(|(name, source)| (name.as_str(), source))
                .collect()
        };

        for (name, source) in sources_to_check {
            match self.check_source_update(name, source).await {
                Ok(update_info) => {
                    updates.push((name.to_string(), update_info));
                }
                Err(e) => {
                    warn!("Failed to check updates for source '{}': {}", name, e);
                }
            }
        }

        Ok(updates)
    }

    /// Check for updates from a specific source
    async fn check_source_update(
        &self,
        source_name: &str,
        source: &RuleSource,
    ) -> Result<UpdateInfo, AnalysisError> {
        let current_metadata = self.load_metadata(source_name).ok();
        let current_version = current_metadata.as_ref().map(|m| m.version.clone());

        match source.source_type.as_str() {
            "github" => self.check_github_update(source, current_version).await,
            "url" => self.check_url_update(source, current_version).await,
            "local" => self.check_local_update(source, current_version),
            _ => Err(AnalysisError::Generic(format!(
                "Unsupported source type: {}",
                source.source_type
            ))),
        }
    }

    /// Check for GitHub repository updates
    async fn check_github_update(
        &self,
        source: &RuleSource,
        current_version: Option<String>,
    ) -> Result<UpdateInfo, AnalysisError> {
        let api_url = format!(
            "https://api.github.com/repos/{}/commits/{}",
            source.location, source.reference
        );

        debug!("Checking GitHub updates from: {}", api_url);

        let response = self
            .client
            .get(&api_url)
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| AnalysisError::Generic(format!("GitHub API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AnalysisError::Generic(format!(
                "GitHub API returned status: {}",
                response.status()
            )));
        }

        let commit_info: serde_json::Value = response.json().await.map_err(|e| {
            AnalysisError::Generic(format!("Failed to parse GitHub response: {}", e))
        })?;

        let latest_sha = commit_info["sha"]
            .as_str()
            .unwrap_or("unknown")
            .chars()
            .take(8)
            .collect::<String>();

        let commit_message = commit_info["commit"]["message"]
            .as_str()
            .map(|s| s.to_string());

        let update_available = current_version
            .as_ref()
            .map(|cv| cv != &latest_sha)
            .unwrap_or(true);

        Ok(UpdateInfo {
            current_version,
            available_version: latest_sha,
            update_available,
            changelog: commit_message,
            download_size: None,
        })
    }

    /// Check for URL-based updates
    async fn check_url_update(
        &self,
        source: &RuleSource,
        current_version: Option<String>,
    ) -> Result<UpdateInfo, AnalysisError> {
        let response = self
            .client
            .head(&source.location)
            .send()
            .await
            .map_err(|e| AnalysisError::Generic(format!("URL check failed: {}", e)))?;

        let etag = response
            .headers()
            .get("etag")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string()
            });

        let content_length = response
            .headers()
            .get("content-length")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse().ok());

        let update_available = current_version
            .as_ref()
            .map(|cv| cv != &etag)
            .unwrap_or(true);

        Ok(UpdateInfo {
            current_version,
            available_version: etag,
            update_available,
            changelog: None,
            download_size: content_length,
        })
    }

    /// Check for local file updates
    fn check_local_update(
        &self,
        source: &RuleSource,
        current_version: Option<String>,
    ) -> Result<UpdateInfo, AnalysisError> {
        let path = Path::new(&source.location);
        if !path.exists() {
            return Err(AnalysisError::Generic(format!(
                "Local path does not exist: {}",
                source.location
            )));
        }

        let metadata = path.metadata().map_err(|e| AnalysisError::IOError(e))?;

        let modified_time = metadata
            .modified()
            .map_err(|e| AnalysisError::IOError(e))?
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let version = modified_time.to_string();
        let update_available = current_version
            .as_ref()
            .map(|cv| cv != &version)
            .unwrap_or(true);

        Ok(UpdateInfo {
            current_version,
            available_version: version,
            update_available,
            changelog: None,
            download_size: Some(metadata.len()),
        })
    }

    /// Update rules from a specific source
    pub async fn update_rules(
        &mut self,
        args: &UpdateRulesArgs,
    ) -> Result<RuleMetadata, AnalysisError> {
        let source_name = if let Some(repo) = &args.repository {
            // Custom repository specified
            let custom_source = RuleSource {
                source_type: args.source.clone(),
                location: repo.clone(),
                reference: args.reference.clone(),
                description: format!("Custom source: {}", repo),
                enabled: true,
                last_updated: None,
                priority: 50,
            };

            self.add_source("custom".to_string(), custom_source);
            "custom"
        } else {
            // Use default source
            "official"
        };

        let source = self
            .sources
            .get(source_name)
            .ok_or_else(|| AnalysisError::Generic(format!("Source '{}' not found", source_name)))?
            .clone();

        info!(
            "Updating rules from source: {} ({})",
            source_name, source.location
        );

        // Create backup if requested
        if args.backup {
            self.create_backup(source_name)?;
        }

        // Download and install rules
        let metadata = match source.source_type.as_str() {
            "github" => {
                self.download_github_rules(&source, source_name, args)
                    .await?
            }
            "url" => self.download_url_rules(&source, source_name, args).await?,
            "local" => self.copy_local_rules(&source, source_name, args)?,
            _ => {
                return Err(AnalysisError::Generic(format!(
                    "Unsupported source type: {}",
                    source.source_type
                )));
            }
        };

        // Verify rules if requested
        if args.verify {
            self.verify_rules(source_name, &metadata)?;
        }

        // Update source metadata
        if let Some(source_mut) = self.sources.get_mut(source_name) {
            source_mut.last_updated = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
        }

        // Save metadata
        self.save_metadata(source_name, &metadata)?;

        info!(
            "Successfully updated {} rules from {}",
            metadata.rule_count, source_name
        );
        Ok(metadata)
    }

    /// Download rules from GitHub
    async fn download_github_rules(
        &self,
        source: &RuleSource,
        source_name: &str,
        args: &UpdateRulesArgs,
    ) -> Result<RuleMetadata, AnalysisError> {
        let download_url = format!(
            "https://github.com/{}/archive/{}.zip",
            source.location, source.reference
        );

        debug!("Downloading rules from GitHub: {}", download_url);

        let response = self
            .client
            .get(&download_url)
            .send()
            .await
            .map_err(|e| AnalysisError::Generic(format!("Download failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AnalysisError::Generic(format!(
                "Download failed with status: {}",
                response.status()
            )));
        }

        let content = response
            .bytes()
            .await
            .map_err(|e| AnalysisError::Generic(format!("Failed to read response: {}", e)))?;

        // Extract and install rules
        self.extract_and_install_rules(source_name, &content, args)
            .await
    }

    /// Download rules from URL
    async fn download_url_rules(
        &self,
        source: &RuleSource,
        source_name: &str,
        args: &UpdateRulesArgs,
    ) -> Result<RuleMetadata, AnalysisError> {
        debug!("Downloading rules from URL: {}", source.location);

        let response = self
            .client
            .get(&source.location)
            .send()
            .await
            .map_err(|e| AnalysisError::Generic(format!("Download failed: {}", e)))?;

        let content = response
            .bytes()
            .await
            .map_err(|e| AnalysisError::Generic(format!("Failed to read response: {}", e)))?;

        // Determine if it's a zip file or raw rules
        if source.location.ends_with(".zip") {
            self.extract_and_install_rules(source_name, &content, args)
                .await
        } else {
            // Assume it's a raw rule file
            self.install_raw_rules(source_name, &content, args).await
        }
    }

    /// Copy rules from local path
    fn copy_local_rules(
        &self,
        source: &RuleSource,
        source_name: &str,
        _args: &UpdateRulesArgs,
    ) -> Result<RuleMetadata, AnalysisError> {
        let source_path = Path::new(&source.location);
        let target_dir = self.get_source_dir(source_name);

        if source_path.is_file() {
            // Single file
            let file_name = source_path
                .file_name()
                .ok_or_else(|| AnalysisError::Generic("Invalid file path".to_string()))?;

            let target_file = target_dir.join(file_name);
            fs::copy(source_path, &target_file).map_err(|e| AnalysisError::IOError(e))?;

            Ok(RuleMetadata {
                version: "local".to_string(),
                downloaded_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                source: source_name.to_string(),
                rule_count: 1,
                checksum: "local".to_string(),
                files: vec![file_name.to_string_lossy().to_string()],
            })
        } else if source_path.is_dir() {
            // Directory
            self.copy_directory(source_path, &target_dir)?;
            let files = self.list_rule_files(&target_dir)?;

            Ok(RuleMetadata {
                version: "local".to_string(),
                downloaded_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                source: source_name.to_string(),
                rule_count: files.len(),
                checksum: "local".to_string(),
                files,
            })
        } else {
            Err(AnalysisError::Generic(format!(
                "Path is neither file nor directory: {}",
                source.location
            )))
        }
    }

    /// Extract and install rules from zip content
    async fn extract_and_install_rules(
        &self,
        source_name: &str,
        content: &[u8],
        _args: &UpdateRulesArgs,
    ) -> Result<RuleMetadata, AnalysisError> {
        use std::io::Cursor;
        use zip::ZipArchive;

        let cursor = Cursor::new(content);
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| AnalysisError::Generic(format!("Failed to read zip archive: {}", e)))?;

        let target_dir = self.get_source_dir(source_name);

        // Clear target directory
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir).map_err(|e| AnalysisError::IOError(e))?;
        }
        fs::create_dir_all(&target_dir).map_err(|e| AnalysisError::IOError(e))?;

        let mut extracted_files = Vec::new();

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| AnalysisError::Generic(format!("Failed to extract file: {}", e)))?;

            let file_path = file.mangled_name();

            // Skip directories and non-rule files
            if file.is_dir() || !self.is_rule_file(&file_path) {
                continue;
            }

            // Extract relative path (remove top-level directory)
            let relative_path = file_path.components().skip(1).collect::<PathBuf>();

            if relative_path.as_os_str().is_empty() {
                continue;
            }

            let target_file = target_dir.join(&relative_path);

            // Create parent directories
            if let Some(parent) = target_file.parent() {
                fs::create_dir_all(parent).map_err(|e| AnalysisError::IOError(e))?;
            }

            // Extract file
            let mut target =
                fs::File::create(&target_file).map_err(|e| AnalysisError::IOError(e))?;

            std::io::copy(&mut file, &mut target).map_err(|e| AnalysisError::IOError(e))?;

            extracted_files.push(relative_path.to_string_lossy().to_string());
        }

        Ok(RuleMetadata {
            version: "latest".to_string(),
            downloaded_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            source: source_name.to_string(),
            rule_count: extracted_files.len(),
            checksum: format!("{:x}", md5::compute(content)),
            files: extracted_files,
        })
    }

    /// Install raw rule content
    async fn install_raw_rules(
        &self,
        source_name: &str,
        content: &[u8],
        _args: &UpdateRulesArgs,
    ) -> Result<RuleMetadata, AnalysisError> {
        let target_dir = self.get_source_dir(source_name);

        if !target_dir.exists() {
            fs::create_dir_all(&target_dir).map_err(|e| AnalysisError::IOError(e))?;
        }

        let target_file = target_dir.join("rules.yaml");
        fs::write(&target_file, content).map_err(|e| AnalysisError::IOError(e))?;

        Ok(RuleMetadata {
            version: "latest".to_string(),
            downloaded_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            source: source_name.to_string(),
            rule_count: 1,
            checksum: format!("{:x}", md5::compute(content)),
            files: vec!["rules.yaml".to_string()],
        })
    }

    /// Check if a file is a rule file
    fn is_rule_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            matches!(
                ext.to_str(),
                Some("yaml") | Some("yml") | Some("toml") | Some("json")
            )
        } else {
            false
        }
    }

    /// Copy directory recursively
    fn copy_directory(&self, source: &Path, target: &Path) -> Result<(), AnalysisError> {
        if !target.exists() {
            fs::create_dir_all(target).map_err(|e| AnalysisError::IOError(e))?;
        }

        for entry in fs::read_dir(source).map_err(|e| AnalysisError::IOError(e))? {
            let entry = entry.map_err(|e| AnalysisError::IOError(e))?;
            let source_path = entry.path();
            let target_path = target.join(entry.file_name());

            if source_path.is_dir() {
                self.copy_directory(&source_path, &target_path)?;
            } else if self.is_rule_file(&source_path) {
                fs::copy(&source_path, &target_path).map_err(|e| AnalysisError::IOError(e))?;
            }
        }

        Ok(())
    }

    /// Get directory for a specific source
    fn get_source_dir(&self, source_name: &str) -> PathBuf {
        self.rules_dir.join(source_name)
    }

    /// List rule files in a directory
    fn list_rule_files(&self, dir: &Path) -> Result<Vec<String>, AnalysisError> {
        let mut files = Vec::new();

        if !dir.exists() {
            return Ok(files);
        }

        for entry in fs::read_dir(dir).map_err(|e| AnalysisError::IOError(e))? {
            let entry = entry.map_err(|e| AnalysisError::IOError(e))?;
            let path = entry.path();

            if path.is_file() && self.is_rule_file(&path) {
                if let Some(name) = path.file_name() {
                    files.push(name.to_string_lossy().to_string());
                }
            } else if path.is_dir() {
                let subfiles = self.list_rule_files(&path)?;
                for subfile in subfiles {
                    files.push(format!(
                        "{}/{}",
                        path.file_name().unwrap().to_string_lossy(),
                        subfile
                    ));
                }
            }
        }

        Ok(files)
    }

    /// Create backup of existing rules
    fn create_backup(&self, source_name: &str) -> Result<(), AnalysisError> {
        let source_dir = self.get_source_dir(source_name);
        if !source_dir.exists() {
            return Ok(()); // Nothing to backup
        }

        let backup_dir = self.rules_dir.join("backups").join(format!(
            "{}_{}",
            source_name,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ));

        self.copy_directory(&source_dir, &backup_dir)?;
        info!("Created backup at: {}", backup_dir.display());

        Ok(())
    }

    /// Verify downloaded rules
    fn verify_rules(
        &self,
        source_name: &str,
        metadata: &RuleMetadata,
    ) -> Result<(), AnalysisError> {
        let source_dir = self.get_source_dir(source_name);

        debug!("Verifying rules in: {}", source_dir.display());

        // Check if all files exist
        for file in &metadata.files {
            let file_path = source_dir.join(file);
            if !file_path.exists() {
                return Err(AnalysisError::Generic(format!(
                    "Missing rule file: {}",
                    file
                )));
            }
        }

        // Basic YAML/TOML syntax validation
        for file in &metadata.files {
            let file_path = source_dir.join(file);
            let content = fs::read_to_string(&file_path).map_err(|e| AnalysisError::IOError(e))?;

            if file.ends_with(".yaml") || file.ends_with(".yml") {
                serde_yaml::from_str::<serde_yaml::Value>(&content).map_err(|e| {
                    AnalysisError::Generic(format!("Invalid YAML in {}: {}", file, e))
                })?;
            } else if file.ends_with(".toml") {
                toml::from_str::<toml::Value>(&content).map_err(|e| {
                    AnalysisError::Generic(format!("Invalid TOML in {}: {}", file, e))
                })?;
            } else if file.ends_with(".json") {
                serde_json::from_str::<serde_json::Value>(&content).map_err(|e| {
                    AnalysisError::Generic(format!("Invalid JSON in {}: {}", file, e))
                })?;
            }
        }

        info!("Successfully verified {} rule files", metadata.files.len());
        Ok(())
    }

    /// Save metadata for a source
    fn save_metadata(
        &self,
        source_name: &str,
        metadata: &RuleMetadata,
    ) -> Result<(), AnalysisError> {
        let metadata_file = self.get_source_dir(source_name).join("metadata.json");
        let json = serde_json::to_string_pretty(metadata)
            .map_err(|e| AnalysisError::Generic(format!("Failed to serialize metadata: {}", e)))?;

        fs::write(&metadata_file, json).map_err(|e| AnalysisError::IOError(e))?;

        Ok(())
    }

    /// Load metadata for a source
    fn load_metadata(&self, source_name: &str) -> Result<RuleMetadata, AnalysisError> {
        let metadata_file = self.get_source_dir(source_name).join("metadata.json");
        let content = fs::read_to_string(&metadata_file).map_err(|e| AnalysisError::IOError(e))?;

        serde_json::from_str(&content)
            .map_err(|e| AnalysisError::Generic(format!("Failed to parse metadata: {}", e)))
    }

    /// Get all installed rule sources
    pub fn get_installed_sources(&self) -> Result<Vec<(String, RuleMetadata)>, AnalysisError> {
        let mut installed = Vec::new();

        if !self.rules_dir.exists() {
            return Ok(installed);
        }

        for entry in fs::read_dir(&self.rules_dir).map_err(|e| AnalysisError::IOError(e))? {
            let entry = entry.map_err(|e| AnalysisError::IOError(e))?;
            let source_name = entry.file_name().to_string_lossy().to_string();

            if source_name == "backups" {
                continue;
            }

            if let Ok(metadata) = self.load_metadata(&source_name) {
                installed.push((source_name, metadata));
            }
        }

        Ok(installed)
    }

    /// Clean up old backups
    pub fn cleanup_backups(&self, keep_count: usize) -> Result<usize, AnalysisError> {
        let backup_dir = self.rules_dir.join("backups");
        if !backup_dir.exists() {
            return Ok(0);
        }

        let mut backups = Vec::new();
        for entry in fs::read_dir(&backup_dir).map_err(|e| AnalysisError::IOError(e))? {
            let entry = entry.map_err(|e| AnalysisError::IOError(e))?;
            let path = entry.path();
            if path.is_dir() {
                let metadata = path.metadata().map_err(|e| AnalysisError::IOError(e))?;
                let modified = metadata.modified().map_err(|e| AnalysisError::IOError(e))?;
                backups.push((path, modified));
            }
        }

        // Sort by modification time (oldest first)
        backups.sort_by_key(|(_, modified)| *modified);

        let mut removed = 0;
        if backups.len() > keep_count {
            let to_remove = backups.len() - keep_count;
            for (path, _) in backups.iter().take(to_remove) {
                fs::remove_dir_all(path).map_err(|e| AnalysisError::IOError(e))?;
                removed += 1;
            }
        }

        Ok(removed)
    }

    /// Get rules directory path
    pub fn get_rules_dir(&self) -> &Path {
        &self.rules_dir
    }

    /// Get total disk usage of all rules
    pub fn get_disk_usage(&self) -> Result<u64, AnalysisError> {
        self.calculate_directory_size(&self.rules_dir)
    }

    /// Calculate directory size recursively
    fn calculate_directory_size(&self, dir: &Path) -> Result<u64, AnalysisError> {
        let mut total_size = 0;

        if !dir.exists() {
            return Ok(0);
        }

        for entry in fs::read_dir(dir).map_err(|e| AnalysisError::IOError(e))? {
            let entry = entry.map_err(|e| AnalysisError::IOError(e))?;
            let path = entry.path();

            if path.is_file() {
                let metadata = path.metadata().map_err(|e| AnalysisError::IOError(e))?;
                total_size += metadata.len();
            } else if path.is_dir() {
                total_size += self.calculate_directory_size(&path)?;
            }
        }

        Ok(total_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_rule_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let manager = RuleManager::new(Some(temp_dir.path().to_path_buf()));
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert!(manager.rules_dir.exists());
        assert!(!manager.sources.is_empty());
    }

    #[test]
    fn test_source_management() {
        let temp_dir = tempdir().unwrap();
        let mut manager = RuleManager::new(Some(temp_dir.path().to_path_buf())).unwrap();

        // Test adding source
        let test_source = RuleSource {
            source_type: "github".to_string(),
            location: "test/repo".to_string(),
            reference: "main".to_string(),
            description: "Test source".to_string(),
            enabled: true,
            last_updated: None,
            priority: 50,
        };

        manager.add_source("test".to_string(), test_source);
        assert!(manager.sources.contains_key("test"));

        // Test removing source
        assert!(manager.remove_source("test"));
        assert!(!manager.sources.contains_key("test"));
    }

    #[test]
    fn test_language_detection() {
        let manager = RuleManager::new(None).unwrap();

        assert!(manager.is_rule_file(Path::new("test.yaml")));
        assert!(manager.is_rule_file(Path::new("test.yml")));
        assert!(manager.is_rule_file(Path::new("test.toml")));
        assert!(manager.is_rule_file(Path::new("test.json")));
        assert!(!manager.is_rule_file(Path::new("test.txt")));
        assert!(!manager.is_rule_file(Path::new("test.rs")));
    }
}

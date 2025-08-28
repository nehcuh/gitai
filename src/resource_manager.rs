use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// Metadata for cached resources
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceMetadata {
    pub version: String,
    pub source_url: String,
    pub last_updated: u64,
    pub checksum: Option<String>,
}

/// Resource manager for handling GitAI resources (rules, grammars, etc.)
pub struct ResourceManager {
    cache_dir: PathBuf,
    config: ResourceConfig,
    offline_mode: bool,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResourceConfig {
    pub sources: SourcesConfig,
    pub network: NetworkConfig,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SourcesConfig {
    pub config_url: String,
    pub rules_url: String,
    pub tree_sitter_url: String,
    pub fallback_sources: Vec<String>,
    pub update_check_interval: u64,
    pub auto_update: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NetworkConfig {
    pub proxy: String,
    pub timeout: u64,
    pub retry_times: u32,
    pub offline_mode: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub path: String,
    pub max_size: String,
    pub ttl: u64,
}

impl ResourceManager {
    /// Create a new ResourceManager
    pub fn new(config: ResourceConfig) -> Result<Self> {
        let cache_dir = PathBuf::from(
            shellexpand::tilde(&config.cache.path).to_string()
        );
        
        let mut client_builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.network.timeout));
        
        // Configure proxy if set
        if !config.network.proxy.is_empty() {
            client_builder = client_builder.proxy(reqwest::Proxy::all(&config.network.proxy)?);
        }
        
        let client = client_builder.build()?;
        
        Ok(Self {
            cache_dir,
            offline_mode: config.network.offline_mode,
            config,
            client,
        })
    }
    
    /// Get OpenGrep rules path, downloading if necessary
    pub async fn get_rules(&self) -> Result<PathBuf> {
        let rules_dir = self.cache_dir.join("rules");
        
        // Check if we need to update
        if self.should_update(&rules_dir).await? {
            if !self.offline_mode {
                self.download_rules().await?;
            } else {
                warn!("Offline mode enabled, using cached rules");
            }
        }
        
        // Check if rules exist
        if !rules_dir.exists() || self.is_dir_empty(&rules_dir)? {
            if self.offline_mode {
                anyhow::bail!("No cached rules found in offline mode");
            }
            
            info!("Rules not found, downloading...");
            self.download_rules().await?;
        }
        
        Ok(rules_dir)
    }
    
    /// Download OpenGrep rules
    async fn download_rules(&self) -> Result<()> {
        let rules_dir = self.cache_dir.join("rules");
        fs::create_dir_all(&rules_dir)?;
        
        info!("Downloading OpenGrep rules from {}", self.config.sources.rules_url);
        
        // Try primary source first
        if let Err(e) = self.download_from_git(&self.config.sources.rules_url, &rules_dir).await {
            warn!("Failed to download from primary source: {}", e);
            
            // Try fallback sources
            for fallback in &self.config.sources.fallback_sources {
                let fallback_url = format!("{}/rules", fallback);
                if self.download_from_git(&fallback_url, &rules_dir).await.is_ok() {
                    info!("Downloaded rules from fallback source: {}", fallback);
                    break;
                }
            }
        }
        
        // Update metadata
        self.update_metadata(&rules_dir, &self.config.sources.rules_url).await?;
        
        Ok(())
    }
    
    /// Get Tree-sitter grammar files
    pub async fn get_grammars(&self, language: &str) -> Result<PathBuf> {
        let grammars_dir = self.cache_dir.join("tree-sitter");
        let lang_dir = grammars_dir.join(language);
        
        // Check if we need to update
        if self.should_update(&grammars_dir).await? {
            if !self.offline_mode {
                self.download_grammars(language).await?;
            }
        }
        
        // Check if grammar exists
        if !lang_dir.exists() || self.is_dir_empty(&lang_dir)? {
            if self.offline_mode {
                anyhow::bail!("No cached grammar found for {} in offline mode", language);
            }
            
            info!("Grammar for {} not found, downloading...", language);
            self.download_grammars(language).await?;
        }
        
        Ok(lang_dir)
    }
    
    /// Download Tree-sitter grammar files
    async fn download_grammars(&self, language: &str) -> Result<()> {
        let grammars_dir = self.cache_dir.join("tree-sitter").join(language);
        fs::create_dir_all(&grammars_dir)?;
        
        let url = format!("{}/{}", self.config.sources.tree_sitter_url, language);
        info!("Downloading Tree-sitter grammar for {} from {}", language, url);
        
        // Download grammar files
        self.download_from_url(&url, &grammars_dir).await?;
        
        // Update metadata
        self.update_metadata(&grammars_dir, &url).await?;
        
        Ok(())
    }
    
    /// Download from Git repository
    async fn download_from_git(&self, repo_url: &str, target_dir: &Path) -> Result<()> {
        // For Git repositories, we'll use git command or download archive
        if repo_url.contains("github.com") {
            // Convert to archive URL
            let archive_url = repo_url
                .replace("github.com", "github.com")
                .replace(".git", "/archive/refs/heads/main.zip");
            
            self.download_archive(&archive_url, target_dir).await
        } else {
            // Try to clone with git
            self.git_clone(repo_url, target_dir).await
        }
    }
    
    /// Download and extract archive
    async fn download_archive(&self, url: &str, target_dir: &Path) -> Result<()> {
        let response = self.client.get(url)
            .send()
            .await
            .with_context(|| format!("Failed to download from {}", url))?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to download: HTTP {}", response.status());
        }
        
        let bytes = response.bytes().await?;
        
        // Save to temp file
        let temp_file = target_dir.with_extension("download.tmp");
        fs::write(&temp_file, &bytes)?;
        
        // Extract based on extension
        if url.ends_with(".zip") {
            self.extract_zip(&temp_file, target_dir)?;
        } else if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
            self.extract_tar_gz(&temp_file, target_dir)?;
        } else {
            // Assume it's a direct file
            fs::rename(&temp_file, target_dir.join("rules.yaml"))?;
        }
        
        // Clean up temp file
        if temp_file.exists() {
            fs::remove_file(temp_file).ok();
        }
        
        Ok(())
    }
    
    /// Extract ZIP archive
    fn extract_zip(&self, archive_path: &Path, target_dir: &Path) -> Result<()> {
        use zip::ZipArchive;
        
        let file = fs::File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = target_dir.join(file.name());
            
            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p)?;
                }
                let mut outfile = fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }
        
        Ok(())
    }
    
    /// Extract tar.gz archive
    fn extract_tar_gz(&self, archive_path: &Path, target_dir: &Path) -> Result<()> {
        use flate2::read::GzDecoder;
        use tar::Archive;
        
        let tar_gz = fs::File::open(archive_path)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        archive.unpack(target_dir)?;
        
        Ok(())
    }
    
    /// Clone git repository
    async fn git_clone(&self, repo_url: &str, target_dir: &Path) -> Result<()> {
        use tokio::process::Command;
        
        let output = Command::new("git")
            .arg("clone")
            .arg("--depth=1")
            .arg(repo_url)
            .arg(target_dir)
            .output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git clone failed: {}", stderr);
        }
        
        Ok(())
    }
    
    /// Download from URL
    async fn download_from_url(&self, url: &str, target_dir: &Path) -> Result<()> {
        let response = self.client.get(url)
            .send()
            .await
            .with_context(|| format!("Failed to download from {}", url))?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to download: HTTP {}", response.status());
        }
        
        let content = response.text().await?;
        let target_file = target_dir.join("content.yaml");
        fs::write(target_file, content)?;
        
        Ok(())
    }
    
    /// Check if update is needed
    async fn should_update(&self, resource_dir: &Path) -> Result<bool> {
        if !self.config.sources.auto_update {
            return Ok(false);
        }
        
        let metadata_file = resource_dir.join(".metadata.json");
        if !metadata_file.exists() {
            return Ok(true);
        }
        
        let metadata: ResourceMetadata = serde_json::from_str(
            &fs::read_to_string(metadata_file)?
        )?;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        
        let time_since_update = now - metadata.last_updated;
        
        Ok(time_since_update > self.config.sources.update_check_interval)
    }
    
    /// Update resource metadata
    async fn update_metadata(&self, resource_dir: &Path, source_url: &str) -> Result<()> {
        let metadata = ResourceMetadata {
            version: "1.0.0".to_string(),
            source_url: source_url.to_string(),
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs(),
            checksum: None,
        };
        
        let metadata_file = resource_dir.join(".metadata.json");
        let json = serde_json::to_string_pretty(&metadata)?;
        fs::write(metadata_file, json)?;
        
        Ok(())
    }
    
    /// Check if directory is empty
    fn is_dir_empty(&self, dir: &Path) -> Result<bool> {
        Ok(fs::read_dir(dir)?
            .filter_map(Result::ok)
            .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
            .count() == 0)
    }
    
    /// Update all resources
    pub async fn update_all(&self) -> Result<()> {
        if self.offline_mode {
            warn!("Cannot update resources in offline mode");
            return Ok(());
        }
        
        info!("Updating all resources...");
        
        // Update rules
        if let Err(e) = self.download_rules().await {
            warn!("Failed to update rules: {}", e);
        }
        
        // Update grammars for supported languages
        let languages = vec!["rust", "python", "javascript", "go", "java"];
        for lang in languages {
            if let Err(e) = self.download_grammars(lang).await {
                warn!("Failed to update grammar for {}: {}", lang, e);
            }
        }
        
        info!("Resource update complete");
        Ok(())
    }
    
    /// Clean expired cache
    pub async fn clean_cache(&self) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        
        // Walk through cache directories
        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let metadata_file = path.join(".metadata.json");
                if metadata_file.exists() {
                    let metadata: ResourceMetadata = serde_json::from_str(
                        &fs::read_to_string(&metadata_file)?
                    )?;
                    
                    let age = now - metadata.last_updated;
                    if age > self.config.cache.ttl {
                        info!("Removing expired cache: {:?}", path);
                        fs::remove_dir_all(&path)?;
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Load resource configuration from file
pub fn load_resource_config(config_path: &Path) -> Result<ResourceConfig> {
    let content = fs::read_to_string(config_path)?;
    let config: toml::Value = toml::from_str(&content)?;
    
    // Provide defaults if sections are missing
    let default_sources = toml::Value::Table({
        let mut table = toml::Table::new();
        table.insert("config_url".to_string(), toml::Value::String(
            "https://raw.githubusercontent.com/nehcuh/gitai/main/assets/config.enhanced.toml".to_string()));
        table.insert("rules_url".to_string(), toml::Value::String(
            "https://github.com/nehcuh/gitai-rules.git".to_string()));
        table.insert("tree_sitter_url".to_string(), toml::Value::String(
            "https://github.com/nehcuh/gitai-tree-sitter.git".to_string()));
        table.insert("fallback_sources".to_string(), toml::Value::Array(vec![]));
        table.insert("update_check_interval".to_string(), toml::Value::Integer(86400));
        table.insert("auto_update".to_string(), toml::Value::Boolean(false));
        table
    });
    
    let default_network = toml::Value::Table({
        let mut table = toml::Table::new();
        table.insert("proxy".to_string(), toml::Value::String(String::new()));
        table.insert("timeout".to_string(), toml::Value::Integer(30));
        table.insert("retry_times".to_string(), toml::Value::Integer(3));
        table.insert("offline_mode".to_string(), toml::Value::Boolean(false));
        table
    });
    
    let default_cache = toml::Value::Table({
        let mut table = toml::Table::new();
        table.insert("enabled".to_string(), toml::Value::Boolean(true));
        table.insert("path".to_string(), toml::Value::String("~/.cache/gitai".to_string()));
        table.insert("max_size".to_string(), toml::Value::String("1GB".to_string()));
        table.insert("ttl".to_string(), toml::Value::Integer(604800));
        table
    });
    
    let sources = config.get("sources").unwrap_or(&default_sources);
    let network = config.get("network").unwrap_or(&default_network);
    let cache = config.get("cache").unwrap_or(&default_cache);
    
    // Try to deserialize from TOML values
    let sources_str = toml::to_string(sources)?;
    let network_str = toml::to_string(network)?;
    let cache_str = toml::to_string(cache)?;
    
    let resource_config = ResourceConfig {
        sources: toml::from_str(&sources_str)?,
        network: toml::from_str(&network_str)?,
        cache: toml::from_str(&cache_str)?,
    };
    
    Ok(resource_config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_resource_manager() {
        let temp_dir = TempDir::new().unwrap();
        
        let config = ResourceConfig {
            sources: SourcesConfig {
                config_url: "https://example.com/config.toml".to_string(),
                rules_url: "https://example.com/rules".to_string(),
                tree_sitter_url: "https://example.com/grammars".to_string(),
                fallback_sources: vec![],
                update_check_interval: 86400,
                auto_update: false,
            },
            network: NetworkConfig {
                proxy: String::new(),
                timeout: 30,
                retry_times: 3,
                offline_mode: true,
            },
            cache: CacheConfig {
                enabled: true,
                path: temp_dir.path().to_string_lossy().to_string(),
                max_size: "1GB".to_string(),
                ttl: 604800,
            },
        };
        
        let manager = ResourceManager::new(config).unwrap();
        
        // Test that offline mode prevents downloads
        assert!(manager.get_rules().await.is_err());
    }
}

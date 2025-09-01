use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Configuration initializer for GitAI
/// Handles first-run setup, config migration, and resource initialization
pub struct ConfigInitializer {
    config_dir: PathBuf,
    cache_dir: PathBuf,
    config_url: Option<String>,
    offline_mode: bool,
}

impl Default for ConfigInitializer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigInitializer {
    /// Create a new ConfigInitializer
    pub fn new() -> Self {
        let home = dirs::home_dir().expect("Failed to get home directory");
        let config_dir = home.join(".config").join("gitai");
        let cache_dir = home.join(".cache").join("gitai");

        Self {
            config_dir,
            cache_dir,
            config_url: None,
            offline_mode: false,
        }
    }

    /// Set custom config URL
    pub fn with_config_url(mut self, url: Option<String>) -> Self {
        self.config_url = url;
        self
    }

    /// Set offline mode
    pub fn with_offline_mode(mut self, offline: bool) -> Self {
        self.offline_mode = offline;
        self
    }

    /// Initialize the configuration system
    pub async fn initialize(&self) -> Result<PathBuf> {
        // Create directories if they don't exist
        self.ensure_directories()?;

        // Check if config exists
        let config_path = self.config_dir.join("config.toml");

        if !config_path.exists() {
            info!("No configuration found, initializing...");

            if self.offline_mode {
                // In offline mode, copy from embedded template
                self.copy_default_config(&config_path)?;
            } else if let Some(url) = &self.config_url {
                // Download from specified URL
                self.download_config(url, &config_path).await?;
            } else {
                // Copy from embedded template or try to download default
                if let Err(e) = self.try_download_default_config(&config_path).await {
                    warn!(
                        "Failed to download default config: {}, using embedded template",
                        e
                    );
                    self.copy_default_config(&config_path)?;
                }
            }
        } else {
            // Check if migration is needed
            self.migrate_config_if_needed(&config_path)?;
        }

        // Initialize prompts if needed
        self.initialize_prompts()?;

        // Create version file
        self.create_version_file()?;

        Ok(config_path)
    }

    /// Ensure all required directories exist
    fn ensure_directories(&self) -> Result<()> {
        // Create config directory
        fs::create_dir_all(&self.config_dir)
            .with_context(|| format!("Failed to create config directory: {:?}", self.config_dir))?;

        // Create cache directory and subdirectories
        fs::create_dir_all(&self.cache_dir)
            .with_context(|| format!("Failed to create cache directory: {:?}", self.cache_dir))?;

        fs::create_dir_all(self.cache_dir.join("rules"))?;
        fs::create_dir_all(self.cache_dir.join("tree-sitter"))?;
        fs::create_dir_all(self.cache_dir.join("config"))?;

        debug!("Ensured all directories exist");
        Ok(())
    }

    /// Copy default config from embedded template
    fn copy_default_config(&self, target: &Path) -> Result<()> {
        // First try the enhanced config
        let enhanced_config = include_str!("../assets/config.enhanced.toml");

        fs::write(target, enhanced_config)
            .with_context(|| format!("Failed to write config to {target:?}"))?;

        info!("Created default configuration at {:?}", target);
        Ok(())
    }

    /// Download config from URL
    async fn download_config(&self, url: &str, target: &Path) -> Result<()> {
        info!("Downloading configuration from {}", url);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let response = client
            .get(url)
            .send()
            .await
            .with_context(|| format!("Failed to download config from {url}"))?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download config: HTTP {}", response.status());
        }

        let content = response.text().await?;

        // Validate it's valid TOML
        toml::from_str::<toml::Value>(&content)
            .with_context(|| "Downloaded config is not valid TOML")?;

        fs::write(target, content)?;
        info!("Downloaded configuration to {:?}", target);

        Ok(())
    }

    /// Try to download the default config from GitHub
    async fn try_download_default_config(&self, target: &Path) -> Result<()> {
        let default_url =
            "https://raw.githubusercontent.com/nehcuh/gitai/main/assets/config.enhanced.toml";
        self.download_config(default_url, target).await
    }

    /// Check and migrate config if needed
    fn migrate_config_if_needed(&self, config_path: &Path) -> Result<()> {
        let content = fs::read_to_string(config_path)?;

        // Try to parse as TOML to check version
        if let Ok(mut config) = toml::from_str::<toml::Value>(&content) {
            // Check if version field exists
            if config.get("version").is_none() {
                info!("Migrating configuration to new format...");

                // Add version field
                if let Some(table) = config.as_table_mut() {
                    table.insert(
                        "version".to_string(),
                        toml::Value::String("1.0.0".to_string()),
                    );

                    // Add sources section if not exists
                    if !table.contains_key("sources") {
                        let mut sources = toml::Table::new();
                        sources.insert("config_url".to_string(), 
                            toml::Value::String("https://raw.githubusercontent.com/nehcuh/gitai/main/assets/config.enhanced.toml".to_string()));
                        sources.insert(
                            "rules_url".to_string(),
                            toml::Value::String(
                                "https://github.com/nehcuh/gitai-rules.git".to_string(),
                            ),
                        );
                        sources.insert(
                            "tree_sitter_url".to_string(),
                            toml::Value::String(
                                "https://github.com/nehcuh/gitai-tree-sitter.git".to_string(),
                            ),
                        );

                        table.insert("sources".to_string(), toml::Value::Table(sources));
                    }

                    // Add network section if not exists
                    if !table.contains_key("network") {
                        let mut network = toml::Table::new();
                        network.insert("proxy".to_string(), toml::Value::String("".to_string()));
                        network.insert("timeout".to_string(), toml::Value::Integer(30));
                        network.insert("retry_times".to_string(), toml::Value::Integer(3));
                        network.insert("offline_mode".to_string(), toml::Value::Boolean(false));

                        table.insert("network".to_string(), toml::Value::Table(network));
                    }

                    // Add cache section if not exists
                    if !table.contains_key("cache") {
                        let mut cache = toml::Table::new();
                        cache.insert("enabled".to_string(), toml::Value::Boolean(true));
                        cache.insert(
                            "path".to_string(),
                            toml::Value::String(self.cache_dir.to_string_lossy().to_string()),
                        );
                        cache.insert(
                            "max_size".to_string(),
                            toml::Value::String("1GB".to_string()),
                        );
                        cache.insert("ttl".to_string(), toml::Value::Integer(604800));

                        table.insert("cache".to_string(), toml::Value::Table(cache));
                    }

                    // Backup old config
                    let backup_path = config_path.with_extension("toml.backup");
                    fs::copy(config_path, &backup_path)?;
                    info!("Backed up old configuration to {:?}", backup_path);

                    // Write migrated config
                    let migrated = toml::to_string_pretty(&config)?;
                    fs::write(config_path, migrated)?;
                    info!("Configuration migrated successfully");
                }
            }
        }

        Ok(())
    }

    /// Initialize prompt templates
    fn initialize_prompts(&self) -> Result<()> {
        let prompts_dir = self.config_dir.join("prompts");

        if !prompts_dir.exists() {
            fs::create_dir_all(&prompts_dir)?;

            // Copy default prompts from assets
            let commit_prompt = include_str!("../assets/prompts/commit.md");
            let review_prompt = include_str!("../assets/prompts/review.md");
            let deviation_prompt = include_str!("../assets/prompts/deviation.md");

            fs::write(prompts_dir.join("commit.md"), commit_prompt)?;
            fs::write(prompts_dir.join("review.md"), review_prompt)?;
            fs::write(prompts_dir.join("deviation.md"), deviation_prompt)?;

            info!("Initialized default prompt templates");
        }

        Ok(())
    }

    /// Create version file for tracking
    fn create_version_file(&self) -> Result<()> {
        let version_file = self.config_dir.join(".version");

        #[derive(Serialize, Deserialize)]
        struct VersionInfo {
            config_version: String,
            last_update_check: u64,
            gitai_version: String,
        }

        let version_info = VersionInfo {
            config_version: "1.0.0".to_string(),
            last_update_check: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            gitai_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let json = serde_json::to_string_pretty(&version_info)?;
        fs::write(version_file, json)?;

        Ok(())
    }

    /// Check if initialization is needed
    pub fn needs_init(&self) -> bool {
        !self.config_dir.join("config.toml").exists()
    }

    /// Get config directory path
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Get cache directory path  
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

/// Initialize configuration with environment variable overrides
pub async fn init_with_env() -> Result<PathBuf> {
    let mut initializer = ConfigInitializer::new();

    // Check for environment variable overrides
    if let Ok(url) = std::env::var("GITAI_CONFIG_URL") {
        initializer = initializer.with_config_url(Some(url));
    }

    if let Ok(offline) = std::env::var("GITAI_OFFLINE") {
        initializer = initializer.with_offline_mode(offline.to_lowercase() == "true");
    }

    initializer.initialize().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_config_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join(".config/gitai");
        let cache_dir = temp_dir.path().join(".cache/gitai");

        // Create initializer with test directories
        let initializer = ConfigInitializer {
            config_dir,
            cache_dir,
            config_url: None,
            offline_mode: true, // Use offline mode for testing
        };

        // Initialize
        let config_path = initializer.initialize().await.unwrap();

        // Check that config was created
        assert!(config_path.exists());

        // Check that directories were created
        assert!(initializer.config_dir.exists());
        assert!(initializer.cache_dir.exists());
        assert!(initializer.cache_dir.join("rules").exists());
        assert!(initializer.cache_dir.join("tree-sitter").exists());

        // Check that prompts were initialized
        assert!(initializer.config_dir.join("prompts").exists());
            assert!(initializer
            .config_dir
            .join("prompts/commit.md")
            .exists());
    }
}

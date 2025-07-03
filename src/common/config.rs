// 新的配置管理系统
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::common::{AppResult, AppError};
use crate::common::types::SupportedLanguage;
use crate::common::utils::{expand_path, ensure_dir_exists, read_file_safe, write_file_safe};

/// 配置文件路径常量
pub struct ConfigPaths {
    pub user_config_dir: PathBuf,
    pub config_file: PathBuf,
    pub prompts_dir: PathBuf,
    pub rules_dir: PathBuf,
}

impl ConfigPaths {
    /// Creates a new `ConfigPaths` instance with default directories for configuration, prompts, and rules under `~/.config/gitai`.
    ///
    /// The returned struct contains paths for the main config file, prompts directory, rules directory, and the user config directory itself.
    ///
    /// # Examples
    ///
    /// ```
    /// let paths = ConfigPaths::new();
    /// assert!(paths.config_file.ends_with("config.toml"));
    /// ```
    pub fn new() -> Self {
        let config_dir = expand_path("~/.config/gitai");
        Self {
            config_file: config_dir.join("config.toml"),
            prompts_dir: config_dir.join("prompts"),
            rules_dir: config_dir.join("rules"),
            user_config_dir: config_dir,
        }
    }

    /// Ensures that the user config, prompts, and rules directories exist, creating them if necessary.
    ///
    /// Returns an error if any directory cannot be created.
    pub fn ensure_dirs_exist(&self) -> AppResult<()> {
        ensure_dir_exists(&self.user_config_dir)?;
        ensure_dir_exists(&self.prompts_dir)?;
        ensure_dir_exists(&self.rules_dir)?;
        Ok(())
    }
}

/// 主配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitAIConfig {
    pub ai: AIConfig,
    pub git: GitConfig,
    pub translation: TranslationConfig,
    pub devops: Option<DevOpsConfig>,
    pub general: GeneralConfig,
}

impl Default for GitAIConfig {
    /// Returns a `GitAIConfig` instance with default values for all configuration sections.
    ///
    /// The AI, Git, Translation, and General configurations are set to their respective defaults.
    /// The DevOps configuration is set to `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// let config = GitAIConfig::default();
    /// assert!(config.devops.is_none());
    /// ```
    fn default() -> Self {
        Self {
            ai: AIConfig::default(),
            git: GitConfig::default(),
            translation: TranslationConfig::default(),
            devops: None,
            general: GeneralConfig::default(),
        }
    }
}

/// AI 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub api_url: String,
    pub model_name: String,
    pub temperature: f32,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
}

impl Default for AIConfig {
    /// Returns the default AI configuration for the application.
    ///
    /// The default settings use the OpenAI API with the "gpt-3.5-turbo" model, a temperature of 0.7, a 30-second timeout, and 3 retries. Optional fields such as API key, max tokens, top_p, and top_k are unset.
    ///
    /// # Examples
    ///
    /// ```
    /// let default_ai = AIConfig::default();
    /// assert_eq!(default_ai.model_name, "gpt-3.5-turbo");
    /// ```
    fn default() -> Self {
        Self {
            api_url: "https://api.openai.com/v1/chat/completions".to_string(),
            model_name: "gpt-3.5-turbo".to_string(),
            temperature: 0.7,
            api_key: None,
            timeout_seconds: 30,
            max_retries: 3,
            max_tokens: None,
            top_p: None,
            top_k: None,
        }
    }
}

/// Git 相关配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub auto_stage: bool,
    pub verify_ssl: bool,
    pub commit_template: Option<String>,
    pub default_branch: String,
    pub excluded_files: Vec<String>,
    pub max_diff_lines: usize,
    pub max_commit_message_length: usize,
}

impl Default for GitConfig {
    /// Returns the default Git configuration settings for the application.
    ///
    /// The defaults include no auto-staging, SSL verification enabled, no commit template,
    /// "main" as the default branch, exclusion of common sensitive files, a maximum of 1000 diff lines,
    /// and a maximum commit message length of 100 characters.
    fn default() -> Self {
        Self {
            auto_stage: false,
            verify_ssl: true,
            commit_template: None,
            default_branch: "main".to_string(),
            excluded_files: vec![
                ".env".to_string(),
                ".env.local".to_string(),
                "*.key".to_string(),
                "*.pem".to_string(),
            ],
            max_diff_lines: 1000,
            max_commit_message_length: 100,
        }
    }
}

/// 翻译配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationConfig {
    pub default_language: SupportedLanguage,
    pub fallback_language: SupportedLanguage,
    pub cache_enabled: bool,
    pub cache_ttl_hours: u64,
}

impl Default for TranslationConfig {
    /// Returns the default translation configuration.
    ///
    /// The default uses automatic language detection, English as a fallback, enables caching, and sets the cache TTL to 24 hours.
    fn default() -> Self {
        Self {
            default_language: SupportedLanguage::Auto,
            fallback_language: SupportedLanguage::English,
            cache_enabled: true,
            cache_ttl_hours: 24,
        }
    }
}

/// DevOps 集成配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevOpsConfig {
    pub platform: String,
    pub base_url: String,
    pub token: String,
    pub timeout_seconds: u64,
    pub retry_count: u32,
    pub project_id: Option<String>,
    pub workspace_id: Option<String>,
}

impl Default for DevOpsConfig {
    /// Returns the default DevOps configuration for GitAI.
    ///
    /// The default configuration uses the GitHub platform with its API URL, an empty token,
    /// a 30-second timeout, 3 retries, and no project or workspace IDs specified.
    fn default() -> Self {
        Self {
            platform: "github".to_string(),
            base_url: "https://api.github.com".to_string(),
            token: String::new(),
            timeout_seconds: 30,
            retry_count: 3,
            project_id: None,
            workspace_id: None,
        }
    }
}

/// 通用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub log_level: String,
    pub output_format: String,
    pub color_enabled: bool,
    pub progress_enabled: bool,
    pub parallel_processing: bool,
    pub max_parallel_jobs: usize,
}

impl Default for GeneralConfig {
    /// Returns the default general configuration settings for the application.
    ///
    /// The default values include "info" log level, "text" output format, color and progress enabled,
    /// parallel processing enabled, and the maximum number of parallel jobs set to the number of CPU cores.
    ///
    /// # Examples
    ///
    /// ```
    /// let general_config = GeneralConfig::default();
    /// assert_eq!(general_config.log_level, "info");
    /// assert!(general_config.color_enabled);
    /// ```
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            output_format: "text".to_string(),
            color_enabled: true,
            progress_enabled: true,
            parallel_processing: true,
            max_parallel_jobs: num_cpus::get(),
        }
    }
}

/// 配置管理器
pub struct ConfigManager {
    paths: ConfigPaths,
    config: GitAIConfig,
    prompts: HashMap<String, String>,
}

impl ConfigManager {
    /// Creates and initializes a new configuration manager.
    ///
    /// Loads configuration from disk or creates a default configuration if none exists, ensures required directories are present, and loads prompt files into memory.
    ///
    /// # Returns
    ///
    /// A result containing the initialized `ConfigManager` or an error if initialization fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let manager = ConfigManager::new().unwrap();
    /// assert!(manager.config().ai.api_url.contains("openai"));
    /// ```
    pub fn new() -> AppResult<Self> {
        let paths = ConfigPaths::new();
        paths.ensure_dirs_exist()?;
        
        let config = Self::load_config(&paths)?;
        let prompts = Self::load_prompts(&paths)?;

        Ok(Self {
            paths,
            config,
            prompts,
        })
    }

    /// Loads the GitAI configuration from the specified file path, creating a default configuration if the file does not exist.
    ///
    /// If the configuration file is missing, a default configuration is created and saved. The loaded configuration is then updated with any relevant environment variable overrides.
    ///
    /// # Returns
    /// The loaded and environment-overridden `GitAIConfig` instance.
    ///
    /// # Errors
    /// Returns an error if reading, parsing, or saving the configuration file fails.
    fn load_config(paths: &ConfigPaths) -> AppResult<GitAIConfig> {
        if !paths.config_file.exists() {
            tracing::info!("配置文件不存在，创建默认配置: {:?}", paths.config_file);
            let default_config = GitAIConfig::default();
            Self::save_config_to_file(&default_config, &paths.config_file)?;
            return Ok(default_config);
        }

        let content = read_file_safe(&paths.config_file)?;
        let mut config: GitAIConfig = toml::from_str(&content)
            .map_err(|e| AppError::config_with_source("配置文件解析失败", e))?;

        // 从环境变量覆盖配置
        Self::apply_env_overrides(&mut config)?;

        Ok(config)
    }

    /// Applies environment variable overrides to the provided configuration.
    ///
    /// This function updates fields in the given `GitAIConfig` based on the presence of specific environment variables.
    /// AI, DevOps, and general configuration fields are overridden if their corresponding environment variables are set.
    ///
    /// Environment variables checked:
    /// - AI: `GITAI_API_KEY`, `GITAI_API_URL`, `GITAI_MODEL`
    /// - DevOps: `GITAI_DEVOPS_PLATFORM`, `GITAI_DEVOPS_BASE_URL`, `GITAI_DEVOPS_TOKEN`
    /// - General: `GITAI_LOG_LEVEL`
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if overrides are applied successfully.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::{GitAIConfig, apply_env_overrides};
    /// std::env::set_var("GITAI_API_KEY", "test-key");
    /// let mut config = GitAIConfig::default();
    /// apply_env_overrides(&mut config).unwrap();
    /// assert_eq!(config.ai.api_key.as_deref(), Some("test-key"));
    /// ```
    fn apply_env_overrides(config: &mut GitAIConfig) -> AppResult<()> {
        // AI 配置
        if let Ok(api_key) = std::env::var("GITAI_API_KEY") {
            config.ai.api_key = Some(api_key);
        }
        if let Ok(api_url) = std::env::var("GITAI_API_URL") {
            config.ai.api_url = api_url;
        }
        if let Ok(model) = std::env::var("GITAI_MODEL") {
            config.ai.model_name = model;
        }

        // DevOps 配置
        if let Ok(platform) = std::env::var("GITAI_DEVOPS_PLATFORM") {
            let devops = config.devops.get_or_insert_with(DevOpsConfig::default);
            devops.platform = platform;
        }
        if let Ok(base_url) = std::env::var("GITAI_DEVOPS_BASE_URL") {
            let devops = config.devops.get_or_insert_with(DevOpsConfig::default);
            devops.base_url = base_url;
        }
        if let Ok(token) = std::env::var("GITAI_DEVOPS_TOKEN") {
            let devops = config.devops.get_or_insert_with(DevOpsConfig::default);
            devops.token = token;
        }

        // 通用配置
        if let Ok(log_level) = std::env::var("GITAI_LOG_LEVEL") {
            config.general.log_level = log_level;
        }

        Ok(())
    }

    /// Loads predefined prompt files from the prompts directory into a hashmap.
    ///
    /// Attempts to read a set of known prompt markdown files from the specified prompts directory.
    /// Each successfully loaded file is inserted into the hashmap with its filename (without the `.md` extension) as the key.
    /// Missing files are logged as warnings.
    ///
    /// # Returns
    /// A hashmap mapping prompt names to their file contents.
    ///
    /// # Examples
    ///
    /// ```
    /// let paths = ConfigPaths::new();
    /// let prompts = load_prompts(&paths).unwrap();
    /// assert!(prompts.contains_key("helper-prompt"));
    /// ```
    fn load_prompts(paths: &ConfigPaths) -> AppResult<HashMap<String, String>> {
        let mut prompts = HashMap::new();
        
        let prompt_files = [
            "helper-prompt.md",
            "translator.md", 
            "commit-generator.md",
            "commit-deviation.md",
            "review.md"
        ];

        for filename in &prompt_files {
            let prompt_path = paths.prompts_dir.join(filename);
            if prompt_path.exists() {
                let content = read_file_safe(&prompt_path)?;
                let key = filename.strip_suffix(".md").unwrap_or(filename);
                prompts.insert(key.to_string(), content);
            } else {
                tracing::warn!("提示文件不存在: {:?}", prompt_path);
            }
        }

        Ok(prompts)
    }

    /// Saves the provided configuration to a file in pretty TOML format.
    ///
    /// Serializes the given `GitAIConfig` and writes it to the specified file path, overwriting any existing content.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails or if the file cannot be written.
    fn save_config_to_file(config: &GitAIConfig, path: &Path) -> AppResult<()> {
        let content = toml::to_string_pretty(config)
            .map_err(|e| AppError::config_with_source("配置序列化失败", e))?;
        write_file_safe(path, &content)?;
        Ok(())
    }

    /// Saves the current configuration to the configuration file on disk.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the configuration was successfully saved, or an error if the operation fails.
    pub fn save_config(&self) -> AppResult<()> {
        Self::save_config_to_file(&self.config, &self.paths.config_file)
    }

    /// Returns a reference to the current GitAI configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// let manager = ConfigManager::new().unwrap();
    /// let config = manager.config();
    /// assert_eq!(config.ai.model_name, "gpt-3.5-turbo");
    /// ```
    pub fn config(&self) -> &GitAIConfig {
        &self.config
    }

    /// Returns a mutable reference to the current GitAI configuration.
    ///
    /// Allows modification of the application's configuration in place.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut manager = ConfigManager::new().unwrap();
    /// manager.config_mut().ai.temperature = 1.0;
    /// ```
    pub fn config_mut(&mut self) -> &mut GitAIConfig {
        &mut self.config
    }

    /// Returns the content of a prompt by its name, if it exists.
    ///
    /// # Examples
    ///
    /// ```
    /// let manager = ConfigManager::new().unwrap();
    /// if let Some(content) = manager.get_prompt("commit") {
    ///     assert!(content.contains("commit"));
    /// }
    /// ```
    pub fn get_prompt(&self, name: &str) -> Option<&String> {
        self.prompts.get(name)
    }

    /// Returns a reference to all loaded prompt templates.
    ///
    /// The returned map contains prompt names as keys and their corresponding content as values.
    pub fn prompts(&self) -> &HashMap<String, String> {
        &self.prompts
    }

    /// Reloads the configuration and prompts from disk, replacing the current values.
    ///
    /// Returns an error if loading the configuration or prompts fails.
    pub fn reload(&mut self) -> AppResult<()> {
        self.config = Self::load_config(&self.paths)?;
        self.prompts = Self::load_prompts(&self.paths)?;
        Ok(())
    }

    /// Validates the current configuration for required fields and value ranges.
    ///
    /// Checks that the AI API key and URL are set and non-empty, and that the AI temperature is within the allowed range (0.0 to 2.0). If DevOps configuration is present, ensures its token and base URL are set and non-empty. Returns an error if any validation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let manager = ConfigManager::new().unwrap();
    /// assert!(manager.validate().is_ok());
    /// ```
    pub fn validate(&self) -> AppResult<()> {
        // 验证 AI 配置
        if self.config.ai.api_key.is_none() || self.config.ai.api_key.as_ref().unwrap().is_empty() {
            return Err(AppError::config("AI API 密钥未设置"));
        }

        if self.config.ai.api_url.is_empty() {
            return Err(AppError::config("AI API URL 未设置"));
        }

        if !(0.0..=2.0).contains(&self.config.ai.temperature) {
            return Err(AppError::config("AI 温度值必须在 0.0 到 2.0 之间"));
        }

        // 验证 DevOps 配置（如果存在）
        if let Some(devops) = &self.config.devops {
            if devops.token.is_empty() {
                return Err(AppError::config("DevOps 令牌未设置"));
            }
            if devops.base_url.is_empty() {
                return Err(AppError::config("DevOps 基础 URL 未设置"));
            }
        }

        Ok(())
    }

    /// Returns a reference to the configuration file and directory paths used by the application.
    ///
    /// This includes paths for the user config directory, main config file, prompts directory, and rules directory.
    pub fn paths(&self) -> &ConfigPaths {
        &self.paths
    }
}

impl Default for ConfigManager {
    /// Creates a new `ConfigManager` instance with default settings, panicking if initialization fails.
    ///
    /// # Panics
    ///
    /// Panics if the configuration manager cannot be created due to file system or configuration errors.
    ///
    /// # Examples
    ///
    /// ```
    /// let manager = ConfigManager::default();
    /// assert!(manager.config().ai.api_url.contains("openai"));
    /// ```
    fn default() -> Self {
        Self::new().expect("无法创建默认配置管理器")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_paths() {
        let paths = ConfigPaths::new();
        assert!(paths.config_file.to_string_lossy().contains("gitai"));
        assert!(paths.prompts_dir.to_string_lossy().contains("prompts"));
    }

    #[test]
    fn test_default_configs() {
        let ai_config = AIConfig::default();
        assert_eq!(ai_config.temperature, 0.7);
        assert_eq!(ai_config.max_retries, 3);

        let git_config = GitConfig::default();
        assert_eq!(git_config.default_branch, "main");
        assert!(!git_config.auto_stage);

        let general_config = GeneralConfig::default();
        assert_eq!(general_config.log_level, "info");
        assert!(general_config.color_enabled);
    }

    #[test]
    fn test_gitai_config_serialization() {
        let config = GitAIConfig::default();
        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("[ai]"));
        assert!(serialized.contains("[git]"));
        assert!(serialized.contains("[translation]"));
        
        let deserialized: GitAIConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.ai.temperature, config.ai.temperature);
    }
}
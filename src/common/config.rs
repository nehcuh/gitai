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
    pub fn new() -> Self {
        let config_dir = expand_path("~/.config/gitai");
        Self {
            config_file: config_dir.join("config.toml"),
            prompts_dir: config_dir.join("prompts"),
            rules_dir: config_dir.join("rules"),
            user_config_dir: config_dir,
        }
    }

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
    /// 创建新的配置管理器
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

    /// 加载配置文件
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

    /// 应用环境变量覆盖
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

    /// 加载提示文件
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

    /// 保存配置到文件
    fn save_config_to_file(config: &GitAIConfig, path: &Path) -> AppResult<()> {
        let content = toml::to_string_pretty(config)
            .map_err(|e| AppError::config_with_source("配置序列化失败", e))?;
        write_file_safe(path, &content)?;
        Ok(())
    }

    /// 保存配置
    pub fn save_config(&self) -> AppResult<()> {
        Self::save_config_to_file(&self.config, &self.paths.config_file)
    }

    /// 获取配置
    pub fn config(&self) -> &GitAIConfig {
        &self.config
    }

    /// 获取可变配置引用
    pub fn config_mut(&mut self) -> &mut GitAIConfig {
        &mut self.config
    }

    /// 获取提示内容
    pub fn get_prompt(&self, name: &str) -> Option<&String> {
        self.prompts.get(name)
    }

    /// 获取所有提示
    pub fn prompts(&self) -> &HashMap<String, String> {
        &self.prompts
    }

    /// 重新加载配置
    pub fn reload(&mut self) -> AppResult<()> {
        self.config = Self::load_config(&self.paths)?;
        self.prompts = Self::load_prompts(&self.paths)?;
        Ok(())
    }

    /// 验证配置
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

    /// 获取配置路径
    pub fn paths(&self) -> &ConfigPaths {
        &self.paths
    }
}

impl Default for ConfigManager {
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
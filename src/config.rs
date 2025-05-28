use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, io::ErrorKind, path::PathBuf};

use crate::errors::ConfigError;

// Configuration location
const USER_CONFIG_PATH: &str = "~/.config/gitai";
const USER_PROMPT_PATH: &str = "~/.config/gitai/prompts";
const USER_RULES_PATH: &str = "~/.config/gitai/rules";

// Fully configuration files
const CONFIG_FILE_NAME: &str = "config.toml";
const HELPER_PROMPT: &str = "helper-prompt.md";
const TRANSLATOR_PROMPT: &str = "translator.md";
const COMMIT_GENERATOR_PROMPT: &str = "commit-generator.md";
const COMMIT_DIVIATION_PROMPT: &str = "commit-deviation.md"; // For future develop, calculate diviation of code and user develop tasks
const REVIEW_PROMPT: &str = "review.md";

// Templates files
const TEMPLATE_CONFIG_FILE: &str = "assets/config.example.toml";
const TEMPLATE_HELPER: &str = "assets/helper-prompt.md";
const TEMPLATE_TRANSLATOR: &str = "assets/translator.md";
const TEMPLATE_COMMIT_GENERATOR: &str = "assets/commit-generator.md";
const TEMPLATE_COMMIT_DEVIATION: &str = "assets/commit-deviation.md";
const TEMPLATE_REVIEW: &str = "assets/review.md";

// Total configuration files
const TOTAL_CONFIG_FILE_COUNT: u32 = 6;

/// Get the absolute path of a template file based on project root
fn abs_template_path(relative_path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

/// AI Configuration
#[allow(unused)]
#[derive(Deserialize, Debug, Clone, Default)]
pub struct AIConfig {
    pub api_url: String,
    pub model_name: String,
    pub temperature: f32,
    pub api_key: Option<String>,
    // optional: top_k, top_p
}

/// Account Configuration for DevOps platforms
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct AccountConfig {
    pub devops_platform: String,
    pub base_url: String,
    pub token: String,
    pub timeout: Option<u64>,
    pub retry_count: Option<u32>,
}

// Default implementation for AccountConfig
// Note: User must provide devops_platform, base_url, and token.
// Defaults are provided for timeout and retry_count.
impl Default for AccountConfig {
    fn default() -> Self {
        Self {
            devops_platform: String::new(), // Or some default platform if applicable
            base_url: String::new(),
            token: String::new(),
            timeout: Some(30000), // Default timeout 30 seconds
            retry_count: Some(3), // Default retry count 3
        }
    }
}

impl AccountConfig {
    // Note: The user story's example took `config: Option<AccountConfig>`.
    // We'll take `file_account_config: Option<Self>` to represent the config loaded from the file.
    // And `env_map: &HashMap<String, String>` to pass pre-fetched environment variables.
    pub fn from_env_or_file(
        file_account_config: Option<Self>, // Config directly from TOML
        env_map: &HashMap<String, String>, // Pre-fetched relevant environment variables
    ) -> Result<Option<Self>, ConfigError> {
        let devops_platform_env = env_map.get("GITAI_DEVOPS_PLATFORM");
        let base_url_env = env_map.get("GITAI_DEVOPS_BASE_URL");
        let token_env = env_map.get("GITAI_DEVOPS_TOKEN");

        // Extract values from file_account_config if it exists
        let file_platform = file_account_config
            .as_ref()
            .map(|c| c.devops_platform.clone());
        let file_base_url = file_account_config.as_ref().map(|c| c.base_url.clone());
        let file_token = file_account_config.as_ref().map(|c| c.token.clone());
        let file_timeout = file_account_config.as_ref().and_then(|c| c.timeout);
        let file_retry_count = file_account_config.as_ref().and_then(|c| c.retry_count);

        // Determine final values, giving priority to environment variables
        let devops_platform = devops_platform_env.map(|s| s.to_string()).or(file_platform);
        let base_url = base_url_env.map(|s| s.to_string()).or(file_base_url);
        let token = token_env.map(|s| s.to_string()).or(file_token);

        // Optional fields: only take from file config if no env override is present for mandatory fields.
        // The user story implies env vars only for platform, base_url, token.
        // So, timeout and retry_count will come from file_account_config if it exists.
        let timeout = file_timeout;
        let retry_count = file_retry_count;

        // Check if any of the core fields were specified at all (either env or file)
        if devops_platform.is_none() && base_url.is_none() && token.is_none() {
            // If no core fields are found from any source, it means no account config is intended.
            return Ok(None);
        }

        // Check if core fields have meaningful values (not None and not empty)
        let platform_is_meaningful = devops_platform.as_ref().map_or(false, |s| !s.is_empty());
        let url_is_meaningful = base_url.as_ref().map_or(false, |s| !s.is_empty());
        let token_is_meaningful = token.as_ref().map_or(false, |s| !s.is_empty());

        if !platform_is_meaningful && !url_is_meaningful && !token_is_meaningful {
            // If no core fields have non-empty values from any source,
            // it means no account config is intended, even if optional fields like timeout/retry exist.
            return Ok(None);
        }

        // If some core fields are present, then all three (platform, url, token) must be resolvable.
        let final_devops_platform = devops_platform.ok_or_else(|| {
            ConfigError::DevOpsConfigMissing(
                "account.devops_platform (and GITAI_DEVOPS_PLATFORM not set)".to_string(),
            )
        })?;
        let final_base_url = base_url.ok_or_else(|| {
            ConfigError::DevOpsConfigMissing(
                "account.base_url (and GITAI_DEVOPS_BASE_URL not set)".to_string(),
            )
        })?;
        let final_token = token.ok_or_else(|| {
            ConfigError::DevOpsConfigMissing(
                "account.token (and GITAI_DEVOPS_TOKEN not set)".to_string(),
            )
        })?;

        Ok(Some(Self {
            devops_platform: final_devops_platform,
            base_url: final_base_url,
            token: final_token,
            timeout,     // Will be None if not in file_account_config
            retry_count, // Will be None if not in file_account_config
        }))
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate platform support
        match self.devops_platform.to_lowercase().as_str() {
            "coding" | "jira" | "azure-devops" => { /* Platform is supported */ }
            _ => {
                return Err(ConfigError::UnsupportedPlatform(
                    self.devops_platform.clone(),
                ));
            }
        }

        // Validate URL format
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err(ConfigError::InvalidUrl(self.base_url.clone()));
        }

        // Validate token format (presence)
        if self.token.is_empty() {
            return Err(ConfigError::EmptyToken);
        }

        Ok(())
    }
}

/// Tree-sitter Configuration
#[allow(unused)]
#[derive(Deserialize, Debug, Clone)]
pub struct TreeSitterConfig {
    /// Represents if enable AST analysis
    #[serde(default)]
    pub enabled: bool,

    /// Analysis depth: "shallow", "medium", "deep"
    #[serde(default = "default_analysis_depth")]
    pub analysis_depth: String,

    /// Is cache enabled
    #[serde(default = "default_cache_enabled")]
    pub cache_enabled: bool,

    /// List of supported languages
    #[serde(default = "default_languages")]
    pub languages: Vec<String>,
}

impl Default for TreeSitterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            analysis_depth: default_analysis_depth(),
            cache_enabled: default_cache_enabled(),
            languages: default_languages(),
        }
    }
}

/// Configuration for review functionality
#[derive(Deserialize, Debug, Clone)]
pub struct ReviewConfig {
    /// Whether to automatically save review results to local files
    #[serde(default = "default_auto_save")]
    pub auto_save: bool,

    /// Base path for storing review results (supports ~ expansion)
    #[serde(default = "default_storage_path")]
    pub storage_path: String,

    /// Default format for saved review files
    #[serde(default = "default_review_format")]
    pub format: String,

    /// Maximum age in hours to keep review results
    #[serde(default = "default_max_age_hours")]
    pub max_age_hours: u32,

    /// Whether to include review results in commit message generation
    #[serde(default = "default_include_in_commit")]
    pub include_in_commit: bool,
}

impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            auto_save: default_auto_save(),
            storage_path: default_storage_path(),
            format: default_review_format(),
            max_age_hours: default_max_age_hours(),
            include_in_commit: default_include_in_commit(),
        }
    }
}

fn default_analysis_depth() -> String {
    "medium".to_string()
}

fn default_cache_enabled() -> bool {
    true
}

fn default_languages() -> Vec<String> {
    vec![
        "rust".to_string(),
        "c".to_string(),
        "cpp".to_string(),
        "go".to_string(),
        "javascript".to_string(),
        "python".to_string(),
        "java".to_string(),
    ]
}

fn default_auto_save() -> bool {
    true
}

fn default_storage_path() -> String {
    "~/.gitai/review_results".to_string()
}

fn default_review_format() -> String {
    "markdown".to_string()
}

fn default_max_age_hours() -> u32 {
    168 // 7 days
}

fn default_include_in_commit() -> bool {
    true
}

/// Partial loading helper struct for AI configuration
#[derive(Deserialize, Debug, Default, Clone)]
pub struct PartialAIConfig {
    #[serde(default)]
    api_url: Option<String>,
    #[serde(default)]
    model_name: Option<String>,
    #[serde(default)]
    temperature: Option<f32>,
    #[serde(default)]
    api_key: Option<String>,
}

/// Partial loading helper struct for Account configuration
#[derive(Debug, Deserialize, Default, Clone)]
pub struct PartialAccountConfig {
    pub devops_platform: Option<String>,
    pub base_url: Option<String>,
    pub token: Option<String>,
    pub timeout: Option<u64>,
    pub retry_count: Option<u32>,
}

/// Partial loading helper structure for Tree-sitter configuration
#[derive(Deserialize, Debug, Default, Clone)]
pub struct PartialTreeSitterConfig {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    analysis_depth: Option<String>,
    #[serde(default)]
    cache_enabled: Option<bool>,
    #[serde(default)]
    languages: Option<Vec<String>>,
}

/// Partial loading helper structure for review configuration
#[derive(Deserialize, Debug, Default, Clone)]
pub struct PartialReviewConfig {
    #[serde(default)]
    pub auto_save: Option<bool>,
    #[serde(default)]
    pub storage_path: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub max_age_hours: Option<u32>,
    #[serde(default)]
    pub include_in_commit: Option<bool>,
}

/// Application overall configuration
#[allow(unused)]
#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    #[serde(default)]
    pub ai: AIConfig,

    #[serde(default)]
    pub tree_sitter: TreeSitterConfig,

    #[serde(default)]
    pub review: ReviewConfig,

    #[serde(default)]
    pub account: Option<AccountConfig>,

    #[serde(skip)]
    pub prompts: HashMap<String, String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct PartialAppConfig {
    ai: Option<PartialAIConfig>,
    tree_sitter: Option<PartialTreeSitterConfig>,
    review: Option<PartialReviewConfig>,
    account: Option<PartialAccountConfig>,
}

impl AppConfig {
    pub fn initialize_config() -> Result<(PathBuf, HashMap<String, PathBuf>), ConfigError> {
        let user_config_path = Self::extract_file_path(USER_CONFIG_PATH, CONFIG_FILE_NAME)?;

        // Commit generator prompt
        let commit_generator_prompt_path =
            Self::extract_file_path(USER_PROMPT_PATH, COMMIT_GENERATOR_PROMPT)?;
        // Commit deviation prompt
        let commit_deviation_prompt_path =
            Self::extract_file_path(USER_PROMPT_PATH, COMMIT_DIVIATION_PROMPT)?;

        // General AI support
        let general_helper_prompt_path = Self::extract_file_path(USER_PROMPT_PATH, HELPER_PROMPT)?;

        // Translator AI support
        let translator_prompt_path = Self::extract_file_path(USER_PROMPT_PATH, TRANSLATOR_PROMPT)?;

        // Review AI support
        let review_prompt_path = Self::extract_file_path(USER_PROMPT_PATH, REVIEW_PROMPT)?;

        let mut user_prompt_paths = HashMap::new();
        user_prompt_paths.insert(
            "commit-generator".to_string(),
            commit_generator_prompt_path.clone(),
        );
        user_prompt_paths.insert(
            "commit-deviation".to_string(),
            commit_deviation_prompt_path.clone(),
        );
        user_prompt_paths.insert(
            "general-helper".to_string(),
            general_helper_prompt_path.clone(),
        );
        user_prompt_paths.insert("translator".to_string(), translator_prompt_path.clone());
        user_prompt_paths.insert("review".to_string(), review_prompt_path.clone());

        let mut existing_files = Vec::new();
        let mut existing_count = 0;

        if user_config_path.exists() {
            existing_count += 1;
            existing_files.push(format!("用户配置已存在于 {:?}", user_config_path));
        }

        if commit_generator_prompt_path.exists() {
            existing_count += 1;
            existing_files.push(format!(
                "用户 commit-generator.md 已存在于 {:?}",
                commit_generator_prompt_path
            ));
        }

        if commit_deviation_prompt_path.exists() {
            existing_count += 1;
            existing_files.push(format!(
                "用户 commit-deviation.md 已存在于 {:?}",
                commit_deviation_prompt_path
            ));
        }

        if general_helper_prompt_path.exists() {
            existing_count += 1;
            existing_files.push(format!(
                "用户 genral-helper.md 已存在于 {:?}",
                general_helper_prompt_path
            ));
        }

        if translator_prompt_path.exists() {
            existing_count += 1;
            existing_files.push(format!(
                "用户 translator.md 已存在于 {:?}",
                translator_prompt_path
            ));
        }

        if review_prompt_path.exists() {
            existing_count += 1;
            existing_files.push(format!("用户 review.md 已存在于 {:?}", review_prompt_path));
        }

        if existing_count > 0 {
            if existing_count == TOTAL_CONFIG_FILE_COUNT {
                tracing::info!("所有 {} 个配置文件已存在，将直接使用", existing_count);
            } else if existing_count < TOTAL_CONFIG_FILE_COUNT {
                tracing::info!(
                    "发现 {}/{} 个配置文件已存在，将补充缺失的配置",
                    existing_count,
                    TOTAL_CONFIG_FILE_COUNT
                );
            } else {
                return Err(ConfigError::Other(format!(
                    "发现 {}/{} 个配置文件，超过全局配置文件需求",
                    existing_count, TOTAL_CONFIG_FILE_COUNT,
                )));
            }

            if !existing_files.is_empty() {
                tracing::debug!("{}", existing_files.join("\n"));
            }
        } else {
            tracing::info!("未发现任何配置文件，将创建并使用默认配置文件")
        }

        let user_config_dir = match user_config_path.parent() {
            Some(dir) => dir.to_path_buf(),
            None => {
                return Err(ConfigError::FileWrite(
                    user_config_path.to_string_lossy().to_string(),
                    std::io::Error::new(ErrorKind::Other, "Invalid config path"),
                ));
            }
        };

        let user_prompt_dir = match commit_generator_prompt_path.parent() {
            Some(dir) => dir.to_path_buf(),
            None => {
                return Err(ConfigError::FileWrite(
                    commit_generator_prompt_path.to_string_lossy().to_string(),
                    std::io::Error::new(ErrorKind::Other, "Invalid config path"),
                ));
            }
        };

        tracing::debug!("准备创建默认配置目录: {:?}", user_config_dir);
        std::fs::create_dir_all(&user_config_dir).map_err(|e| {
            ConfigError::FileWrite(user_config_dir.to_string_lossy().to_string(), e)
        })?;

        tracing::debug!("准备创建默认 prompts 目录: {:?}", user_prompt_dir);
        std::fs::create_dir_all(&user_prompt_dir).map_err(|e| {
            ConfigError::FileWrite(user_prompt_dir.to_string_lossy().to_string(), e)
        })?;

        // Initialize missing config files
        if !user_config_path.exists() {
            tracing::info!("配置文件 {} 不存在，正在初始化", CONFIG_FILE_NAME);
            Self::initialize_config_file(
                &user_config_path,
                TEMPLATE_CONFIG_FILE,
                "GITAI_ASSETS_CONFIG",
                "Config",
            )?;
        }

        if !commit_generator_prompt_path.exists() {
            tracing::info!("{} 不存在，正在初始化", COMMIT_GENERATOR_PROMPT);
            Self::initialize_config_file(
                &commit_generator_prompt_path,
                TEMPLATE_COMMIT_GENERATOR,
                "GITAI_COMMIT_GENERATOR_PROMPT",
                "Commit generator prompt",
            )?;
        }

        if !commit_deviation_prompt_path.exists() {
            tracing::info!("{} 不存在，正在初始化", COMMIT_DIVIATION_PROMPT);
            Self::initialize_config_file(
                &commit_deviation_prompt_path,
                TEMPLATE_COMMIT_DEVIATION,
                "GITAI_COMMIT_DEVIATION_PROMPT",
                "Commit deviation prompt",
            )?;
        }

        if !general_helper_prompt_path.exists() {
            tracing::info!("{} 不存在，正在初始化", HELPER_PROMPT);
            Self::initialize_config_file(
                &general_helper_prompt_path,
                TEMPLATE_HELPER,
                "GITAI_GENERAL_HELP_PROMPT",
                "Git general prompt",
            )?;
        }

        if !translator_prompt_path.exists() {
            tracing::info!("{} 不存在，正在初始化", TRANSLATOR_PROMPT);
            Self::initialize_config_file(
                &translator_prompt_path,
                TEMPLATE_TRANSLATOR,
                "GITAI_TRANSLATOR_PROMPT",
                "Git translator prompt",
            )?;
        }

        if !review_prompt_path.exists() {
            tracing::info!("{} 不存在，正在初始化", REVIEW_PROMPT);
            Self::initialize_config_file(
                &review_prompt_path,
                TEMPLATE_REVIEW,
                "GITAI_REVIEW_PROMPT",
                "Git review prompt",
            )?;
        }

        Ok((user_config_path, user_prompt_paths))
    }

    /// Copy a template configuration file to a user destination path if it doesn't exist
    pub fn initialize_config_file(
        user_file_path: &PathBuf,
        template_file_name: &str,
        env_var_name: &str,
        file_description: &str,
    ) -> Result<(), ConfigError> {
        if user_file_path.exists() {
            tracing::debug!("配置文件已存在，跳过复制: {:?}", user_file_path);
            return Ok(());
        }

        let template_path = if let Ok(path) = std::env::var(env_var_name) {
            PathBuf::from(path)
        } else {
            abs_template_path(template_file_name)
        };

        if !template_path.exists() && !user_file_path.exists() {
            return Err(ConfigError::FileRead(
                format!(
                    "{} template not found at {}",
                    file_description,
                    template_path.display()
                ),
                std::io::Error::new(
                    ErrorKind::NotFound,
                    format!("{} template file not found", file_description),
                ),
            ));
        }

        tracing::debug!("复制配置模板 {:?} 到 {:?}", template_path, user_file_path);

        std::fs::copy(&template_path, user_file_path).map_err(|e| {
            ConfigError::FileWrite(
                format!(
                    "Failed to copy source prompt file {} to target prompt file {}",
                    template_path.display(),
                    user_file_path.display()
                ),
                e,
            )
        })?;

        tracing::info!("已成功初始化配置文件: {:?}", user_file_path);

        Ok(())
    }

    pub fn extract_file_path(dir_name: &str, filename: &str) -> Result<PathBuf, ConfigError> {
        if dir_name.starts_with("~") {
            if let Some(stripped_path) = dir_name.strip_prefix("~/") {
                let home_str = env::var("HOME")
                    .ok()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| {
                        dirs::home_dir()
                            .expect("Could not determine home directory")
                            .to_string_lossy()
                            .to_string()
                    });
                return Ok(PathBuf::from(home_str).join(stripped_path).join(filename));
            }
        }
        Ok(PathBuf::from(dir_name).join(filename))
    }

    pub fn load() -> Result<Self, ConfigError> {
        let start_time = std::time::Instant::now();

        tracing::info!("加载 AST Query 配置");
        Self::initialize_rules()?;

        let (user_config_path, user_prompt_paths) = match Self::initialize_config() {
            Ok(result) => {
                tracing::debug!("配置初始化完成，用时 {:?}", start_time.elapsed());
                result
            }
            Err(e) => {
                tracing::error!("配置初始化失败: {}", e);
                return Err(e);
            }
        };

        tracing::info!("正在从用户目录加载配置: {:?}", user_config_path);

        tracing::debug!("将加载以下提示文件:");
        for (prompt_type, path) in &user_prompt_paths {
            tracing::debug!("  - {} 提示文件: {:?}", prompt_type, path);
        }

        Self::load_config_from_file(&user_config_path, &user_prompt_paths)
    }

    /// copy default Tree-sitter rule files to user's config directory if missing
    fn initialize_rules() -> Result<(), ConfigError> {
        // resolve user rules base
        let base = Self::extract_file_path(USER_RULES_PATH, "")?;
        std::fs::create_dir_all(&base)
            .map_err(|e| ConfigError::FileWrite(base.to_string_lossy().into(), e))?;
        // default queries directory in project
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_string());
        let project_queries = PathBuf::from(manifest_dir).join("queries");

        // Skip if queries directory doesn't exist (e.g., in tests)
        if !project_queries.exists() {
            tracing::debug!(
                "Queries directory {:?} does not exist, skipping rules initialization",
                project_queries
            );
            return Ok(());
        }

        let scm_files = ["highlights.scm", "injections.scm", "locals.scm"];
        for entry in std::fs::read_dir(&project_queries)
            .map_err(|e| ConfigError::FileRead("read project queries".into(), e))?
        {
            let dir = entry
                .map_err(|e| ConfigError::FileRead("iter project queries".into(), e))?
                .path();
            if dir.is_dir() {
                let lang = dir.file_name().unwrap();
                let dest = base.join(lang);
                std::fs::create_dir_all(&dest)
                    .map_err(|e| ConfigError::FileWrite(dest.to_string_lossy().into(), e))?;
                for file in &scm_files {
                    let src = dir.join(file);
                    let dst = dest.join(file);
                    if !dst.exists() {
                        std::fs::copy(&src, &dst)
                            .map_err(|e| ConfigError::FileWrite(dst.to_string_lossy().into(), e))?;
                    }
                }
            }
        }
        Ok(())
    }

    fn load_config_from_file(
        config_path: &std::path::Path,
        prompt_paths: &HashMap<String, PathBuf>,
    ) -> Result<Self, ConfigError> {
        tracing::info!("正在读取配置文件: {:?}", config_path);

        let start_time = std::time::Instant::now();
        let config_content = match std::fs::read_to_string(config_path) {
            Ok(content) => {
                tracing::debug!("配置文件读取成功，大小: {} 字节", content.len());
                content
            }
            Err(e) => {
                tracing::error!("读取配置文件失败 {:?}: {}", config_path, e);
                return Err(ConfigError::FileRead(
                    config_path.to_string_lossy().to_string(),
                    e,
                ));
            }
        };

        tracing::debug!("正在解析配置文件 TOML 格式...");
        let mut partial_config: PartialAppConfig = match toml::from_str(&config_content) {
            Ok(config) => {
                tracing::debug!("TOML 解析成功, 用时 {:?}", start_time.elapsed());
                config
            }
            Err(e) => {
                tracing::error!("解析配置文件失败 {:?}: {}", config_path, e);
                return Err(ConfigError::TomlParse(
                    config_path.to_string_lossy().to_string(),
                    e,
                ));
            }
        };

        if let Some(ai) = &mut partial_config.ai {
            if let Some(api_key) = &ai.api_key {
                if api_key == "YOUR_API_KEY_IF_NEEDED" || api_key.is_empty() {
                    ai.api_key = None;
                    tracing::info!("发现 API 密钥占位符或空字符串。视为无 API 密钥。");
                }
            }
        }

        if partial_config.ai.is_none() {
            tracing::info!("配置文件中未找到 AI 配置部分，使用默认值");
            partial_config.ai = Some(PartialAIConfig::default());
        }

        if partial_config.tree_sitter.is_none() {
            tracing::info!("配置文件中未找到 Tree-sitter 配置部分，使用默认值");
            partial_config.tree_sitter = Some(PartialTreeSitterConfig::default());
        }

        if partial_config.review.is_none() {
            tracing::info!("配置文件中未找到 Review 配置部分，使用默认值");
            partial_config.review = Some(PartialReviewConfig::default());
        }

        let mut prompts = HashMap::new();
        let prompt_start_time = std::time::Instant::now();

        for (prompt_type, prompt_path) in prompt_paths {
            if !prompt_path.exists() {
                tracing::warn!("提示文件不存在: {:?}，跳过此文件", prompt_path);
                continue;
            }

            tracing::debug!("正在读取提示文件: {:?}", prompt_path);
            match std::fs::read_to_string(prompt_path) {
                Ok(content) => {
                    if content.trim().is_empty() {
                        tracing::warn!("提示文件 {:?} 内容为空，跳过", prompt_path);
                        continue;
                    }
                    tracing::debug!(
                        "提示文件 {:?} 读取成功，大小: {} 字节",
                        prompt_path,
                        content.len()
                    );
                    prompts.insert(prompt_type.clone(), content);
                }
                Err(e) => {
                    tracing::warn!("读取提示文件 {:?} 失败: {}, 跳过此文件", prompt_path, e);
                }
            }
        }

        tracing::debug!(
            "读取全部提示文件完成，用时 {:?}",
            prompt_start_time.elapsed()
        );

        let partial_ai_config = partial_config.ai.unwrap_or_default();

        let default_api_url = "http://localhost:11434/v1/chat/completions".to_string();
        let default_model = "qwen3:32b-q8_0".to_string();
        let default_temperature = 0.7;

        let api_url = partial_ai_config.api_url.unwrap_or_else(|| {
            tracing::debug!("未指定 API URL，使用默认值: {}", default_api_url);
            default_api_url
        });

        let model_name = partial_ai_config.model_name.unwrap_or_else(|| {
            tracing::debug!("未指定模型名称，使用默认值: {}", default_model);
            default_model
        });

        let temperature = partial_ai_config.temperature.unwrap_or_else(|| {
            tracing::debug!("未指定温度参数，使用默认值: {}", default_temperature);
            default_temperature
        });

        let ai_config = AIConfig {
            api_url: api_url.clone(),
            model_name: model_name.clone(),
            temperature,
            api_key: partial_ai_config.api_key.clone(),
        };

        tracing::info!(
            "AI 配置信息: API URL: {}, 模型: {}, 温度: {:?}, API密钥: {}",
            api_url,
            model_name,
            temperature,
            if partial_ai_config.api_key.is_some() {
                "已设置"
            } else {
                "未设置"
            }
        );

        let partial_tree_sitter_config = partial_config.tree_sitter.unwrap_or_default();

        let enabled = partial_tree_sitter_config.enabled.unwrap_or(true);
        let analysis_depth = partial_tree_sitter_config
            .analysis_depth
            .unwrap_or_else(default_analysis_depth);
        let cache_enabled = partial_tree_sitter_config.cache_enabled.unwrap_or(true);
        let languages = partial_tree_sitter_config
            .languages
            .unwrap_or_else(default_languages);

        let tree_sitter_config = TreeSitterConfig {
            enabled,
            analysis_depth: analysis_depth.clone(),
            cache_enabled,
            languages: languages.clone(),
        };

        tracing::debug!(
            "Tree-sitter 配置: 启用状态: {}, 分析深度: {}, 缓存启用: {}, 支持语言数量: {}",
            enabled,
            analysis_depth,
            cache_enabled,
            languages.len()
        );

        if enabled {
            tracing::debug!("Tree-sitter 支持的语言: {}", languages.join(", "));
        }

        // --- Review Configuration Loading ---
        let partial_review_config = partial_config.review.unwrap_or_default();

        let auto_save = partial_review_config
            .auto_save
            .unwrap_or_else(default_auto_save);
        let storage_path = partial_review_config
            .storage_path
            .unwrap_or_else(default_storage_path);
        let format = partial_review_config
            .format
            .unwrap_or_else(default_review_format);
        let max_age_hours = partial_review_config
            .max_age_hours
            .unwrap_or_else(default_max_age_hours);
        let include_in_commit = partial_review_config
            .include_in_commit
            .unwrap_or_else(default_include_in_commit);

        let review_config = ReviewConfig {
            auto_save,
            storage_path: storage_path.clone(),
            format: format.clone(),
            max_age_hours,
            include_in_commit,
        };

        tracing::debug!(
            "Review 配置: 自动保存: {}, 存储路径: {}, 格式: {}, 最大保留时间: {}小时, 包含在提交中: {}",
            auto_save,
            storage_path,
            format,
            max_age_hours,
            include_in_commit
        );
        // --- End Review Configuration Loading ---

        // --- Account Configuration Loading ---
        // Helper function to check if a value is a placeholder
        let is_placeholder = |value: &Option<String>| -> bool {
            match value {
                Some(s) => {
                    let s = s.trim();
                    s.is_empty() || 
                    s == "YOUR_DEVOPS_API_TOKEN" || 
                    s == "YOUR_API_KEY_IF_NEEDED" ||
                    s.starts_with("YOUR_") ||
                    s == "your-devops-instance.com" ||
                    s.contains("your-devops-instance")
                }
                None => true,
            }
        };

        let file_loaded_account_config: Option<AccountConfig> =
            partial_config.account.and_then(|p_acc| {
                // Check if any of the required fields have meaningful values
                if is_placeholder(&p_acc.devops_platform) && 
                   is_placeholder(&p_acc.base_url) && 
                   is_placeholder(&p_acc.token) {
                    tracing::debug!("账户配置中只包含占位符值，将忽略文件中的账户配置");
                    None
                } else {
                    // Only create AccountConfig if we have at least some meaningful values
                    Some(AccountConfig {
                        devops_platform: p_acc.devops_platform.unwrap_or_default(),
                        base_url: p_acc.base_url.unwrap_or_default(),
                        token: p_acc.token.unwrap_or_default(),
                        timeout: p_acc.timeout.or(AccountConfig::default().timeout),
                        retry_count: p_acc.retry_count.or(AccountConfig::default().retry_count),
                    })
                }
            });

        let mut env_map = HashMap::new();
        if let Ok(val) = env::var("GITAI_DEVOPS_PLATFORM") {
            env_map.insert("GITAI_DEVOPS_PLATFORM".to_string(), val);
        }
        if let Ok(val) = env::var("GITAI_DEVOPS_BASE_URL") {
            env_map.insert("GITAI_DEVOPS_BASE_URL".to_string(), val);
        }
        if let Ok(val) = env::var("GITAI_DEVOPS_TOKEN") {
            env_map.insert("GITAI_DEVOPS_TOKEN".to_string(), val);
        }

        let final_account_config =
            match AccountConfig::from_env_or_file(file_loaded_account_config, &env_map)? {
                Some(acc_conf) => {
                    acc_conf.validate()?;
                    Some(acc_conf)
                }
                None => None,
            };

        if final_account_config.is_some() {
            tracing::info!("Account configuration loaded successfully.");
        } else {
            tracing::info!("No account configuration found or loaded.");
        }
        // --- End Account Configuration Loading ---

        if prompts.is_empty() {
            tracing::warn!("未能加载任何提示文件，配置可能不完整");
        } else if prompts.len() < prompt_paths.len() {
            tracing::warn!(
                "只加载了部分提示文件 ({}/{})",
                prompts.len(),
                prompt_paths.len()
            );
            tracing::debug!(
                "已加载的提示文件类型: {}",
                prompts
                    .keys()
                    .map(|k| k.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        } else {
            tracing::info!("成功加载全部 {} 个提示文件", prompts.len());
        }

        let config = Self {
            ai: ai_config,
            tree_sitter: tree_sitter_config,
            review: review_config,
            account: final_account_config,
            prompts,
        };

        tracing::info!("配置加载完成，Gitai 准备就绪");
        Ok(config)
    }
}

// Helper to create HashMap for environment variables
fn make_env_map(vars: &[(&str, &str)]) -> HashMap<String, String> {
    vars.iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // Helper to create a temporary directory and set it as HOME
    fn setup_test_environment() -> Result<(TempDir, PathBuf, PathBuf), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let home_path = temp_dir.path().to_path_buf();

        // Mock home directory by setting HOME env var
        unsafe { std::env::set_var("HOME", home_path.to_str().unwrap()) };

        // Construct paths based on the mocked home_path
        // USER_CONFIG_PATH is "~/.config/gitai"
        // USER_PROMPT_PATH is "~/.config/gitai/prompts"
        let user_config_dir_name = USER_CONFIG_PATH
            .strip_prefix("~/")
            .unwrap_or(USER_CONFIG_PATH);
        let user_prompts_dir_name = USER_PROMPT_PATH
            .strip_prefix("~/")
            .unwrap_or(USER_PROMPT_PATH);

        let user_config_base_dir = home_path.join(user_config_dir_name);
        let user_prompts_dir = home_path.join(user_prompts_dir_name);

        fs::create_dir_all(&user_config_base_dir)?; // This creates .config/gitai under mocked home
        fs::create_dir_all(&user_prompts_dir)?; // This creates .config/gitai/prompts under mocked home

        // Create a dummy test_assets directory for template loading
        let test_assets_dir = temp_dir.path().join("test_assets");
        fs::create_dir_all(&test_assets_dir)?;

        let templates = [
            (
                TEMPLATE_CONFIG_FILE,
                "api_url = \"http://localhost:11434/v1/chat/completions\"\nmodel_name = \"test-model\"",
            ),
            (TEMPLATE_COMMIT_GENERATOR, "Generate a commit message."),
            (TEMPLATE_COMMIT_DEVIATION, "Explain commit deviation."),
            (TEMPLATE_HELPER, "General AI help prompt."),
            (TEMPLATE_REVIEW, "Review prompt."),
            (TEMPLATE_TRANSLATOR, "Translator AI help prompt."),
        ];

        // Write template files to test_assets_dir
        for (template_name, content) in &templates {
            let file_path = test_assets_dir.join(template_name);
            if let Some(parent_dir) = file_path.parent() {
                // Ensure the parent directory exists.
                // fs::create_dir_all is idempotent, so it's safe to call even if the directory already exists.
                fs::create_dir_all(parent_dir)?;
            }
            let mut file = File::create(&file_path)?;
            writeln!(file, "{}", content)?;
        }

        unsafe {
            env::set_var(
                "GITAI_ASSETS_CONFIG",
                test_assets_dir.join(TEMPLATE_CONFIG_FILE).to_str().unwrap(),
            )
        };
        unsafe {
            env::set_var(
                "GITAI_COMMIT_GENERATOR_PROMPT",
                test_assets_dir
                    .join(TEMPLATE_COMMIT_GENERATOR)
                    .to_str()
                    .unwrap(),
            )
        };
        unsafe {
            env::set_var(
                "GITAI_COMMIT_DEVIATION_PROMPT",
                test_assets_dir
                    .join(TEMPLATE_COMMIT_DEVIATION)
                    .to_str()
                    .unwrap(),
            )
        };
        unsafe {
            env::set_var(
                "GITAI_GENERAL_HELP_PROMPT",
                test_assets_dir.join(TEMPLATE_HELPER).to_str().unwrap(),
            )
        };
        unsafe {
            env::set_var(
                "GITAI_TRANSLATOR_PROMPT",
                test_assets_dir.join(TEMPLATE_TRANSLATOR).to_str().unwrap(),
            )
        };
        unsafe {
            env::set_var(
                "GITAI_REVIEW_PROMPT",
                test_assets_dir.join(TEMPLATE_REVIEW).to_str().unwrap(),
            )
        };

        let fake_target_tmp_dir = temp_dir.path().join("target").join("tmp");
        fs::create_dir_all(&fake_target_tmp_dir)?;
        env::set_current_dir(&fake_target_tmp_dir)?;

        // Return the actual paths where config files will be written by the functions under test
        Ok((temp_dir, user_config_base_dir, user_prompts_dir))
    }

    #[test]
    fn test_account_config_from_env_or_file_all_from_file() {
        let file_config = Some(AccountConfig {
            devops_platform: "coding".to_string(),
            base_url: "https://file.example.com".to_string(),
            token: "file_token".to_string(),
            timeout: Some(10000),
            retry_count: Some(1),
        });
        let env_map = HashMap::new();
        let result = AccountConfig::from_env_or_file(file_config.clone(), &env_map).unwrap();
        assert_eq!(result, file_config);
    }

    #[test]
    fn test_account_config_from_env_or_file_all_from_env() {
        let env_map = make_env_map(&[
            ("GITAI_DEVOPS_PLATFORM", "jira"),
            ("GITAI_DEVOPS_BASE_URL", "https://env.example.com"),
            ("GITAI_DEVOPS_TOKEN", "env_token"),
        ]);
        let result = AccountConfig::from_env_or_file(None, &env_map).unwrap();
        assert!(result.is_some());
        let config = result.unwrap();
        assert_eq!(config.devops_platform, "jira");
        assert_eq!(config.base_url, "https://env.example.com");
        assert_eq!(config.token, "env_token");
        assert_eq!(config.timeout, None); // No file config, so None
        assert_eq!(config.retry_count, None); // No file config, so None
    }

    #[test]
    fn test_account_config_from_env_or_file_mixed_env_overrides() {
        let file_config = Some(AccountConfig {
            devops_platform: "coding".to_string(),
            base_url: "https://file.example.com".to_string(),
            token: "file_token".to_string(),
            timeout: Some(10000),
            retry_count: Some(1),
        });
        let env_map = make_env_map(&[
            ("GITAI_DEVOPS_PLATFORM", "jira"),
            ("GITAI_DEVOPS_BASE_URL", "https://env.example.com"),
            ("GITAI_DEVOPS_TOKEN", "env_token"),
        ]);
        let result = AccountConfig::from_env_or_file(file_config, &env_map).unwrap();
        assert!(result.is_some());
        let config = result.unwrap();
        assert_eq!(config.devops_platform, "jira");
        assert_eq!(config.base_url, "https://env.example.com");
        assert_eq!(config.token, "env_token");
        assert_eq!(config.timeout, Some(10000)); // From file
        assert_eq!(config.retry_count, Some(1)); // From file
    }

    #[test]
    fn test_account_config_from_env_or_file_partial_file_completed_by_env() {
        let file_config = Some(AccountConfig {
            devops_platform: "coding".to_string(),
            base_url: "".to_string(), //  empty, to be overridden or completed
            token: "".to_string(),    // empty
            timeout: Some(5000),
            retry_count: None,
        });
        let env_map = make_env_map(&[
            ("GITAI_DEVOPS_BASE_URL", "https://env.example.com"),
            ("GITAI_DEVOPS_TOKEN", "env_token_for_partial_file"),
        ]);
        let result = AccountConfig::from_env_or_file(file_config, &env_map).unwrap();
        assert!(result.is_some());
        let config = result.unwrap();
        assert_eq!(config.devops_platform, "coding"); // From file
        assert_eq!(config.base_url, "https://env.example.com"); // From env
        assert_eq!(config.token, "env_token_for_partial_file"); // From env
        assert_eq!(config.timeout, Some(5000));
        assert_eq!(config.retry_count, None);
    }

    #[test]
    fn test_account_config_from_env_or_file_partial_env_completed_by_file() {
        let file_config = Some(AccountConfig {
            devops_platform: "".to_string(), // empty, should be completed by file if env is not set
            base_url: "https://file.example.com".to_string(),
            token: "file_token_for_partial_env".to_string(),
            timeout: None,
            retry_count: Some(2),
        });
        let env_map = make_env_map(&[("GITAI_DEVOPS_PLATFORM", "azure-devops")]);
        let result = AccountConfig::from_env_or_file(file_config, &env_map).unwrap();
        assert!(result.is_some());
        let config = result.unwrap();
        assert_eq!(config.devops_platform, "azure-devops"); // From env
        assert_eq!(config.base_url, "https://file.example.com"); // From file
        assert_eq!(config.token, "file_token_for_partial_env"); // From file
        assert_eq!(config.timeout, None);
        assert_eq!(config.retry_count, Some(2));
    }

    #[test]
    fn test_account_config_from_env_or_file_all_missing() {
        let result = AccountConfig::from_env_or_file(None, &HashMap::new()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_account_config_from_env_or_file_only_optional_fields_in_file() {
        let file_config = Some(AccountConfig {
            devops_platform: "".to_string(),
            base_url: "".to_string(),
            token: "".to_string(),
            timeout: Some(10000),
            retry_count: Some(5),
        });
        let env_map = HashMap::new();
        let result = AccountConfig::from_env_or_file(file_config, &env_map).unwrap();
        // Since platform, base_url, and token are empty, it should be treated as no config provided.
        assert!(result.is_none());
    }

    #[test]
    fn test_initialize_config_creates_files_when_missing() -> Result<(), Box<dyn std::error::Error>>
    {
        let (_temp_dir_guard, user_config_base_dir, user_prompts_dir) = setup_test_environment()?;

        let expected_config_file_path = user_config_base_dir.join(CONFIG_FILE_NAME);
        let expected_commit_gen_prompt_path = user_prompts_dir.join(COMMIT_GENERATOR_PROMPT);
        let expected_commit_dev_prompt_path = user_prompts_dir.join(COMMIT_DIVIATION_PROMPT);
        let expected_helper_prompt_path = user_prompts_dir.join(HELPER_PROMPT);
        let expected_translator_prompt_path = user_prompts_dir.join(TRANSLATOR_PROMPT);
        let expected_review_prompt_path = user_prompts_dir.join(REVIEW_PROMPT);
        assert!(
            !expected_config_file_path.exists(),
            "Config file should not exist before init"
        );
        assert!(
            !expected_commit_gen_prompt_path.exists(),
            "Commit gen prompt should not exist before init"
        );
        assert!(
            !expected_commit_dev_prompt_path.exists(),
            "Commit dev prompt should not exist before init"
        );
        assert!(
            !expected_helper_prompt_path.exists(),
            "Helper prompt should not exist before init"
        );
        assert!(
            !expected_translator_prompt_path.exists(),
            "Translator prompt should not exist before init"
        );
        assert!(
            !expected_review_prompt_path.exists(),
            "Review prompt should not exist before init"
        );

        let (config_path, prompt_paths) = AppConfig::initialize_config()?;

        assert_eq!(
            config_path, expected_config_file_path,
            "Returned config path mismatch"
        );
        assert_eq!(
            prompt_paths.get("commit-generator").unwrap(),
            &expected_commit_gen_prompt_path,
            "Commit gen path mismatch"
        );
        assert_eq!(
            prompt_paths.get("commit-deviation").unwrap(),
            &expected_commit_dev_prompt_path,
            "Commit dev path mismatch"
        );
        assert_eq!(
            prompt_paths.get("general-helper").unwrap(),
            &expected_helper_prompt_path,
            "Helper path mismatch"
        );

        assert_eq!(
            prompt_paths.get("translator").unwrap(),
            &expected_translator_prompt_path,
            "Translator path mismatch"
        );
        assert_eq!(
            prompt_paths.get("review").unwrap(),
            &expected_review_prompt_path,
            "Review path mismatch"
        );

        assert!(
            expected_config_file_path.exists(),
            "Config file was not created"
        );
        assert!(
            expected_commit_gen_prompt_path.exists(),
            "Commit gen prompt was not created"
        );
        assert!(
            expected_commit_dev_prompt_path.exists(),
            "Commit dev prompt was not created"
        );
        assert!(
            expected_helper_prompt_path.exists(),
            "Helper prompt was not created"
        );

        let config_content = fs::read_to_string(expected_config_file_path)?;
        assert!(
            config_content.contains("test-model"),
            "Config content mismatch"
        );

        Ok(())
    }

    #[test]
    fn test_initialize_config_uses_existing_files() -> Result<(), Box<dyn std::error::Error>> {
        let (_temp_dir_guard, user_config_base_dir, user_prompts_dir) = setup_test_environment()?;

        let expected_config_file_path = user_config_base_dir.join(CONFIG_FILE_NAME);

        let unique_content = "# My Custom Config";
        fs::write(&expected_config_file_path, unique_content)?;

        // This one will be created by initialize_config
        let (config_path, prompt_paths) = AppConfig::initialize_config()?;

        assert_eq!(config_path, expected_config_file_path);

        let config_content_after_init = fs::read_to_string(expected_config_file_path)?;
        assert_eq!(
            config_content_after_init.trim(),
            unique_content.trim(),
            "Existing config file was overwritten"
        );

        assert!(
            prompt_paths.get("commit-generator").unwrap().exists(),
            "Commit gen prompt should have been created"
        );
        assert!(
            user_prompts_dir.join(COMMIT_DIVIATION_PROMPT).exists(),
            "Commit dev prompt should have been created"
        );
        assert!(
            user_prompts_dir.join(HELPER_PROMPT).exists(),
            "Helper prompt should have been created"
        );

        Ok(())
    }

    #[test]
    fn test_load_successful_default_config() -> Result<(), Box<dyn std::error::Error>> {
        let (_temp_dir_guard, _user_config_base_dir, _user_prompts_dir) = setup_test_environment()?;

        let app_config = AppConfig::load()?;

        assert_eq!(
            app_config.ai.api_url,
            "http://localhost:11434/v1/chat/completions"
        );
        assert_eq!(app_config.ai.model_name, "qwen3:32b-q8_0");
        assert!((app_config.ai.temperature - 0.7).abs() < f32::EPSILON);
        assert!(app_config.ai.api_key.is_none());

        assert!(app_config.prompts.contains_key("translator"));
        assert_eq!(
            app_config.prompts.get("translator").unwrap().trim(),
            "Translator AI help prompt."
        );

        assert!(app_config.prompts.contains_key("review"));
        assert_eq!(
            app_config.prompts.get("review").unwrap().trim(),
            "Review prompt."
        );

        assert!(app_config.prompts.contains_key("commit-deviation"));
        assert_eq!(
            app_config.prompts.get("commit-deviation").unwrap().trim(),
            "Explain commit deviation."
        );

        assert!(app_config.prompts.contains_key("general-helper"));
        assert_eq!(
            app_config.prompts.get("general-helper").unwrap().trim(),
            "General AI help prompt."
        );

        let default_ts_config = TreeSitterConfig::default();
        assert_eq!(app_config.tree_sitter.enabled, default_ts_config.enabled);
        assert_eq!(
            app_config.tree_sitter.analysis_depth,
            default_ts_config.analysis_depth
        );
        assert_eq!(
            app_config.tree_sitter.cache_enabled,
            default_ts_config.cache_enabled
        );
        assert_eq!(
            app_config.tree_sitter.languages,
            default_ts_config.languages
        );

        Ok(())
    }

    #[test]
    fn test_load_successful_custom_config() -> Result<(), Box<dyn std::error::Error>> {
        let (_temp_dir_guard, user_config_base_dir, user_prompts_dir) = setup_test_environment()?;

        let custom_config_content = r#"
[ai]
api_url = "https://custom.api.com/v1"
model_name = "custom-gpt"
temperature = 0.9
api_key = "CUSTOM_KEY_123"

[tree_sitter]
enabled = true
analysis_depth = "medium"
cache_enabled = false
languages = ["rust", "python"]
"#;
        let config_file_path = user_config_base_dir.join(CONFIG_FILE_NAME);
        fs::write(&config_file_path, custom_config_content)?;

        let custom_prompt_content = "My custom commit generation prompt.";
        let commit_gen_prompt_path = user_prompts_dir.join(COMMIT_GENERATOR_PROMPT);
        fs::write(&commit_gen_prompt_path, custom_prompt_content)?;

        let app_config = AppConfig::load()?;

        assert_eq!(app_config.ai.api_url, "https://custom.api.com/v1");
        assert_eq!(app_config.ai.model_name, "custom-gpt");
        assert!((app_config.ai.temperature - 0.9).abs() < f32::EPSILON);
        assert_eq!(app_config.ai.api_key, Some("CUSTOM_KEY_123".to_string()));

        assert_eq!(
            app_config.prompts.get("commit-generator").unwrap().trim(),
            custom_prompt_content.trim()
        );

        assert!(app_config.prompts.contains_key("commit-deviation")); // Should load from template
        assert_eq!(
            app_config.prompts.get("commit-deviation").unwrap().trim(),
            "Explain commit deviation."
        );

        assert_eq!(app_config.tree_sitter.enabled, true);
        assert_eq!(app_config.tree_sitter.analysis_depth, "medium");
        assert_eq!(app_config.tree_sitter.cache_enabled, false);
        assert_eq!(
            app_config.tree_sitter.languages,
            vec!["rust".to_string(), "python".to_string()]
        );

        Ok(())
    }

    #[test]
    fn test_load_toml_parse_error() -> Result<(), Box<dyn std::error::Error>> {
        let (_temp_dir_guard, user_config_base_dir, _user_prompts_dir) = setup_test_environment()?;

        let invalid_toml_content = "this is not valid toml content {";
        let config_file_path = user_config_base_dir.join(CONFIG_FILE_NAME);
        fs::write(&config_file_path, invalid_toml_content)?;

        // Verify file exists with invalid content
        assert!(config_file_path.exists(), "Config file should exist");

        // Attempt to load config with invalid TOML
        let result = AppConfig::load();

        // Verify we get an error
        assert!(result.is_err());
        match result {
            Err(ConfigError::TomlParse(_, _)) => {
                // Expected error type
            }
            _ => panic!("Expected TomlParse error"),
        }

        Ok(())
    }

    #[test]
    fn test_load_missing_prompt_file() -> Result<(), Box<dyn std::error::Error>> {
        let (_temp_dir_guard, user_config_base_dir, user_prompts_dir) = setup_test_environment()?;

        let config_content = "[ai]\nmodel_name = \"test-model\"";
        let config_file_path = user_config_base_dir.join(CONFIG_FILE_NAME);
        fs::write(&config_file_path, config_content)?;

        // Create all configuration files
        let (config_path, prompt_paths) = AppConfig::initialize_config()?;

        // Delete the commit-generator prompt to simulate it missing
        let commit_gen_prompt_path = user_prompts_dir.join(COMMIT_GENERATOR_PROMPT);
        assert!(commit_gen_prompt_path.exists());
        fs::remove_file(&commit_gen_prompt_path)?;
        assert!(!commit_gen_prompt_path.exists());

        // Load the configuration with the missing file
        let app_config = AppConfig::load_config_from_file(&config_path, &prompt_paths)?;

        // Verify the missing prompt wasn't loaded
        assert!(!app_config.prompts.contains_key("commit-generator"));
        assert!(app_config.prompts.contains_key("commit-deviation"));
        assert!(app_config.prompts.contains_key("general-helper"));
        assert!(app_config.prompts.contains_key("review"));

        Ok(())
    }

    #[test]
    fn test_load_empty_prompt_file() -> Result<(), Box<dyn std::error::Error>> {
        let (_temp_dir_guard, user_config_base_dir, user_prompts_dir) = setup_test_environment()?;

        let config_content = "[ai]\nmodel_name = \"test-model\"";
        let config_file_path = user_config_base_dir.join(CONFIG_FILE_NAME);
        fs::write(&config_file_path, config_content)?;

        // Create all configuration files
        let (config_path, prompt_paths) = AppConfig::initialize_config()?;

        // Make the general-helper prompt empty
        let general_helper_path = user_prompts_dir.join(HELPER_PROMPT);
        fs::write(&general_helper_path, "")?;

        // Verify it's empty
        assert!(general_helper_path.exists());
        assert_eq!(fs::read_to_string(&general_helper_path)?, "");

        // Load the configuration with the empty file
        let app_config = AppConfig::load_config_from_file(&config_path, &prompt_paths)?;

        // Verify empty prompt is not loaded
        assert!(app_config.prompts.contains_key("commit-generator"));
        assert!(app_config.prompts.contains_key("commit-deviation"));
        assert!(
            !app_config.prompts.contains_key("general-helper"),
            "Empty general-helper prompt should not be loaded"
        );

        Ok(())
    }

    #[test]
    fn test_initialize_rules_copies_missing_files() -> Result<(), Box<dyn std::error::Error>> {
        use std::fs::{self, File};
        use std::io::Write;
        let (temp_dir, _user_config_base, _user_prompts_dir) = setup_test_environment()?;

        // Mimic project root (where queries should be found)
        let project_root = temp_dir.path();
        let queries_dir = project_root.join("queries/rust");
        fs::create_dir_all(&queries_dir)?;

        let scm_files = ["highlights.scm", "injections.scm", "locals.scm"];
        for f in &scm_files {
            let mut file = File::create(queries_dir.join(f))?;
            writeln!(file, "test content for {f}")?;
        }

        // Set CARGO_MANIFEST_DIR so abs_template_path works (simulate from project root)
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", &project_root) };

        // Patch the extract_file_path function if needed, or ensure USER_RULES_PATH resolves into temp_dir
        // (if not, you may want to temporarily override USER_RULES_PATH or mock it)

        // Call initialize_rules and check results
        let result = AppConfig::initialize_rules();
        assert!(result.is_ok());

        // The user rules directory should now have "rust/{*.scm}"
        let user_rules_dir = temp_dir.path().join(".config/gitai/rules/rust");
        fs::create_dir_all(&user_rules_dir)?;

        for f in &scm_files {
            let user_file = user_rules_dir.join(f);
            assert!(
                user_file.exists(),
                "File {f} should be copied to user rules dir"
            );
            let content = fs::read_to_string(&user_file)?;
            assert!(
                content.contains("test content"),
                "File contents should match"
            );
        }

        Ok(())
    }
}

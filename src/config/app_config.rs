use serde::Deserialize;
use std::{collections::HashMap, env, path::PathBuf};
use crate::errors::ConfigError;

use super::{
    ai_config::{AIConfig, PartialAIConfig},
    devops_config::{AccountConfig, PartialAccountConfig},
    tree_sitter_config::{TreeSitterConfig, PartialTreeSitterConfig},
    review_config::{ReviewConfig, PartialReviewConfig},
    scan_config::{ScanConfig, PartialScanConfig},
    language_config::LanguageConfig,
    loader::ConfigLoader,
};

// Configuration location constants
pub const USER_CONFIG_PATH: &str = "~/.config/gitai";
pub const USER_PROMPT_PATH: &str = "~/.config/gitai/prompts";
pub const USER_RULES_PATH: &str = "~/.config/gitai/rules";

// Configuration file names
pub const CONFIG_FILE_NAME: &str = "config.toml";
pub const HELPER_PROMPT: &str = "helper-prompt.md";
pub const TRANSLATOR_PROMPT: &str = "translator.md";
pub const COMMIT_GENERATOR_PROMPT: &str = "commit-generator.md";
pub const COMMIT_DIVIATION_PROMPT: &str = "commit-deviation.md";
pub const REVIEW_PROMPT: &str = "review.md";

// Template file paths
const TEMPLATE_CONFIG_FILE: &str = "assets/config.example.toml";
const TEMPLATE_HELPER: &str = "assets/helper-prompt.md";
const TEMPLATE_TRANSLATOR: &str = "assets/translator.md";
const TEMPLATE_COMMIT_GENERATOR: &str = "assets/commit-generator.md";
const TEMPLATE_COMMIT_DEVIATION: &str = "assets/commit-deviation.md";
const TEMPLATE_REVIEW: &str = "assets/review.md";

// Total configuration files
pub const TOTAL_CONFIG_FILE_COUNT: u32 = 6;

/// Main Application Configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub ai: AIConfig,
    pub tree_sitter: TreeSitterConfig,
    pub review: ReviewConfig,
    pub account: Option<AccountConfig>,
    pub language: LanguageConfig,
    #[serde(skip)]
    pub prompts: HashMap<String, String>,
    pub scan: ScanConfig,
}

/// Partial Application Configuration for loading from files
#[derive(Deserialize, Debug, Default)]
pub struct PartialAppConfig {
    ai: Option<PartialAIConfig>,
    tree_sitter: Option<PartialTreeSitterConfig>,
    review: Option<PartialReviewConfig>,
    account: Option<PartialAccountConfig>,
    language: Option<LanguageConfig>,
    scan: Option<PartialScanConfig>,
}

impl AppConfig {
    /// Load configuration from file and environment
    pub fn load() -> Result<Self, ConfigError> {
        let loader = ConfigLoader::new();
        loader.load_config()
    }

    /// Load configuration with custom base path (for testing)
    pub fn load_with_base_path(base_path: PathBuf) -> Result<Self, ConfigError> {
        let loader = ConfigLoader::with_base_path(base_path);
        loader.load_config()
    }

    /// Create AppConfig from partial config and environment
    pub fn from_partial_and_env(
        partial: Option<PartialAppConfig>,
        env_map: HashMap<String, String>,
        prompts: HashMap<String, String>,
    ) -> Result<Self, ConfigError> {
        let partial = partial.unwrap_or_default();

        // Load AI config
        let ai = AIConfig::from_env_or_file(partial.ai, &env_map)?;

        // Load DevOps account config (optional)
        let account = AccountConfig::from_env_or_file(partial.account, &env_map)?;

        // Load other configs
        let tree_sitter = TreeSitterConfig::from_partial(partial.tree_sitter);
        let review = ReviewConfig::from_partial(partial.review);
        let scan = ScanConfig::from_partial(partial.scan);
        let language = partial.language.unwrap_or_default();

        Ok(AppConfig {
            ai,
            tree_sitter,
            review,
            account,
            language,
            prompts,
            scan,
        })
    }

    /// Get the effective language for output
    pub fn get_effective_language(&self, override_lang: Option<&str>) -> String {
        self.language.get_effective_language(override_lang)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate DevOps account if present
        if let Some(account) = &self.account {
            account.validate()?;
        }

        // Validate language settings
        let effective_lang = self.get_effective_language(None);
        if !LanguageConfig::is_supported_language(&effective_lang) {
            return Err(ConfigError::Other(format!(
                "Unsupported language: {}",
                effective_lang
            )));
        }

        Ok(())
    }
}

/// Get the absolute path of a template file based on project root
pub fn abs_template_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

/// Get template paths for all configuration files
pub fn get_template_paths() -> HashMap<&'static str, &'static str> {
    let mut templates = HashMap::new();
    templates.insert("config", TEMPLATE_CONFIG_FILE);
    templates.insert("helper", TEMPLATE_HELPER);
    templates.insert("translator", TEMPLATE_TRANSLATOR);
    templates.insert("commit_generator", TEMPLATE_COMMIT_GENERATOR);
    templates.insert("commit_deviation", TEMPLATE_COMMIT_DEVIATION);
    templates.insert("review", TEMPLATE_REVIEW);
    templates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abs_template_path() {
        let path = abs_template_path("assets/test.toml");
        assert!(path.to_string_lossy().contains("assets/test.toml"));
    }

    #[test]
    fn test_get_template_paths() {
        let templates = get_template_paths();
        assert_eq!(templates.len(), 6);
        assert!(templates.contains_key("config"));
        assert!(templates.contains_key("helper"));
    }

    #[test]
    fn test_language_override() {
        let config = AppConfig {
            ai: AIConfig::default(),
            tree_sitter: TreeSitterConfig::default(),
            review: ReviewConfig::default(),
            account: None,
            language: LanguageConfig::default(),
            prompts: HashMap::new(),
            scan: ScanConfig::default(),
        };

        assert_eq!(config.get_effective_language(Some("us")), "us");
        assert_eq!(config.get_effective_language(None), "cn"); // default primary
    }
}
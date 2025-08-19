use serde::Deserialize;
use std::{collections::HashMap, env, path::PathBuf};
use crate::errors::{AppError, config_error};

use super::{
    ai_config::{AIConfig, ResolvedAIConfig},
    devops_config::{AccountConfig, ResolvedAccountConfig},
    tree_sitter_config::TreeSitterConfig,
    review_config::ReviewConfig,
    scan_config::ScanConfig,
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
// 1 config file + 5 default prompts + 5 cn prompts + 5 en prompts = 16 files
pub const TOTAL_CONFIG_FILE_COUNT: u32 = 16;

/// 主应用配置 - 使用解析后的配置
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub ai: ResolvedAIConfig,
    pub tree_sitter: TreeSitterConfig,
    pub review: ReviewConfig,
    pub account: Option<ResolvedAccountConfig>,
    pub prompts: HashMap<String, String>,
    pub scan: ScanConfig,
}

/// 应用配置 - 简化版本，直接使用Option字段
#[derive(Deserialize, Debug, Default)]
pub struct PartialAppConfig {
    pub ai: Option<AIConfig>,
    pub tree_sitter: Option<TreeSitterConfig>,
    pub review: Option<ReviewConfig>,
    pub account: Option<AccountConfig>,
    pub scan: Option<ScanConfig>,
}

impl AppConfig {
    /// Load configuration from file and environment
    pub fn load() -> Result<Self, AppError> {
        let loader = ConfigLoader::new();
        loader.load_config()
    }


    /// 从部分配置和环境变量创建AppConfig
    pub fn from_partial_and_env(
        partial: Option<PartialAppConfig>,
        env_map: HashMap<String, String>,
        prompts: HashMap<String, String>,
    ) -> Result<Self, AppError> {
        let partial = partial.unwrap_or_default();

        // 加载并解析AI配置
        let ai_config = partial.ai.unwrap_or_default();
        let ai = ai_config.merge_with_env(&env_map)?;

        // 加载并解析DevOps账户配置（可选）
        let account_config = partial.account.unwrap_or_default();
        let account = account_config.merge_with_env(&env_map).resolve();

        // 加载并解析其他配置
        let tree_sitter = partial.tree_sitter.unwrap_or_default().resolve();
        let review = partial.review.unwrap_or_default().resolve();
        let scan = partial.scan.unwrap_or_default().resolve();
        
        Ok(AppConfig {
            ai,
            tree_sitter,
            review,
            account,
            prompts,
            scan,
        })
    }

    
    /// Get prompt file path (for backward compatibility)
    pub fn get_prompt_path(&self, prompt_key: &str) -> Result<std::path::PathBuf, AppError> {
        let prompt_file = match prompt_key {
            "translator" => TRANSLATOR_PROMPT,
            "helper" => HELPER_PROMPT,
            "commit_generator" => COMMIT_GENERATOR_PROMPT,
            "commit_deviation" => COMMIT_DIVIATION_PROMPT,
            "review" => REVIEW_PROMPT,
            _ => return Err(config_error(format!("未知的提示词类型: {}", prompt_key))),
        };
        Ok(std::path::PathBuf::from(USER_PROMPT_PATH).join(prompt_file))
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
}
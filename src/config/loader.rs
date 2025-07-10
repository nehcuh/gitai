use std::{collections::HashMap, env, fs, path::PathBuf};
use crate::errors::ConfigError;

use super::{
    app_config::{AppConfig, PartialAppConfig, abs_template_path, get_template_paths},
    app_config::{
        USER_CONFIG_PATH, USER_PROMPT_PATH, CONFIG_FILE_NAME,
        HELPER_PROMPT, TRANSLATOR_PROMPT, COMMIT_GENERATOR_PROMPT,
        COMMIT_DIVIATION_PROMPT, REVIEW_PROMPT, TOTAL_CONFIG_FILE_COUNT
    },
};

/// Configuration loader responsible for loading config from files and environment
pub struct ConfigLoader {
    base_path: Option<PathBuf>,
}

impl ConfigLoader {
    /// Create a new config loader with default paths
    pub fn new() -> Self {
        Self { base_path: None }
    }

    /// Create a config loader with custom base path (for testing)
    pub fn with_base_path(base_path: PathBuf) -> Self {
        Self { base_path: Some(base_path) }
    }

    /// Load complete application configuration
    pub fn load_config(&self) -> Result<AppConfig, ConfigError> {
        // Initialize configuration files if they don't exist
        let (config_path, prompt_paths) = self.initialize_config()?;

        // Load partial config from file
        let partial_config = self.load_partial_config(&config_path)?;

        // Collect environment variables
        let env_map = self.collect_env_vars();

        // Load prompts
        let prompts = self.load_prompts(&prompt_paths)?;

        // Create final config
        AppConfig::from_partial_and_env(partial_config, env_map, prompts)
    }

    /// Initialize configuration files and directories
    pub fn initialize_config(&self) -> Result<(PathBuf, HashMap<String, PathBuf>), ConfigError> {
        let user_config_path = self.extract_file_path(USER_CONFIG_PATH, CONFIG_FILE_NAME)?;

        // Build prompt file paths for all languages
        let mut user_prompt_paths = HashMap::new();
        
        // Supported languages
        let languages = ["cn", "en"];
        let prompt_types = [
            ("commit_generator", COMMIT_GENERATOR_PROMPT),
            ("commit_deviation", COMMIT_DIVIATION_PROMPT),
            ("general_helper", HELPER_PROMPT),
            ("translator", TRANSLATOR_PROMPT),
            ("review", REVIEW_PROMPT),
        ];

        // Add default prompts (fallback)
        for (key, filename) in &prompt_types {
            let path = self.extract_file_path(USER_PROMPT_PATH, filename)?;
            user_prompt_paths.insert(key.to_string(), path);
        }

        // Add language-specific prompts
        for lang in &languages {
            for (key, _) in &prompt_types {
                let lang_filename = format!("{}.{}.md", key.replace("_", "-"), lang);
                let path = self.extract_file_path(USER_PROMPT_PATH, &lang_filename)?;
                let lang_key = format!("{}_{}", key, lang);
                user_prompt_paths.insert(lang_key, path);
            }
        }

        // Check existing files
        let existing_count = self.count_existing_files(&user_config_path, &user_prompt_paths);
        self.log_existing_files(existing_count)?;

        // Create directories - use any prompt path for directory creation
        let sample_prompt_path = user_prompt_paths.values().next()
            .ok_or_else(|| ConfigError::Other("No prompt paths available".to_string()))?;
        self.create_config_directories(&user_config_path, sample_prompt_path)?;

        // Initialize missing files
        self.initialize_missing_files(&user_config_path, &user_prompt_paths)?;

        Ok((user_config_path, user_prompt_paths))
    }

    /// Extract file path with tilde expansion and base path override
    fn extract_file_path(&self, base_dir: &str, file_name: &str) -> Result<PathBuf, ConfigError> {
        let expanded_base = if let Some(base_path) = &self.base_path {
            // For testing: use custom base path
            base_path.join(base_dir.trim_start_matches("~/"))
        } else {
            // Normal operation: expand tilde
            let expanded = shellexpand::tilde(base_dir);
            PathBuf::from(expanded.as_ref())
        };

        Ok(expanded_base.join(file_name))
    }

    /// Count existing configuration files
    fn count_existing_files(
        &self,
        config_path: &PathBuf,
        prompt_paths: &HashMap<String, PathBuf>
    ) -> u32 {
        let mut count = 0;
        if config_path.exists() { count += 1; }
        for path in prompt_paths.values() {
            if path.exists() { count += 1; }
        }
        count
    }

    /// Log information about existing files
    fn log_existing_files(&self, existing_count: u32) -> Result<(), ConfigError> {
        match existing_count {
            count if count == TOTAL_CONFIG_FILE_COUNT => {
                tracing::info!("所有 {} 个配置文件已存在，将直接使用", count);
            }
            count if count < TOTAL_CONFIG_FILE_COUNT => {
                tracing::info!(
                    "发现 {}/{} 个配置文件已存在，将补充缺失的配置",
                    count, TOTAL_CONFIG_FILE_COUNT
                );
            }
            count if count > TOTAL_CONFIG_FILE_COUNT => {
                return Err(ConfigError::Other(format!(
                    "发现 {}/{} 个配置文件，超过全局配置文件需求",
                    count, TOTAL_CONFIG_FILE_COUNT,
                )));
            }
            0 => {
                tracing::info!("未发现任何配置文件，将创建并使用默认配置文件");
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    /// Create configuration directories
    fn create_config_directories(
        &self,
        config_path: &PathBuf,
        prompt_path: &PathBuf
    ) -> Result<(), ConfigError> {
        // Create config directory
        if let Some(config_dir) = config_path.parent() {
            fs::create_dir_all(config_dir).map_err(|e| {
                ConfigError::FileWrite(config_dir.to_string_lossy().to_string(), e)
            })?;
        }

        // Create prompt directory
        if let Some(prompt_dir) = prompt_path.parent() {
            fs::create_dir_all(prompt_dir).map_err(|e| {
                ConfigError::FileWrite(prompt_dir.to_string_lossy().to_string(), e)
            })?;
        }

        Ok(())
    }

    /// Initialize missing configuration files
    fn initialize_missing_files(
        &self,
        config_path: &PathBuf,
        prompt_paths: &HashMap<String, PathBuf>
    ) -> Result<(), ConfigError> {
        let templates = get_template_paths();

        // Initialize config file
        if !config_path.exists() {
            tracing::info!("配置文件 {} 不存在，正在初始化", CONFIG_FILE_NAME);
            self.initialize_config_file(config_path, templates["config"])?;
        }

        // Initialize prompt files for all languages
        self.initialize_prompt_files_for_all_languages(prompt_paths, &templates)?;

        Ok(())
    }

    /// Initialize a single config file from template
    fn initialize_config_file(&self, target_path: &PathBuf, template_path: &str) -> Result<(), ConfigError> {
        let template_full_path = abs_template_path(template_path);
        let content = fs::read_to_string(&template_full_path).map_err(|e| {
            ConfigError::FileRead(template_full_path.to_string_lossy().to_string(), e)
        })?;

        fs::write(target_path, content).map_err(|e| {
            ConfigError::FileWrite(target_path.to_string_lossy().to_string(), e)
        })?;

        tracing::info!("已初始化配置文件: {:?}", target_path);
        Ok(())
    }

    /// Initialize prompt files for all languages
    fn initialize_prompt_files_for_all_languages(
        &self,
        prompt_paths: &HashMap<String, PathBuf>,
        templates: &HashMap<&str, &str>
    ) -> Result<(), ConfigError> {
        // Initialize language-specific prompts
        let languages = ["cn", "en"];
        let prompt_types = [
            ("commit_generator", "commit-generator"),
            ("commit_deviation", "commit-deviation"),
            ("general_helper", "helper-prompt"),
            ("translator", "translator"),
            ("review", "review"),
        ];

        // Initialize default prompts first (fallback)
        for (key, template_key) in &prompt_types {
            if let Some(path) = prompt_paths.get(*key) {
                if !path.exists() {
                    tracing::info!("提示词文件 {:?} 不存在，正在初始化", path.file_name());
                    self.initialize_prompt_file_from_assets(path, template_key, "cn")?;
                }
            }
        }

        // Initialize language-specific prompts
        for lang in &languages {
            for (key, template_key) in &prompt_types {
                let lang_key = format!("{}_{}", key, lang);
                if let Some(path) = prompt_paths.get(&lang_key) {
                    if !path.exists() {
                        tracing::info!("语言特定提示词文件 {:?} 不存在，正在初始化", path.file_name());
                        self.initialize_prompt_file_from_assets(path, template_key, lang)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Initialize a single prompt file from template
    fn initialize_prompt_file(&self, target_path: &PathBuf, template_path: &str) -> Result<(), ConfigError> {
        let template_full_path = abs_template_path(template_path);
        let content = fs::read_to_string(&template_full_path).map_err(|e| {
            ConfigError::FileRead(template_full_path.to_string_lossy().to_string(), e)
        })?;

        fs::write(target_path, content).map_err(|e| {
            ConfigError::FileWrite(target_path.to_string_lossy().to_string(), e)
        })?;

        tracing::info!("已初始化提示词文件: {:?}", target_path);
        Ok(())
    }

    /// Initialize a prompt file from assets directory with language support
    fn initialize_prompt_file_from_assets(&self, target_path: &PathBuf, template_key: &str, language: &str) -> Result<(), ConfigError> {
        let template_path = format!("assets/prompts/{}/{}.md", language, template_key.replace("_", "-"));
        let template_full_path = abs_template_path(&template_path);
        
        let content = if template_full_path.exists() {
            // Use language-specific template if available
            fs::read_to_string(&template_full_path).map_err(|e| {
                ConfigError::FileRead(template_full_path.to_string_lossy().to_string(), e)
            })?
        } else {
            // Fall back to default template from root assets directory
            let fallback_template = format!("assets/{}.md", template_key.replace("_", "-"));
            let fallback_path = abs_template_path(&fallback_template);
            fs::read_to_string(&fallback_path).map_err(|e| {
                ConfigError::FileRead(fallback_path.to_string_lossy().to_string(), e)
            })?
        };

        fs::write(target_path, content).map_err(|e| {
            ConfigError::FileWrite(target_path.to_string_lossy().to_string(), e)
        })?;

        tracing::info!("已初始化{}语言提示词文件: {:?}", language, target_path);
        Ok(())
    }

    /// Load partial configuration from TOML file
    fn load_partial_config(&self, config_path: &PathBuf) -> Result<Option<PartialAppConfig>, ConfigError> {
        if !config_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(config_path).map_err(|e| {
            ConfigError::FileRead(config_path.to_string_lossy().to_string(), e)
        })?;

        let partial_config: PartialAppConfig = toml::from_str(&content).map_err(|e| {
            ConfigError::Other(format!("Failed to parse config file: {}", e))
        })?;

        Ok(Some(partial_config))
    }

    /// Collect relevant environment variables
    fn collect_env_vars(&self) -> HashMap<String, String> {
        let env_keys = [
            // AI config
            "GITAI_AI_API_URL",
            "GITAI_AI_MODEL", 
            "GITAI_AI_TEMPERATURE",
            "GITAI_AI_API_KEY",
            // DevOps config
            "GITAI_DEVOPS_PLATFORM",
            "GITAI_DEVOPS_BASE_URL",
            "GITAI_DEVOPS_TOKEN",
        ];

        let mut env_map = HashMap::new();
        for key in &env_keys {
            if let Ok(value) = env::var(key) {
                env_map.insert(key.to_string(), value);
            }
        }
        env_map
    }

    /// Load prompt templates from files
    fn load_prompts(&self, prompt_paths: &HashMap<String, PathBuf>) -> Result<HashMap<String, String>, ConfigError> {
        let mut prompts = HashMap::new();

        for (key, path) in prompt_paths {
            if path.exists() {
                let content = fs::read_to_string(path).map_err(|e| {
                    ConfigError::FileRead(path.to_string_lossy().to_string(), e)
                })?;
                prompts.insert(key.clone(), content);
            }
        }

        Ok(prompts)
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_loader() -> (ConfigLoader, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let loader = ConfigLoader::with_base_path(temp_dir.path().to_path_buf());
        (loader, temp_dir)
    }

    #[test]
    fn test_extract_file_path() {
        let (loader, _temp_dir) = create_test_loader();
        let path = loader.extract_file_path("~/.config/gitai", "config.toml").unwrap();
        assert!(path.to_string_lossy().contains("config.toml"));
    }

    #[test]
    fn test_count_existing_files() {
        let (loader, temp_dir) = create_test_loader();
        
        // Create config file
        let config_path = temp_dir.path().join(".config/gitai/config.toml");
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        fs::write(&config_path, "# test config").unwrap();

        let prompt_paths = HashMap::new();
        let count = loader.count_existing_files(&config_path, &prompt_paths);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_collect_env_vars() {
        env::set_var("GITAI_AI_API_URL", "http://test.com");
        env::set_var("GITAI_AI_MODEL", "test-model");
        
        let loader = ConfigLoader::new();
        let env_map = loader.collect_env_vars();
        
        assert_eq!(env_map.get("GITAI_AI_API_URL"), Some(&"http://test.com".to_string()));
        assert_eq!(env_map.get("GITAI_AI_MODEL"), Some(&"test-model".to_string()));
        
        // Clean up
        env::remove_var("GITAI_AI_API_URL");
        env::remove_var("GITAI_AI_MODEL");
    }
}
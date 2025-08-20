use serde::Deserialize;

/// Tree-sitter 配置 - 简化版本，直接使用Option字段
#[derive(Deserialize, Debug, Clone, Default)]
pub struct TreeSitterConfig {
    /// 是否启用AST分析
    pub enabled: Option<bool>,

    /// 是否启用缓存
    pub cache_enabled: Option<bool>,

    /// 支持的语言列表
    pub languages: Option<Vec<String>>,
}

/// Tree-sitter 配置建造者
#[derive(Debug, Clone)]
pub struct TreeSitterConfigBuilder {
    config: TreeSitterConfig,
}

impl TreeSitterConfigBuilder {
    /// 创建新的建造者实例
    pub fn new() -> Self {
        Self {
            config: TreeSitterConfig::default(),
        }
    }

    /// 设置是否启用AST分析
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = Some(enabled);
        self
    }

  
    /// 设置是否启用缓存
    pub fn cache_enabled(mut self, enabled: bool) -> Self {
        self.config.cache_enabled = Some(enabled);
        self
    }

    /// 设置支持的语言列表
    pub fn languages(mut self, languages: Vec<String>) -> Result<Self, String> {
        if languages.is_empty() {
            return Err("languages list cannot be empty".to_string());
        }
        
        // 检查是否有重复的语言
        let mut unique_languages = std::collections::HashSet::new();
        for lang in &languages {
            if !unique_languages.insert(lang.clone()) {
                return Err(format!("duplicate language: {}", lang));
            }
        }
        
        self.config.languages = Some(languages);
        Ok(self)
    }

    /// 添加单个语言
    pub fn add_language<S: Into<String>>(mut self, language: S) -> Self {
        let lang_str = language.into();
        if let Some(ref mut languages) = self.config.languages {
            if !languages.contains(&lang_str) {
                languages.push(lang_str);
            }
        } else {
            self.config.languages = Some(vec![lang_str]);
        }
        self
    }

    /// 构建配置
    pub fn build(self) -> Result<TreeSitterConfig, String> {
        // 验证配置
        
        if let Some(ref languages) = self.config.languages {
            if languages.is_empty() {
                return Err("languages list cannot be empty".to_string());
            }
        }
        
        Ok(self.config)
    }

    /// 构建并解析配置（应用默认值）
    pub fn build_resolved(self) -> Result<TreeSitterConfig, crate::errors::AppError> {
        let config = self.build()
            .map_err(|e| crate::errors::config_error(e))?;
        config.resolve()
    }
}

impl Default for TreeSitterConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeSitterConfig {
    /// 创建建造者
    pub fn builder() -> TreeSitterConfigBuilder {
        TreeSitterConfigBuilder::new()
    }

    /// 解析配置，应用默认值
    pub fn resolve(self) -> Result<Self, crate::errors::AppError> {
        let resolved = Self {
            enabled: Some(self.enabled.unwrap_or_else(default_enabled)),
            cache_enabled: Some(self.cache_enabled.unwrap_or_else(default_cache_enabled)),
            languages: Some(self.languages.unwrap_or_else(default_languages)),
        };
        
        // 验证配置
        resolved.validate()?;
        
        Ok(resolved)
    }

    /// 验证配置值的有效性
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        // 验证语言列表
        if let Some(ref languages) = self.languages {
            if languages.is_empty() {
                return Err(crate::errors::config_error("languages list cannot be empty"));
            }
            
            // 检查是否有重复的语言
            let mut unique_languages = std::collections::HashSet::new();
            for lang in languages {
                if !unique_languages.insert(lang.clone()) {
                    return Err(crate::errors::config_invalid_value("languages", format!("duplicate language: {}", lang)));
                }
            }
        }
        
        Ok(())
    }

    /// 获取缓存状态
    pub fn is_cache_enabled(&self) -> bool {
        self.cache_enabled.unwrap_or_else(default_cache_enabled)
    }

    /// 获取支持的语言
    pub fn get_languages(&self) -> Vec<String> {
        self.languages.as_ref()
            .unwrap_or(&default_languages())
            .clone()
    }

    /// 检查是否支持特定语言
    pub fn is_language_supported(&self, language: &str) -> bool {
        self.get_languages().contains(&language.to_string())
    }

    }

// Default functions
fn default_enabled() -> bool {
    true
}


fn default_cache_enabled() -> bool {
    true
}

fn default_languages() -> Vec<String> {
    vec![
        "rust".to_string(),
        "python".to_string(),
        "javascript".to_string(),
        "typescript".to_string(),
        "go".to_string(),
        "java".to_string(),
        "c".to_string(),
        "cpp".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TreeSitterConfig::default();
        assert!(config.is_cache_enabled());
        assert_eq!(config.get_languages().len(), 8);
    }

    #[test]
    fn test_resolve_config() {
        let config = TreeSitterConfig {
            enabled: Some(false),
            cache_enabled: None, // Should use default
            languages: Some(vec!["rust".to_string(), "python".to_string()]),
        };

        let resolved = config.resolve().unwrap();
        assert!(!resolved.enabled.unwrap()); // from config
        assert!(resolved.cache_enabled.unwrap()); // default
        assert_eq!(resolved.languages.unwrap().len(), 2); // from config
    }

    #[test]
    fn test_getter_methods() {
        let config = TreeSitterConfig {
            enabled: Some(false),
            cache_enabled: None,
            languages: None,
        };

        assert!(!config.enabled.unwrap()); // from config
        assert!(config.is_cache_enabled()); // default
        assert_eq!(config.get_languages().len(), 8); // default
    }

    #[test]
    fn test_validate_config() {
        let config = TreeSitterConfig {
            enabled: Some(true),
            cache_enabled: Some(true),
            languages: Some(vec!["rust".to_string()]),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_languages() {
        let config = TreeSitterConfig {
            enabled: Some(true),
            cache_enabled: Some(true),
            languages: Some(vec![]),
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_duplicate_languages() {
        let config = TreeSitterConfig {
            enabled: Some(true),
            cache_enabled: Some(true),
            languages: Some(vec!["rust".to_string(), "rust".to_string()]),
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_is_language_supported() {
        let config = TreeSitterConfig {
            enabled: Some(true),
            cache_enabled: Some(true),
            languages: Some(vec!["rust".to_string(), "python".to_string()]),
        };

        assert!(config.is_language_supported("rust"));
        assert!(!config.is_language_supported("java"));
    }


    // 建造者模式测试
    #[test]
    fn test_builder_basic() {
        let config = TreeSitterConfig::builder()
            .enabled(false)
            .cache_enabled(true)
            .languages(vec!["rust".to_string(), "python".to_string()]).unwrap()
            .build()
            .unwrap();

        assert!(!config.enabled.unwrap());
        assert!(config.cache_enabled.unwrap());
        assert_eq!(config.languages.unwrap().len(), 2);
    }

    #[test]
    fn test_builder_add_language() {
        let config = TreeSitterConfig::builder()
            .enabled(true)
            .add_language("rust")
            .add_language("python")
            .add_language("rust") // 重复添加，应该被忽略
            .build()
            .unwrap();

        let languages = config.languages.unwrap();
        assert_eq!(languages.len(), 2);
        assert!(languages.contains(&"rust".to_string()));
        assert!(languages.contains(&"python".to_string()));
    }

  
    #[test]
    fn test_builder_empty_languages() {
        let result = TreeSitterConfig::builder()
            .languages(vec![])
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("languages list cannot be empty"));
    }

    #[test]
    fn test_builder_duplicate_languages() {
        let result = TreeSitterConfig::builder()
            .languages(vec!["rust".to_string(), "rust".to_string()])
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("duplicate language"));
    }

    #[test]
    fn test_builder_build_resolved() {
        let config = TreeSitterConfig::builder()
            .enabled(false)
            .add_language("rust")
            .build_resolved()
            .unwrap();

        assert!(!config.enabled.unwrap());
        assert!(config.is_cache_enabled()); // 使用默认值
        assert_eq!(config.get_languages().len(), 1);
    }

    #[test]
    fn test_builder_default() {
        let config = TreeSitterConfigBuilder::default()
            .build()
            .unwrap();

        assert!(config.is_cache_enabled());
        assert_eq!(config.get_languages().len(), 8);
    }
}
use serde::Deserialize;

/// Tree-sitter 配置 - 简化版本，直接使用Option字段
#[derive(Deserialize, Debug, Clone, Default)]
pub struct TreeSitterConfig {
    /// 是否启用AST分析
    pub enabled: Option<bool>,

    /// 分析深度: "shallow", "medium", "deep"
    pub analysis_depth: Option<String>,

    /// 是否启用缓存
    pub cache_enabled: Option<bool>,

    /// 支持的语言列表
    pub languages: Option<Vec<String>>,
}

impl TreeSitterConfig {
    /// 解析配置，应用默认值
    pub fn resolve(self) -> Result<Self, crate::errors::AppError> {
        let resolved = Self {
            enabled: Some(self.enabled.unwrap_or_else(default_enabled)),
            analysis_depth: Some(self.analysis_depth.unwrap_or_else(default_analysis_depth)),
            cache_enabled: Some(self.cache_enabled.unwrap_or_else(default_cache_enabled)),
            languages: Some(self.languages.unwrap_or_else(default_languages)),
        };
        
        // 验证配置
        resolved.validate()?;
        
        Ok(resolved)
    }

    /// 验证配置值的有效性
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        // 验证分析深度
        if let Some(ref depth) = self.analysis_depth {
            if !["shallow", "basic", "medium", "deep"].contains(&depth.as_str()) {
                return Err(crate::errors::AppError::Config(crate::errors::ConfigError::InvalidValue {
                    field: "analysis_depth".to_string(),
                    value: depth.clone(),
                }));
            }
        }
        
        // 验证语言列表
        if let Some(ref languages) = self.languages {
            if languages.is_empty() {
                return Err(crate::errors::AppError::Config(crate::errors::ConfigError::Validation(
                    "languages list cannot be empty".to_string(),
                )));
            }
            
            // 检查是否有重复的语言
            let mut unique_languages = std::collections::HashSet::new();
            for lang in languages {
                if !unique_languages.insert(lang.clone()) {
                    return Err(crate::errors::AppError::Config(crate::errors::ConfigError::InvalidValue {
                        field: "languages".to_string(),
                        value: format!("duplicate language: {}", lang),
                    }));
                }
            }
        }
        
        Ok(())
    }

    /// 获取分析深度
    pub fn get_analysis_depth(&self) -> String {
        self.analysis_depth.as_ref()
            .unwrap_or(&default_analysis_depth())
            .clone()
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

    /// 获取规范化的分析深度
    pub fn get_normalized_analysis_depth(&self) -> &'static str {
        match self.get_analysis_depth().as_str() {
            "shallow" | "basic" => "basic",
            "medium" => "medium",
            "deep" => "deep",
            _ => "medium", // 默认值
        }
    }
}

// Default functions
fn default_enabled() -> bool {
    true
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
        assert_eq!(config.get_analysis_depth(), "medium");
        assert!(config.is_cache_enabled());
        assert_eq!(config.get_languages().len(), 8);
    }

    #[test]
    fn test_resolve_config() {
        let config = TreeSitterConfig {
            enabled: Some(false),
            analysis_depth: Some("deep".to_string()),
            cache_enabled: None, // Should use default
            languages: Some(vec!["rust".to_string(), "python".to_string()]),
        };

        let resolved = config.resolve().unwrap();
        assert!(!resolved.enabled.unwrap()); // from config
        assert_eq!(resolved.analysis_depth.unwrap(), "deep"); // from config
        assert!(resolved.cache_enabled.unwrap()); // default
        assert_eq!(resolved.languages.unwrap().len(), 2); // from config
    }

    #[test]
    fn test_getter_methods() {
        let config = TreeSitterConfig {
            enabled: Some(false),
            analysis_depth: Some("deep".to_string()),
            cache_enabled: None,
            languages: None,
        };

        assert!(!config.enabled.unwrap()); // from config
        assert_eq!(config.get_analysis_depth(), "deep"); // from config
        assert!(config.is_cache_enabled()); // default
        assert_eq!(config.get_languages().len(), 8); // default
    }

    #[test]
    fn test_validate_config() {
        let config = TreeSitterConfig {
            enabled: Some(true),
            analysis_depth: Some("invalid".to_string()),
            cache_enabled: Some(true),
            languages: Some(vec!["rust".to_string()]),
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_empty_languages() {
        let config = TreeSitterConfig {
            enabled: Some(true),
            analysis_depth: Some("medium".to_string()),
            cache_enabled: Some(true),
            languages: Some(vec![]),
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_duplicate_languages() {
        let config = TreeSitterConfig {
            enabled: Some(true),
            analysis_depth: Some("medium".to_string()),
            cache_enabled: Some(true),
            languages: Some(vec!["rust".to_string(), "rust".to_string()]),
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_is_language_supported() {
        let config = TreeSitterConfig {
            enabled: Some(true),
            analysis_depth: Some("medium".to_string()),
            cache_enabled: Some(true),
            languages: Some(vec!["rust".to_string(), "python".to_string()]),
        };

        assert!(config.is_language_supported("rust"));
        assert!(!config.is_language_supported("java"));
    }

    #[test]
    fn test_get_normalized_analysis_depth() {
        let config = TreeSitterConfig {
            enabled: Some(true),
            analysis_depth: Some("shallow".to_string()),
            cache_enabled: Some(true),
            languages: Some(vec!["rust".to_string()]),
        };

        assert_eq!(config.get_normalized_analysis_depth(), "basic");
    }
}
use serde::Deserialize;
use crate::tree_sitter_analyzer::query_manager::QueryManagerConfig;

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

    /// 查询管理器配置
    #[serde(skip)]
    pub query_manager_config: QueryManagerConfig,
}

impl TreeSitterConfig {
    /// 解析配置，应用默认值
    pub fn resolve(self) -> Self {
        Self {
            enabled: Some(self.enabled.unwrap_or_else(default_enabled)),
            analysis_depth: Some(self.analysis_depth.unwrap_or_else(default_analysis_depth)),
            cache_enabled: Some(self.cache_enabled.unwrap_or_else(default_cache_enabled)),
            languages: Some(self.languages.unwrap_or_else(default_languages)),
            query_manager_config: QueryManagerConfig::default(),
        }
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
            query_manager_config: QueryManagerConfig::default(),
        };

        let resolved = config.resolve();
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
            query_manager_config: QueryManagerConfig::default(),
        };

        assert!(!config.enabled.unwrap()); // from config
        assert_eq!(config.get_analysis_depth(), "deep"); // from config
        assert!(config.is_cache_enabled()); // default
        assert_eq!(config.get_languages().len(), 8); // default
    }
}
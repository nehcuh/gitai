use serde::Deserialize;
use crate::tree_sitter_analyzer::query_manager::QueryManagerConfig;

/// Tree-sitter Configuration
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

    /// Query manager configuration
    #[serde(skip)]
    pub query_manager_config: QueryManagerConfig,
}

/// Partial Tree-sitter Configuration for loading from files
#[derive(Deserialize, Debug, Default)]
pub struct PartialTreeSitterConfig {
    pub enabled: Option<bool>,
    pub analysis_depth: Option<String>,
    pub cache_enabled: Option<bool>,
    pub languages: Option<Vec<String>>,
}

impl Default for TreeSitterConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            analysis_depth: default_analysis_depth(),
            cache_enabled: default_cache_enabled(),
            languages: default_languages(),
            query_manager_config: QueryManagerConfig::default(),
        }
    }
}

impl TreeSitterConfig {
    /// Create TreeSitterConfig from partial config with defaults
    pub fn from_partial(partial: Option<PartialTreeSitterConfig>) -> Self {
        let partial = partial.unwrap_or_default();
        
        Self {
            enabled: partial.enabled.unwrap_or_else(default_enabled),
            analysis_depth: partial.analysis_depth.unwrap_or_else(default_analysis_depth),
            cache_enabled: partial.cache_enabled.unwrap_or_else(default_cache_enabled),
            languages: partial.languages.unwrap_or_else(default_languages),
            query_manager_config: QueryManagerConfig::default(),
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
        assert!(config.enabled);
        assert_eq!(config.analysis_depth, "medium");
        assert!(config.cache_enabled);
        assert_eq!(config.languages.len(), 8);
    }

    #[test]
    fn test_from_partial_config() {
        let partial = PartialTreeSitterConfig {
            enabled: Some(false),
            analysis_depth: Some("deep".to_string()),
            cache_enabled: None, // Should use default
            languages: Some(vec!["rust".to_string(), "python".to_string()]),
        };

        let config = TreeSitterConfig::from_partial(Some(partial));
        assert!(!config.enabled); // from partial
        assert_eq!(config.analysis_depth, "deep"); // from partial
        assert!(config.cache_enabled); // default
        assert_eq!(config.languages.len(), 2); // from partial
    }

    #[test]
    fn test_from_none_partial() {
        let config = TreeSitterConfig::from_partial(None);
        assert!(config.enabled);
        assert_eq!(config.analysis_depth, "medium");
        assert!(config.cache_enabled);
        assert_eq!(config.languages.len(), 8);
    }
}
//! Translation module for AST-Grep rules localization
//!
//! This module provides functionality to translate AST-Grep rules from English
//! to other languages (primarily Chinese) and manage translation caching.

pub mod cache_manager;
pub mod rule_localizer;
pub mod translator;

// Re-export main types for easier access
pub use cache_manager::{TranslationCache, TranslationCacheManager};
pub use rule_localizer::{LocalizedRule, RuleLocalizer};
pub use translator::{RuleTranslator, TranslationProvider};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported languages for rule translation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupportedLanguage {
    /// English (original language)
    English,
    /// Simplified Chinese
    Chinese,
    /// Auto-detect based on system locale
    Auto,
}

impl SupportedLanguage {
    /// Get the language code string
    pub fn code(&self) -> &'static str {
        match self {
            SupportedLanguage::English => "en",
            SupportedLanguage::Chinese => "zh",
            SupportedLanguage::Auto => "auto",
        }
    }

    /// Parse language from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Some(SupportedLanguage::English),
            "zh" | "chinese" | "zh-cn" | "zh_cn" => Some(SupportedLanguage::Chinese),
            "auto" => Some(SupportedLanguage::Auto),
            _ => None,
        }
    }

    /// Get system default language
    pub fn system_default() -> Self {
        // Try to detect system language, fallback to English
        if let Ok(lang) = std::env::var("LANG") {
            if lang.starts_with("zh") {
                return SupportedLanguage::Chinese;
            }
        }
        SupportedLanguage::English
    }
}

/// Translation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationConfig {
    /// Whether translation is enabled
    pub enabled: bool,
    /// Default target language
    pub default_language: SupportedLanguage,
    /// Whether to use cache
    pub cache_enabled: bool,
    /// Translation provider
    pub provider: String,
    /// Custom cache directory
    pub cache_dir: Option<std::path::PathBuf>,
    /// AI provider specific settings
    pub provider_settings: HashMap<String, String>,
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for backward compatibility
            default_language: SupportedLanguage::Auto,
            cache_enabled: true,
            provider: "openai".to_string(),
            cache_dir: None,
            provider_settings: HashMap::new(),
        }
    }
}

/// Error types for translation operations
#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    #[error("Translation provider error: {0}")]
    ProviderError(String),

    #[error("Cache operation failed: {0}")]
    CacheError(String),

    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("Translation timeout")]
    Timeout,

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Result type for translation operations
pub type TranslationResult<T> = Result<T, TranslationError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_language_code() {
        assert_eq!(SupportedLanguage::English.code(), "en");
        assert_eq!(SupportedLanguage::Chinese.code(), "zh");
        assert_eq!(SupportedLanguage::Auto.code(), "auto");
    }

    #[test]
    fn test_supported_language_from_str() {
        assert_eq!(
            SupportedLanguage::from_str("en"),
            Some(SupportedLanguage::English)
        );
        assert_eq!(
            SupportedLanguage::from_str("zh"),
            Some(SupportedLanguage::Chinese)
        );
        assert_eq!(
            SupportedLanguage::from_str("auto"),
            Some(SupportedLanguage::Auto)
        );
        assert_eq!(SupportedLanguage::from_str("invalid"), None);
    }

    #[test]
    fn test_translation_config_default() {
        let config = TranslationConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.default_language, SupportedLanguage::Auto);
        assert!(config.cache_enabled);
        assert_eq!(config.provider, "openai");
    }
}

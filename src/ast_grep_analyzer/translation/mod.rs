//! Translation module for AST-Grep rules localization
//!
//! This module provides functionality to translate AST-Grep rules from English
//! to other languages (primarily Chinese) and manage translation caching.

pub mod cache_manager;
pub mod manager;
pub mod rule_localizer;
pub mod translator;

// Re-export main types for easier access
pub use cache_manager::{TranslationCache, TranslationCacheManager};
pub use manager::{TranslationManager, TranslationStats};
pub use rule_localizer::{LocalizedRule, RuleLocalizer};
pub use translator::{RuleTranslator, TranslationProvider};

pub use crate::common::types::SupportedLanguage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;


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

impl TranslationConfig {
    /// Validate the translation configuration
    pub fn validate(&self) -> TranslationResult<()> {
        if !self.enabled {
            // If disabled, no need to validate further
            return Ok(());
        }

        // Validate provider
        if self.provider.trim().is_empty() {
            return Err(TranslationError::ConfigError(
                "Translation provider cannot be empty when translation is enabled".to_string(),
            ));
        }

        // Validate supported providers
        match self.provider.as_str() {
            "openai" | "azure" | "anthropic" | "local" => {
                // Valid providers
            }
            _ => {
                return Err(TranslationError::ConfigError(format!(
                    "Unsupported translation provider: {}. Supported providers: openai, azure, anthropic, local",
                    self.provider
                )));
            }
        }

        // Validate cache directory if specified
        if let Some(cache_dir) = &self.cache_dir {
            if cache_dir.as_os_str().is_empty() {
                return Err(TranslationError::ConfigError(
                    "Cache directory path cannot be empty".to_string(),
                ));
            }
        }

        // Provider-specific validation
        self.validate_provider_settings()?;

        Ok(())
    }

    /// Validate provider-specific settings
    fn validate_provider_settings(&self) -> TranslationResult<()> {
        match self.provider.as_str() {
            "openai" => {
                // OpenAI requires API key in settings or environment
                if !self.provider_settings.contains_key("api_key")
                    && std::env::var("OPENAI_API_KEY").is_err()
                {
                    return Err(TranslationError::ConfigError(
                        "OpenAI provider requires 'api_key' in provider_settings or OPENAI_API_KEY environment variable".to_string(),
                    ));
                }
            }
            "azure" => {
                // Azure requires endpoint and api_key
                let required_keys = ["endpoint", "api_key"];
                for key in &required_keys {
                    if !self.provider_settings.contains_key(*key) {
                        return Err(TranslationError::ConfigError(format!(
                            "Azure provider requires '{}' in provider_settings",
                            key
                        )));
                    }
                }
            }
            "anthropic" => {
                // Anthropic requires API key
                if !self.provider_settings.contains_key("api_key")
                    && std::env::var("ANTHROPIC_API_KEY").is_err()
                {
                    return Err(TranslationError::ConfigError(
                        "Anthropic provider requires 'api_key' in provider_settings or ANTHROPIC_API_KEY environment variable".to_string(),
                    ));
                }
            }
            "local" => {
                // Local provider might require endpoint
                if let Some(endpoint) = self.provider_settings.get("endpoint") {
                    if endpoint.trim().is_empty() {
                        return Err(TranslationError::ConfigError(
                            "Local provider endpoint cannot be empty".to_string(),
                        ));
                    }
                }
            }
            _ => {
                // Unknown provider - should have been caught in main validation
            }
        }

        Ok(())
    }

    /// Get a user-friendly configuration status description
    pub fn status_description(&self) -> String {
        if !self.enabled {
            "Translation disabled".to_string()
        } else {
            match self.validate() {
                Ok(()) => format!(
                    "Translation enabled - Provider: {}, Language: {:?}, Cache: {}",
                    self.provider,
                    self.default_language,
                    if self.cache_enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                ),
                Err(e) => format!("Translation configuration error: {}", e),
            }
        }
    }

    /// Check if the configuration is operational (enabled and valid)
    pub fn is_operational(&self) -> bool {
        self.enabled && self.validate().is_ok()
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

    #[test]
    fn test_translation_config_validation_disabled() {
        let config = TranslationConfig::default();
        assert!(config.validate().is_ok());
        assert!(!config.is_operational());
    }

    #[test]
    fn test_translation_config_validation_enabled_valid() {
        let mut config = TranslationConfig::default();
        config.enabled = true;
        config
            .provider_settings
            .insert("api_key".to_string(), "test-key".to_string());

        assert!(config.validate().is_ok());
        assert!(config.is_operational());
    }

    #[test]
    fn test_translation_config_validation_empty_provider() {
        let mut config = TranslationConfig::default();
        config.enabled = true;
        config.provider = "".to_string();

        assert!(config.validate().is_err());
        assert!(!config.is_operational());
    }

    #[test]
    fn test_translation_config_validation_unsupported_provider() {
        let mut config = TranslationConfig::default();
        config.enabled = true;
        config.provider = "invalid-provider".to_string();

        assert!(config.validate().is_err());
        assert!(!config.is_operational());
    }

    #[test]
    fn test_translation_config_validation_openai_missing_key() {
        let mut config = TranslationConfig::default();
        config.enabled = true;
        config.provider = "openai".to_string();
        // No API key provided

        // This test might pass if OPENAI_API_KEY env var is set
        // So we'll check that validation either passes or gives the expected error
        if let Err(e) = config.validate() {
            assert!(e.to_string().contains("api_key"));
        }
    }

    #[test]
    fn test_translation_config_status_description() {
        let mut config = TranslationConfig::default();
        assert_eq!(config.status_description(), "Translation disabled");

        config.enabled = true;
        config
            .provider_settings
            .insert("api_key".to_string(), "test-key".to_string());
        let status = config.status_description();
        assert!(status.contains("Translation enabled"));
        assert!(status.contains("openai"));
    }
}

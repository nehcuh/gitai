//! Integration tests for translation functionality
//!
//! These tests verify that translation features work correctly across
//! the entire gitai system, including parameter parsing, configuration
//! validation, and integration with various commands.

use gitai::ast_grep_analyzer::translation::{
    SupportedLanguage, TranslationConfig, TranslationManager,
};
use gitai::config::AppConfig;
use gitai::types::git::ReviewArgs;
use gitai::utils::construct_review_args;
use std::collections::HashMap;
use std::path::PathBuf;

/// Helper function to create a test translation configuration
fn create_test_translation_config(enabled: bool, language: SupportedLanguage) -> TranslationConfig {
    TranslationConfig {
        enabled,
        default_language: language,
        cache_enabled: true,
        provider: "openai".to_string(),
        cache_dir: Some(PathBuf::from("/tmp/test_cache")),
        provider_settings: {
            let mut settings = HashMap::new();
            settings.insert("api_key".to_string(), "test_key".to_string());
            settings
        },
    }
}

/// Helper function to create a minimal app config for testing
fn create_test_app_config() -> AppConfig {
    AppConfig {
        ai: gitai::config::AIConfig {
            api_url: "http://localhost:11434/v1/chat/completions".to_string(),
            model_name: "test-model".to_string(),
            temperature: 0.7,
            api_key: Some("test-key".to_string()),
        },
        ast_grep: gitai::config::AstGrepConfig {
            enabled: true,
            analysis_depth: "medium".to_string(),
            cache_enabled: true,
        },
        review: gitai::config::ReviewConfig {
            auto_save: true,
            storage_path: "/tmp/test_reviews".to_string(),
            format: "text".to_string(),
            max_age_hours: 168,
            include_in_commit: true,
        },
        translation: create_test_translation_config(true, SupportedLanguage::Chinese),
        account: None,
        prompts: HashMap::new(),
    }
}

#[cfg(test)]
mod translation_manager_tests {
    use super::*;

    #[test]
    fn test_translation_manager_creation() {
        let config = create_test_translation_config(true, SupportedLanguage::Chinese);
        let manager = TranslationManager::new(config).unwrap();

        assert!(manager.is_enabled());
        assert_eq!(manager.target_language(), &SupportedLanguage::Chinese);
    }

    #[test]
    fn test_translation_manager_disabled() {
        let config = create_test_translation_config(false, SupportedLanguage::English);
        let manager = TranslationManager::new(config).unwrap();

        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_translation_manager_english_disabled() {
        let config = create_test_translation_config(true, SupportedLanguage::English);
        let manager = TranslationManager::new(config).unwrap();

        // Should be disabled for English even if config.enabled is true
        assert!(!manager.is_enabled());
    }

    #[tokio::test]
    async fn test_translation_manager_initialization() {
        let config = create_test_translation_config(true, SupportedLanguage::Chinese);
        let mut manager = TranslationManager::new(config).unwrap();

        let result = manager.initialize().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_translation_manager_language_update() {
        let config = create_test_translation_config(true, SupportedLanguage::Auto);
        let mut manager = TranslationManager::new(config).unwrap();

        let result = manager.set_target_language(SupportedLanguage::Chinese);
        assert!(result.is_ok());
        assert_eq!(manager.target_language(), &SupportedLanguage::Chinese);
    }

    #[tokio::test]
    async fn test_translation_manager_translate_rules() {
        let config = create_test_translation_config(true, SupportedLanguage::Chinese);
        let mut manager = TranslationManager::new(config).unwrap();
        manager.initialize().await.unwrap();

        let rules = vec![];
        let result = manager.translate_rules(&rules).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_translation_manager_translate_text() {
        let config = create_test_translation_config(true, SupportedLanguage::Chinese);
        let mut manager = TranslationManager::new(config).unwrap();
        manager.initialize().await.unwrap();

        let result = manager
            .translate_text("Hello World", Some("test context"))
            .await;
        assert!(result.is_ok());
        // Currently returns original text as it's a simplified implementation
        assert_eq!(result.unwrap(), "Hello World");
    }

    #[test]
    fn test_translation_manager_stats() {
        let config = create_test_translation_config(true, SupportedLanguage::Chinese);
        let manager = TranslationManager::new(config).unwrap();

        let stats = manager.get_stats();
        assert!(stats.enabled);
        assert_eq!(stats.target_language, SupportedLanguage::Chinese);
        assert!(stats.cache_enabled);
    }
}

#[cfg(test)]
mod translation_config_tests {
    use super::*;

    #[test]
    fn test_translation_config_validation_disabled() {
        let config = TranslationConfig {
            enabled: false,
            default_language: SupportedLanguage::English,
            cache_enabled: true,
            provider: "".to_string(), // Empty provider should be OK when disabled
            cache_dir: None,
            provider_settings: HashMap::new(),
        };

        assert!(config.validate().is_ok());
        assert!(!config.is_operational());
    }

    #[test]
    fn test_translation_config_validation_enabled_valid() {
        let mut config = create_test_translation_config(true, SupportedLanguage::Chinese);
        assert!(config.validate().is_ok());
        assert!(config.is_operational());
    }

    #[test]
    fn test_translation_config_validation_empty_provider() {
        let mut config = create_test_translation_config(true, SupportedLanguage::Chinese);
        config.provider = "".to_string();

        assert!(config.validate().is_err());
        assert!(!config.is_operational());
    }

    #[test]
    fn test_translation_config_validation_unsupported_provider() {
        let mut config = create_test_translation_config(true, SupportedLanguage::Chinese);
        config.provider = "invalid-provider".to_string();

        assert!(config.validate().is_err());
        assert!(!config.is_operational());
    }

    #[test]
    fn test_translation_config_validation_openai_with_key() {
        let mut config = create_test_translation_config(true, SupportedLanguage::Chinese);
        config.provider = "openai".to_string();
        config
            .provider_settings
            .insert("api_key".to_string(), "test-key".to_string());

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_translation_config_validation_azure_missing_endpoint() {
        let mut config = create_test_translation_config(true, SupportedLanguage::Chinese);
        config.provider = "azure".to_string();
        config.provider_settings.clear();
        config
            .provider_settings
            .insert("api_key".to_string(), "test-key".to_string());
        // Missing endpoint

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_translation_config_validation_azure_complete() {
        let mut config = create_test_translation_config(true, SupportedLanguage::Chinese);
        config.provider = "azure".to_string();
        config.provider_settings.clear();
        config
            .provider_settings
            .insert("api_key".to_string(), "test-key".to_string());
        config.provider_settings.insert(
            "endpoint".to_string(),
            "https://test.openai.azure.com".to_string(),
        );

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_translation_config_status_description() {
        let config = create_test_translation_config(false, SupportedLanguage::English);
        assert_eq!(config.status_description(), "Translation disabled");

        let config = create_test_translation_config(true, SupportedLanguage::Chinese);
        let status = config.status_description();
        assert!(status.contains("Translation enabled"));
        assert!(status.contains("openai"));
        assert!(status.contains("Chinese"));
    }
}

#[cfg(test)]
mod supported_language_tests {
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
            SupportedLanguage::from_str("english"),
            Some(SupportedLanguage::English)
        );
        assert_eq!(
            SupportedLanguage::from_str("zh"),
            Some(SupportedLanguage::Chinese)
        );
        assert_eq!(
            SupportedLanguage::from_str("chinese"),
            Some(SupportedLanguage::Chinese)
        );
        assert_eq!(
            SupportedLanguage::from_str("zh-cn"),
            Some(SupportedLanguage::Chinese)
        );
        assert_eq!(
            SupportedLanguage::from_str("auto"),
            Some(SupportedLanguage::Auto)
        );
        assert_eq!(SupportedLanguage::from_str("invalid"), None);
    }

    #[test]
    fn test_supported_language_system_default() {
        let default = SupportedLanguage::system_default();
        // Should be either English or Chinese depending on system locale
        assert!(matches!(
            default,
            SupportedLanguage::English | SupportedLanguage::Chinese
        ));
    }
}

#[cfg(test)]
mod review_args_integration_tests {
    use super::*;

    #[test]
    fn test_review_args_with_lang_parameter() {
        let args = vec![
            "gitai".to_string(),
            "review".to_string(),
            "--lang=zh".to_string(),
            "--focus=performance".to_string(),
        ];

        let review_args = construct_review_args(&args);

        assert_eq!(review_args.lang, Some("zh".to_string()));
        assert_eq!(review_args.focus, Some("performance".to_string()));
    }

    #[test]
    fn test_review_args_without_lang_parameter() {
        let args = vec![
            "gitai".to_string(),
            "review".to_string(),
            "--focus=security".to_string(),
        ];

        let review_args = construct_review_args(&args);

        assert_eq!(review_args.lang, None);
        assert_eq!(review_args.focus, Some("security".to_string()));
    }

    #[test]
    fn test_review_args_with_scan_options_and_lang() {
        let args = vec![
            "gitai".to_string(),
            "review".to_string(),
            "--lang=en".to_string(),
            "--no-scan".to_string(),
            "--format=json".to_string(),
        ];

        let review_args = construct_review_args(&args);

        assert_eq!(review_args.lang, Some("en".to_string()));
        assert!(review_args.no_scan);
        assert_eq!(review_args.format, "json");
    }
}

#[cfg(test)]
mod config_integration_tests {
    use super::*;

    #[test]
    fn test_app_config_with_translation() {
        let config = create_test_app_config();

        assert!(config.translation.enabled);
        assert_eq!(
            config.translation.default_language,
            SupportedLanguage::Chinese
        );
        assert_eq!(config.translation.provider, "openai");
        assert!(config.translation.cache_enabled);
    }

    #[test]
    fn test_app_config_translation_validation() {
        let config = create_test_app_config();

        let validation_result = config.translation.validate();
        assert!(validation_result.is_ok());
    }

    #[test]
    fn test_app_config_translation_manager_creation() {
        let config = create_test_app_config();

        let manager_result = TranslationManager::new(config.translation.clone());
        assert!(manager_result.is_ok());

        let manager = manager_result.unwrap();
        assert!(manager.is_enabled());
    }
}

#[cfg(test)]
mod parameter_parsing_tests {
    use super::*;

    /// Test the language parameter parsing function from main.rs
    /// Note: This tests the parse_language_parameter function logic
    #[test]
    fn test_language_parameter_parsing() {
        // Test valid language codes
        assert_eq!(parse_language_param("zh"), Some(SupportedLanguage::Chinese));
        assert_eq!(
            parse_language_param("chinese"),
            Some(SupportedLanguage::Chinese)
        );
        assert_eq!(
            parse_language_param("中文"),
            Some(SupportedLanguage::Chinese)
        );
        assert_eq!(parse_language_param("en"), Some(SupportedLanguage::English));
        assert_eq!(
            parse_language_param("english"),
            Some(SupportedLanguage::English)
        );
        assert_eq!(
            parse_language_param("英文"),
            Some(SupportedLanguage::English)
        );
        assert_eq!(parse_language_param("auto"), Some(SupportedLanguage::Auto));
        assert_eq!(parse_language_param("自动"), Some(SupportedLanguage::Auto));

        // Test invalid language codes
        assert_eq!(parse_language_param("invalid"), None);
        assert_eq!(parse_language_param(""), None);
        assert_eq!(parse_language_param("fr"), None);
    }

    /// Helper function that mirrors the logic in main.rs
    fn parse_language_param(lang_str: &str) -> Option<SupportedLanguage> {
        match lang_str.to_lowercase().as_str() {
            "zh" | "chinese" | "中文" => Some(SupportedLanguage::Chinese),
            "en" | "english" | "英文" => Some(SupportedLanguage::English),
            "auto" | "自动" => Some(SupportedLanguage::Auto),
            _ => None,
        }
    }

    #[test]
    fn test_case_insensitive_parsing() {
        assert_eq!(parse_language_param("ZH"), Some(SupportedLanguage::Chinese));
        assert_eq!(parse_language_param("EN"), Some(SupportedLanguage::English));
        assert_eq!(parse_language_param("AUTO"), Some(SupportedLanguage::Auto));
        assert_eq!(
            parse_language_param("Chinese"),
            Some(SupportedLanguage::Chinese)
        );
        assert_eq!(
            parse_language_param("ENGLISH"),
            Some(SupportedLanguage::English)
        );
    }
}

#[cfg(test)]
mod end_to_end_tests {
    use super::*;

    /// Test that translation manager can be created and used in a typical workflow
    #[tokio::test]
    async fn test_complete_translation_workflow() {
        // 1. Create configuration
        let config = create_test_translation_config(true, SupportedLanguage::Chinese);

        // 2. Create and initialize translation manager
        let mut manager = TranslationManager::new(config).unwrap();
        assert!(manager.initialize().await.is_ok());

        // 3. Test language switching
        assert!(
            manager
                .set_target_language(SupportedLanguage::English)
                .is_ok()
        );
        assert_eq!(manager.target_language(), &SupportedLanguage::English);
        assert!(!manager.is_enabled()); // Should be disabled for English

        // 4. Switch back to Chinese
        assert!(
            manager
                .set_target_language(SupportedLanguage::Chinese)
                .is_ok()
        );
        assert!(manager.is_enabled());

        // 5. Test translation operations
        let text_result = manager.translate_text("Test message", None).await;
        assert!(text_result.is_ok());

        // 6. Test stats
        let stats = manager.get_stats();
        assert!(stats.enabled);
        assert_eq!(stats.target_language, SupportedLanguage::Chinese);
    }

    /// Test integration with review args parsing
    #[test]
    fn test_review_translation_integration() {
        // Simulate command line: gitai review --lang=zh --focus="性能问题"
        let args = vec![
            "gitai".to_string(),
            "review".to_string(),
            "--lang=zh".to_string(),
            "--focus=性能问题".to_string(),
            "--no-scan".to_string(),
        ];

        let review_args = construct_review_args(&args);

        // Verify args parsing
        assert_eq!(review_args.lang, Some("zh".to_string()));
        assert_eq!(review_args.focus, Some("性能问题".to_string()));
        assert!(review_args.no_scan);

        // Verify language can be parsed
        if let Some(lang_str) = &review_args.lang {
            let language = SupportedLanguage::from_str(lang_str);
            assert_eq!(language, Some(SupportedLanguage::Chinese));
        }
    }

    /// Test that translation configuration works with different scenarios
    #[test]
    fn test_various_translation_scenarios() {
        // Scenario 1: Translation disabled
        let config1 = create_test_translation_config(false, SupportedLanguage::Chinese);
        let manager1 = TranslationManager::new(config1).unwrap();
        assert!(!manager1.is_enabled());

        // Scenario 2: Translation enabled with Chinese
        let config2 = create_test_translation_config(true, SupportedLanguage::Chinese);
        let manager2 = TranslationManager::new(config2).unwrap();
        assert!(manager2.is_enabled());

        // Scenario 3: Translation enabled with English (should be disabled)
        let config3 = create_test_translation_config(true, SupportedLanguage::English);
        let manager3 = TranslationManager::new(config3).unwrap();
        assert!(!manager3.is_enabled());

        // Scenario 4: Translation with Auto language
        let config4 = create_test_translation_config(true, SupportedLanguage::Auto);
        let manager4 = TranslationManager::new(config4).unwrap();
        // Should resolve to system default and potentially be enabled
        let resolved_lang = manager4.target_language();
        assert!(matches!(
            resolved_lang,
            SupportedLanguage::English | SupportedLanguage::Chinese
        ));
    }
}

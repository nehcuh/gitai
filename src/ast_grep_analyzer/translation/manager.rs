//! Translation Manager
//!
//! Coordinates translation functionality across the AST-Grep analyzer system.
//! Handles language detection, rule localization, and result translation.

use crate::ast_grep_analyzer::core::{AnalysisRule, CodeIssue};
use crate::ast_grep_analyzer::translation::{
    SupportedLanguage, TranslationConfig, TranslationResult,
};
use tracing::{debug, info, warn};

/// Translation manager that coordinates all translation operations
#[derive(Debug)]
pub struct TranslationManager {
    /// Translation configuration
    config: TranslationConfig,
    /// Current target language
    target_language: SupportedLanguage,
    /// Whether the manager is initialized
    initialized: bool,
}

impl TranslationManager {
    /// Create a new translation manager
    pub fn new(config: TranslationConfig) -> TranslationResult<Self> {
        let target_language = Self::resolve_target_language(&config.default_language);

        info!(
            "Initializing translation manager with target language: {:?}",
            target_language
        );

        let manager = Self {
            config,
            target_language,
            initialized: false,
        };

        Ok(manager)
    }

    /// Initialize the translation manager with all components
    pub async fn initialize(&mut self) -> TranslationResult<()> {
        if !self.config.enabled {
            debug!("Translation is disabled, skipping initialization");
            self.initialized = true;
            return Ok(());
        }

        info!("Translation manager initialized (simplified implementation)");
        self.initialized = true;
        Ok(())
    }

    /// Check if translation is enabled and available
    pub fn is_enabled(&self) -> bool {
        self.config.enabled && self.target_language != SupportedLanguage::English
    }

    /// Get the current target language
    pub fn target_language(&self) -> &SupportedLanguage {
        &self.target_language
    }

    /// Update the target language
    pub fn set_target_language(&mut self, language: SupportedLanguage) -> TranslationResult<()> {
        let resolved_language = Self::resolve_target_language(&language);

        if resolved_language != self.target_language {
            info!(
                "Updating target language from {:?} to {:?}",
                self.target_language, resolved_language
            );
            self.target_language = resolved_language;
        }

        Ok(())
    }

    /// Translate a list of analysis rules
    pub async fn translate_rules(
        &mut self,
        rules: &[AnalysisRule],
    ) -> TranslationResult<Vec<AnalysisRule>> {
        if !self.is_enabled() {
            debug!("Translation disabled or target language is English, returning original rules");
            return Ok(rules.to_vec());
        }

        let mut translated_rules = Vec::with_capacity(rules.len());

        for rule in rules {
            match self.translate_single_rule(rule).await {
                Ok(translated_rule) => translated_rules.push(translated_rule),
                Err(e) => {
                    warn!("Failed to translate rule '{}': {}", rule.id, e);
                    // Fallback to original rule
                    translated_rules.push(rule.clone());
                }
            }
        }

        info!(
            "Translated {} rules to {:?}",
            translated_rules.len(),
            self.target_language
        );
        Ok(translated_rules)
    }

    /// Translate code issues (results from scanning)
    pub async fn translate_issues(
        &mut self,
        issues: &[CodeIssue],
    ) -> TranslationResult<Vec<CodeIssue>> {
        if !self.is_enabled() {
            debug!("Translation disabled or target language is English, returning original issues");
            return Ok(issues.to_vec());
        }

        let mut translated_issues = Vec::with_capacity(issues.len());

        for issue in issues {
            match self.translate_single_issue(issue).await {
                Ok(translated_issue) => translated_issues.push(translated_issue),
                Err(e) => {
                    warn!("Failed to translate issue: {}", e);
                    // Fallback to original issue
                    translated_issues.push(issue.clone());
                }
            }
        }

        info!(
            "Translated {} issues to {:?}",
            translated_issues.len(),
            self.target_language
        );
        Ok(translated_issues)
    }

    /// Translate a review text or analysis summary
    pub async fn translate_text(
        &mut self,
        text: &str,
        _context: Option<&str>,
    ) -> TranslationResult<String> {
        if !self.is_enabled() {
            debug!("Translation disabled or target language is English, returning original text");
            return Ok(text.to_string());
        }

        // Simplified implementation - just return original text for now
        // TODO: Implement actual translation logic when translator API is stable
        debug!("Translation requested but not implemented, returning original text");
        Ok(text.to_string())
    }

    /// Get translation statistics
    pub fn get_stats(&self) -> TranslationStats {
        TranslationStats {
            enabled: self.config.enabled,
            target_language: self.target_language.clone(),
            cache_enabled: self.config.cache_enabled,
            cache_stats: None,
            localizer_available: false,
            translator_available: false,
        }
    }

    /// Clean up expired cache entries
    pub async fn cleanup_cache(&mut self) -> TranslationResult<usize> {
        // Simplified implementation
        Ok(0)
    }

    // Private helper methods

    /// Resolve the actual target language from a potentially auto-detect setting
    fn resolve_target_language(language: &SupportedLanguage) -> SupportedLanguage {
        match language {
            SupportedLanguage::Auto => SupportedLanguage::system_default(),
            other => other.clone(),
        }
    }

    /// Translate a single analysis rule
    async fn translate_single_rule(
        &mut self,
        rule: &AnalysisRule,
    ) -> TranslationResult<AnalysisRule> {
        // Simplified implementation - just return the original rule
        // TODO: Implement actual rule translation when API is stable
        Ok(rule.clone())
    }

    /// Translate a single code issue
    async fn translate_single_issue(&mut self, issue: &CodeIssue) -> TranslationResult<CodeIssue> {
        // Simplified implementation - just return the original issue
        // TODO: Implement actual issue translation when API is stable
        Ok(issue.clone())
    }
}

/// Translation statistics
#[derive(Debug, Clone)]
pub struct TranslationStats {
    /// Whether translation is enabled
    pub enabled: bool,
    /// Current target language
    pub target_language: SupportedLanguage,
    /// Whether caching is enabled
    pub cache_enabled: bool,
    /// Cache statistics if available
    pub cache_stats: Option<()>,
    /// Whether rule localizer is available
    pub localizer_available: bool,
    /// Whether AI translator is available
    pub translator_available: bool,
}

impl TranslationStats {
    /// Check if translation is fully operational
    pub fn is_operational(&self) -> bool {
        self.enabled && (self.localizer_available || self.translator_available)
    }

    /// Get a human-readable status description
    pub fn status_description(&self) -> String {
        if !self.enabled {
            "Translation disabled".to_string()
        } else if self.target_language == SupportedLanguage::English {
            "Using English (no translation needed)".to_string()
        } else if self.is_operational() {
            format!("Active - translating to {:?}", self.target_language)
        } else {
            "Enabled but not operational (no translators available)".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> TranslationConfig {
        use std::collections::HashMap;
        TranslationConfig {
            enabled: true,
            default_language: SupportedLanguage::Chinese,
            cache_enabled: true,
            provider: "test".to_string(),
            cache_dir: None,
            provider_settings: HashMap::new(),
        }
    }

    #[test]
    fn test_translation_manager_creation() {
        let config = create_test_config();
        let manager = TranslationManager::new(config).unwrap();

        assert!(manager.is_enabled());
        assert_eq!(manager.target_language(), &SupportedLanguage::Chinese);
    }

    #[test]
    fn test_resolve_target_language() {
        assert_eq!(
            TranslationManager::resolve_target_language(&SupportedLanguage::Chinese),
            SupportedLanguage::Chinese
        );
        assert_eq!(
            TranslationManager::resolve_target_language(&SupportedLanguage::English),
            SupportedLanguage::English
        );
        // Auto should resolve to system default
        let auto_result = TranslationManager::resolve_target_language(&SupportedLanguage::Auto);
        assert!(matches!(
            auto_result,
            SupportedLanguage::English | SupportedLanguage::Chinese
        ));
    }

    #[test]
    fn test_disabled_translation() {
        let mut config = create_test_config();
        config.enabled = false;

        let manager = TranslationManager::new(config).unwrap();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_english_translation() {
        let mut config = create_test_config();
        config.default_language = SupportedLanguage::English;

        let manager = TranslationManager::new(config).unwrap();
        assert!(!manager.is_enabled()); // Should be disabled for English
    }

    #[test]
    fn test_translation_stats() {
        let config = create_test_config();
        let manager = TranslationManager::new(config).unwrap();

        let stats = manager.get_stats();
        assert!(stats.enabled);
        assert_eq!(stats.target_language, SupportedLanguage::Chinese);
        assert!(stats.cache_enabled);
    }
}

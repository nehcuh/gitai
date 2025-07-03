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
    /// Creates a new `TranslationManager` with the specified configuration.
    ///
    /// Resolves the target language from the configuration's default language and prepares the manager for initialization.
    ///
    /// # Returns
    ///
    /// A `TranslationManager` instance wrapped in a `TranslationResult`.
    ///
    /// # Examples
    ///
    /// ```
    /// let config = create_test_config();
    /// let manager = TranslationManager::new(config).unwrap();
    /// assert_eq!(manager.target_language(), &SupportedLanguage::Chinese);
    /// ```
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

    /// Initializes the translation manager and its components.
    ///
    /// If translation is disabled in the configuration, initialization is skipped but the manager is marked as initialized. Otherwise, marks the manager as initialized. This implementation does not perform any actual setup of translation components.
    #[allow(clippy::unused_async)]
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

    /// Returns true if translation is enabled in the configuration and the target language is not English.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled && self.target_language != SupportedLanguage::English
    }

    /// Returns a reference to the current target language used for translation.
    ///
    /// # Examples
    ///
    /// ```
    /// let manager = TranslationManager::new(config).unwrap();
    /// let lang = manager.target_language();
    /// assert_eq!(*lang, SupportedLanguage::Chinese);
    /// ```
    pub fn target_language(&self) -> &SupportedLanguage {
        &self.target_language
    }

    /// Sets the target language for translation operations.
    ///
    /// If the specified language is `Auto`, it resolves to the system default language. If the resolved language differs from the current target language, updates the manager's target language accordingly.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the target language was set successfully.
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

    /// Translates a list of analysis rules to the target language.
    ///
    /// If translation is disabled or the target language is English, returns the original rules unchanged. Otherwise, attempts to translate each rule; if translation of a rule fails, the original rule is used as a fallback.
    ///
    /// # Returns
    ///
    /// A vector of translated analysis rules, or the original rules if translation is not performed.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut manager = TranslationManager::new(config).unwrap();
    /// let rules = vec![AnalysisRule::default()];
    /// let translated = tokio_test::block_on(manager.translate_rules(&rules)).unwrap();
    /// assert_eq!(translated.len(), rules.len());
    /// ```
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

    /// Translates a list of code issues to the target language.
    ///
    /// If translation is disabled or the target language is English, returns the original issues.
    /// Otherwise, attempts to translate each issue; on failure, the original issue is retained in the result.
    ///
    /// # Returns
    ///
    /// A vector of code issues, translated to the target language when possible.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut manager = TranslationManager::new(config).unwrap();
    /// let issues = vec![CodeIssue::default()];
    /// let translated = manager.translate_issues(&issues).await.unwrap();
    /// assert_eq!(translated.len(), issues.len());
    /// ```
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

    /// Translates a review text or analysis summary to the target language.
    ///
    /// If translation is disabled or the target language is English, returns the original text.
    /// Currently, this method does not perform actual translation and always returns the input text.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut manager = TranslationManager::new(config).unwrap();
    /// let translated = tokio_test::block_on(manager.translate_text("Hello, world!", None)).unwrap();
    /// assert_eq!(translated, "Hello, world!");
    /// ```
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

    /// Returns the current translation status and configuration details.
    ///
    /// The returned `TranslationStats` includes whether translation is enabled, the target language,
    /// cache settings, and the availability of localizer and translator components. Some fields may be
    /// set to default values if not implemented.
    ///
    /// # Examples
    ///
    /// ```
    /// let manager = TranslationManager::new(config).unwrap();
    /// let stats = manager.get_stats();
    /// assert_eq!(stats.enabled, config.enabled);
    /// ```
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

    /// Removes expired entries from the translation cache.
    ///
    /// Returns the number of cache entries removed. The current implementation does not perform any cleanup and always returns zero.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut manager = TranslationManager::new(config).unwrap();
    /// let removed = futures::executor::block_on(manager.cleanup_cache()).unwrap();
    /// assert_eq!(removed, 0);
    /// ```
    pub async fn cleanup_cache(&mut self) -> TranslationResult<usize> {
        // Simplified implementation
        Ok(0)
    }

    // Private helper methods

    /// Resolves the effective target language, converting `Auto` to the system default language.
    ///
    /// If the input language is `Auto`, returns the system's default language; otherwise, returns the provided language unchanged.
    fn resolve_target_language(language: &SupportedLanguage) -> SupportedLanguage {
        match language {
            SupportedLanguage::Auto => SupportedLanguage::system_default(),
            other => other.clone(),
        }
    }

    /// Translates a single analysis rule to the target language.
    ///
    /// Currently returns the original rule without modification.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut manager = TranslationManager::new(create_test_config()).unwrap();
    /// let rule = AnalysisRule::default();
    /// let translated = futures::executor::block_on(manager.translate_single_rule(&rule)).unwrap();
    /// assert_eq!(translated, rule);
    /// ```
    async fn translate_single_rule(
        &mut self,
        rule: &AnalysisRule,
    ) -> TranslationResult<AnalysisRule> {
        // Simplified implementation - just return the original rule
        // TODO: Implement actual rule translation when API is stable
        Ok(rule.clone())
    }

    /// Returns a translated version of a single code issue.
    ///
    /// Currently, this method returns a clone of the original issue without modification.
    /// Intended for future integration with translation APIs.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut manager = TranslationManager::new(config).unwrap();
    /// let issue = CodeIssue::default();
    /// let translated = futures::executor::block_on(manager.translate_single_issue(&issue)).unwrap();
    /// assert_eq!(issue, translated);
    /// ```
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
    /// Returns true if translation is enabled and at least one translation component (localizer or translator) is available.
    pub fn is_operational(&self) -> bool {
        self.enabled && (self.localizer_available || self.translator_available)
    }

    /// Returns a human-readable string describing the current translation status.
    ///
    /// The description reflects whether translation is enabled, the target language,
    /// and the operational state of translation components.
    ///
    /// # Examples
    ///
    /// ```
    /// let stats = TranslationStats {
    ///     enabled: true,
    ///     target_language: SupportedLanguage::Chinese,
    ///     cache_enabled: false,
    ///     cache_stats: None,
    ///     localizer_available: false,
    ///     translator_available: false,
    /// };
    /// let desc = stats.status_description();
    /// assert!(desc.contains("Active") || desc.contains("not operational"));
    /// ```
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

    /// Creates a sample `TranslationConfig` for testing purposes.
    ///
    /// The returned configuration has translation enabled, Chinese as the default language,
    /// caching enabled, a test provider, and no provider-specific settings.
    ///
    /// # Examples
    ///
    /// ```
    /// let config = create_test_config();
    /// assert!(config.enabled);
    /// assert_eq!(config.default_language, SupportedLanguage::Chinese);
    /// ```
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

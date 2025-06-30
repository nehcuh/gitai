//! Rule localization processor for AST-Grep rules
//!
//! This module provides functionality to manage localized rule sets,
//! apply translations, and handle language switching for AST-Grep rules.

use super::{SupportedLanguage, TranslationError, TranslationResult};
use crate::ast_grep_analyzer::core::{AnalysisRule, IssueCategory, IssueSeverity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

/// A localized version of an analysis rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedRule {
    /// Original rule
    pub original_rule: AnalysisRule,
    /// Localized versions by language
    pub localizations: HashMap<String, AnalysisRule>,
    /// Default language for this rule
    pub default_language: SupportedLanguage,
    /// Whether this rule has been successfully localized
    pub is_localized: bool,
}

impl LocalizedRule {
    /// Create a new localized rule from an original rule
    pub fn new(original_rule: AnalysisRule) -> Self {
        Self {
            original_rule,
            localizations: HashMap::new(),
            default_language: SupportedLanguage::English,
            is_localized: false,
        }
    }

    /// Add a localized version of the rule
    pub fn add_localization(&mut self, language: &SupportedLanguage, localized_rule: AnalysisRule) {
        self.localizations
            .insert(language.code().to_string(), localized_rule);
        self.is_localized = true;
    }

    /// Get the rule in the specified language, fallback to original if not available
    pub fn get_rule_for_language(&self, language: &SupportedLanguage) -> &AnalysisRule {
        if let Some(localized) = self.localizations.get(language.code()) {
            localized
        } else {
            &self.original_rule
        }
    }

    /// Check if localization exists for a language
    pub fn has_localization_for(&self, language: &SupportedLanguage) -> bool {
        self.localizations.contains_key(language.code())
    }

    /// Get available languages for this rule
    pub fn available_languages(&self) -> Vec<SupportedLanguage> {
        let mut languages = vec![SupportedLanguage::English]; // Original is always English

        for lang_code in self.localizations.keys() {
            if let Some(lang) = SupportedLanguage::from_str(lang_code) {
                if lang != SupportedLanguage::English {
                    languages.push(lang);
                }
            }
        }

        languages
    }
}

/// Rule localizer manages localized rule sets
#[derive(Debug)]
pub struct RuleLocalizer {
    /// Map of rule ID to localized rule
    localized_rules: HashMap<String, LocalizedRule>,
    /// Current active language
    current_language: SupportedLanguage,
    /// Cache directory for localized rules
    cache_dir: Option<PathBuf>,
    /// Statistics
    stats: LocalizationStats,
}

impl RuleLocalizer {
    /// Create a new rule localizer
    pub fn new(language: SupportedLanguage, cache_dir: Option<PathBuf>) -> Self {
        Self {
            localized_rules: HashMap::new(),
            current_language: language,
            cache_dir,
            stats: LocalizationStats::default(),
        }
    }

    /// Add original rules to the localizer
    pub fn add_original_rules(&mut self, rules: Vec<AnalysisRule>) {
        for rule in rules {
            let rule_id = rule.id.clone();
            let localized_rule = LocalizedRule::new(rule);
            self.localized_rules.insert(rule_id, localized_rule);
        }

        self.stats.total_rules = self.localized_rules.len();
        info!(
            "Added {} original rules to localizer",
            self.stats.total_rules
        );
    }

    /// Add a localized version of a rule
    pub fn add_localized_rule(
        &mut self,
        rule_id: &str,
        language: &SupportedLanguage,
        localized_rule: AnalysisRule,
    ) -> TranslationResult<()> {
        if let Some(local_rule) = self.localized_rules.get_mut(rule_id) {
            local_rule.add_localization(language, localized_rule);
            self.stats.localized_rules += 1;
            debug!(
                "Added localization for rule {} in {}",
                rule_id,
                language.code()
            );
            Ok(())
        } else {
            Err(TranslationError::ConfigError(format!(
                "Original rule with ID {} not found",
                rule_id
            )))
        }
    }

    /// Get all rules in the current language
    pub fn get_rules_for_current_language(&self) -> Vec<AnalysisRule> {
        self.get_rules_for_language(&self.current_language)
    }

    /// Get all rules in the specified language
    pub fn get_rules_for_language(&self, language: &SupportedLanguage) -> Vec<AnalysisRule> {
        let effective_language = self.resolve_language(language);

        self.localized_rules
            .values()
            .map(|localized_rule| {
                localized_rule
                    .get_rule_for_language(&effective_language)
                    .clone()
            })
            .collect()
    }

    /// Get a specific rule by ID in the current language
    pub fn get_rule_by_id(&self, rule_id: &str) -> Option<AnalysisRule> {
        self.get_rule_by_id_for_language(rule_id, &self.current_language)
    }

    /// Get a specific rule by ID in the specified language
    pub fn get_rule_by_id_for_language(
        &self,
        rule_id: &str,
        language: &SupportedLanguage,
    ) -> Option<AnalysisRule> {
        let effective_language = self.resolve_language(language);

        self.localized_rules.get(rule_id).map(|localized_rule| {
            localized_rule
                .get_rule_for_language(&effective_language)
                .clone()
        })
    }

    /// Set the current active language
    pub fn set_current_language(&mut self, language: SupportedLanguage) {
        self.current_language = language;
        info!(
            "Changed current language to: {}",
            self.current_language.code()
        );
    }

    /// Get the current active language
    pub fn get_current_language(&self) -> &SupportedLanguage {
        &self.current_language
    }

    /// Get localization statistics
    pub fn get_stats(&self) -> &LocalizationStats {
        &self.stats
    }

    /// Check if a rule exists
    pub fn has_rule(&self, rule_id: &str) -> bool {
        self.localized_rules.contains_key(rule_id)
    }

    /// Get all rule IDs
    pub fn get_rule_ids(&self) -> Vec<String> {
        self.localized_rules.keys().cloned().collect()
    }

    /// Filter rules by category
    pub fn get_rules_by_category(&self, category: IssueCategory) -> Vec<AnalysisRule> {
        self.get_rules_for_current_language()
            .into_iter()
            .filter(|rule| rule.category == category)
            .collect()
    }

    /// Filter rules by severity
    pub fn get_rules_by_severity(&self, severity: IssueSeverity) -> Vec<AnalysisRule> {
        self.get_rules_for_current_language()
            .into_iter()
            .filter(|rule| rule.severity == severity)
            .collect()
    }

    /// Get rules that need localization for the specified language
    pub fn get_rules_needing_localization(
        &self,
        language: &SupportedLanguage,
    ) -> Vec<&AnalysisRule> {
        self.localized_rules
            .values()
            .filter(|localized_rule| !localized_rule.has_localization_for(language))
            .map(|localized_rule| &localized_rule.original_rule)
            .collect()
    }

    /// Update localization statistics
    pub fn update_stats(&mut self) {
        let mut localized_count = 0;
        let current_lang_code = self.current_language.code();

        for localized_rule in self.localized_rules.values() {
            if localized_rule.localizations.contains_key(current_lang_code) {
                localized_count += 1;
            }
        }

        self.stats.localized_rules = localized_count;
        self.stats.localization_coverage = if self.stats.total_rules > 0 {
            (localized_count as f64 / self.stats.total_rules as f64) * 100.0
        } else {
            0.0
        };
    }

    /// Clear all localizations for a specific language
    pub fn clear_localizations_for_language(&mut self, language: &SupportedLanguage) {
        let lang_code = language.code();
        let mut cleared_count = 0;

        for localized_rule in self.localized_rules.values_mut() {
            if localized_rule.localizations.remove(lang_code).is_some() {
                cleared_count += 1;
            }
        }

        self.update_stats();
        info!(
            "Cleared {} localizations for language: {}",
            cleared_count, lang_code
        );
    }

    /// Resolve the effective language (handle Auto language)
    fn resolve_language(&self, language: &SupportedLanguage) -> SupportedLanguage {
        match language {
            SupportedLanguage::Auto => SupportedLanguage::system_default(),
            _ => language.clone(),
        }
    }

    /// Export localized rules to file
    pub fn export_localized_rules(
        &self,
        language: &SupportedLanguage,
        output_path: &PathBuf,
    ) -> TranslationResult<()> {
        let rules = self.get_rules_for_language(language);
        let json_content = serde_json::to_string_pretty(&rules)?;

        std::fs::write(output_path, json_content).map_err(|e| TranslationError::IoError(e))?;

        info!(
            "Exported {} localized rules to: {}",
            rules.len(),
            output_path.display()
        );
        Ok(())
    }

    /// Import localized rules from file
    pub fn import_localized_rules(
        &mut self,
        language: &SupportedLanguage,
        input_path: &PathBuf,
    ) -> TranslationResult<usize> {
        let content =
            std::fs::read_to_string(input_path).map_err(|e| TranslationError::IoError(e))?;

        let rules: Vec<AnalysisRule> = serde_json::from_str(&content)?;
        let mut imported_count = 0;

        for rule in rules {
            if let Ok(()) = self.add_localized_rule(&rule.id.clone(), language, rule) {
                imported_count += 1;
            }
        }

        self.update_stats();
        info!(
            "Imported {} localized rules from: {}",
            imported_count,
            input_path.display()
        );
        Ok(imported_count)
    }
}

/// Localization statistics
#[derive(Debug, Clone, Default)]
pub struct LocalizationStats {
    /// Total number of rules
    pub total_rules: usize,
    /// Number of localized rules
    pub localized_rules: usize,
    /// Localization coverage percentage
    pub localization_coverage: f64,
}

impl LocalizationStats {
    /// Get rules that still need localization
    pub fn rules_needing_localization(&self) -> usize {
        self.total_rules.saturating_sub(self.localized_rules)
    }

    /// Check if localization is complete
    pub fn is_complete(&self) -> bool {
        self.localization_coverage >= 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rule(id: &str, message: &str) -> AnalysisRule {
        AnalysisRule {
            id: id.to_string(),
            name: format!("Test Rule {}", id),
            description: format!("Test rule description for {}", id),
            enabled: true,
            language: "javascript".to_string(),
            severity: IssueSeverity::Warning,
            category: IssueCategory::CodeQuality,
            pattern: "$VAR.test()".to_string(),
            message: message.to_string(),
            suggestion: Some("Test suggestion".to_string()),
        }
    }

    #[test]
    fn test_localized_rule_creation() {
        let original_rule = create_test_rule("test-1", "Original message");
        let localized_rule = LocalizedRule::new(original_rule.clone());

        assert_eq!(localized_rule.original_rule.id, original_rule.id);
        assert!(!localized_rule.is_localized);
        assert_eq!(
            localized_rule.available_languages(),
            vec![SupportedLanguage::English]
        );
    }

    #[test]
    fn test_add_localization() {
        let original_rule = create_test_rule("test-1", "Original message");
        let mut localized_rule = LocalizedRule::new(original_rule);

        let chinese_rule = create_test_rule("test-1", "中文消息");
        localized_rule.add_localization(&SupportedLanguage::Chinese, chinese_rule);

        assert!(localized_rule.is_localized);
        assert!(localized_rule.has_localization_for(&SupportedLanguage::Chinese));

        let rule_zh = localized_rule.get_rule_for_language(&SupportedLanguage::Chinese);
        assert_eq!(rule_zh.message, "中文消息");
    }

    #[test]
    fn test_rule_localizer() {
        let mut localizer = RuleLocalizer::new(SupportedLanguage::English, None);

        let rules = vec![
            create_test_rule("rule-1", "Message 1"),
            create_test_rule("rule-2", "Message 2"),
        ];

        localizer.add_original_rules(rules);
        assert_eq!(localizer.get_stats().total_rules, 2);

        // Add Chinese localization
        let chinese_rule = create_test_rule("rule-1", "中文消息1");
        let result =
            localizer.add_localized_rule("rule-1", &SupportedLanguage::Chinese, chinese_rule);
        assert!(result.is_ok());

        // Test rule retrieval
        localizer.set_current_language(SupportedLanguage::Chinese);
        let rules_zh = localizer.get_rules_for_current_language();
        assert_eq!(rules_zh.len(), 2);
        assert_eq!(rules_zh[0].message, "中文消息1"); // First rule is localized
        assert_eq!(rules_zh[1].message, "Message 2"); // Second rule fallback to English
    }

    #[test]
    fn test_localization_stats() {
        let mut localizer = RuleLocalizer::new(SupportedLanguage::Chinese, None);

        let rules = vec![
            create_test_rule("rule-1", "Message 1"),
            create_test_rule("rule-2", "Message 2"),
        ];

        localizer.add_original_rules(rules);
        localizer.update_stats();

        let stats = localizer.get_stats();
        assert_eq!(stats.total_rules, 2);
        assert_eq!(stats.localized_rules, 0);
        assert_eq!(stats.localization_coverage, 0.0);

        // Add one localization
        let chinese_rule = create_test_rule("rule-1", "中文消息1");
        let _ = localizer.add_localized_rule("rule-1", &SupportedLanguage::Chinese, chinese_rule);
        localizer.update_stats();

        let stats = localizer.get_stats();
        assert_eq!(stats.localized_rules, 1);
        assert_eq!(stats.localization_coverage, 50.0);
    }

    #[test]
    fn test_rules_needing_localization() {
        let mut localizer = RuleLocalizer::new(SupportedLanguage::Chinese, None);

        let rules = vec![
            create_test_rule("rule-1", "Message 1"),
            create_test_rule("rule-2", "Message 2"),
        ];

        localizer.add_original_rules(rules);

        let needing_localization =
            localizer.get_rules_needing_localization(&SupportedLanguage::Chinese);
        assert_eq!(needing_localization.len(), 2);

        // Add localization for one rule
        let chinese_rule = create_test_rule("rule-1", "中文消息1");
        let _ = localizer.add_localized_rule("rule-1", &SupportedLanguage::Chinese, chinese_rule);

        let needing_localization =
            localizer.get_rules_needing_localization(&SupportedLanguage::Chinese);
        assert_eq!(needing_localization.len(), 1);
        assert_eq!(needing_localization[0].id, "rule-2");
    }
}

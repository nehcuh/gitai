//! Rule localization processor for AST-Grep rules
//!
//! This module provides functionality to manage localized rule sets,
//! apply translations, and handle language switching for AST-Grep rules.

use super::{SupportedLanguage, TranslationError, TranslationResult};
use crate::ast_grep_analyzer::core::{AnalysisRule, IssueCategory, IssueSeverity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info};

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
    /// Creates a new `LocalizedRule` with the given original rule and no localizations.
    ///
    /// The resulting rule defaults to English and is not marked as localized.
    pub fn new(original_rule: AnalysisRule) -> Self {
        Self {
            original_rule,
            localizations: HashMap::new(),
            default_language: SupportedLanguage::English,
            is_localized: false,
        }
    }

    /// Adds a localized version of the rule for the specified language.
    ///
    /// Marks the rule as localized if at least one localization exists.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut localized_rule = LocalizedRule::new(original_rule);
    /// localized_rule.add_localization(&SupportedLanguage::French, french_rule);
    /// assert!(localized_rule.has_localization_for(&SupportedLanguage::French));
    /// ```
    pub fn add_localization(&mut self, language: &SupportedLanguage, localized_rule: AnalysisRule) {
        self.localizations
            .insert(language.code().to_string(), localized_rule);
        self.is_localized = true;
    }

    /// Returns the rule localized to the specified language, or the original rule if no localization exists.
    ///
    /// # Examples
    ///
    /// ```
    /// let rule = localized_rule.get_rule_for_language(&SupportedLanguage::French);
    /// // Returns the French version if available, otherwise returns the original English rule.
    /// ```
    pub fn get_rule_for_language(&self, language: &SupportedLanguage) -> &AnalysisRule {
        if let Some(localized) = self.localizations.get(language.code()) {
            localized
        } else {
            &self.original_rule
        }
    }

    /// Returns `true` if a localized rule exists for the specified language; otherwise returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut rule = LocalizedRule::new(original_rule);
    /// assert!(!rule.has_localization_for(&SupportedLanguage::French));
    /// rule.add_localization(SupportedLanguage::French, localized_rule);
    /// assert!(rule.has_localization_for(&SupportedLanguage::French));
    /// ```
    pub fn has_localization_for(&self, language: &SupportedLanguage) -> bool {
        self.localizations.contains_key(language.code())
    }

    /// Returns a list of languages for which this rule is available, always including English.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut rule = LocalizedRule::new(original_rule);
    /// rule.add_localization(SupportedLanguage::Japanese, localized_rule_jp);
    /// let langs = rule.available_languages();
    /// assert!(langs.contains(&SupportedLanguage::English));
    /// assert!(langs.contains(&SupportedLanguage::Japanese));
    /// ```
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
    /// Statistics
    stats: LocalizationStats,
}

impl RuleLocalizer {
    /// Creates a new `RuleLocalizer` with the specified active language.
    ///
    /// Initializes an empty set of localized rules and resets localization statistics.
    ///
    /// # Examples
    ///
    /// ```
    /// let localizer = RuleLocalizer::new(SupportedLanguage::English, None);
    /// assert_eq!(localizer.get_current_language(), SupportedLanguage::English);
    /// ```
    pub fn new(language: SupportedLanguage, _cache_dir: Option<PathBuf>) -> Self {
        Self {
            localized_rules: HashMap::new(),
            current_language: language,
            stats: LocalizationStats::default(),
        }
    }

    /// Adds a list of original (English) analysis rules to the localizer, replacing any existing rules and updating the total rule count.
    ///
    /// # Parameters
    /// - `rules`: A vector of original `AnalysisRule` instances to be managed by the localizer.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut localizer = RuleLocalizer::new(SupportedLanguage::English, None);
    /// localizer.add_original_rules(vec![rule1, rule2]);
    /// assert_eq!(localizer.get_stats().total_rules, 2);
    /// ```
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

    /// Adds a localized version of an existing rule by rule ID and language.
    ///
    /// Returns an error if the original rule with the specified ID does not exist.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the localization was added successfully.
    /// - `Err(TranslationError)` if the original rule is not found.
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

    /// Returns all analysis rules localized to the current language.
    ///
    /// # Examples
    ///
    /// ```
    /// let localizer = RuleLocalizer::new(SupportedLanguage::English, None);
    /// let rules = localizer.get_rules_for_current_language();
    /// assert!(rules.is_empty() || rules.iter().all(|r| r.language == SupportedLanguage::English));
    /// ```
    pub fn get_rules_for_current_language(&self) -> Vec<AnalysisRule> {
        self.get_rules_for_language(&self.current_language)
    }

    /// Returns all analysis rules localized to the specified language, falling back to the original rule if a localization is unavailable.
    ///
    /// # Parameters
    /// - `language`: The target language for which to retrieve rules. If set to `Auto`, the system default language is used.
    ///
    /// # Returns
    /// A vector of `AnalysisRule` instances in the specified language, with original rules provided where localizations are missing.
    ///
    /// # Examples
    ///
    /// ```
    /// let rules = rule_localizer.get_rules_for_language(&SupportedLanguage::French);
    /// assert!(!rules.is_empty());
    /// ```
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

    /// Retrieves a rule by its ID, returning the version localized to the current language if available.
    ///
    /// Returns `Some(AnalysisRule)` if the rule exists, otherwise `None`.
    pub fn get_rule_by_id(&self, rule_id: &str) -> Option<AnalysisRule> {
        self.get_rule_by_id_for_language(rule_id, &self.current_language)
    }

    /// Retrieves a rule by its ID, returning the localized version for the specified language if available, or the original rule otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let rule = localizer.get_rule_by_id_for_language("no-foo", &SupportedLanguage::French);
    /// assert!(rule.is_some());
    /// ```
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

    /// Sets the current active language for rule localization.
    ///
    /// Updates the language used for retrieving localized rules.
    pub fn set_current_language(&mut self, language: SupportedLanguage) {
        self.current_language = language;
        info!(
            "Changed current language to: {}",
            self.current_language.code()
        );
    }

    /// Returns the currently active language used for rule localization.
    ///
    /// # Examples
    ///
    /// ```
    /// let localizer = RuleLocalizer::new(SupportedLanguage::English, None);
    /// assert_eq!(localizer.get_current_language(), &SupportedLanguage::English);
    /// ```
    pub fn get_current_language(&self) -> &SupportedLanguage {
        &self.current_language
    }

    /// Returns statistics about rule localization coverage and progress.
    ///
    /// The returned `LocalizationStats` includes the total number of rules, the number of localized rules in the current language, and the localization coverage percentage.
    pub fn get_stats(&self) -> &LocalizationStats {
        &self.stats
    }

    /// Returns `true` if a rule with the specified ID exists in the localizer.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut localizer = RuleLocalizer::new(SupportedLanguage::English, None);
    /// // Assume "rule1" has been added previously
    /// assert!(localizer.has_rule("rule1"));
    /// assert!(!localizer.has_rule("nonexistent_rule"));
    /// ```
    pub fn has_rule(&self, rule_id: &str) -> bool {
        self.localized_rules.contains_key(rule_id)
    }

    /// Returns a vector of all rule IDs managed by the localizer.
    pub fn get_rule_ids(&self) -> Vec<String> {
        self.localized_rules.keys().cloned().collect()
    }

    /// Returns all analysis rules in the current language that match the specified issue category.
    ///
    /// # Examples
    ///
    /// ```
    /// let rules = localizer.get_rules_by_category(IssueCategory::Security);
    /// assert!(rules.iter().all(|r| r.category == IssueCategory::Security));
    /// ```
    pub fn get_rules_by_category(&self, category: IssueCategory) -> Vec<AnalysisRule> {
        self.get_rules_for_current_language()
            .into_iter()
            .filter(|rule| rule.category == category)
            .collect()
    }

    /// Returns all analysis rules with the specified severity in the current language.
    ///
    /// # Examples
    ///
    /// ```
    /// let rules = localizer.get_rules_by_severity(IssueSeverity::Warning);
    /// assert!(rules.iter().all(|r| r.severity == IssueSeverity::Warning));
    /// ```
    pub fn get_rules_by_severity(&self, severity: IssueSeverity) -> Vec<AnalysisRule> {
        self.get_rules_for_current_language()
            .into_iter()
            .filter(|rule| rule.severity == severity)
            .collect()
    }

    /// Returns a list of original rules that lack localization for the specified language.
    ///
    /// # Parameters
    ///
    /// - `language`: The language for which to check localization status.
    ///
    /// # Returns
    ///
    /// A vector of references to `AnalysisRule` instances that do not have a localized version for the given language.
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

    /// Updates the statistics for localized rules, including the count and coverage percentage for the current language.
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

    /// Removes all localized rule variants for the specified language and updates localization statistics.
    ///
    /// # Arguments
    ///
    /// * `language` - The language for which all localizations should be cleared.
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

    /// Resolves the effective language, converting `Auto` to the system default language.
    ///
    /// If the provided language is `SupportedLanguage::Auto`, returns the system default language; otherwise, returns the given language unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// let localizer = RuleLocalizer::new(SupportedLanguage::English, None);
    /// assert_eq!(localizer.resolve_language(&SupportedLanguage::Auto), SupportedLanguage::system_default());
    /// assert_eq!(localizer.resolve_language(&SupportedLanguage::French), SupportedLanguage::French);
    /// ```
    fn resolve_language(&self, language: &SupportedLanguage) -> SupportedLanguage {
        match language {
            SupportedLanguage::Auto => SupportedLanguage::system_default(),
            _ => language.clone(),
        }
    }

    /// Exports all localized rules for the specified language to a JSON file.
    ///
    /// Serializes the localized rules for the given language and writes them to the provided output path.
    /// Returns an error if serialization or file writing fails.
    ///
    /// # Arguments
    ///
    /// * `language` - The language for which to export localized rules.
    /// * `output_path` - The file path where the JSON output will be written.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the export succeeds, or a `TranslationError` if an error occurs.
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

    /// Imports localized analysis rules from a JSON file for a specified language.
    ///
    /// Reads a JSON file containing a list of `AnalysisRule` objects, adds each as a localized rule for the given language, updates localization statistics, and returns the number of successfully imported rules.
    ///
    /// # Returns
    /// The number of localized rules successfully imported.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or if deserialization fails.
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
    /// Returns the number of rules that have not yet been localized.
    ///
    /// This value is calculated as the total number of rules minus the number of localized rules, never returning a negative number.
    pub fn rules_needing_localization(&self) -> usize {
        self.total_rules.saturating_sub(self.localized_rules)
    }

    /// Returns true if all rules are localized (coverage is 100% or more).
    ///
    /// # Examples
    ///
    /// ```
    /// let stats = LocalizationStats { total_rules: 10, localized_rules: 10, localization_coverage: 100.0 };
    /// assert!(stats.is_complete());
    /// ```
    pub fn is_complete(&self) -> bool {
        self.localization_coverage >= 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a test `AnalysisRule` with the specified ID and message.
    ///
    /// The returned rule uses fixed values for all other fields, suitable for unit testing scenarios.
    ///
    /// # Parameters
    /// - `id`: The unique identifier for the rule.
    /// - `message`: The message to associate with the rule.
    ///
    /// # Returns
    /// An `AnalysisRule` instance populated with test data.
    ///
    /// # Examples
    ///
    /// ```
    /// let rule = create_test_rule("test_id", "This is a test message");
    /// assert_eq!(rule.id, "test_id");
    /// assert_eq!(rule.message, "This is a test message");
    /// ```
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

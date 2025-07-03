//! Translation cache management for AST-Grep rules
//!
//! This module provides functionality to cache translated rules to avoid
//! redundant translation requests and improve performance.

use super::{SupportedLanguage, TranslationError, TranslationResult};
use crate::ast_grep_analyzer::core::AnalysisRule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// Cache entry for a translated rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationCache {
    /// Original rule ID
    pub rule_id: String,
    /// Hash of the source rule content
    pub source_hash: String,
    /// Target language
    pub target_language: String,
    /// Translated rule
    pub translated_rule: AnalysisRule,
    /// Timestamp when the translation was created
    pub created_at: u64,
    /// Version of the source rule set
    pub source_version: String,
    /// Translation provider used
    pub provider: String,
}

impl TranslationCache {
    /// Create a new translation cache entry
    pub fn new(
        rule_id: String,
        source_hash: String,
        target_language: String,
        translated_rule: AnalysisRule,
        source_version: String,
        provider: String,
    ) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            rule_id,
            source_hash,
            target_language,
            translated_rule,
            created_at,
            source_version,
            provider,
        }
    }

    /// Check if the cache entry is expired
    pub fn is_expired(&self, max_age_hours: u32) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let max_age_seconds = max_age_hours as u64 * 3600;
        now.saturating_sub(self.created_at) > max_age_seconds
    }

    /// Get cache key for this entry
    pub fn cache_key(&self) -> String {
        format!(
            "{}:{}:{}",
            self.rule_id, self.source_hash, self.target_language
        )
    }
}

/// Translation cache manager
#[derive(Debug)]
pub struct TranslationCacheManager {
    /// Base directory for cache storage
    cache_dir: PathBuf,
    /// In-memory cache for fast access
    memory_cache: HashMap<String, TranslationCache>,
    /// Maximum cache entries in memory
    max_memory_entries: usize,
    /// Default cache expiration time in hours
    default_expiry_hours: u32,
}

impl TranslationCacheManager {
    /// Create a new cache manager
    pub fn new(cache_dir: Option<PathBuf>) -> TranslationResult<Self> {
        let cache_dir = cache_dir.unwrap_or_else(|| {
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("gitai")
                .join("translation_cache")
        });

        // Ensure cache directory exists
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir).map_err(|e| {
                TranslationError::CacheError(format!("Failed to create cache directory: {}", e))
            })?;
        }

        let mut manager = Self {
            cache_dir,
            memory_cache: HashMap::new(),
            max_memory_entries: 1000,     // Limit memory usage
            default_expiry_hours: 24 * 7, // 1 week default
        };

        // Load existing cache from disk
        manager.load_disk_cache()?;

        Ok(manager)
    }

    /// Get cached translation for a rule
    pub fn get_cached_translation(
        &self,
        rule_id: &str,
        source_hash: &str,
        target_language: &SupportedLanguage,
    ) -> Option<AnalysisRule> {
        let cache_key = format!("{}:{}:{}", rule_id, source_hash, target_language.code());

        if let Some(cache_entry) = self.memory_cache.get(&cache_key) {
            if !cache_entry.is_expired(self.default_expiry_hours) {
                debug!(
                    "Cache hit for rule: {} ({})",
                    rule_id,
                    target_language.code()
                );
                return Some(cache_entry.translated_rule.clone());
            } else {
                debug!(
                    "Cache entry expired for rule: {} ({})",
                    rule_id,
                    target_language.code()
                );
            }
        }

        None
    }

    /// Store a translation in cache
    pub fn store_translation(
        &mut self,
        rule_id: String,
        source_hash: String,
        target_language: &SupportedLanguage,
        translated_rule: AnalysisRule,
        source_version: String,
        provider: String,
    ) -> TranslationResult<()> {
        let cache_entry = TranslationCache::new(
            rule_id.clone(),
            source_hash,
            target_language.code().to_string(),
            translated_rule,
            source_version,
            provider,
        );

        let cache_key = cache_entry.cache_key();

        // Store in memory cache
        if self.memory_cache.len() >= self.max_memory_entries {
            // Remove oldest entries if memory cache is full
            self.evict_oldest_entries(self.max_memory_entries / 10);
        }

        self.memory_cache
            .insert(cache_key.clone(), cache_entry.clone());

        // Store to disk
        self.save_cache_entry_to_disk(&cache_entry)?;

        debug!(
            "Stored translation cache for rule: {} ({})",
            rule_id,
            target_language.code()
        );
        Ok(())
    }

    /// Check if a translation is cached and valid
    pub fn is_cached_and_valid(
        &self,
        rule_id: &str,
        source_hash: &str,
        target_language: &SupportedLanguage,
    ) -> bool {
        self.get_cached_translation(rule_id, source_hash, target_language)
            .is_some()
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        let total_entries = self.memory_cache.len();
        let expired_entries = self
            .memory_cache
            .values()
            .filter(|entry| entry.is_expired(self.default_expiry_hours))
            .count();

        CacheStats {
            total_entries,
            expired_entries,
            valid_entries: total_entries - expired_entries,
            cache_dir: self.cache_dir.clone(),
        }
    }

    /// Clean up expired cache entries
    pub fn cleanup_expired_cache(&mut self) -> TranslationResult<usize> {
        let mut removed_count = 0;

        // Clean memory cache
        let expired_keys: Vec<String> = self
            .memory_cache
            .iter()
            .filter(|(_, entry)| entry.is_expired(self.default_expiry_hours))
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.memory_cache.remove(&key);
            removed_count += 1;
        }

        // Clean disk cache
        removed_count += self.cleanup_disk_cache()?;

        if removed_count > 0 {
            info!(
                "Cleaned up {} expired translation cache entries",
                removed_count
            );
        }

        Ok(removed_count)
    }

    /// Get cache directory path
    pub fn get_cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Load cache from disk into memory
    fn load_disk_cache(&mut self) -> TranslationResult<()> {
        if !self.cache_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.cache_dir).map_err(|e| {
            TranslationError::CacheError(format!("Failed to read cache directory: {}", e))
        })?;

        let mut loaded_count = 0;
        for entry in entries {
            let entry = entry.map_err(|e| {
                TranslationError::CacheError(format!("Failed to read cache entry: {}", e))
            })?;

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_cache_entry_from_disk(&path) {
                    Ok(cache_entry) => {
                        let cache_key = cache_entry.cache_key();
                        self.memory_cache.insert(cache_key, cache_entry);
                        loaded_count += 1;
                    }
                    Err(e) => {
                        warn!("Failed to load cache entry from {}: {}", path.display(), e);
                        // Remove corrupted cache file
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }

        if loaded_count > 0 {
            debug!(
                "Loaded {} translation cache entries from disk",
                loaded_count
            );
        }

        Ok(())
    }

    /// Save a cache entry to disk
    fn save_cache_entry_to_disk(&self, cache_entry: &TranslationCache) -> TranslationResult<()> {
        let filename = format!("{}.json", cache_entry.cache_key().replace(':', "_"));
        let file_path = self.cache_dir.join(filename);

        let json_content = serde_json::to_string_pretty(cache_entry)?;
        fs::write(&file_path, json_content).map_err(|e| {
            TranslationError::CacheError(format!("Failed to write cache file: {}", e))
        })?;

        Ok(())
    }

    /// Load a cache entry from disk
    fn load_cache_entry_from_disk(&self, file_path: &Path) -> TranslationResult<TranslationCache> {
        let content = fs::read_to_string(file_path).map_err(|e| {
            TranslationError::CacheError(format!("Failed to read cache file: {}", e))
        })?;

        let cache_entry: TranslationCache = serde_json::from_str(&content)?;
        Ok(cache_entry)
    }

    /// Clean up expired entries from disk
    fn cleanup_disk_cache(&self) -> TranslationResult<usize> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let entries = fs::read_dir(&self.cache_dir).map_err(|e| {
            TranslationError::CacheError(format!("Failed to read cache directory: {}", e))
        })?;

        let mut removed_count = 0;
        for entry in entries {
            let entry = entry.map_err(|e| {
                TranslationError::CacheError(format!("Failed to read cache entry: {}", e))
            })?;

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_cache_entry_from_disk(&path) {
                    Ok(cache_entry) => {
                        if cache_entry.is_expired(self.default_expiry_hours) {
                            if fs::remove_file(&path).is_ok() {
                                removed_count += 1;
                            }
                        }
                    }
                    Err(_) => {
                        // Remove corrupted cache files
                        let _ = fs::remove_file(&path);
                        removed_count += 1;
                    }
                }
            }
        }

        Ok(removed_count)
    }

    /// Evict oldest entries from memory cache
    fn evict_oldest_entries(&mut self, count: usize) {
        let mut entries: Vec<(String, u64)> = self
            .memory_cache
            .iter()
            .map(|(key, entry)| (key.clone(), entry.created_at))
            .collect();

        entries.sort_by_key(|(_, created_at)| *created_at);

        for (key, _) in entries.into_iter().take(count) {
            self.memory_cache.remove(&key);
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub valid_entries: usize,
    pub cache_dir: PathBuf,
}

impl CacheStats {
    /// Get cache hit ratio as percentage
    pub fn hit_ratio(&self) -> f64 {
        if self.total_entries == 0 {
            0.0
        } else {
            (self.valid_entries as f64 / self.total_entries as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_rule() -> AnalysisRule {
        AnalysisRule {
            id: "test-rule".to_string(),
            name: "Test Rule".to_string(),
            description: "Test rule description".to_string(),
            enabled: true,
            language: "javascript".to_string(),
            severity: crate::ast_grep_analyzer::core::IssueSeverity::Warning,
            category: crate::ast_grep_analyzer::core::IssueCategory::CodeQuality,
            pattern: "$VAR.test()".to_string(),
            message: "Test message".to_string(),
            suggestion: Some("Test suggestion".to_string()),
        }
    }

    #[test]
    fn test_translation_cache_creation() {
        let rule = create_test_rule();
        let cache = TranslationCache::new(
            "test-rule".to_string(),
            "hash123".to_string(),
            "zh".to_string(),
            rule,
            "v1.0".to_string(),
            "openai".to_string(),
        );

        assert_eq!(cache.rule_id, "test-rule");
        assert_eq!(cache.source_hash, "hash123");
        assert_eq!(cache.target_language, "zh");
        assert!(!cache.is_expired(24)); // Should not be expired within 24 hours
    }

    #[test]
    fn test_cache_key_generation() {
        let rule = create_test_rule();
        let cache = TranslationCache::new(
            "test-rule".to_string(),
            "hash123".to_string(),
            "zh".to_string(),
            rule,
            "v1.0".to_string(),
            "openai".to_string(),
        );

        assert_eq!(cache.cache_key(), "test-rule:hash123:zh");
    }

    #[test]
    fn test_cache_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cache_manager = TranslationCacheManager::new(Some(temp_dir.path().to_path_buf()));

        assert!(cache_manager.is_ok());
        let manager = cache_manager.unwrap();
        assert_eq!(manager.get_cache_dir(), temp_dir.path());
    }

    #[test]
    fn test_cache_store_and_retrieve() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache_manager =
            TranslationCacheManager::new(Some(temp_dir.path().to_path_buf())).unwrap();

        let rule = create_test_rule();
        let language = SupportedLanguage::Chinese;

        // Store translation
        let result = cache_manager.store_translation(
            "test-rule".to_string(),
            "hash123".to_string(),
            &language,
            rule.clone(),
            "v1.0".to_string(),
            "openai".to_string(),
        );
        assert!(result.is_ok());

        // Retrieve translation
        let retrieved = cache_manager.get_cached_translation("test-rule", "hash123", &language);
        assert!(retrieved.is_some());

        let retrieved_rule = retrieved.unwrap();
        assert_eq!(retrieved_rule.id, rule.id);
        assert_eq!(retrieved_rule.message, rule.message);
    }

    #[test]
    fn test_cache_stats() {
        let temp_dir = TempDir::new().unwrap();
        let cache_manager =
            TranslationCacheManager::new(Some(temp_dir.path().to_path_buf())).unwrap();

        let stats = cache_manager.get_cache_stats();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.valid_entries, 0);
        assert_eq!(stats.hit_ratio(), 0.0);
    }
}

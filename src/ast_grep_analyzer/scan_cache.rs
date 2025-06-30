//! Scan result cache management for AST-Grep analysis
//!
//! This module provides functionality to cache scan results to avoid
//! redundant scanning and improve performance when integrating with review.

use crate::errors::AppError;
use crate::handlers::scan::{FileResult, ScanResults};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

/// Cache entry for scan results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanCacheEntry {
    /// File path being cached
    pub file_path: String,
    /// Hash of the file content when scanned
    pub file_hash: String,
    /// Scan results for this file
    pub scan_results: ScanResults,
    /// Timestamp when the scan was performed
    pub created_at: u64,
    /// File size when scanned
    pub file_size: u64,
    /// Last modified time of the file when scanned
    pub last_modified: u64,
}

impl ScanCacheEntry {
    /// Create a new scan cache entry
    pub fn new(
        file_path: String,
        file_hash: String,
        scan_results: ScanResults,
        file_size: u64,
        last_modified: u64,
    ) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            file_path,
            file_hash,
            scan_results,
            created_at,
            file_size,
            last_modified,
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
        format!("{}:{}", self.file_path.replace('/', "_"), self.file_hash)
    }

    /// Check if cache is valid for current file state
    pub fn is_valid_for_file(&self, file_path: &Path) -> Result<bool, AppError> {
        if !file_path.exists() {
            return Ok(false);
        }

        let metadata = file_path.metadata().map_err(|e| {
            AppError::IO(
                format!("Failed to get file metadata: {}", file_path.display()),
                e,
            )
        })?;

        let current_size = metadata.len();
        let current_modified = metadata
            .modified()
            .map_err(|e| AppError::IO("Failed to get file modified time".to_string(), e))?
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(self.file_size == current_size && self.last_modified == current_modified)
    }
}

/// Scan cache manager
#[derive(Debug)]
pub struct ScanCacheManager {
    /// Base directory for cache storage
    cache_dir: PathBuf,
    /// In-memory cache for fast access
    memory_cache: HashMap<String, ScanCacheEntry>,
    /// Maximum cache entries in memory
    max_memory_entries: usize,
    /// Default cache expiration time in hours
    default_expiry_hours: u32,
}

impl ScanCacheManager {
    /// Create a new scan cache manager
    pub fn new(cache_dir: Option<PathBuf>) -> Result<Self, AppError> {
        let cache_dir = cache_dir.unwrap_or_else(|| {
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("gitai")
                .join("scan_cache")
        });

        // Ensure cache directory exists
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir).map_err(|e| {
                AppError::IO(
                    format!(
                        "Failed to create scan cache directory: {}",
                        cache_dir.display()
                    ),
                    e,
                )
            })?;
        }

        let mut manager = Self {
            cache_dir,
            memory_cache: HashMap::new(),
            max_memory_entries: 500,  // Limit memory usage
            default_expiry_hours: 24, // 1 day default for scan cache
        };

        // Load existing cache from disk
        manager.load_disk_cache()?;

        Ok(manager)
    }

    /// Get cached scan results for a file
    pub fn get_cached_results(&self, file_path: &Path, file_hash: &str) -> Option<ScanResults> {
        let cache_key = format!(
            "{}:{}",
            file_path.to_string_lossy().replace('/', "_"),
            file_hash
        );

        if let Some(cache_entry) = self.memory_cache.get(&cache_key) {
            if !cache_entry.is_expired(self.default_expiry_hours) {
                // Additional validation: check if cache is still valid for current file state
                match cache_entry.is_valid_for_file(file_path) {
                    Ok(true) => {
                        debug!("Scan cache hit for file: {}", file_path.display());
                        return Some(cache_entry.scan_results.clone());
                    }
                    Ok(false) => {
                        debug!(
                            "Scan cache invalidated due to file changes: {}",
                            file_path.display()
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Failed to validate cache for {}: {}",
                            file_path.display(),
                            e
                        );
                    }
                }
            } else {
                debug!("Scan cache entry expired for file: {}", file_path.display());
            }
        }

        None
    }

    /// Store scan results in cache
    pub fn store_results(
        &mut self,
        file_path: &Path,
        file_hash: &str,
        results: &ScanResults,
    ) -> Result<(), AppError> {
        let metadata = file_path.metadata().map_err(|e| {
            AppError::IO(
                format!("Failed to get file metadata: {}", file_path.display()),
                e,
            )
        })?;

        let file_size = metadata.len();
        let last_modified = metadata
            .modified()
            .map_err(|e| AppError::IO("Failed to get file modified time".to_string(), e))?
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let cache_entry = ScanCacheEntry::new(
            file_path.to_string_lossy().to_string(),
            file_hash.to_string(),
            results.clone(),
            file_size,
            last_modified,
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

        debug!("Stored scan cache for file: {}", file_path.display());
        Ok(())
    }

    /// Check if scan results are cached and valid
    pub fn is_cache_valid(&self, file_path: &Path, file_hash: &str) -> bool {
        self.get_cached_results(file_path, file_hash).is_some()
    }

    /// Clean up expired cache entries
    pub fn cleanup_expired_cache(&mut self) -> Result<usize, AppError> {
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
            info!("Cleaned up {} expired scan cache entries", removed_count);
        }

        Ok(removed_count)
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> ScanCacheStats {
        let total_entries = self.memory_cache.len();
        let expired_entries = self
            .memory_cache
            .values()
            .filter(|entry| entry.is_expired(self.default_expiry_hours))
            .count();

        ScanCacheStats {
            total_entries,
            expired_entries,
            valid_entries: total_entries - expired_entries,
            cache_dir: self.cache_dir.clone(),
        }
    }

    /// Get cache directory path
    pub fn get_cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Calculate file hash for caching
    pub fn calculate_file_hash(&self, file_path: &Path) -> Result<String, AppError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let content = fs::read(file_path).map_err(|e| {
            AppError::IO(format!("Failed to read file: {}", file_path.display()), e)
        })?;

        let metadata = file_path.metadata().map_err(|e| {
            AppError::IO(
                format!("Failed to get file metadata: {}", file_path.display()),
                e,
            )
        })?;

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        metadata.len().hash(&mut hasher);
        metadata
            .modified()
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .hash(&mut hasher);

        Ok(format!("{:x}", hasher.finish()))
    }

    /// Load cache from disk into memory
    fn load_disk_cache(&mut self) -> Result<(), AppError> {
        if !self.cache_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.cache_dir).map_err(|e| {
            AppError::IO(
                format!(
                    "Failed to read scan cache directory: {}",
                    self.cache_dir.display()
                ),
                e,
            )
        })?;

        let mut loaded_count = 0;
        for entry in entries {
            let entry = entry
                .map_err(|e| AppError::IO("Failed to read scan cache entry".to_string(), e))?;

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_cache_entry_from_disk(&path) {
                    Ok(cache_entry) => {
                        let cache_key = cache_entry.cache_key();
                        self.memory_cache.insert(cache_key, cache_entry);
                        loaded_count += 1;
                    }
                    Err(e) => {
                        warn!(
                            "Failed to load scan cache entry from {}: {}",
                            path.display(),
                            e
                        );
                        // Remove corrupted cache file
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }

        if loaded_count > 0 {
            debug!("Loaded {} scan cache entries from disk", loaded_count);
        }

        Ok(())
    }

    /// Save a cache entry to disk
    fn save_cache_entry_to_disk(&self, cache_entry: &ScanCacheEntry) -> Result<(), AppError> {
        let filename = format!("{}.json", cache_entry.cache_key());
        let file_path = self.cache_dir.join(filename);

        let json_content = serde_json::to_string_pretty(cache_entry).map_err(|e| {
            AppError::Generic(format!("Failed to serialize scan cache entry: {}", e))
        })?;

        fs::write(&file_path, json_content).map_err(|e| {
            AppError::IO(
                format!("Failed to write scan cache file: {}", file_path.display()),
                e,
            )
        })?;

        Ok(())
    }

    /// Load a cache entry from disk
    fn load_cache_entry_from_disk(&self, file_path: &Path) -> Result<ScanCacheEntry, AppError> {
        let content = fs::read_to_string(file_path).map_err(|e| {
            AppError::IO(
                format!("Failed to read scan cache file: {}", file_path.display()),
                e,
            )
        })?;

        let cache_entry: ScanCacheEntry = serde_json::from_str(&content)
            .map_err(|e| AppError::Generic(format!("Failed to parse scan cache entry: {}", e)))?;

        Ok(cache_entry)
    }

    /// Clean up expired entries from disk
    fn cleanup_disk_cache(&self) -> Result<usize, AppError> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let entries = fs::read_dir(&self.cache_dir).map_err(|e| {
            AppError::IO(
                format!(
                    "Failed to read scan cache directory: {}",
                    self.cache_dir.display()
                ),
                e,
            )
        })?;

        let mut removed_count = 0;
        for entry in entries {
            let entry = entry
                .map_err(|e| AppError::IO("Failed to read scan cache entry".to_string(), e))?;

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

/// Scan cache statistics
#[derive(Debug, Clone)]
pub struct ScanCacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub valid_entries: usize,
    pub cache_dir: PathBuf,
}

impl ScanCacheStats {
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
    use crate::handlers::scan::ScanResults;
    use tempfile::TempDir;

    fn create_test_scan_results() -> ScanResults {
        ScanResults {
            files_scanned: 1,
            total_issues: 0,
            issues_by_severity: std::collections::HashMap::new(),
            issues_by_language: std::collections::HashMap::new(),
            issues_by_rule: std::collections::HashMap::new(),
            scan_duration_ms: 10,
            file_results: vec![],
            language_stats: Some(crate::ast_grep_analyzer::language_support::LanguageStats {
                total_languages: 1,
                enabled_languages: 1,
                supported_languages: 1,
                languages_by_category: std::collections::HashMap::new(),
            }),
            scan_config: crate::handlers::scan::ScanConfig {
                target: ".".to_string(),
                languages: vec![],
                rules: vec![],
                severity_levels: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                parallel: false,
                max_issues: 0,
            },
        }
    }

    #[test]
    fn test_scan_cache_entry_creation() {
        let results = create_test_scan_results();
        let cache_entry = ScanCacheEntry::new(
            "test.rs".to_string(),
            "hash123".to_string(),
            results,
            1024,
            1234567890,
        );

        assert_eq!(cache_entry.file_path, "test.rs");
        assert_eq!(cache_entry.file_hash, "hash123");
        assert!(!cache_entry.is_expired(24)); // Should not be expired within 24 hours
    }

    #[test]
    fn test_cache_key_generation() {
        let results = create_test_scan_results();
        let cache_entry = ScanCacheEntry::new(
            "src/main.rs".to_string(),
            "hash123".to_string(),
            results,
            1024,
            1234567890,
        );

        assert_eq!(cache_entry.cache_key(), "src_main.rs:hash123");
    }

    #[test]
    fn test_scan_cache_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cache_manager = ScanCacheManager::new(Some(temp_dir.path().to_path_buf()));

        assert!(cache_manager.is_ok());
        let manager = cache_manager.unwrap();
        assert_eq!(manager.get_cache_dir(), temp_dir.path());
    }

    #[test]
    fn test_cache_stats() {
        let temp_dir = TempDir::new().unwrap();
        let cache_manager = ScanCacheManager::new(Some(temp_dir.path().to_path_buf())).unwrap();

        let stats = cache_manager.get_cache_stats();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.valid_entries, 0);
        assert_eq!(stats.hit_ratio(), 0.0);
    }
}

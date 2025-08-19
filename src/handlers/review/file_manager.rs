use crate::{
    config::ReviewConfig,
    errors::AppError,
};
use std::{fs, io::Write, path::PathBuf};
use tracing;

use super::{output::OutputFormatter, types::SaveConfig};

/// Handles file operations for review results
pub struct FileManager {
    save_config: SaveConfig,
    output_formatter: OutputFormatter,
}

impl FileManager {
    /// Create a new file manager
    pub fn new(review_config: &ReviewConfig, output_formatter: OutputFormatter) -> Self {
        let save_config = SaveConfig {
            auto_save: review_config.is_auto_save_enabled(),
            format: review_config.get_format(),
            base_path: review_config.get_storage_path(),
        };

        Self {
            save_config,
            output_formatter,
        }
    }

    /// Save review results to file if auto-save is enabled
    pub async fn save_review_results(&self, content: &str) -> Result<Option<PathBuf>, AppError> {
        if !self.save_config.auto_save {
            return Ok(None);
        }

        let file_path = self.generate_file_path()?;
        let formatted_content = self.format_for_saving(content, &self.save_config.format);

        self.write_to_file(&file_path, &formatted_content).await?;
        
        tracing::info!("审查结果已保存到: {:?}", file_path);
        Ok(Some(file_path))
    }

    /// Save review results to a specific file path
    pub async fn save_to_path(&self, content: &str, file_path: &PathBuf, format: &str) -> Result<(), AppError> {
        let formatted_content = self.format_for_saving(content, format);
        self.write_to_file(file_path, &formatted_content).await?;
        tracing::info!("审查结果已保存到: {:?}", file_path);
        Ok(())
    }

    /// Format content for saving based on the specified format
    pub fn format_for_saving(&self, content: &str, format: &str) -> String {
        self.output_formatter.format_for_saving(content, format)
    }

    /// Generate file path for saving
    fn generate_file_path(&self) -> Result<PathBuf, AppError> {
        let expanded_path = self.expand_path(&self.save_config.base_path);
        let base_path = PathBuf::from(expanded_path);

        // Create directory if it doesn't exist
        if let Some(parent) = base_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Generate file name with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let file_extension = self.get_file_extension(&self.save_config.format);
        let file_name = format!("review_{}_{}.{}", timestamp, self.generate_hash(), file_extension);

        let file_path = base_path.join(file_name);
        Ok(file_path)
    }

    /// Write content to file
    async fn write_to_file(&self, file_path: &PathBuf, content: &str) -> Result<(), AppError> {
        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write content to file
        let mut file = fs::File::create(file_path)?;

        file.write_all(content.as_bytes())?;

        file.flush()?;

        Ok(())
    }

    /// Expand tilde and environment variables in path
    pub fn expand_path(&self, path: &str) -> String {
        if path.starts_with('~') {
            if let Some(home_dir) = dirs::home_dir() {
                path.replacen('~', &home_dir.to_string_lossy(), 1)
            } else {
                path.to_string()
            }
        } else {
            path.to_string()
        }
    }

    /// Get file extension based on format
    fn get_file_extension(&self, format: &str) -> &str {
        match format {
            "json" => "json",
            "markdown" | "md" => "md",
            "html" => "html",
            _ => "txt",
        }
    }

    /// Generate a short hash for uniqueness
    fn generate_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        std::process::id().hash(&mut hasher);
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0).hash(&mut hasher);
        
        format!("{:x}", hasher.finish()).chars().take(8).collect()
    }

    /// List existing review files in the storage directory
    pub fn list_review_files(&self) -> Result<Vec<PathBuf>, AppError> {
        let expanded_path = self.expand_path(&self.save_config.base_path);
        let base_path = PathBuf::from(expanded_path);

        if !base_path.exists() {
            return Ok(Vec::new());
        }

        let mut review_files = Vec::new();
        
        let entries = fs::read_dir(&base_path).map_err(|e| {
            AppError::IO(e)
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                AppError::IO(e)
            })?;

            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    if file_name.to_string_lossy().starts_with("review_") {
                        review_files.push(path);
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        review_files.sort_by(|a, b| {
            let a_modified = fs::metadata(a).and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let b_modified = fs::metadata(b).and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            b_modified.cmp(&a_modified)
        });

        Ok(review_files)
    }

    /// Clean up old review files based on age
    pub fn cleanup_old_files(&self, max_age_hours: u32) -> Result<usize, AppError> {
        let review_files = self.list_review_files()?;
        let mut removed_count = 0;
        let cutoff_time = std::time::SystemTime::now() - std::time::Duration::from_secs(max_age_hours as u64 * 3600);

        for file_path in review_files {
            if let Ok(metadata) = fs::metadata(&file_path) {
                if let Ok(modified_time) = metadata.modified() {
                    if modified_time < cutoff_time {
                        match fs::remove_file(&file_path) {
                            Ok(_) => {
                                tracing::debug!("已删除过期文件: {:?}", file_path);
                                removed_count += 1;
                            }
                            Err(e) => {
                                tracing::warn!("删除文件失败 {:?}: {}", file_path, e);
                            }
                        }
                    }
                }
            }
        }

        if removed_count > 0 {
            tracing::info!("清理了 {} 个过期的审查文件", removed_count);
        }

        Ok(removed_count)
    }

    /// Get storage statistics
    pub fn get_storage_stats(&self) -> Result<StorageStats, AppError> {
        let review_files = self.list_review_files()?;
        let mut total_size = 0u64;
        let mut oldest_file: Option<std::time::SystemTime> = None;
        let mut newest_file: Option<std::time::SystemTime> = None;

        for file_path in &review_files {
            if let Ok(metadata) = fs::metadata(file_path) {
                total_size += metadata.len();
                
                if let Ok(modified_time) = metadata.modified() {
                    if oldest_file.is_none() || Some(modified_time) < oldest_file {
                        oldest_file = Some(modified_time);
                    }
                    if newest_file.is_none() || Some(modified_time) > newest_file {
                        newest_file = Some(modified_time);
                    }
                }
            }
        }

        Ok(StorageStats {
            file_count: review_files.len(),
            total_size_bytes: total_size,
            oldest_file,
            newest_file,
        })
    }
}

/// Storage statistics for review files
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub file_count: usize,
    pub total_size_bytes: u64,
    pub oldest_file: Option<std::time::SystemTime>,
    pub newest_file: Option<std::time::SystemTime>,
}

impl StorageStats {
    /// Format storage size in human-readable format
    pub fn format_size(&self) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = self.total_size_bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::review::output::OutputFormatter;
    use tempfile::TempDir;

    fn create_test_config() -> ReviewConfig {
        ReviewConfig {
            auto_save: true,
            storage_path: "~/test_reviews".to_string(),
            format: "markdown".to_string(),
            max_age_hours: 168,
            include_in_commit: true,
        }
    }

    #[test]
    fn test_expand_path() {
        let config = create_test_config();
        let formatter = OutputFormatter::default();
        let manager = FileManager::new(&config, formatter);

        let expanded = manager.expand_path("~/test/path");
        assert!(!expanded.contains('~'));
    }

    #[test]
    fn test_get_file_extension() {
        let config = create_test_config();
        let formatter = OutputFormatter::default();
        let manager = FileManager::new(&config, formatter);

        assert_eq!(manager.get_file_extension("json"), "json");
        assert_eq!(manager.get_file_extension("markdown"), "md");
        assert_eq!(manager.get_file_extension("html"), "html");
        assert_eq!(manager.get_file_extension("plain"), "txt");
    }

    #[test]
    fn test_generate_hash() {
        let config = create_test_config();
        let formatter = OutputFormatter::default();
        let manager = FileManager::new(&config, formatter);

        let hash1 = manager.generate_hash();
        let hash2 = manager.generate_hash();

        assert_eq!(hash1.len(), 8);
        assert_eq!(hash2.len(), 8);
        // Hashes should be different due to timestamp difference
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_format_storage_size() {
        let stats = StorageStats {
            file_count: 5,
            total_size_bytes: 1536, // 1.5 KB
            oldest_file: None,
            newest_file: None,
        };

        let formatted = stats.format_size();
        assert!(formatted.contains("1.50 KB"));
    }
}
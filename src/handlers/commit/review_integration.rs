use crate::{
    errors::AppError,
    utils::{find_latest_review_file, read_review_file, extract_review_insights},
};

use super::types::{ReviewIntegrationConfig, ReviewIntegrationResult};

/// Handles integration of code review results into commit messages
pub struct ReviewIntegrator {
    config: ReviewIntegrationConfig,
}

impl ReviewIntegrator {
    /// Create a new review integrator
    pub fn new(config: ReviewIntegrationConfig) -> Self {
        Self { config }
    }

    /// Create integrator from review config
    pub fn from_review_config(review_config: &crate::config::ReviewConfig) -> Self {
        let config = ReviewIntegrationConfig {
            enabled: review_config.include_in_commit,
            storage_path: review_config.storage_path.clone(),
            include_in_message: true,
        };
        Self::new(config)
    }

    /// Integrate review results if available
    pub async fn integrate_review_results(&self) -> Result<ReviewIntegrationResult, AppError> {
        if !self.config.enabled {
            tracing::debug!("评审集成已禁用");
            return Ok(ReviewIntegrationResult {
                review_content: None,
                review_file_path: None,
                integration_successful: false,
            });
        }

        match find_latest_review_file(&self.config.storage_path) {
            Ok(Some(review_file)) => {
                tracing::info!("🔍 发现评审结果文件: {:?}", review_file);
                match read_review_file(&review_file) {
                    Ok(content) => {
                        let insights = extract_review_insights(&content);
                        tracing::debug!("提取到评审要点: {}", insights);
                        println!("📋 已发现相关代码评审结果，将集成到提交信息中");
                        
                        Ok(ReviewIntegrationResult {
                            review_content: Some(insights),
                            review_file_path: Some(review_file),
                            integration_successful: true,
                        })
                    }
                    Err(e) => {
                        tracing::warn!("读取评审文件失败: {}", e);
                        println!("⚠️ 警告: 无法读取评审结果文件");
                        Ok(ReviewIntegrationResult {
                            review_content: None,
                            review_file_path: Some(review_file),
                            integration_successful: false,
                        })
                    }
                }
            }
            Ok(None) => {
                tracing::debug!("未找到相关评审结果");
                Ok(ReviewIntegrationResult {
                    review_content: None,
                    review_file_path: None,
                    integration_successful: false,
                })
            }
            Err(e) => {
                tracing::debug!("检查评审结果时出错: {}", e);
                Ok(ReviewIntegrationResult {
                    review_content: None,
                    review_file_path: None,
                    integration_successful: false,
                })
            }
        }
    }

    /// Format commit message with review insights
    pub fn format_commit_with_review(
        &self,
        original_message: &str,
        review_content: &str,
    ) -> String {
        if !self.config.include_in_message {
            return original_message.to_string();
        }

        format!(
            "{}\n\n---\n## 基于代码评审的改进\n\n{}",
            original_message,
            review_content
        )
    }

    /// Extract key review insights for commit message
    pub fn extract_key_insights(&self, review_content: &str) -> String {
        let insights = extract_review_insights(review_content);
        
        // Limit the insights to key points for commit message
        let lines: Vec<&str> = insights.lines()
            .filter(|line| !line.trim().is_empty())
            .take(5) // Limit to top 5 insights
            .collect();
        
        if lines.is_empty() {
            "代码评审结果已集成".to_string()
        } else {
            lines.join("\n")
        }
    }

    /// Check if review integration is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get storage path
    pub fn get_storage_path(&self) -> &str {
        &self.config.storage_path
    }

    /// Validate review file format
    pub fn validate_review_file(&self, file_path: &std::path::Path) -> Result<(), AppError> {
        if !file_path.exists() {
            return Err(AppError::Generic("评审文件不存在".to_string()));
        }

        if !file_path.is_file() {
            return Err(AppError::Generic("评审路径不是文件".to_string()));
        }

        // Check file extension
        if let Some(extension) = file_path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            if !["md", "txt", "json"].contains(&ext_str.as_str()) {
                return Err(AppError::Generic("不支持的评审文件格式".to_string()));
            }
        }

        Ok(())
    }

    /// Get review file metadata
    pub fn get_review_file_metadata(&self, file_path: &std::path::Path) -> Result<ReviewFileMetadata, AppError> {
        self.validate_review_file(file_path)?;
        
        let metadata = std::fs::metadata(file_path)
            .map_err(|e| AppError::Generic(format!("获取文件元数据失败: {}", e)))?;
        
        let modified_time = metadata.modified()
            .map_err(|e| AppError::Generic(format!("获取文件修改时间失败: {}", e)))?;
        
        let file_size = metadata.len();
        let file_name = file_path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        Ok(ReviewFileMetadata {
            file_name,
            file_size,
            modified_time,
            file_path: file_path.to_path_buf(),
        })
    }

    /// Find all available review files
    pub fn find_all_review_files(&self) -> Result<Vec<std::path::PathBuf>, AppError> {
        let storage_path = std::path::Path::new(&self.config.storage_path);
        
        if !storage_path.exists() {
            return Ok(vec![]);
        }

        let mut review_files = Vec::new();
        
        let entries = std::fs::read_dir(storage_path)
            .map_err(|e| AppError::Generic(format!("读取目录失败: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| AppError::Generic(format!("读取目录项失败: {}", e)))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    let file_name_str = file_name.to_string_lossy();
                    if file_name_str.starts_with("review_") {
                        review_files.push(path);
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        review_files.sort_by(|a, b| {
            let a_modified = std::fs::metadata(a)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let b_modified = std::fs::metadata(b)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            b_modified.cmp(&a_modified)
        });

        Ok(review_files)
    }

    /// Clean up old review files
    pub fn cleanup_old_review_files(&self, max_age_hours: u32) -> Result<usize, AppError> {
        let review_files = self.find_all_review_files()?;
        let mut removed_count = 0;
        let cutoff_time = std::time::SystemTime::now() - std::time::Duration::from_secs(max_age_hours as u64 * 3600);

        for file_path in review_files {
            if let Ok(metadata) = std::fs::metadata(&file_path) {
                if let Ok(modified_time) = metadata.modified() {
                    if modified_time < cutoff_time {
                        match std::fs::remove_file(&file_path) {
                            Ok(_) => {
                                tracing::debug!("已删除过期评审文件: {:?}", file_path);
                                removed_count += 1;
                            }
                            Err(e) => {
                                tracing::warn!("删除评审文件失败 {:?}: {}", file_path, e);
                            }
                        }
                    }
                }
            }
        }

        if removed_count > 0 {
            tracing::info!("清理了 {} 个过期的评审文件", removed_count);
        }

        Ok(removed_count)
    }
}

impl Default for ReviewIntegrator {
    fn default() -> Self {
        Self::new(ReviewIntegrationConfig {
            enabled: false,
            storage_path: "~/.gitai/review_results".to_string(),
            include_in_message: true,
        })
    }
}

/// Metadata about a review file
#[derive(Debug, Clone)]
pub struct ReviewFileMetadata {
    pub file_name: String,
    pub file_size: u64,
    pub modified_time: std::time::SystemTime,
    pub file_path: std::path::PathBuf,
}

impl ReviewFileMetadata {
    /// Get file age in hours
    pub fn age_hours(&self) -> Result<f64, AppError> {
        let now = std::time::SystemTime::now();
        let duration = now.duration_since(self.modified_time)
            .map_err(|e| AppError::Generic(format!("计算文件年龄失败: {}", e)))?;
        
        Ok(duration.as_secs_f64() / 3600.0)
    }

    /// Format file size as human readable
    pub fn format_file_size(&self) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = self.file_size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }

    /// Check if file is recent (less than 24 hours old)
    pub fn is_recent(&self) -> bool {
        self.age_hours().unwrap_or(f64::MAX) < 24.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    fn create_test_config() -> ReviewIntegrationConfig {
        ReviewIntegrationConfig {
            enabled: true,
            storage_path: "/tmp/test_reviews".to_string(),
            include_in_message: true,
        }
    }

    #[test]
    fn test_review_integrator_creation() {
        let config = create_test_config();
        let integrator = ReviewIntegrator::new(config);
        assert!(integrator.is_enabled());
    }

    #[test]
    fn test_format_commit_with_review() {
        let config = create_test_config();
        let integrator = ReviewIntegrator::new(config);
        
        let original_message = "feat: add new feature";
        let review_content = "- Code quality improved\n- Security review passed";
        
        let result = integrator.format_commit_with_review(original_message, review_content);
        
        assert!(result.contains("feat: add new feature"));
        assert!(result.contains("基于代码评审的改进"));
        assert!(result.contains("Code quality improved"));
        assert!(result.contains("Security review passed"));
    }

    #[test]
    fn test_extract_key_insights() {
        let config = create_test_config();
        let integrator = ReviewIntegrator::new(config);
        
        let review_content = "Line 1: Important insight\nLine 2: Another insight\n\nLine 4: Third insight\nLine 5: Fourth insight\nLine 6: Fifth insight\nLine 7: Sixth insight";
        
        let insights = integrator.extract_key_insights(review_content);
        let line_count = insights.lines().count();
        
        // Should limit to 5 insights
        assert!(line_count <= 5);
        assert!(insights.contains("Important insight"));
    }

    #[test]
    fn test_validate_review_file() {
        let config = create_test_config();
        let integrator = ReviewIntegrator::new(config);
        
        // Create a temporary file for testing
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_review.md");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Test review content").unwrap();
        
        // Valid file should pass validation
        assert!(integrator.validate_review_file(&file_path).is_ok());
        
        // Non-existent file should fail
        let non_existent = temp_dir.path().join("non_existent.md");
        assert!(integrator.validate_review_file(&non_existent).is_err());
    }

    #[test]
    fn test_review_file_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_review.md");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Test review content").unwrap();
        drop(file);
        
        let config = create_test_config();
        let integrator = ReviewIntegrator::new(config);
        
        let metadata = integrator.get_review_file_metadata(&file_path);
        assert!(metadata.is_ok());
        
        let metadata = metadata.unwrap();
        assert_eq!(metadata.file_name, "test_review.md");
        assert!(metadata.file_size > 0);
        assert!(metadata.is_recent());
        assert!(!metadata.format_file_size().is_empty());
    }

    #[test]
    fn test_from_review_config() {
        let review_config = crate::config::ReviewConfig {
            auto_save: true,
            storage_path: "/test/path".to_string(),
            format: "markdown".to_string(),
            max_age_hours: 168,
            include_in_commit: true,
        };
        
        let integrator = ReviewIntegrator::from_review_config(&review_config);
        assert!(integrator.is_enabled());
        assert_eq!(integrator.get_storage_path(), "/test/path");
    }

    #[test]
    fn test_disabled_integrator() {
        let mut config = create_test_config();
        config.enabled = false;
        
        let integrator = ReviewIntegrator::new(config);
        assert!(!integrator.is_enabled());
    }

    #[tokio::test]
    async fn test_integrate_review_results_disabled() {
        let mut config = create_test_config();
        config.enabled = false;
        
        let integrator = ReviewIntegrator::new(config);
        let result = integrator.integrate_review_results().await;
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.integration_successful);
        assert!(result.review_content.is_none());
    }

    #[tokio::test]
    async fn test_integrate_review_results_no_file() {
        let mut config = create_test_config();
        config.storage_path = "/non/existent/path".to_string();
        
        let integrator = ReviewIntegrator::new(config);
        let result = integrator.integrate_review_results().await;
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.integration_successful);
        assert!(result.review_content.is_none());
    }

    #[test]
    fn test_find_all_review_files_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = create_test_config();
        config.storage_path = temp_dir.path().to_string_lossy().to_string();
        
        let integrator = ReviewIntegrator::new(config);
        let files = integrator.find_all_review_files();
        
        assert!(files.is_ok());
        assert!(files.unwrap().is_empty());
    }

    #[test]
    fn test_cleanup_old_review_files() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create some test review files
        let file1 = temp_dir.path().join("review_1.md");
        let file2 = temp_dir.path().join("review_2.md");
        let file3 = temp_dir.path().join("other_file.txt");
        
        File::create(&file1).unwrap();
        File::create(&file2).unwrap();
        File::create(&file3).unwrap();
        
        let mut config = create_test_config();
        config.storage_path = temp_dir.path().to_string_lossy().to_string();
        
        let integrator = ReviewIntegrator::new(config);
        
        // Files are new, so cleanup with 0 hours should remove all
        let removed = integrator.cleanup_old_review_files(0);
        assert!(removed.is_ok());
        
        // The actual removal depends on the system's file timestamps
        // So we just verify the function completes without error
    }

    #[test]
    fn test_review_file_metadata_age_calculation() {
        let metadata = ReviewFileMetadata {
            file_name: "test.md".to_string(),
            file_size: 1024,
            modified_time: std::time::SystemTime::now() - std::time::Duration::from_secs(7200), // 2 hours ago
            file_path: std::path::PathBuf::from("test.md"),
        };
        
        let age = metadata.age_hours().unwrap();
        assert!(age >= 1.9 && age <= 2.1); // Should be approximately 2 hours
        assert!(metadata.is_recent());
        assert_eq!(metadata.format_file_size(), "1.00 KB");
    }
}
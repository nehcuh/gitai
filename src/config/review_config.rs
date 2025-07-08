use serde::Deserialize;

/// Configuration for review functionality
#[derive(Deserialize, Debug, Clone)]
pub struct ReviewConfig {
    /// Whether to automatically save review results to local files
    #[serde(default = "default_auto_save")]
    pub auto_save: bool,

    /// Base path for storing review results (supports ~ expansion)
    #[serde(default = "default_storage_path")]
    pub storage_path: String,

    /// Default format for saved review files
    #[serde(default = "default_review_format")]
    pub format: String,

    /// Maximum age in hours to keep review results
    #[serde(default = "default_max_age_hours")]
    #[allow(dead_code)]
    pub max_age_hours: u32,

    /// Whether to include review results in commit message generation
    #[serde(default = "default_include_in_commit")]
    pub include_in_commit: bool,
}

/// Partial Review Configuration for loading from files
#[derive(Deserialize, Debug, Default)]
pub struct PartialReviewConfig {
    pub auto_save: Option<bool>,
    pub storage_path: Option<String>,
    pub format: Option<String>,
    #[serde(default)]
    pub max_age_hours: Option<u32>,
    #[serde(default)]
    pub include_in_commit: Option<bool>,
}

impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            auto_save: default_auto_save(),
            storage_path: default_storage_path(),
            format: default_review_format(),
            max_age_hours: default_max_age_hours(),
            include_in_commit: default_include_in_commit(),
        }
    }
}

impl ReviewConfig {
    /// Create ReviewConfig from partial config with defaults
    pub fn from_partial(partial: Option<PartialReviewConfig>) -> Self {
        let partial = partial.unwrap_or_default();
        
        Self {
            auto_save: partial.auto_save.unwrap_or_else(default_auto_save),
            storage_path: partial.storage_path.unwrap_or_else(default_storage_path),
            format: partial.format.unwrap_or_else(default_review_format),
            max_age_hours: partial.max_age_hours.unwrap_or_else(default_max_age_hours),
            include_in_commit: partial.include_in_commit.unwrap_or_else(default_include_in_commit),
        }
    }
}

// Default functions
fn default_auto_save() -> bool {
    true
}

fn default_storage_path() -> String {
    "~/.gitai/review_results".to_string()
}

fn default_review_format() -> String {
    "markdown".to_string()
}

fn default_max_age_hours() -> u32 {
    168 // 7 days
}

fn default_include_in_commit() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_review_config() {
        let config = ReviewConfig::default();
        assert!(config.auto_save);
        assert_eq!(config.storage_path, "~/.gitai/review_results");
        assert_eq!(config.format, "markdown");
        assert_eq!(config.max_age_hours, 168);
        assert!(config.include_in_commit);
    }

    #[test]
    fn test_from_partial_config() {
        let partial = PartialReviewConfig {
            auto_save: Some(false),
            storage_path: Some("/custom/path".to_string()),
            format: Some("json".to_string()),
            max_age_hours: Some(72),
            include_in_commit: None, // Should use default
        };

        let config = ReviewConfig::from_partial(Some(partial));
        assert!(!config.auto_save); // from partial
        assert_eq!(config.storage_path, "/custom/path"); // from partial
        assert_eq!(config.format, "json"); // from partial
        assert_eq!(config.max_age_hours, 72); // from partial
        assert!(config.include_in_commit); // default
    }

    #[test]
    fn test_from_none_partial() {
        let config = ReviewConfig::from_partial(None);
        assert!(config.auto_save);
        assert_eq!(config.storage_path, "~/.gitai/review_results");
        assert_eq!(config.format, "markdown");
        assert_eq!(config.max_age_hours, 168);
        assert!(config.include_in_commit);
    }
}
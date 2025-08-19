use serde::Deserialize;

/// 代码审查配置 - 简化版本，直接使用Option字段
#[derive(Deserialize, Debug, Clone, Default)]
pub struct ReviewConfig {
    /// 是否自动保存审查结果到本地文件
    pub auto_save: Option<bool>,

    /// 存储审查结果的基础路径（支持 ~ 展开）
    pub storage_path: Option<String>,

    /// 保存审查文件的默认格式
    pub format: Option<String>,

    /// 保留审查结果的最大小时数
    pub max_age_hours: Option<u32>,

    /// 是否在提交消息生成中包含审查结果
    pub include_in_commit: Option<bool>,
}

impl ReviewConfig {
    /// 解析配置，应用默认值
    pub fn resolve(self) -> Self {
        Self {
            auto_save: Some(self.auto_save.unwrap_or_else(default_auto_save)),
            storage_path: Some(self.storage_path.unwrap_or_else(default_storage_path)),
            format: Some(self.format.unwrap_or_else(default_review_format)),
            max_age_hours: Some(self.max_age_hours.unwrap_or_else(default_max_age_hours)),
            include_in_commit: Some(self.include_in_commit.unwrap_or_else(default_include_in_commit)),
        }
    }

    /// 获取自动保存状态
    pub fn is_auto_save_enabled(&self) -> bool {
        self.auto_save.unwrap_or_else(default_auto_save)
    }

    /// 获取存储路径
    pub fn get_storage_path(&self) -> String {
        self.storage_path.as_ref()
            .unwrap_or(&default_storage_path())
            .clone()
    }

    /// 获取格式
    pub fn get_format(&self) -> String {
        self.format.as_ref()
            .unwrap_or(&default_review_format())
            .clone()
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
        assert!(config.is_auto_save_enabled());
        assert_eq!(config.get_storage_path(), "~/.gitai/review_results");
        assert_eq!(config.get_format(), "markdown");
        assert_eq!(config.max_age_hours.unwrap(), 168);
        assert!(config.include_in_commit.unwrap());
    }

    #[test]
    fn test_resolve_config() {
        let config = ReviewConfig {
            auto_save: Some(false),
            storage_path: Some("/custom/path".to_string()),
            format: Some("json".to_string()),
            max_age_hours: Some(72),
            include_in_commit: None, // Should use default
        };

        let resolved = config.resolve();
        assert!(!resolved.auto_save.unwrap()); // from config
        assert_eq!(resolved.storage_path.unwrap(), "/custom/path"); // from config
        assert_eq!(resolved.format.unwrap(), "json"); // from config
        assert_eq!(resolved.max_age_hours.unwrap(), 72); // from config
        assert!(resolved.include_in_commit.unwrap()); // default
    }

    #[test]
    fn test_getter_methods() {
        let config = ReviewConfig {
            auto_save: Some(false),
            storage_path: Some("/custom/path".to_string()),
            format: Some("json".to_string()),
            max_age_hours: Some(72),
            include_in_commit: None,
        };

        assert!(!config.is_auto_save_enabled()); // from config
        assert_eq!(config.get_storage_path(), "/custom/path"); // from config
        assert_eq!(config.get_format(), "json"); // from config
        assert_eq!(config.max_age_hours.unwrap(), 72); // from config
        assert!(config.include_in_commit.unwrap()); // default
    }
}
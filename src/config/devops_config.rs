use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::errors::{AppError, config_error};

/// DevOps 账户配置 - 简化版本
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Default)]
pub struct AccountConfig {
    pub devops_platform: Option<String>,
    pub base_url: Option<String>,
    pub token: Option<String>,
    pub timeout: Option<u64>,
    pub retry_count: Option<u32>,
}

/// 解析后的DevOps配置，所有必需字段都有值
#[derive(Debug, Clone)]
pub struct ResolvedAccountConfig {
    pub devops_platform: String,
    pub base_url: String,
    pub token: String,
    pub timeout: u64,
    pub retry_count: u32,
}

impl AccountConfig {
    /// 简化的配置合并逻辑
    pub fn resolve(self) -> Option<ResolvedAccountConfig> {
        // 检查是否有任何配置
        let has_config = self.devops_platform.is_some() 
            || self.base_url.is_some() 
            || self.token.is_some();
        
        if !has_config {
            return None;
        }

        // 验证必需字段
        let platform = self.devops_platform?;
        let base_url = self.base_url?;
        let token = self.token?;

        if platform.is_empty() || base_url.is_empty() || token.is_empty() {
            return None;
        }

        Some(ResolvedAccountConfig {
            devops_platform: platform,
            base_url,
            token,
            timeout: self.timeout.unwrap_or(30000),
            retry_count: self.retry_count.unwrap_or(3),
        })
    }

    /// 从环境变量合并配置
    pub fn merge_with_env(mut self, env_map: &HashMap<String, String>) -> Self {
        if self.devops_platform.is_none() {
            self.devops_platform = env_map.get("GITAI_DEVOPS_PLATFORM").cloned();
        }
        if self.base_url.is_none() {
            self.base_url = env_map.get("GITAI_DEVOPS_BASE_URL").cloned();
        }
        if self.token.is_none() {
            self.token = env_map.get("GITAI_DEVOPS_TOKEN").cloned();
        }
        self
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), AppError> {
        if let Some(ref token) = self.token {
            if token.is_empty() {
                return Err(config_error("Token 不能为空"));
            }
        }
        
        if let Some(ref url) = self.base_url {
            if url.is_empty() {
                return Err(config_error("Base URL 不能为空"));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_config_when_all_empty() {
        let config = AccountConfig::default();
        assert!(config.resolve().is_none());
    }

    #[test]
    fn test_complete_config() {
        let config = AccountConfig {
            devops_platform: Some("coding".to_string()),
            base_url: Some("https://test.coding.net".to_string()),
            token: Some("test-token".to_string()),
            timeout: Some(60000),
            retry_count: Some(5),
        };

        let resolved = config.resolve().unwrap();
        assert_eq!(resolved.devops_platform, "coding");
        assert_eq!(resolved.base_url, "https://test.coding.net");
        assert_eq!(resolved.token, "test-token");
        assert_eq!(resolved.timeout, 60000);
        assert_eq!(resolved.retry_count, 5);
    }

    #[test]
    fn test_missing_required_field() {
        let config = AccountConfig {
            devops_platform: Some("coding".to_string()),
            base_url: None, // Missing
            token: Some("test-token".to_string()),
            timeout: Some(60000),
            retry_count: Some(5),
        };

        assert!(config.resolve().is_none());
    }

    #[test]
    fn test_merge_with_env() {
        let config = AccountConfig {
            devops_platform: Some("file-platform".to_string()),
            base_url: Some("file-url".to_string()),
            token: Some("file-token".to_string()),
            timeout: Some(60000),
            retry_count: Some(5),
        };

        let mut env_map = HashMap::new();
        env_map.insert("GITAI_DEVOPS_PLATFORM".to_string(), "env-platform".to_string());
        env_map.insert("GITAI_DEVOPS_TOKEN".to_string(), "env-token".to_string());

        let merged = config.merge_with_env(&env_map);
        let resolved = merged.resolve().unwrap();
        
        assert_eq!(resolved.devops_platform, "env-platform"); // env override
        assert_eq!(resolved.base_url, "file-url"); // from file
        assert_eq!(resolved.token, "env-token"); // env override
        assert_eq!(resolved.timeout, 60000); // from file
        assert_eq!(resolved.retry_count, 5); // from file
    }

    #[test]
    fn test_validation() {
        // 有效配置
        let config = AccountConfig {
            devops_platform: Some("coding".to_string()),
            base_url: Some("https://test.coding.net".to_string()),
            token: Some("test-token".to_string()),
            timeout: None,
            retry_count: None,
        };
        assert!(config.validate().is_ok());

        // 无效配置 - 空token
        let config = AccountConfig {
            devops_platform: Some("coding".to_string()),
            base_url: Some("https://test.coding.net".to_string()),
            token: Some("".to_string()),
            timeout: None,
            retry_count: None,
        };
        assert!(config.validate().is_err());
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::errors::ConfigError;

/// Account Configuration for DevOps platforms
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct AccountConfig {
    pub devops_platform: String,
    pub base_url: String,
    pub token: String,
    pub timeout: Option<u64>,
    pub retry_count: Option<u32>,
}

/// Partial Account Configuration for loading from files
#[derive(Deserialize, Debug, Default)]
pub struct PartialAccountConfig {
    pub devops_platform: Option<String>,
    pub base_url: Option<String>,
    pub token: Option<String>,
    pub timeout: Option<u64>,
    pub retry_count: Option<u32>,
}

impl Default for AccountConfig {
    fn default() -> Self {
        Self {
            devops_platform: String::new(),
            base_url: String::new(),
            token: String::new(),
            timeout: Some(30000), // Default timeout 30 seconds
            retry_count: Some(3), // Default retry count 3
        }
    }
}

impl AccountConfig {
    /// Create AccountConfig from environment variables and file config
    /// Returns None if no DevOps configuration is intended
    pub fn from_env_or_file(
        file_account_config: Option<PartialAccountConfig>,
        env_map: &HashMap<String, String>,
    ) -> Result<Option<Self>, ConfigError> {
        let devops_platform_env = env_map.get("GITAI_DEVOPS_PLATFORM");
        let base_url_env = env_map.get("GITAI_DEVOPS_BASE_URL");
        let token_env = env_map.get("GITAI_DEVOPS_TOKEN");

        // Extract values from file_account_config if it exists
        let file_platform = file_account_config
            .as_ref()
            .and_then(|c| c.devops_platform.clone());
        let file_base_url = file_account_config
            .as_ref()
            .and_then(|c| c.base_url.clone());
        let file_token = file_account_config
            .as_ref()
            .and_then(|c| c.token.clone());
        let file_timeout = file_account_config
            .as_ref()
            .and_then(|c| c.timeout);
        let file_retry_count = file_account_config
            .as_ref()
            .and_then(|c| c.retry_count);

        // Determine final values, giving priority to environment variables
        let devops_platform = devops_platform_env.map(|s| s.to_string()).or(file_platform);
        let base_url = base_url_env.map(|s| s.to_string()).or(file_base_url);
        let token = token_env.map(|s| s.to_string()).or(file_token);

        // Optional fields come from file config
        let timeout = file_timeout;
        let retry_count = file_retry_count;

        // Check if any of the core fields were specified at all (either env or file)
        if devops_platform.is_none() && base_url.is_none() && token.is_none() {
            // If no core fields are found from any source, it means no account config is intended.
            return Ok(None);
        }

        // Check if core fields have meaningful values (not None and not empty)
        let platform_is_meaningful = devops_platform.as_ref().is_some_and(|s| !s.is_empty());
        let url_is_meaningful = base_url.as_ref().is_some_and(|s| !s.is_empty());
        let token_is_meaningful = token.as_ref().map_or(false, |s| !s.is_empty());

        if !platform_is_meaningful && !url_is_meaningful && !token_is_meaningful {
            // If no core fields have non-empty values from any source,
            // it means no account config is intended, even if optional fields like timeout/retry exist.
            return Ok(None);
        }

        // If some core fields are present, then all three (platform, url, token) must be resolvable.
        let final_devops_platform = devops_platform.ok_or_else(|| {
            ConfigError::DevOpsConfigMissing(
                "account.devops_platform (and GITAI_DEVOPS_PLATFORM not set)".to_string(),
            )
        })?;
        let final_base_url = base_url.ok_or_else(|| {
            ConfigError::DevOpsConfigMissing(
                "account.base_url (and GITAI_DEVOPS_BASE_URL not set)".to_string(),
            )
        })?;
        let final_token = token.ok_or_else(|| {
            ConfigError::DevOpsConfigMissing(
                "account.token (and GITAI_DEVOPS_TOKEN not set)".to_string(),
            )
        })?;

        // Now check for empty values
        if final_devops_platform.is_empty() {
            return Err(ConfigError::DevOpsConfigMissing(
                "account.devops_platform cannot be empty".to_string(),
            ));
        }
        if final_base_url.is_empty() {
            return Err(ConfigError::DevOpsConfigMissing(
                "account.base_url cannot be empty".to_string(),
            ));
        }
        if final_token.is_empty() {
            return Err(ConfigError::DevOpsConfigMissing(
                "account.token cannot be empty".to_string(),
            ));
        }

        Ok(Some(AccountConfig {
            devops_platform: final_devops_platform,
            base_url: final_base_url,
            token: final_token,
            timeout,
            retry_count,
        }))
    }

    /// Validate the account configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.token.is_empty() {
            return Err(ConfigError::EmptyToken);
        }
        
        if self.base_url.is_empty() {
            return Err(ConfigError::InvalidUrl("Base URL is empty".to_string()));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_config_when_all_empty() {
        let result = AccountConfig::from_env_or_file(None, &HashMap::new()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_complete_env_config() {
        let mut env_map = HashMap::new();
        env_map.insert("GITAI_DEVOPS_PLATFORM".to_string(), "coding".to_string());
        env_map.insert("GITAI_DEVOPS_BASE_URL".to_string(), "https://test.coding.net".to_string());
        env_map.insert("GITAI_DEVOPS_TOKEN".to_string(), "test-token".to_string());

        let result = AccountConfig::from_env_or_file(None, &env_map).unwrap().unwrap();
        assert_eq!(result.devops_platform, "coding");
        assert_eq!(result.base_url, "https://test.coding.net");
        assert_eq!(result.token, "test-token");
    }

    #[test]
    fn test_missing_required_field() {
        let mut env_map = HashMap::new();
        env_map.insert("GITAI_DEVOPS_PLATFORM".to_string(), "coding".to_string());
        // Missing base_url and token

        let result = AccountConfig::from_env_or_file(None, &env_map);
        assert!(result.is_err());
    }

    #[test]
    fn test_env_overrides_file() {
        let file_config = PartialAccountConfig {
            devops_platform: Some("file-platform".to_string()),
            base_url: Some("file-url".to_string()),
            token: Some("file-token".to_string()),
            timeout: Some(60000),
            retry_count: Some(5),
        };

        let mut env_map = HashMap::new();
        env_map.insert("GITAI_DEVOPS_PLATFORM".to_string(), "env-platform".to_string());
        env_map.insert("GITAI_DEVOPS_BASE_URL".to_string(), "env-url".to_string());
        env_map.insert("GITAI_DEVOPS_TOKEN".to_string(), "env-token".to_string());

        let result = AccountConfig::from_env_or_file(Some(file_config), &env_map).unwrap().unwrap();
        assert_eq!(result.devops_platform, "env-platform"); // env override
        assert_eq!(result.base_url, "env-url"); // env override
        assert_eq!(result.token, "env-token"); // env override
        assert_eq!(result.timeout, Some(60000)); // from file
        assert_eq!(result.retry_count, Some(5)); // from file
    }
}
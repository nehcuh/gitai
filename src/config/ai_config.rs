use serde::Deserialize;
use std::collections::HashMap;
use crate::errors::{AppError, config_error};

/// AI Configuration - 简化版本，直接使用Option字段
#[derive(Deserialize, Debug, Clone, Default)]
pub struct AIConfig {
    pub api_url: Option<String>,
    pub model_name: Option<String>,
    pub temperature: Option<f32>,
    pub api_key: Option<String>,
}

/// 解析后的AI配置，所有字段都有值
#[derive(Debug, Clone)]
pub struct ResolvedAIConfig {
    pub api_url: String,
    pub model_name: String,
    pub temperature: f32,
    pub api_key: Option<String>,
}

impl AIConfig {
    /// 从环境变量和文件配置合并
    pub fn merge_with_env(
        mut self,
        env_map: &HashMap<String, String>,
    ) -> Result<ResolvedAIConfig, AppError> {
        // 环境变量优先，其次是文件配置，最后是默认值
        if self.api_url.is_none() {
            self.api_url = env_map.get("GITAI_AI_API_URL").cloned();
        }
        if self.model_name.is_none() {
            self.model_name = env_map.get("GITAI_AI_MODEL").cloned();
        }
        if self.temperature.is_none() {
            self.temperature = env_map.get("GITAI_AI_TEMPERATURE")
                .and_then(|s| s.parse().ok());
        }
        if self.api_key.is_none() {
            self.api_key = env_map.get("GITAI_AI_API_KEY").cloned();
        }

        Ok(ResolvedAIConfig {
            api_url: self.api_url.unwrap_or_else(default_ai_api_url),
            model_name: self.model_name.unwrap_or_else(default_ai_model_name),
            temperature: self.temperature.unwrap_or_else(default_ai_temperature),
            api_key: self.api_key,
        })
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), AppError> {
        if let Some(ref url) = self.api_url {
            if url.trim().is_empty() {
                return Err(config_error("API URL 不能为空"));
            }
        }
        if let Some(ref model) = self.model_name {
            if model.trim().is_empty() {
                return Err(config_error("模型名称不能为空"));
            }
        }
        Ok(())
    }
}

// Default functions
fn default_ai_api_url() -> String {
    "http://localhost:11434/v1/chat/completions".to_string()
}

fn default_ai_model_name() -> String {
    "qwen2.5:32b".to_string()
}

fn default_ai_temperature() -> f32 {
    0.3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_config_merge_with_env() {
        let config = AIConfig::default();
        let mut env_map = HashMap::new();
        env_map.insert("GITAI_AI_API_URL".to_string(), "http://test.com".to_string());
        env_map.insert("GITAI_AI_MODEL".to_string(), "test-model".to_string());

        let resolved = config.merge_with_env(&env_map).unwrap();
        assert_eq!(resolved.api_url, "http://test.com");
        assert_eq!(resolved.model_name, "test-model");
        assert_eq!(resolved.temperature, 0.3); // default
    }

    #[test]
    fn test_ai_config_from_file() {
        let config = AIConfig {
            api_url: Some("http://file.com".to_string()),
            model_name: Some("file-model".to_string()),
            temperature: Some(0.7),
            api_key: Some("file-key".to_string()),
        };

        let resolved = config.merge_with_env(&HashMap::new()).unwrap();
        assert_eq!(resolved.api_url, "http://file.com");
        assert_eq!(resolved.model_name, "file-model");
        assert_eq!(resolved.temperature, 0.7);
        assert_eq!(resolved.api_key, Some("file-key".to_string()));
    }

    #[test]
    fn test_env_overrides_file() {
        let config = AIConfig {
            api_url: Some("http://file.com".to_string()),
            model_name: Some("file-model".to_string()),
            temperature: Some(0.7),
            api_key: Some("file-key".to_string()),
        };

        let mut env_map = HashMap::new();
        env_map.insert("GITAI_AI_API_URL".to_string(), "http://env.com".to_string());
        env_map.insert("GITAI_AI_API_KEY".to_string(), "env-key".to_string());

        let resolved = config.merge_with_env(&env_map).unwrap();
        assert_eq!(resolved.api_url, "http://env.com"); // env override
        assert_eq!(resolved.model_name, "file-model"); // from file
        assert_eq!(resolved.temperature, 0.7); // from file
        assert_eq!(resolved.api_key, Some("env-key".to_string())); // env override
    }

    #[test]
    fn test_validation() {
        // 有效配置
        let config = AIConfig {
            api_url: Some("http://test.com".to_string()),
            model_name: Some("test-model".to_string()),
            temperature: Some(0.7),
            api_key: None,
        };
        assert!(config.validate().is_ok());

        // 无效配置 - 空URL
        let config = AIConfig {
            api_url: Some("".to_string()),
            model_name: Some("test-model".to_string()),
            temperature: Some(0.7),
            api_key: None,
        };
        assert!(config.validate().is_err());
    }
}
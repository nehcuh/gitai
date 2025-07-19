use crate::errors::ConfigError;
use serde::Deserialize;
use std::collections::HashMap;

#[cfg(test)]
use serde_json;

/// AI Configuration
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct AIConfig {
    #[serde(default = "default_ai_api_url")]
    pub api_url: String,
    #[serde(default = "default_ai_model_name")]
    pub model_name: String,
    #[serde(default = "default_ai_temperature")]
    pub temperature: f32,
    pub api_key: Option<String>,
}

impl AIConfig {
    /// Create AIConfig from environment variables and file config
    pub fn from_env_or_file(
        file_ai_config: Option<AIConfig>,
        env_map: &HashMap<String, String>,
    ) -> Result<Self, ConfigError> {
        // Get values from environment with fallback to file config
        let api_url = env_map
            .get("GITAI_AI_API_URL")
            .cloned()
            .or_else(|| file_ai_config.as_ref().map(|c| c.api_url.clone()))
            .unwrap_or_else(default_ai_api_url);

        let model_name = env_map
            .get("GITAI_AI_MODEL")
            .cloned()
            .or_else(|| file_ai_config.as_ref().map(|c| c.model_name.clone()))
            .unwrap_or_else(default_ai_model_name);

        let temperature = env_map
            .get("GITAI_AI_TEMPERATURE")
            .and_then(|s| s.parse().ok())
            .or_else(|| file_ai_config.as_ref().map(|c| c.temperature))
            .unwrap_or_else(default_ai_temperature);

        let api_key = env_map
            .get("GITAI_AI_API_KEY")
            .cloned()
            .or_else(|| file_ai_config.as_ref().and_then(|c| c.api_key.clone()));

        Ok(AIConfig {
            api_url,
            model_name,
            temperature,
            api_key,
        })
    }
}

// Default functions
fn default_ai_api_url() -> String {
    "http://localhost:11434/v1/chat/completions".to_string()
}

fn default_ai_model_name() -> String {
    "deepseek-r1-0528-qwen3-8b-awq".to_string()
}

fn default_ai_temperature() -> f32 {
    0.3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_config_from_env() {
        let mut env_map = HashMap::new();
        env_map.insert(
            "GITAI_AI_API_URL".to_string(),
            "http://test.com".to_string(),
        );
        env_map.insert("GITAI_AI_MODEL".to_string(), "test-model".to_string());

        let config = AIConfig::from_env_or_file(None, &env_map).unwrap();
        assert_eq!(config.api_url, "http://test.com");
        assert_eq!(config.model_name, "test-model");
        assert_eq!(config.temperature, 0.3); // default
    }

    #[test]
    fn test_ai_config_from_file() {
        let file_config = AIConfig {
            api_url: "http://file.com".to_string(),
            model_name: "file-model".to_string(),
            temperature: 0.7,
            api_key: Some("file-key".to_string()),
        };

        let config = AIConfig::from_env_or_file(Some(file_config), &HashMap::new()).unwrap();
        assert_eq!(config.api_url, "http://file.com");
        assert_eq!(config.model_name, "file-model");
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.api_key, Some("file-key".to_string()));
    }

    #[test]
    fn test_env_overrides_file() {
        let file_config = AIConfig {
            api_url: "http://file.com".to_string(),
            model_name: "file-model".to_string(),
            temperature: 0.7,
            api_key: Some("file-key".to_string()),
        };

        let mut env_map = HashMap::new();
        env_map.insert("GITAI_AI_API_URL".to_string(), "http://env.com".to_string());
        env_map.insert("GITAI_AI_API_KEY".to_string(), "env-key".to_string());

        let config = AIConfig::from_env_or_file(Some(file_config), &env_map).unwrap();
        assert_eq!(config.api_url, "http://env.com"); // env override
        assert_eq!(config.model_name, "file-model"); // from file
        assert_eq!(config.temperature, 0.7); // from file
        assert_eq!(config.api_key, Some("env-key".to_string())); // env override
    }

    #[test]
    fn test_serde_deserialize_partial() {
        let json_partial = r#"{"api_url": "http://partial.com"}"#;
        let config: AIConfig = serde_json::from_str(json_partial).unwrap();

        assert_eq!(config.api_url, "http://partial.com");
        assert_eq!(config.model_name, "deepseek-r1-0528-qwen3-8b-awq"); // default
        assert_eq!(config.temperature, 0.3); // default
        assert_eq!(config.api_key, None); // default
    }

    #[test]
    fn test_serde_deserialize_empty() {
        let json_empty = r#"{}"#;
        let config: AIConfig = serde_json::from_str(json_empty).unwrap();

        assert_eq!(config.api_url, "http://localhost:11434/v1/chat/completions"); // default
        assert_eq!(config.model_name, "deepseek-r1-0528-qwen3-8b-awq"); // default
        assert_eq!(config.temperature, 0.3); // default
        assert_eq!(config.api_key, None); // default
    }
}

//! AI adapter traits and models

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// AI generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    /// prompt or instruction
    pub prompt: String,
    /// optional context map
    pub context: Option<std::collections::HashMap<String, String>>,
    /// max tokens
    pub max_tokens: Option<u32>,
    /// temperature
    pub temperature: Option<f32>,
}

/// AI generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    /// generated text
    pub content: String,
    /// provider model
    pub model: String,
    /// usage information (optional)
    pub usage: Option<UsageInfo>,
    /// ISO8601 timestamp
    pub generated_at: String,
}

/// token usage info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    /// prompt tokens
    pub prompt_tokens: u32,
    /// completion tokens
    pub completion_tokens: u32,
    /// total tokens
    pub total_tokens: u32,
}

/// AI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderConfig {
    /// api base url
    pub api_url: String,
    /// model name
    pub model: String,
    /// api key (optional if local)
    pub api_key: Option<String>,
    /// request timeout seconds
    pub timeout: Option<u64>,
}

/// Adapter trait for AI providers
#[async_trait]
pub trait AiAdapter: Send + Sync {
    /// provider name (e.g., openai, ollama)
    fn name(&self) -> &'static str;

    /// generate content from prompt
    async fn generate(&self, request: AiRequest) -> anyhow::Result<AiResponse>;
}

/// Helper to build a basic response with timestamp
pub fn build_response(content: String, model: impl Into<String>, usage: Option<UsageInfo>) -> AiResponse {
    AiResponse {
        content,
        model: model.into(),
        usage,
        generated_at: Utc::now().to_rfc3339(),
    }
}


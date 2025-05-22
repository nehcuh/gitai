use serde::{Deserialize, Serialize};

/// Represents a chat message with a role and content
///
/// This structure is used for both requests to and responses from AI chat models
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct OpenAIChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub stream: bool,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[allow(dead_code)] // Add allow(dead_code) to suppress warnings for unused fields
pub struct OpenAIChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64, // Typically a UNIX Timestamp
    pub model: String,
    pub system_fingerprint: Option<String>, // This field exists based on the exampale provided
    pub choices: Vec<OpenAIChoice>,
    pub usage: OpenAIUsage,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[allow(dead_code)] // Add allow(dead_code) to suppress warnings for unused fields
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[allow(dead_code)] // Add allow(dead_code) to suppress warnings for unused fields
pub struct OpenAIChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
    // pub logprobs: Option<serde_json::Value> // If logprobs parsing is needed
}

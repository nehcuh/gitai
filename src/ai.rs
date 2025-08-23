use crate::config::Config;
use serde::{Deserialize, Serialize};

/// AI请求
#[derive(Serialize)]
struct AiRequest {
    model: String,
    messages: Vec<AiMessage>,
    temperature: f32,
}

/// AI消息
#[derive(Serialize, Deserialize)]
struct AiMessage {
    role: String,
    content: String,
}

/// AI响应
#[derive(Deserialize)]
struct AiResponse {
    choices: Vec<AiChoice>,
}

#[derive(Deserialize)]
struct AiChoice {
    message: AiMessage,
}

/// 简化的AI调用
pub async fn call_ai(config: &Config, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    let request = AiRequest {
        model: config.ai.model.clone(),
        messages: vec![
            AiMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant for Git operations.".to_string(),
            },
            AiMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ],
        temperature: config.ai.temperature,
    };
    
    let response = client
        .post(&config.ai.api_url)
        .json(&request)
        .header("Authorization", config.ai.api_key.as_deref().unwrap_or("Bearer"))
        .send()
        .await?;
    
    let ai_response: AiResponse = response.json().await?;
    let content = ai_response.choices.first()
        .ok_or("No response from AI")?
        .message
        .content
        .clone();
    
    Ok(content)
}

/// 生成提交信息
pub async fn generate_commit_message(config: &Config, diff: &str) -> Result<String, Box<dyn std::error::Error>> {
    let prompt = format!(
        "Generate a concise commit message for the following changes:\n\n{diff}"
    );
    
    call_ai(config, &prompt).await
}

/// 代码评审
pub async fn review_code(config: &Config, diff: &str) -> Result<String, Box<dyn std::error::Error>> {
    let prompt = format!(
        "Review the following code changes and provide feedback:\n\n{diff}"
    );
    
    call_ai(config, &prompt).await
}
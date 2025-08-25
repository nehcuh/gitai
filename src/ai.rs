use crate::config::Config;
use crate::prompts::{PromptManager, PromptContext};
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
pub async fn call_ai(config: &Config, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    
    let mut req = client
        .post(&config.ai.api_url)
        .json(&request);
    if let Some(ref key) = config.ai.api_key {
        req = req.header("Authorization", format!("Bearer {}", key));
    }
    let response = req
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
pub async fn generate_commit_message(config: &Config, diff: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let prompt = format!(
        "Generate a concise commit message for the following changes:\n\n{diff}"
    );
    
    call_ai(config, &prompt).await
}

/// 代码评审
pub async fn review_code(config: &Config, diff: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let prompt = format!(
        "Review the following code changes and provide feedback:\n\n{diff}"
    );
    
    call_ai(config, &prompt).await
}

/// 使用提示词模板调用AI
pub async fn call_ai_with_template(
    config: &Config, 
    template_name: &str, 
    context: &PromptContext
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let prompt_manager = PromptManager::new(config.clone());
    let language = prompt_manager.get_language();
    
    // 加载并渲染提示词模板
    let rendered_prompt = prompt_manager.load_and_render(template_name, context, language)?;
    
    let client = reqwest::Client::new();
    
    let request = AiRequest {
        model: config.ai.model.clone(),
        messages: vec![
            AiMessage {
                role: "user".to_string(),
                content: rendered_prompt,
            },
        ],
        temperature: config.ai.temperature,
    };
    
    let mut req = client
        .post(&config.ai.api_url)
        .json(&request);
    if let Some(ref key) = config.ai.api_key {
        req = req.header("Authorization", format!("Bearer {}", key));
    }
    let response = req
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

/// 使用review模板进行代码评审
pub async fn review_code_with_template(
    config: &Config, 
    diff: &str, 
    tree_sitter_summary: Option<&str>
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut context = PromptContext::new()
        .with_variable("diff", diff);
    
    if let Some(summary) = tree_sitter_summary {
        context = context.with_variable("tree_sitter_summary", summary);
    } else {
        context = context.with_variable("tree_sitter_summary", "");
    }
    
    call_ai_with_template(config, "review", &context).await
}

/// 使用commit-generator模板生成提交信息
pub async fn generate_commit_message_with_template(
    config: &Config, 
    diff: &str
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let context = PromptContext::new()
        .with_variable("diff", diff);
    
    call_ai_with_template(config, "commit-generator", &context).await
}

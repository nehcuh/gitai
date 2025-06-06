use crate::{
    config::AppConfig,
    errors::AIError,
    types::ai::{ChatMessage, OpenAIChatCompletionResponse, OpenAIChatRequest},
};
use lazy_static::lazy_static;
use regex::Regex;

/// Takes the raw output from a Git command (typically its help text)
/// This function can handle both standard git help output and gitai-enhanced help.
pub async fn explain_git_command_output(
    config: &AppConfig,
    command_output: &str,
) -> Result<String, AIError> {
    if command_output.trim().is_empty() {
        // This is not an error, but a valid case where there's nothing to explain
        return Ok("该命令没有产生输出供 AI 解释。\
            这可能是一个成功时不打印到标准输出/标准错误的命令，\
            或者需要特定条件才能产生输出。"
            .to_string());
    }

    tracing::debug!(
        "请求 AI 解释命令输出 (前 200 个字符):\n---\n{}\n---",
        command_output.chars().take(200).collect::<String>()
    );

    // Determine if this contains gitai custom help
    // let contains_gitai_help = command_output.contains("gitai: Git with AI assistance")
    //     || command_output.contains("Gitai 特有命令");

    // Enhance system prompt to handle gitie-specific commands
    tracing::debug!("system_prompt_content: {:#?}\n\n", config.prompts);
    let system_prompt_content = config
        .prompts
        .get("general-helper")
        .cloned()
        .unwrap_or_else(|| {
            tracing::warn!("在配置中未找到 Git AI helper 提示词，使用空字符串");
            "".to_string()
        });

    // Add gitai-specified instructions if needed
    // if contains_gitai_help {
    //     system_prompt_content = format!("{}\n\n此帮助内容包含标准 Git 命令", system_prompt_content);
    // }

    let user_prompt = format!(
        "请解释以下{}帮助信息，重点说明每个命令的作用和用法：\n\n{}",
        "Git ", command_output
    );

    match execute_explain_request(config, &system_prompt_content, &user_prompt).await {
        Ok(ai_explanation) => {
            let formatted_output = format!(
                "#Git 命令帮助\n\n##原始帮助输出\n\n```text\n{}\n```\n## AI 解释\n\n{}\n",
                ai_explanation, command_output
            );
            Ok(formatted_output)
        }
        Err(e) => Err(e),
    }
}

/// Generic function to execute AI request with configurable options
pub async fn execute_ai_request_generic(
    config: &AppConfig,
    messages: Vec<ChatMessage>,
    log_prefix: &str,
    clean_output: bool,
) -> Result<String, AIError> {
    let request_payload = OpenAIChatRequest {
        model: config.ai.model_name.clone(),
        messages,
        temperature: Some(config.ai.temperature),
        stream: false,
    };

    if let Ok(json_string) = serde_json::to_string_pretty(&request_payload) {
        tracing::debug!("正在发送 JSON 数据到 AI 进行{}:\n{}", log_prefix, json_string);
    } else {
        tracing::warn!("序列化 AI 请求数据用于调试失败。");
    }

    let client = reqwest::Client::new();
    let mut request_builder = client.post(&config.ai.api_url);

    // Add authorization header if api_key present
    if let Some(api_key) = &config.ai.api_key {
        if !api_key.is_empty() {
            tracing::debug!("正在使用 API 密钥进行 AI {}", log_prefix);
            request_builder = request_builder.bearer_auth(api_key);
        }
    }

    let openai_response = request_builder
        .json(&request_payload)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("发送 AI {}请求失败: {}", log_prefix, e);
            AIError::RequestFailed(e)
        })?;

    if !openai_response.status().is_success() {
        let status_code = openai_response.status();
        let body = openai_response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body from AI response".to_string());
        tracing::error!("AI {} API 请求失败，状态码: {}: {}", log_prefix, status_code, body);
        return Err(AIError::ApiResponseError(status_code, body));
    }

    // Successfully received a response, now parse it.
    match openai_response.json::<OpenAIChatCompletionResponse>().await {
        Ok(response_data) => {
            if let Some(choice) = response_data.choices.get(0) {
                let original_content = &choice.message.content;
                if original_content.trim().is_empty() {
                    tracing::warn!("AI {}返回了空的消息内容。", log_prefix);
                    Err(AIError::EmptyMessage)
                } else {
                    let final_content = if clean_output {
                        clean_ai_output(original_content)
                    } else {
                        original_content.clone()
                    };
                    
                    let log_msg = if clean_output { "清理后的" } else { "" };
                    tracing::debug!(
                        "收到{}AI {}: \"{}\"",
                        log_msg,
                        log_prefix,
                        final_content.chars().take(100).collect::<String>()
                    );
                    Ok(final_content)
                }
            } else {
                tracing::warn!("在 AI {}响应中未找到选项。", log_prefix);
                Err(AIError::NoChoiceInResponse)
            }
        }
        Err(e) => {
            tracing::error!("解析来自 AI {}的 JSON 响应失败: {}", log_prefix, e);
            Err(AIError::ResponseParseFailed(e))
        }
    }
}

// Removes <think>...</think> tags and their content from a given string
//
// The regex pattern is compiled once using lazy_static for better performance
// since this function might be called frequently.
lazy_static! {
    static ref RE_THINK_TAGS: Regex = Regex::new(r"(?s)<think>.*?</think>").unwrap();
}

/// Helper function to execute the AI request and process the response
/// This is a wrapper around execute_ai_request_generic with default options for backward compatibility
async fn execute_ai_request(
    config: &AppConfig,
    messages: Vec<ChatMessage>,
) -> Result<String, AIError> {
    execute_ai_request_generic(config, messages, "解释", true).await
}

/// Dedicated function for code review requests
/// Returns the raw AI response without cleaning <think> tags as they might be useful for review context
pub async fn execute_review_request(
    config: &AppConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, AIError> {
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt.to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: user_prompt.to_string(),
        },
    ];

    execute_ai_request_generic(config, messages, "评审", false).await
}

/// Dedicated function for explanation requests  
/// Cleans the output by removing <think> tags for cleaner explanations
pub async fn execute_explain_request(
    config: &AppConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, AIError> {
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt.to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: user_prompt.to_string(),
        },
    ];

    execute_ai_request_generic(config, messages, "解释", true).await
}

/// Helper function to create a review prompt for code changes
pub fn create_review_prompt(
    diff_text: &str,
    analysis: &str,
    focus: Option<&str>,
    languages: &str,
) -> String {
    let focus_instruction = if let Some(focus) = focus {
        format!("\n\n**特别关注的方面:** {}", focus)
    } else {
        String::new()
    };

    let language_context = if !languages.is_empty() {
        format!("\n\n**检测到的编程语言:** {}", languages)
    } else {
        String::new()
    };

    format!(
        "## 代码评审请求{}{}\n\n## 代码结构分析\n\n{}\n\n## 代码变更\n\n```diff\n{}\n```",
        focus_instruction, language_context, analysis, diff_text
    )
}

pub fn clean_ai_output(text: &str) -> String {
    // Using the pre-compiled regex pattern for better performance
    RE_THINK_TAGS.replace_all(text, "").into_owned()
}

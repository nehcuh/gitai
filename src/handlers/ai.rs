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
    explain_git_command_output_with_language(config, command_output, None).await
}

/// Takes the raw output from a Git command with language support
pub async fn explain_git_command_output_with_language(
    config: &AppConfig,
    command_output: &str,
    language: Option<&str>,
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

    // Get language-specific prompt content
    let effective_language_string = match language {
        Some(lang) => lang.to_string(),
        None => config.get_output_language(None)
    };
    let effective_language = effective_language_string.as_str();
    
    let system_prompt_content = config
        .get_language_prompt_content("general-helper", effective_language)
        .unwrap_or_else(|e| {
            tracing::warn!("获取{}语言的general-helper prompt失败: {}，尝试使用默认prompt", effective_language, e);
            config.prompts
                .get("general-helper")
                .cloned()
                .unwrap_or_else(|| {
                    tracing::warn!("在配置中未找到默认Git AI helper提示词，使用空字符串");
                    "".to_string()
                })
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
    execute_ai_request_with_language(config, messages, log_prefix, clean_output, None).await
}

/// Generic function to execute AI request with language support
pub async fn execute_ai_request_with_language(
    config: &AppConfig,
    messages: Vec<ChatMessage>,
    log_prefix: &str,
    clean_output: bool,
    _language: Option<&str>, // Language parameter for future use
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
/// Dedicated function for code review requests
/// Returns the raw AI response without cleaning <think> tags as they might be useful for review context
pub async fn execute_review_request(
    config: &AppConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, AIError> {
    execute_review_request_with_language(config, system_prompt, user_prompt, None).await
}

/// Execute review request with language support
pub async fn execute_review_request_with_language(
    config: &AppConfig,
    system_prompt: &str,
    user_prompt: &str,
    language: Option<&str>,
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

    let log_prefix = match language {
        Some("us") => "review",
        _ => "评审"
    };
    execute_ai_request_with_language(config, messages, log_prefix, false, language).await
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

/// Dedicated function for translation requests
/// Uses the translator.md prompt for translating scan results
pub async fn execute_translation_request(
    config: &AppConfig,
    content: &str,
    target_language: &str,
) -> Result<String, AIError> {
    // Get translator prompt content
    let translator_prompt = config.prompts
        .get("translator")
        .cloned()
        .unwrap_or_else(|| {
            tracing::warn!("获取translator prompt失败，使用简单翻译模式");
            "You are a translator. Translate the following content to the target language. Keep the original format.".to_string()
        });

    let target_lang_name = match target_language {
        "us" => "English",
        "cn" => "Chinese",
        _ => "the target language"
    };

    let user_prompt = format!(
        "{}\n\nTarget language: {}\n\nContent to translate:\n\n{}",
        translator_prompt,
        target_lang_name,
        content
    );

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "You are a professional translator. Follow the instructions carefully.".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: user_prompt,
        },
    ];

    let log_prefix = match target_language {
        "us" => "translation to English",
        "cn" => "翻译为中文",
        _ => "translation"
    };
    
    execute_ai_request_generic(config, messages, log_prefix, true).await
}

/// Helper function to create a review prompt for code changes
pub fn create_review_prompt(
    diff_text: &str,
    analysis: &str,
    focus: Option<&str>,
    languages: &str,
) -> String {
    create_review_prompt_with_language(diff_text, analysis, focus, languages, None)
}

/// Helper function to create a review prompt for code changes with language support
pub fn create_review_prompt_with_language(
    diff_text: &str,
    analysis: &str,
    focus: Option<&str>,
    languages: &str,
    output_language: Option<&str>,
) -> String {
    // Use language-specific prompts based on output_language
    let (focus_label, language_label, review_header, analysis_header, changes_header) = match output_language {
        Some("us") => (
            "**Special Focus Areas:**",
            "**Detected Programming Languages:**",
            "## Code Review Request",
            "## Code Structure Analysis",
            "## Code Changes"
        ),
        _ => (
            "**特别关注的方面:**",
            "**检测到的编程语言:**",
            "## 代码评审请求",
            "## 代码结构分析",
            "## 代码变更"
        )
    };

    let focus_instruction = if let Some(focus) = focus {
        format!("\n\n{} {}", focus_label, focus)
    } else {
        String::new()
    };

    let language_context = if !languages.is_empty() {
        format!("\n\n{} {}", language_label, languages)
    } else {
        String::new()
    };

    format!(
        "{}{}{}\n\n{}\n\n{}\n\n{}\n\n```diff\n{}\n```",
        review_header, focus_instruction, language_context, 
        analysis_header, analysis, changes_header, diff_text
    )
}

pub fn clean_ai_output(text: &str) -> String {
    // Using the pre-compiled regex pattern for better performance
    RE_THINK_TAGS.replace_all(text, "").into_owned()
}

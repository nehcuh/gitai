use crate::{
    ai_module::utils::clean_ai_output, config_management::settings::AppConfig,
    core::errors::AIError,
};

use super::utils::{ChatMessage, OpenAIChatCompletionResponse, OpenAIChatRequest};

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
    let contains_gitai_help = command_output.contains("gitai: Git with AI assistance")
        || command_output.contains("Gitai 特有命令");

    // Enhance system prompt to handle gitie-specific commands
    let mut system_prompt_content =
        config
            .prompts
            .get("expert-helper")
            .cloned()
            .unwrap_or_else(|| {
                tracing::warn!("在配置中未找到 Git AI helper 提示词，使用空字符串");
                "".to_string()
            });

    // Add gitai-specified instructions if needed
    if contains_gitai_help {
        system_prompt_content = format!(
            "{}\n\n此帮助内容包含标准 Git 命令和 Gitie 特有命令。请分别解释这两部分：\n\
            1. Gitie 特有命令：详细解释这些 AI 增强的命令如何工作以及它们的参数\n\
            2. 标准 Git 命令：提供简洁明了的解释\n\
            始终关注帮助用户理解如何使用 Gitie 的 AI 功能来提高生产力",
            system_prompt_content
        );
    }

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt_content,
        },
        ChatMessage {
            role: "user".to_string(),
            content: format!(
                "请解释以下{}帮助信息，重点说明每个命令的作用和用法：\n\n{}",
                if contains_gitai_help {
                    "Gitai 和 Git "
                } else {
                    "Git "
                },
                command_output
            ),
        },
    ];

    match execute_ai_request(config, messages).await {
        Ok(ai_explanation) => {
            let formatted_output = if contains_gitai_help {
                format!(
                    "# Gitai 命令帮助\n\n## AI 解释\n\n{}\n\n##原始帮助输出\n\n```text\n{}\n```",
                    ai_explanation, command_output
                )
            } else {
                format!(
                    "#Git 命令帮助\n\n## AI 解释\n\n{}\n\n##原始帮助输出\n\n```text\n{}\n```",
                    ai_explanation, command_output
                )
            };
            Ok(formatted_output)
        }
        Err(e) => Err(e),
    }
}

/// Takes a Git command (as a sequence of its parts/arguments)
/// and returns an AI-generated explanation of what that command does
pub async fn explain_git_command(
    config: &AppConfig,
    command_parts: &[String],
) -> Result<String, AIError> {
    if command_parts.is_empty() {
        // This is not an error from AI's perspective but an invalid input to this function.
        return Ok("没有提供命令部分供 AI 解释".to_string());
    }

    let command_to_explain = format!("git {}", command_parts.join(" "));
    tracing::debug!("请求 AI 解释命令: {}", command_to_explain);

    let user_message_content = command_to_explain;

    let system_prompt_content = config
        .prompts
        .get("expoert-helper")
        .cloned()
        .unwrap_or_else(|| {
            tracing::warn!("在配置中未找到 Git AI helper 提示词，使用空字符串");
            "".to_string()
        });

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt_content, // Use the prompt from config
        },
        ChatMessage {
            role: "user".to_string(),
            content: user_message_content,
        },
    ];

    execute_ai_request(config, messages).await
}

/// Helpter function to execute the AI request and process the response
async fn execute_ai_request(
    config: &AppConfig,
    messages: Vec<ChatMessage>,
) -> Result<String, AIError> {
    let request_payload = OpenAIChatRequest {
        model: config.ai.model_name.clone(),
        messages,
        temperature: Some(config.ai.temperature),
        stream: false,
    };

    if let Ok(json_string) = serde_json::to_string_pretty(&request_payload) {
        tracing::debug!("正在发送 JSON 数据到 AI 进行解释:\n{}", json_string);
    } else {
        tracing::warn!("序列化 AI 请求数据用于调试失败。");
    }

    let mut client = reqwest::Client::new();
    let mut request_builder = client.post(&config.ai.api_url);

    // Add authorization header i api_key represent
    if let Some(api_key) = &config.ai.api_key {
        if !api_key.is_empty() {
            tracing::debug!("正在使用 API 密钥进行 AI 解释");
            request_builder = request_builder.bearer_auth(api_key);
        }
    }

    let openai_response = request_builder
        .json(&request_payload)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("发送 AI 解释请求失败: {}", e);
            // This error could be a network issue, DNS resolution failure, etc.
            // AIError::RequestFailed is a general error for reqwest issues.
            // AIError::ExplainerNetworkError could be used if a more specific categorization is needed
            // and can be reliably determined from `e`.
            AIError::RequestFailed(e)
        })?;

    if !openai_response.status().is_success() {
        let status_code = openai_response.status();
        let body = openai_response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body from AI response".to_string());
        tracing::error!("AI 解释器 API 请求失败，状态码: {}: {}", status_code, body);
        return Err(AIError::ApiResponseError(status_code, body));
    }

    // Successfully received a response, now parse it.
    match openai_response.json::<OpenAIChatCompletionResponse>().await {
        Ok(response_data) => {
            if let Some(choice) = response_data.choices.get(0) {
                let original_content = &choice.message.content;
                if original_content.trim().is_empty() {
                    tracing::warn!("AI 解释器返回了空的消息内容。");
                    Err(AIError::EmptyMessage)
                } else {
                    let cleaned_content = clean_ai_output(original_content);
                    tracing::debug!(
                        "收到清理后的 AI 解释: \"{}\"",
                        cleaned_content.chars().take(100).collect::<String>()
                    );
                    Ok(cleaned_content)
                }
            } else {
                tracing::warn!("在 AI 解释器响应中未找到选项。");
                Err(AIError::NoChoiceInResponse)
            }
        }
        Err(e) => {
            tracing::error!("解析来自 AI 解释器的 JSON 响应失败: {}", e);
            // This error occurs if the response body is not valid JSON matching OpenAIChatCompletionResponse
            Err(AIError::ResponseParseFailed(e))
        }
    }
}

/// Explains Git errors and provides helpful information
///
/// This function uses the specialized expert-prompt system prompt to analyze Git errors
/// and provide clearer explanations and possible solutions
///
/// # Arguments
///
/// * `config` - Application configuration containing AI parameters and prompts
/// * `error_output` - Error output from Git command execution
/// * `command` - Command string executed by the user, for context
///
/// # Returns
///
/// * `Result<String, AIError>` - Formatted error explanation or error
pub async fn explain_git_error(
    config: &AppConfig,
    error_output: &str,
    command: &str,
) -> Result<String, AIError> {
    // Validate input
    if error_output.trim().is_empty() {
        return Ok(
            "Git 命令未产生错误输出，但执行失败。这可能是权限问题或者其它系统级别的错误。"
                .to_string(),
        );
    }

    tracing::debug!(
        "请求 AI 分析 Git 错误 (命令: {}): {:?}",
        command,
        error_output.chars().take(200).collect::<String>()
    );

    // Get expert-prompt system prompt
    let system_prompt_content = config
        .prompts
        .get("expert-helper")
        .cloned()
        .unwrap_or_else(|| {
            tracing::warn!("Expert prompt 提示词未在配置中找到，使用空字符串");
            "".to_string()
        });

    // Build user message containing command and error output
    let user_message = format!(
        "在执行以下 Git 命令时遇到错误：\n\n命令: {}\n\n错误输出:\n{}\n\n请分析这个错误，解释原因并提供解决方案。",
        command, error_output
    );

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt_content,
        },
        ChatMessage {
            role: "user".to_string(),
            content: user_message,
        },
    ];

    // Send AI request and get explanation
    match execute_ai_request(config, messages).await {
        Ok(ai_explanation) => {
            // Format output, including original error and AI explanation
            let formatted_output = format!(
                "【原始 Git 错误】\n{}\n\n【Gitai AI 帮助】\n{}",
                error_output, ai_explanation
            );
            Ok(formatted_output)
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_management::settings::AIConfig;
    use std::collections::HashMap;

    // Test explain_git_error function with empty error output
    #[tokio::test]
    async fn test_explain_git_error_empty_output() {
        // Create test configuration
        let mut config = AppConfig {
            ai: AIConfig::default(),
            tree_sitter: crate::config_management::settings::TreeSitterConfig::default(),
            prompts: HashMap::new(),
        };
        config
            .prompts
            .insert("expert-helper".to_string(), "测试提示词".to_string());

        // Mock error output
        let error_output = "";
        let command = "git comit -m 'test'";

        // Call function
        let result = explain_git_error(&config, error_output, command).await;

        // Verify results
        assert!(result.is_ok(), "函数应该成功执行");
        let explanation = result.unwrap();
        assert!(
            explanation.contains("Git 命令未产生错误输出"),
            "应该提供默认错误解释"
        );
    }

    // Test error formatting output
    #[test]
    fn test_git_error_formatting() {
        let error_output = "git: 'comit' is not a git command. See 'git --help'.";
        let ai_explanation = "您可能是想输入 'commit' 而不是 'comit'。";

        let formatted = format!(
            "【原始 Git 错误】\n{}\n\n【Gitai AI 帮助】\n{}",
            error_output, ai_explanation
        );

        assert!(
            formatted.contains("【原始 Git 错误】"),
            "格式化输出应包含原始错误部分"
        );
        assert!(
            formatted.contains("【Gitai AI 帮助】"),
            "格式化输出应包含 AI 帮助部分"
        );
        assert!(
            formatted.contains(error_output),
            "格式化输出应包含原始错误内容"
        );
        assert!(
            formatted.contains(ai_explanation),
            "格式化输出应包含 AI 解释内容"
        );
    }

    // Test explain_git_error function with non-empty error output
    #[tokio::test]
    async fn test_explain_git_error_with_output() {
        // Create test configuration
        let mut config = AppConfig {
            ai: AIConfig::default(), // Default config likely leads to AI request failure
            tree_sitter: crate::config_management::settings::TreeSitterConfig::default(),
            prompts: HashMap::new(),
        };
        // explain_git_error uses "expert-helper" prompt
        config.prompts.insert(
            "expert-helper".to_string(),
            "Test expert prompt".to_string(),
        );

        // Mock error output
        let error_output = "fatal: not a git repository (or any of the parent directories): .git";
        let command = "git status";

        // Call function
        let result = explain_git_error(&config, error_output, command).await;

        // Verify results
        // Expecting an error because the AI request will likely fail with default/test config
        // This confirms the function didn't return early due to empty error_output
        assert!(
            result.is_err(),
            "Function should return an error when AI request fails for non-empty error output. Actual: {:?}",
            result
        );
    }
}

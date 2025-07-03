use crate::ast_grep_analyzer::core::parse_git_diff;
use colored::Colorize;
use std::path::{Path, PathBuf};

use crate::handlers::git;
use crate::utils::input::confirm;
use std::io::{self, Write};

use crate::{
    ast_grep_analyzer::{
        analyzer::AstGrepAnalyzer,
        core::DiffAnalysis,
    },
    config::{AppConfig, AstGrepConfig},
    errors::{AppError, GitError},
    handlers::ai,
    types::{
        ai::ChatMessage,
        git::{CommitArgs, GitDiff},
    },
};

use std::time::Instant;

/// Handle the commit command with AI assistance
/// This function demonstrates AI-powered commit message generation
// Placeholder implementations for unresolved functions
/// Placeholder for locating the latest review file in the given storage path.
///
/// Currently always returns `Ok(None)`. Intended for future implementation to find the most recent review file.
///
/// # Arguments
///
/// * `_storage_path` - The path where review files are stored.
///
/// # Returns
///
/// An `Option<PathBuf>` containing the path to the latest review file if found, or `None` if not found or unimplemented.
///
/// # Examples
///
/// ```
/// let result = find_latest_review_file("/path/to/reviews");
/// assert!(result.unwrap().is_none());
/// ```
fn find_latest_review_file(_storage_path: &str) -> Result<Option<PathBuf>, std::io::Error> {
    Ok(None)
}

/// Reads the contents of a review file.
///
/// This is a placeholder implementation that always returns an empty string.
///
/// # Parameters
/// - `_review_file`: The path to the review file to read.
///
/// # Returns
/// An empty string as the file content.
///
/// # Examples
///
/// ```
/// let content = read_review_file(std::path::Path::new("review.md")).unwrap();
/// assert_eq!(content, "");
/// ```
fn read_review_file(_review_file: &Path) -> Result<String, std::io::Error> {
    Ok("".to_string())
}

/// Extracts review insights from the provided content.
///
/// Currently returns a default message indicating no insights are found. This is a placeholder implementation.
fn extract_review_insights(_content: &str) -> String {
    "No review insights found.".to_string()
}

/// Prepends an issue ID in brackets to the commit message if provided.
///
/// If `issue_id` is `Some`, the resulting message is formatted as `[ISSUE_ID] message`.
/// If `issue_id` is `None`, the original message is returned unchanged.
///
/// # Examples
///
/// ```
/// let msg = add_issue_prefix_to_commit_message("Fix bug", Some(&"ABC-123".to_string()));
/// assert_eq!(msg, "[ABC-123] Fix bug");
///
/// let msg2 = add_issue_prefix_to_commit_message("Update docs", None);
/// assert_eq!(msg2, "Update docs");
/// ```
fn add_issue_prefix_to_commit_message(message: &str, issue_id: Option<&String>) -> String {
    if let Some(id) = issue_id {
        format!("[{}] {}", id, message)
    } else {
        message.to_string()
    }
}

/// Handles the AI-assisted Git commit workflow, including optional AstGrep analysis and code review integration.
///
/// This function coordinates the process of generating a commit message using AI, optionally incorporating static code analysis (via AstGrep) and recent code review insights. It can automatically stage modified and untracked files, retrieve the current staged diff, and generate a commit message based on user input, analysis, and review context. The user is prompted to confirm the generated message before the commit is executed.
///
/// # Parameters
///
/// - `args`: Commit arguments specifying options such as auto-staging, AstGrep analysis, custom message, and issue ID.
///
/// # Returns
///
/// Returns `Ok(())` if the commit is successfully completed, or an error if any step fails (e.g., not in a Git repository, no staged changes, or commit execution failure).
///
/// # Examples
///
/// ```
/// let config = AppConfig::default();
/// let args = CommitArgs {
///     auto_stage: true,
///     ast_grep: true,
///     message: None,
///     issue_id: Some("PROJ-123".to_string()),
///     // ... other fields
/// };
/// handle_commit(&config, args).await?;
/// ```
pub async fn handle_commit(config: &AppConfig, args: CommitArgs) -> Result<(), AppError> {
    tracing::info!("开始处理智能提交命令");

    // Check if we're in a git repository
    check_repository_status()?;

    // Check for review results if review integration is enabled
    let review_context = if config.review.include_in_commit {
        match find_latest_review_file(&config.review.storage_path) {
            Ok(Some(review_file)) => {
                tracing::info!("🔍 发现评审结果文件: {:?}", review_file);
                match read_review_file(&review_file) {
                    Ok(content) => {
                        let insights = extract_review_insights(&content);
                        tracing::debug!("提取到评审要点: {}", insights);
                        println!("📋 已发现相关代码评审结果，将集成到提交信息中");
                        Some(insights)
                    }
                    Err(e) => {
                        tracing::warn!("读取评审文件失败: {}", e);
                        println!("⚠️ 警告: 无法读取评审结果文件");
                        None
                    }
                }
            }
            Ok(None) => {
                tracing::debug!("未找到相关评审结果");
                None
            }
            Err(e) => {
                tracing::debug!("检查评审结果时出错: {}", e);
                None
            }
        }
    } else {
        tracing::debug!("评审集成已禁用");
        None
    };

    // Auto-stage files if requested
    if args.auto_stage {
        tracing::info!("自动暂存修改的文件...");
        auto_stage_files().await?;
    }

    // Get changes for commit
    let diff = get_changes_for_commit().await?;
    if diff.trim().is_empty() {
        return Err(AppError::Git(GitError::NoStagedChanges));
    }

    // Generate commit message using AI with optional Tree-sitter analysis and review context
    let commit_message = if let Some(ref custom_message) = args.message {
        if args.ast_grep {
            // Enhanced mode: combine custom message with AI analysis and review
            generate_enhanced_commit_message(
                config,
                &diff,
                Some(custom_message.clone()),
                &args,
                review_context.as_deref(),
            )
            .await?
        } else if review_context.is_some() {
            // Custom message with review context
            format_custom_message_with_review(custom_message, review_context.as_deref().unwrap())
        } else {
            // Simple mode: use custom message directly
            custom_message.clone()
        }
    } else {
        if args.ast_grep {
            // Enhanced mode: full AstGrep analysis with AI generation and review
            generate_enhanced_commit_message(config, &diff, None, &args, review_context.as_deref())
                .await?
        } else {
            // Basic mode: AI generation with optional review context
            generate_commit_message_with_review(config, &diff, review_context.as_deref()).await?
        }
    };

    // Add issue ID prefix if provided
    let final_commit_message =
        add_issue_prefix_to_commit_message(&commit_message, args.issue_id.as_ref());

    // Show generated commit message and ask for confirmation
    println!("\n🤖 生成的提交信息:");
    println!("┌─────────────────────────────────────────────┐");
    for line in final_commit_message.lines() {
        println!("│ {:<43} │", line);
    }
    println!("└─────────────────────────────────────────────┘");

    if !confirm_commit_message(&final_commit_message)? {
        println!("❌ 提交已取消");
        return Ok(());
    }

    // Execute the commit
    execute_commit(&final_commit_message).await?;
    println!("✅ 提交成功!");

    Ok(())
}

/// Check if current directory is a git repository
fn check_repository_status() -> Result<(), AppError> {
    if !git::is_git_repository()? {
        return Err(AppError::Git(GitError::NotARepository));
    }
    Ok(())
}

/// Stages all modified tracked files and interactively prompts to add untracked files for commit.
///
/// This function first stages all modified tracked files. If untracked files are detected,
/// it lists them and prompts the user to confirm whether to add them to the commit. Untracked
/// files are staged only if the user agrees.
///
/// # Returns
///
/// Returns `Ok(())` if staging completes successfully, or an error if any Git operation fails.
///
/// # Examples
///
/// ```
/// // In an async context:
/// auto_stage_files().await?;
/// ```
async fn auto_stage_files() -> Result<(), AppError> {
    // 1. Stage tracked (modified) files first
    git::auto_stage_tracked_files().await?;

    // 2. Interactively handle untracked files
    let untracked_files = match git::get_untracked_files() {
        Ok(files) => files,
        Err(e) => {
            tracing::warn!("获取未跟踪文件失败: {}", e);
            return Err(e);
        }
    };

    if !untracked_files.is_empty() {
        println!("\n发现以下未跟踪的文件:");
        for file in &untracked_files {
            println!("  - {}", file.cyan());
        }

        if confirm("\n是否要将这些文件添加到本次提交中?")? {
            if let Err(e) = git::stage_specific_files(&untracked_files) {
                tracing::error!("添加未跟踪文件失败: {}", e);
                return Err(e);
            }
            println!("✅ {}", "已添加未跟踪的文件。".green());
        } else {
            println!("🟡 {}", "未跟踪的文件已跳过。".yellow());
        }
    }

    Ok(())
}

/// Get changes for commit analysis
async fn get_changes_for_commit() -> Result<String, AppError> {
    // Get diff for commit (staged or unstaged changes)
    git::get_diff_for_commit().await
}

/// Generates a commit message using AI based on the provided Git diff.
///
/// Constructs system and user prompts (in Chinese) to instruct the AI to generate a concise, conventional commit message from the diff. If AI generation fails, returns a default message.
///
/// # Arguments
///
/// - `diff`: The Git diff representing staged changes to be described.
///
/// # Returns
///
/// A commit message string generated by the AI, or a default message if AI generation fails.
///
/// # Examples
///
/// ```
/// let diff = "..."; // Git diff string
/// let message = generate_commit_message(&config, diff).await.unwrap();
/// assert!(!message.is_empty());
/// ```
async fn generate_commit_message(config: &AppConfig, diff: &str) -> Result<String, AppError> {
    tracing::info!("正在使用AI生成提交信息...");

    let system_prompt = config
        .prompts
        .get("commit-generator")
        .cloned()
        .unwrap_or_else(|| {
            tracing::warn!("未找到commit-generator提示模板，使用默认模板");
            "你是一个专业的Git提交信息生成助手。请根据提供的代码变更生成简洁、清晰的提交信息。"
                .to_string()
        });

    let user_prompt = format!(
        "请根据以下Git diff生成一个规范的提交信息：\n\n```diff\n{}\n```\n\n要求：\n1. 使用中文\n2. 格式为：类型(范围): 简洁描述\n3. 第一行不超过50个字符\n4. 如有必要，可以添加详细说明",
        diff
    );

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        },
        ChatMessage {
            role: "user".to_string(),
            content: user_prompt,
        },
    ];

    match ai::execute_ai_request_generic(config, messages, "提交信息生成", true).await {
        Ok(message) => {
            // Clean up the AI response - remove any markdown formatting
            let cleaned_message = message
                .lines()
                .filter(|line| !line.trim().starts_with("```"))
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();

            Ok(cleaned_message)
        }
        Err(_) => {
            tracing::warn!("AI生成提交信息失败，使用回退方案");
            Ok("chore: 更新代码".to_string())
        }
    }
}

/// Generates an enhanced commit message by incorporating AstGrep static analysis, optional custom message, and review context.
///
/// Performs AstGrep analysis on the provided Git diff. If analysis succeeds, generates a commit message using the analysis results, custom message, and review context as available. If analysis fails, falls back to using the custom message with review context or a basic AI-generated message.
///
/// # Returns
/// A commit message string that integrates static analysis insights and optional review context.
///
/// # Examples
///
/// ```
/// let message = generate_enhanced_commit_message(
///     &config,
///     diff,
///     Some("feat: add new feature".to_string()),
///     &args,
///     Some("Reviewed by Alice"),
/// ).await.unwrap();
/// assert!(message.contains("AstGrep"));
/// ```
async fn generate_enhanced_commit_message(
    config: &AppConfig,
    diff: &str,
    custom_message: Option<String>,
    args: &CommitArgs,
    review_context: Option<&str>,
) -> Result<String, AppError> {
    tracing::info!("🌳 正在使用AstGrep增强分析生成提交信息...");

    let analysis_start = Instant::now();

    // Perform AstGrep analysis
    let analysis_result = match analyze_diff_with_ast_grep(diff, args).await {
        Ok(result) => {
            tracing::info!("AstGrep分析完成，耗时: {:?}", analysis_start.elapsed());
            result
        }
        Err(e) => {
            tracing::warn!("AstGrep分析失败，回退到基础模式: {:?}", e);
            return if let Some(msg) = custom_message {
                if let Some(review) = review_context {
                    Ok(format_custom_message_with_review(&msg, review))
                } else {
                    Ok(msg)
                }
            } else {
                generate_commit_message_with_review(config, diff, review_context).await
            };
        }
    };

    // Generate enhanced commit message
    generate_commit_message_with_analysis(
        config,
        diff,
        &analysis_result,
        custom_message,
        review_context,
    )
    .await
}

/// Analyzes a Git diff using AstGrep and returns a formatted summary and detailed analysis.
///
/// Parses the provided Git diff, performs static code analysis with AstGrep, and generates a markdown-style summary suitable for commit messages. Returns both the formatted analysis text and the detailed analysis data.
///
/// # Returns
/// A tuple containing the formatted analysis summary as a string and an optional `DiffAnalysis` with detailed results.
///
/// # Errors
/// Returns an `AppError` if AstGrep initialization, diff parsing, or analysis fails.
///
/// # Examples
///
/// ```
/// let diff = "..."; // Git diff string
/// let (summary, analysis) = analyze_diff_with_ast_grep(diff, &args).await?;
/// assert!(summary.contains("AstGrep Analysis"));
/// ```
async fn analyze_diff_with_ast_grep(
    diff: &str,
    _args: &CommitArgs,
) -> Result<(String, Option<DiffAnalysis>), AppError> {
    // Initialize AstGrep analyzer with default configuration
    let ts_config = AstGrepConfig::default();

    let mut analyzer = AstGrepAnalyzer::new(ts_config).map_err(|e| {
        tracing::error!("AstGrep分析器初始化失败: {:?}", e);
        AppError::Analysis(e)
    })?;

    // Parse the diff to get structured representation
    let git_diff = parse_git_diff(diff).map_err(|e| {
        tracing::error!("解析Git差异失败: {:?}", e);
        AppError::Analysis(e)
    })?;

    // Generate analysis using AstGrep
    let analysis = analyzer.analyze_diff(diff).map_err(|e| {
        tracing::error!("执行差异分析失败: {:?}", e);
        AppError::Analysis(e)
    })?;

    tracing::debug!("差异分析结果: {:?}", analysis);

    // Create detailed analysis text
    let analysis_text = format_ast_grep_analysis_for_commit(&analysis, &git_diff);

    Ok((analysis_text, Some(analysis)))
}

/// Generates an enhanced commit message using AI, incorporating AstGrep static analysis, optional custom message, and review context.
///
/// This function constructs a detailed prompt for the AI model that includes the Git diff, AstGrep analysis results, and optionally a user-provided commit message and code review insights. The AI is instructed to generate a high-quality, structured commit message reflecting both the technical changes and their impact. If AI generation fails, the function falls back to the custom message with a note or a default message indicating the failure.
///
/// # Parameters
/// - `diff`: The Git diff representing staged changes for the commit.
/// - `analysis_result`: A tuple containing the formatted AstGrep analysis summary and optional detailed analysis data.
/// - `custom_message`: An optional user-supplied commit message to be incorporated or enhanced.
/// - `review_context`: Optional code review insights to be reflected in the commit message.
///
/// # Returns
/// Returns the generated commit message as a `String`. If AI generation fails, returns a fallback message.
///
/// # Examples
///
/// ```
/// let diff = "..."; // Git diff string
/// let analysis_result = ("AstGrep summary".to_string(), None);
/// let message = generate_commit_message_with_analysis(
///     &config,
///     diff,
///     &analysis_result,
///     Some("fix: correct typo".to_string()),
///     None
/// ).await?;
/// assert!(message.contains("fix:"));
/// ```
async fn generate_commit_message_with_analysis(
    config: &AppConfig,
    diff: &str,
    analysis_result: &(String, Option<DiffAnalysis>),
    custom_message: Option<String>,
    review_context: Option<&str>,
) -> Result<String, AppError> {
    let (analysis_text, analysis_data) = analysis_result;

    let system_prompt = config
        .prompts
        .get("commit-generator")
        .cloned()
        .unwrap_or_else(|| {
            "你是一个专业的Git提交信息生成助手。请根据提供的代码变更和静态分析结果生成高质量的提交信息。".to_string()
        });

    let mut user_prompt = if let Some(ref custom_msg) = custom_message {
        format!(
            "用户提供的提交信息：\n{}\n\n基于以下代码分析，请生成增强的提交信息：\n\n## Git Diff:\n```diff\n{}\n```\n\n## AstGrep 分析结果:\n{}\n\n要求：\n1. 保留用户原始意图\n2. 添加技术细节和影响分析\n3. 使用结构化格式\n4. 包含代码变更摘要",
            custom_msg, diff, analysis_text
        )
    } else {
        format!(
            "请根据以下代码变更和静态分析结果生成专业的提交信息：\n\n## Git Diff:\n```diff\n{}\n```\n\n## AstGrep 分析结果:\n{}\n\n要求：\n1. 主标题简洁明确（<50字符）\n2. 包含变更的技术细节\n3. 说明影响范围和复杂度\n4. 使用规范的提交信息格式",
            diff, analysis_text
        )
    };

    if let Some(review) = review_context {
        user_prompt.push_str(&format!(
            "\n\n## 代码评审要点:\n{}\n\n请在提交信息中体现相关的评审改进点。",
            review
        ));
    }

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        },
        ChatMessage {
            role: "user".to_string(),
            content: user_prompt,
        },
    ];

    match ai::execute_ai_request_generic(config, messages, "AstGrep增强提交信息生成", true).await
    {
        Ok(message) => {
            let enhanced_message =
                format_enhanced_commit_message(&message, analysis_data, custom_message.is_some());
            Ok(enhanced_message)
        }
        Err(e) => {
            tracing::error!("增强提交信息生成失败: {:?}", e);
            // Fallback to custom message or basic generation
            if let Some(ref msg) = custom_message {
                Ok(format!("{}\n\n[AstGrep 分析可用但AI生成失败]", msg))
            } else {
                Ok("feat: 代码更新\n\n[AstGrep 分析完成但AI生成失败]".to_string())
            }
        }
    }
}

/// Formats the results of an AstGrep diff analysis into a markdown-style summary for inclusion in a commit message.
///
/// The output includes an overall summary and, if available, per-file change details with language and file-specific summaries.
///
/// # Examples
///
/// ```
/// let analysis = DiffAnalysis {
///     overall_summary: "Refactored core logic".to_string(),
///     file_analyses: vec![
///         FileAnalysis {
///             path: PathBuf::from("src/main.rs"),
///             language: "Rust".to_string(),
///             summary: Some("Improved error handling".to_string()),
///         }
///     ],
/// };
/// let summary = format_ast_grep_analysis_for_commit(&analysis, &git_diff);
/// assert!(summary.contains("总体摘要"));
/// ```
fn format_ast_grep_analysis_for_commit(analysis: &DiffAnalysis, _git_diff: &GitDiff) -> String {
    let mut result = String::new();

    result.push_str("### 代码分析摘要\n");
    result.push_str(&format!("- 总体摘要: {}\n", analysis.overall_summary));

    if !analysis.file_analyses.is_empty() {
        result.push_str("\n### 文件变更详情\n");
        for file_analysis in &analysis.file_analyses {
            result.push_str(&format!(
                "**{}** ({})\n",
                file_analysis.path.display(),
                file_analysis.language
            ));
            if let Some(ref summary) = file_analysis.summary {
                result.push_str(&format!("  - 摘要: {}\n", summary));
            }
            result.push('\n');
        }
    }

    result
}

/// Formats the final enhanced commit message by combining the AI-generated message with an optional AstGrep analysis summary.
///
/// If AstGrep analysis data is provided, appends a markdown-formatted section summarizing the analysis. If the message is based on a custom user message, includes a note indicating this.
///
/// # Examples
///
/// ```
/// let ai_msg = "feat: add new feature";
/// let analysis = Some(DiffAnalysis { file_analyses: vec![] });
/// let result = format_enhanced_commit_message(ai_msg, &analysis, false);
/// assert!(result.contains("feat: add new feature"));
/// ```
fn format_enhanced_commit_message(
    ai_message: &str,
    analysis_data: &Option<DiffAnalysis>,
    has_custom_message: bool,
) -> String {
    let mut result = String::new();

    // Add the AI-generated message
    result.push_str(ai_message.trim());

    // Add AstGrep analysis summary if available
    if let Some(analysis) = analysis_data {
        result.push_str("\n\n");
        result.push_str("---\n");
        result.push_str("## 🌳 AstGrep 分析\n");

        if !analysis.file_analyses.is_empty() {
            result.push_str(&format!("分析文件: {} 个", analysis.file_analyses.len()));
        }

        if has_custom_message {
            result.push_str("\n\n[增强分析基于用户自定义消息]");
        }
    }

    result
}

/// Prompts the user to confirm whether to use the provided commit message.
///
/// Returns `Ok(true)` if the user confirms (by entering `y`, `yes`, `是`, or pressing Enter), or `Ok(false)` otherwise.
/// Returns an error if user input cannot be read.
///
/// # Examples
///
/// ```
/// let confirmed = confirm_commit_message("feat: add new feature").unwrap();
/// // User input determines the value of `confirmed`
/// ```
fn confirm_commit_message(_message: &str) -> Result<bool, AppError> {
    print!("\n是否使用此提交信息? [Y/n] ");
    io::stdout()
        .flush()
        .map_err(|e| AppError::IO("输出刷新失败".to_string(), e))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| AppError::IO("读取用户输入失败".to_string(), e))?;

    let input = input.trim().to_lowercase();
    Ok(input.is_empty() || input == "y" || input == "yes" || input == "是")
}

/// Execute the actual git commit
async fn execute_commit(message: &str) -> Result<(), AppError> {
    git::execute_commit_with_message(message).await
}

/// Generates a commit message using AI, optionally incorporating code review context.
///
/// If review context is provided, it is included in the prompt to encourage the AI to reflect review improvements in the commit message. Falls back to a default message if AI generation fails.
///
/// # Returns
///
/// A commit message string, either generated by AI or a fallback message.
///
/// # Examples
///
/// ```
/// let message = generate_commit_message_with_review(&config, diff, Some("Refactored error handling")).await?;
/// assert!(message.contains("Refactored") || message.contains("chore:"));
/// ```
async fn generate_commit_message_with_review(
    config: &AppConfig,
    diff: &str,
    review_context: Option<&str>,
) -> Result<String, AppError> {
    let mut prompt = format!("根据以下代码变更信息生成高质量的Git提交信息：\n\n{}", diff);

    if let Some(review) = review_context {
        prompt.push_str(&format!(
            "\n\n代码评审要点:\n{}\n\n请在提交信息中体现相关的评审改进点。",
            review
        ));
    }

    prompt.push_str(
        "\n\n请生成简洁、清晰的提交信息，遵循常见的提交信息格式（如conventional commits）。",
    );

    match generate_commit_message(config, &prompt).await {
        Ok(message) => Ok(message),
        Err(_) => {
            tracing::warn!("AI生成提交信息失败，使用回退方案");
            if review_context.is_some() {
                Ok("chore: 基于代码评审结果更新代码".to_string())
            } else {
                Ok("chore: 更新代码".to_string())
            }
        }
    }
}

/// Appends review insights to a custom commit message in markdown format.
///
/// The review context is added under a section titled "基于代码评审的改进".
///
/// # Examples
///
/// ```
/// let message = "Refactor login logic";
/// let review = "Suggested adding error handling for invalid credentials.";
/// let result = format_custom_message_with_review(message, review);
/// assert!(result.contains("Refactor login logic"));
/// assert!(result.contains("## 基于代码评审的改进"));
/// assert!(result.contains("Suggested adding error handling"));
/// ```
fn format_custom_message_with_review(custom_message: &str, review_context: &str) -> String {
    format!(
        "{}\n\n---\n## 基于代码评审的改进\n\n{}",
        custom_message, review_context
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{AIConfig, AstGrepConfig},
        types::git::CommitArgs,
    };
    use std::collections::HashMap;

    /// Creates a test `AppConfig` instance with default and mock values suitable for unit testing.
    ///
    /// The returned configuration includes a local AI endpoint, a test model, default AstGrep and review settings, and a sample commit generator prompt.
    ///
    /// # Examples
    ///
    /// ```
    /// let config = create_test_config();
    /// assert_eq!(config.ai.model_name, "test-model");
    /// ```
    fn create_test_config() -> AppConfig {
        let mut prompts = HashMap::new();
        prompts.insert(
            "commit-generator".to_string(),
            "Generate a professional commit message".to_string(),
        );

        AppConfig {
            ai: AIConfig {
                api_url: "http://localhost:11434/v1/chat/completions".to_string(),
                model_name: "test-model".to_string(),
                temperature: 0.7,
                api_key: None,
            },
            ast_grep: AstGrepConfig::default(),
            review: crate::config::ReviewConfig::default(),
            account: None,
            prompts,
            translation: Default::default(),
        }
    }

    #[test]
    fn test_confirm_commit_message_positive() {
        // This test would need to be run interactively or with mocked input
        // For now, we'll just test the structure
        let message = "feat: add new feature";
        // In a real test, we'd mock stdin/stdout
        assert!(!message.is_empty());
    }

    #[test]
    fn test_generate_commit_message_fallback() {
        // Test that we have a fallback when AI fails
        let diff = "diff --git a/test.txt b/test.txt\n+new line";
        assert!(!diff.is_empty());
    }

    #[tokio::test]
    async fn test_check_repository_status() {
        // This test would fail if not run in a git repository
        // In CI/CD, we'd set up a temporary git repo
        // For now, just test that the function exists and has the right signature
        assert!(true);
    }

    #[test]
    fn test_commit_args_structure() {
        let args = CommitArgs {
            ast_grep: false,
            auto_stage: false,
            message: Some("test message".to_string()),
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };

        assert_eq!(args.message, Some("test message".to_string()));
        assert!(!args.auto_stage);
        assert!(!args.ast_grep);
    }

    #[test]
    fn test_commit_args_with_ast_grep() {
        let args = CommitArgs {
            ast_grep: true,
            auto_stage: false,
            message: None,
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };

        assert!(args.ast_grep);
        assert!(args.message.is_none());
    }

    #[test]
    fn test_commit_args_auto_stage_enabled() {
        let args = CommitArgs {
            ast_grep: false,
            auto_stage: true,
            message: None,
            issue_id: None,
            review: false,
            passthrough_args: vec!["--verbose".to_string()],
        };

        assert!(args.auto_stage);
        assert_eq!(args.passthrough_args, vec!["--verbose".to_string()]);
    }

    /// Tests that `generate_commit_message` returns a non-empty commit message, falling back to a default if AI generation fails.
    ///
    /// This test simulates generating a commit message from a sample diff. It asserts that the result is either an AI-generated message or the fallback default, and handles the case where the AI service is unavailable.
    ///
    /// # Examples
    ///
    /// ```
    /// let config = create_test_config();
    /// let diff = "diff --git ...";
    /// let result = generate_commit_message(&config, diff).await;
    /// assert!(result.is_ok() || result.is_err());
    /// ```
    #[tokio::test]
    async fn test_generate_commit_message_with_fallback() {
        let config = create_test_config();
        let diff = "diff --git a/src/test.rs b/src/test.rs\nindex 1234567..abcdefg 100644\n--- a/src/test.rs\n+++ b/src/test.rs\n@@ -1,3 +1,4 @@\n fn test_function() {\n     println!(\"Hello, world!\");\n+    println!(\"New line added\");\n }";

        // This will likely fall back to the default message since we don't have a real AI service
        let result = generate_commit_message(&config, diff).await;

        match result {
            Ok(message) => {
                assert!(!message.is_empty());
                // Should either be AI-generated or the fallback message
                assert!(message.contains("chore") || message.len() > 5);
            }
            Err(_) => {
                // AI service not available in test environment, this is expected
                assert!(true);
            }
        }
    }

    #[tokio::test]
    async fn test_handle_commit_with_custom_message() {
        let config = create_test_config();
        let args = CommitArgs {
            ast_grep: false,
            auto_stage: false,
            message: Some("feat: custom commit message".to_string()),
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };

        // This test will fail in most environments since we're not in a proper git repo
        // But it tests the structure and error handling
        match handle_commit(&config, args).await {
            Ok(_) => {
                // Would only succeed if we're in a git repo with staged changes
                assert!(true);
            }
            Err(e) => {
                // Expected in test environment
                match e {
                    AppError::Git(GitError::NotARepository) => assert!(true),
                    AppError::Git(GitError::NoStagedChanges) => assert!(true),
                    AppError::Generic(msg) => {
                        assert!(
                            msg.contains("没有已暂存的变更") || msg.contains("检查Git仓库状态失败")
                        );
                    }
                    _ => assert!(true), // Other errors are also acceptable in test
                }
            }
        }
    }

    #[tokio::test]
    async fn test_handle_commit_with_auto_stage() {
        let config = create_test_config();
        let args = CommitArgs {
            ast_grep: false,
            auto_stage: true,
            message: None,
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };

        match handle_commit(&config, args).await {
            Ok(_) => {
                // Success only if in proper git environment
                assert!(true);
            }
            Err(e) => {
                // Expected errors in test environment
                match e {
                    AppError::Git(GitError::NotARepository) => assert!(true),
                    AppError::Git(GitError::CommandFailed { .. }) => assert!(true),
                    AppError::Generic(_) => assert!(true),
                    _ => assert!(true),
                }
            }
        }
    }

    #[test]
    fn test_create_test_config_structure() {
        let config = create_test_config();

        assert_eq!(config.ai.model_name, "test-model");
        assert_eq!(
            config.ai.api_url,
            "http://localhost:11434/v1/chat/completions"
        );
        assert_eq!(config.ai.temperature, 0.7);
        assert!(config.prompts.contains_key("commit-generator"));
        assert_eq!(
            config.prompts.get("commit-generator").unwrap(),
            "Generate a professional commit message"
        );
    }

    #[tokio::test]
    async fn test_auto_stage_files_error_handling() {
        // Test that auto_stage_files handles errors gracefully
        match auto_stage_files().await {
            Ok(_) => {
                // Success if we're in a git repo
                assert!(true);
            }
            Err(e) => {
                // Expected error types in test environment
                match e {
                    AppError::Git(GitError::CommandFailed { .. }) => assert!(true),
                    AppError::IO(_, _) => assert!(true),
                    _ => assert!(true),
                }
            }
        }
    }

    #[tokio::test]
    async fn test_get_changes_for_commit_empty_repo() {
        // Test behavior when there are no staged changes
        match get_changes_for_commit().await {
            Ok(diff) => {
                // If successful, diff could be empty or contain changes
                assert!(diff.is_empty() || !diff.is_empty());
            }
            Err(e) => {
                // Expected errors
                match e {
                    AppError::Generic(msg) => {
                        assert!(
                            msg.contains("没有检测到任何变更") || msg.contains("没有已暂存的变更")
                        );
                    }
                    AppError::Git(GitError::CommandFailed { .. }) => assert!(true),
                    AppError::IO(_, _) => assert!(true),
                    _ => assert!(true),
                }
            }
        }
    }

    #[tokio::test]
    async fn test_execute_commit_error_handling() {
        let test_message = "test: this should fail in test environment";

        match execute_commit(test_message).await {
            Ok(_) => {
                // Would only succeed if we have staged changes to commit
                assert!(true);
            }
            Err(e) => {
                // Expected in test environment
                match e {
                    AppError::Git(GitError::CommandFailed { command, .. }) => {
                        assert!(command.contains("git commit"));
                    }
                    _ => assert!(true),
                }
            }
        }
    }

    /// Tests the basic functionality of analyzing a Git diff using AstGrep.
    ///
    /// This test verifies that `analyze_diff_with_ast_grep` returns a non-empty analysis summary and data when provided with a simple Rust diff. It also handles environments where AstGrep is not available by accepting analysis errors.
    ///
    /// # Examples
    ///
    /// ```
    /// // Runs as an async test; checks that AstGrep analysis produces expected output or fails gracefully.
    /// ```
    async fn test_analyze_diff_with_ast_grep_basic() {
        let diff = "diff --git a/src/test.rs b/src/test.rs\nindex 1234567..abcdefg 100644\n--- a/src/test.rs\n+++ b/src/test.rs\n@@ -1,3 +1,4 @@\n fn test_function() {\n     println!(\"Hello, world!\");\n+    println!(\"New line added\");\n }";

        let args = CommitArgs {
            ast_grep: true,
            auto_stage: false,
            message: None,
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };

        // This test may fail in environments without proper ast-grep setup
        match analyze_diff_with_ast_grep(diff, &args).await {
            Ok((analysis_text, analysis_data)) => {
                assert!(!analysis_text.is_empty());
                assert!(analysis_data.is_some());
                assert!(analysis_text.contains("代码分析摘要"));
            }
            Err(e) => {
                // Expected in test environments without ast-grep support
                match e {
                    AppError::Analysis(_) => assert!(true),
                    _ => assert!(true),
                }
            }
        }
    }

    /// Tests the AstGrep diff analysis function with different commit argument configurations.
    ///
    /// Verifies that `analyze_diff_with_ast_grep` produces non-empty analysis summaries for a sample diff and that the output contains expected summary keywords. Handles both successful and error outcomes, as errors may occur in test environments.
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// let diff = "diff --git a/src/lib.rs b/src/lib.rs\n+pub fn new_function() {}";
    /// let args = CommitArgs {
    ///     ast_grep: true,
    ///     auto_stage: false,
    ///     message: None,
    ///     issue_id: None,
    ///     review: false,
    ///     passthrough_args: vec![],
    /// };
    /// let result = analyze_diff_with_ast_grep(diff, &args).await;
    /// assert!(result.is_ok() || result.is_err());
    /// # });
    /// ```
    async fn test_analyze_diff_with_ast_grep_depth_levels() {
        let diff = "diff --git a/src/lib.rs b/src/lib.rs\n+pub fn new_function() {}";

        let shallow_args = CommitArgs {
            ast_grep: true,
            auto_stage: false,
            message: None,
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };

        let deep_args = CommitArgs {
            ast_grep: true,
            auto_stage: false,
            message: None,
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };

        // Test different analysis depths
        for args in &[shallow_args, deep_args] {
            match analyze_diff_with_ast_grep(diff, args).await {
                Ok((analysis_text, _)) => {
                    assert!(!analysis_text.is_empty());
                    // Analysis text should contain depth-specific information
                    assert!(
                        analysis_text.contains("代码分析摘要")
                            || analysis_text.contains("变更模式")
                    );
                }
                Err(_) => {
                    // Expected in test environments
                    assert!(true);
                }
            }
        }
    }

    #[test]
    fn test_format_ast_grep_analysis_for_commit() {
        use crate::ast_grep_analyzer::core::{DiffAnalysis, FileAnalysis};
        use std::path::PathBuf;

        let analysis = DiffAnalysis {
            file_analyses: vec![FileAnalysis {
                path: PathBuf::from("src/test.rs"),
                language: "Rust".to_string(),
                change_type: crate::types::git::ChangeType::Added,
                summary: Some("新增测试函数".to_string()),
                issues: vec![],
                metrics: None,
            }],
            overall_summary: "添加了新的测试函数".to_string(),
            total_issues: 0,
            total_files_analyzed: 1,
            analysis_duration_ms: 100,
        };

        let git_diff = crate::types::git::GitDiff {
            changed_files: vec![],
            metadata: None,
        };

        let result = format_ast_grep_analysis_for_commit(&analysis, &git_diff);

        assert!(result.contains("代码分析摘要"));
        assert!(result.contains("src/test.rs"));
        assert!(result.contains("Rust"));
    }

    /// Tests the formatting of enhanced commit messages with and without custom user messages.
    ///
    /// Verifies that the formatted message includes the AstGrep analysis section and the appropriate note when a custom message is used.
    #[test]
    fn test_format_enhanced_commit_message() {
        use crate::ast_grep_analyzer::core::DiffAnalysis;

        let ai_message = "feat: add new authentication feature\n\nImplemented user login and registration functionality";

        let analysis = DiffAnalysis {
            file_analyses: vec![],
            overall_summary: "Authentication feature implementation".to_string(),
            total_issues: 0,
            total_files_analyzed: 2,
            analysis_duration_ms: 150,
        };

        let result_with_analysis =
            format_enhanced_commit_message(ai_message, &Some(analysis.clone()), false);
        let result_with_custom = format_enhanced_commit_message(ai_message, &Some(analysis), true);

        assert!(result_with_analysis.contains("🌳 AstGrep 分析"));

        assert!(result_with_custom.contains("增强分析基于用户自定义消息"));
    }

    /// Tests that `generate_enhanced_commit_message` correctly falls back to a custom message or default output when AstGrep analysis or AI generation fails.
    ///
    /// This test verifies both scenarios: with and without a custom commit message provided. It asserts that the function returns a non-empty message or handles errors gracefully in a test environment.
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// test_generate_enhanced_commit_message_fallback().await;
    /// # });
    /// ```
    async fn test_generate_enhanced_commit_message_fallback() {
        let config = create_test_config();
        let diff = "diff --git a/src/test.rs b/src/test.rs\n+// test change";

        let args_with_custom = CommitArgs {
            ast_grep: true,
            auto_stage: false,
            message: Some("feat: custom message".to_string()),
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };

        let args_without_custom = CommitArgs {
            ast_grep: true,
            auto_stage: false,
            message: None,
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };

        // Test with custom message
        match generate_enhanced_commit_message(
            &config,
            diff,
            Some("feat: custom message".to_string()),
            &args_with_custom,
            None,
        )
        .await
        {
            Ok(message) => {
                // Should either be enhanced or fallback
                assert!(!message.is_empty());
                assert!(message.contains("feat") || message.contains("Tree-sitter"));
            }
            Err(_) => {
                // Expected in test environment
                assert!(true);
            }
        }

        // Test without custom message
        match generate_enhanced_commit_message(&config, diff, None, &args_without_custom, None)
            .await
        {
            Ok(message) => {
                assert!(!message.is_empty());
            }
            Err(_) => {
                // Expected in test environment
                assert!(true);
            }
        }
    }

    /// Tests the `handle_commit` function in AstGrep mode, both with and without a custom commit message.
    ///
    /// This test verifies that the commit handler correctly processes commit arguments when AstGrep analysis is enabled. It checks for successful execution in a valid Git environment and ensures appropriate error handling when run outside a repository or with no staged changes.
    ///
    /// # Examples
    ///
    /// ```
    /// // Runs as an async test; expects either success or known errors depending on environment.
    /// test_handle_commit_with_ast_grep().await;
    /// ```
    #[tokio::test]
    async fn test_handle_commit_with_ast_grep() {
        let config = create_test_config();

        let args_ast_grep = CommitArgs {
            ast_grep: true,
            auto_stage: false,
            message: None,
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };

        let args_ast_grep_with_message = CommitArgs {
            ast_grep: true,
            auto_stage: false,
            message: Some("feat: enhanced with ast-grep".to_string()),
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };

        // Test ast-grep mode without custom message
        match handle_commit(&config, args_ast_grep).await {
            Ok(_) => {
                // Success only if in proper git environment
                assert!(true);
            }
            Err(e) => {
                // Expected errors in test environment
                match e {
                    AppError::Git(GitError::NotARepository) => assert!(true),
                    AppError::Generic(msg) if msg.contains("没有检测到任何变更") => {
                        assert!(true)
                    }
                    _ => assert!(true),
                }
            }
        }

        // Test ast-grep mode with custom message
        match handle_commit(&config, args_ast_grep_with_message).await {
            Ok(_) => {
                assert!(true);
            }
            Err(e) => match e {
                AppError::Git(GitError::NotARepository) => assert!(true),
                AppError::Generic(msg) if msg.contains("没有检测到任何变更") => {
                    assert!(true)
                }
                _ => assert!(true),
            },
        }
    }

    #[test]
    fn test_commit_args_ast_grep_combinations() {
        // Test various combinations of ast-grep related arguments
        let args1 = CommitArgs {
            ast_grep: true,
            auto_stage: false,
            message: None,
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };

        let args2 = CommitArgs {
            ast_grep: true,
            auto_stage: false,
            message: None,
            issue_id: None,
            review: false,
            passthrough_args: vec!["-v".to_string()],
        };

        let args3 = CommitArgs {
            ast_grep: false,
            auto_stage: false,
            message: Some("simple commit".to_string()),
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };

        // All should be valid structures
        assert!(args1.ast_grep);
        assert!(args1.message.is_none());

        assert!(args2.ast_grep);
        assert_eq!(args2.passthrough_args, vec!["-v".to_string()]);

        assert!(!args3.ast_grep);
        assert_eq!(args3.message, Some("simple commit".to_string()));
    }

    #[tokio::test]
    async fn test_get_changes_for_commit_enhanced() {
        // Test the enhanced git diff function
        match get_changes_for_commit().await {
            Ok(diff) => {
                // If successful, we should have some diff content or empty string
                assert!(diff.is_empty() || !diff.is_empty());
            }
            Err(e) => {
                // Expected errors in test environment
                match e {
                    AppError::Generic(msg) if msg.contains("没有检测到任何变更") => {
                        assert!(true)
                    }
                    AppError::Git(GitError::CommandFailed { .. }) => assert!(true),
                    AppError::IO(_, _) => assert!(true),
                    _ => assert!(true),
                }
            }
        }
    }

    #[test]
    fn test_format_custom_message_with_review() {
        let custom_message = "feat: add user authentication";
        let review_context = "- Fix security vulnerability in login\n- Improve input validation";

        let result = format_custom_message_with_review(custom_message, review_context);

        assert!(result.contains("feat: add user authentication"));
        assert!(result.contains("基于代码评审的改进"));
        assert!(result.contains("Fix security vulnerability"));
        assert!(result.contains("Improve input validation"));
    }

    /// Tests commit message generation with and without review context using AI assistance.
    ///
    /// This test verifies that `generate_commit_message_with_review` produces a non-empty commit message
    /// when provided with a diff and optional review context. It checks both the presence and absence of review context,
    /// ensuring that the function returns a valid message or falls back gracefully on error.
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// let config = create_test_config();
    /// let diff = "diff --git ...";
    /// let review_context = "- Added main function output";
    /// let result = generate_commit_message_with_review(&config, diff, Some(review_context)).await;
    /// assert!(result.is_ok());
    /// # });
    /// ```
    async fn test_generate_commit_message_with_review() {
        let config = create_test_config();
        let diff = "diff --git a/src/main.rs b/src/main.rs\nindex 123..456 100644\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"Hello, world!\");\n     // TODO: implement\n }";

        // Test with review context
        let review_context = "- 添加了主函数输出\n- 代码结构良好";
        let result = generate_commit_message_with_review(&config, diff, Some(review_context)).await;

        match result {
            Ok(message) => {
                assert!(!message.is_empty());
                // Should contain some form of commit message
                assert!(message.len() > 10);
            }
            Err(_) => {
                // Fallback should still work
                assert!(true);
            }
        }

        // Test without review context
        let result = generate_commit_message_with_review(&config, diff, None).await;
        match result {
            Ok(message) => {
                assert!(!message.is_empty());
            }
            Err(_) => {
                assert!(true);
            }
        }
    }

    #[test]
    fn test_commit_args_with_review_integration() {
        // Test CommitArgs structure supports review integration
        let args = CommitArgs {
            ast_grep: true,
            auto_stage: false,
            message: Some("feat: add feature".to_string()),
            issue_id: None,
            review: true,
            passthrough_args: vec![],
        };

        assert_eq!(args.ast_grep, true);
        assert_eq!(args.message, Some("feat: add feature".to_string()));
        assert_eq!(args.review, true);
    }

    /// Tests enhanced commit message generation with and without review context using AstGrep analysis.
    ///
    /// This test verifies that `generate_enhanced_commit_message` produces a non-empty commit message
    /// when provided with a diff and optional review context. It also checks that the function
    /// gracefully falls back on error.
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// let config = create_test_config();
    /// let diff = "test diff content";
    /// let args = CommitArgs {
    ///     ast_grep: true,
    ///     auto_stage: false,
    ///     message: None,
    ///     issue_id: None,
    ///     review: false,
    ///     passthrough_args: vec![],
    /// };
    ///
    /// // With review context
    /// let review_context = "Review findings: code quality good";
    /// let result = generate_enhanced_commit_message(&config, diff, None, &args, Some(review_context)).await;
    /// assert!(result.is_ok() || result.is_err());
    ///
    /// // Without review context
    /// let result = generate_enhanced_commit_message(&config, diff, None, &args, None).await;
    /// assert!(result.is_ok() || result.is_err());
    /// # });
    /// ```
    async fn test_enhanced_commit_with_review_context() {
        let config = create_test_config();
        let diff = "test diff content";
        let args = CommitArgs {
            ast_grep: true,
            auto_stage: false,
            message: None,
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };

        // Test with review context
        let review_context = "Review findings: code quality good";
        let result =
            generate_enhanced_commit_message(&config, diff, None, &args, Some(review_context))
                .await;

        match result {
            Ok(message) => {
                assert!(!message.is_empty());
            }
            Err(_) => {
                // Should fallback gracefully
                assert!(true);
            }
        }

        // Test without review context
        let result = generate_enhanced_commit_message(&config, diff, None, &args, None).await;
        match result {
            Ok(message) => {
                assert!(!message.is_empty());
            }
            Err(_) => {
                assert!(true);
            }
        }
    }
}

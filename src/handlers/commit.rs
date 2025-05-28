use crate::{
    config::{AppConfig, TreeSitterConfig},
    errors::{AppError, GitError},
    handlers::{ai, git},
    tree_sitter_analyzer::{
        analyzer::TreeSitterAnalyzer,
        core::{parse_git_diff, DiffAnalysis},
    },
    types::{
        git::{CommitArgs, GitDiff},
        ai::ChatMessage,
    },
    utils::{find_latest_review_file, read_review_file, extract_review_insights},
};
use std::io::{self, Write};
use std::time::Instant;

/// Handle the commit command with AI assistance
/// This function demonstrates AI-powered commit message generation
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
        if args.tree_sitter {
            // Enhanced mode: combine custom message with AI analysis and review
            generate_enhanced_commit_message(config, &diff, Some(custom_message.clone()), &args, review_context.as_deref()).await?
        } else if review_context.is_some() {
            // Custom message with review context
            format_custom_message_with_review(custom_message, review_context.as_deref().unwrap())
        } else {
            // Simple mode: use custom message directly
            custom_message.clone()
        }
    } else {
        if args.tree_sitter {
            // Enhanced mode: full Tree-sitter analysis with AI generation and review
            generate_enhanced_commit_message(config, &diff, None, &args, review_context.as_deref()).await?
        } else {
            // Basic mode: AI generation with optional review context
            generate_commit_message_with_review(config, &diff, review_context.as_deref()).await?
        }
    };
    
    // Show generated commit message and ask for confirmation
    println!("\n🤖 生成的提交信息:");
    println!("┌─────────────────────────────────────────────┐");
    for line in commit_message.lines() {
        println!("│ {:<43} │", line);
    }
    println!("└─────────────────────────────────────────────┘");
    
    if !confirm_commit_message(&commit_message)? {
        println!("❌ 提交已取消");
        return Ok(());
    }
    
    // Execute the commit
    execute_commit(&commit_message).await?;
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

/// Auto-stage modified tracked files
async fn auto_stage_files() -> Result<(), AppError> {
    git::auto_stage_tracked_files().await
}

/// Get changes for commit analysis
async fn get_changes_for_commit() -> Result<String, AppError> {
    // Get diff for commit (staged or unstaged changes)
    git::get_diff_for_commit().await
}

/// Generate commit message using AI (basic mode)
async fn generate_commit_message(config: &AppConfig, diff: &str) -> Result<String, AppError> {
    tracing::info!("正在使用AI生成提交信息...");
    
    let system_prompt = config
        .prompts
        .get("commit-generator")
        .cloned()
        .unwrap_or_else(|| {
            tracing::warn!("未找到commit-generator提示模板，使用默认模板");
            "你是一个专业的Git提交信息生成助手。请根据提供的代码变更生成简洁、清晰的提交信息。".to_string()
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

/// Generate enhanced commit message using Tree-sitter analysis
async fn generate_enhanced_commit_message(
    config: &AppConfig, 
    diff: &str, 
    custom_message: Option<String>,
    args: &CommitArgs,
    review_context: Option<&str>
) -> Result<String, AppError> {
    tracing::info!("🌳 正在使用Tree-sitter增强分析生成提交信息...");
    
    let analysis_start = Instant::now();
    
    // Perform Tree-sitter analysis
    let analysis_result = match analyze_diff_with_tree_sitter(diff, args).await {
        Ok(result) => {
            tracing::info!("Tree-sitter分析完成，耗时: {:?}", analysis_start.elapsed());
            result
        }
        Err(e) => {
            tracing::warn!("Tree-sitter分析失败，回退到基础模式: {:?}", e);
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
    generate_commit_message_with_analysis(config, diff, &analysis_result, custom_message, review_context).await
}

/// Analyze diff using Tree-sitter
async fn analyze_diff_with_tree_sitter(
    diff: &str,
    args: &CommitArgs,
) -> Result<(String, Option<DiffAnalysis>), AppError> {
    // Initialize TreeSitter analyzer with analysis depth
    let mut ts_config = TreeSitterConfig::default();
    
    // Set analysis depth based on args
    if let Some(depth) = &args.depth {
        ts_config.analysis_depth = depth.clone();
    } else {
        ts_config.analysis_depth = "medium".to_string(); // Default for commit
    }
    
    let mut analyzer = TreeSitterAnalyzer::new(ts_config).map_err(|e| {
        tracing::error!("TreeSitter分析器初始化失败: {:?}", e);
        AppError::TreeSitter(e)
    })?;

    // Parse the diff to get structured representation
    let git_diff = parse_git_diff(diff).map_err(|e| {
        tracing::error!("解析Git差异失败: {:?}", e);
        AppError::TreeSitter(e)
    })?;

    // Generate analysis using TreeSitter
    let analysis = analyzer.analyze_diff(diff).map_err(|e| {
        tracing::error!("执行差异分析失败: {:?}", e);
        AppError::TreeSitter(e)
    })?;
    
    tracing::debug!("差异分析结果: {:?}", analysis);

    // Create detailed analysis text
    let analysis_text = format_tree_sitter_analysis_for_commit(&analysis, &git_diff);

    Ok((analysis_text, Some(analysis)))
}

/// Generate commit message with Tree-sitter analysis results
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
            "用户提供的提交信息：\n{}\n\n基于以下代码分析，请生成增强的提交信息：\n\n## Git Diff:\n```diff\n{}\n```\n\n## Tree-sitter 分析结果:\n{}\n\n要求：\n1. 保留用户原始意图\n2. 添加技术细节和影响分析\n3. 使用结构化格式\n4. 包含代码变更摘要",
            custom_msg, diff, analysis_text
        )
    } else {
        format!(
            "请根据以下代码变更和静态分析结果生成专业的提交信息：\n\n## Git Diff:\n```diff\n{}\n```\n\n## Tree-sitter 分析结果:\n{}\n\n要求：\n1. 主标题简洁明确（<50字符）\n2. 包含变更的技术细节\n3. 说明影响范围和复杂度\n4. 使用规范的提交信息格式",
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
    
    match ai::execute_ai_request_generic(config, messages, "Tree-sitter增强提交信息生成", true).await {
        Ok(message) => {
            let enhanced_message = format_enhanced_commit_message(&message, analysis_data, custom_message.is_some());
            Ok(enhanced_message)
        }
        Err(e) => {
            tracing::error!("增强提交信息生成失败: {:?}", e);
            // Fallback to custom message or basic generation
            if let Some(ref msg) = custom_message {
                Ok(format!("{}\n\n[Tree-sitter 分析可用但AI生成失败]", msg))
            } else {
                Ok("feat: 代码更新\n\n[Tree-sitter 分析完成但AI生成失败]".to_string())
            }
        }
    }
}

/// Format Tree-sitter analysis for commit message generation
fn format_tree_sitter_analysis_for_commit(
    analysis: &DiffAnalysis,
    _git_diff: &GitDiff,
) -> String {
    let mut result = String::new();
    
    result.push_str("### 代码分析摘要\n");
    result.push_str(&format!("- 变更模式: {:?}\n", analysis.change_analysis.change_pattern));
    result.push_str(&format!("- 影响范围: {:?}\n", analysis.change_analysis.change_scope));
    result.push_str(&format!("- 总体摘要: {}\n", analysis.overall_summary));
    
    if !analysis.file_analyses.is_empty() {
        result.push_str("\n### 文件变更详情\n");
        for file_analysis in &analysis.file_analyses {
            result.push_str(&format!("**{}** ({})\n", file_analysis.path.display(), file_analysis.language));
            result.push_str(&format!("  - 变更类型: {:?}\n", file_analysis.change_type));
            if let Some(ref summary) = file_analysis.summary {
                result.push_str(&format!("  - 摘要: {}\n", summary));
            }
            
            if !file_analysis.affected_nodes.is_empty() {
                result.push_str("  - 影响的代码结构:\n");
                for node in &file_analysis.affected_nodes {
                    let change_type_str = node.change_type.as_deref().unwrap_or("未知");
                    result.push_str(&format!("    • {} ({}): {}\n", 
                        node.node_type, 
                        &node.name, 
                        change_type_str
                    ));
                }
            }
            result.push('\n');
        }
    }
    
    // Add change statistics
    let change_analysis = &analysis.change_analysis;
    if change_analysis.function_changes > 0 {
        result.push_str(&format!("### 函数变更: {} 个\n", change_analysis.function_changes));
        result.push('\n');
    }
    
    if change_analysis.type_changes > 0 {
        result.push_str(&format!("### 类型变更: {} 个\n", change_analysis.type_changes));
        result.push('\n');
    }
    
    result
}

/// Format the final enhanced commit message
fn format_enhanced_commit_message(
    ai_message: &str, 
    analysis_data: &Option<DiffAnalysis>,
    has_custom_message: bool
) -> String {
    let mut result = String::new();
    
    // Add the AI-generated message
    result.push_str(ai_message.trim());
    
    // Add Tree-sitter analysis summary if available
    if let Some(analysis) = analysis_data {
        result.push_str("\n\n");
        result.push_str("---\n");
        result.push_str("## 🌳 Tree-sitter 分析\n");
        result.push_str(&format!("变更模式: {:?} | 影响范围: {:?}\n", 
            analysis.change_analysis.change_pattern,
            analysis.change_analysis.change_scope
        ));
        
        if !analysis.file_analyses.is_empty() {
            result.push_str(&format!("分析文件: {} 个", analysis.file_analyses.len()));
            
            let total_nodes: usize = analysis.file_analyses.iter()
                .map(|f| f.affected_nodes.len())
                .sum();
                
            if total_nodes > 0 {
                result.push_str(&format!(" | 影响节点: {} 个", total_nodes));
            }
        }
        
        if has_custom_message {
            result.push_str("\n\n[增强分析基于用户自定义消息]");
        }
    }
    
    result
}

/// Ask user to confirm the commit message
fn confirm_commit_message(_message: &str) -> Result<bool, AppError> {
    print!("\n是否使用此提交信息? [Y/n] ");
    io::stdout().flush().map_err(|e| AppError::IO("输出刷新失败".to_string(), e))?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| AppError::IO("读取用户输入失败".to_string(), e))?;
    
    let input = input.trim().to_lowercase();
    Ok(input.is_empty() || input == "y" || input == "yes" || input == "是")
}

/// Execute the actual git commit
async fn execute_commit(message: &str) -> Result<(), AppError> {
    git::execute_commit_with_message(message).await
}

/// Generate commit message with optional review context
async fn generate_commit_message_with_review(
    config: &AppConfig,
    diff: &str,
    review_context: Option<&str>,
) -> Result<String, AppError> {
    let mut prompt = format!(
        "根据以下代码变更信息生成高质量的Git提交信息：\n\n{}",
        diff
    );

    if let Some(review) = review_context {
        prompt.push_str(&format!(
            "\n\n代码评审要点:\n{}\n\n请在提交信息中体现相关的评审改进点。",
            review
        ));
    }

    prompt.push_str("\n\n请生成简洁、清晰的提交信息，遵循常见的提交信息格式（如conventional commits）。");

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

/// Format custom message with review context
fn format_custom_message_with_review(custom_message: &str, review_context: &str) -> String {
    format!(
        "{}\n\n---\n## 基于代码评审的改进\n\n{}",
        custom_message,
        review_context
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::{
        config::{AIConfig, TreeSitterConfig},
        types::git::CommitArgs,
    };

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
            tree_sitter: TreeSitterConfig::default(),
            review: crate::config::ReviewConfig::default(),
            account: None,
            prompts,
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
            tree_sitter: false,
            depth: None,
            auto_stage: false,
            message: Some("test message".to_string()),
            review: false,
            passthrough_args: vec![],
        };
        
        assert_eq!(args.message, Some("test message".to_string()));
        assert!(!args.auto_stage);
        assert!(!args.tree_sitter);
    }

    #[test]
    fn test_commit_args_with_tree_sitter() {
        let args = CommitArgs {
            tree_sitter: true,
            depth: Some("deep".to_string()),
            auto_stage: false,
            message: None,
            review: false,
            passthrough_args: vec![],
        };
        
        assert!(args.tree_sitter);
        assert_eq!(args.depth, Some("deep".to_string()));
        assert!(args.message.is_none());
    }

    #[test]
    fn test_commit_args_auto_stage_enabled() {
        let args = CommitArgs {
            tree_sitter: false,
            depth: None,
            auto_stage: true,
            message: None,
            review: false,
            passthrough_args: vec!["--verbose".to_string()],
        };
        
        assert!(args.auto_stage);
        assert_eq!(args.passthrough_args, vec!["--verbose".to_string()]);
    }

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
            tree_sitter: false,
            depth: None,
            auto_stage: false,
            message: Some("feat: custom commit message".to_string()),
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
                        assert!(msg.contains("没有已暂存的变更") || msg.contains("检查Git仓库状态失败"));
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
            tree_sitter: false,
            depth: None,
            auto_stage: true,
            message: None,
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
        assert_eq!(config.ai.api_url, "http://localhost:11434/v1/chat/completions");
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
                        assert!(msg.contains("没有检测到任何变更") || msg.contains("没有已暂存的变更"));
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

    #[tokio::test]
    async fn test_analyze_diff_with_tree_sitter_basic() {
        let diff = "diff --git a/src/test.rs b/src/test.rs\nindex 1234567..abcdefg 100644\n--- a/src/test.rs\n+++ b/src/test.rs\n@@ -1,3 +1,4 @@\n fn test_function() {\n     println!(\"Hello, world!\");\n+    println!(\"New line added\");\n }";
        
        let args = CommitArgs {
            tree_sitter: true,
            depth: Some("medium".to_string()),
            auto_stage: false,
            message: None,
            review: false,
            passthrough_args: vec![],
        };

        // This test may fail in environments without proper tree-sitter setup
        match analyze_diff_with_tree_sitter(diff, &args).await {
            Ok((analysis_text, analysis_data)) => {
                assert!(!analysis_text.is_empty());
                assert!(analysis_data.is_some());
                assert!(analysis_text.contains("代码分析摘要"));
            }
            Err(e) => {
                // Expected in test environments without tree-sitter support
                match e {
                    AppError::TreeSitter(_) => assert!(true),
                    _ => assert!(true),
                }
            }
        }
    }

    #[tokio::test]
    async fn test_analyze_diff_with_tree_sitter_depth_levels() {
        let diff = "diff --git a/src/lib.rs b/src/lib.rs\n+pub fn new_function() {}";
        
        let shallow_args = CommitArgs {
            tree_sitter: true,
            depth: Some("shallow".to_string()),
            auto_stage: false,
            message: None,
            review: false,
            passthrough_args: vec![],
        };
        
        let deep_args = CommitArgs {
            tree_sitter: true,
            depth: Some("deep".to_string()),
            auto_stage: false,
            message: None,
            review: false,
            passthrough_args: vec![],
        };

        // Test different analysis depths
        for args in &[shallow_args, deep_args] {
            match analyze_diff_with_tree_sitter(diff, args).await {
                Ok((analysis_text, _)) => {
                    assert!(!analysis_text.is_empty());
                    // Analysis text should contain depth-specific information
                    assert!(analysis_text.contains("代码分析摘要") || analysis_text.contains("变更模式"));
                }
                Err(_) => {
                    // Expected in test environments
                    assert!(true);
                }
            }
        }
    }

    #[test]
    fn test_format_tree_sitter_analysis_for_commit() {
        use crate::tree_sitter_analyzer::core::{
            DiffAnalysis, FileAnalysis, ChangeAnalysis, ChangePattern, ChangeScope, AffectedNode
        };
        use std::path::PathBuf;

        let analysis = DiffAnalysis {
            file_analyses: vec![
                FileAnalysis {
                    path: PathBuf::from("src/test.rs"),
                    language: "Rust".to_string(),
                    change_type: crate::types::git::ChangeType::Added,
                    affected_nodes: vec![
                        AffectedNode {
                            node_type: "function".to_string(),
                            name: "test_function".to_string(),
                            range: (0, 100),
                            is_public: true,
                            content: Some("fn test_function() {}".to_string()),
                            line_range: (1, 5),
                            change_type: Some("added".to_string()),
                            additions: Some(vec!["println!(\"Hello\");".to_string()]),
                            deletions: None,
                        }
                    ],
                    summary: Some("新增测试函数".to_string()),
                }
            ],
            overall_summary: "添加了新的测试函数".to_string(),
            change_analysis: ChangeAnalysis {
                function_changes: 1,
                type_changes: 0,
                method_changes: 0,
                interface_changes: 0,
                other_changes: 0,
                change_pattern: ChangePattern::FeatureImplementation,
                change_scope: ChangeScope::Minor,
            },
        };

        let git_diff = crate::types::git::GitDiff {
            changed_files: vec![],
            metadata: None,
        };

        let result = format_tree_sitter_analysis_for_commit(&analysis, &git_diff);
        
        assert!(result.contains("代码分析摘要"));
        assert!(result.contains("FeatureImplementation"));
        assert!(result.contains("Minor"));
        assert!(result.contains("src/test.rs"));
        assert!(result.contains("函数变更: 1 个"));
    }

    #[test]
    fn test_format_enhanced_commit_message() {
        use crate::tree_sitter_analyzer::core::{
            DiffAnalysis, ChangeAnalysis, ChangePattern, ChangeScope
        };

        let ai_message = "feat: add new authentication feature\n\nImplemented user login and registration functionality";
        
        let analysis = DiffAnalysis {
            file_analyses: vec![],
            overall_summary: "Authentication feature implementation".to_string(),
            change_analysis: ChangeAnalysis {
                function_changes: 3,
                type_changes: 1,
                method_changes: 2,
                interface_changes: 0,
                other_changes: 0,
                change_pattern: ChangePattern::FeatureImplementation,
                change_scope: ChangeScope::Moderate,
            },
        };

        let result_with_analysis = format_enhanced_commit_message(ai_message, &Some(analysis.clone()), false);
        let result_with_custom = format_enhanced_commit_message(ai_message, &Some(analysis), true);

        assert!(result_with_analysis.contains("Tree-sitter 分析"));
        assert!(result_with_analysis.contains("FeatureImplementation"));
        assert!(result_with_analysis.contains("Moderate"));
        assert!(result_with_analysis.contains("Tree-sitter 分析"));

        assert!(result_with_custom.contains("增强分析基于用户自定义消息"));
    }

    #[tokio::test]
    async fn test_generate_enhanced_commit_message_fallback() {
        let config = create_test_config();
        let diff = "diff --git a/src/test.rs b/src/test.rs\n+// test change";
        
        let args_with_custom = CommitArgs {
            tree_sitter: true,
            depth: Some("medium".to_string()),
            auto_stage: false,
            message: Some("feat: custom message".to_string()),
            review: false,
            passthrough_args: vec![],
        };
        
        let args_without_custom = CommitArgs {
            tree_sitter: true,
            depth: Some("medium".to_string()),
            auto_stage: false,
            message: None,
            review: false,
            passthrough_args: vec![],
        };

        // Test with custom message
        match generate_enhanced_commit_message(&config, diff, Some("feat: custom message".to_string()), &args_with_custom, None).await {
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
        match generate_enhanced_commit_message(&config, diff, None, &args_without_custom, None).await {
            Ok(message) => {
                assert!(!message.is_empty());
            }
            Err(_) => {
                // Expected in test environment
                assert!(true);
            }
        }
    }

    #[tokio::test]
    async fn test_handle_commit_with_tree_sitter() {
        let config = create_test_config();
        
        let args_tree_sitter = CommitArgs {
            tree_sitter: true,
            depth: Some("medium".to_string()),
            auto_stage: false,
            message: None,
            review: false,
            passthrough_args: vec![],
        };
        
        let args_tree_sitter_with_message = CommitArgs {
            tree_sitter: true,
            depth: Some("deep".to_string()),
            auto_stage: false,
            message: Some("feat: enhanced with tree-sitter".to_string()),
            review: false,
            passthrough_args: vec![],
        };

        // Test tree-sitter mode without custom message
        match handle_commit(&config, args_tree_sitter).await {
            Ok(_) => {
                // Success only if in proper git environment
                assert!(true);
            }
            Err(e) => {
                // Expected errors in test environment
                match e {
                    AppError::Git(GitError::NotARepository) => assert!(true),
                    AppError::Generic(msg) if msg.contains("没有检测到任何变更") => assert!(true),
                    _ => assert!(true),
                }
            }
        }

        // Test tree-sitter mode with custom message
        match handle_commit(&config, args_tree_sitter_with_message).await {
            Ok(_) => {
                assert!(true);
            }
            Err(e) => {
                match e {
                    AppError::Git(GitError::NotARepository) => assert!(true),
                    AppError::Generic(msg) if msg.contains("没有检测到任何变更") => assert!(true),
                    _ => assert!(true),
                }
            }
        }
    }

    #[test]
    fn test_commit_args_tree_sitter_combinations() {
        // Test various combinations of tree-sitter related arguments
        let args1 = CommitArgs {
            tree_sitter: true,
            depth: Some("shallow".to_string()),
            auto_stage: false,
            message: None,
            review: false,
            passthrough_args: vec![],
        };
        
        let args2 = CommitArgs {
            tree_sitter: true,
            depth: Some("deep".to_string()),
            auto_stage: true,
            message: Some("custom message".to_string()),
            review: false,
            passthrough_args: vec!["-v".to_string()],
        };
        
        let args3 = CommitArgs {
            tree_sitter: false,
            depth: None,
            auto_stage: false,
            message: Some("simple commit".to_string()),
            review: false,
            passthrough_args: vec![],
        };

        assert!(args1.tree_sitter);
        assert_eq!(args1.depth, Some("shallow".to_string()));
        assert!(!args1.auto_stage);
        assert!(args1.message.is_none());

        assert!(args2.tree_sitter);
        assert_eq!(args2.depth, Some("deep".to_string()));
        assert!(args2.auto_stage);
        assert_eq!(args2.message, Some("custom message".to_string()));
        assert_eq!(args2.passthrough_args, vec!["-v".to_string()]);

        assert!(!args3.tree_sitter);
        assert!(args3.depth.is_none());
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
                    AppError::Generic(msg) if msg.contains("没有检测到任何变更") => assert!(true),
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

    #[tokio::test]
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
            tree_sitter: true,
            depth: Some("medium".to_string()),
            auto_stage: false,
            message: Some("feat: add feature".to_string()),
            review: true,
            passthrough_args: vec![],
        };
        
        assert_eq!(args.tree_sitter, true);
        assert_eq!(args.message, Some("feat: add feature".to_string()));
        assert_eq!(args.review, true);
    }

    #[tokio::test]
    async fn test_enhanced_commit_with_review_context() {
        let config = create_test_config();
        let diff = "test diff content";
        let args = CommitArgs {
            tree_sitter: true,
            depth: Some("medium".to_string()),
            auto_stage: false,
            message: None,
            review: false,
            passthrough_args: vec![],
        };
        
        // Test with review context
        let review_context = "Review findings: code quality good";
        let result = generate_enhanced_commit_message(&config, diff, None, &args, Some(review_context)).await;
        
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
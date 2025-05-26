use crate::{
    config::{AppConfig, TreeSitterConfig},
    errors::{AIError, AppError},
    tree_sitter_analyzer::{
        analyzer::TreeSitterAnalyzer,
        core::{detect_language_from_extension, parse_git_diff},
    },
    types::{
        git::{GitDiff, ReviewArgs},
    },
};

use super::{ai::{execute_review_request, create_review_prompt}, git::extract_diff_for_review};
use chrono;
use colored::Colorize;
use std::{collections::HashMap, env, fs, io::Write, time::Instant};

pub async fn handle_review(
    config: &mut AppConfig,
    review_args: ReviewArgs,
) -> Result<(), AppError> {
    let start_time = Instant::now();
    tracing::info!(
        "开始执行代码评审，参数: depth={}, format={}, tree_sitter={}",
        review_args.depth,
        review_args.format,
        review_args.tree_sitter
    );

    // Extract the Git diff
    tracing::debug!("提取Git差异信息...");
    let diff_text = extract_diff_for_review(&review_args).await?;

    if diff_text.trim().is_empty() {
        tracing::warn!("未检测到代码变更");
        return Err(AppError::Generic(
            "没有检测到代码变更，无法执行评审。请确保已暂存变更或指定了有效的提交范围。"
                .to_string(),
        ));
    }

    tracing::debug!("检测到差异信息，长度: {} 字符", diff_text.len());

    // Determine if TreeSitter should be used
    let use_tree_sitter = review_args.tree_sitter;
    tracing::debug!(
        "TreeSitter分析: {}",
        if use_tree_sitter { "启用" } else { "禁用" }
    );

    // Analyze the diff with appropriate analyzer
    let analyze_start = Instant::now();
    let (git_diff, analysis_text, analysis_results) = if use_tree_sitter {
        tracing::info!("使用TreeSitter进行深度代码分析");
        analyze_diff_with_tree_sitter(&diff_text, &review_args.depth, config)
            .await
            .map_err(|e| {
                tracing::error!("TreeSitter分析失败: {:?}", e);
                e
            })?
    } else {
        tracing::info!("使用简化的代码分析");
        analyze_diff_simple(&diff_text).await?
    };

    tracing::info!("代码分析完成，耗时: {:?}", analyze_start.elapsed());

    // 提取语言信息用于AI评审
    let language_info = extract_language_info(&git_diff, &analysis_results);
    tracing::debug!("检测到的语言: {}", language_info);

    // Generate AI prompt with enhanced context
    let prompt = generate_ai_review_prompt(
        config,
        &diff_text,
        &analysis_text,
        &review_args,
        &git_diff,
        &language_info,
    )
    .await?;

    // Try to send to AI
    let ai_start = Instant::now();
    tracing::info!("发送至 AI 进行代码评审");
    let ai_response = match send_review_to_ai(config, &prompt).await {
        Ok(response) => {
            tracing::info!("AI评审完成，耗时: {:?}", ai_start.elapsed());
            tracing::debug!("AI响应长度: {} 字符", response.len());
            response
        }
        Err(e) => {
            tracing::warn!("AI请求失败: {}，生成离线评审结果", e);
            generate_fallback_review(&analysis_text, &git_diff, &analysis_results)
        }
    };

    // Format and output the review
    tracing::debug!("格式化并输出评审结果");
    format_and_output_review(&ai_response, &review_args).await?;

    let total_time = start_time.elapsed();
    tracing::info!("代码评审完成，总耗时: {:?}", total_time);

    // 输出统计信息
    if tracing::enabled!(tracing::Level::DEBUG) {
        output_review_stats(&git_diff, &analysis_results);
    }

    Ok(())
}

/// Analyze diff with TreeSitter
async fn analyze_diff_with_tree_sitter(
    diff_text: &str,
    depth: &str,
    _config: &AppConfig,
) -> Result<
    (
        GitDiff,
        String,
        Option<crate::tree_sitter_analyzer::core::DiffAnalysis>,
    ),
    AppError,
> {
    // Initialize TreeSitter analyzer with analysis depth
    let mut config = TreeSitterConfig::default();
    config.analysis_depth = depth.to_string();
    let mut analyzer = TreeSitterAnalyzer::new(config).map_err(|e| {
        tracing::error!("TreeSitter分析器初始化失败: {:?}", e);
        AppError::TreeSitter(e)
    })?;

    // Parse the diff to get structured representation
    let git_diff = parse_git_diff(diff_text).map_err(|e| {
        tracing::error!("解析Git差异失败: {:?}", e);
        AppError::TreeSitter(e)
    })?;

    // Generate analysis using TreeSitter
    let analysis = analyzer.analyze_diff(diff_text).map_err(|e| {
        tracing::error!("执行差异分析失败: {:?}", e);
        AppError::TreeSitter(e)
    })?;
    tracing::debug!("差异分析结果: {:?}", analysis);

    // Create detailed analysis text
    let analysis_text = format_tree_sitter_analysis(&analysis, &git_diff);

    Ok((git_diff, analysis_text, Some(analysis)))
}

/// Simple diff analysis without TreeSitter
async fn analyze_diff_simple(
    diff_text: &str,
) -> Result<
    (
        GitDiff,
        String,
        Option<crate::tree_sitter_analyzer::core::DiffAnalysis>,
    ),
    AppError,
> {
    let git_diff = parse_git_diff(diff_text).map_err(|e| AppError::TreeSitter(e))?;

    let mut analysis_text = String::new();
    analysis_text.push_str("## 代码变更分析\n\n");
    analysis_text.push_str("### 变更文件摘要\n\n");

    if git_diff.changed_files.is_empty() {
        analysis_text.push_str("- 未检测到代码变更\n");
    } else {
        for file in &git_diff.changed_files {
            analysis_text.push_str(&format!(
                "- **{}** ({})\n",
                file.path.display(),
                match file.change_type {
                    crate::types::git::ChangeType::Added => "新增",
                    crate::types::git::ChangeType::Modified => "修改",
                    crate::types::git::ChangeType::Deleted => "删除",
                    crate::types::git::ChangeType::Renamed => "重命名",
                    crate::types::git::ChangeType::Copied => "复制",
                    crate::types::git::ChangeType::TypeChanged => "类型变更",
                }
            ));
        }
    }

    analysis_text.push_str("\n### 分析结果\n\n");
    analysis_text.push_str("- ℹ️ **简化分析模式**\n");
    analysis_text.push_str("  - 未启用TreeSitter进行深度分析\n");
    analysis_text.push_str("  - 建议使用 `--tree-sitter` 参数启用更详细的分析\n");

    Ok((git_diff, analysis_text, None))
}

/// Format TreeSitter analysis results into readable text
fn format_tree_sitter_analysis(
    analysis: &crate::tree_sitter_analyzer::core::DiffAnalysis,
    _git_diff: &GitDiff,
) -> String {
    let mut text = String::new();

    text.push_str("## TreeSitter 代码结构分析\n\n");
    text.push_str(&format!("### 总体摘要\n\n{}\n\n", analysis.overall_summary));

    text.push_str("### 变更统计\n\n");
    text.push_str(&format!(
        "- 影响文件数: **{}**\n",
        analysis.file_analyses.len()
    ));
    text.push_str(&format!(
        "- 函数变更: **{}**\n",
        analysis.change_analysis.function_changes
    ));
    text.push_str(&format!(
        "- 类型变更: **{}**\n",
        analysis.change_analysis.type_changes
    ));
    text.push_str(&format!(
        "- 方法变更: **{}**\n",
        analysis.change_analysis.method_changes
    ));
    text.push_str(&format!(
        "- 接口变更: **{}**\n",
        analysis.change_analysis.interface_changes
    ));
    text.push_str(&format!(
        "- 其他变更: **{}**\n\n",
        analysis.change_analysis.other_changes
    ));

    // 按语言分组显示文件分析
    let mut language_groups: HashMap<
        String,
        Vec<&crate::tree_sitter_analyzer::core::FileAnalysis>,
    > = HashMap::new();
    for file_analysis in &analysis.file_analyses {
        language_groups
            .entry(file_analysis.language.clone())
            .or_default()
            .push(file_analysis);
    }

    for (language, files) in language_groups {
        if language == "unknown" || language.is_empty() {
            continue;
        }

        text.push_str(&format!("### {} 文件变更\n\n", language.to_uppercase()));
        for file_analysis in files {
            text.push_str(&format!("- **{}**\n", file_analysis.path.display()));

            if let Some(summary) = &file_analysis.summary {
                text.push_str(&format!("  - {}\n", summary));
            }

            if !file_analysis.affected_nodes.is_empty() {
                text.push_str("  - 受影响的代码结构:\n");
                for node in &file_analysis.affected_nodes {
                    let visibility = if node.is_public { "公开" } else { "私有" };
                    let change_type = match &node.change_type {
                        Some(change) => match change.as_str() {
                            "added" | "added_content" => "➕ ",
                            "deleted" => "❌ ",
                            "modified" | "modified_with_deletion" => "🔄 ",
                            _ => "",
                        },
                        None => "",
                    };

                    text.push_str(&format!(
                        "    - {}**{}** `{}` ({})\n",
                        change_type, node.node_type, node.name, visibility
                    ));
                }
            }
        }
        text.push_str("\n");
    }

    // 添加评审建议
    text.push_str("### 评审重点建议\n\n");
    match &analysis.change_analysis.change_pattern {
        crate::tree_sitter_analyzer::core::ChangePattern::FeatureImplementation => {
            text.push_str("- 🆕 **新功能实现**\n");
            text.push_str("  - 建议关注功能完整性和边界情况处理\n");
            text.push_str("  - 确认是否有足够的测试覆盖新功能\n");
        }
        crate::tree_sitter_analyzer::core::ChangePattern::BugFix => {
            text.push_str("- 🐛 **Bug修复**\n");
            text.push_str("  - 确认修复是否解决了根本问题\n");
            text.push_str("  - 检查是否有回归测试防止问题再次出现\n");
        }
        crate::tree_sitter_analyzer::core::ChangePattern::Refactoring => {
            text.push_str("- ♻️ **代码重构**\n");
            text.push_str("  - 关注功能等价性，确保重构不改变行为\n");
            text.push_str("  - 检查性能影响，尤其是循环和算法改变\n");
        }
        _ => {
            text.push_str("- ℹ️ **代码评审**\n");
            text.push_str("  - 使用 AI 进行深度评审，提供详细反馈\n");
        }
    }

    text
}

/// Extract language information from diff
fn extract_language_info(
    git_diff: &GitDiff,
    analysis_results: &Option<crate::tree_sitter_analyzer::core::DiffAnalysis>,
) -> String {
    if let Some(analysis) = analysis_results {
        // 从TreeSitter分析中获取详细语言信息
        analysis
            .file_analyses
            .iter()
            .filter(|f| !f.language.is_empty() && f.language != "unknown" && f.language != "error")
            .map(|f| f.language.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        // 从文件扩展名猜测语言
        git_diff
            .changed_files
            .iter()
            .filter_map(|f| {
                f.path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .and_then(|ext| detect_language_from_extension(ext))
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Generate AI review prompt using review.md template
async fn generate_ai_review_prompt(
    _config: &AppConfig,
    diff_text: &str,
    analysis: &str,
    args: &ReviewArgs,
    _git_diff: &GitDiff,
    languages: &str,
) -> Result<String, AppError> {
    let prompt = create_review_prompt(
        diff_text,
        analysis,
        args.focus.as_deref(),
        languages,
    );

    Ok(prompt)
}

/// Send review request to AI
async fn send_review_to_ai(config: &AppConfig, prompt: &str) -> Result<String, AIError> {
    // Load system prompt from review.md
    let system_prompt = match config.prompts.get("review") {
        Some(prompt) => prompt.clone(),
        None => {
            // Fallback to embedded assets/review.md if not configured
            include_str!("../../assets/review.md").to_string()
        }
    };

    execute_review_request(config, &system_prompt, prompt).await
}



/// Generate fallback review when AI is unavailable
fn generate_fallback_review(
    analysis_text: &str,
    git_diff: &GitDiff,
    analysis_results: &Option<crate::tree_sitter_analyzer::core::DiffAnalysis>,
) -> String {
    let mut review = String::new();

    review.push_str("# 代码评审结果 (离线模式)\n\n");
    review.push_str("⚠️ **无法连接到 AI 服务，以下是基于静态分析的评审结果**\n\n");

    review.push_str("## 基本代码检查\n\n");

    if let Some(analysis) = analysis_results {
        review.push_str(&format!(
            "- 检测到 {} 个文件变更\n",
            analysis.file_analyses.len()
        ));
        review.push_str(&format!(
            "- 函数变更: {}\n",
            analysis.change_analysis.function_changes
        ));
        review.push_str(&format!(
            "- 类型变更: {}\n",
            analysis.change_analysis.type_changes
        ));
        review.push_str(&format!(
            "- 变更模式: {:?}\n",
            analysis.change_analysis.change_pattern
        ));
        review.push_str(&format!(
            "- 变更范围: {:?}\n",
            analysis.change_analysis.change_scope
        ));
    } else {
        review.push_str(&format!(
            "- 检测到 {} 个文件变更\n",
            git_diff.changed_files.len()
        ));
    }

    review.push_str("\n## 分析结果\n\n");
    review.push_str(analysis_text);

    review.push_str("\n## 建议\n\n");
    review.push_str("- 请检查网络连接和 AI 配置\n");
    review.push_str("- 建议手动检查代码质量和安全性\n");
    review.push_str("- 考虑使用本地代码质量工具进行补充检查\n");

    review
}

/// Format and output the review results
async fn format_and_output_review(review_text: &str, args: &ReviewArgs) -> Result<(), AppError> {
    tracing::debug!(
        "格式化输出，格式: {}, 输出文件: {:?}",
        args.format,
        args.output
    );

    let formatted_output = match args.format.to_lowercase().as_str() {
        "json" => {
            tracing::debug!("使用JSON格式输出");
            let timestamp = chrono::Utc::now().to_rfc3339();
            serde_json::json!({
                "review": review_text,
                "timestamp": timestamp,
                "format_version": "1.0",
                "generator": "gitai",
                "analysis_depth": args.depth,
                "focus": args.focus,
                "language": args.lang
            })
            .to_string()
        }
        "html" => {
            tracing::debug!("使用HTML格式输出");
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
            let processed_content = review_text
                .replace("&", "&amp;")
                .replace("<", "&lt;")
                .replace(">", "&gt;")
                .replace("\n", "<br>\n");

            format!(
                "<!DOCTYPE html>\n<html lang=\"zh-CN\">\n<head>\n\
                <meta charset=\"UTF-8\">\n\
                <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n\
                <title>Gitai 代码评审报告</title>\n\
                <style>\n\
                body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 20px; line-height: 1.6; }}\n\
                .header {{ background: #f8f9fa; padding: 20px; border-radius: 5px; margin-bottom: 20px; }}\n\
                .content {{ background: white; padding: 20px; border: 1px solid #e9ecef; border-radius: 5px; }}\n\
                </style>\n\
                </head>\n<body>\n\
                <div class=\"header\">\n\
                <h1>🔍 Gitai 代码评审报告</h1>\n\
                <p>生成时间: {}</p>\n\
                </div>\n\
                <div class=\"content\">{}</div>\n\
                </body>\n</html>",
                timestamp, processed_content
            )
        }
        "markdown" | "md" => {
            tracing::debug!("使用Markdown格式输出");
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
            format!(
                "# 🔍 Gitai 代码评审报告\n\n\
                **生成时间**: {}\n\
                **分析深度**: {}\n\n\
                ---\n\n\
                {}",
                timestamp, args.depth, review_text
            )
        }
        _ => {
            tracing::debug!("使用默认文本格式输出");
            review_text.to_string()
        }
    };

    if let Some(output_file) = &args.output {
        let expanded_path = expand_tilde(output_file);
        tracing::debug!("输出路径: {}", expanded_path);

        // 确保目录存在
        if let Some(parent_dir) = std::path::Path::new(&expanded_path).parent() {
            if !parent_dir.exists() {
                std::fs::create_dir_all(parent_dir)
                    .map_err(|e| AppError::IO(format!("无法创建输出目录: {:?}", parent_dir), e))?;
            }
        }

        let mut file = fs::File::create(&expanded_path)
            .map_err(|e| AppError::IO(format!("无法创建输出文件: {}", expanded_path), e))?;

        file.write_all(formatted_output.as_bytes())
            .map_err(|e| AppError::IO(format!("写入输出文件失败: {}", expanded_path), e))?;

        file.flush()
            .map_err(|e| AppError::IO(format!("刷新文件缓冲区失败: {}", expanded_path), e))?;

        println!(
            "{} 评审结果已保存到: {}",
            "✅".green(),
            expanded_path.bold()
        );
        tracing::info!("评审结果已成功保存到文件: {}", expanded_path);
    } else {
        // 输出到控制台
        match args.format.to_lowercase().as_str() {
            "json" | "html" => {
                println!("{}", formatted_output);
            }
            _ => {
                println!("\n{}", "🔍 代码评审结果".bold().green());
                println!("{}", "==================".green());
                println!();

                for line in formatted_output.lines() {
                    if line.starts_with("# ") {
                        println!("{}", line.bold().blue());
                    } else if line.starts_with("## ") {
                        println!("{}", line.bold().cyan());
                    } else if line.starts_with("### ") {
                        println!("{}", line.bold().yellow());
                    } else if line.starts_with("- ") || line.starts_with("* ") {
                        println!("  {}", line.dimmed());
                    } else if line.trim().is_empty() {
                        println!();
                    } else {
                        println!("{}", line);
                    }
                }

                println!("\n{}", "==================".green());
                println!("{} {}", "✨".green(), "评审完成".green());
            }
        }

        tracing::debug!("评审结果已输出到控制台");
    }

    Ok(())
}

/// Expand tilde in file paths
fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") || path == "~" {
        if let Ok(home) = env::var("HOME") {
            return path.replacen("~", &home, 1);
        }
    }
    path.to_string()
}

/// Output review statistics for debugging
fn output_review_stats(
    git_diff: &GitDiff,
    analysis_results: &Option<crate::tree_sitter_analyzer::core::DiffAnalysis>,
) {
    if let Some(analysis) = analysis_results {
        tracing::debug!(
            "评审统计: 文件数={}, 函数变更={}, 类型变更={}, 方法变更={}",
            analysis.file_analyses.len(),
            analysis.change_analysis.function_changes,
            analysis.change_analysis.type_changes,
            analysis.change_analysis.method_changes
        );
    } else {
        tracing::debug!("评审统计: 文件数={}", git_diff.changed_files.len());
    }
}

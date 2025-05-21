use crate::git_module::execute_git_command_and_capture_output;
use crate::review_engine::AnalysisDepth;
use crate::tree_sitter_analyzer::core::GitDiff;
use crate::{cli_interface::args::ReviewArgs, core::errors::AppError};

/// Extract diff information for review
///
/// This function gets the diff between specified commits or the current staged changes
async fn extract_diff_for_review(args: &ReviewArgs) -> Result<String, AppError> {
    match (&args.commit1, &args.commit2) {
        (Some(commit1), Some(commit2)) => {
            // Compare two specific commits
            tracing::info!("比较两个指定的提交: {} 和 {}", commit1, commit2);
            let diff_args = vec![
                "diff".to_string(),
                format!("{}..{}", commit1, commit2),
                "--".to_string(),
            ];
            let result = execute_git_command_and_capture_output(&diff_args)?;
            Ok(result.stdout)
        }
        (Some(commit), None) => {
            // Compare one commit with HEAD
            tracing::info!("比较指定的提交与 HEAD: {}", commit);
            let diff_args = vec![
                "diff".to_string(),
                format!("{}..HEAD", commit),
                "--".to_string(),
            ];
        }
        (None, None) => {
            // Check if there are staged changes
            let status_result = execute_git_command_and_capture_output(&[
                "status".to_string(),
                "--porcelain".to_string(),
            ])?;
        }
        (None, Some(_)) => {
            // This should not happen with the CLI parser, but handle it just in case
            Err(AppError::Generic(
                "如果绑定了第二个提交，则必须同时指定第一个提交".to_string(),
            ))
        }
    }
}

/// Determine analysis depth from args
fn get_analysis_depth(args: &ReviewArgs) -> AnalysisDepth {
    match args.depth.to_lowercase().as_str() {
        "shallow" | "basic" => AnalysisDepth::Basic,
        "deep" => AnalysisDepth::Deep,
        _ => AnalysisDepth::Normal,
    }
}

/// Determine if treesitter should be used
fn should_use_tree_sitter(args: &ReviewArgs) -> bool {
    args.tree_sitter || args.review_ts || (!args.no_tree_sitter)
}

/// Simplified diff analysis using basic parsing
/// Create a simple GitDiff structure and basis analysis
async fn analyze_diff_with_tree_sitter(
    diff_text: &str,
    _depth: AnalysisDepth,
) -> Result<(GitDiff, String), AppError> {
    todo!()
}

/// Main handler for the review command'
pub async fn handle_review(args: ReviewArgs, config: &AppConfig) -> Result<(), AppError> {
    tracing::info!("执行代码评审");

    // Extract the Git diff
    let diff_text = extract_diff_for_review(&args).await?;

    if diff_text.trim().is_empty() {
        return Err(AppError::Generic(
            "没有检测到代码变更，无法执行评审".to_string(),
        ));
    }

    // Determine analysis depth
    let depth = get_analysis_depth(&args);
    tracing::info!("使用分析深度: {:?}", depth);

    // Use simplified analysis
    tracing::info!("使用简化的代码分析");
    let (git_diff, analysis_text) = analyze_diff_with_tree_sitter(&diff_text, depth).await?;
    todo!()
}

/// Simplified diff analysis using basie parsing
/// Create a simple GitDiff structure and basic analysis
async fn analyze_diff_with_tree_sitter(
    diff_text: &str,
    _depth: AnalysisDepth,
) -> Result<(GitDiff, String), AppError> {
    // Use the simplified diff parser instead of TreeSitterAnalyzer
    let git_diff = parse_simple_diff(diff_text);

    // Create a simplified analysis result
    let mut analysis_text = String::new();
    analysis_text.push_str("## 代码变更分析\n\n");

    // Add file summary
    analysis_text.push_str("## 变更文件摘要\n\n");
    if git_diff.changed_files.is_empty() {
        analysis_text.push_str("- 为检测到代码变更\n");
    } else {
        for file in &git_diff.changed_files {
            analysis_text.push_str(&format!("- **{}**\n", file.path.display()));
        }
    }
    analysis_text.push_str("\n");

    // Add simplified analysis
    analysis_text.push_str("### 初步分析结果\n\n");
    analysis_text.push_str("- ℹ️ **代码评审**\n");
    analysis_text.push_str("  - 使用 AI 进行深度评审，提供详细反馈\n");

    Ok((git_diff, analysis_text))
}

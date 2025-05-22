use crate::{config::TreeSitterConfig, errors::AppError, tree_sitter_analyzer::core::GitDiff};

use super::git::extract_diff_for_review;

pub async fn handle_review(config: &AppConfig, args: Vec<String>, use_ai: bool) -> Result<CommandOutput, AppError>) {
    tracing::info!("执行代码评审");

    // Extract the Git diff
    let diff_text = extract_diff_for_review(&args).await?;

    if diff_text.trim().is_empty() {
        return Err(AppError::Generic("没有检测到代码变更，无法执行评审。".to_string()));
    }

    // Determine analysis depth
    let depth = get_analysis_depth(&args);
    tracing::info!("使用分析深度: {:?}", depth);

    // Determine if TreeSitter should be used
    let use_tree_sitter = should_use_tree_sitter(&args);

    // Analyze the diff with appropriate analyzer
    let (git_diff, analysis_text, analysis_results) = if use_tree_sitter {
        tracing::info!("使用TreeSitter进行深度代码分析");
        let (diff, text) = analyze_diff_with_tree_sitter(&diff_text, depth).await?;
    } else {
        tracing::info!("使用简化的代码分析");
    }

    todo!()
}


/// Determine analysis depth from args
fn get_analysis_depth(args: &ReviewArgs) -> AnalysisDepth {
    match args.depth.to_lowercase().as_str() {
        "shallow" => AnalysisDepth::Basic,
        "deep" => AnalysisDepth::Deep,
        _ => AnalysisDepth::Normal, // Default to normal if not recognized
    }
}

/// Determine if TreeSitter should be used
fn should_use_tree_sitter(args: &ReviewArgs) -> bool {
    args.tree_sitter || args.review_ts || (!args.no_tree_sitter)
}

/// Advanced diff analysis using TreeSitter for language-aware parsing
/// Creates a detailed GitDiff structure with structural code analysis
async fn analyze_diff_with_tree_sitter(
    diff_text: &str,
    depth: AnalysisDepth,
) -> Result<(GitDiff, String), AppError> {
    // Initialize Tree-sitter analyzer with config
    let mut config = TreeSitterConfig::default();
    config.analysis_depth = match depth {
        AnalysisDepth::Basic => "shallow".to_string(),
        AnalysisDepth::Normal => "medium".to_string(),
        AnalysisDepth::Deep => "deep".to_string(),
    };

    let mut analyzer = TreeSitterAnalyzer::new(config)
        .map_err(|e| AppError::TreeSitter(e))?;

    // Parse the diff to get structured representation
    let git_diff = analyzer.parse_git_diff_text(diff_text)
        .map_err(|e| AppError::TreeSitter(e))?;

}

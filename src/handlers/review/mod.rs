pub mod analysis;
pub mod ai;
pub mod types;

use crate::{
    config::AppConfig,
    errors::AppError,
    handlers::git::extract_diff_for_review_in_dir,
    types::git::ReviewArgs,
};
use analysis::DiffAnalyzer;
use ai::AIReviewEngine;
use types::StandardReviewRequest;
use std::sync::Arc;

/// 核心review逻辑，返回分析结果
async fn perform_review(config: &AppConfig, args: ReviewArgs) -> Result<String, AppError> {
    let diff_text = extract_diff_for_review_in_dir(&args, args.path.as_deref()).await?;
    if diff_text.trim().is_empty() {
        return Err(AppError::Generic("没有找到需要审查的代码变更".to_string()));
    }
    
    let config_arc = Arc::new(config.clone());
    let ai_analysis_engine = Arc::new(crate::handlers::analysis::AIAnalysisEngine::new(config_arc.clone()));
    
    let diff_analyzer = DiffAnalyzer::new(config.tree_sitter.clone(), ai_analysis_engine.clone());
    let analysis_result = diff_analyzer.analyze_diff(&diff_text, true).await?;
    
    let ai_engine = AIReviewEngine::new(config_arc, ai_analysis_engine);
    let request = StandardReviewRequest {
        diff_text: diff_text.clone(),
        analysis_text: analysis_result.analysis_text.clone(),
        language_info: analysis_result.language_info.clone(),
    };
    
    ai_engine.perform_standard_review(request).await.map(|result| result.content)
}

/// 执行review并打印结果
pub async fn handle_review(config: &AppConfig, args: ReviewArgs) -> Result<(), AppError> {
    let result = perform_review(config, args).await?;
    println!("{}", result);
    Ok(())
}

/// 执行review并返回结果
pub async fn handle_review_with_output(config: &AppConfig, args: ReviewArgs) -> Result<String, AppError> {
    perform_review(config, args).await
}

pub async fn handle_review_with_output_in_dir(
    config: &mut AppConfig,
    args: ReviewArgs,
    _dir: Option<&str>,
) -> Result<String, AppError> {
    handle_review_with_output(config, args).await
}
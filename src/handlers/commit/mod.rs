pub mod analysis;
pub mod generator;
pub mod interaction;
pub mod repository;
pub mod review_integration;
pub mod types;

use crate::{
    config::AppConfig,
    errors::AppError,
    types::git::CommitArgs,
};
use analysis::CommitAnalyzer;
use generator::CommitMessageGenerator;
use interaction::UserInteractionManager;
use repository::RepositoryManager;
use review_integration::ReviewIntegrator;
use types::*;
use std::sync::Arc;

pub async fn handle_commit(config: &AppConfig, args: CommitArgs) -> Result<(), AppError> {
    let repo_manager = RepositoryManager::new();
    let git_result = repo_manager.perform_git_operations(GitOperationRequest {
        auto_stage: args.auto_stage,
        check_repository: true,
    }).await?;
    
    if !git_result.has_changes {
        return Err(AppError::Generic("没有检测到任何变更".to_string()));
    }
    
    let config_arc = Arc::new(config.clone());
    let review_integrator = ReviewIntegrator::from_review_config(&config.review);
    let review_result = review_integrator.integrate_review_results().await?;
    
    let message = if args.tree_sitter && config.tree_sitter.enabled.unwrap_or(false) {
        let analyzer = CommitAnalyzer::new(config.tree_sitter.clone());
        let analysis = analyzer.analyze_diff_for_commit(&git_result.diff_content, &args).await?;
        let request = EnhancedCommitRequest {
            diff_content: git_result.diff_content.clone(),
            custom_message: args.message.clone(),
            review_context: review_result.review_content.clone(),
        };
        let mut result = CommitMessageGenerator::new(config.clone())
            .generate_enhanced_commit_message(request).await?;
        result.tree_sitter_analysis = analysis.analysis_data;
        result.message
    } else if let Some(msg) = args.message {
        msg
    } else {
        let request = BasicCommitRequest {
            diff_content: git_result.diff_content.clone(),
            review_context: review_result.review_content.clone(),
        };
        CommitMessageGenerator::new(config.clone())
            .generate_basic_commit_message(request).await?.message
    };
    
    let final_message = crate::utils::add_issue_prefix_to_commit_message(&message, args.issue_id.as_ref());
    
    let interaction_manager = UserInteractionManager::new(UserInteractionConfig {
        require_confirmation: true,
        show_analysis: true,
        format_output: true,
    });
    
    let confirmation = interaction_manager.confirm_commit_message(&final_message)?;
    if !confirmation.confirmed {
        println!("❌ 提交已取消");
        return Ok(());
    }
    
    repo_manager.execute_commit(&final_message).await?;
    println!("✅ 提交成功");
    Ok(())
}
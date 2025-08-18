//! Review module - Modular code review functionality
//! 
//! This module provides a comprehensive code review system with the following components:
//! - DevOps integration for work item context
//! - Advanced diff analysis using TreeSitter
//! - AI-powered review generation
//! - Flexible output formatting
//! - Automatic file management and storage

pub mod analysis;
pub mod ai;
pub mod devops;
pub mod file_manager;
pub mod output;
pub mod types;

use crate::{
    config::AppConfig,
    errors::AppError,
    handlers::git::extract_diff_for_review_in_dir,
    types::git::ReviewArgs,
};
use std::{sync::Arc, time::Instant};
use tracing;

use analysis::DiffAnalyzer;
use ai::AIReviewEngine;
use devops::DevOpsWorkItemFetcher;
use file_manager::FileManager;
use output::OutputFormatter;
use types::*;

/// Main orchestrator for the review process
pub struct ReviewOrchestrator {
    devops_fetcher: DevOpsWorkItemFetcher,
    diff_analyzer: DiffAnalyzer,
    ai_engine: AIReviewEngine,
    output_formatter: OutputFormatter,
    file_manager: FileManager,
    config: Arc<AppConfig>,
}

impl ReviewOrchestrator {
    /// Create a new review orchestrator
    pub fn new(config: Arc<AppConfig>) -> Result<Self, AppError> {
        // Initialize AI analysis engine
        let ai_analysis_engine = Arc::new(crate::handlers::analysis::AIAnalysisEngine::new(
            config.clone()
        ));

        // Initialize components
        let devops_fetcher = DevOpsWorkItemFetcher::new(&config);
        let diff_analyzer = DiffAnalyzer::new(config.tree_sitter.clone(), ai_analysis_engine.clone());
        let ai_engine = AIReviewEngine::new(config.clone(), ai_analysis_engine);
        
        let output_config = OutputConfig {
            format: "console".to_string(),
            show_stats: true,
            verbose: false,
        };
        let output_formatter = OutputFormatter::new(output_config);
        
        let file_manager = FileManager::new(&config.review, output_formatter.clone());

        Ok(Self {
            devops_fetcher,
            diff_analyzer,
            ai_engine,
            output_formatter,
            file_manager,
            config,
        })
    }

    /// Main entry point for handling review requests
    pub async fn handle_review(
        &mut self,
        args: ReviewArgs,
    ) -> Result<(), AppError> {
        let start_time = Instant::now();

        tracing::info!("开始代码审查流程");

        // Step 1: Fetch work items if specified
        let work_items = self.devops_fetcher.fetch_work_items(&args).await?;
        if !work_items.is_empty() {
            tracing::info!("成功获取 {} 个工作项", work_items.len());
        }

        // Step 2: Extract diff for review
        let diff_text = extract_diff_for_review_in_dir(&args, args.path.as_deref()).await?;
        if diff_text.trim().is_empty() {
            return Err(AppError::Generic("没有找到需要审查的代码变更".to_string()));
        }

        // Step 3: Analyze diff
        let use_tree_sitter = self.should_use_tree_sitter(&args);
        let analysis_result = self.diff_analyzer.analyze_diff(&diff_text, use_tree_sitter).await?;
        
        tracing::info!("差分分析完成，语言类型: {}", analysis_result.language_info);

        // Step 4: Perform AI analysis
        let ai_result = if !work_items.is_empty() {
            // Enhanced analysis with work items
            let request = EnhancedAnalysisRequest {
                diff_text: diff_text.clone(),
                work_items: work_items.clone(),
                args: args.clone(),
                language_info: analysis_result.language_info.clone(),
            };
            self.ai_engine.perform_enhanced_analysis(request).await?
        } else {
            // Standard analysis
            let request = StandardReviewRequest {
                diff_text: diff_text.clone(),
                analysis_text: analysis_result.analysis_text.clone(),
                language_info: analysis_result.language_info.clone(),
            };
            self.ai_engine.perform_standard_review(request).await?
        };

        // Step 5: Format and display output
        let work_item_analysis: Vec<crate::types::devops::AnalysisWorkItem> = work_items.iter()
            .map(|item| crate::types::devops::AnalysisWorkItem::from(item))
            .collect();

        let formatted_output = if !work_items.is_empty() {
            self.output_formatter.format_enhanced_result(&ai_result, &work_item_analysis)
        } else {
            self.output_formatter.format_for_display(&ai_result.content)
        };

        // Display the result
        println!("{}", formatted_output);

        // Step 6: Save results if auto-save is enabled
        if let Some(saved_path) = self.file_manager.save_review_results(&ai_result.content).await? {
            println!("\n💾 审查结果已自动保存到: {}", saved_path.display());
        }

        // Step 7: Show statistics if enabled
        if self.output_formatter.config.show_stats {
            let stats = self.output_formatter.output_review_stats(start_time, &ai_result.analysis_type);
            println!("{}", stats);
        }

        tracing::info!("代码审查流程完成");
        Ok(())
    }

    /// Handle review and return formatted output instead of printing to console
    pub async fn handle_review_with_output(
        &mut self,
        args: ReviewArgs,
    ) -> Result<String, AppError> {
        self.handle_review_with_output_in_dir(args, None).await
    }

    /// Handle review and return formatted output for specified directory
    pub async fn handle_review_with_output_in_dir(
        &mut self,
        args: ReviewArgs,
        dir: Option<&str>,
    ) -> Result<String, AppError> {
        let start_time = Instant::now();

        tracing::info!("开始代码审查流程");

        // Step 1: Fetch work items if specified
        let work_items = self.devops_fetcher.fetch_work_items(&args).await?;
        if !work_items.is_empty() {
            tracing::info!("成功获取 {} 个工作项", work_items.len());
        }

        // Step 2: Extract diff for review
        let diff_text = extract_diff_for_review_in_dir(&args, dir).await?;
        if diff_text.trim().is_empty() {
            return Err(AppError::Generic("没有找到需要审查的代码变更".to_string()));
        }

        // Step 3: Analyze diff
        let use_tree_sitter = self.should_use_tree_sitter(&args);
        let analysis_result = self.diff_analyzer.analyze_diff(&diff_text, use_tree_sitter).await?;
        
        tracing::info!("差分分析完成，语言类型: {}", analysis_result.language_info);

        // Step 4: Perform AI analysis
        let ai_result = if !work_items.is_empty() {
            // Enhanced analysis with work items
            let request = EnhancedAnalysisRequest {
                diff_text: diff_text.clone(),
                work_items: work_items.clone(),
                args: args.clone(),
                language_info: analysis_result.language_info.clone(),
            };
            self.ai_engine.perform_enhanced_analysis(request).await?
        } else {
            // Standard analysis
            let request = StandardReviewRequest {
                diff_text: diff_text.clone(),
                analysis_text: analysis_result.analysis_text.clone(),
                language_info: analysis_result.language_info.clone(),
            };
            self.ai_engine.perform_standard_review(request).await?
        };

        // Step 5: Format output
        let work_item_analysis: Vec<crate::types::devops::AnalysisWorkItem> = work_items.iter()
            .map(|item| crate::types::devops::AnalysisWorkItem::from(item))
            .collect();

        let formatted_output = if !work_items.is_empty() {
            self.output_formatter.format_enhanced_result(&ai_result, &work_item_analysis)
        } else {
            self.output_formatter.format_for_display(&ai_result.content)
        };

        // Step 6: Save results if auto-save is enabled
        let mut result_text = formatted_output.clone();
        if let Some(saved_path) = self.file_manager.save_review_results(&ai_result.content).await? {
            result_text.push_str(&format!("\n\n💾 审查结果已自动保存到: {}", saved_path.display()));
        }

        // Step 7: Add statistics if enabled
        if self.output_formatter.config.show_stats {
            let stats = self.output_formatter.output_review_stats(start_time, &ai_result.analysis_type);
            result_text.push_str(&format!("\n\n{}", stats));
        }

        tracing::info!("代码审查流程完成");
        Ok(result_text)
    }

    /// Determine whether to use TreeSitter analysis
    fn should_use_tree_sitter(&self, _args: &ReviewArgs) -> bool {
        // Use TreeSitter for enhanced analysis by default
        // Could be made configurable based on args or config
        true
    }
}

/// Legacy function for backward compatibility
pub async fn handle_review(
    config: &mut AppConfig,
    review_args: ReviewArgs,
) -> Result<(), AppError> {
    let config_arc = Arc::new(config.clone());
    let mut orchestrator = ReviewOrchestrator::new(config_arc)?;
    orchestrator.handle_review(review_args).await
}

/// Handle review and return the formatted output for MCP clients
pub async fn handle_review_with_output(
    config: &mut AppConfig,
    review_args: ReviewArgs,
) -> Result<String, AppError> {
    let config_arc = Arc::new(config.clone());
    let mut orchestrator = ReviewOrchestrator::new(config_arc)?;
    orchestrator.handle_review_with_output(review_args).await
}

/// Handle review and return the formatted output for MCP clients in specified directory
pub async fn handle_review_with_output_in_dir(
    config: &mut AppConfig,
    review_args: ReviewArgs,
    dir: Option<&str>,
) -> Result<String, AppError> {
    let config_arc = Arc::new(config.clone());
    let mut orchestrator = ReviewOrchestrator::new(config_arc)?;
    orchestrator.handle_review_with_output_in_dir(review_args, dir).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::git::{DefectList, StoryList, TaskList};

    fn create_test_config() -> Arc<AppConfig> {
        Arc::new(AppConfig::default())
    }

    fn create_test_args() -> ReviewArgs {
        ReviewArgs {
            files: None,
            commits: None,
            range: None,
            depth: None,
            format: None,
            output: None,
            language: None,
            stories: Some(StoryList(vec![1, 2])),
            tasks: Some(TaskList(vec![3])),
            defects: Some(DefectList(vec![])),
            space_id: Some(12345),
        }
    }

    #[test]
    fn test_orchestrator_creation() {
        let config = create_test_config();
        let result = ReviewOrchestrator::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_use_tree_sitter() {
        let config = create_test_config();
        let orchestrator = ReviewOrchestrator::new(config).unwrap();
        let args = create_test_args();
        
        // Should default to true for now
        assert!(orchestrator.should_use_tree_sitter(&args));
    }
}
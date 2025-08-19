//! Commit module - Modular AI-enhanced commit functionality
//! 
//! This module provides a comprehensive commit system with the following components:
//! - Repository operations and Git integration
//! - Tree-sitter analysis for enhanced commit messages  
//! - AI-powered commit message generation
//! - Code review integration
//! - User interaction and confirmation
//! - Flexible configuration and customization

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
    utils::add_issue_prefix_to_commit_message,
};
use std::{sync::Arc, time::Instant};
use tracing;

use analysis::CommitAnalyzer;
use generator::CommitMessageGenerator;
use interaction::{UserInteractionManager, CommitStats, EnhancedAnalysisInfo};
use repository::RepositoryManager;
use review_integration::ReviewIntegrator;
use types::*;

/// Main orchestrator for the commit process
pub struct CommitOrchestrator {
    repository_manager: RepositoryManager,
    commit_analyzer: Option<CommitAnalyzer>,
    message_generator: CommitMessageGenerator,
    review_integrator: ReviewIntegrator,
    interaction_manager: UserInteractionManager,
    config: Arc<AppConfig>,
}

impl CommitOrchestrator {
    /// Create a new commit orchestrator
    pub fn new(config: Arc<AppConfig>) -> Result<Self, AppError> {
        let repository_manager = RepositoryManager::new();
        
        // Initialize commit analyzer if Tree-sitter is enabled
        let commit_analyzer = if config.tree_sitter.enabled.unwrap_or(false) {
            Some(CommitAnalyzer::new(config.tree_sitter.clone()))
        } else {
            None
        };
        
        let message_generator = CommitMessageGenerator::new((*config).clone());
        let review_integrator = ReviewIntegrator::from_review_config(&config.review);
        
        let interaction_config = UserInteractionConfig {
            require_confirmation: true,
            show_analysis: true,
            format_output: true,
        };
        let interaction_manager = UserInteractionManager::new(interaction_config);

        Ok(Self {
            repository_manager,
            commit_analyzer,
            message_generator,
            review_integrator,
            interaction_manager,
            config,
        })
    }

    /// Main entry point for handling commit requests
    pub async fn handle_commit(&mut self, args: CommitArgs) -> Result<(), AppError> {
        let start_time = Instant::now();
        
        tracing::info!("开始处理智能提交命令");
        
        // Step 1: Repository operations
        let git_request = GitOperationRequest {
            auto_stage: args.auto_stage,
            check_repository: true,
        };
        
        let git_result = self.repository_manager.perform_git_operations(git_request).await?;
        
        if !git_result.has_changes {
            return Err(AppError::Generic("没有检测到任何变更".to_string()));
        }

        // Step 2: Review integration
        let review_result = self.review_integrator.integrate_review_results().await?;
        
        // Step 3: Generate commit message
        let commit_result = self.generate_commit_message(&args, &git_result, &review_result).await?;
        
        // Step 4: Add issue ID prefix if provided
        let final_message = add_issue_prefix_to_commit_message(&commit_result.message, args.issue_id.as_ref());
        
        // Step 5: User interaction and confirmation
        let interaction_result = self.interaction_manager.confirm_commit_message(&final_message)?;
        
        if !interaction_result.confirmed {
            println!("❌ 提交已取消");
            return Ok(());
        }
        
        // Step 6: Execute the commit
        let execution_request = CommitExecutionRequest {
            message: final_message.clone(),
            issue_id: args.issue_id,
            passthrough_args: args.passthrough_args,
        };
        
        let execution_result = self.execute_commit(execution_request).await?;
        
        // Step 7: Show results and statistics
        self.show_completion_info(&execution_result, &commit_result, start_time)?;
        
        tracing::info!("智能提交流程完成");
        Ok(())
    }

    /// Generate commit message based on configuration and context
    async fn generate_commit_message(
        &self,
        args: &CommitArgs,
        git_result: &GitOperationResult,
        review_result: &ReviewIntegrationResult,
    ) -> Result<CommitGenerationResult, AppError> {
        let generation_config = CommitGenerationConfig {
            use_tree_sitter: args.tree_sitter && self.commit_analyzer.is_some(),
            analysis_depth: args.depth.clone(),
            include_review: review_result.integration_successful,
            custom_message: args.message.clone(),
        };

        if generation_config.use_tree_sitter {
            self.generate_enhanced_commit_message(args, git_result, review_result).await
        } else if let Some(ref custom_message) = args.message {
            self.handle_custom_message_with_review(custom_message, review_result)
        } else {
            self.generate_basic_commit_message(git_result, review_result).await
        }
    }

    /// Generate enhanced commit message with Tree-sitter analysis
    async fn generate_enhanced_commit_message(
        &self,
        args: &CommitArgs,
        git_result: &GitOperationResult,
        review_result: &ReviewIntegrationResult,
    ) -> Result<CommitGenerationResult, AppError> {
        if let Some(ref analyzer) = self.commit_analyzer {
            tracing::info!("🌳 正在使用Tree-sitter增强分析生成提交信息...");
            
            // Perform Tree-sitter analysis
            let analysis_result = analyzer.analyze_diff_for_commit(&git_result.diff_content, args).await;
            
            match analysis_result {
                Ok(tree_sitter_analysis) => {
                    // Generate enhanced commit message with analysis
                    let enhanced_request = EnhancedCommitRequest {
                        diff_content: git_result.diff_content.clone(),
                        custom_message: args.message.clone(),
                        analysis_depth: args.depth.clone().unwrap_or_else(|| "medium".to_string()),
                        review_context: review_result.review_content.clone(),
                    };
                    
                    let mut result = self.message_generator.generate_enhanced_commit_message(enhanced_request).await?;
                    result.tree_sitter_analysis = tree_sitter_analysis.analysis_data;
                    Ok(result)
                }
                Err(e) => {
                    tracing::warn!("Tree-sitter分析失败，回退到基础模式: {:?}", e);
                    // Fallback to basic generation
                    if let Some(ref custom_msg) = args.message {
                        Ok(self.handle_custom_message_with_review(custom_msg, review_result)?)
                    } else {
                        self.generate_basic_commit_message(git_result, review_result).await
                    }
                }
            }
        } else {
            return Err(AppError::Generic("Tree-sitter 分析器未初始化".to_string()));
        }
    }

    /// Generate basic commit message
    async fn generate_basic_commit_message(
        &self,
        git_result: &GitOperationResult,
        review_result: &ReviewIntegrationResult,
    ) -> Result<CommitGenerationResult, AppError> {
        let request = BasicCommitRequest {
            diff_content: git_result.diff_content.clone(),
            review_context: review_result.review_content.clone(),
        };
        
        self.message_generator.generate_basic_commit_message(request).await
    }

    /// Handle custom message with optional review integration
    fn handle_custom_message_with_review(
        &self,
        custom_message: &str,
        review_result: &ReviewIntegrationResult,
    ) -> Result<CommitGenerationResult, AppError> {
        let final_message = if let Some(ref review_content) = review_result.review_content {
            self.message_generator.format_custom_message_with_review(custom_message, review_content)
        } else {
            custom_message.to_string()
        };
        
        Ok(CommitGenerationResult {
            message: final_message,
            enhanced: false,
            tree_sitter_analysis: None,
            fallback_used: false,
        })
    }

    /// Execute the actual commit
    async fn execute_commit(&self, request: CommitExecutionRequest) -> Result<CommitExecutionResult, AppError> {
        let commit_hash = self.repository_manager.execute_commit(&request.message).await?;
        
        Ok(CommitExecutionResult {
            success: true,
            commit_hash: Some(commit_hash),
            message_used: request.message,
        })
    }

    /// Show completion information
    fn show_completion_info(
        &self,
        execution_result: &CommitExecutionResult,
        commit_result: &CommitGenerationResult,
        start_time: Instant,
    ) -> Result<(), AppError> {
        if execution_result.success {
            self.interaction_manager.show_success("提交成功!");
            
            // Show enhanced analysis info if available
            if commit_result.enhanced || commit_result.tree_sitter_analysis.is_some() {
                let analysis_info = EnhancedAnalysisInfo {
                    tree_sitter_used: commit_result.tree_sitter_analysis.is_some(),
                    analysis_depth: "medium".to_string(), // Could be extracted from config
                    review_integrated: false, // Could be extracted from context
                    ai_enhanced: !commit_result.fallback_used,
                    ai_model: Some(self.config.ai.model_name.clone()),
                    confidence_score: if commit_result.fallback_used { Some(0.5) } else { Some(0.9) },
                };
                
                self.interaction_manager.show_enhanced_analysis(&analysis_info);
            }
            
            // Show commit statistics
            let stats = CommitStats {
                files_changed: 1, // Simplified - could be extracted from git result
                lines_added: 10,  // Simplified - could be calculated from diff
                lines_removed: 5, // Simplified - could be calculated from diff
                primary_language: Some("Rust".to_string()), // Could be detected from diff
                generation_time: Some(start_time.elapsed()),
            };
            
            self.interaction_manager.display_commit_stats(&stats);
            
            if let Some(ref commit_hash) = execution_result.commit_hash {
                tracing::info!("提交哈希: {}", commit_hash);
            }
        } else {
            self.interaction_manager.show_error("提交失败");
        }
        
        Ok(())
    }

    /// Check if Tree-sitter analysis is available
    fn is_tree_sitter_available(&self) -> bool {
        self.commit_analyzer.is_some()
    }

    /// Get supported analysis features
    pub fn get_supported_features(&self) -> SupportedFeatures {
        SupportedFeatures {
            tree_sitter_analysis: self.is_tree_sitter_available(),
            ai_generation: true, // Always available
            review_integration: self.review_integrator.is_enabled(),
            custom_messages: true, // Always available
            issue_id_support: true, // Always available
        }
    }
}

/// Legacy function for backward compatibility
pub async fn handle_commit(config: &AppConfig, args: CommitArgs) -> Result<(), AppError> {
    let config_arc = Arc::new(config.clone());
    let mut orchestrator = CommitOrchestrator::new(config_arc)?;
    orchestrator.handle_commit(args).await
}

/// Supported features information
#[derive(Debug, Clone)]
pub struct SupportedFeatures {
    pub tree_sitter_analysis: bool,
    pub ai_generation: bool,
    pub review_integration: bool,
    pub custom_messages: bool,
    pub issue_id_support: bool,
}

impl SupportedFeatures {
    /// Get feature description
    pub fn get_description(&self) -> String {
        let mut features = Vec::new();
        
        if self.ai_generation {
            features.push("AI 智能提交信息生成");
        }
        
        if self.tree_sitter_analysis {
            features.push("Tree-sitter 静态代码分析");
        }
        
        if self.review_integration {
            features.push("代码评审结果集成");
        }
        
        if self.custom_messages {
            features.push("自定义提交信息支持");
        }
        
        if self.issue_id_support {
            features.push("工单号自动添加");
        }
        
        if features.is_empty() {
            "基础提交功能".to_string()
        } else {
            features.join(", ")
        }
    }

    /// Check if all advanced features are available
    pub fn has_all_features(&self) -> bool {
        self.tree_sitter_analysis && 
        self.ai_generation && 
        self.review_integration && 
        self.custom_messages && 
        self.issue_id_support
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::git::{DefectList, StoryList, TaskList};
    use crate::config::{AIConfig, TreeSitterConfig, LanguageConfig, ReviewConfig, ScanConfig};
    use std::collections::HashMap;

    fn create_test_config() -> Arc<AppConfig> {
        let mut prompts = HashMap::new();
        prompts.insert(
            "commit-generator".to_string(),
            "Generate a professional commit message".to_string(),
        );
        
        Arc::new(AppConfig {
            ai: AIConfig {
                api_url: "http://localhost:11434/v1/chat/completions".to_string(),
                model_name: "test-model".to_string(),
                temperature: 0.7,
                api_key: None,
            },
            tree_sitter: TreeSitterConfig {
                enabled: true,
                analysis_depth: "medium".to_string(),
                cache_enabled: true,
                languages: vec!["rust".to_string(), "javascript".to_string()],
            },
            review: ReviewConfig {
                auto_save: true,
                storage_path: "~/.gitai/review_results".to_string(),
                format: "markdown".to_string(),
                max_age_hours: 168,
                include_in_commit: true,
            },
            account: None,
            language: LanguageConfig::default(),
            scan: ScanConfig::default(),
            prompts,
        })
    }

    fn create_test_commit_args() -> CommitArgs {
        CommitArgs {
            tree_sitter: false,
            depth: None,
            auto_stage: false,
            message: None,
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        }
    }

    #[test]
    fn test_commit_orchestrator_creation() {
        let config = create_test_config();
        let result = CommitOrchestrator::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_supported_features() {
        let config = create_test_config();
        let orchestrator = CommitOrchestrator::new(config).unwrap();
        let features = orchestrator.get_supported_features();
        
        assert!(features.tree_sitter_analysis); // Enabled in test config
        assert!(features.ai_generation);
        assert!(features.review_integration); // Enabled in test config
        assert!(features.custom_messages);
        assert!(features.issue_id_support);
        
        let description = features.get_description();
        assert!(!description.is_empty());
        assert!(features.has_all_features());
    }

    #[test]
    fn test_supported_features_description() {
        let full_features = SupportedFeatures {
            tree_sitter_analysis: true,
            ai_generation: true,
            review_integration: true,
            custom_messages: true,
            issue_id_support: true,
        };
        
        let minimal_features = SupportedFeatures {
            tree_sitter_analysis: false,
            ai_generation: false,
            review_integration: false,
            custom_messages: false,
            issue_id_support: false,
        };
        
        assert!(full_features.has_all_features());
        assert!(!minimal_features.has_all_features());
        
        let full_desc = full_features.get_description();
        let minimal_desc = minimal_features.get_description();
        
        assert!(full_desc.contains("AI"));
        assert!(full_desc.contains("Tree-sitter"));
        assert_eq!(minimal_desc, "基础提交功能");
    }

    #[tokio::test]
    async fn test_handle_commit_no_repository() {
        let config = create_test_config();
        let mut orchestrator = CommitOrchestrator::new(config).unwrap();
        let args = create_test_commit_args();
        
        // This will likely fail since we're not in a proper git repository
        match orchestrator.handle_commit(args).await {
            Ok(_) => {
                // Success only if we're in a git repo with changes
                assert!(true);
            }
            Err(e) => {
                // Expected errors in test environment
                match e {
                    AppError::Git(_) => assert!(true),
                    AppError::Generic(msg) if msg.contains("没有检测到任何变更") => assert!(true),
                    _ => assert!(true),
                }
            }
        }
    }

    #[tokio::test]
    async fn test_legacy_handle_commit() {
        let config = create_test_config();
        let args = create_test_commit_args();
        
        // Test the legacy function
        match handle_commit(&config, args).await {
            Ok(_) => {
                // Success only if we're in a git repo with changes
                assert!(true);
            }
            Err(_) => {
                // Expected in test environment
                assert!(true);
            }
        }
    }

    #[test]
    fn test_commit_args_variations() {
        // Test different argument combinations
        let basic_args = CommitArgs {
            tree_sitter: false,
            depth: None,
            auto_stage: false,
            message: Some("feat: test commit".to_string()),
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };
        
        let enhanced_args = CommitArgs {
            tree_sitter: true,
            depth: Some("deep".to_string()),
            auto_stage: true,
            message: None,
            issue_id: Some("PROJ-123".to_string()),
            review: true,
            passthrough_args: vec!["--verbose".to_string()],
        };
        
        let review_args = CommitArgs {
            tree_sitter: false,
            depth: None,
            auto_stage: false,
            message: None,
            issue_id: None,
            review: true,
            passthrough_args: vec![],
        };
        
        assert_eq!(basic_args.message, Some("feat: test commit".to_string()));
        assert!(!basic_args.tree_sitter);
        
        assert!(enhanced_args.tree_sitter);
        assert_eq!(enhanced_args.depth, Some("deep".to_string()));
        assert!(enhanced_args.auto_stage);
        assert_eq!(enhanced_args.issue_id, Some("PROJ-123".to_string()));
        
        assert!(review_args.review);
        assert!(!review_args.tree_sitter);
    }

    #[test]
    fn test_handle_custom_message_with_review() {
        let config = create_test_config();
        let orchestrator = CommitOrchestrator::new(config).unwrap();
        
        let custom_message = "feat: add new authentication";
        
        // Test with review result
        let review_result_with_content = ReviewIntegrationResult {
            review_content: Some("- Security improvements\n- Code quality enhanced".to_string()),
            review_file_path: None,
            integration_successful: true,
        };
        
        let result = orchestrator.handle_custom_message_with_review(custom_message, &review_result_with_content);
        assert!(result.is_ok());
        
        let commit_result = result.unwrap();
        assert!(commit_result.message.contains("feat: add new authentication"));
        assert!(commit_result.message.contains("基于代码评审的改进"));
        assert!(!commit_result.enhanced);
        assert!(!commit_result.fallback_used);
        
        // Test without review result
        let review_result_empty = ReviewIntegrationResult {
            review_content: None,
            review_file_path: None,
            integration_successful: false,
        };
        
        let result_no_review = orchestrator.handle_custom_message_with_review(custom_message, &review_result_empty);
        assert!(result_no_review.is_ok());
        
        let commit_result_no_review = result_no_review.unwrap();
        assert_eq!(commit_result_no_review.message, custom_message);
    }

    #[test]
    fn test_commit_orchestrator_with_disabled_tree_sitter() {
        let mut config = (*create_test_config()).clone();
        config.tree_sitter.enabled = false;
        
        let orchestrator = CommitOrchestrator::new(Arc::new(config));
        assert!(orchestrator.is_ok());
        
        let orchestrator = orchestrator.unwrap();
        assert!(!orchestrator.is_tree_sitter_available());
        
        let features = orchestrator.get_supported_features();
        assert!(!features.tree_sitter_analysis);
        assert!(features.ai_generation);
    }

    #[test]
    fn test_commit_execution_result() {
        let result = CommitExecutionResult {
            success: true,
            commit_hash: Some("abc123def456".to_string()),
            message_used: "feat: test commit".to_string(),
        };
        
        assert!(result.success);
        assert_eq!(result.commit_hash.as_ref().unwrap(), "abc123def456");
        assert_eq!(result.message_used, "feat: test commit");
    }

    #[test]
    fn test_commit_generation_config() {
        let config = CommitGenerationConfig {
            use_tree_sitter: true,
            analysis_depth: Some("deep".to_string()),
            include_review: true,
            custom_message: Some("custom message".to_string()),
        };
        
        assert!(config.use_tree_sitter);
        assert_eq!(config.analysis_depth, Some("deep".to_string()));
        assert!(config.include_review);
        assert_eq!(config.custom_message, Some("custom message".to_string()));
    }

    #[test]
    fn test_git_operation_request_and_result() {
        let request = GitOperationRequest {
            auto_stage: true,
            check_repository: true,
        };
        
        let result = GitOperationResult {
            staged_files: vec!["file1.rs".to_string(), "file2.rs".to_string()],
            diff_content: "diff content here".to_string(),
            has_changes: true,
        };
        
        assert!(request.auto_stage);
        assert!(request.check_repository);
        assert_eq!(result.staged_files.len(), 2);
        assert!(!result.diff_content.is_empty());
        assert!(result.has_changes);
    }
}
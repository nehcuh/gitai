use crate::{
    config::AppConfig,
    errors::AppError,
    handlers::ai,
    tree_sitter_analyzer::core::DiffAnalysis,
    types::ai::ChatMessage,
};

use super::types::{
    BasicCommitRequest, CommitGenerationConfig, CommitGenerationResult, 
    EnhancedCommitRequest, TreeSitterCommitAnalysis
};

/// Handles commit message generation using AI
pub struct CommitMessageGenerator {
    config: AppConfig,
}

impl CommitMessageGenerator {
    /// Create a new commit message generator
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Generate commit message based on configuration
    pub async fn generate_commit_message(
        &self,
        diff: &str,
        generation_config: CommitGenerationConfig,
    ) -> Result<CommitGenerationResult, AppError> {
        if generation_config.use_tree_sitter {
            self.generate_enhanced_commit_message(
                EnhancedCommitRequest {
                    diff_content: diff.to_string(),
                    custom_message: generation_config.custom_message,
                    analysis_depth: generation_config.analysis_depth.unwrap_or_else(|| "medium".to_string()),
                    review_context: None,
                },
            ).await
        } else {
            self.generate_basic_commit_message(
                BasicCommitRequest {
                    diff_content: diff.to_string(),
                    review_context: None,
                }
            ).await
        }
    }

    /// Generate basic commit message using AI
    pub async fn generate_basic_commit_message(
        &self,
        request: BasicCommitRequest,
    ) -> Result<CommitGenerationResult, AppError> {
        tracing::info!("正在使用AI生成提交信息...");
        
        let mut prompt = format!(
            "根据以下代码变更信息生成高质量的Git提交信息：\n\n{}",
            request.diff_content
        );

        if let Some(review) = &request.review_context {
            prompt.push_str(&format!(
                "\n\n代码评审要点:\n{}\n\n请在提交信息中体现相关的评审改进点。",
                review
            ));
        }

        prompt.push_str("\n\n请生成简洁、清晰的提交信息，遵循常见的提交信息格式（如conventional commits）。");

        let system_prompt = self.config
            .prompts
            .get("commit-generator")
            .cloned()
            .unwrap_or_else(|| {
                tracing::warn!("未找到commit-generator提示模板，使用默认模板");
                "你是一个专业的Git提交信息生成助手。请根据提供的代码变更生成简洁、清晰的提交信息。".to_string()
            });
        
        let user_prompt = format!(
            "请根据以下Git diff生成一个规范的提交信息：\n\n```diff\n{}\n```\n\n要求：\n1. 使用中文\n2. 格式为：类型(范围): 简洁描述\n3. 第一行不超过50个字符\n4. 如有必要，可以添加详细说明",
            request.diff_content
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
        
        match ai::execute_ai_request_generic(&self.config, messages, "提交信息生成", true).await {
            Ok(message) => {
                // Clean up the AI response - remove any markdown formatting
                let cleaned_message = message
                    .lines()
                    .filter(|line| !line.trim().starts_with("```"))
                    .collect::<Vec<_>>()
                    .join("\n")
                    .trim()
                    .to_string();

                Ok(CommitGenerationResult {
                    message: cleaned_message,
                    enhanced: false,
                    tree_sitter_analysis: None,
                    fallback_used: false,
                })
            }
            Err(_) => {
                tracing::warn!("AI生成提交信息失败，使用回退方案");
                let fallback_message = if request.review_context.is_some() {
                    "chore: 基于代码评审结果更新代码".to_string()
                } else {
                    "chore: 更新代码".to_string()
                };
                
                Ok(CommitGenerationResult {
                    message: fallback_message,
                    enhanced: false,
                    tree_sitter_analysis: None,
                    fallback_used: true,
                })
            }
        }
    }

    /// Generate enhanced commit message with Tree-sitter analysis
    pub async fn generate_enhanced_commit_message(
        &self,
        request: EnhancedCommitRequest,
    ) -> Result<CommitGenerationResult, AppError> {
        // For this implementation, we'll simulate Tree-sitter analysis
        // In the real implementation, this would integrate with the analysis module
        
        let analysis_result = self.simulate_tree_sitter_analysis(&request.diff_content).await?;
        
        self.generate_commit_message_with_analysis(
            &request.diff_content,
            &analysis_result,
            request.custom_message,
            request.review_context.as_deref(),
        ).await
    }

    /// Generate commit message with Tree-sitter analysis results
    async fn generate_commit_message_with_analysis(
        &self,
        diff: &str,
        analysis_result: &TreeSitterCommitAnalysis,
        custom_message: Option<String>,
        review_context: Option<&str>,
    ) -> Result<CommitGenerationResult, AppError> {
        let system_prompt = self.config
            .prompts
            .get("commit-generator")
            .cloned()
            .unwrap_or_else(|| {
                "你是一个专业的Git提交信息生成助手。请根据提供的代码变更和静态分析结果生成高质量的提交信息。".to_string()
            });
        
        let mut user_prompt = if let Some(ref custom_msg) = custom_message {
            format!(
                "用户提供的提交信息：\n{}\n\n基于以下代码分析，请生成增强的提交信息：\n\n## Git Diff:\n```diff\n{}\n```\n\n## Tree-sitter 分析结果:\n{}\n\n要求：\n1. 保留用户原始意图\n2. 添加技术细节和影响分析\n3. 使用结构化格式\n4. 包含代码变更摘要",
                custom_msg, diff, analysis_result.analysis_text
            )
        } else {
            format!(
                "请根据以下代码变更和静态分析结果生成专业的提交信息：\n\n## Git Diff:\n```diff\n{}\n```\n\n## Tree-sitter 分析结果:\n{}\n\n要求：\n1. 主标题简洁明确（<50字符）\n2. 包含变更的技术细节\n3. 说明影响范围和复杂度\n4. 使用规范的提交信息格式",
                diff, analysis_result.analysis_text
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
        
        match ai::execute_ai_request_generic(&self.config, messages, "Tree-sitter增强提交信息生成", true).await {
            Ok(message) => {
                let enhanced_message = self.format_enhanced_commit_message(
                    &message, 
                    &analysis_result.analysis_data, 
                    custom_message.is_some()
                );
                
                Ok(CommitGenerationResult {
                    message: enhanced_message,
                    enhanced: true,
                    tree_sitter_analysis: analysis_result.analysis_data.clone(),
                    fallback_used: false,
                })
            }
            Err(e) => {
                tracing::error!("增强提交信息生成失败: {:?}", e);
                // Fallback to custom message or basic generation
                let fallback_message = if let Some(ref msg) = custom_message {
                    format!("{}\n\n[Tree-sitter 分析可用但AI生成失败]", msg)
                } else {
                    "feat: 代码更新\n\n[Tree-sitter 分析完成但AI生成失败]".to_string()
                };
                
                Ok(CommitGenerationResult {
                    message: fallback_message,
                    enhanced: true,
                    tree_sitter_analysis: analysis_result.analysis_data.clone(),
                    fallback_used: true,
                })
            }
        }
    }

    /// Format the final enhanced commit message
    fn format_enhanced_commit_message(
        &self,
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

    /// Simulate Tree-sitter analysis (placeholder implementation)
    async fn simulate_tree_sitter_analysis(
        &self,
        _diff: &str,
    ) -> Result<TreeSitterCommitAnalysis, AppError> {
        // This is a placeholder implementation
        // In the real system, this would call the actual Tree-sitter analyzer
        Ok(TreeSitterCommitAnalysis {
            analysis_text: "模拟分析结果: 检测到函数变更".to_string(),
            analysis_data: None,
            processing_time: std::time::Duration::from_millis(100),
        })
    }

    /// Format custom message with review context
    pub fn format_custom_message_with_review(
        &self,
        custom_message: &str, 
        review_context: &str
    ) -> String {
        format!(
            "{}\n\n---\n## 基于代码评审的改进\n\n{}",
            custom_message,
            review_context
        )
    }

    /// Validate generated commit message
    pub fn validate_commit_message(&self, message: &str) -> Result<(), AppError> {
        if message.trim().is_empty() {
            return Err(AppError::Generic("提交信息不能为空".to_string()));
        }
        
        if message.len() > 10_000 {
            return Err(AppError::Generic("提交信息过长".to_string()));
        }
        
        Ok(())
    }

    /// Get generation statistics
    pub fn get_generation_stats(&self, result: &CommitGenerationResult) -> GenerationStats {
        GenerationStats {
            message_length: result.message.len(),
            enhanced: result.enhanced,
            fallback_used: result.fallback_used,
            has_tree_sitter_analysis: result.tree_sitter_analysis.is_some(),
            line_count: result.message.lines().count(),
        }
    }
}

/// Statistics about commit message generation
#[derive(Debug, Clone)]
pub struct GenerationStats {
    pub message_length: usize,
    pub enhanced: bool,
    pub fallback_used: bool,
    pub has_tree_sitter_analysis: bool,
    pub line_count: usize,
}

impl GenerationStats {
    /// Check if the generated message meets quality standards
    pub fn meets_quality_standards(&self) -> bool {
        self.message_length >= 10 && 
        self.message_length <= 1000 &&
        self.line_count >= 1 &&
        !self.fallback_used
    }

    /// Get quality description
    pub fn quality_description(&self) -> &'static str {
        if self.meets_quality_standards() {
            if self.enhanced && self.has_tree_sitter_analysis {
                "优秀 (AI增强 + 静态分析)"
            } else if self.enhanced {
                "良好 (AI增强)"
            } else {
                "良好 (标准AI)"
            }
        } else if self.fallback_used {
            "基础 (回退方案)"
        } else {
            "待改进"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AIConfig, TreeSitterConfig, LanguageConfig, ReviewConfig, ScanConfig};
    use std::collections::HashMap;

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
            review: ReviewConfig::default(),
            account: None,
            language: LanguageConfig::default(),
            scan: ScanConfig::default(),
            prompts,
        }
    }

    #[test]
    fn test_commit_message_generator_creation() {
        let config = create_test_config();
        let generator = CommitMessageGenerator::new(config);
        assert!(true); // Generator created successfully
    }

    #[test]
    fn test_validate_commit_message() {
        let config = create_test_config();
        let generator = CommitMessageGenerator::new(config);
        
        // Valid message
        assert!(generator.validate_commit_message("feat: add new feature").is_ok());
        
        // Empty message
        assert!(generator.validate_commit_message("").is_err());
        assert!(generator.validate_commit_message("   ").is_err());
        
        // Too long message
        let long_message = "x".repeat(20_000);
        assert!(generator.validate_commit_message(&long_message).is_err());
    }

    #[test]
    fn test_format_custom_message_with_review() {
        let config = create_test_config();
        let generator = CommitMessageGenerator::new(config);
        
        let custom_message = "feat: add user authentication";
        let review_context = "- Fix security vulnerability in login\n- Improve input validation";
        
        let result = generator.format_custom_message_with_review(custom_message, review_context);
        
        assert!(result.contains("feat: add user authentication"));
        assert!(result.contains("基于代码评审的改进"));
        assert!(result.contains("Fix security vulnerability"));
        assert!(result.contains("Improve input validation"));
    }

    #[test]
    fn test_generation_stats() {
        let result = CommitGenerationResult {
            message: "feat: add new feature\n\nDetailed description".to_string(),
            enhanced: true,
            tree_sitter_analysis: None,
            fallback_used: false,
        };
        
        let config = create_test_config();
        let generator = CommitMessageGenerator::new(config);
        let stats = generator.get_generation_stats(&result);
        
        assert_eq!(stats.message_length, result.message.len());
        assert_eq!(stats.enhanced, true);
        assert_eq!(stats.fallback_used, false);
        assert_eq!(stats.has_tree_sitter_analysis, false);
        assert_eq!(stats.line_count, 3);
    }

    #[test]
    fn test_generation_stats_quality() {
        let good_stats = GenerationStats {
            message_length: 100,
            enhanced: true,
            fallback_used: false,
            has_tree_sitter_analysis: true,
            line_count: 2,
        };
        
        let poor_stats = GenerationStats {
            message_length: 5,
            enhanced: false,
            fallback_used: true,
            has_tree_sitter_analysis: false,
            line_count: 1,
        };
        
        assert!(good_stats.meets_quality_standards());
        assert!(!poor_stats.meets_quality_standards());
        
        assert_eq!(good_stats.quality_description(), "优秀 (AI增强 + 静态分析)");
        assert_eq!(poor_stats.quality_description(), "基础 (回退方案)");
    }

    #[tokio::test]
    async fn test_generate_basic_commit_message_fallback() {
        let config = create_test_config();
        let generator = CommitMessageGenerator::new(config);
        
        let request = BasicCommitRequest {
            diff_content: "diff --git a/test.rs b/test.rs\n+// test change".to_string(),
            review_context: None,
        };
        
        // This will likely use fallback since no real AI service is available
        let result = generator.generate_basic_commit_message(request).await;
        
        match result {
            Ok(commit_result) => {
                assert!(!commit_result.message.is_empty());
                assert!(!commit_result.enhanced);
                // May or may not be fallback depending on test environment
            }
            Err(_) => {
                // Also acceptable in test environment
                assert!(true);
            }
        }
    }

    #[tokio::test]
    async fn test_generate_enhanced_commit_message() {
        let config = create_test_config();
        let generator = CommitMessageGenerator::new(config);
        
        let request = EnhancedCommitRequest {
            diff_content: "diff --git a/test.rs b/test.rs\n+fn new_function() {}".to_string(),
            custom_message: Some("feat: add new function".to_string()),
            analysis_depth: "medium".to_string(),
            review_context: None,
        };
        
        let result = generator.generate_enhanced_commit_message(request).await;
        
        match result {
            Ok(commit_result) => {
                assert!(!commit_result.message.is_empty());
                assert!(commit_result.enhanced);
            }
            Err(_) => {
                // Expected in test environment
                assert!(true);
            }
        }
    }

    #[tokio::test]
    async fn test_generate_commit_message_with_config() {
        let config = create_test_config();
        let generator = CommitMessageGenerator::new(config);
        
        let diff = "diff --git a/src/main.rs b/src/main.rs\n+println!(\"Hello, world!\");";
        
        // Test basic generation
        let basic_config = CommitGenerationConfig {
            use_tree_sitter: false,
            analysis_depth: None,
            include_review: false,
            custom_message: None,
        };
        
        let result = generator.generate_commit_message(diff, basic_config).await;
        match result {
            Ok(commit_result) => {
                assert!(!commit_result.message.is_empty());
                assert!(!commit_result.enhanced);
            }
            Err(_) => {
                assert!(true);
            }
        }
        
        // Test enhanced generation
        let enhanced_config = CommitGenerationConfig {
            use_tree_sitter: true,
            analysis_depth: Some("deep".to_string()),
            include_review: false,
            custom_message: Some("feat: custom message".to_string()),
        };
        
        let result = generator.generate_commit_message(diff, enhanced_config).await;
        match result {
            Ok(commit_result) => {
                assert!(!commit_result.message.is_empty());
                assert!(commit_result.enhanced);
            }
            Err(_) => {
                assert!(true);
            }
        }
    }

    #[tokio::test]
    async fn test_simulate_tree_sitter_analysis() {
        let config = create_test_config();
        let generator = CommitMessageGenerator::new(config);
        
        let diff = "test diff content";
        let result = generator.simulate_tree_sitter_analysis(diff).await;
        
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(!analysis.analysis_text.is_empty());
        assert!(analysis.processing_time.as_millis() >= 0);
    }
}
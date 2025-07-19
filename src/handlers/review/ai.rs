use crate::{
    config::AppConfig,
    errors::{AIError, AppError},
    handlers::analysis::AIAnalysisEngine,
    handlers::ai::execute_review_request_with_language,
};
use std::sync::Arc;
use tracing;

use super::types::{
    AIAnalysisResult, AnalysisType, EnhancedAnalysisRequest, PromptRequest,
    StandardReviewRequest,
};

/// Handles AI integration for code review
pub struct AIReviewEngine {
    config: Arc<AppConfig>,
    analysis_engine: Arc<AIAnalysisEngine>,
}

impl AIReviewEngine {
    /// Create a new AI review engine
    pub fn new(config: Arc<AppConfig>, analysis_engine: Arc<AIAnalysisEngine>) -> Self {
        Self {
            config,
            analysis_engine,
        }
    }

    /// Perform enhanced AI analysis with work items
    pub async fn perform_enhanced_analysis(
        &self,
        request: EnhancedAnalysisRequest,
    ) -> Result<AIAnalysisResult, AppError> {
        tracing::info!("执行增强 AI 分析");

        let prompt_request = PromptRequest {
            diff_text: request.diff_text.clone(),
            analysis_text: "".to_string(), // Will be filled by prompt generator
            work_items: request.work_items,
            language_info: request.language_info,
            enhanced: true,
        };

        let prompt = self.generate_enhanced_prompt(&prompt_request)?;

        match self.send_to_ai(&prompt, request.args.language.as_deref()).await {
            Ok(content) => Ok(AIAnalysisResult {
                content,
                is_fallback: false,
                analysis_type: AnalysisType::Enhanced,
            }),
            Err(e) => {
                tracing::warn!("增强 AI 分析失败: {:?}", e);
                // Generate fallback review
                let fallback_content = self.generate_fallback_review(&prompt_request);
                Ok(AIAnalysisResult {
                    content: fallback_content,
                    is_fallback: true,
                    analysis_type: AnalysisType::Fallback,
                })
            }
        }
    }

    /// Perform standard AI review
    pub async fn perform_standard_review(
        &self,
        request: StandardReviewRequest,
    ) -> Result<AIAnalysisResult, AppError> {
        tracing::info!("执行标准 AI 审查");

        let prompt_request = PromptRequest {
            diff_text: request.diff_text.clone(),
            analysis_text: request.analysis_text,
            work_items: Vec::new(),
            language_info: request.language_info,
            enhanced: false,
        };

        let prompt = self.generate_standard_prompt(&prompt_request)?;

        match self.send_to_ai(&prompt, None).await {
            Ok(content) => Ok(AIAnalysisResult {
                content,
                is_fallback: false,
                analysis_type: AnalysisType::Standard,
            }),
            Err(e) => {
                tracing::warn!("标准 AI 审查失败: {:?}", e);
                // Generate fallback review
                let fallback_content = self.generate_fallback_review(&prompt_request);
                Ok(AIAnalysisResult {
                    content: fallback_content,
                    is_fallback: true,
                    analysis_type: AnalysisType::Fallback,
                })
            }
        }
    }

    /// Send request to AI service
    pub async fn send_to_ai(&self, prompt: &str, language: Option<&str>) -> Result<String, AIError> {
        tracing::debug!("发送请求到 AI 服务");

        // Get effective language for prompt selection
        let effective_language_string = match language {
            Some(lang) => {
                let lang_str = lang.to_string();
                self.config.get_output_language(Some(&lang_str))
            }
            None => self.config.get_output_language(None)
        };
        let effective_language = effective_language_string.as_str();
        
        // Load language-specific system prompt from configuration
        let system_prompt = self.config
            .get_language_prompt_content("review", &effective_language)
            .unwrap_or_else(|e| {
                tracing::warn!("获取{}语言的review prompt失败: {}，尝试使用默认prompt", effective_language, e);
                self.config.prompts
                    .get("review")
                    .cloned()
                    .unwrap_or_else(|| {
                        tracing::warn!("未找到默认review prompt，使用内置prompt");
                        "你是一个专业的代码审查助手，请提供详细的代码审查反馈。".to_string()
                    })
            });

        match execute_review_request_with_language(&self.config, &system_prompt, prompt, language).await {
            Ok(response) => {
                tracing::info!("AI 分析完成");
                Ok(response)
            }
            Err(e) => {
                tracing::error!("AI 请求失败: {:?}", e);
                Err(e)
            }
        }
    }

    /// Generate enhanced prompt with work items
    fn generate_enhanced_prompt(&self, request: &PromptRequest) -> Result<String, AppError> {
        let mut prompt = String::new();

        prompt.push_str("## Enhanced Code Review with Work Item Context\n\n");

        // Add work item context
        if !request.work_items.is_empty() {
            prompt.push_str("### Related Work Items:\n");
            for item in &request.work_items {
                prompt.push_str(&format!(
                    "- **{}** (ID: {}): {}\n",
                    item.issue_type_detail.name, item.id, item.name
                ));
                prompt.push_str(&format!("  Description: {}\n", item.description));
            }
            prompt.push_str("\n");
        }

        // Add language context
        prompt.push_str(&format!("### Language Context: {}\n\n", request.language_info));

        // Add diff content
        prompt.push_str("### Code Changes:\n");
        prompt.push_str("```diff\n");
        prompt.push_str(&request.diff_text);
        prompt.push_str("\n```\n\n");

        // Add analysis request
        prompt.push_str("### Review Requirements:\n");
        prompt.push_str("Please provide a comprehensive code review that:\n");
        prompt.push_str("1. Analyzes the code changes in the context of the related work items\n");
        prompt.push_str("2. Identifies potential issues, improvements, and best practices\n");
        prompt.push_str("3. Evaluates whether the changes align with the work item requirements\n");
        prompt.push_str("4. Provides specific, actionable feedback\n");
        prompt.push_str("5. Considers security, performance, and maintainability aspects\n");

        Ok(prompt)
    }

    /// Generate standard prompt
    fn generate_standard_prompt(&self, request: &PromptRequest) -> Result<String, AppError> {
        let mut prompt = String::new();

        prompt.push_str("## Code Review Request\n\n");

        // Add language context
        prompt.push_str(&format!("### Language Context: {}\n\n", request.language_info));

        // Add analysis if available
        if !request.analysis_text.is_empty() {
            prompt.push_str("### Code Analysis:\n");
            prompt.push_str(&request.analysis_text);
            prompt.push_str("\n\n");
        }

        // Add diff content
        prompt.push_str("### Code Changes:\n");
        prompt.push_str("```diff\n");
        prompt.push_str(&request.diff_text);
        prompt.push_str("\n```\n\n");

        // Add review request
        prompt.push_str("### Review Requirements:\n");
        prompt.push_str("Please provide a comprehensive code review that:\n");
        prompt.push_str("1. Identifies potential issues and improvements\n");
        prompt.push_str("2. Suggests best practices and coding standards\n");
        prompt.push_str("3. Evaluates code quality, readability, and maintainability\n");
        prompt.push_str("4. Provides specific, actionable feedback\n");

        Ok(prompt)
    }

    /// Generate fallback review when AI is unavailable
    fn generate_fallback_review(&self, request: &PromptRequest) -> String {
        let mut review = String::new();

        review.push_str("# 代码审查报告 (离线模式)\n\n");
        review.push_str("⚠️ **注意**: AI 服务暂时不可用，以下是基于静态分析的基础审查报告。\n\n");

        // Add basic information
        review.push_str(&format!("## 基本信息\n"));
        review.push_str(&format!("- 语言类型: {}\n", request.language_info));
        review.push_str(&format!("- 分析时间: {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        // Add work item context if available
        if !request.work_items.is_empty() {
            review.push_str("\n## 相关工作项\n");
            for item in &request.work_items {
                review.push_str(&format!("- {} (ID: {}): {}\n", item.issue_type_detail.name, item.id, item.name));
            }
        }

        // Add static analysis
        if !request.analysis_text.is_empty() {
            review.push_str("\n## 静态分析结果\n");
            review.push_str(&request.analysis_text);
        }

        // Add basic recommendations
        review.push_str("\n## 建议\n");
        review.push_str("1. 🔍 请手动审查代码变更确保符合项目标准\n");
        review.push_str("2. 🧪 确保所有相关测试用例已更新并通过\n");
        review.push_str("3. 📚 检查是否需要更新相关文档\n");
        review.push_str("4. 🔒 验证安全性和性能影响\n");
        review.push_str("\n💡 建议稍后重试以获取完整的 AI 分析报告。\n");

        review
    }

    /// Check if AI service is available
    pub async fn is_available(&self) -> bool {
        // Simple check - could be expanded to ping the AI service
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::git::ReviewArgs;

    fn create_test_request() -> PromptRequest {
        PromptRequest {
            diff_text: "test diff content".to_string(),
            analysis_text: "test analysis".to_string(),
            work_items: vec![],
            language_info: "Rust (2 个文件)".to_string(),
            enhanced: false,
        }
    }

    #[test]
    fn test_generate_standard_prompt() {
        let config = Arc::new(AppConfig::from_partial_and_env(
            None, 
            std::collections::HashMap::new(),
            std::collections::HashMap::new()
        ).unwrap());
        let ai_engine = Arc::new(AIAnalysisEngine::new(config.clone()));
        let engine = AIReviewEngine::new(config, ai_engine);

        let request = create_test_request();
        let prompt = engine.generate_standard_prompt(&request);

        assert!(prompt.is_ok());
        let prompt_text = prompt.unwrap();
        assert!(prompt_text.contains("Code Review Request"));
        assert!(prompt_text.contains("Rust (2 个文件)"));
        assert!(prompt_text.contains("test diff content"));
    }

    #[test]
    fn test_generate_fallback_review() {
        let config = Arc::new(AppConfig::from_partial_and_env(
            None, 
            std::collections::HashMap::new(),
            std::collections::HashMap::new()
        ).unwrap());
        let ai_engine = Arc::new(AIAnalysisEngine::new(config.clone()));
        let engine = AIReviewEngine::new(config, ai_engine);

        let request = create_test_request();
        let review = engine.generate_fallback_review(&request);

        assert!(review.contains("代码审查报告 (离线模式)"));
        assert!(review.contains("AI 服务暂时不可用"));
        assert!(review.contains("Rust (2 个文件)"));
    }
}
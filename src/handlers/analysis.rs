use crate::{
    config::AppConfig,
    errors::{AIError, AppError},
    handlers::ai::execute_review_request,
    types::{
        ai::{
            AnalysisDepth, AnalysisRequest, AnalysisResult, CodeQualityAnalysis, Deviation,
            DeviationSeverity, Recommendation, RequirementAnalysis, RiskAssessment,
        },
        devops::AnalysisWorkItem,
    },
};
use serde_json;
use std::sync::Arc;
use tracing;

/// AI-powered code analysis engine that combines DevOps work item descriptions
/// with Git diff content to provide intelligent requirement-implementation consistency analysis
pub struct AIAnalysisEngine {
    config: Arc<AppConfig>,
}

impl AIAnalysisEngine {
    /// Creates a new AI analysis engine with the provided configuration
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    /// Performs comprehensive analysis combining work items and code changes
    pub async fn analyze_with_requirements(
        &self,
        request: AnalysisRequest,
    ) -> Result<AnalysisResult, AppError> {
        tracing::info!(
            "Starting AI analysis with {} work items and {} chars of diff",
            request.work_items.len(),
            request.git_diff.len()
        );

        // Build enhanced analysis prompt
        let enhanced_prompt = self.build_requirement_analysis_prompt(&request)?;
        
        tracing::debug!("Built analysis prompt, length: {} chars", enhanced_prompt.len());

        // Execute AI analysis
        let ai_response = self.execute_ai_analysis(&enhanced_prompt).await?;
        
        tracing::debug!("Received AI response, length: {} chars", ai_response.len());

        // Parse and structure AI response
        let analysis_result = self.parse_analysis_response(&ai_response, &request)?;
        
        tracing::info!(
            "Analysis completed successfully, overall score: {}",
            analysis_result.overall_score
        );

        Ok(analysis_result)
    }

    /// Builds a comprehensive prompt for requirement analysis
    fn build_requirement_analysis_prompt(&self, request: &AnalysisRequest) -> Result<String, AppError> {
        let work_items_description = self.format_work_items_for_analysis(&request.work_items)?;
        let focus_areas_text = self.format_focus_areas(&request.focus_areas);
        let analysis_depth_instruction = self.get_analysis_depth_instruction(&request.analysis_depth);

        let prompt = format!(
            r#"你是一位资深的代码评审专家和需求分析师。请分析以下代码变更与业务需求的一致性。

## 工作项信息
{work_items_description}

## 代码变更
```diff
{git_diff}
```

## 分析要求
{analysis_depth_instruction}

{focus_areas_text}

请从以下维度进行详细分析：

### 1. 需求实现完整性 (0-100分)
- 代码是否完整实现了所有需求功能
- 是否存在需求遗漏或功能缺失
- 实现是否超出了需求范围
- 验收标准是否得到满足

### 2. 业务逻辑正确性 (0-100分)
- 代码逻辑是否符合业务规则
- 边界条件和异常情况处理是否恰当
- 用户体验是否符合预期
- 数据流和状态管理是否正确

### 3. 技术实现质量 (0-100分)
- 代码结构和设计是否合理
- 性能和安全性考虑是否充分
- 可维护性和扩展性评估
- 编码规范和最佳实践检查

### 4. 偏离度分析
- 量化实现与需求的匹配程度（总体评分0-100）
- 识别主要的偏离点和风险
- 按严重程度分类问题（Critical/High/Medium/Low）
- 提供具体的改进建议和优先级

请以JSON格式输出分析结果，包含以下结构：
```json
{{
  "overall_score": 85,
  "requirement_consistency": {{
    "completion_score": 80,
    "accuracy_score": 90,
    "missing_features": ["特性A", "特性B"],
    "extra_implementations": ["额外功能C"]
  }},
  "code_quality": {{
    "quality_score": 85,
    "maintainability_score": 80,
    "performance_score": 75,
    "security_score": 90,
    "structure_assessment": "代码结构评估说明"
  }},
  "deviations": [
    {{
      "severity": "High",
      "category": "Logic Error",
      "description": "问题描述",
      "file_location": "src/example.rs:42",
      "suggestion": "修复建议"
    }}
  ],
  "recommendations": [
    {{
      "priority": 1,
      "title": "建议标题",
      "description": "详细描述",
      "expected_impact": "预期影响",
      "effort_estimate": "Medium"
    }}
  ],
  "risk_assessment": {{
    "risk_level": "Medium",
    "business_impact": "业务影响评估",
    "technical_risks": ["技术风险1", "技术风险2"],
    "mitigation_strategies": ["缓解策略1", "缓解策略2"]
  }}
}}
```

请确保分析客观、准确，提供可执行的建议。"#,
            work_items_description = work_items_description,
            git_diff = request.git_diff,
            analysis_depth_instruction = analysis_depth_instruction,
            focus_areas_text = focus_areas_text
        );

        Ok(prompt)
    }

    /// Formats work items for inclusion in the analysis prompt
    fn format_work_items_for_analysis(&self, work_items: &[AnalysisWorkItem]) -> Result<String, AppError> {
        if work_items.is_empty() {
            return Ok("无关联的工作项信息".to_string());
        }

        let mut formatted = String::new();
        
        for (index, item) in work_items.iter().enumerate() {
            formatted.push_str(&format!("\n### 工作项 {} - {}\n", 
                index + 1, 
                item.item_type_name.as_deref().unwrap_or("未知类型")
            ));
            
            if let Some(id) = item.id {
                formatted.push_str(&format!("**ID**: {}\n", id));
            }
            
            if let Some(code) = item.code {
                formatted.push_str(&format!("**编号**: {}\n", code));
            }
            
            if let Some(project) = &item.project_name {
                formatted.push_str(&format!("**项目**: {}\n", project));
            }
            
            if let Some(title) = &item.title {
                formatted.push_str(&format!("**标题**: {}\n", title));
            }
            
            if let Some(description) = &item.description {
                formatted.push_str(&format!("**描述**:\n{}\n", description));
            }
            
            formatted.push('\n');
        }

        Ok(formatted)
    }

    /// Formats focus areas for the analysis prompt
    fn format_focus_areas(&self, focus_areas: &Option<Vec<String>>) -> String {
        match focus_areas {
            Some(areas) if !areas.is_empty() => {
                format!(
                    "## 重点关注领域\n请特别关注以下方面：{}\n",
                    areas.join("、")
                )
            }
            _ => String::new(),
        }
    }

    /// Gets analysis depth specific instructions
    fn get_analysis_depth_instruction(&self, depth: &AnalysisDepth) -> &'static str {
        match depth {
            AnalysisDepth::Basic => "请进行基础分析，关注主要的功能实现和明显的问题。",
            AnalysisDepth::Normal => "请进行标准分析，全面评估实现质量和需求一致性。",
            AnalysisDepth::Deep => "请进行深度分析，详细检查代码逻辑、性能、安全性和最佳实践。",
        }
    }

    /// Executes the AI analysis request
    async fn execute_ai_analysis(&self, prompt: &str) -> Result<String, AppError> {
        tracing::debug!("Executing AI analysis request");
        
        // Use existing AI infrastructure from handlers/ai.rs
        match execute_review_request(&self.config, "You are a senior code reviewer and requirements analyst.", prompt).await {
            Ok(response) => {
                tracing::debug!("AI analysis completed successfully");
                Ok(response)
            }
            Err(AIError::RequestFailed(e)) => {
                tracing::error!("AI request failed during analysis: {}", e);
                Err(AppError::AI(AIError::RequestFailed(e)))
            }
            Err(AIError::ResponseParseFailed(e)) => {
                tracing::error!("AI response parse failed during analysis: {}", e);
                Err(AppError::AI(AIError::ResponseParseFailed(e)))
            }
            Err(AIError::ApiResponseError(status, msg)) => {
                tracing::error!("AI API response error during analysis: {} - {}", status, msg);
                Err(AppError::AI(AIError::ApiResponseError(status, msg)))
            }
            Err(e) => {
                tracing::error!("Unexpected AI error during analysis: {:?}", e);
                Err(AppError::AI(e))
            }
        }
    }

    /// Parses AI response into structured analysis result
    fn parse_analysis_response(&self, response: &str, request: &AnalysisRequest) -> Result<AnalysisResult, AppError> {
        tracing::debug!("Parsing AI analysis response");

        // Try to extract JSON from the response
        let json_content = self.extract_json_from_response(response)?;
        
        match serde_json::from_str::<AnalysisResult>(&json_content) {
            Ok(result) => {
                tracing::debug!("Successfully parsed structured analysis result");
                Ok(result)
            }
            Err(e) => {
                tracing::warn!("Failed to parse structured response, creating fallback result: {}", e);
                Ok(self.create_fallback_analysis_result(response, request))
            }
        }
    }

    /// Extracts JSON content from AI response
    fn extract_json_from_response(&self, response: &str) -> Result<String, AppError> {
        // Look for JSON block in the response
        if let Some(start) = response.find("```json") {
            let json_start = start + 7; // Skip "```json"
            if let Some(end_relative) = response[json_start..].find("```") {
                let json_end = json_start + end_relative;
                return Ok(response[json_start..json_end].trim().to_string());
            }
        }

        // Look for direct JSON object
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                if end > start {
                    return Ok(response[start..=end].to_string());
                }
            }
        }

        Err(AppError::Generic(
            "Could not extract JSON from AI response".to_string()
        ))
    }

    /// Creates a fallback analysis result when AI response cannot be parsed
    fn create_fallback_analysis_result(&self, response: &str, _request: &AnalysisRequest) -> AnalysisResult {
        tracing::info!("Creating fallback analysis result");
        
        // Basic scoring based on response content analysis
        let overall_score = self.estimate_score_from_text(response);
        
        AnalysisResult {
            overall_score,
            requirement_consistency: RequirementAnalysis {
                completion_score: overall_score.saturating_sub(5),
                accuracy_score: overall_score,
                missing_features: vec!["需要进一步手动分析".to_string()],
                extra_implementations: vec![],
            },
            code_quality: CodeQualityAnalysis {
                quality_score: overall_score,
                maintainability_score: overall_score.saturating_sub(5),
                performance_score: overall_score.saturating_sub(10),
                security_score: overall_score.saturating_sub(5),
                structure_assessment: "基于AI文本响应的自动评估，建议人工复核".to_string(),
            },
            deviations: vec![
                Deviation {
                    severity: DeviationSeverity::Medium,
                    category: "Analysis Limitation".to_string(),
                    description: "AI响应格式无法完全解析，建议人工复核分析结果".to_string(),
                    file_location: None,
                    suggestion: "重新运行分析或联系技术支持".to_string(),
                }
            ],
            recommendations: vec![
                Recommendation {
                    priority: 2,
                    title: "完善分析结果".to_string(),
                    description: "由于AI响应解析问题，建议进行人工代码评审以补充分析结果".to_string(),
                    expected_impact: "提高代码质量和需求一致性评估准确性".to_string(),
                    effort_estimate: "Medium".to_string(),
                }
            ],
            risk_assessment: RiskAssessment {
                risk_level: DeviationSeverity::Medium,
                business_impact: "分析结果可能不完整，建议谨慎采纳建议".to_string(),
                technical_risks: vec!["分析准确性受限".to_string()],
                mitigation_strategies: vec!["进行人工复核".to_string(), "重新运行分析".to_string()],
            },
        }
    }

    /// Estimates a score from text content analysis
    fn estimate_score_from_text(&self, text: &str) -> u8 {
        let positive_keywords = ["good", "correct", "implemented", "完成", "正确", "良好", "符合"];
        let negative_keywords = ["error", "missing", "incorrect", "错误", "缺失", "不正确", "问题"];
        
        let positive_count = positive_keywords.iter()
            .map(|keyword| text.matches(keyword).count())
            .sum::<usize>();
            
        let negative_count = negative_keywords.iter()
            .map(|keyword| text.matches(keyword).count())
            .sum::<usize>();
        
        // Basic scoring algorithm
        let base_score = 70;
        let positive_bonus = (positive_count * 3).min(20) as i32;
        let negative_penalty = (negative_count * 5).min(30) as i32;
        
        let score = base_score + positive_bonus - negative_penalty;
        score.clamp(0, 100) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AIConfig, AppConfig};

    fn create_test_config() -> Arc<AppConfig> {
        use std::collections::HashMap;
        
        Arc::new(AppConfig {
            ai: AIConfig {
                api_url: "http://localhost:11434/v1/chat/completions".to_string(),
                model_name: "test-model".to_string(),
                temperature: 0.7,
                api_key: Some("test-key".to_string()),
            },
            tree_sitter: Default::default(),
            review: Default::default(),
            account: None,
            prompts: HashMap::new(),
        })
    }

    fn create_test_work_item() -> AnalysisWorkItem {
        AnalysisWorkItem {
            id: Some(123),
            code: Some(99),
            project_name: Some("测试项目".to_string()),
            item_type_name: Some("用户故事".to_string()),
            title: Some("测试功能".to_string()),
            description: Some("实现测试功能的详细描述".to_string()),
        }
    }

    #[test]
    fn test_format_work_items_for_analysis() {
        let engine = AIAnalysisEngine::new(create_test_config());
        let work_items = vec![create_test_work_item()];
        
        let result = engine.format_work_items_for_analysis(&work_items).unwrap();
        
        assert!(result.contains("工作项 1"));
        assert!(result.contains("用户故事"));
        assert!(result.contains("测试功能"));
        assert!(result.contains("测试项目"));
    }

    #[test]
    fn test_format_focus_areas() {
        let engine = AIAnalysisEngine::new(create_test_config());
        
        let focus_areas = Some(vec!["安全性".to_string(), "性能".to_string()]);
        let result = engine.format_focus_areas(&focus_areas);
        
        assert!(result.contains("重点关注领域"));
        assert!(result.contains("安全性"));
        assert!(result.contains("性能"));
    }

    #[test]
    fn test_get_analysis_depth_instruction() {
        let engine = AIAnalysisEngine::new(create_test_config());
        
        let basic = engine.get_analysis_depth_instruction(&AnalysisDepth::Basic);
        let normal = engine.get_analysis_depth_instruction(&AnalysisDepth::Normal);
        let deep = engine.get_analysis_depth_instruction(&AnalysisDepth::Deep);
        
        assert!(basic.contains("基础分析"));
        assert!(normal.contains("标准分析"));
        assert!(deep.contains("深度分析"));
    }

    #[test]
    fn test_extract_json_from_response() {
        let engine = AIAnalysisEngine::new(create_test_config());
        
        let response_with_json_block = "这是一些文本\n```json\n{\"test\": \"value\"}\n```\n更多文本";
        let result = engine.extract_json_from_response(response_with_json_block).unwrap();
        assert_eq!(result, "{\"test\": \"value\"}");
        
        let response_with_direct_json = "一些文本 {\"direct\": \"json\"} 结束";
        let result = engine.extract_json_from_response(response_with_direct_json).unwrap();
        assert_eq!(result, "{\"direct\": \"json\"}");
    }

    #[test]
    fn test_estimate_score_from_text() {
        let engine = AIAnalysisEngine::new(create_test_config());
        
        let positive_text = "代码实现正确，功能完成良好，符合需求";
        let score = engine.estimate_score_from_text(positive_text);
        assert!(score > 70);
        
        let negative_text = "代码存在错误，功能缺失，实现不正确";
        let score = engine.estimate_score_from_text(negative_text);
        assert!(score < 70);
    }

    #[test]
    fn test_create_fallback_analysis_result() {
        let engine = AIAnalysisEngine::new(create_test_config());
        let request = AnalysisRequest {
            work_items: vec![create_test_work_item()],
            git_diff: "test diff".to_string(),
            focus_areas: None,
            analysis_depth: AnalysisDepth::Normal,
            output_format: crate::types::ai::OutputFormat::Json,
        };
        
        let result = engine.create_fallback_analysis_result("some response text", &request);
        
        assert!(result.overall_score <= 100);
        assert!(!result.deviations.is_empty());
        assert!(!result.recommendations.is_empty());
    }

    #[test]
    fn test_format_work_items_empty() {
        let engine = AIAnalysisEngine::new(create_test_config());
        let work_items = vec![];
        
        let result = engine.format_work_items_for_analysis(&work_items).unwrap();
        
        assert_eq!(result, "无关联的工作项信息");
    }

    #[test]
    fn test_format_work_items_multiple() {
        let engine = AIAnalysisEngine::new(create_test_config());
        let work_items = vec![
            create_test_work_item(),
            AnalysisWorkItem {
                id: Some(456),
                code: Some(200),
                project_name: Some("另一个项目".to_string()),
                item_type_name: Some("缺陷".to_string()),
                title: Some("修复错误".to_string()),
                description: Some("修复一个重要错误".to_string()),
            }
        ];
        
        let result = engine.format_work_items_for_analysis(&work_items).unwrap();
        
        assert!(result.contains("工作项 1"));
        assert!(result.contains("工作项 2"));
        assert!(result.contains("用户故事"));
        assert!(result.contains("缺陷"));
        assert!(result.contains("测试功能"));
        assert!(result.contains("修复错误"));
    }

    #[test]
    fn test_format_focus_areas_multiple() {
        let engine = AIAnalysisEngine::new(create_test_config());
        
        let focus_areas = Some(vec![
            "安全性".to_string(), 
            "性能".to_string(), 
            "可维护性".to_string()
        ]);
        let result = engine.format_focus_areas(&focus_areas);
        
        assert!(result.contains("重点关注领域"));
        assert!(result.contains("安全性、性能、可维护性"));
    }

    #[test]
    fn test_format_focus_areas_empty_vector() {
        let engine = AIAnalysisEngine::new(create_test_config());
        
        let focus_areas = Some(vec![]);
        let result = engine.format_focus_areas(&focus_areas);
        
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_json_complex_response() {
        let engine = AIAnalysisEngine::new(create_test_config());
        
        let complex_response = r#"
        这里是一些分析文本
        
        ```json
        {
          "overall_score": 85,
          "requirement_consistency": {
            "completion_score": 80,
            "accuracy_score": 90,
            "missing_features": ["特性A"],
            "extra_implementations": []
          }
        }
        ```
        
        更多分析内容
        "#;
        
        let result = engine.extract_json_from_response(complex_response).unwrap();
        assert!(result.contains("overall_score"));
        assert!(result.contains("requirement_consistency"));
    }

    #[test]
    fn test_extract_json_malformed_response() {
        let engine = AIAnalysisEngine::new(create_test_config());
        
        let malformed_response = "这里没有有效的JSON内容";
        let result = engine.extract_json_from_response(malformed_response);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_estimate_score_balanced_text() {
        let engine = AIAnalysisEngine::new(create_test_config());
        
        let balanced_text = "代码实现正确但存在一些错误，功能完成度良好，需要修复缺失的部分";
        let score = engine.estimate_score_from_text(balanced_text);
        
        // Should be around the base score since positive and negative balance out
        assert!(score >= 60);
        assert!(score <= 80);
    }

    #[test]
    fn test_estimate_score_very_negative_text() {
        let engine = AIAnalysisEngine::new(create_test_config());
        
        let negative_text = "代码错误很多，实现不正确，功能缺失严重，存在大量问题";
        let score = engine.estimate_score_from_text(negative_text);
        
        assert!(score < 60);
    }

    #[test]
    fn test_estimate_score_very_positive_text() {
        let engine = AIAnalysisEngine::new(create_test_config());
        
        let positive_text = "代码实现完美，功能完成优秀，质量良好，符合所有需求，正确无误";
        let score = engine.estimate_score_from_text(positive_text);
        
        assert!(score > 80);
    }

    #[test]
    fn test_build_requirement_analysis_prompt_with_focus() {
        let engine = AIAnalysisEngine::new(create_test_config());
        let request = AnalysisRequest {
            work_items: vec![create_test_work_item()],
            git_diff: "diff content".to_string(),
            focus_areas: Some(vec!["安全性".to_string(), "性能".to_string()]),
            analysis_depth: AnalysisDepth::Deep,
            output_format: crate::types::ai::OutputFormat::Json,
        };
        
        let result = engine.build_requirement_analysis_prompt(&request).unwrap();
        
        assert!(result.contains("工作项信息"));
        assert!(result.contains("代码变更"));
        assert!(result.contains("深度分析"));
        assert!(result.contains("重点关注领域"));
        assert!(result.contains("安全性、性能"));
        assert!(result.contains("diff content"));
    }

    #[test]
    fn test_build_requirement_analysis_prompt_basic_depth() {
        let engine = AIAnalysisEngine::new(create_test_config());
        let request = AnalysisRequest {
            work_items: vec![],
            git_diff: "simple diff".to_string(),
            focus_areas: None,
            analysis_depth: AnalysisDepth::Basic,
            output_format: crate::types::ai::OutputFormat::Text,
        };
        
        let result = engine.build_requirement_analysis_prompt(&request).unwrap();
        
        assert!(result.contains("基础分析"));
        assert!(result.contains("无关联的工作项信息"));
        assert!(!result.contains("重点关注领域"));
    }

    #[test]
    fn test_create_fallback_analysis_with_different_texts() {
        let engine = AIAnalysisEngine::new(create_test_config());
        let request = AnalysisRequest {
            work_items: vec![],
            git_diff: "test".to_string(),
            focus_areas: None,
            analysis_depth: AnalysisDepth::Normal,
            output_format: crate::types::ai::OutputFormat::Json,
        };
        
        // Test with very positive text
        let positive_result = engine.create_fallback_analysis_result("excellent code, good implementation, correct functionality", &request);
        
        // Test with very negative text  
        let negative_result = engine.create_fallback_analysis_result("terrible code, many errors, incorrect implementation", &request);
        
        // Positive text should yield higher score
        assert!(positive_result.overall_score > negative_result.overall_score);
        
        // Both should have fallback recommendations
        assert!(!positive_result.recommendations.is_empty());
        assert!(!negative_result.recommendations.is_empty());
    }
}
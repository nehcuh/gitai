use serde::{Deserialize, Serialize};
use crate::types::devops::AnalysisWorkItem;

/// Represents a chat message with a role and content
///
/// This structure is used for both requests to and responses from AI chat models
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct OpenAIChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub stream: bool,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[allow(dead_code)] // Add allow(dead_code) to suppress warnings for unused fields
pub struct OpenAIChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64, // Typically a UNIX Timestamp
    pub model: String,
    pub system_fingerprint: Option<String>, // This field exists based on the exampale provided
    pub choices: Vec<OpenAIChoice>,
    pub usage: OpenAIUsage,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[allow(dead_code)] // Add allow(dead_code) to suppress warnings for unused fields
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[allow(dead_code)] // Add allow(dead_code) to suppress warnings for unused fields
pub struct OpenAIChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
    // pub logprobs: Option<serde_json::Value> // If logprobs parsing is needed
}

// ============================================================================
// AI Analysis Types for Story 04: AI Analysis Integration
// ============================================================================

/// Output format options for analysis results
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
    Markdown,
    Html,
}

/// Severity levels for deviations found during analysis
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DeviationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Request structure for AI-powered code analysis
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalysisRequest {
    /// Collection of work items to be analyzed together
    pub work_items: Vec<AnalysisWorkItem>,
    /// Git diff content to analyze
    pub git_diff: String,
    /// Optional focus areas for analysis (e.g., "security", "performance")
    pub focus_areas: Option<Vec<String>>,
    /// Output format for results
    pub output_format: OutputFormat,
}

/// Comprehensive analysis result structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalysisResult {
    /// Overall score from 0-100 indicating requirement-implementation alignment
    pub overall_score: u8,
    /// Detailed requirement consistency analysis
    pub requirement_consistency: RequirementAnalysis,
    /// Code quality assessment
    pub code_quality: CodeQualityAnalysis,
    /// List of identified deviations and issues
    pub deviations: Vec<Deviation>,
    /// Actionable recommendations for improvement
    pub recommendations: Vec<Recommendation>,
    /// Risk assessment and impact analysis
    pub risk_assessment: RiskAssessment,
}

/// Analysis of requirement implementation consistency
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequirementAnalysis {
    /// Completion score (0-100) indicating how completely requirements are implemented
    pub completion_score: u8,
    /// Accuracy score (0-100) indicating correctness of implementation
    pub accuracy_score: u8,
    /// List of features missing from implementation
    pub missing_features: Vec<String>,
    /// List of implementations that exceed original requirements
    pub extra_implementations: Vec<String>,
}

/// Code quality assessment results
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeQualityAnalysis {
    /// Overall code quality score (0-100)
    pub quality_score: u8,
    /// Maintainability assessment (0-100)
    pub maintainability_score: u8,
    /// Performance considerations score (0-100)
    pub performance_score: u8,
    /// Security assessment score (0-100)
    pub security_score: u8,
    /// Code structure and design patterns assessment
    pub structure_assessment: String,
}

/// Individual deviation or issue found during analysis
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Deviation {
    /// Severity level of the deviation
    pub severity: DeviationSeverity,
    /// Category of the deviation (e.g., "Logic Error", "Missing Feature")
    pub category: String,
    /// Detailed description of the deviation
    pub description: String,
    /// Optional file location where the deviation was found
    pub file_location: Option<String>,
    /// Suggested action to address the deviation
    pub suggestion: String,
}

/// Actionable recommendation for improvement
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Recommendation {
    /// Priority level (1-5, where 1 is highest priority)
    pub priority: u8,
    /// Title of the recommendation
    pub title: String,
    /// Detailed description of the recommendation
    pub description: String,
    /// Expected impact of implementing the recommendation
    pub expected_impact: String,
    /// Estimated effort required (e.g., "Low", "Medium", "High")
    pub effort_estimate: String,
}

/// Risk assessment for the analyzed changes
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RiskAssessment {
    /// Overall risk level
    pub risk_level: DeviationSeverity,
    /// Business impact assessment
    pub business_impact: String,
    /// Technical risks identified
    pub technical_risks: Vec<String>,
    /// Mitigation strategies
    pub mitigation_strategies: Vec<String>,
}

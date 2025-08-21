use crate::types::git::GitDiff;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of diff analysis containing both text and structured analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffAnalysisResult {
    pub git_diff: GitDiff,
    pub analysis_text: String,
    pub tree_sitter_analysis: Option<TreeSitterAnalysis>,
    pub language_info: String,
}

/// TreeSitter analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeSitterAnalysis {
    pub structural_changes: String,
    pub complexity_metrics: HashMap<String, f64>,
    pub affected_nodes: Vec<String>,
}

/// Result of AI analysis
#[derive(Debug, Clone)]
pub struct AIAnalysisResult {
    pub content: String,
    pub is_fallback: bool,
    pub analysis_type: AnalysisType,
}

/// Types of analysis that can be performed
#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisType {
    Standard,
    Enhanced,
    Fallback,
}

/// Request for enhanced analysis with work items
#[derive(Debug, Clone)]
pub struct EnhancedAnalysisRequest {
    pub diff_text: String,
    pub analysis_text: String,
    pub work_items: Vec<crate::types::ai::WorkItem>,
    pub language_info: String,
}

/// Request for standard review
#[derive(Debug, Clone)]
pub struct StandardReviewRequest {
    pub diff_text: String,
    pub analysis_text: String,
    pub language_info: String,
}

/// Internal request structure for prompt generation
#[derive(Debug, Clone)]
pub struct PromptRequest {
    pub diff_text: String,
    pub analysis_text: String,
    pub work_items: Vec<crate::types::ai::WorkItem>,
    pub language_info: String,
    pub enhanced: bool,
}
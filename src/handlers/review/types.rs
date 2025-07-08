use crate::types::{
    devops::WorkItem,
    git::{GitDiff, ReviewArgs},
};
use std::collections::HashMap;

/// Context information for a review operation
#[derive(Debug, Clone)]
pub struct ReviewContext {
    pub args: ReviewArgs,
    pub work_items: Vec<WorkItem>,
    pub analysis_result: DiffAnalysisResult,
}

/// Result of diff analysis operations
#[derive(Debug, Clone)]
pub struct DiffAnalysisResult {
    pub git_diff: GitDiff,
    pub analysis_text: String,
    pub tree_sitter_analysis: Option<TreeSitterAnalysis>,
    pub language_info: String,
}

/// TreeSitter analysis result
#[derive(Debug, Clone)]
pub struct TreeSitterAnalysis {
    pub structural_changes: String,
    pub complexity_metrics: HashMap<String, f64>,
    pub affected_nodes: Vec<String>,
}

/// Request for enhanced AI analysis
#[derive(Debug, Clone)]
pub struct EnhancedAnalysisRequest {
    pub diff_text: String,
    pub work_items: Vec<WorkItem>,
    pub args: ReviewArgs,
    pub language_info: String,
}

/// Request for standard AI review
#[derive(Debug, Clone)]
pub struct StandardReviewRequest {
    pub diff_text: String,
    pub analysis_text: String,
    pub language_info: String,
}

/// Request for prompt generation
#[derive(Debug, Clone)]
pub struct PromptRequest {
    pub diff_text: String,
    pub analysis_text: String,
    pub work_items: Vec<WorkItem>,
    pub language_info: String,
    pub enhanced: bool,
}

/// Configuration for AI analysis
#[derive(Debug, Clone)]
pub struct AIAnalysisConfig {
    pub use_enhanced_analysis: bool,
    pub include_tree_sitter: bool,
    pub output_format: String,
}

/// Result of AI analysis
#[derive(Debug, Clone)]
pub struct AIAnalysisResult {
    pub content: String,
    pub is_fallback: bool,
    pub analysis_type: AnalysisType,
}

/// Type of analysis performed
#[derive(Debug, Clone)]
pub enum AnalysisType {
    Enhanced,
    Standard,
    Fallback,
}

/// File save configuration
#[derive(Debug, Clone)]
pub struct SaveConfig {
    pub auto_save: bool,
    pub format: String,
    pub base_path: String,
}

/// Review output configuration
#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub format: String,
    pub show_stats: bool,
    pub verbose: bool,
}
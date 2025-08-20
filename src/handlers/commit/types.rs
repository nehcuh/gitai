use crate::types::git::CommitArgs;
use crate::tree_sitter_analyzer::core::DiffAnalysis;

/// Context information for a commit operation
#[derive(Debug, Clone)]
pub struct CommitContext {
    pub args: CommitArgs,
    pub diff_content: String,
    pub review_context: Option<String>,
}

/// Configuration for commit message generation
#[derive(Debug, Clone)]
pub struct CommitGenerationConfig {
    pub use_tree_sitter: bool,
    pub include_review: bool,
    pub custom_message: Option<String>,
}

/// Result of commit message generation
#[derive(Debug, Clone)]
pub struct CommitGenerationResult {
    pub message: String,
    pub enhanced: bool,
    pub tree_sitter_analysis: Option<DiffAnalysis>,
    pub fallback_used: bool,
}

/// Request for basic commit message generation
#[derive(Debug, Clone)]
pub struct BasicCommitRequest {
    pub diff_content: String,
    pub review_context: Option<String>,
}

/// Request for enhanced commit message generation with Tree-sitter
#[derive(Debug, Clone)]
pub struct EnhancedCommitRequest {
    pub diff_content: String,
    pub custom_message: Option<String>,
    pub review_context: Option<String>,
}

/// Result of Tree-sitter analysis for commit
#[derive(Debug, Clone)]
pub struct TreeSitterCommitAnalysis {
    pub analysis_text: String,
    pub analysis_data: Option<DiffAnalysis>,
    pub processing_time: std::time::Duration,
}

/// Git repository operation request
#[derive(Debug, Clone)]
pub struct GitOperationRequest {
    pub auto_stage: bool,
    pub check_repository: bool,
}

/// Git repository operation result
#[derive(Debug, Clone)]
pub struct GitOperationResult {
    pub staged_files: Vec<String>,
    pub diff_content: String,
    pub has_changes: bool,
}

/// Review integration configuration
#[derive(Debug, Clone)]
pub struct ReviewIntegrationConfig {
    pub enabled: bool,
    pub storage_path: String,
    pub include_in_message: bool,
}

/// Review integration result
#[derive(Debug, Clone)]
pub struct ReviewIntegrationResult {
    pub review_content: Option<String>,
    pub review_file_path: Option<std::path::PathBuf>,
    pub integration_successful: bool,
}

/// User interaction configuration
#[derive(Debug, Clone)]
pub struct UserInteractionConfig {
    pub require_confirmation: bool,
    pub show_analysis: bool,
    pub format_output: bool,
}

/// User interaction result
#[derive(Debug, Clone)]
pub struct UserInteractionResult {
    pub confirmed: bool,
    pub modified_message: Option<String>,
}

/// Final commit execution request
#[derive(Debug, Clone)]
pub struct CommitExecutionRequest {
    pub message: String,
    pub issue_id: Option<String>,
    pub passthrough_args: Vec<String>,
}

/// Final commit execution result
#[derive(Debug, Clone)]
pub struct CommitExecutionResult {
    pub success: bool,
    pub commit_hash: Option<String>,
    pub message_used: String,
}
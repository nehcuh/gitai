// Git 操作模块
// Re-export implementations from git_impl

use gitai_types::Result;

// Include git_impl module as sibling file
#[path = "git_impl.rs"]
mod git_impl;

// Re-export the actual implementations
pub use git_impl::{
    run_git, run_git_capture,
    get_diff, get_all_diff,
    has_unstaged_changes, has_staged_changes,
    get_status, git_commit, git_add_all,
    get_unpushed_diff
};

// These are still stubs - need to be implemented or removed
pub fn get_staged_diff() -> Result<String> {
    // Use the actual implementation from git_impl
    git_impl::run_git(&["diff".to_string(), "--cached".to_string()])
        .map_err(|e| gitai_types::GitAIError::Git(e.to_string()))
}

pub fn get_current_branch() -> Result<String> {
    git_impl::run_git(&["symbolic-ref".to_string(), "--short".to_string(), "HEAD".to_string()])
        .map_err(|e| gitai_types::GitAIError::Git(e.to_string()))
        .map(|s| s.trim().to_string())
}

pub fn get_current_commit() -> Result<String> {
    git_impl::run_git(&["rev-parse".to_string(), "HEAD".to_string()])
        .map_err(|e| gitai_types::GitAIError::Git(e.to_string()))
        .map(|s| s.trim().to_string())
}

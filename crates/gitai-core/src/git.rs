// Git 操作模块
// Re-export implementations from git_impl

use gitai_types::Result;

// Include git_impl module as sibling file
#[path = "git_impl.rs"]
mod git_impl;

// Re-export the actual implementations
pub use git_impl::{
    get_all_diff, get_diff, get_status, get_unpushed_diff, git_add_all, git_commit,
    has_staged_changes, has_unstaged_changes, run_git, run_git_capture,
};

// These are still stubs - need to be implemented or removed
/// 获取暂存区的 diff
pub fn get_staged_diff() -> Result<String> {
    // Use the actual implementation from git_impl
    git_impl::run_git(&["diff".to_string(), "--cached".to_string()])
}

/// 获取当前分支名
pub fn get_current_branch() -> Result<String> {
    git_impl::run_git(&[
        "symbolic-ref".to_string(),
        "--short".to_string(),
        "HEAD".to_string(),
    ])
    .map(|s| s.trim().to_string())
}

/// 获取当前提交的 hash
pub fn get_current_commit() -> Result<String> {
    git_impl::run_git(&["rev-parse".to_string(), "HEAD".to_string()])
        .map(|s| s.trim().to_string())
}

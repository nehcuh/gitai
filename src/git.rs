use std::process::Command;

/// 简化的Git命令处理（禁用pager，保证非交互输出稳定）
pub fn run_git(args: &[String]) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let output = Command::new("git")
        .env("GIT_PAGER", "cat")
        .args(args)
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git command failed: {stderr}").into());
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

/// 运行Git并捕获退出码、stdout、stderr（不因非零退出中断），禁用pager
pub fn run_git_capture(args: &[String]) -> std::io::Result<(Option<i32>, String, String)> {
    let output = Command::new("git")
        .env("GIT_PAGER", "cat")
        .args(args)
        .output()?;
    let code = output.status.code();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok((code, stdout, stderr))
}

/// 获取Git diff
pub fn get_diff() -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
    run_git(&["diff".to_string(), "--cached".to_string()])
}

/// 获取所有变更（包括工作区和暂存区）
pub fn get_all_diff() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let staged_diff = run_git(&["diff".to_string(), "--cached".to_string()]).unwrap_or_default();
    let unstaged_diff = run_git(&["diff".to_string()]).unwrap_or_default();
    
    let mut all_diff = String::new();
    
    if !staged_diff.trim().is_empty() {
        all_diff.push_str("## 已暂存的变更 (Staged Changes):\n");
        all_diff.push_str(&staged_diff);
        all_diff.push('\n');
    }
    
    if !unstaged_diff.trim().is_empty() {
        all_diff.push_str("## 未暂存的变更 (Unstaged Changes):\n");
        all_diff.push_str(&unstaged_diff);
    }
    
    if all_diff.trim().is_empty() {
        return Err("没有检测到任何变更".into());
    }
    
    Ok(all_diff)
}

/// 检查是否有未暂存的变更
pub fn has_unstaged_changes() -> Result<bool, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let output = run_git(&["diff".to_string(), "--name-only".to_string()])?;
    Ok(!output.trim().is_empty())
}

/// 检查是否有已暂存的变更
pub fn has_staged_changes() -> Result<bool, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let output = run_git(&["diff".to_string(), "--cached".to_string(), "--name-only".to_string()])?;
    Ok(!output.trim().is_empty())
}

/// 获取Git状态
#[allow(dead_code)]
pub fn get_status() -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
    run_git(&["status".to_string(), "--porcelain".to_string()])
}

/// 执行Git提交
pub fn git_commit(message: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
    run_git(&["commit".to_string(), "-m".to_string(), message.to_string()])
}

/// 自动暂存所有变更
pub fn git_add_all() -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
    run_git(&["add".to_string(), ".".to_string()])
}
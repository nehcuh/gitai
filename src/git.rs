use std::process::Command;

/// 简化的Git命令处理
pub fn run_git(args: &[String]) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(args)
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git command failed: {stderr}").into());
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

/// 获取Git diff
pub fn get_diff() -> Result<String, Box<dyn std::error::Error>> {
    run_git(&["diff".to_string(), "--cached".to_string()])
}

/// 获取Git状态
#[allow(dead_code)]
pub fn get_status() -> Result<String, Box<dyn std::error::Error>> {
    run_git(&["status".to_string(), "--porcelain".to_string()])
}

/// 执行Git提交
pub fn git_commit(message: &str) -> Result<String, Box<dyn std::error::Error>> {
    run_git(&["commit".to_string(), "-m".to_string(), message.to_string()])
}

/// 自动暂存所有变更
pub fn git_add_all() -> Result<String, Box<dyn std::error::Error>> {
    run_git(&["add".to_string(), ".".to_string()])
}
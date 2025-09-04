use std::path::Path;
use std::process::Command;

/// 简化的Git命令处理（禁用pager，保证非交互输出稳定）
pub fn run_git(args: &[String]) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let output = Command::new("git")
        .env("GIT_PAGER", "cat")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute git command: {e}"))?;

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
pub fn get_diff() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    run_git(&["diff".to_string(), "--cached".to_string()])
}

/// 获取所有变更（包括工作区、暂存区和未推送的提交）
pub fn get_all_diff() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Git 的 diff 命令会自动排除未跟踪且在 .gitignore 中的文件
    let staged_diff = run_git(&["diff".to_string(), "--cached".to_string()]).unwrap_or_default();
    let unstaged_diff = run_git(&["diff".to_string()]).unwrap_or_default();

    // 检查未推送的提交
    let unpushed_diff = get_unpushed_diff().unwrap_or_default();

    let mut all_diff = String::new();

    // 优先级：未推送的提交 > 已暂存的变更 > 未暂存的变更
    if !unpushed_diff.trim().is_empty() {
        all_diff.push_str("## 未推送的提交变更 (Unpushed Commits):\n");
        all_diff.push_str(&unpushed_diff);
        all_diff.push('\n');
    }

    if !staged_diff.trim().is_empty() {
        all_diff.push_str("## 已暂存的变更 (Staged Changes):\n");
        all_diff.push_str(&staged_diff);
        all_diff.push('\n');
    }

    if !unstaged_diff.trim().is_empty() {
        all_diff.push_str("## 未暂存的变更 (Unstaged Changes):\n");
        all_diff.push_str(&unstaged_diff);
    }

    // 如果没有任何变更，不要自动返回最后一次提交
    // 让调用方决定如何处理这种情况
    if all_diff.trim().is_empty() {
        return Err("没有检测到任何变更".into());
    }

    Ok(all_diff)
}

/// 检查是否有未暂存的变更
pub fn has_unstaged_changes() -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let output = run_git(&["diff".to_string(), "--name-only".to_string()])?;
    Ok(!output.trim().is_empty())
}

/// 检查是否有已暂存的变更
pub fn has_staged_changes() -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let output = run_git(&[
        "diff".to_string(),
        "--cached".to_string(),
        "--name-only".to_string(),
    ])?;
    Ok(!output.trim().is_empty())
}

/// 获取Git状态
#[allow(dead_code)]
pub fn get_status() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    run_git(&["status".to_string(), "--porcelain".to_string()])
}

/// 执行Git提交
pub fn git_commit(message: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    run_git(&["commit".to_string(), "-m".to_string(), message.to_string()])
}

/// 自动暂存所有变更
pub fn git_add_all() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    run_git(&["add".to_string(), ".".to_string()])
}

/// 获取未推送的提交的diff
pub fn get_unpushed_diff() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // 首先检查是否有远程分支
    let remote_branch = get_upstream_branch();

    match remote_branch {
        Ok(upstream) => {
            // 有远程分支，比较本地与远程的差异
            log::debug!("检查未推送的提交: 本地 vs {upstream}");
            let diff = run_git(&["diff".to_string(), format!("{upstream}..HEAD")]);

            match diff {
                Ok(content) => {
                    if content.trim().is_empty() {
                        log::debug!("没有未推送的提交");
                        Ok(String::new())
                    } else {
                        log::debug!("找到未推送的提交，差异长度: {}", content.len());
                        Ok(content)
                    }
                }
                Err(e) => {
                    log::warn!("获取未推送的diff失败: {e}");
                    Ok(String::new())
                }
            }
        }
        Err(_) => {
            // 没有远程分支，检查本地是否有提交
            log::debug!("没有远程分支，检查本地提交历史");
            match run_git(&[
                "log".to_string(),
                "--oneline".to_string(),
                "-n".to_string(),
                "1".to_string(),
            ]) {
                Ok(log_output) => {
                    if log_output.trim().is_empty() {
                        log::debug!("没有本地提交");
                        Ok(String::new())
                    } else {
                        // 有本地提交但没有远程，显示所有提交的diff
                        log::debug!("有本地提交但没有远程分支，显示最近提交的diff");
                        match run_git(&[
                            "show".to_string(),
                            "HEAD".to_string(),
                            "--format=format:".to_string(),
                        ]) {
                            Ok(diff) => Ok(diff),
                            Err(_) => Ok(String::new()),
                        }
                    }
                }
                Err(_) => {
                    log::debug!("未初始化的Git仓库");
                    Ok(String::new())
                }
            }
        }
    }
}

/// 获取最后一次提交的 diff
pub fn get_last_commit_diff() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // 获取最后一次提交的 diff
    run_git(&[
        "diff".to_string(),
        "HEAD~1".to_string(),
        "HEAD".to_string(),
    ])
}

/// 获取当前分支的上游分支
pub fn get_upstream_branch() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // 尝试获取当前分支的上游分支
    match run_git(&[
        "rev-parse".to_string(),
        "--abbrev-ref".to_string(),
        "@{upstream}".to_string(),
    ]) {
        Ok(upstream) => {
            let upstream = upstream.trim().to_string();
            if upstream.is_empty() {
                Err("没有配置上游分支".into())
            } else {
                Ok(upstream)
            }
        }
        Err(_) => {
            // 如果没有上游分支，尝试使用 origin/当前分支名
            match run_git(&[
                "rev-parse".to_string(),
                "--abbrev-ref".to_string(),
                "HEAD".to_string(),
            ]) {
                Ok(current_branch) => {
                    let current_branch = current_branch.trim();
                    let origin_branch = format!("origin/{current_branch}");

                    // 检查 origin/branch 是否存在
                    match run_git(&[
                        "rev-parse".to_string(),
                        "--verify".to_string(),
                        origin_branch.clone(),
                    ]) {
                        Ok(_) => Ok(origin_branch),
                        Err(_) => Err("没有找到对应的远程分支".into()),
                    }
                }
                Err(e) => Err(format!("无法获取当前分支: {e}").into()),
            }
        }
    }
}

/// 获取所有变更（包括最后一次提交）- 用于 MCP 调用
pub fn get_all_diff_or_last_commit() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // 首先尝试获取当前的变更
    match get_all_diff() {
        Ok(diff) => Ok(diff),
        Err(_) => {
            // 如果没有当前变更，尝试获取最后一次提交的 diff
            match get_last_commit_diff() {
                Ok(last_diff) if !last_diff.trim().is_empty() => {
                    Ok(format!("## 最后一次提交的变更 (Last Commit):\n{}", last_diff))
                }
                _ => Err("没有检测到任何变更".into()),
            }
        }
    }
}

/// 过滤掉被 .gitignore 忽略的文件路径
pub fn filter_ignored_files(paths: Vec<String>) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    if paths.is_empty() {
        return Ok(paths);
    }
    
    // 使用 git check-ignore 来检查哪些文件被忽略
    let output = Command::new("git")
        .arg("check-ignore")
        .args(&paths)
        .output()?;
    
    let ignored = String::from_utf8_lossy(&output.stdout);
    let ignored_set: std::collections::HashSet<_> = ignored.lines().collect();
    
    Ok(paths.into_iter().filter(|p| !ignored_set.contains(p.as_str())).collect())
}

/// 获取当前仓库中被跟踪的文件列表（排除 .gitignore 中的文件）
pub fn get_tracked_files() -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    // git ls-files 只会列出被跟踪的文件，自动排除 .gitignore 中的文件
    let output = run_git(&["ls-files".to_string()])?;
    Ok(output.lines().map(|s| s.to_string()).collect())
}

/// 检查文件是否被 .gitignore 忽略
pub fn is_file_ignored(file_path: &Path) -> bool {
    let path_str = file_path.to_string_lossy();
    
    // 使用 git check-ignore 命令检查文件
    match Command::new("git")
        .arg("check-ignore")
        .arg(path_str.as_ref())
        .output()
    {
        Ok(output) => {
            // 如果退出码为 0，说明文件被忽略
            output.status.success()
        }
        Err(_) => false,
    }
}

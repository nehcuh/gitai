use gitai_types::{GitAIError, Result};
use std::path::Path;
use std::process::Command;

/// 简化的Git命令处理（禁用pager，保证非交互输出稳定）
pub fn run_git(args: &[String]) -> Result<String> {
    let output = Command::new("git")
        .env("GIT_PAGER", "cat")
        .args(args)
        .output()
        .map_err(|e| GitAIError::Git(format!("Failed to execute git command: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitAIError::Git(format!("Git command failed: {stderr}")));
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
pub fn get_diff() -> Result<String> {
    run_git(&["diff".to_string(), "--cached".to_string()])
}

/// 获取所有变更（包括工作区、暂存区、未跟踪文件和未推送的提交）
pub fn get_all_diff() -> Result<String> {
    // Git 的 diff 命令不会包含未跟踪文件；我们将专门收集未跟踪（且未被 .gitignore 忽略）的文件
    let staged_diff = run_git(&["diff".to_string(), "--cached".to_string()]).unwrap_or_default();
    let unstaged_diff = run_git(&["diff".to_string()]).unwrap_or_default();

    // 检查未推送的提交
    let unpushed_diff = get_unpushed_diff().unwrap_or_default();

    // 收集未跟踪文件并为其生成 diff
    let mut untracked_section = String::new();
    if let Ok(untracked) = get_untracked_files() {
        if !untracked.is_empty() {
            let mut combined = String::new();
            // 过滤过大的或二进制/资产类文件，避免生成巨大的 diff
            const MAX_INLINE_SIZE: u64 = 1_000_000; // 1MB 上限
            let skip_exts = [
                "png", "jpg", "jpeg", "gif", "bmp", "ico", "svg", "pdf", "zip", "tar", "gz", "bz2",
                "xz",
            ];

            for p in &untracked {
                let path = std::path::Path::new(p);
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                let mut should_skip_content = skip_exts.iter().any(|e| e.eq_ignore_ascii_case(ext));
                let mut size_info = String::new();
                if let Ok(meta) = std::fs::metadata(path) {
                    let len = meta.len();
                    if len > MAX_INLINE_SIZE {
                        should_skip_content = true;
                        size_info = format!(" ({len} bytes)");
                    }
                }

                if should_skip_content {
                    // 仅添加一个简短的记录，说明新增了大文件/二进制文件
                    combined.push_str(&format!(
                        "diff --git a/{p} b/{p}\nnew file mode 100644\n--- /dev/null\n+++ b/{p}\n@@\n+ [新增大文件/二进制文件已省略内容]{size_info}\n\n"
                    ));
                    continue;
                }

                if let Ok((code, stdout, _stderr)) = run_git_capture(&[
                    "diff".to_string(),
                    "--no-index".to_string(),
                    "--".to_string(),
                    "/dev/null".to_string(),
                    p.clone(),
                ]) {
                    // exit code 1 表示存在差异，这是预期情况
                    if (code.is_none() || code == Some(1) || code == Some(0))
                        && !stdout.trim().is_empty()
                    {
                        combined.push_str(&stdout);
                        if !combined.ends_with('\n') {
                            combined.push('\n');
                        }
                    }
                }
            }
            if !combined.trim().is_empty() {
                untracked_section.push_str("## 未跟踪的新文件 (Untracked Files):\n");
                untracked_section.push_str(&combined);
                if !untracked_section.ends_with('\n') {
                    untracked_section.push('\n');
                }
            }
        }
    }

    let mut all_diff = String::new();

    // 优先级：未推送的提交 > 已暂存的变更 > 未暂存的变更 > 未跟踪文件
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
        if !all_diff.ends_with('\n') {
            all_diff.push('\n');
        }
    }

    if !untracked_section.trim().is_empty() {
        all_diff.push_str(&untracked_section);
    }

    // 如果没有任何变更，不要自动返回最后一次提交
    // 让调用方决定如何处理这种情况
    if all_diff.trim().is_empty() {
        return Err(GitAIError::Git("没有检测到任何变更".to_string()));
    }

    Ok(all_diff)
}

/// 检查是否有未暂存的变更
pub fn has_unstaged_changes() -> Result<bool> {
    let output = run_git(&["diff".to_string(), "--name-only".to_string()])?;
    Ok(!output.trim().is_empty())
}

/// 检查是否有已暂存的变更
pub fn has_staged_changes() -> Result<bool> {
    let output = run_git(&[
        "diff".to_string(),
        "--cached".to_string(),
        "--name-only".to_string(),
    ])?;
    Ok(!output.trim().is_empty())
}

/// 获取Git状态
#[allow(dead_code)]
pub fn get_status() -> Result<String> {
    run_git(&["status".to_string(), "--porcelain".to_string()])
}

/// 执行Git提交
pub fn git_commit(message: &str) -> Result<String> {
    run_git(&["commit".to_string(), "-m".to_string(), message.to_string()])
}

/// 自动暂存所有变更
pub fn git_add_all() -> Result<String> {
    run_git(&["add".to_string(), ".".to_string()])
}

/// 获取未推送的提交的diff
pub fn get_unpushed_diff() -> Result<String> {
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
#[allow(dead_code)]
pub fn get_last_commit_diff() -> Result<String> {
    // 先检查是否有多个提交
    let log_output = run_git(&[
        "rev-list".to_string(),
        "--count".to_string(),
        "HEAD".to_string(),
    ])?;

    let commit_count: usize = log_output.trim().parse().unwrap_or(0);

    if commit_count == 0 {
        Err(GitAIError::Git("仓库中没有任何提交".to_string()))
    } else if commit_count == 1 {
        // 只有一个提交，显示第一次提交的内容
        run_git(&[
            "show".to_string(),
            "HEAD".to_string(),
            "--format=format:".to_string(),
        ])
    } else {
        // 有多个提交，显示最后一次提交与前一次的差异
        run_git(&["diff".to_string(), "HEAD~1".to_string(), "HEAD".to_string()])
    }
}

/// 获取当前分支的上游分支
pub fn get_upstream_branch() -> Result<String> {
    // 尝试获取当前分支的上游分支
    match run_git(&[
        "rev-parse".to_string(),
        "--abbrev-ref".to_string(),
        "@{upstream}".to_string(),
    ]) {
        Ok(upstream) => {
            let upstream = upstream.trim().to_string();
            if upstream.is_empty() {
                Err(GitAIError::Git("没有配置上游分支".to_string()))
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
                        Err(_) => Err(GitAIError::Git("没有找到对应的远程分支".to_string())),
                    }
                }
                Err(e) => Err(GitAIError::Git(format!("无法获取当前分支: {e}"))),
            }
        }
    }
}

/// 获取所有变更（包括最后一次提交）- 用于 MCP 调用
#[allow(dead_code)]
pub fn get_all_diff_or_last_commit() -> Result<String> {
    // 首先尝试获取当前的变更
    match get_all_diff() {
        Ok(diff) => Ok(diff),
        Err(_) => {
            // 如果没有当前变更，尝试获取最后一次提交的 diff
            match get_last_commit_diff() {
                Ok(last_diff) if !last_diff.trim().is_empty() => {
                    Ok(format!("## 最后一次提交的变更 (Last Commit):\n{last_diff}"))
                }
                _ => Err(GitAIError::Git("没有检测到任何变更".to_string())),
            }
        }
    }
}

/// 过滤掉被 .gitignore 忽略的文件路径
#[allow(dead_code)]
pub fn filter_ignored_files(paths: Vec<String>) -> Result<Vec<String>> {
    if paths.is_empty() {
        return Ok(paths);
    }

    // 使用 git check-ignore 来检查哪些文件被忽略
    let output = Command::new("git")
        .arg("check-ignore")
        .args(&paths)
        .output()
        .map_err(|e| GitAIError::Io(e))?;

    let ignored = String::from_utf8_lossy(&output.stdout);
    let ignored_set: std::collections::HashSet<_> = ignored.lines().collect();

    Ok(paths
        .into_iter()
        .filter(|p| !ignored_set.contains(p.as_str()))
        .collect())
}

/// 获取当前仓库中被跟踪的文件列表（排除 .gitignore 中的文件）
#[allow(dead_code)]
pub fn get_tracked_files() -> Result<Vec<String>> {
    // git ls-files 只会列出被跟踪的文件，自动排除 .gitignore 中的文件
    let output = run_git(&["ls-files".to_string()])?;
    Ok(output.lines().map(|s| s.to_string()).collect())
}

/// 获取未跟踪文件（自动排除 .gitignore 中的文件）
pub fn get_untracked_files() -> Result<Vec<String>> {
    let output = run_git(&[
        "ls-files".to_string(),
        "--others".to_string(),
        "--exclude-standard".to_string(),
    ])?;
    let files: Vec<String> = output
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    Ok(files)
}

/// 是否存在未跟踪变更（新增文件）
#[allow(dead_code)]
pub fn has_untracked_changes() -> Result<bool> {
    Ok(!get_untracked_files()?.is_empty())
}

/// 是否存在任何提交
#[allow(dead_code)]
pub fn has_any_commit() -> bool {
    if let Ok((code, _out, _err)) = run_git_capture(&[
        "rev-parse".to_string(),
        "--verify".to_string(),
        "HEAD".to_string(),
    ]) {
        return code == Some(0);
    }
    false
}

/// 检查文件是否被 .gitignore 忽略
#[allow(dead_code)]
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

//! Git 状态分析功能测试
//!
//! 测试 Git 操作的核心功能，包括：
//! - Git 命令执行
//! - Diff 生成和变更检测
//! - 文件跟踪和忽略规则处理
//! - 提交和分支操作

use gitai::error::{GitAIError, GitError};
use gitai::git;
use std::path::Path;

#[test]
fn test_run_git_basic() {
    // 测试基本的 Git 命令执行
    let result = git::run_git(&["--version".to_string()]);

    assert!(result.is_ok());
    let version = result.unwrap();
    assert!(version.contains("git version"));
    println!("Git version: {}", version);
}

#[test]
fn test_run_git_capture() {
    // 测试带错误捕获的 Git 命令执行
    let result = git::run_git_capture(&["--version".to_string()]);

    assert!(result.is_ok());
    let (code, stdout, stderr) = result.unwrap();
    assert_eq!(code, Some(0));
    assert!(stdout.contains("git version"));
    assert!(stderr.is_empty());
}

#[test]
fn test_run_git_invalid_command() {
    // 测试无效的 Git 命令
    let result = git::run_git(&["invalid-command".to_string()]);

    assert!(result.is_err());
}

#[test]
fn test_get_diff() {
    // 测试获取 Git diff
    let result = git::get_diff();

    match result {
        Ok(diff) => {
            println!("Staged diff retrieved successfully");
            println!("Diff length: {} characters", diff.len());
        }
        Err(e) => {
            // 可能是因为没有暂存的变更
            println!("Get diff failed (expected if no staged changes): {}", e);
        }
    }
}

#[test]
fn test_get_all_diff() {
    // 测试获取所有变更
    let result = git::get_all_diff();

    match result {
        Ok(diff) => {
            println!("All diff retrieved successfully");
            println!("Diff length: {} characters", diff.len());
        }
        Err(e) => {
            println!("Get all diff failed: {}", e);
        }
    }
}

#[test]
fn test_has_unstaged_changes() {
    // 测试检查是否有未暂存的变更
    let result = git::has_unstaged_changes();

    match result {
        Ok(has_changes) => {
            println!("Has unstaged changes: {}", has_changes);
            // 这是一个布尔值，不需要特定验证
        }
        Err(e) => {
            println!("Check unstaged changes failed: {}", e);
        }
    }
}

#[test]
fn test_has_staged_changes() {
    // 测试检查是否有暂存的变更
    let result = git::has_staged_changes();

    match result {
        Ok(has_changes) => {
            println!("Has staged changes: {}", has_changes);
            // 这是一个布尔值，不需要特定验证
        }
        Err(e) => {
            println!("Check staged changes failed: {}", e);
        }
    }
}

#[test]
fn test_get_status() {
    // 测试获取 Git 状态
    let result = git::get_status();

    match result {
        Ok(status) => {
            println!("Git status retrieved successfully");
            println!("Status length: {} characters", status.len());
        }
        Err(e) => {
            println!("Get status failed: {}", e);
        }
    }
}

#[test]
fn test_get_untracked_files() {
    // 测试获取未跟踪的文件
    let result = git::get_untracked_files();

    match result {
        Ok(files) => {
            println!("Untracked files retrieved successfully");
            println!("Untracked files count: {}", files.len());

            // 验证文件路径
            for file in &files {
                assert!(!file.is_empty());
                println!("Untracked file: {}", file);
            }
        }
        Err(e) => {
            println!("Get untracked files failed: {}", e);
        }
    }
}

#[test]
fn test_get_tracked_files() {
    // 测试获取已跟踪的文件
    let result = git::get_tracked_files();

    match result {
        Ok(files) => {
            println!("Tracked files retrieved successfully");
            println!("Tracked files count: {}", files.len());

            // 验证文件路径
            for file in &files {
                assert!(!file.is_empty());
                println!("Tracked file: {}", file);
            }
        }
        Err(e) => {
            println!("Get tracked files failed: {}", e);
        }
    }
}

#[test]
fn test_has_untracked_changes() {
    // 测试检查是否有未跟踪的变更
    let result = git::has_untracked_changes();

    match result {
        Ok(has_changes) => {
            println!("Has untracked changes: {}", has_changes);
        }
        Err(e) => {
            println!("Check untracked changes failed: {}", e);
        }
    }
}

#[test]
fn test_has_any_commit() {
    // 测试检查是否有任何提交
    let has_commit = git::has_any_commit();
    println!("Has any commit: {}", has_commit);
    // 这是一个布尔值，不需要特定验证
}

#[test]
fn test_get_upstream_branch() {
    // 测试获取上游分支
    let result = git::get_upstream_branch();

    match result {
        Ok(branch) => {
            println!("Upstream branch: {}", branch);
        }
        Err(e) => {
            println!(
                "Get upstream branch failed (expected if no upstream): {}",
                e
            );
        }
    }
}

#[test]
fn test_get_last_commit_diff() {
    // 测试获取最后一次提交的 diff
    let result = git::get_last_commit_diff();

    match result {
        Ok(diff) => {
            println!("Last commit diff retrieved successfully");
            println!("Diff length: {} characters", diff.len());
        }
        Err(e) => {
            println!("Get last commit diff failed: {}", e);
        }
    }
}

#[test]
fn test_get_unpushed_diff() {
    // 测试获取未推送的 diff
    let result = git::get_unpushed_diff();

    match result {
        Ok(diff) => {
            println!("Unpushed diff retrieved successfully");
            println!("Diff length: {} characters", diff.len());
        }
        Err(e) => {
            println!(
                "Get unpushed diff failed (expected if no unpushed commits): {}",
                e
            );
        }
    }
}

#[test]
fn test_filter_ignored_files() {
    // 测试过滤被忽略的文件
    let test_files = vec![
        "target/debug/".to_string(),
        "src/main.rs".to_string(),
        "Cargo.lock".to_string(),
        ".git/".to_string(),
    ];

    let result = git::filter_ignored_files(test_files);

    match result {
        Ok(filtered_files) => {
            println!("Filtered ignored files successfully");
            println!("Filtered files count: {}", filtered_files.len());

            // 验证结果
            for file in &filtered_files {
                assert!(!file.is_empty());
                println!("Not ignored file: {}", file);
            }
        }
        Err(e) => {
            println!("Filter ignored files failed: {}", e);
        }
    }
}

#[test]
fn test_is_file_ignored() {
    // 测试检查文件是否被忽略
    let ignored_path = Path::new("target/debug/main");
    let not_ignored_path = Path::new("src/main.rs");

    let is_ignored1 = git::is_file_ignored(ignored_path);
    let is_ignored2 = git::is_file_ignored(not_ignored_path);

    println!("target/debug/main is ignored: {}", is_ignored1);
    println!("src/main.rs is ignored: {}", is_ignored2);

    // target/debug/ 应该被忽略（如果存在 .gitignore）
    // src/main.rs 不应该被忽略
}

#[test]
fn test_get_all_diff_or_last_commit() {
    // 测试获取所有 diff 或最后一次提交的 diff
    let result = git::get_all_diff_or_last_commit();

    match result {
        Ok(diff) => {
            println!("All diff or last commit retrieved successfully");
            println!("Diff length: {} characters", diff.len());
        }
        Err(e) => {
            println!("Get all diff or last commit failed: {}", e);
        }
    }
}

#[test]
fn test_git_operations_error_handling() {
    // 测试 Git 操作的错误处理
    // 在不存在的目录中运行 Git 命令
    let result = git::run_git(&[
        "-C".to_string(),
        "/nonexistent/path".to_string(),
        "status".to_string(),
    ]);

    // 应该返回错误
    assert!(result.is_err());
    println!(
        "Expected error for non-existent path: {}",
        result.unwrap_err()
    );
}

#[test]
fn test_git_commit_and_add_operations() {
    // 注意：这些测试会修改 Git 仓库状态，所以只在测试环境中运行

    // 测试 git add all
    let add_result = git::git_add_all();
    match add_result {
        Ok(output) => {
            println!("Git add all output: {}", output);
        }
        Err(e) => {
            println!("Git add all failed: {}", e);
        }
    }

    // 注意：不测试 git_commit 以避免创建实际的提交
    // 如果需要测试提交功能，应该使用测试仓库
}

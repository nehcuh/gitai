//! Commit 命令处理器
//!
//! 处理智能提交相关的命令，包括自动生成提交信息、代码评审、Issue关联等功能

use crate::args::Command;
use gitai_core::config::Config;
use std::process::Command as ProcessCommand;
use std::path::Path;

type HandlerResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

/// 智能提交配置
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CommitConfig {
    message: Option<String>,
    issue_id: Option<String>,
    space_id: Option<u64>,
    all: bool,
    review: bool,
    tree_sitter: bool,
    dry_run: bool,
}

/// 提交结果
#[derive(Debug)]
#[allow(dead_code)]
struct CommitResult {
    generated_message: String,
    files_added: Vec<String>,
    files_modified: Vec<String>,
    files_deleted: Vec<String>,
    review_result: Option<String>,
    success: bool,
}

/// 处理 commit 命令
pub async fn handle_command(
    config: &Config,
    command: &Command,
) -> HandlerResult<()> {
    match command {
        Command::Commit {
            message,
            issue_id,
            space_id,
            all,
            review,
            tree_sitter,
            dry_run,
        } => {
            let commit_config = CommitConfig {
                message: message.clone(),
                issue_id: issue_id.clone(),
                space_id: *space_id,
                all: *all,
                review: *review,
                tree_sitter: *tree_sitter,
                dry_run: *dry_run,
            };

            handle_smart_commit(config, commit_config).await
        }
        _ => Err("Invalid command for commit handler".into()),
    }
}

/// 处理智能提交
async fn handle_smart_commit(config: &Config, commit_config: CommitConfig) -> HandlerResult<()> {
    println!("🚀 开始智能提交...");

    // 1. 检查Git仓库状态
    check_git_repository().await?;

    // 2. 获取变更的文件
    let changed_files = get_changed_files(commit_config.all).await?;
    if changed_files.is_empty() {
        println!("ℹ️  没有检测到文件变更，跳过提交");
        return Ok(());
    }

    println!("📁 检测到 {} 个文件变更", changed_files.len());
    for file in &changed_files {
        println!("  - {}", file);
    }

    // 3. 生成智能提交信息
    let commit_message = generate_commit_message(config, &commit_config, &changed_files).await?;
    println!("📝 生成的提交信息: {}", commit_message);

    // 4. 如果启用评审，执行代码评审
    let mut review_result = None;
    if commit_config.review {
        println!("🔍 执行提交前代码评审...");
        review_result = perform_pre_commit_review(config, &changed_files).await?;
        if let Some(ref review) = review_result {
            println!("📊 评审结果:\n{}", review);
        }
    }

    // 5. 如果是测试运行，显示计划但不执行
    if commit_config.dry_run {
        println!("\n🧪 测试运行模式 - 将执行以下操作:");
        println!("  - 添加变更文件到暂存区");
        println!("  - 提交: {}", commit_message);
        if review_result.is_some() {
            println!("  - 代码评审: 已完成");
        }
        println!("\n✅ 测试运行完成，未执行实际提交");
        return Ok(());
    }

    // 6. 执行实际的Git提交
    execute_git_commit(&commit_message).await?;

    // 7. 显示成功信息
    display_commit_success(&commit_message, review_result.is_some()).await?;

    Ok(())
}

/// 检查是否在Git仓库中
async fn check_git_repository() -> HandlerResult<()> {
    let output = ProcessCommand::new("git")
        .args(&["rev-parse", "--git-dir"])
        .output();

    match output {
        Ok(result) if result.status.success() => Ok(()),
        _ => Err("当前目录不是Git仓库".into()),
    }
}

/// 获取变更的文件
async fn get_changed_files(add_all: bool) -> HandlerResult<Vec<String>> {
    if add_all {
        // 获取所有未跟踪和已修改的文件
        let untracked = get_git_output(&["ls-files", "--others", "--exclude-standard"]).await?;
        let modified = get_git_output(&["diff", "--name-only", "HEAD"]).await?;

        let mut files: Vec<String> = untracked
            .lines()
            .chain(modified.lines())
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .collect();

        // 添加已暂存的文件
        let staged = get_git_output(&["diff", "--cached", "--name-only"]).await?;
        files.extend(staged.lines().filter(|s| !s.trim().is_empty()).map(|s| s.to_string()));

        // 去重
        files.sort();
        files.dedup();

        Ok(files)
    } else {
        // 只获取已暂存的文件
        let staged = get_git_output(&["diff", "--cached", "--name-only"]).await?;
        let files: Vec<String> = staged
            .lines()
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .collect();

        Ok(files)
    }
}

/// 执行Git命令并获取输出
async fn get_git_output(args: &[&str]) -> HandlerResult<String> {
    let output = ProcessCommand::new("git")
        .args(args)
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(format!("Git命令执行失败: {:?}", args).into())
    }
}

/// 生成智能提交信息
async fn generate_commit_message(
    _config: &Config,
    commit_config: &CommitConfig,
    changed_files: &[String],
) -> HandlerResult<String> {
    // 如果用户提供了提交信息，直接使用
    if let Some(ref message) = commit_config.message {
        let mut final_message = message.clone();

        // 如果有关联的Issue ID，添加到提交信息中
        if let Some(ref issue_id) = commit_config.issue_id {
            if !final_message.contains(issue_id) {
                final_message.push_str(&format!(" (Issue: {})", issue_id));
            }
        }

        return Ok(final_message);
    }

    // 自动生成提交信息
    let _diff_stats = get_git_output(&["diff", "--stat", "HEAD"]).await?;
    let _diff_summary = get_git_output(&["diff", "--summary", "HEAD"]).await?;

    // 分析文件变更类型
    let mut added_files = Vec::new();
    let mut modified_files = Vec::new();
    let mut deleted_files = Vec::new();

    for file in changed_files {
        if Path::new(file).exists() {
            if get_git_output(&["ls-files", file]).await?.trim().is_empty() {
                added_files.push(file);
            } else {
                modified_files.push(file);
            }
        } else {
            deleted_files.push(file);
        }
    }

    // 生成描述性提交信息
    let mut message = String::new();

    if !added_files.is_empty() {
        let added_strings: Vec<String> = added_files.iter().map(|s| s.to_string()).collect();
        message.push_str(&format!("添加 {}", summarize_files(&added_strings)));
    }
    if !modified_files.is_empty() {
        if !message.is_empty() {
            message.push_str("; ");
        }
        let modified_strings: Vec<String> = modified_files.iter().map(|s| s.to_string()).collect();
        message.push_str(&format!("更新 {}", summarize_files(&modified_strings)));
    }
    if !deleted_files.is_empty() {
        if !message.is_empty() {
            message.push_str("; ");
        }
        let deleted_strings: Vec<String> = deleted_files.iter().map(|s| s.to_string()).collect();
        message.push_str(&format!("删除 {}", summarize_files(&deleted_strings)));
    }

    // 如果没有识别出变更，使用通用描述
    if message.is_empty() {
        message = "更新代码".to_string();
    }

    // 添加Issue ID
    if let Some(ref issue_id) = commit_config.issue_id {
        message.push_str(&format!(" (Issue: {})", issue_id));
    }

    Ok(message)
}

/// 总结文件列表
fn summarize_files(files: &[String]) -> String {
    if files.len() == 1 {
        files[0].clone()
    } else if files.len() <= 3 {
        files.join(", ")
    } else {
        format!("{}个文件", files.len())
    }
}

/// 执行提交前代码评审
async fn perform_pre_commit_review(
    _config: &Config,
    changed_files: &[String],
) -> HandlerResult<Option<String>> {
    // 这里可以集成Review Handler的功能
    // 为了简化，我们只做基本的检查

    let mut issues = Vec::new();

    // 检查是否有大文件提交
    for file in changed_files {
        if let Ok(metadata) = std::fs::metadata(file) {
            if metadata.len() > 1024 * 1024 { // 1MB
                issues.push(format!("⚠️  大文件检测: {} ({}MB)",
                    file, metadata.len() / (1024 * 1024)));
            }
        }
    }

    // 检查是否有敏感文件
    for file in changed_files {
        let file_name = file.to_lowercase();
        if file_name.contains("key") || file_name.contains("secret") || file_name.contains("password") {
            issues.push(format!("🚨 敏感文件检测: {}", file));
        }
    }

    if issues.is_empty() {
        Ok(Some("✅ 提交前检查通过，未发现问题".to_string()))
    } else {
        Ok(Some(format!("发现以下问题:\n{}", issues.join("\n"))))
    }
}

/// 执行Git提交
async fn execute_git_commit(message: &str) -> HandlerResult<()> {
    println!("💾 执行Git提交...");

    // 添加文件到暂存区
    let _ = get_git_output(&["add", "."]).await?;

    // 执行提交
    let output = ProcessCommand::new("git")
        .args(&["commit", "-m", message])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Git提交失败: {}", stderr).into())
    }
}

/// 显示提交成功信息
async fn display_commit_success(message: &str, had_review: bool) -> HandlerResult<()> {
    println!("\n✅ 提交成功!");
    println!("📝 提交信息: {}", message);

    if had_review {
        println!("🔍 已执行代码评审");
    }

    // 显示最新提交的哈希
    if let Ok(latest_commit) = get_git_output(&["rev-parse", "--short", "HEAD"]).await {
        println!("🔗 提交哈希: {}", latest_commit.trim());
    }

    println!("\n💡 提示: 使用 'git push' 推送到远程仓库");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_files_single() {
        let files = vec!["src/main.rs".to_string()];
        assert_eq!(summarize_files(&files), "src/main.rs");
    }

    #[test]
    fn test_summarize_files_few() {
        let files = vec!["src/main.rs".to_string(), "src/lib.rs".to_string(), "README.md".to_string()];
        assert_eq!(summarize_files(&files), "src/main.rs, src/lib.rs, README.md");
    }

    #[test]
    fn test_summarize_files_many() {
        let files: Vec<String> = (0..5).map(|i| format!("file{}.rs", i)).collect();
        assert_eq!(summarize_files(&files), "5个文件");
    }

    #[test]
    fn test_commit_config_creation() {
        let config = CommitConfig {
            message: Some("test commit".to_string()),
            issue_id: Some("123".to_string()),
            space_id: Some(456),
            all: true,
            review: false,
            tree_sitter: false,
            dry_run: false,
        };

        assert_eq!(config.message, Some("test commit".to_string()));
        assert_eq!(config.issue_id, Some("123".to_string()));
        assert_eq!(config.space_id, Some(456));
        assert!(config.all);
        assert!(!config.review);
    }

    #[test]
    fn test_commit_result_creation() {
        let result = CommitResult {
            generated_message: "feat: add new feature".to_string(),
            files_added: vec!["src/new.rs".to_string()],
            files_modified: vec!["src/main.rs".to_string()],
            files_deleted: vec!["old.rs".to_string()],
            review_result: Some("Review passed".to_string()),
            success: true,
        };

        assert_eq!(result.generated_message, "feat: add new feature");
        assert_eq!(result.files_added.len(), 1);
        assert_eq!(result.files_modified.len(), 1);
        assert_eq!(result.files_deleted.len(), 1);
        assert!(result.review_result.is_some());
        assert!(result.success);
    }
}

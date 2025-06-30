use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::types::git::{GitaiArgs, GitaiSubCommand, ReviewArgs, CommitArgs};
use crate::errors::AppError;

pub fn construct_review_args(args: &[String]) -> ReviewArgs {
    // 重构review命令参数以便使用clap解析
    let mut review_args_vec = vec!["gitai".to_string(), "review".to_string()];

    // 获取review之后的所有其他参数
    let review_index = args
        .iter()
        .position(|a| a == "review" || a == "rv")
        .unwrap_or(0);
    if review_index + 1 < args.len() {
        review_args_vec.extend_from_slice(&args[review_index + 1..]);
    }

    tracing::debug!("重构的review命令: {:?}", review_args_vec);

    if let Ok(parsed_args) = GitaiArgs::try_parse_from(&review_args_vec) {
        match parsed_args.command {
            GitaiSubCommand::Review(review_args) => {
                tracing::debug!("解析出来的 review 结构为: {:?}", review_args);
                return review_args;
            }
            _ => panic!("无法解析 git review 命令,命令为: {:?}", args),
        }
    } else {
        tracing::warn!("解析review命令失败");
        // 创建默认的ReviewArgs
        ReviewArgs {
            depth: "medium".to_string(),
            focus: None,
            lang: None,
            format: "text".to_string(),
            output: None,
            ast_grep: false,
            passthrough_args: vec![],
            commit1: None,
            commit2: None,
            stories: None,
            tasks: None,
            defects: None,
            space_id: None,
        }
    }
}

pub fn construct_commit_args(args: &[String]) -> CommitArgs {
    // 重构commit命令参数以便使用clap解析
    let mut commit_args_vec = vec!["gitai".to_string(), "commit".to_string()];

    // 获取commit之后的所有其他参数
    let commit_index = args
        .iter()
        .position(|a| a == "commit" || a == "cm")
        .unwrap_or(0);
    if commit_index + 1 < args.len() {
        commit_args_vec.extend_from_slice(&args[commit_index + 1..]);
    }

    tracing::debug!("重构的commit命令: {:?}", commit_args_vec);

    if let Ok(parsed_args) = GitaiArgs::try_parse_from(&commit_args_vec) {
        match parsed_args.command {
            GitaiSubCommand::Commit(commit_args) => {
                tracing::debug!("解析出来的 commit 结构为: {:?}", commit_args);
                return commit_args;
            }
            _ => panic!("无法解析 git commit 命令,命令为: {:?}", args),
        }
    } else {
        tracing::warn!("解析commit命令失败");
        // 创建默认的CommitArgs
        CommitArgs {
            ast_grep: false,
            depth: None,
            auto_stage: false,
            message: None,
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        }
    }
}

/// Generates custom help information for gitai, including gitai-specific
/// commands and options not included in standard git help.
pub fn generate_gitai_help() -> String {
    let mut help = String::new();

    // Add header and introduction
    help.push_str("gitai: Git with AI assistance\n");
    help.push_str("============================\n\n");
    help.push_str("gitai 是一个增强型 git 工具，提供 AI 辅助功能来简化 git 使用。\n");
    help.push_str("它可以像标准 git 一样使用，同时提供额外的 AI 功能。\n\n");

    // Global options
    help.push_str("全局选项:\n");
    help.push_str("  --ai                启用 AI 功能\n");
    help.push_str("                     （可选，启用该选项会使用 AI 捕获标准输出与错误输出，即使运行成功也会启用 AI 解释，默认仅捕捉错误信息）\n");
    help.push_str("  --noai              禁用 AI 功能\n");
    help.push_str("                     （可选，启用该选项会使得 gitai 退化为标准 git）\n");
    // Subcommands
    help.push_str("Gitai 特有命令:\n");
    help.push_str("  commit (cm)         增强的 git commit 命令，提供 AI 生成提交信息\n");
    help.push_str("    选项:\n");
    help.push_str("      -t, --ast-grep\n");
    help.push_str("                      启用 AstGrep 语法分析以改进提交信息\n");
    help.push_str("      -l, --level=AST_GREP_LEVEL\n");
    help.push_str("                  AstGrep 语法分析程度，\n");
    help.push_str("                      可选值: shallow, medium (默认), deep\n\n");
    help.push_str("      -a, --all       自动暂存所有已跟踪的修改文件（类似 git commit -a）\n");
    help.push_str("      -m, --message   直接传递消息给提交\n");
    help.push_str("      --issue-id=ISSUE_IDS\n");
    help.push_str("                      在提交信息前添加issue ID前缀 (例如: \"#123,#354\")\n");
    help.push_str("      -r, --review        在提交前执行代码评审\n\n");

    help.push_str("  review (rv)          执行 AI 辅助的代码评审\n");
    help.push_str("    选项:\n");
    help.push_str("      --depth=LEVEL    分析深度级别 (默认: medium)\n");
    help.push_str("      --focus=AREA     评审重点区域\n");
    help.push_str("      --lang=LANGUAGE  限制分析到特定语言\n");
    help.push_str("      --format=FORMAT  输出格式 (默认: text)\n");
    help.push_str("      --output=FILE    输出文件\n");
    help.push_str("      --ast-grep    使用 AstGrep 进行增强代码分析（默认）\n");
    help.push_str("      --commit1=COMMIT 第一个提交引用\n");
    help.push_str("      --commit2=COMMIT 第二个提交引用（如果比较两个提交）\n");
    help.push_str("      --stories=IDs    用户故事 ID 列表 (例如: 123,456)\n");
    help.push_str("      --tasks=IDs      任务 ID 列表 (例如: 789,101)\n");
    help.push_str("      --defects=IDs    缺陷 ID 列表 (例如: 202,303)\n");
    help.push_str("      --space-id=ID    DevOps 空间/项目 ID (当指定工作项 ID 时必须提供)\n\n");

    help.push_str("标准 git 命令:\n");
    help.push_str("  所有标准 git 命令都可以正常使用，例如:\n");
    help.push_str("  gitai status, gitai add, gitai push, 等等\n\n");
    help.push_str("示例:\n");
    help.push_str("  gitai commit        使用 AI 辅助生成提交信息\n");
    help.push_str("  gitai commit --noai 禁用 AI，使用标准 git commit\n");
    help.push_str("  gitai review        对当前更改执行 AI 辅助代码评审\n");
    help.push_str("  gitai review --depth=deep --focus=\"性能问题\"\n");
    help.push_str("                      执行深度代码评审，重点关注性能问题\n");

    help.push_str("参考：原始 git 命令:\n");
    help.push_str(
        "
        用法: git [-v | --version] [-h | --help] [-C <路径>] [-c <名称>=<值>]
                   [--exec-path[=<路径>]] [--html-path] [--man-path] [--info-path]
                   [-p | --paginate | -P | --no-pager] [--no-replace-objects] [--bare]
                   [--git-dir=<路径>] [--work-tree=<路径>] [--namespace=<名称>]
                   [--super-prefix=<路径>] [--config-env=<名称>=<环境变量>]
                   <命令> [<参数>]

        以下是不同场景中常用的Git命令：

        开始工作区域（另见：git help tutorial）
           clone     将仓库克隆到新目录
           init      创建空Git仓库或重新初始化现有仓库

        处理当前变更（另见：git help everyday）
           add       将文件内容添加到索引
           mv        移动或重命名文件、目录或符号链接
           restore   恢复工作树文件
           rm        从工作树和索引中移除文件

        查看历史与状态（另见：git help revisions）
           bisect    使用二分查找定位引入缺陷的提交
           diff      显示提交间差异、提交与工作树差异等
           grep      打印匹配指定模式的行
           log       显示提交日志
           show      显示各类对象
           status    显示工作树状态

        扩展、标记和调整公共历史
           branch    列出、创建或删除分支
           commit    记录仓库变更
           merge     合并两个或多个开发历史
           rebase    在另一基底上重新应用提交
           reset     将当前HEAD重置到指定状态
           switch    切换分支
           tag       创建、列出、删除或验证GPG签名的标签对象

        协作（另见：git help workflows）
           fetch     从另一仓库下载对象和引用
           pull      从另一仓库或本地分支获取并整合变更
           push      更新远程引用及其关联对象

        'git help -a' 和 'git help -g' 会列出可用子命令和部分
        概念指南。查看特定子命令或概念请使用 'git help <命令>' 或 'git help <概念>'。
        系统概述请查看 'git help git'。\n\n
        ",
    );
    help
}

/// Get the current Git repository name
pub fn get_git_repo_name() -> Result<String, AppError> {
    let output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| AppError::IO("Failed to get git repository path".to_string(), e))?;
    
    if !output.status.success() {
        return Err(AppError::Generic("Not in a Git repository".to_string()));
    }
    
    let binding = String::from_utf8_lossy(&output.stdout);
    let repo_path = binding.trim();
    let repo_name = Path::new(repo_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AppError::Generic("Failed to extract repository name".to_string()))?;
    
    Ok(repo_name.to_string())
}

/// Get the current commit ID (HEAD)
pub fn get_current_commit_id() -> Result<String, AppError> {
    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .map_err(|e| AppError::IO("Failed to get current commit ID".to_string(), e))?;
    
    if !output.status.success() {
        return Err(AppError::Generic("Failed to get current commit ID".to_string()));
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Expand tilde (~) in file paths to user home directory
pub fn expand_tilde_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    if path.starts_with("~") {
        if let Some(home_dir) = dirs::home_dir() {
            if path == Path::new("~") {
                return home_dir;
            }
            if let Ok(stripped) = path.strip_prefix("~/") {
                return home_dir.join(stripped);
            }
        }
    }
    path.to_path_buf()
}

/// Generate the review file path for the current repository and commit
pub fn generate_review_file_path(
    storage_base_path: &str,
    format: &str,
) -> Result<PathBuf, AppError> {
    let repo_name = get_git_repo_name()?;
    let commit_id = get_current_commit_id()?;
    
    let expanded_base = expand_tilde_path(storage_base_path);
    let file_extension = match format.to_lowercase().as_str() {
        "json" => "json",
        "html" => "html",
        "markdown" | "md" => "md",
        _ => "txt",
    };
    
    let filename = format!("review_{}.{}", commit_id, file_extension);
    let file_path = expanded_base.join(&repo_name).join(filename);
    
    Ok(file_path)
}

/// Find the most recent review file for the current repository
pub fn find_latest_review_file(storage_base_path: &str) -> Result<Option<PathBuf>, AppError> {
    let repo_name = get_git_repo_name()?;
    let expanded_base = expand_tilde_path(storage_base_path);
    let repo_dir = expanded_base.join(&repo_name);
    
    if !repo_dir.exists() {
        return Ok(None);
    }
    
    let mut review_files = Vec::new();
    
    for entry in std::fs::read_dir(&repo_dir)
        .map_err(|e| AppError::IO(format!("Failed to read review directory: {:?}", repo_dir), e))?
    {
        let entry = entry.map_err(|e| AppError::IO("Failed to read directory entry".to_string(), e))?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("review_") && 
                   (filename.ends_with(".md") || filename.ends_with(".txt") || 
                    filename.ends_with(".json") || filename.ends_with(".html")) {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            review_files.push((path, modified));
                        }
                    }
                }
            }
        }
    }
    
    if review_files.is_empty() {
        return Ok(None);
    }
    
    // Sort by modification time, most recent first
    review_files.sort_by(|a, b| b.1.cmp(&a.1));
    
    Ok(Some(review_files[0].0.clone()))
}

/// Read and parse review file content
pub fn read_review_file(file_path: &Path) -> Result<String, AppError> {
    if !file_path.exists() {
        return Err(AppError::Generic(format!("Review file does not exist: {:?}", file_path)));
    }
    
    std::fs::read_to_string(file_path)
        .map_err(|e| AppError::IO(format!("Failed to read review file: {:?}", file_path), e))
}

/// Extract key insights from review content for commit message integration
/// Parse comma-separated issue IDs from a string (e.g., "#123,#354" or "123,354")
pub fn parse_issue_ids(issue_id_str: &str) -> Vec<String> {
    if issue_id_str.trim().is_empty() {
        return Vec::new();
    }
    
    issue_id_str
        .split(',')
        .map(|id| {
            let trimmed = id.trim();
            if trimmed.starts_with('#') {
                trimmed.to_string()
            } else {
                format!("#{}", trimmed)
            }
        })
        .filter(|id| id.len() > 1) // Filter out empty or just "#" entries
        .collect()
}

/// Format issue IDs as a prefix for commit messages
pub fn format_issue_prefix(issue_ids: &[String]) -> String {
    if issue_ids.is_empty() {
        String::new()
    } else {
        format!("{} ", issue_ids.join(","))
    }
}

/// Add issue ID prefix to commit message if issue IDs are provided
pub fn add_issue_prefix_to_commit_message(commit_message: &str, issue_id_option: Option<&String>) -> String {
    match issue_id_option {
        Some(issue_id_str) => {
            let issue_ids = parse_issue_ids(issue_id_str);
            if issue_ids.is_empty() {
                commit_message.to_string()
            } else {
                let prefix = format_issue_prefix(&issue_ids);
                format!("{}{}", prefix, commit_message)
            }
        }
        None => commit_message.to_string(),
    }
}

pub fn extract_review_insights(content: &str) -> String {
    let mut insights = Vec::new();
    
    // Extract lines that look like important findings or suggestions
    for line in content.lines() {
        let line = line.trim();
        
        // Skip empty lines and basic headers
        if line.is_empty() || line.starts_with('#') && line.len() < 50 {
            continue;
        }
        
        // Look for key indicators of important content
        if line.starts_with("- ") || line.starts_with("* ") {
            // Bullet points are often key findings
            if line.len() > 10 && (
                line.to_lowercase().contains("fix") ||
                line.to_lowercase().contains("issue") ||
                line.to_lowercase().contains("improve") ||
                line.to_lowercase().contains("security") ||
                line.to_lowercase().contains("performance") ||
                line.to_lowercase().contains("bug") ||
                line.to_lowercase().contains("error") ||
                line.contains("建议") || line.contains("问题") || line.contains("改进") ||
                line.contains("优化") || line.contains("修复")
            ) {
                insights.push(line.to_string());
            }
        } else if line.contains("建议") || line.contains("问题") || line.contains("改进") ||
                  line.contains("优化") || line.contains("修复") {
            // Chinese keywords for suggestions and issues
            insights.push(line.to_string());
        }
    }
    
    if insights.is_empty() {
        // If no specific insights found, try to get a summary section
        let lines: Vec<&str> = content.lines().collect();
        let mut summary_start = None;
        
        for (i, line) in lines.iter().enumerate() {
            if line.to_lowercase().contains("summary") || 
               line.to_lowercase().contains("总结") ||
               line.to_lowercase().contains("摘要") {
                summary_start = Some(i + 1);
                break;
            }
        }
        
        if let Some(start) = summary_start {
            for line in lines.iter().skip(start).take(5) {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    insights.push(line.to_string());
                }
            }
        }
    }
    
    if insights.is_empty() {
        "基于代码审查结果".to_string()
    } else {
        insights.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::git::{CommaSeparatedU32List, ReviewArgs, CommitArgs};

    fn make_args(vec: Vec<&str>) -> Vec<String> {
        vec.into_iter().map(String::from).collect()
    }

    #[test]
    fn test_construct_review_args_default() {
        let args = make_args(vec!["gitai", "review"]);
        let expected = ReviewArgs {
            depth: "medium".to_string(),
            focus: None,
            lang: None,
            format: "text".to_string(),
            output: None,
            tree_sitter: false,
            passthrough_args: vec![],
            commit1: None,
            commit2: None,
            stories: None,
            tasks: None,
            defects: None,
            space_id: None,
        };
        assert_eq!(construct_review_args(&args), expected);
    }

    #[test]
    fn test_construct_review_args_with_all_options() {
        let args = make_args(vec![
            "gitai", "review",
            "--depth=deep",
            "--focus", "performance",
            "--lang", "Rust",
            "--format", "json",
            "--output", "out.txt",
            "--tree-sitter",
            "--commit1", "abc123",
            "--commit2", "def456",
            "--stories=1,2,3",
            "--tasks=4,5",
            "--defects=6",
            "--space-id=12345",
            "--", "--extra", "flag"
        ]);
        let expected = ReviewArgs {
            depth: "deep".to_string(),
            focus: Some("performance".to_string()),
            lang: Some("Rust".to_string()),
            format: "json".to_string(),
            output: Some("out.txt".to_string()),
            tree_sitter: true,
            passthrough_args: vec!["--extra".to_string(), "flag".to_string()],
            commit1: Some("abc123".to_string()),
            commit2: Some("def456".to_string()),
            stories: Some(CommaSeparatedU32List(vec![1, 2, 3])),
            tasks: Some(CommaSeparatedU32List(vec![4, 5])),
            defects: Some(CommaSeparatedU32List(vec![6])),
            space_id: Some(12345),
        };
        assert_eq!(construct_review_args(&args), expected);
    }

    #[test]
    fn test_construct_review_args_alias_rv() {
        let args = make_args(vec!["gitai", "rv", "--depth=shallow"]);
        let expected = ReviewArgs {
            depth: "shallow".to_string(),
            focus: None,
            lang: None,
            format: "text".to_string(),
            output: None,
            tree_sitter: false,
            passthrough_args: vec![],
            commit1: None,
            commit2: None,
            stories: None,
            tasks: None,
            defects: None,
            space_id: None,
        };
        assert_eq!(construct_review_args(&args), expected);
    }

    #[test]
    fn test_construct_review_args_with_some_work_items() {
        let args = make_args(vec![
            "gitai", "review",
            "--stories=7,8",
            "--space-id=98765",
        ]);
        let expected = ReviewArgs {
            depth: "medium".to_string(),
            focus: None,
            lang: None,
            format: "text".to_string(),
            output: None,
            tree_sitter: false,
            passthrough_args: vec![],
            commit1: None,
            commit2: None,
            stories: Some(CommaSeparatedU32List(vec![7, 8])),
            tasks: None,
            defects: None,
            space_id: Some(98765),
        };
        assert_eq!(construct_review_args(&args), expected);
    }

    #[test]
    fn test_construct_review_args_with_empty_work_item_lists() {
        let args = make_args(vec![
            "gitai", "review",
            "--stories=",
            "--tasks=",
            "--defects=",
            "--space-id=123",
        ]);
        let expected = ReviewArgs {
            depth: "medium".to_string(),
            focus: None,
            lang: None,
            format: "text".to_string(),
            output: None,
            tree_sitter: false,
            passthrough_args: vec![],
            commit1: None,
            commit2: None,
            stories: Some(CommaSeparatedU32List(vec![])),
            tasks: Some(CommaSeparatedU32List(vec![])),
            defects: Some(CommaSeparatedU32List(vec![])),
            space_id: Some(123),
        };
        assert_eq!(construct_review_args(&args), expected);
    }

    #[test]
    fn test_construct_commit_args_default() {
        let args = make_args(vec!["gitai", "commit"]);
        let expected = CommitArgs {
            tree_sitter: false,
            depth: None,
            auto_stage: false,
            message: None,
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };
        assert_eq!(construct_commit_args(&args), expected);
    }

    #[test]
    fn test_construct_commit_args_with_options() {
        let args = make_args(vec![
            "gitai", "commit",
            "-t",
            "-l", "deep",
            "-a",
            "-m", "test commit message",
            "-r",
            "--", "--extra", "flag"
        ]);
        let expected = CommitArgs {
            tree_sitter: true,
            depth: Some("deep".to_string()),
            auto_stage: true,
            message: Some("test commit message".to_string()),
            issue_id: None,
            review: true,
            passthrough_args: vec!["--extra".to_string(), "flag".to_string()],
        };
        assert_eq!(construct_commit_args(&args), expected);
    }

    #[test]
    fn test_construct_commit_args_alias_cm() {
        let args = make_args(vec!["gitai", "cm", "-m", "quick commit"]);
        let expected = CommitArgs {
            tree_sitter: false,
            depth: None,
            auto_stage: false,
            message: Some("quick commit".to_string()),
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };
        assert_eq!(construct_commit_args(&args), expected);
    }

    #[test]
    fn test_construct_commit_args_auto_stage_only() {
        let args = make_args(vec!["gitai", "commit", "-a"]);
        let expected = CommitArgs {
            tree_sitter: false,
            depth: None,
            auto_stage: true,
            message: None,
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };
        assert_eq!(construct_commit_args(&args), expected);
    }

    #[test]
    fn test_construct_commit_args_with_issue_id() {
        let args = make_args(vec!["gitai", "commit", "--issue-id", "#123,#456", "-m", "test message"]);
        let expected = CommitArgs {
            tree_sitter: false,
            depth: None,
            auto_stage: false,
            message: Some("test message".to_string()),
            issue_id: Some("#123,#456".to_string()),
            review: false,
            passthrough_args: vec![],
        };
        assert_eq!(construct_commit_args(&args), expected);
    }

    #[test]
    fn test_construct_commit_args_issue_id_without_hash() {
        let args = make_args(vec!["gitai", "commit", "--issue-id", "123,456"]);
        let expected = CommitArgs {
            tree_sitter: false,
            depth: None,
            auto_stage: false,
            message: None,
            issue_id: Some("123,456".to_string()),
            review: false,
            passthrough_args: vec![],
        };
        assert_eq!(construct_commit_args(&args), expected);
    }

    #[test]
    fn test_expand_tilde_path() {
        // Test with tilde
        let path = expand_tilde_path("~/Documents/test");
        assert!(path.to_string_lossy().contains("Documents/test"));
        
        // Test with just tilde
        let path = expand_tilde_path("~");
        if let Some(home) = dirs::home_dir() {
            assert_eq!(path, home);
        }
        
        // Test without tilde
        let path = expand_tilde_path("/absolute/path");
        assert_eq!(path, Path::new("/absolute/path"));
        
        // Test relative path without tilde
        let path = expand_tilde_path("relative/path");
        assert_eq!(path, Path::new("relative/path"));
    }

    #[test]
    fn test_extract_review_insights() {
        let review_content = r#"
# 代码评审报告

## 主要发现

- 需要修复安全漏洞在登录模块
- 性能问题需要优化数据库查询
- 代码质量良好

## 建议

改进错误处理机制

## 总结

整体代码质量不错，但需要注意安全性问题。
        "#;
        
        let insights = extract_review_insights(review_content);
        assert!(insights.contains("修复安全漏洞"));
        assert!(insights.contains("性能问题需要优化"));
        assert!(insights.contains("改进错误处理机制"));
    }

    #[test]
    fn test_extract_review_insights_empty() {
        let review_content = "# Simple Header\n\nSome basic text without insights.";
        let insights = extract_review_insights(review_content);
        assert_eq!(insights, "基于代码审查结果");
    }

    #[test]
    fn test_extract_review_insights_with_english_keywords() {
        let review_content = r#"
## Issues Found

- Fix memory leak in authentication module
- Improve error handling
- Security vulnerability in input validation

## Performance Analysis

The code has performance issues that need attention.
        "#;
        
        let insights = extract_review_insights(review_content);
        assert!(insights.contains("Fix memory leak"));
        assert!(insights.contains("Improve error handling"));
        assert!(insights.contains("Security vulnerability"));
    }

    #[test]
    fn test_parse_issue_ids() {
        // Test with hash prefixes
        let result = parse_issue_ids("#123,#456");
        assert_eq!(result, vec!["#123", "#456"]);

        // Test without hash prefixes
        let result = parse_issue_ids("123,456");
        assert_eq!(result, vec!["#123", "#456"]);

        // Test mixed format
        let result = parse_issue_ids("#123,456,#789");
        assert_eq!(result, vec!["#123", "#456", "#789"]);

        // Test with spaces
        let result = parse_issue_ids(" #123 , 456 , #789 ");
        assert_eq!(result, vec!["#123", "#456", "#789"]);

        // Test empty string
        let result = parse_issue_ids("");
        assert_eq!(result, Vec::<String>::new());

        // Test single issue
        let result = parse_issue_ids("123");
        assert_eq!(result, vec!["#123"]);
    }

    #[test]
    fn test_format_issue_prefix() {
        // Test with multiple issues
        let issues = vec!["#123".to_string(), "#456".to_string()];
        let result = format_issue_prefix(&issues);
        assert_eq!(result, "#123,#456 ");

        // Test with single issue
        let issues = vec!["#123".to_string()];
        let result = format_issue_prefix(&issues);
        assert_eq!(result, "#123 ");

        // Test with empty vector
        let issues: Vec<String> = vec![];
        let result = format_issue_prefix(&issues);
        assert_eq!(result, "");
    }

    #[test]
    fn test_add_issue_prefix_to_commit_message() {
        let commit_message = "feat: add new feature";

        // Test with issue IDs
        let result = add_issue_prefix_to_commit_message(commit_message, Some(&"#123,#456".to_string()));
        assert_eq!(result, "#123,#456 feat: add new feature");

        // Test without issue IDs
        let result = add_issue_prefix_to_commit_message(commit_message, None);
        assert_eq!(result, "feat: add new feature");

        // Test with empty issue ID string
        let result = add_issue_prefix_to_commit_message(commit_message, Some(&"".to_string()));
        assert_eq!(result, "feat: add new feature");

        // Test with single issue ID
        let result = add_issue_prefix_to_commit_message(commit_message, Some(&"123".to_string()));
        assert_eq!(result, "#123 feat: add new feature");
    }
}

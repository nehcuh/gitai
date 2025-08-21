use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::types::git::{GitaiArgs, GitaiSubCommand, ReviewArgs, CommitArgs, TranslateArgs, CommaSeparatedU32List};
use crate::errors::{AppError, file_error};


pub fn construct_review_args(args: &[String]) -> ReviewArgs {
    let mut review_args = ReviewArgs::default();
    let mut i = 1; // 跳过 "review" 或 "rv" 命令
    
    while i < args.len() {
        match args[i].as_str() {
            "--depth" => {
                if i + 1 < args.len() {
                    review_args.depth = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    break;
                }
            }
            "--focus" => {
                if i + 1 < args.len() {
                    review_args.focus = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    break;
                }
            }
            "--language" => {
                if i + 1 < args.len() {
                    review_args.language = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    break;
                }
            }
            "--format" => {
                if i + 1 < args.len() {
                    review_args.format = args[i + 1].clone();
                    i += 2;
                } else {
                    break;
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    review_args.output = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    break;
                }
            }
            "--tree-sitter" => {
                review_args.tree_sitter = true;
                i += 1;
            }
            "--commit1" => {
                if i + 1 < args.len() {
                    review_args.commit1 = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    break;
                }
            }
            "--commit2" => {
                if i + 1 < args.len() {
                    review_args.commit2 = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    break;
                }
            }
            "--stories" => {
                if i + 1 < args.len() {
                    if let Ok(stories) = args[i + 1].parse::<CommaSeparatedU32List>() {
                        review_args.stories = Some(stories);
                    }
                    i += 2;
                } else {
                    break;
                }
            }
            "--tasks" => {
                if i + 1 < args.len() {
                    if let Ok(tasks) = args[i + 1].parse::<CommaSeparatedU32List>() {
                        review_args.tasks = Some(tasks);
                    }
                    i += 2;
                } else {
                    break;
                }
            }
            "--defects" => {
                if i + 1 < args.len() {
                    if let Ok(defects) = args[i + 1].parse::<CommaSeparatedU32List>() {
                        review_args.defects = Some(defects);
                    }
                    i += 2;
                } else {
                    break;
                }
            }
            "--space-id" => {
                if i + 1 < args.len() {
                    if let Ok(space_id) = args[i + 1].parse::<u32>() {
                        review_args.space_id = Some(space_id);
                    }
                    i += 2;
                } else {
                    break;
                }
            }
            "--" => {
                // 剩余的参数都作为passthrough_args
                review_args.passthrough_args = args[i + 1..].to_vec();
                break;
            }
            _ => {
                // 未识别的参数，跳过
                i += 1;
            }
        }
    }
    
    review_args
}

pub fn construct_commit_args(args: &[String]) -> CommitArgs {
    let mut commit_args = CommitArgs::default();
    let mut i = 1; // 跳过 "commit" 或 "cm" 命令
    
    while i < args.len() {
        match args[i].as_str() {
            "-t" | "--tree-sitter" => {
                commit_args.tree_sitter = true;
                i += 1;
            }
            "-l" | "--depth" => {
                if i + 1 < args.len() {
                    commit_args.depth = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    break;
                }
            }
            "-a" | "--all" => {
                commit_args.auto_stage = true;
                i += 1;
            }
            "-m" | "--message" => {
                if i + 1 < args.len() {
                    commit_args.message = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    break;
                }
            }
            "--issue-id" => {
                if i + 1 < args.len() {
                    commit_args.issue_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    break;
                }
            }
            "-r" | "--review" => {
                commit_args.review = true;
                i += 1;
            }
            "--" => {
                // 剩余的参数都作为passthrough_args
                commit_args.passthrough_args = args[i + 1..].to_vec();
                break;
            }
            _ => {
                // 未识别的参数，跳过
                i += 1;
            }
        }
    }
    
    commit_args
}

pub fn construct_translate_args(args: &[String]) -> TranslateArgs {
    // 重构translate命令参数以便使用clap解析
    let mut translate_args_vec = vec!["gitai".to_string(), "translate".to_string()];

    // 获取translate之后的所有其他参数
    let translate_index = args
        .iter()
        .position(|a| a == "translate")
        .unwrap_or(0);
    if translate_index + 1 < args.len() {
        translate_args_vec.extend_from_slice(&args[translate_index + 1..]);
    }

    tracing::debug!("重构的translate命令: {:?}", translate_args_vec);

    if let Ok(parsed_args) = GitaiArgs::try_parse_from(&translate_args_vec) {
        match parsed_args.command {
            GitaiSubCommand::Translate(translate_args) => {
                tracing::debug!("解析出来的 translate 结构为: {:?}", translate_args);
                return translate_args;
            }
            _ => panic!("无法解析 translate 命令,命令为: {:?}", args),
        }
    } else {
        tracing::warn!("解析translate命令失败");
        // 创建默认的TranslateArgs
        TranslateArgs {
            target: "rules".to_string(),
            force: false,
            output: None,
            to_language: "cn".to_string(),
        }
    }
}

/// Generates custom help information for gitai, including gitai-specific
/// commands and options not included in standard git help.
pub fn generate_gitai_help() -> String {
    let mut help = String::new();

    // Add header and introduction
    help.push_str("\x1b[1;36mgitai: Git with AI assistance\x1b[0m\n");
    help.push_str("===============================\n\n");
    help.push_str("\x1b[1mgitai\x1b[0m 是一个完全兼容的 Git 替代品，在保持 100% Git 兼容性的同时，\n");
    help.push_str("为常见的 Git 操作添加了智能 AI 辅助功能。\n\n");
    help.push_str("\x1b[33m💡 使用提示：\x1b[0m 你可以将 \x1b[1mgitai\x1b[0m 作为 \x1b[1mgit\x1b[0m 的直接替代品使用！\n");
    help.push_str("   例如：\x1b[1mgitai status\x1b[0m, \x1b[1mgitai add .\x1b[0m, \x1b[1mgitai push\x1b[0m 等等\n\n");

    // AI Intelligence Modes Section
    help.push_str("🤖 \x1b[1;32mAI 智能模式\x1b[0m\n");
    help.push_str("─────────────────\n");
    help.push_str("  \x1b[1m--ai\x1b[0m                   强制启用 AI 解释所有命令输出\n");
    help.push_str("                         (成功执行的命令也会显示 AI 分析)\n");
    help.push_str("  \x1b[1m--noai\x1b[0m                 完全禁用 AI，使用纯 Git 行为\n");
    help.push_str("  \x1b[33m默认模式\x1b[0m                只在命令失败时提供 AI 错误解释\n\n");

    // AI-Enhanced Commands Section
    help.push_str("🚀 \x1b[1;34mAI 增强命令\x1b[0m (gitai 特有功能)\n");
    help.push_str("─────────────────────────────────\n");
    
    // Commit command
    help.push_str("  \x1b[1mcommit\x1b[0m (别名: \x1b[1mcm\x1b[0m)      AI 智能提交信息生成\n");
    help.push_str("    \x1b[36m-t, --tree-sitter\x1b[0m     启用语法分析增强提交信息质量\n");
    help.push_str("    \x1b[36m-l, --level LEVEL\x1b[0m     分析深度: shallow | medium | deep\n");
    help.push_str("    \x1b[36m-a, --all\x1b[0m             自动暂存已跟踪文件 (同 git commit -a)\n");
    help.push_str("    \x1b[36m-m, --message MSG\x1b[0m     指定提交信息 (禁用 AI 生成)\n");
    help.push_str("    \x1b[36m--issue-id IDS\x1b[0m        添加 issue 前缀 (如: \"#123,#456\")\n");
    help.push_str("    \x1b[36m-r, --review\x1b[0m          提交前自动执行代码评审\n\n");
    
    // Review command
    help.push_str("  \x1b[1mreview\x1b[0m (别名: \x1b[1mrv\x1b[0m)      AI 代码评审和质量分析\n");
    help.push_str("    \x1b[36m--depth LEVEL\x1b[0m         分析深度: shallow | medium | deep\n");
    help.push_str("    \x1b[36m--focus AREA\x1b[0m          重点关注领域 (如: \"性能\", \"安全\")\n");
    help.push_str("    \x1b[36m--lang LANGUAGE\x1b[0m       限制分析的编程语言\n");
    help.push_str("    \x1b[36m--format FORMAT\x1b[0m       输出格式: text | json | markdown\n");
    help.push_str("    \x1b[36m--output FILE\x1b[0m         保存结果到文件\n");
    help.push_str("    \x1b[36m--commit1 HASH\x1b[0m        指定第一个提交 (比较模式)\n");
    help.push_str("    \x1b[36m--commit2 HASH\x1b[0m        指定第二个提交 (比较模式)\n");
    help.push_str("    \x1b[36m--stories IDS\x1b[0m         关联用户故事 ID\n");
    help.push_str("    \x1b[36m--tasks IDS\x1b[0m           关联任务 ID\n");
    help.push_str("    \x1b[36m--defects IDS\x1b[0m         关联缺陷 ID\n");
    help.push_str("    \x1b[36m--space-id ID\x1b[0m         DevOps 空间/项目 ID\n");
    help.push_str("    \x1b[36m--scan-results PATH\x1b[0m    使用扫描结果辅助评审 (文件路径或提交ID)\n\n");
    
    // Scan command
    help.push_str("  \x1b[1mscan\x1b[0m                   代码安全和质量扫描\n");
    help.push_str("    \x1b[36m--path PATH\x1b[0m           指定扫描路径 (默认: 当前目录)\n");
    help.push_str("    \x1b[36m--full\x1b[0m                全量扫描 (默认: 增量扫描)\n");
    help.push_str("    \x1b[36m--update-rules\x1b[0m        强制更新扫描规则\n");
    help.push_str("    \x1b[36m--output FILE\x1b[0m         保存扫描结果\n");
    help.push_str("    \x1b[36m--remote\x1b[0m              使用远程扫描服务\n\n");
    
    // Translate command
    help.push_str("  \x1b[1mtranslate\x1b[0m              AI 翻译各种资源\n");
    help.push_str("    \x1b[36mTARGET\x1b[0m                翻译目标 (目前支持: rules)\n");
    help.push_str("    \x1b[36m-f, --force\x1b[0m           强制重新翻译已存在的文件\n");
    help.push_str("    \x1b[36m-o, --output DIR\x1b[0m      指定翻译结果输出目录\n");
    help.push_str("    \x1b[36m-l, --to-lang LANG\x1b[0m    目标语言 (cn|us, 默认: cn)\n\n");

    // Standard Git Commands Section  
    help.push_str("📦 \x1b[1;35m标准 Git 命令\x1b[0m (完全兼容)\n");
    help.push_str("─────────────────────────────\n");
    help.push_str("  所有标准 Git 命令都可以直接使用，并自动获得智能错误解释：\n");
    help.push_str("  \x1b[1mgitai status\x1b[0m, \x1b[1mgitai add\x1b[0m, \x1b[1mgitai push\x1b[0m, \x1b[1mgitai pull\x1b[0m, \x1b[1mgitai merge\x1b[0m, \x1b[1mgitai rebase\x1b[0m...\n\n");

    // Management Commands Section
    help.push_str("🔧 \x1b[1;33m管理命令\x1b[0m\n");
    help.push_str("─────────────\n");
    help.push_str("  \x1b[1mupdate-queries\x1b[0m         更新 Tree-sitter 查询文件\n");
    help.push_str("  \x1b[1mcleanup-queries\x1b[0m        清理无用的查询文件\n");
    help.push_str("  \x1b[1mquery-status\x1b[0m           显示查询文件状态\n");
    help.push_str("  \x1b[1mupdate-scan-rules\x1b[0m      更新代码扫描规则\n");
    help.push_str("  \x1b[1minstall-ast-grep\x1b[0m       自动安装 ast-grep 可执行文件\n");
    help.push_str("  \x1b[1mcheck-ast-grep\x1b[0m         检查 ast-grep 安装状态\n");
    help.push_str("  \x1b[1mstart-mcp\x1b[0m              启动 GitAI MCP 服务 (Model Context Protocol)\n\n");

    // Usage Examples Section
    help.push_str("📚 \x1b[1;37m使用示例\x1b[0m\n");
    help.push_str("─────────────\n");
    help.push_str("  \x1b[32m# AI 增强的提交流程\x1b[0m\n");
    help.push_str("  gitai add .                    # 添加文件\n");
    help.push_str("  gitai commit                   # AI 生成提交信息\n");
    help.push_str("  gitai commit -r                # 提交前自动代码评审\n\n");
    
    help.push_str("  \x1b[32m# 代码质量分析\x1b[0m\n");
    help.push_str("  gitai review                   # 评审当前更改\n");
    help.push_str("  gitai review --depth=deep --focus=\"性能优化\"\n");
    help.push_str("  gitai review --scan-results=abc123  # 结合扫描结果评审\n");
    help.push_str("  gitai scan                     # 代码安全扫描\n");
    help.push_str("  gitai scan --full --update-rules\n\n");
    
    help.push_str("  \x1b[32m# ast-grep 工具管理\x1b[0m\n");
    help.push_str("  gitai check-ast-grep           # 检查 ast-grep 安装状态\n");
    help.push_str("  gitai install-ast-grep         # 自动安装 ast-grep\n\n");
    
    help.push_str("  \x1b[32m# MCP 服务管理\x1b[0m\n");
    help.push_str("  gitai start-mcp                # 启动 GitAI MCP 服务\n");
    help.push_str("                                 # 使 GitAI 功能通过 MCP 协议可用\n\n");
    
    help.push_str("  \x1b[32m# 标准 Git 操作 (带智能错误提示)\x1b[0m\n");
    help.push_str("  gitai status                   # 查看状态\n");
    help.push_str("  gitai push origin main         # 推送到远程\n");
    help.push_str("  gitai merge feature-branch     # 合并分支\n");
    help.push_str("  gitai rebase main              # 变基操作\n\n");
    
    help.push_str("  \x1b[32m# AI 模式控制\x1b[0m\n");
    help.push_str("  gitai --ai status              # 强制 AI 解释成功输出\n");
    help.push_str("  gitai --noai commit            # 禁用 AI，纯 Git 行为\n\n");

    // Quick Reference Section
    help.push_str("📖 \x1b[1;36m快速参考\x1b[0m\n");
    help.push_str("─────────────\n");
    help.push_str("  \x1b[33m获取更多帮助：\x1b[0m\n");
    help.push_str("  gitai help                     # 显示此帮助信息\n");
    help.push_str("  gitai <command> --help         # 获取具体命令帮助\n");
    help.push_str("  git help <git-command>         # 查看标准 Git 命令帮助\n\n");
    
    help.push_str("  \x1b[33m版本信息：\x1b[0m\n");
    help.push_str("  gitai --version                # 显示 gitai 版本\n");
    help.push_str("  git --version                  # 显示底层 Git 版本\n\n");
    
    help.push_str("\x1b[90m💡 提示：gitai 是 Git 的完全兼容替代品，所有 Git 命令都能正常工作！\x1b[0m\n");
    help.push_str("\x1b[90m🔗 更多信息：https://github.com/your-repo/gitai\x1b[0m\n");
    help
}

/// Get the current Git repository name
pub fn get_git_repo_name() -> Result<String, AppError> {
    let output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .map_err(AppError::IO)?;
    
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
        .map_err(AppError::IO)?;
    
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
        .map_err(|e| file_error(format!("Failed to read review directory: {:?}: {}", repo_dir, e)))?
    {
        let entry = entry.map_err(|e| file_error(format!("Failed to read directory entry: {}", e)))?;
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
        .map_err(|e| file_error(format!("Failed to read review file: {:?}: {}", file_path, e)))
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
            language: None,
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
            "--language", "Rust",
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
            language: Some("Rust".to_string()),
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
            language: None,
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
            language: None,
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
            language: None,
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

pub fn construct_scan_args(args: &[String]) -> crate::types::git::ScanArgs {
    let mut scan_args = crate::types::git::ScanArgs::default();
    let mut i = 1; // 跳过 "scan" 命令
    
    while i < args.len() {
        match args[i].as_str() {
            "--path" => {
                if i + 1 < args.len() {
                    scan_args.path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    break;
                }
            }
            "--full" => {
                scan_args.full = true;
                i += 1;
            }
            "--remote" => {
                scan_args.remote = true;
                i += 1;
            }
            "--update-rules" => {
                scan_args.update_rules = true;
                i += 1;
            }
            "--output" => {
                if i + 1 < args.len() {
                    scan_args.output = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    break;
                }
            }
            "--format" => {
                if i + 1 < args.len() {
                    scan_args.format = args[i + 1].clone();
                    i += 2;
                } else {
                    break;
                }
            }
            "--translate" => {
                scan_args.translate = true;
                i += 1;
            }
            "--language" => {
                if i + 1 < args.len() {
                    scan_args.language = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    break;
                }
            }
            "--focus" => {
                if i + 1 < args.len() {
                    scan_args.focus = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    break;
                }
            }
            "--tool" => {
                if i + 1 < args.len() {
                    if let Ok(tool) = args[i + 1].parse::<crate::types::scan::types::ScanTool>() {
                        scan_args.tool = Some(tool);
                    }
                    i += 2;
                } else {
                    break;
                }
            }
            "--" => {
                // 剩余的参数都作为passthrough_args
                scan_args.passthrough_args = args[i + 1..].to_vec();
                break;
            }
            _ => {
                // 未识别的参数，跳过
                i += 1;
            }
        }
    }
    
    scan_args
}
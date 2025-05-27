use clap::Parser;

use crate::types::git::{GitaiArgs, GitaiSubCommand, ReviewArgs};

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
            tree_sitter: false,
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
    help.push_str("      -t, --tree-sitter\n");
    help.push_str("                      启用 Tree-sitter 语法分析以改进提交信息\n");
    help.push_str("      -l, --level=TREESITTER_LEVEL\n");
    help.push_str("                  Tree-sitter 语法分析程度，\n");
    help.push_str("                      可选值: shallow, medium (默认), deep\n\n");
    help.push_str("      -a, --all       自动暂存所有已跟踪的修改文件（类似 git commit -a）\n");
    help.push_str("      -m, --message   直接传递消息给提交\n");
    help.push_str("      -r, --review        在提交前执行代码评审\n\n");

    help.push_str("  review (rv)          执行 AI 辅助的代码评审\n");
    help.push_str("    选项:\n");
    help.push_str("      --depth=LEVEL    分析深度级别 (默认: medium)\n");
    help.push_str("      --focus=AREA     评审重点区域\n");
    help.push_str("      --lang=LANGUAGE  限制分析到特定语言\n");
    help.push_str("      --format=FORMAT  输出格式 (默认: text)\n");
    help.push_str("      --output=FILE    输出文件\n");
    help.push_str("      --tree-sitter    使用 Tree-sitter 进行增强代码分析（默认）\n");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::git::ReviewArgs;

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
            stories: Some(vec![1, 2, 3]),
            tasks: Some(vec![4, 5]),
            defects: Some(vec![6]),
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
            stories: Some(vec![7, 8]),
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
            stories: Some(vec![]),
            tasks: Some(vec![]),
            defects: Some(vec![]),
            space_id: Some(123),
        };
        assert_eq!(construct_review_args(&args), expected);
    }
}

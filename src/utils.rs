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

    help.push_str("  review (rv)         执行 AI 辅助的代码评审\n");
    help.push_str("    选项:\n");
    help.push_str("      --depth=LEVEL   分析深度级别 (默认: medium)\n");
    help.push_str("      --focus=AREA    评审重点区域\n");
    help.push_str("      --lang=LANGUAGE 限制分析到特定语言\n");
    help.push_str("      --format=FORMAT 输出格式 (默认: text)\n");
    help.push_str("      --output=FILE   输出文件\n");
    help.push_str("      --ts            使用 Tree-sitter 进行增强代码分析（默认）\n");
    help.push_str("      --no-ts         禁用 Tree-sitter 分析\n");
    help.push_str("      --review-ts     结合评审与 tree-sitter 分析\n");
    help.push_str("      --commit1=COMMIT 第一个提交引用\n");
    help.push_str("      --commit2=COMMIT 第二个提交引用（如果比较两个提交）\n\n");

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

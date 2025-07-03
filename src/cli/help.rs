use crate::common::SupportedLanguage;

/// 生成 GitAI 帮助信息
pub fn generate_help(language: Option<&SupportedLanguage>) -> String {
    match language {
        Some(SupportedLanguage::Chinese) => generate_help_zh(),
        Some(SupportedLanguage::English) => generate_help_en(),
        _ => generate_help_en(), // 默认英文
    }
}

/// 生成英文帮助信息
fn generate_help_en() -> String {
    r#"GitAI - AI-powered Git Tools Suite

USAGE:
    gitai [OPTIONS] [COMMAND] [ARGS]...

OPTIONS:
    --ai           Enable AI functionality globally for all commands
    --noai         Disable AI functionality globally for all commands  
    --lang <LANG>  Specify output language (zh|en|auto)
    -h, --help     Print help information
    -V, --version  Print version information

COMMANDS:
    commit, cm         AI-powered commit message generation
    review, rv         AI-driven code review
    scan, sc          AST-Grep powered code scanning
    update-rules, ur  Update AST-Grep rules
    help              Show help information

EXAMPLES:
    # Basic AI-powered commit
    gitai commit

    # Commit with AST-Grep analysis
    gitai commit --ast-grep

    # Code review with focus area
    gitai review --focus="security"

    # Global AI mode for any git command
    gitai --ai status

    # Chinese output
    gitai --lang=zh review

    # Scan current directory
    gitai scan

For more information on a specific command, use:
    gitai <command> --help

GitAI integrates AI capabilities into your Git workflow for smarter
development experience."#.to_string()
}

/// 生成中文帮助信息
fn generate_help_zh() -> String {
    r#"GitAI - AI 驱动的 Git 工具套件

用法:
    gitai [选项] [命令] [参数]...

选项:
    --ai           为所有命令全局启用 AI 功能
    --noai         为所有命令全局禁用 AI 功能
    --lang <语言>   指定输出语言 (zh|en|auto)
    -h, --help     显示帮助信息
    -V, --version  显示版本信息

命令:
    commit, cm         AI 驱动的提交信息生成
    review, rv         AI 驱动的代码审查
    scan, sc          AST-Grep 驱动的代码扫描
    update-rules, ur  更新 AST-Grep 规则
    help              显示帮助信息

示例:
    # 基本的 AI 提交
    gitai commit

    # 使用 AST-Grep 分析的提交
    gitai commit --ast-grep

    # 关注安全性的代码审查
    gitai review --focus="安全性"

    # 全局 AI 模式用于任何 git 命令
    gitai --ai status

    # 中文输出
    gitai --lang=zh review

    # 扫描当前目录
    gitai scan

查看特定命令的详细信息:
    gitai <命令> --help

GitAI 将 AI 能力集成到您的 Git 工作流中，提供更智能的
开发体验。"#.to_string()
}

/// 生成命令特定的帮助信息
pub fn generate_command_help(command: &str, language: Option<&SupportedLanguage>) -> String {
    match command {
        "commit" | "cm" => generate_commit_help(language),
        "review" | "rv" => generate_review_help(language),
        "scan" | "sc" => generate_scan_help(language),
        "update-rules" | "ur" => generate_update_rules_help(language),
        _ => generate_help(language),
    }
}

fn generate_commit_help(language: Option<&SupportedLanguage>) -> String {
    match language {
        Some(SupportedLanguage::Chinese) => {
            r#"gitai commit - AI 驱动的提交信息生成

用法:
    gitai commit [选项]

选项:
    -t, --ast-grep      启用 AST-Grep 分析以增强提交信息
    -a, --all          自动暂存所有已跟踪的修改文件
    -m, --message <信息> 指定提交信息
    --issue-id <ID>    包含 issue ID 到提交信息中
    -r, --review       提交前执行代码审查
    -h, --help         显示帮助信息

示例:
    gitai commit                    # 基本 AI 提交
    gitai commit --ast-grep         # 使用 AST-Grep 分析
    gitai commit -m "修复bug"       # 自定义消息
    gitai commit --issue-id="123"   # 包含 issue ID"#.to_string().to_string()
        },
        _ => {
            r#"gitai commit - AI-powered commit message generation

USAGE:
    gitai commit [OPTIONS]

OPTIONS:
    -t, --ast-grep      Enable AST-Grep analysis for enhanced commit messages
    -a, --all          Automatically stage all tracked, modified files
    -m, --message <MSG> Specify commit message
    --issue-id <ID>    Include issue ID in commit message
    -r, --review       Perform code review before commit
    -h, --help         Print help information

EXAMPLES:
    gitai commit                    # Basic AI commit
    gitai commit --ast-grep         # With AST-Grep analysis
    gitai commit -m "Fix bug"       # Custom message
    gitai commit --issue-id="123"   # Include issue ID"#.to_string().to_string()
        }
    }
}

fn generate_review_help(language: Option<&SupportedLanguage>) -> String {
    match language {
        Some(SupportedLanguage::Chinese) => {
            r#"gitai review - AI 驱动的代码审查

用法:
    gitai review [选项]

选项:
    --focus <领域>      审查关注领域
    --format <格式>     输出格式 (text|markdown|json)
    --output <文件>     输出文件
    --ast-grep         启用 AST-Grep 分析
    --no-scan          禁用自动代码扫描
    --force-scan       强制新扫描（忽略缓存）
    --commit1 <提交>    第一个提交引用
    --commit2 <提交>    第二个提交引用
    -h, --help         显示帮助信息

示例:
    gitai review                    # 基本代码审查
    gitai review --focus="安全性"    # 关注安全性
    gitai review --format=markdown  # Markdown 格式输出"#.to_string()
        },
        _ => {
            r#"gitai review - AI-driven code review

USAGE:
    gitai review [OPTIONS]

OPTIONS:
    --focus <AREA>      Focus areas for the review
    --format <FORMAT>   Output format (text|markdown|json)
    --output <FILE>     Output file
    --ast-grep         Enable AST-Grep analysis
    --no-scan          Disable automatic code scanning
    --force-scan       Force new scan (ignore cache)
    --commit1 <COMMIT>  First commit reference
    --commit2 <COMMIT>  Second commit reference
    -h, --help         Print help information

EXAMPLES:
    gitai review                    # Basic code review
    gitai review --focus="security" # Focus on security
    gitai review --format=markdown  # Markdown output"#.to_string()
        }
    }
}

fn generate_scan_help(language: Option<&SupportedLanguage>) -> String {
    match language {
        Some(SupportedLanguage::Chinese) => {
            r#"gitai scan - AST-Grep 驱动的代码扫描

用法:
    gitai scan [选项] [路径]

选项:
    --languages <语言>   扫描的编程语言（逗号分隔）
    --rules <规则>      运行的特定规则（逗号分隔）
    --severity <级别>   规则严重级别
    --format <格式>     输出格式 (text|json|sarif|csv)
    --output <文件>     输出文件路径
    --parallel         启用并行处理
    --verbose          详细输出
    --stats           显示规则统计
    -h, --help        显示帮助信息

示例:
    gitai scan                     # 扫描当前目录
    gitai scan src/               # 扫描 src 目录
    gitai scan --languages=rust   # 只扫描 Rust 代码"#.to_string()
        },
        _ => {
            r#"gitai scan - AST-Grep powered code scanning

USAGE:
    gitai scan [OPTIONS] [PATH]

OPTIONS:
    --languages <LANGS> Programming languages to scan (comma-separated)
    --rules <RULES>     Specific rules to run (comma-separated)
    --severity <LEVEL>  Rule severity levels
    --format <FORMAT>   Output format (text|json|sarif|csv)
    --output <FILE>     Output file path
    --parallel         Enable parallel processing
    --verbose          Verbose output
    --stats           Show rule statistics
    -h, --help        Print help information

EXAMPLES:
    gitai scan                     # Scan current directory
    gitai scan src/               # Scan src directory
    gitai scan --languages=rust   # Scan only Rust code"#.to_string()
        }
    }
}

fn generate_update_rules_help(language: Option<&SupportedLanguage>) -> String {
    match language {
        Some(SupportedLanguage::Chinese) => {
            r#"gitai update-rules - 更新 AST-Grep 规则

用法:
    gitai update-rules [选项]

选项:
    --source <来源>     更新规则的来源 (github|local|url)
    --repository <仓库> 特定的仓库或 URL
    --reference <引用>  使用的分支或标签
    --target-dir <目录> 规则的目标目录
    --force           强制更新即使规则更新
    --backup          更新前备份现有规则
    --verify          下载后验证规则
    --list-sources    列出可用的规则源
    --verbose         详细输出
    -h, --help        显示帮助信息

示例:
    gitai update-rules             # 从默认源更新
    gitai update-rules --force     # 强制更新
    gitai update-rules --backup    # 备份后更新"#.to_string()
        },
        _ => {
            r#"gitai update-rules - Update AST-Grep rules

USAGE:
    gitai update-rules [OPTIONS]

OPTIONS:
    --source <SOURCE>   Source to update rules from (github|local|url)
    --repository <REPO> Specific repository or URL
    --reference <REF>   Branch or tag to use
    --target-dir <DIR>  Target directory for rules
    --force            Force update even if rules are newer
    --backup           Backup existing rules before update
    --verify           Verify rules after download
    --list-sources     List available rule sources
    --verbose          Verbose output
    -h, --help         Print help information

EXAMPLES:
    gitai update-rules             # Update from default source
    gitai update-rules --force     # Force update
    gitai update-rules --backup    # Update with backup"#.to_string()
        }
    }
}
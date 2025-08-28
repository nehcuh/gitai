use clap::Parser;
use std::path::PathBuf;

/// GitAI - 简化的参数解析
#[derive(Parser, Debug)]
#[command(name = "gitai")]
#[command(about = "AI驱动的Git工作流助手 - 智能提交、代码评审、安全扫描")]
pub struct Args {
    /// 子命令
    #[command(subcommand)]
    pub command: Command,
    
    /// 启用AI解释（为Git命令输出提供智能解释和建议）
    #[arg(long)]
    pub ai: bool,

    /// 显式禁用AI（用于覆盖默认或别名设置）
    #[arg(long)]
    pub noai: bool,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// AI驱动的代码评审（支持安全扫描集成）
    Review {
        /// 语言
        #[arg(long)]
        language: Option<String>,
        /// 输出格式
        #[arg(long, default_value = "text")]
        format: String,
        /// 输出文件
        #[arg(long)]
        output: Option<PathBuf>,
        /// 启用Tree-sitter
        #[arg(long)]
        tree_sitter: bool,
        /// 启用安全扫描
        #[arg(long)]
        security_scan: bool,
        /// 扫描工具
        #[arg(long)]
        scan_tool: Option<String>,
        /// 阻止严重问题
        #[arg(long)]
        block_on_critical: bool,
        /// 关联的Issue ID
        #[arg(long)]
        issue_id: Option<String>,
        /// 启用偏离度分析
        #[arg(long)]
        deviation_analysis: bool,
    },
    /// 代码安全扫描（基于OpenGrep）
    Scan {
        /// 扫描路径
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        /// 扫描工具 (opengrep|auto)
        #[arg(long, default_value = "auto")]
        tool: String,
        /// 全量扫描
        #[arg(long)]
        full: bool,
        /// 远程扫描
        #[arg(long)]
        remote: bool,
        /// 更新规则
        #[arg(long)]
        update_rules: bool,
        /// 输出格式
        #[arg(long, default_value = "text")]
        format: String,
        /// 输出文件
        #[arg(long)]
        output: Option<PathBuf>,
        /// 启用翻译
        #[arg(long)]
        translate: bool,
        /// 自动安装缺失的工具
        #[arg(long)]
        auto_install: bool,
        /// 指定规则语言（例如：java、python），将直接使用对应子目录，跳过自动检测
        #[arg(long)]
        lang: Option<String>,
        /// 不保存扫描历史（用于基准测试/提高性能）
        #[arg(long)]
        no_history: bool,
        /// 覆盖超时时间（秒），直通 opengrep --timeout
        #[arg(long)]
        timeout: Option<u64>,
        /// 基准模式：禁用历史、跳过版本查询等非必要逻辑
        #[arg(long)]
        benchmark: bool,
    },
    /// 查看历史扫描记录
    ScanHistory {
        /// 显示最近N次扫描
        #[arg(long, default_value = "10")]
        limit: usize,
        /// 输出格式
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// 管理AI提示词模板
    Prompts {
        #[command(subcommand)]
        action: PromptAction,
    },
    /// 智能提交（自动生成信息，支持Issue关联）
    Commit {
        /// 提交信息
        #[arg(short, long)]
        message: Option<String>,
        /// 关联的Issue ID
        #[arg(long)]
        issue_id: Option<String>,
        /// 添加所有变更文件
        #[arg(short, long)]
        all: bool,
        /// 启用代码评审
        #[arg(long)]
        review: bool,
        /// 启用Tree-sitter结构分析
        #[arg(long)]
        tree_sitter: bool,
        /// 测试运行，不实际提交
        #[arg(long)]
        dry_run: bool,
    },
    /// 更新安全扫描规则库
    Update {
        /// 仅检查状态，不执行更新
        #[arg(long)]
        check: bool,
        /// 输出格式（text|json）
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// 通用Git命令（带AI解释）
    #[command(external_subcommand)]
    Git(Vec<String>),
    /// 启动MCP服务器
    Mcp {
        /// 传输协议 (stdio|tcp|sse)
        #[arg(long, default_value = "stdio")]
        transport: String,
        /// 监听地址 (tcp/sse)
        #[arg(long, default_value = "127.0.0.1:8080")]
        addr: String,
    },
}

/// 提示词操作
#[derive(Parser, Debug)]
pub enum PromptAction {
    /// 列出可用提示词
    List,
    /// 显示提示词内容
    Show { 
        /// 提示词名称
        name: String,
        /// 语言
        #[arg(long)]
        language: Option<String>,
    },
    /// 更新所有提示词
    Update,
    /// 初始化提示词目录
    Init,
}

impl Args {
    /// 解析命令行参数
    pub fn parse() -> Self {
        <Self as clap::Parser>::parse()
    }
}
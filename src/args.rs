use clap::Parser;
use std::path::PathBuf;

/// GitAI - 简化的参数解析
#[derive(Parser, Debug)]
#[command(name = "gitai")]
#[command(about = "Git with AI assistance")]
pub struct Args {
    /// 子命令
    #[command(subcommand)]
    pub command: Command,
    
    /// 禁用AI功能
    #[arg(long)]
    pub noai: bool,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// 代码评审
    Review {
        /// 分析深度
        #[arg(long)]
        depth: Option<String>,
        /// 关注点
        #[arg(long)]
        focus: Option<String>,
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
    },
    /// 提交
    Commit {
        /// 自定义提交信息
        #[arg(short, long)]
        message: Option<String>,
        /// 启用Tree-sitter
        #[arg(short, long)]
        tree_sitter: bool,
        /// 自动暂存
        #[arg(short, long)]
        auto_stage: bool,
        /// 关联Issue ID
        #[arg(long)]
        issue_id: Option<String>,
        /// 启用审查
        #[arg(long)]
        review: bool,
    },
    /// 安全扫描
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
    },
    /// 查看扫描历史
    ScanHistory {
        /// 显示最近N次扫描
        #[arg(long, default_value = "10")]
        limit: usize,
        /// 输出格式
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// 通用Git命令（带AI解释）
    #[command(external_subcommand)]
    Git(Vec<String>),
}

impl Args {
    /// 解析命令行参数
    pub fn parse() -> Self {
        <Self as clap::Parser>::parse()
    }
}
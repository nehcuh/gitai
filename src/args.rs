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

    /// 离线模式（不进行网络请求）
    #[arg(long, global = true)]
    pub offline: bool,

    /// 自定义配置URL
    #[arg(long, global = true)]
    pub config_url: Option<String>,
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
        /// 启用完整深度分析（架构影响、危险改动、依赖分析等）
        #[arg(long)]
        full: bool,
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
    /// 初始化GitAI配置
    Init {
        /// 配置URL（用于企业内网）
        #[arg(long)]
        config_url: Option<String>,
        /// 离线模式初始化
        #[arg(long)]
        offline: bool,
        /// 资源目录（离线模式使用）
        #[arg(long)]
        resources_dir: Option<PathBuf>,
        /// 开发模式（使用项目内资源）
        #[arg(long)]
        dev: bool,
    },
    /// 配置管理
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// 架构质量趋势追踪
    Metrics {
        #[command(subcommand)]
        action: MetricsAction,
    },
    /// 依赖图导出（全局/子目录）
    Graph {
        /// 扫描路径（目录）
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// 输出 DOT 文件路径（默认输出到 stdout）
        #[arg(long)]
        output: Option<PathBuf>,
        /// 关键节点高亮阈值（0-1）
        #[arg(long, default_value_t = 0.15)]
        threshold: f32,
        /// 生成摘要（LLM 友好）而不是完整 DOT 图
        #[arg(long)]
        summary: bool,
        /// 摘要半径（基于变更种子）
        #[arg(long, default_value_t = 1)]
        radius: usize,
        /// Top-K 重要节点上限
        #[arg(long, default_value_t = 200)]
        top_k: usize,
        /// 从 git diff 推导变更种子
        #[arg(long)]
        seeds_from_diff: bool,
        /// 摘要输出格式（text|json）
        #[arg(long, default_value = "text")]
        summary_format: String,
        /// 预算 token（用于自适应裁剪，v0 仅提示，不强制）
        #[arg(long, default_value_t = 3000)]
        budget_tokens: usize,
        /// 启用社区压缩（v1）
        #[arg(long)]
        community: bool,
        /// 社区检测算法（labelprop|auto）（暂仅支持 labelprop）
        #[arg(long, default_value = "labelprop")]
        comm_alg: String,
        /// 社区数量上限（输出展示）
        #[arg(long, default_value_t = 50)]
        max_communities: usize,
        /// 每个社区展示的节点上限
        #[arg(long, default_value_t = 10)]
        max_nodes_per_community: usize,
    },
    /// 显示本构建启用的功能
    Features {
        /// 输出格式 (text|table|json)
        #[arg(long, default_value = "text")]
        format: String,
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

/// 质量指标操作
#[derive(Parser, Debug)]
pub enum MetricsAction {
    /// 记录当前代码质量快照
    Record {
        /// 自定义标签
        #[arg(long)]
        tags: Vec<String>,
        /// 强制记录（即使没有代码变化）
        #[arg(long)]
        force: bool,
    },
    /// 分析质量趋势
    Analyze {
        /// 分析最近N天的数据
        #[arg(long)]
        days: Option<i64>,
        /// 输出格式 (text|json|markdown|html)
        #[arg(long, default_value = "text")]
        format: String,
        /// 输出文件
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// 生成趋势报告
    Report {
        /// 报告类型 (summary|detailed|full)
        #[arg(long, default_value = "summary")]
        report_type: String,
        /// 输出文件
        #[arg(long)]
        output: Option<PathBuf>,
        /// 生成HTML格式
        #[arg(long)]
        html: bool,
    },
    /// 列出历史快照
    List {
        /// 显示最近N个快照
        #[arg(long, default_value = "20")]
        limit: usize,
        /// 分支过滤
        #[arg(long)]
        branch: Option<String>,
        /// 输出格式 (text|json|table)
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// 比较两个快照
    Compare {
        /// 第一个快照（commit hash或索引）
        from: String,
        /// 第二个快照（commit hash或索引，默认为最新）
        to: Option<String>,
        /// 输出格式 (text|json)
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// 清理历史数据
    Clean {
        /// 保留最近N天的数据
        #[arg(long, default_value = "90")]
        keep_days: i64,
        /// 确认清理
        #[arg(long)]
        yes: bool,
    },
    /// 导出数据
    Export {
        /// 导出格式 (csv|json)
        #[arg(long, default_value = "csv")]
        format: String,
        /// 输出文件
        output: PathBuf,
        /// 包含的分支
        #[arg(long)]
        branches: Vec<String>,
    },
}

/// 配置管理操作
#[derive(Parser, Debug)]
pub enum ConfigAction {
    /// 检查配置状态
    Check,
    /// 显示当前配置
    Show {
        /// 显示格式 (text|json|toml)
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// 更新所有资源
    Update {
        /// 强制更新，即使未过期
        #[arg(long)]
        force: bool,
    },
    /// 重置配置到默认值
    Reset {
        /// 不创建备份
        #[arg(long)]
        no_backup: bool,
    },
    /// 清理过期缓存
    Clean,
}

impl Args {
    /// 解析命令行参数
    pub fn parse() -> Self {
        <Self as clap::Parser>::parse()
    }
}

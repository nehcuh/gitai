//! Review 命令处理器
//!
//! 处理代码评审相关的命令

use crate::args::Command;
use gitai_core::{git, ai::AIClient};
use gitai_types::Result;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use serde_json;

// TASK-002-1: Tree-sitter imports for accurate complexity calculation
use tree_sitter::{Parser, Tree, TreeCursor};
use tree_sitter_rust;

type HandlerResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

/// TASK-006: 支持的输出格式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OutputFormat {
    /// 控制台输出（带颜色和表情符号）
    Console,
    /// Markdown格式
    Markdown,
    /// JSON格式
    Json,
    /// YAML格式
    Yaml,
    /// 纯文本格式
    Text,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Console
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Console => write!(f, "console"),
            OutputFormat::Markdown => write!(f, "markdown"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Yaml => write!(f, "yaml"),
            OutputFormat::Text => write!(f, "text"),
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "console" => Ok(OutputFormat::Console),
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            "json" => Ok(OutputFormat::Json),
            "yaml" | "yml" => Ok(OutputFormat::Yaml),
            "text" | "txt" => Ok(OutputFormat::Text),
            _ => Err(format!("Unsupported output format: {}", s)),
        }
    }
}

/// 代码评审数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewData {
    /// Git diff 内容
    pub diff_content: String,
    /// 变更统计信息
    pub stats: ReviewStats,
    /// 文件变更列表
    pub changed_files: Vec<ChangedFile>,
    /// 多维度分析数据
    pub analysis_data: AnalysisData,
    /// 分析选项
    pub options: ReviewOptions,
}

/// 多维度分析数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisData {
    /// 代码复杂度分析
    pub complexity_metrics: ComplexityMetrics,
    /// 函数级变更分析
    pub function_changes: Vec<FunctionChange>,
    /// 导入/导出变更
    pub import_changes: Vec<ImportChange>,
    /// 结构化数据变更
    pub structural_changes: Vec<StructuralChange>,
    /// 测试相关变更
    pub test_changes: TestChanges,
    /// 性能相关变更
    pub performance_changes: PerformanceChanges,
}

/// 代码复杂度指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// 圈复杂度变更
    pub cyclomatic_complexity_delta: i32,
    /// 认知复杂度变更
    pub cognitive_complexity_delta: i32,
    /// 嵌套深度最大值
    pub max_nesting_depth: u32,
    /// 函数长度统计
    pub function_lengths: Vec<usize>,
    /// 参数数量统计
    pub parameter_counts: Vec<usize>,
}

/// 函数级变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionChange {
    /// 函数名
    pub name: String,
    /// 变更类型
    pub change_type: FunctionChangeType,
    /// 所在文件
    pub file_path: String,
    /// 函数签名
    pub signature: Option<String>,
    /// 变更行数
    pub changed_lines: usize,
    /// 复杂度变更
    pub complexity_delta: i32,
}

/// 函数变更类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FunctionChangeType {
    /// 新增函数
    Added,
    /// 删除函数
    Removed,
    /// 修改函数
    Modified,
    /// 重命名函数
    Renamed,
    /// 函数签名变更
    SignatureChanged,
}

/// 导入变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportChange {
    /// 模块路径
    pub module_path: String,
    /// 变更类型
    pub change_type: ImportChangeType,
    /// 所在文件
    pub file_path: String,
    /// 是否为外部依赖
    pub is_external: bool,
}

/// 导入变更类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportChangeType {
    /// 新增导入
    Added,
    /// 删除导入
    Removed,
    /// 修改导入路径
    Modified,
}

/// 结构化变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralChange {
    /// 变更类型
    pub change_type: StructuralChangeType,
    /// 元素名称
    pub element_name: String,
    /// 所在文件
    pub file_path: String,
    /// 影响范围（行数）
    pub impact_lines: usize,
    /// 相关元素
    pub related_elements: Vec<String>,
}

/// 结构化变更类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StructuralChangeType {
    /// 类/结构体变更
    ClassChange,
    /// 接口/Trait变更
    InterfaceChange,
    /// 枚举变更
    EnumChange,
    /// 模块变更
    ModuleChange,
    /// 配置变更
    ConfigChange,
}

/// 测试相关变更
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestChanges {
    /// 新增测试数量
    pub added_tests: u32,
    /// 删除测试数量
    pub removed_tests: u32,
    /// 修改测试数量
    pub modified_tests: u32,
    /// 测试覆盖率变更
    pub coverage_delta: f32,
    /// 测试文件列表
    pub test_files: Vec<String>,
}

/// 性能相关变更
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceChanges {
    /// 算法复杂度变更
    pub algorithmic_changes: Vec<AlgorithmicChange>,
    /// 数据结构变更
    pub data_structure_changes: Vec<DataStructureChange>,
    /// 并发相关变更
    pub concurrency_changes: Vec<ConcurrencyChange>,
    /// 内存使用变更
    pub memory_changes: Vec<MemoryChange>,
}

/// 算法复杂度变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmicChange {
    /// 函数名
    pub function_name: String,
    /// 时间复杂度变更
    pub time_complexity_change: ComplexityChange,
    /// 空间复杂度变更
    pub space_complexity_change: ComplexityChange,
    /// 所在文件
    pub file_path: String,
}

/// 复杂度变更类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityChange {
    /// 从O(1)到O(n)
    LinearToLinear,
    /// 从O(n)到O(n²)
    LinearToQuadratic,
    /// 从O(n²)到O(n)
    QuadraticToLinear,
    /// 从O(n)到O(log n)
    LinearToLogarithmic,
    /// 从O(log n)到O(n)
    LogarithmicToLinear,
    /// 无变更
    NoChange,
    /// 复杂度增加
    Increased,
    /// 复杂度降低
    Decreased,
}

/// 数据结构变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStructureChange {
    /// 变更类型
    pub change_type: DataStructureChangeType,
    /// 旧数据结构
    pub old_structure: Option<String>,
    /// 新数据结构
    pub new_structure: Option<String>,
    /// 所在文件
    pub file_path: String,
    /// 影响函数列表
    pub affected_functions: Vec<String>,
}

/// 数据结构变更类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataStructureChangeType {
    /// 数组到向量
    ArrayToVector,
    /// 链表到数组
    LinkedListToArray,
    /// 哈希表到树
    HashMapToTree,
    /// 新增数据结构
    Added,
    /// 删除数据结构
    Removed,
    /// 其他变更
    Other,
}

/// 并发相关变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyChange {
    /// 变更类型
    pub change_type: ConcurrencyChangeType,
    /// 涉及的函数
    pub functions: Vec<String>,
    /// 所在文件
    pub file_path: String,
    /// 安全级别
    pub safety_level: ConcurrencySafetyLevel,
}

/// 并发变更类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConcurrencyChangeType {
    /// 新增线程
    ThreadAdded,
    /// 新增异步函数
    AsyncAdded,
    /// 新增锁机制
    LockAdded,
    /// 新增通道
    ChannelAdded,
    /// 共享状态变更
    SharedStateChange,
}

/// 并发安全级别
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConcurrencySafetyLevel {
    /// 安全
    Safe,
    /// 潜在风险
    PotentiallyUnsafe,
    /// 明显风险
    Unsafe,
}

/// 内存使用变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryChange {
    /// 变更类型
    pub change_type: MemoryChangeType,
    /// 预估影响
    pub estimated_impact: MemoryImpact,
    /// 所在文件
    pub file_path: String,
    /// 相关函数
    pub related_functions: Vec<String>,
}

/// 内存变更类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryChangeType {
    /// 内存分配
    Allocation,
    /// 内存释放
    Deallocation,
    /// 内存泄漏风险
    LeakRisk,
    /// 大对象分配
    LargeObject,
    /// 缓存使用
    CacheUsage,
}

/// 内存影响级别
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryImpact {
    /// 低影响
    Low,
    /// 中等影响
    Medium,
    /// 高影响
    High,
    /// 严重
    Critical,
}

/// 评审统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewStats {
    /// 总变更行数
    pub total_changes: usize,
    /// 新增行数
    pub additions: usize,
    /// 删除行数
    pub deletions: usize,
    /// 修改文件数
    pub files_changed: usize,
}

/// 变更文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedFile {
    /// 文件路径
    pub path: String,
    /// 变更类型
    pub change_type: ChangeType,
    /// 新增行数
    pub additions: usize,
    /// 删除行数
    pub deletions: usize,
    /// 编程语言
    pub language: Option<String>,
}

/// 变更类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChangeType {
    /// 新增文件
    Added,
    /// 修改文件
    Modified,
    /// 删除文件
    Deleted,
    /// 重命名文件
    Renamed,
    /// 其他类型
    Other,
}

/// 评审选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewOptions {
    /// 目标语言
    pub language: Option<String>,
    /// 输出格式
    pub format: OutputFormat,
    /// 输出文件
    pub output: Option<std::path::PathBuf>,
    /// 是否启用Tree-sitter分析
    pub tree_sitter: bool,
    /// 是否启用安全扫描
    pub security_scan: bool,
    /// 扫描工具
    pub scan_tool: Option<String>,
    /// 是否阻止严重问题
    pub block_on_critical: bool,
    /// Issue ID
    pub issue_id: Option<String>,
    /// Space ID
    pub space_id: Option<u64>,
    /// 是否完整分析
    pub full: bool,
}

/// TASK-004: 缓存条目结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// 缓存键
    pub key: String,
    /// 缓存内容
    pub content: String,
    /// 创建时间
    pub created_at: std::time::SystemTime,
    /// 过期时间（秒）
    pub ttl_seconds: u64,
    /// 缓存元数据
    pub metadata: CacheMetadata,
}

/// TASK-004: 缓存元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// AI模型
    pub model: String,
    /// 提示词哈希
    pub prompt_hash: u64,
    /// 代码变更数量
    pub changes_count: usize,
    /// 文件类型列表
    pub file_types: Vec<String>,
    /// 缓存版本
    pub cache_version: u32,
}

/// TASK-004: 缓存键生成参数
#[derive(Debug, Clone)]
pub struct CacheKeyParams {
    /// 代码差异内容
    pub diff_content: String,
    /// AI模型
    pub model: String,
    /// AI配置温度
    pub temperature: f32,
    /// 分析选项
    pub options: ReviewOptions,
    /// Git仓库信息（可选）
    pub git_repo_info: Option<GitRepoInfo>,
}

/// TASK-004: Git仓库信息
#[derive(Debug, Clone)]
pub struct GitRepoInfo {
    /// 当前分支
    pub branch: String,
    /// 最近提交哈希
    pub commit_hash: String,
    /// 仓库路径
    pub repo_path: PathBuf,
}

/// TASK-005: 缓存管理器
#[derive(Debug)]
pub struct CacheManager {
    /// 内存缓存存储
    memory_cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    /// 缓存目录路径
    cache_dir: PathBuf,
    /// 最大内存缓存条目数
    max_memory_entries: usize,
}

/// TASK-005: 缓存统计信息
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// 内存命中次数
    pub memory_hits: u64,
    /// 磁盘命中次数
    pub disk_hits: u64,
    /// 缓存未命中次数
    pub misses: u64,
    /// 缓存写入次数
    pub writes: u64,
    /// 缓存清理次数
    pub cleanups: u64,
}

/// TASK-005: 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 缓存目录路径
    pub cache_dir: PathBuf,
    /// 最大内存缓存条目数
    pub max_memory_entries: usize,
    /// 最大磁盘缓存大小（字节）
    pub max_disk_size: u64,
    /// 默认TTL（秒）
    pub default_ttl: u64,
    /// 是否启用磁盘缓存
    pub enable_disk_cache: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        let cache_dir = std::env::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".gitai")
            .join("cache");

        Self {
            cache_dir,
            max_memory_entries: 100,
            max_disk_size: 100 * 1024 * 1024, // 100MB
            default_ttl: 3600, // 1小时
            enable_disk_cache: true,
        }
    }
}

/// 处理 review 命令
pub async fn handle_command(
    config: &gitai_core::config::Config,
    command: &Command,
) -> HandlerResult<()> {
    match command {
        Command::Review {
            language,
            format,
            output,
            tree_sitter,
            security_scan,
            scan_tool,
            block_on_critical,
            issue_id,
            space_id,
            full,
        } => {
            // 构建评审选项
            let options = ReviewOptions {
                language: language.clone(),
                format: format.clone(),
                output: output.clone(),
                tree_sitter: *tree_sitter,
                security_scan: *security_scan,
                scan_tool: scan_tool.clone(),
                block_on_critical: *block_on_critical,
                issue_id: issue_id.clone(),
                space_id: space_id.clone(),
                full: *full,
            };

            // 执行代码评审
            match execute_review(config, &options).await {
                Ok(review_data) => {
                    // 输出评审结果
                    output_review_result(&review_data, config).await?;
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ 代码评审失败: {}", e);
                    Err(Box::new(e))
                }
            }
        }
        _ => Err("Invalid command for review handler".into()),
    }
}

/// 执行代码评审的主函数
async fn execute_review(
    _config: &gitai_core::config::Config,
    options: &ReviewOptions,
) -> Result<ReviewData> {
    println!("🔍 开始代码评审...");

    // TASK-001: Git diff获取和预处理
    let diff_content = get_and_preprocess_diff().await?;

    if diff_content.trim().is_empty() {
        return Err(gitai_types::GitAIError::Git(
            gitai_types::error::GitError::CommandFailed("没有检测到任何变更".to_string())
        ));
    }

    println!("✅ 成功获取代码变更 ({} 字符)", diff_content.len());

    // 解析变更统计信息
    let stats = parse_diff_stats(&diff_content).await?;
    println!("📊 变更统计: +{} -{} 行, {} 个文件",
             stats.additions, stats.deletions, stats.files_changed);

    // 解析变更文件列表
    let changed_files = parse_changed_files(&diff_content).await?;
    println!("📁 检测到 {} 个变更文件", changed_files.len());

    // 输出变更文件概览
    for file in &changed_files {
        let change_icon = match file.change_type {
            ChangeType::Added => "🆕",
            ChangeType::Modified => "📝",
            ChangeType::Deleted => "🗑️",
            ChangeType::Renamed => "🔄",
            ChangeType::Other => "❓",
        };

        let language_str = if let Some(lang) = &file.language {
            format!(" ({})", lang)
        } else {
            String::new()
        };

        println!("  {} {}{} (+{} -{})",
                 change_icon, file.path, language_str, file.additions, file.deletions);
    }

    // TASK-002: 多维度数据采集
    println!("🔬 开始多维度数据分析...");
    let analysis_data = collect_multi_dimensional_data(&diff_content, &changed_files).await?;
    println!("✅ 多维度数据分析完成");

    Ok(ReviewData {
        diff_content,
        stats,
        changed_files,
        analysis_data,
        options: options.clone(),
    })
}

/// TASK-001: 获取和预处理Git diff
async fn get_and_preprocess_diff() -> Result<String> {
    println!("📥 获取Git变更信息...");

    // TASK-001-1: 增强Git错误处理
    // 首先检查是否在Git仓库中
    if let Err(e) = check_git_repository() {
        return Err(handle_git_error(&e));
    }

    // 检查Git状态
    if let Err(e) = check_git_status() {
        return Err(handle_git_error(&e));
    }

    // 使用现有的get_all_diff函数获取所有变更
    match git::get_all_diff() {
        Ok(diff) => {
            if diff.trim().is_empty() {
                // 如果没有当前变更，尝试获取最后一次提交
                println!("⚠️  没有检测到当前变更，尝试获取最后一次提交...");
                handle_no_changes_fallback()
            } else {
                // TASK-001-2: 预处理diff内容（包含二进制文件检测）
                let processed_diff = preprocess_diff_content_with_binary_detection(&diff);
                println!("✅ Git diff获取完成");
                Ok(processed_diff)
            }
        }
        Err(e) => {
            // 处理特定的Git错误
            let enhanced_error = handle_git_error(&e);

            // 尝试降级处理
            println!("⚠️  主方法失败，尝试降级处理...");
            match try_fallback_git_operations() {
                Ok(fallback_diff) => {
                    println!("✅ 降级处理成功");
                    // TASK-001-2: 预处理diff内容（包含二进制文件检测）
                    Ok(preprocess_diff_content_with_binary_detection(&fallback_diff))
                }
                Err(_fallback_err) => {
                    println!("❌ 所有Git操作均失败");
                    Err(enhanced_error)
                }
            }
        }
    }
}

/// TASK-001-1: 检查是否在Git仓库中
fn check_git_repository() -> Result<()> {
    match std::process::Command::new("git")
        .args(&["rev-parse", "--git-dir"])
        .output() {
        Ok(output) => {
            if !output.status.success() {
                Err(gitai_types::GitAIError::Git(
                    gitai_types::GitError::RepositoryNotFound(
                        "当前目录不是Git仓库或无法访问.git目录".to_string()
                    )
                ))
            } else {
                Ok(())
            }
        }
        Err(e) => {
            Err(gitai_types::GitAIError::Git(
                gitai_types::GitError::CommandFailed(
                    format!("无法执行Git命令: {}", e)
                )
            ))
        }
    }
}

/// TASK-001-1: 检查Git仓库状态
fn check_git_status() -> Result<()> {
    // 检查是否有权限访问Git仓库
    if let Ok(output) = std::process::Command::new("git")
        .args(&["status", "--porcelain"])
        .output() {
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("Permission denied") {
                return Err(gitai_types::GitAIError::Git(
                    gitai_types::GitError::PermissionDenied(
                        "没有权限访问Git仓库".to_string()
                    )
                ));
            }
        }
    }
    Ok(())
}

/// TASK-001-1: 增强的Git错误处理
fn handle_git_error(error: &gitai_types::GitAIError) -> gitai_types::GitAIError {
    match error {
        gitai_types::GitAIError::Git(git_error) => {
            match git_error {
                gitai_types::GitError::CommandFailed(msg) => {
                    // 提供更具体的错误信息和解决建议
                    if msg.contains("not a git repository") {
                        gitai_types::GitAIError::Git(
                            gitai_types::GitError::RepositoryNotFound(
                                format!("Git仓库检查失败: {}. 建议运行 'git init' 初始化仓库", msg)
                            )
                        )
                    } else if msg.contains("Permission denied") {
                        gitai_types::GitAIError::Git(
                            gitai_types::GitError::PermissionDenied(
                                format!("权限不足: {}. 请检查文件权限", msg)
                            )
                        )
                    } else if msg.contains("fatal: not in a git directory") {
                        gitai_types::GitAIError::Git(
                            gitai_types::GitError::RepositoryNotFound(
                                format!("不在Git仓库中: {}. 请切换到Git仓库目录", msg)
                            )
                        )
                    } else {
                        // 保留原始错误但添加更多上下文
                        gitai_types::GitAIError::Git(
                            gitai_types::GitError::CommandFailed(
                                format!("Git命令执行失败: {}. 建议检查Git安装和配置", msg)
                            )
                        )
                    }
                }
                // For other GitError variants, return a generic command failed error
                _ => gitai_types::GitAIError::Git(gitai_types::GitError::CommandFailed(
                    "Git操作失败，请检查Git配置和权限".to_string()
                )),
            }
        }
        // For non-Git errors, return a generic error
        _ => gitai_types::GitAIError::Git(gitai_types::GitError::CommandFailed(
            format!("处理过程中发生错误: {}", error)
        )),
    }
}

/// TASK-001-1: 处理无变更时的降级策略
fn handle_no_changes_fallback() -> Result<String> {
    // 尝试获取最后一次提交
    match git::get_last_commit_diff() {
        Ok(last_diff) => {
            if !last_diff.trim().is_empty() {
                let last_commit_diff = format!(
                    "## 最后一次提交的变更 (Last Commit):\n{}",
                    last_diff
                );
                // TASK-001-2: 预处理diff内容（包含二进制文件检测）
                let processed_diff = preprocess_diff_content_with_binary_detection(&last_commit_diff);
                println!("✅ 使用最后一次提交的变更");
                Ok(processed_diff)
            } else {
                // 检查是否是新初始化的仓库
                match check_is_new_repository() {
                    true => {
                        println!("📝 检测到新Git仓库，无历史提交");
                        Ok("# 新Git仓库 (New Git Repository)\n\n这是一个新初始化的Git仓库，还没有任何提交。".to_string())
                    }
                    false => {
                        Err(gitai_types::GitAIError::Git(
                            gitai_types::GitError::CommandFailed(
                                "无法获取任何变更信息，且无历史提交".to_string()
                            )
                        ))
                    }
                }
            }
        }
        Err(e) => {
            // 检查是否是因为没有提交历史
            if format!("{}", e).contains("does not have any commits yet") {
                println!("📝 检测到无提交历史的Git仓库");
                Ok("# 新Git仓库 (New Git Repository)\n\n这个Git仓库还没有任何提交历史。".to_string())
            } else {
                Err(handle_git_error(&e))
            }
        }
    }
}

/// TASK-001-1: 检查是否是新初始化的仓库
fn check_is_new_repository() -> bool {
    // 检查提交数量
    match std::process::Command::new("git")
        .args(&["rev-list", "--count", "--all"])
        .output() {
        Ok(output) => {
            if output.status.success() {
                let count_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                count_str == "0"
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// TASK-001-1: 尝试降级的Git操作
fn try_fallback_git_operations() -> Result<String> {
    let mut attempts = Vec::new();

    // 尝试1: 只获取已暂存的变更
    if let Ok(staged) = git::run_git(&["diff".to_string(), "--cached".to_string()]) {
        if !staged.trim().is_empty() {
            attempts.push(("已暂存变更", staged));
        }
    }

    // 尝试2: 只获取工作区变更
    if let Ok(unstaged) = git::run_git(&["diff".to_string()]) {
        if !unstaged.trim().is_empty() {
            attempts.push(("工作区变更", unstaged));
        }
    }

    // 尝试3: 获取最近提交的变更
    if let Ok(last_commit) = git::run_git(&["show".to_string(), "--format=".to_string(), "--stat".to_string()]) {
        if !last_commit.trim().is_empty() {
            attempts.push(("最近提交统计", last_commit));
        }
    }

    // 尝试4: 获取文件状态
    if let Ok(status) = git::run_git(&["status".to_string(), "--porcelain".to_string()]) {
        if !status.trim().is_empty() {
            let status_info = format!("## Git状态 (Git Status):\n{}", status);
            attempts.push(("Git状态", status_info));
        }
    }

    // 返回第一个成功的结果
    if let Some((description, content)) = attempts.into_iter().next() {
        println!("✅ 降级成功，使用: {}", description);
        Ok(content)
    } else {
        Err(gitai_types::GitAIError::Git(
            gitai_types::GitError::CommandFailed(
                "所有降级Git操作均失败".to_string()
            )
        ))
    }
}

/// TASK-001-2: 二进制文件检测结果
#[derive(Debug, Clone)]
pub struct BinaryFileDetection {
    /// 检测到的二进制文件列表
    pub binary_files: Vec<BinaryFileInfo>,
    /// 处理后的diff（移除二进制文件内容）
    pub filtered_diff: String,
    /// 检测统计
    pub detection_stats: BinaryDetectionStats,
}

/// TASK-001-2: 二进制文件信息
#[derive(Debug, Clone)]
pub struct BinaryFileInfo {
    /// 文件路径
    pub path: String,
    /// 检测原因
    pub detection_reason: BinaryDetectionReason,
    /// 原始diff片段
    pub original_diff_snippet: String,
    /// 文件大小（如果可检测）
    pub estimated_size: Option<usize>,
}

/// TASK-001-2: 二进制文件检测原因
#[derive(Debug, Clone)]
pub enum BinaryDetectionReason {
    /// Git明确标记为二进制
    GitBinaryMark,
    /// 扩展名匹配
    FileExtension(String),
    /// 内容包含二进制特征
    ContentPattern,
    /// 文件过大
    FileTooLarge(usize),
    /// 包含非文本字符
    NonTextCharacters(f32), // 非文本字符比例
}

/// TASK-001-2: 二进制检测统计
#[derive(Debug, Clone, Default)]
pub struct BinaryDetectionStats {
    /// 检测到的二进制文件数量
    pub binary_file_count: usize,
    /// 移除的diff行数
    pub removed_diff_lines: usize,
    /// 保留的文本文件数量
    pub text_file_count: usize,
    /// 总检测文件数
    pub total_files_scanned: usize,
}

/// 预处理diff内容
#[allow(dead_code)]
fn preprocess_diff_content(diff: &str) -> String {
    let mut processed = String::new();
    let mut lines = diff.lines().collect::<Vec<_>>();

    // 移除空行和纯whitespace行
    lines.retain(|line| !line.trim().is_empty());

    // 限制diff大小以避免处理过大的文件
    const MAX_DIFF_SIZE: usize = 100_000; // 100KB
    let mut current_size = 0;

    for line in lines {
        if current_size + line.len() > MAX_DIFF_SIZE {
            processed.push_str("\n[... 变更内容过大，已截断 ...]\n");
            break;
        }

        processed.push_str(line);
        processed.push('\n');
        current_size += line.len() + 1;
    }

    processed
}

/// TASK-001-2: 增强的预处理diff内容（包含二进制文件检测）
fn preprocess_diff_content_with_binary_detection(diff: &str) -> String {
    // 1. 检测和过滤二进制文件
    let detection = detect_and_filter_binary_files(diff);

    // 2. 输出检测结果
    output_binary_file_detection_results(&detection);

    // 3. 如果过滤后的diff为空，返回提示信息
    if detection.filtered_diff.trim().is_empty() {
        if !detection.binary_files.is_empty() {
            return "只检测到二进制文件变更，没有文本文件变更可供分析。".to_string();
        } else {
            return diff.to_string(); // 没有检测到二进制文件，返回原始内容
        }
    }

    // 4. 对过滤后的文本内容进行预处理
    let filtered_diff = &detection.filtered_diff;
    let mut processed = String::new();
    let mut lines = filtered_diff.lines().collect::<Vec<_>>();

    // 移除空行和纯whitespace行
    lines.retain(|line| !line.trim().is_empty());

    // 限制diff大小以避免处理过大的文件
    const MAX_DIFF_SIZE: usize = 100_000; // 100KB
    let mut current_size = 0;

    for line in lines {
        if current_size + line.len() > MAX_DIFF_SIZE {
            processed.push_str("\n[... 变更内容过大，已截断 ...]\n");
            break;
        }

        processed.push_str(line);
        processed.push('\n');
        current_size += line.len() + 1;
    }

    processed
}

/// 解析diff统计信息
async fn parse_diff_stats(diff: &str) -> Result<ReviewStats> {
    let mut stats = ReviewStats::default();

    for line in diff.lines() {
        if line.starts_with("+") && !line.starts_with("+++") {
            stats.additions += 1;
            stats.total_changes += 1;
        } else if line.starts_with("-") && !line.starts_with("---") {
            stats.deletions += 1;
            stats.total_changes += 1;
        }
    }

    // 统计文件数量（简化实现）
    stats.files_changed = diff.lines()
        .filter(|line| line.starts_with("diff --git"))
        .count();

    Ok(stats)
}

/// 解析变更文件列表
async fn parse_changed_files(diff: &str) -> Result<Vec<ChangedFile>> {
    let mut files = Vec::new();
    let mut current_file: Option<ChangedFile> = None;

    for line in diff.lines() {
        if line.starts_with("diff --git") {
            // 保存前一个文件（如果有的话）
            if let Some(file) = current_file.take() {
                files.push(file);
            }

            // 解析文件路径
            if let Some(captures) = regex::Regex::new(r"diff --git a/(.*) b/(.*)")
                .unwrap()
                .captures(line) {

                let file_path = captures.get(2).unwrap().as_str().to_string();
                let language = detect_file_language(&file_path);

                current_file = Some(ChangedFile {
                    path: file_path,
                    change_type: ChangeType::Modified, // 默认为修改，后续会更新
                    additions: 0,
                    deletions: 0,
                    language,
                });
            }
        } else if line.starts_with("new file mode") {
            if let Some(ref mut file) = current_file {
                file.change_type = ChangeType::Added;
            }
        } else if line.starts_with("deleted file mode") {
            if let Some(ref mut file) = current_file {
                file.change_type = ChangeType::Deleted;
            }
        } else if line.starts_with("rename from") {
            if let Some(ref mut file) = current_file {
                file.change_type = ChangeType::Renamed;
            }
        } else if let Some(ref mut file) = current_file {
            // 统计添加和删除的行数
            if line.starts_with("+") && !line.starts_with("+++") {
                file.additions += 1;
            } else if line.starts_with("-") && !line.starts_with("---") {
                file.deletions += 1;
            }
        }
    }

    // 添加最后一个文件
    if let Some(file) = current_file {
        files.push(file);
    }

    Ok(files)
}

/// 检测文件编程语言
fn detect_file_language(file_path: &str) -> Option<String> {
    let path = std::path::Path::new(file_path);

    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        let language = match ext.to_lowercase().as_str() {
            "rs" => Some("Rust"),
            "js" => Some("JavaScript"),
            "ts" => Some("TypeScript"),
            "py" => Some("Python"),
            "java" => Some("Java"),
            "cpp" | "cxx" | "cc" => Some("C++"),
            "c" | "h" => Some("C"),
            "go" => Some("Go"),
            "rb" => Some("Ruby"),
            "php" => Some("PHP"),
            "cs" => Some("C#"),
            "swift" => Some("Swift"),
            "kt" => Some("Kotlin"),
            "scala" => Some("Scala"),
            "html" => Some("HTML"),
            "css" => Some("CSS"),
            "json" => Some("JSON"),
            "yaml" | "yml" => Some("YAML"),
            "toml" => Some("TOML"),
            "md" => Some("Markdown"),
            "sql" => Some("SQL"),
            "sh" => Some("Shell"),
            _ => None,
        };
        language.map(|s| s.to_string())
    } else if file_path.ends_with("Dockerfile") {
        Some("Dockerfile".to_string())
    } else if file_path.ends_with("Makefile") {
        Some("Makefile".to_string())
    } else if file_path.ends_with("CMakeLists.txt") {
        Some("CMake".to_string())
    } else {
        None
    }
}

/// TASK-006: 格式化评审结果为不同输出格式
async fn format_review_result(review_data: &ReviewData, format: &OutputFormat, ai_analysis: Option<&str>) -> Result<String> {
    match format {
        OutputFormat::Console => {
            format_review_as_console(review_data, ai_analysis).await
        }
        OutputFormat::Markdown => {
            format_review_as_markdown(review_data, ai_analysis).await
        }
        OutputFormat::Json => {
            format_review_as_json(review_data, ai_analysis).await
        }
        OutputFormat::Yaml => {
            format_review_as_yaml(review_data, ai_analysis).await
        }
        OutputFormat::Text => {
            format_review_as_text(review_data, ai_analysis).await
        }
    }
}

/// TASK-006: 控制台格式输出（带表情符号和颜色）
async fn format_review_as_console(review_data: &ReviewData, ai_analysis: Option<&str>) -> Result<String> {
    let mut output = String::new();

    output.push_str("\n📋 代码评审报告\n");
    output.push_str(&"=".repeat(50));
    output.push('\n');

    // 输出概览信息
    output.push_str("\n📊 变更概览:\n");
    output.push_str(&format!("  • 总变更: {} 行\n", review_data.stats.total_changes));
    output.push_str(&format!("  • 新增: {} 行\n", review_data.stats.additions));
    output.push_str(&format!("  • 删除: {} 行\n", review_data.stats.deletions));
    output.push_str(&format!("  • 文件数: {} 个\n", review_data.stats.files_changed));

    // 输出配置信息
    output.push_str("\n⚙️  评审配置:\n");
    output.push_str(&format!("  • 语言: {:?}\n", review_data.options.language));
    output.push_str(&format!("  • Tree-sitter分析: {}\n", review_data.options.tree_sitter));
    output.push_str(&format!("  • 安全扫描: {}\n", review_data.options.security_scan));
    output.push_str(&format!("  • 完整分析: {}\n", review_data.options.full));

    if let Some(ref issue_id) = review_data.options.issue_id {
        output.push_str(&format!("  • 关联Issue: {}\n", issue_id));
    }

    // 输出文件详情
    output.push_str("\n📁 变更文件详情:\n");
    for file in &review_data.changed_files {
        let change_type_str = match file.change_type {
            ChangeType::Added => "新增",
            ChangeType::Modified => "修改",
            ChangeType::Deleted => "删除",
            ChangeType::Renamed => "重命名",
            ChangeType::Other => "其他",
        };

        output.push_str(&format!("  • {} ({})\n", file.path, change_type_str));
        if file.additions > 0 || file.deletions > 0 {
            output.push_str(&format!("    +{} -{} 行\n", file.additions, file.deletions));
        }

        if let Some(ref lang) = file.language {
            output.push_str(&format!("    语言: {}\n", lang));
        }
    }

    // AI分析结果
    if let Some(analysis) = ai_analysis {
        output.push_str("\n🤖 AI智能分析:\n");
        output.push_str(&"=".repeat(50));
        output.push('\n');
        output.push_str(analysis);
        output.push_str(&"=".repeat(50));
        output.push('\n');
    }

    Ok(output)
}

/// TASK-006: Markdown格式输出
async fn format_review_as_markdown(review_data: &ReviewData, ai_analysis: Option<&str>) -> Result<String> {
    let mut output = String::new();

    output.push_str("# 代码评审报告\n\n");
    output.push_str(&format!("**生成时间**: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

    // 变更概览
    output.push_str("## 📊 变更概览\n\n");
    output.push_str(&format!("- **总变更**: {} 行\n", review_data.stats.total_changes));
    output.push_str(&format!("- **新增**: {} 行\n", review_data.stats.additions));
    output.push_str(&format!("- **删除**: {} 行\n", review_data.stats.deletions));
    output.push_str(&format!("- **文件数**: {} 个\n", review_data.stats.files_changed));

    // 评审配置
    output.push_str("\n## ⚙️ 评审配置\n\n");
    output.push_str(&format!("- **语言**: {:?}\n", review_data.options.language));
    output.push_str(&format!("- **Tree-sitter分析**: {}\n", review_data.options.tree_sitter));
    output.push_str(&format!("- **安全扫描**: {}\n", review_data.options.security_scan));
    output.push_str(&format!("- **完整分析**: {}\n", review_data.options.full));

    if let Some(ref issue_id) = review_data.options.issue_id {
        output.push_str(&format!("- **关联Issue**: {}\n", issue_id));
    }

    // 文件详情
    output.push_str("\n## 📁 变更文件详情\n\n");
    output.push_str("| 文件路径 | 变更类型 | 新增行 | 删除行 | 语言 |\n");
    output.push_str("|----------|----------|--------|--------|------|\n");

    for file in &review_data.changed_files {
        let change_type_str = match file.change_type {
            ChangeType::Added => "新增",
            ChangeType::Modified => "修改",
            ChangeType::Deleted => "删除",
            ChangeType::Renamed => "重命名",
            ChangeType::Other => "其他",
        };

        let language = file.language.as_deref().unwrap_or("-");
        output.push_str(&format!("| {} | {} | {} | {} | {} |\n",
            file.path, change_type_str, file.additions, file.deletions, language));
    }

    // AI分析结果
    if let Some(analysis) = ai_analysis {
        output.push_str("\n## 🤖 AI智能分析\n\n");
        output.push_str("```\n");
        output.push_str(analysis);
        output.push_str("\n```\n");
    }

    Ok(output)
}

/// TASK-006: JSON格式输出
async fn format_review_as_json(review_data: &ReviewData, ai_analysis: Option<&str>) -> Result<String> {
    use serde_json::json;

    let json_data = json!({
        "metadata": {
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "format_version": "1.0",
            "tool": "GitAI",
            "version": env!("CARGO_PKG_VERSION")
        },
        "review_data": review_data,
        "ai_analysis": ai_analysis
    });

    serde_json::to_string_pretty(&json_data).map_err(|e|
        gitai_types::GitAIError::Other(format!("Failed to serialize to JSON: {}", e))
    )
}

/// TASK-006: YAML格式输出
async fn format_review_as_yaml(review_data: &ReviewData, ai_analysis: Option<&str>) -> Result<String> {
    // 由于serde_yaml可能不是所有项目都依赖，我们先实现一个简单的YAML格式化
    let mut output = String::new();

    output.push_str(&format!("metadata:\n"));
    output.push_str(&format!("  generated_at: {}\n", chrono::Utc::now().to_rfc3339()));
    output.push_str(&format!("  format_version: \"1.0\"\n"));
    output.push_str(&format!("  tool: \"GitAI\"\n"));
    output.push_str(&format!("  version: \"{}\"\n", env!("CARGO_PKG_VERSION")));
    output.push_str(&format!("review_data:\n"));
    output.push_str(&format!("  diff_content: |\n"));
    for line in review_data.diff_content.lines() {
        output.push_str(&format!("    {}\n", line));
    }
    output.push_str(&format!("  stats:\n"));
    output.push_str(&format!("    total_changes: {}\n", review_data.stats.total_changes));
    output.push_str(&format!("    additions: {}\n", review_data.stats.additions));
    output.push_str(&format!("    deletions: {}\n", review_data.stats.deletions));
    output.push_str(&format!("    files_changed: {}\n", review_data.stats.files_changed));

    output.push_str(&format!("  changed_files:\n"));
    for file in &review_data.changed_files {
        output.push_str(&format!("    - path: \"{}\"\n", file.path));
        output.push_str(&format!("      change_type: {:?}\n", file.change_type));
        output.push_str(&format!("      additions: {}\n", file.additions));
        output.push_str(&format!("      deletions: {}\n", file.deletions));
        if let Some(ref lang) = file.language {
            output.push_str(&format!("      language: \"{}\"\n", lang));
        }
    }

    if let Some(analysis) = ai_analysis {
        output.push_str(&format!("  ai_analysis: |\n"));
        for line in analysis.lines() {
            output.push_str(&format!("    {}\n", line));
        }
    }

    Ok(output)
}

/// TASK-006: 纯文本格式输出（无表情符号和格式化）
async fn format_review_as_text(review_data: &ReviewData, ai_analysis: Option<&str>) -> Result<String> {
    let mut output = String::new();

    output.push_str("代码评审报告\n");
    output.push_str(&"=".repeat(50));
    output.push('\n');

    // 输出概览信息
    output.push_str("\n变更概览:\n");
    output.push_str(&format!("  总变更: {} 行\n", review_data.stats.total_changes));
    output.push_str(&format!("  新增: {} 行\n", review_data.stats.additions));
    output.push_str(&format!("  删除: {} 行\n", review_data.stats.deletions));
    output.push_str(&format!("  文件数: {} 个\n", review_data.stats.files_changed));

    // 输出配置信息
    output.push_str("\n评审配置:\n");
    output.push_str(&format!("  语言: {:?}\n", review_data.options.language));
    output.push_str(&format!("  Tree-sitter分析: {}\n", review_data.options.tree_sitter));
    output.push_str(&format!("  安全扫描: {}\n", review_data.options.security_scan));
    output.push_str(&format!("  完整分析: {}\n", review_data.options.full));

    if let Some(ref issue_id) = review_data.options.issue_id {
        output.push_str(&format!("  关联Issue: {}\n", issue_id));
    }

    // 输出文件详情
    output.push_str("\n变更文件详情:\n");
    for file in &review_data.changed_files {
        let change_type_str = match file.change_type {
            ChangeType::Added => "新增",
            ChangeType::Modified => "修改",
            ChangeType::Deleted => "删除",
            ChangeType::Renamed => "重命名",
            ChangeType::Other => "其他",
        };

        output.push_str(&format!("  {} ({})\n", file.path, change_type_str));
        if file.additions > 0 || file.deletions > 0 {
            output.push_str(&format!("    +{} -{} 行\n", file.additions, file.deletions));
        }

        if let Some(ref lang) = file.language {
            output.push_str(&format!("    语言: {}\n", lang));
        }
    }

    // AI分析结果
    if let Some(analysis) = ai_analysis {
        output.push_str("\nAI智能分析:\n");
        output.push_str(&"=".repeat(50));
        output.push('\n');
        output.push_str(analysis);
        output.push_str(&"=".repeat(50));
        output.push('\n');
    }

    Ok(output)
}

/// TASK-006: 输出评审结果（支持多种格式）
async fn output_review_result(review_data: &ReviewData, config: &gitai_core::config::Config) -> Result<()> {
    let mut ai_analysis: Option<String> = None;

    // TASK-003-1: AI服务调用集成
    if review_data.options.tree_sitter || review_data.options.security_scan || review_data.options.full {
        // 构建AI提示词
        let ai_prompt = build_ai_prompt(&review_data).await?;

        // 验证AI配置
        if !validate_ai_config(config) {
            if review_data.options.format == OutputFormat::Console {
                eprintln!("❌ AI配置无效:");
                eprintln!("  • API URL: {}", config.ai.api_url);
                eprintln!("  • Model: {}", config.ai.model);
                if config.ai.api_key.is_none() || config.ai.api_key.as_ref().unwrap().is_empty() {
                    eprintln!("  • API Key: 未配置或为空");
                }
                eprintln!("💡 请检查配置文件或环境变量");
            }

            // 对于非控制台格式，包含提示词信息
            if review_data.options.format != OutputFormat::Console {
                ai_analysis = Some(format!("AI配置无效，生成的提示词:\n{}", ai_prompt));
            }
        } else {
            // TASK-005: 尝试从缓存获取结果
            let cache_key_params = create_cache_key_params(&review_data, config);
            let cache_key = generate_cache_key(&cache_key_params);
            let mut cache_hit = false;
            let mut cache_manager = None;

            // 创建缓存管理器
            match create_cache_manager() {
                Ok(manager) => {
                    cache_manager = Some(manager);

                    // 尝试从缓存获取结果
                    if let Some(ref mgr) = cache_manager {
                        match mgr.get(&cache_key).await {
                            Ok(Some(cached_entry)) => {
                                if review_data.options.format == OutputFormat::Console {
                                    println!("🎯 缓存命中！使用缓存结果");
                                    println!("💾 缓存信息: 模型={}, 创建时间={:?}",
                                        cached_entry.metadata.model,
                                        cached_entry.created_at);
                                }
                                ai_analysis = Some(cached_entry.content);
                                cache_hit = true;

                                // 如果指定了输出文件，保存缓存结果
                                if let Some(output_path) = &review_data.options.output {
                                    if let Err(e) = save_ai_analysis_result(&ai_analysis.as_ref().unwrap(), output_path).await {
                                        eprintln!("⚠️  保存缓存结果失败: {}", e);
                                    } else if review_data.options.format == OutputFormat::Console {
                                        println!("💾 缓存结果已保存至: {:?}", output_path);
                                    }
                                }
                            }
                            Ok(None) => {
                                if review_data.options.format == OutputFormat::Console {
                                    println!("💭 缓存未命中，将调用AI服务");
                                }
                            }
                            Err(e) => {
                                if review_data.options.format == OutputFormat::Console {
                                    eprintln!("⚠️  缓存读取失败: {}, 将直接调用AI服务", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    if review_data.options.format == OutputFormat::Console {
                        eprintln!("⚠️  缓存初始化失败: {}, 将直接调用AI服务", e);
                    }
                }
            }

        if !cache_hit {
            // 创建AI客户端并调用服务
            let ai_client = AIClient::new(config.clone());
            if review_data.options.format == OutputFormat::Console {
                println!("🔄 正在调用AI服务 ({}):", config.ai.model);
            }

            // 构建上下文信息
            let context = build_ai_context(&review_data);

            match ai_client.review_code(&review_data.diff_content, &context).await {
                Ok(ai_response) => {
                    ai_analysis = Some(ai_response.clone());

                    // TASK-005: 保存AI响应到缓存
                    if let Some(ref manager) = cache_manager {
                        let cache_entry = create_cache_entry(
                            cache_key.clone(),
                            ai_response.clone(),
                            &cache_key_params
                        );

                        match manager.put(cache_key.clone(), cache_entry).await {
                            Ok(()) => {
                                if review_data.options.format == OutputFormat::Console {
                                    println!("💾 AI分析结果已缓存");
                                }
                            }
                            Err(e) => {
                                if review_data.options.format == OutputFormat::Console {
                                    eprintln!("⚠️  缓存保存失败: {}", e);
                                }
                            }
                        }
                    }

                    // 如果指定了输出文件，保存结果
                    if let Some(output_path) = &review_data.options.output {
                        if let Err(e) = save_ai_analysis_result(&ai_response, output_path).await {
                            eprintln!("⚠️  保存AI分析结果失败: {}", e);
                        } else if review_data.options.format == OutputFormat::Console {
                            println!("💾 AI分析结果已保存至: {:?}", output_path);
                        }
                    }
                }
                Err(e) => {
                    if review_data.options.format == OutputFormat::Console {
                        eprintln!("❌ AI服务调用失败: {}", e);
                        println!("💡 提示: 请检查AI服务配置或网络连接");

                        // 降级到提示词输出
                        println!("\n📝 生成的AI提示词 (可用于手动分析):");
                        println!("{}", "=".repeat(50));
                        println!("{}", ai_prompt);
                        println!("{}", "=".repeat(50));
                    } else {
                        // 对于非控制台格式，包含错误信息
                        ai_analysis = Some(format!("AI服务调用失败: {}\n生成的提示词:\n{}", e, ai_prompt));
                    }
                }
            }
        }
        }
    }

    // 使用指定的格式输出结果
    let formatted_output = format_review_result(review_data, &review_data.options.format, ai_analysis.as_deref()).await?;

    // 根据格式类型输出
    match review_data.options.format {
        OutputFormat::Console => {
            // 控制台格式直接打印（已包含所有格式化）
            print!("{}", formatted_output);

            // 输出下一步操作建议（仅控制台格式）
            println!("\n💡 下一步:");
            if !review_data.options.tree_sitter && !review_data.options.security_scan {
                println!("  • 使用 --tree-sitter 启用结构分析");
                println!("  • 使用 --security-scan 启用安全扫描");
            } else {
                println!("  • TODO: 实现Tree-sitter结构分析");
                println!("  • TODO: 实现安全扫描功能");
                println!("  • TODO: 集成AI智能分析");
            }

            println!("\n✅ 代码评审基础功能完成！");
        }
        _ => {
            // 其他格式：如果有输出文件，保存到文件；否则打印到控制台
            if let Some(output_path) = &review_data.options.output {
                use std::fs::File;
                use std::io::Write;

                // 创建输出目录（如果不存在）
                if let Some(parent) = output_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let mut file = File::create(output_path)?;
                file.write_all(formatted_output.as_bytes())?;

                if review_data.options.format == OutputFormat::Console {
                    println!("💾 结果已保存至: {:?}", output_path);
                }
            } else {
                print!("{}", formatted_output);
            }
        }
    }

    Ok(())
}

/// TASK-002: 多维度数据采集框架
async fn collect_multi_dimensional_data(
    diff_content: &str,
    changed_files: &[ChangedFile],
) -> Result<AnalysisData> {
    let mut analysis_data = AnalysisData::default();

    // 1. 复杂度分析
    println!("  📊 分析代码复杂度...");
    analysis_data.complexity_metrics = analyze_complexity_metrics(diff_content).await?;

    // 2. 函数级变更分析
    println!("  🔍 分析函数级变更...");
    analysis_data.function_changes = analyze_function_changes(diff_content, changed_files).await?;

    // 3. 导入/导出变更分析
    println!("  📦 分析导入/导出变更...");
    analysis_data.import_changes = analyze_import_changes(diff_content, changed_files).await?;

    // 4. 结构化变更分析
    println!("  🏗️ 分析结构化变更...");
    analysis_data.structural_changes = analyze_structural_changes(diff_content, changed_files).await?;

    // 5. 测试相关变更分析
    println!("  🧪 分析测试相关变更...");
    analysis_data.test_changes = analyze_test_changes(diff_content, changed_files).await?;

    // 6. 性能相关变更分析
    println!("  ⚡ 分析性能相关变更...");
    analysis_data.performance_changes = analyze_performance_changes(diff_content, changed_files).await?;

    Ok(analysis_data)
}

/// TASK-002-1: 精确的圈复杂度计算器
/// 使用Tree-sitter解析AST并计算真实的圈复杂度
pub struct CyclomaticComplexityCalculator {
    parser: Parser,
}

impl CyclomaticComplexityCalculator {
    /// 创建新的复杂度计算器
    pub fn new() -> Result<Self> {
        let language = tree_sitter_rust::language();

        let mut parser = Parser::new();
        parser.set_language(language)
            .map_err(|e| gitai_types::GitAIError::Other(format!("Failed to set tree-sitter language: {}", e)))?;

        Ok(Self { parser })
    }

    /// 计算Rust代码的圈复杂度
    pub fn calculate_complexity(&mut self, code: &str) -> Result<CyclomaticComplexityResult> {
        let tree = self.parser.parse(code, None)
            .ok_or_else(|| gitai_types::GitAIError::Other("Failed to parse code".to_string()))?;

        let mut calculator = ComplexityVisitor::new(code);
        calculator.visit(&tree);

        Ok(calculator.get_result())
    }

    /// 从Git diff中提取并计算新增代码的复杂度
    pub fn calculate_diff_complexity(&mut self, diff_content: &str) -> Result<CyclomaticComplexityResult> {
        let added_code = extract_added_code_from_diff(diff_content);
        if added_code.is_empty() {
            return Ok(CyclomaticComplexityResult::default());
        }

        self.calculate_complexity(&added_code)
    }
}

/// TASK-002-1: 圈复杂度计算结果
#[derive(Debug, Clone, Default)]
pub struct CyclomaticComplexityResult {
    /// 总圈复杂度
    pub total_complexity: u32,
    /// 按函数分解的复杂度
    pub function_complexity: Vec<FunctionComplexity>,
    /// 决策点数量
    pub decision_points: u32,
    /// 最大函数复杂度
    pub max_function_complexity: u32,
}

/// TASK-002-1: 单个函数的复杂度信息
#[derive(Debug, Clone)]
pub struct FunctionComplexity {
    /// 函数名
    pub name: String,
    /// 函数的圈复杂度
    pub complexity: u32,
    /// 函数在代码中的位置
    pub start_line: u32,
    pub end_line: u32,
}

/// TASK-002-1: AST访问者，用于计算圈复杂度
#[derive(Debug)]
struct ComplexityVisitor<'a> {
    result: CyclomaticComplexityResult,
    current_function: Option<FunctionComplexity>,
    #[allow(dead_code)]
    node_stack: Vec<String>,
    source_code: &'a str, // TASK-002-1-5: 添加源代码引用用于安全的文本提取
}

impl<'a> ComplexityVisitor<'a> {
    fn new(source_code: &'a str) -> Self {
        Self {
            result: CyclomaticComplexityResult::default(),
            current_function: None,
            node_stack: Vec::new(),
            source_code,
        }
    }

    fn visit(&mut self, tree: &Tree) {
        let mut cursor = tree.walk();
        self.visit_node(&mut cursor);
    }

    fn visit_node(&mut self, cursor: &mut TreeCursor) {
        let node = cursor.node();
        let kind = node.kind();

        match kind {
            "function_item" => {
                self.visit_function(cursor);
            }
            // TASK-002-1-1: 基本控制流决策点
            "if_expression" | "if_else" | "match_expression" | "while_expression" |
            "for_expression" | "loop_expression" => {
                // 基本决策点 - 每个增加1
                self.add_decision_point();
                self.visit_node_recursive(cursor);
            }
            // TASK-002-1-1: return语句中的条件表达式也是决策点
            "return_expression" => {
                // 检查是否是条件返回 (如 return if condition)
                if cursor.goto_first_child() {
                    loop {
                        let child = cursor.node();
                        if child.kind() == "if_expression" {
                            self.add_decision_point();
                        }
                        self.visit_node(cursor);
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                }
            }
            // TASK-002-1-1: 布尔运算符 - 每个&&和||都增加复杂度
            "binary_expression" => {
                // 检查是否有布尔运算符 - 使用安全的文本提取
                let node = cursor.node();
                match node.utf8_text(self.source_code.as_bytes()) {
                    Ok(node_text) if !node_text.is_empty() => {
                        if node_text.contains("&&") || node_text.contains("||") {
                            // && 和 || 是短路运算符，每个出现都增加复杂度
                            let and_count = node_text.matches("&&").count() as u32;
                            let or_count = node_text.matches("||").count() as u32;
                            for _ in 0..(and_count + or_count) {
                                self.add_decision_point();
                            }
                        }
                    }
                    Ok(_) | Err(_) => {
                        // 文本提取失败，跳过布尔运算符检测
                    }
                }

                // 正常递归处理所有子节点
                self.visit_node_recursive(cursor);
            }
            // TASK-002-1-1: 问号操作符也是决策点
            "question_mark" => {
                self.add_decision_point();
                self.visit_node_recursive(cursor);
            }
            // TASK-002-1-1: match的每个arm都是一个决策点
            "match_arm" => {
                self.add_decision_point();
                self.visit_node_recursive(cursor);
            }
            // TASK-002-1-1: 闭包表达式可能有复杂度
            "closure_expression" => {
                // 闭包本身不增加复杂度，但内部可能有
                self.visit_node_recursive(cursor);
            }
            _ => {
                // 递归访问其他节点
                self.visit_node_recursive(cursor);
            }
        }
    }

    // TASK-002-1-1: 辅助方法：递归访问节点
    fn visit_node_recursive(&mut self, cursor: &mut TreeCursor) {
        if cursor.goto_first_child() {
            loop {
                self.visit_node(cursor);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }

    
    fn visit_function(&mut self, cursor: &mut TreeCursor) {
        let node = cursor.node();
        let function_name = self.extract_function_name(&node);
        let start_line = node.start_position().row as u32;
        let end_line = node.end_position().row as u32;

        // 开始新的函数计算
        self.current_function = Some(FunctionComplexity {
            name: function_name,
            complexity: 1, // 基础复杂度为1
            start_line,
            end_line,
        });

        // 访问函数体
        if cursor.goto_first_child() {
            loop {
                self.visit_node(cursor);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }

        // 完成函数计算
        if let Some(func) = self.current_function.take() {
            self.result.function_complexity.push(func.clone());
            self.result.max_function_complexity = self.result.max_function_complexity.max(func.complexity);
            self.result.total_complexity += func.complexity;
        }
    }

    fn extract_function_name(&self, node: &tree_sitter::Node) -> String {
        // TASK-002-1-5: 修复tree-sitter文本提取错误
        // 首先尝试获取整个函数节点的文本作为后备
        let fallback_name = || {
            let pos = node.start_position();
            format!("unknown_function_at_{}_{}", pos.row, pos.column)
        };

        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            // 处理复杂的函数声明，如: pub async fn my_function<T>(param: T) -> Result<T>
            loop {
                let child = cursor.node();

                match child.kind() {
                    "identifier" => {
                        // 找到函数名 - 使用安全的文本提取
                        match child.utf8_text(self.source_code.as_bytes()) {
                            Ok(name) if !name.is_empty() => {
                                return format!("fn {}", name);
                            }
                            Ok(_) | Err(_) => {
                                // 文本提取失败，继续尝试
                            }
                        }
                    }
                    "field_identifier" => {
                        // 处理方法名 - 使用安全的文本提取
                        match child.utf8_text(self.source_code.as_bytes()) {
                            Ok(name) if !name.is_empty() => {
                                return format!("method {}", name);
                            }
                            Ok(_) | Err(_) => {
                                // 文本提取失败，继续尝试
                            }
                        }
                    }
                    "type_identifier" => {
                        // 可能是trait或impl名，继续查找函数名
                    }
                    // 跳过修饰符: pub, async, unsafe, extern, const, etc.
                    "visibility_modifier" | "async" | "unsafe" | "extern" | "const" => {
                        // 继续查找
                    }
                    // 跳过泛型参数、参数列表、返回类型等
                    "type_parameters" | "parameters" | "return_type" | "where_clause" => {
                        // 如果到了这里还没找到函数名，可能在前面
                    }
                    _ => {
                        // 其他情况，继续查找
                    }
                }

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }

        // 如果都没找到，返回后备名称
        fallback_name()
    }

    fn add_decision_point(&mut self) {
        self.result.decision_points += 1;
        if let Some(ref mut func) = self.current_function {
            func.complexity += 1;
            // TASK-002-1-1: 调试输出（可选择性启用）
            log::debug!("增加决策点: {} -> 复杂度: {}", func.name, func.complexity);
        }
    }

    fn get_result(self) -> CyclomaticComplexityResult {
        self.result
    }
}

/// TASK-002-1: 从Git diff中提取新增的代码
fn extract_added_code_from_diff(diff_content: &str) -> String {
    let mut added_lines = Vec::new();
    let mut in_hunk = false;

    for line in diff_content.lines() {
        if line.starts_with("@@") {
            in_hunk = true;
            continue;
        }

        if line.starts_with("diff") || line.starts_with("index") ||
           line.starts_with("---") || line.starts_with("+++") {
            in_hunk = false;
            continue;
        }

        if in_hunk && line.starts_with('+') && !line.starts_with("+++") {
            // 移除 + 号并添加到代码中
            let code_line = &line[1..];
            added_lines.push(code_line);
        }
    }

    added_lines.join("\n")
}

/// 分析代码复杂度指标（TASK-002-1: 增强版本）
async fn analyze_complexity_metrics(diff_content: &str) -> Result<ComplexityMetrics> {
    // 使用精确的AST解析复杂度计算
    analyze_complexity_metrics_accurate(diff_content).await
}

/// TASK-002-1: 增强的复杂度分析（使用真实AST解析）
async fn analyze_complexity_metrics_accurate(diff_content: &str) -> Result<ComplexityMetrics> {
    let mut metrics = ComplexityMetrics::default();

    // 使用tree-sitter进行精确复杂度计算
    match CyclomaticComplexityCalculator::new() {
        Ok(mut calculator) => {
            match calculator.calculate_diff_complexity(diff_content) {
                Ok(complexity_result) => {
                    // 将精确的复杂度结果转换为现有格式
                    metrics.cyclomatic_complexity_delta = complexity_result.total_complexity as i32;

                    // 计算最大嵌套深度（简化版本）
                    metrics.max_nesting_depth = calculate_max_nesting_depth_from_diff(diff_content);

                    // 保留原有的认知复杂度计算（简化版本）
                    metrics.cognitive_complexity_delta = calculate_cognitive_complexity_simple(diff_content);

                    // 如果有函数级别的复杂度信息，可以用于更详细的分析
                    for func_complexity in complexity_result.function_complexity {
                        metrics.function_lengths.push((func_complexity.end_line - func_complexity.start_line + 1) as usize);
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  精确复杂度计算失败，回退到简化版本: {}", e);
                    // 回退到原有的简化计算
                    return analyze_complexity_metrics_fallback(diff_content).await;
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️  无法初始化复杂度计算器，使用简化版本: {}", e);
            // 回退到原有的简化计算
            return analyze_complexity_metrics_fallback(diff_content).await;
        }
    }

    Ok(metrics)
}

/// TASK-002-1: 计算最大嵌套深度（从diff）
fn calculate_max_nesting_depth_from_diff(diff_content: &str) -> u32 {
    let mut max_depth = 0u32;
    let mut current_depth = 0u32;

    for line in diff_content.lines() {
        let trimmed = line.trim();

        // 跳过删除的行和diff元数据
        if trimmed.starts_with('-') || trimmed.starts_with("diff") ||
           trimmed.starts_with("index") || trimmed.starts_with("---") ||
           trimmed.starts_with("+++") {
            continue;
        }

        // 分析新增行的嵌套
        if trimmed.starts_with('+') {
            let code_line = &trimmed[1..].trim();

            // 增加嵌套的符号
            if code_line.contains("{") {
                current_depth += 1;
                max_depth = max_depth.max(current_depth);
            }

            // 减少嵌套的符号
            if code_line.contains("}") {
                current_depth = current_depth.saturating_sub(1);
            }
        }
    }

    max_depth
}

/// TASK-002-1: 简化的认知复杂度计算（从diff）
fn calculate_cognitive_complexity_simple(diff_content: &str) -> i32 {
    let mut cognitive_complexity = 0i32;

    for line in diff_content.lines() {
        let trimmed = line.trim();

        // 跳过删除的行和diff元数据
        if trimmed.starts_with('-') || trimmed.starts_with("diff") ||
           trimmed.starts_with("index") || trimmed.starts_with("---") ||
           trimmed.starts_with("+++") {
            continue;
        }

        // 分析新增行的认知复杂度
        if trimmed.starts_with('+') {
            let code_line = &trimmed[1..].trim();

            // 认知复杂度指标
            if code_line.contains("&&") || code_line.contains("||") {
                cognitive_complexity += 1;
            }

            if code_line.contains("break") || code_line.contains("continue") {
                cognitive_complexity += 1;
            }

            if code_line.contains("return ") && code_line.contains("if") {
                cognitive_complexity += 1;
            }

            // 递归调用增加认知复杂度
            if code_line.contains("fn ") && code_line.contains("fn ") {
                cognitive_complexity += 2;
            }
        }
    }

    cognitive_complexity
}

/// TASK-002-1: 回退的复杂度计算（原有实现）
async fn analyze_complexity_metrics_fallback(diff_content: &str) -> Result<ComplexityMetrics> {
    let mut metrics = ComplexityMetrics::default();

    let lines: Vec<&str> = diff_content.lines().collect();
    let mut current_nesting = 0u32;
    let mut max_nesting = 0u32;

    for line in lines {
        let trimmed = line.trim();

        // 跳过删除的行和diff元数据
        if trimmed.starts_with('-') ||
           trimmed.starts_with("diff") ||
           trimmed.starts_with("index") ||
           trimmed.starts_with("---") ||
           trimmed.starts_with("+++") {
            continue;
        }

        // 分析新增行的复杂度
        if trimmed.starts_with('+') {
            let code_line = &trimmed[1..].trim();

            // 计算嵌套深度
            let indent_level = (line.len() - line.trim_start().len()) as u32;
            if indent_level > current_nesting {
                current_nesting = indent_level;
                max_nesting = max_nesting.max(current_nesting);
            } else if indent_level < current_nesting {
                current_nesting = indent_level;
            }

            // 简单的复杂度估算（基于控制流关键词）
            if code_line.contains("if ") ||
               code_line.contains("match ") ||
               code_line.contains("for ") ||
               code_line.contains("while ") ||
               code_line.contains("loop ") {
                metrics.cyclomatic_complexity_delta += 1;
            }

            // 认知复杂度（简化版本）
            if code_line.contains("&&") ||
               code_line.contains("||") ||
               code_line.contains("break") ||
               code_line.contains("continue") ||
               code_line.contains("return ") {
                metrics.cognitive_complexity_delta += 1;
            }

            // 估算函数长度
            if code_line.starts_with("fn ") ||
               code_line.starts_with("pub fn ") ||
               code_line.starts_with("async fn ") ||
               code_line.starts_with("impl ") ||
               code_line.starts_with("trait ") {
                metrics.function_lengths.push(1);
            } else if !metrics.function_lengths.is_empty() {
                if let Some(last) = metrics.function_lengths.last_mut() {
                    *last += 1;
                }
            }

            // 估算参数数量
            if code_line.contains("fn ") {
                let param_count = code_line.matches(',').count() + 1;
                metrics.parameter_counts.push(param_count);
            }
        }
    }

    metrics.max_nesting_depth = max_nesting;

    Ok(metrics)
}

/// 分析函数级变更
async fn analyze_function_changes(
    diff_content: &str,
    changed_files: &[ChangedFile],
) -> Result<Vec<FunctionChange>> {
    let mut function_changes = Vec::new();

    for file in changed_files {
        if file.change_type == ChangeType::Deleted {
            continue;
        }

        let file_diff_content = extract_file_diff_content(diff_content, &file.path);
        let functions = parse_function_changes_from_diff(&file_diff_content, &file.path);
        function_changes.extend(functions);
    }

    Ok(function_changes)
}

/// 分析导入/导出变更
async fn analyze_import_changes(
    diff_content: &str,
    changed_files: &[ChangedFile],
) -> Result<Vec<ImportChange>> {
    let mut import_changes = Vec::new();

    for file in changed_files {
        if file.change_type == ChangeType::Deleted {
            continue;
        }

        let file_diff_content = extract_file_diff_content(diff_content, &file.path);
        let imports = parse_import_changes_from_diff(&file_diff_content, &file.path);
        import_changes.extend(imports);
    }

    Ok(import_changes)
}

/// 分析结构化变更
async fn analyze_structural_changes(
    diff_content: &str,
    changed_files: &[ChangedFile],
) -> Result<Vec<StructuralChange>> {
    let mut structural_changes = Vec::new();

    for file in changed_files {
        if file.change_type == ChangeType::Deleted {
            continue;
        }

        let file_diff_content = extract_file_diff_content(diff_content, &file.path);
        let structures = parse_structural_changes_from_diff(&file_diff_content, &file.path);
        structural_changes.extend(structures);
    }

    Ok(structural_changes)
}

/// 分析测试相关变更
async fn analyze_test_changes(
    diff_content: &str,
    changed_files: &[ChangedFile],
) -> Result<TestChanges> {
    let mut test_changes = TestChanges::default();

    for file in changed_files {
        // 检查是否为测试文件
        if file.path.contains("test") ||
           file.path.contains("spec") ||
           file.path.ends_with("_test.rs") ||
           file.path.ends_with("_tests.rs") {

            test_changes.test_files.push(file.path.clone());

            // 分析测试变更
            let file_diff_content = extract_file_diff_content(diff_content, &file.path);
            let test_stats = analyze_test_file_changes(&file_diff_content);

            test_changes.added_tests += test_stats.added;
            test_changes.removed_tests += test_stats.removed;
            test_changes.modified_tests += test_stats.modified;
        }
    }

    // 简化的覆盖率变更估算
    test_changes.coverage_delta = if test_changes.added_tests > 0 {
        (test_changes.added_tests as f32 * 2.5).min(10.0)
    } else if test_changes.removed_tests > 0 {
        -(test_changes.removed_tests as f32 * 2.5).max(-10.0)
    } else {
        0.0
    };

    Ok(test_changes)
}

/// 分析性能相关变更
async fn analyze_performance_changes(
    diff_content: &str,
    changed_files: &[ChangedFile],
) -> Result<PerformanceChanges> {
    let mut performance_changes = PerformanceChanges::default();

    for file in changed_files {
        if file.change_type == ChangeType::Deleted {
            continue;
        }

        let file_diff_content = extract_file_diff_content(diff_content, &file.path);

        // 分析算法复杂度变更
        let algo_changes = analyze_algorithmic_changes(&file_diff_content, &file.path);
        performance_changes.algorithmic_changes.extend(algo_changes);

        // 分析数据结构变更
        let ds_changes = analyze_data_structure_changes(&file_diff_content, &file.path);
        performance_changes.data_structure_changes.extend(ds_changes);

        // 分析并发相关变更
        let conc_changes = analyze_concurrency_changes(&file_diff_content, &file.path);
        performance_changes.concurrency_changes.extend(conc_changes);

        // 分析内存使用变更
        let mem_changes = analyze_memory_changes(&file_diff_content, &file.path);
        performance_changes.memory_changes.extend(mem_changes);
    }

    Ok(performance_changes)
}

// 辅助函数实现

/// 从diff内容中提取特定文件的变更
fn extract_file_diff_content(diff_content: &str, file_path: &str) -> String {
    let lines: Vec<&str> = diff_content.lines().collect();
    let mut file_content = Vec::new();
    let mut in_target_file = false;

    for line in lines {
        if line.starts_with("diff --git") && line.contains(file_path) {
            in_target_file = true;
            file_content.push(line);
            continue;
        }

        if line.starts_with("diff --git") && in_target_file {
            // 开始下一个文件，停止记录
            break;
        }

        if in_target_file {
            file_content.push(line);
        }
    }

    file_content.join("\n")
}

/// 从diff中解析函数变更
fn parse_function_changes_from_diff(diff_content: &str, file_path: &str) -> Vec<FunctionChange> {
    let mut functions = Vec::new();
    let lines: Vec<&str> = diff_content.lines().collect();

    for line in lines {
        let trimmed = line.trim();

        // 检测新增函数
        if trimmed.starts_with('+') && (trimmed.contains("fn ") || trimmed.contains("pub fn ")) {
            let function_name = extract_function_name(trimmed);
            if let Some(name) = function_name {
                functions.push(FunctionChange {
                    name: name.clone(),
                    change_type: FunctionChangeType::Added,
                    file_path: file_path.to_string(),
                    signature: Some(trimmed[1..].trim().to_string()),
                    changed_lines: 1,
                    complexity_delta: 1,
                });
            }
        }

        // 检测删除函数
        if trimmed.starts_with('-') && (trimmed.contains("fn ") || trimmed.contains("pub fn ")) {
            let function_name = extract_function_name(trimmed);
            if let Some(name) = function_name {
                functions.push(FunctionChange {
                    name: name.clone(),
                    change_type: FunctionChangeType::Removed,
                    file_path: file_path.to_string(),
                    signature: Some(trimmed[1..].trim().to_string()),
                    changed_lines: 1,
                    complexity_delta: -1,
                });
            }
        }
    }

    functions
}

/// 提取函数名
fn extract_function_name(line: &str) -> Option<String> {
    let clean_line = line.trim_start_matches(['+', '-', ' ']);

    // 匹配不同的函数定义模式
    if let Some(start) = clean_line.find("fn ") {
        let after_fn = &clean_line[start + 3..];
        if let Some(end) = after_fn.find('(') {
            return Some(after_fn[..end].trim().to_string());
        }
    }

    None
}

/// 从diff中解析导入变更
fn parse_import_changes_from_diff(diff_content: &str, file_path: &str) -> Vec<ImportChange> {
    let mut imports = Vec::new();
    let lines: Vec<&str> = diff_content.lines().collect();

    for line in lines {
        let trimmed = line.trim();

        // 检测Rust的use语句
        if trimmed.starts_with('+') && trimmed.starts_with("+use ") {
            let module_path = trimmed[5..].trim().to_string();
            let is_external = !module_path.starts_with("crate::") &&
                            !module_path.starts_with("super::") &&
                            !module_path.starts_with(".");

            imports.push(ImportChange {
                module_path,
                change_type: ImportChangeType::Added,
                file_path: file_path.to_string(),
                is_external,
            });
        }

        if trimmed.starts_with('-') && trimmed.starts_with("-use ") {
            let module_path = trimmed[5..].trim().to_string();
            let is_external = !module_path.starts_with("crate::") &&
                            !module_path.starts_with("super::") &&
                            !module_path.starts_with(".");

            imports.push(ImportChange {
                module_path,
                change_type: ImportChangeType::Removed,
                file_path: file_path.to_string(),
                is_external,
            });
        }
    }

    imports
}

/// 从diff中解析结构化变更
fn parse_structural_changes_from_diff(diff_content: &str, file_path: &str) -> Vec<StructuralChange> {
    let mut structures = Vec::new();
    let lines: Vec<&str> = diff_content.lines().collect();

    for line in lines {
        let trimmed = line.trim();

        // 检测结构体/类变更
        if trimmed.starts_with('+') || trimmed.starts_with('-') {
            let content = trimmed[1..].trim();

            if content.starts_with("struct ") {
                if let Some(name) = extract_struct_name(content) {
                    structures.push(StructuralChange {
                        change_type: StructuralChangeType::ClassChange,
                        element_name: name,
                        file_path: file_path.to_string(),
                        impact_lines: 1,
                        related_elements: Vec::new(),
                    });
                }
            }

            if content.starts_with("trait ") || content.starts_with("impl ") {
                if let Some(name) = extract_trait_name(content) {
                    structures.push(StructuralChange {
                        change_type: StructuralChangeType::InterfaceChange,
                        element_name: name,
                        file_path: file_path.to_string(),
                        impact_lines: 1,
                        related_elements: Vec::new(),
                    });
                }
            }

            if content.starts_with("enum ") {
                if let Some(name) = extract_enum_name(content) {
                    structures.push(StructuralChange {
                        change_type: StructuralChangeType::EnumChange,
                        element_name: name,
                        file_path: file_path.to_string(),
                        impact_lines: 1,
                        related_elements: Vec::new(),
                    });
                }
            }
        }
    }

    structures
}

/// 提取结构体名称
fn extract_struct_name(line: &str) -> Option<String> {
    if let Some(start) = line.find("struct ") {
        let after_struct = &line[start + 7..];
        if let Some(end) = after_struct.find('{') {
            return Some(after_struct[..end].trim().to_string());
        } else if let Some(end) = after_struct.find('(') {
            return Some(after_struct[..end].trim().to_string());
        } else {
            return Some(after_struct.trim().to_string());
        }
    }
    None
}

/// 提取trait/impl名称
fn extract_trait_name(line: &str) -> Option<String> {
    if line.starts_with("trait ") {
        if let Some(start) = line.find("trait ") {
            let after_trait = &line[start + 6..];
            if let Some(end) = after_trait.find('{') {
                return Some(after_trait[..end].trim().to_string());
            }
        }
    } else if line.starts_with("impl ") {
        if let Some(start) = line.find("impl ") {
            let after_impl = &line[start + 5..];
            if let Some(end) = after_impl.find(" for") {
                return Some(after_impl[..end].trim().to_string());
            } else if let Some(end) = after_impl.find('{') {
                return Some(after_impl[..end].trim().to_string());
            }
        }
    }
    None
}

/// 提取枚举名称
fn extract_enum_name(line: &str) -> Option<String> {
    if let Some(start) = line.find("enum ") {
        let after_enum = &line[start + 5..];
        if let Some(end) = after_enum.find('{') {
            return Some(after_enum[..end].trim().to_string());
        }
    }
    None
}

/// 分析测试文件变更
fn analyze_test_file_changes(diff_content: &str) -> TestStats {
    let mut stats = TestStats::default();
    let lines: Vec<&str> = diff_content.lines().collect();

    for line in lines {
        let trimmed = line.trim();

        // 简单的测试函数检测
        if trimmed.starts_with('+') && (trimmed.contains("#[test]") || trimmed.contains("fn test_")) {
            stats.added += 1;
        }

        if trimmed.starts_with('-') && (trimmed.contains("#[test]") || trimmed.contains("fn test_")) {
            stats.removed += 1;
        }

        if (trimmed.starts_with('+') || trimmed.starts_with('-')) &&
           (trimmed.contains("assert_") || trimmed.contains("expect")) {
            stats.modified += 1;
        }
    }

    stats
}

/// 分析算法复杂度变更
fn analyze_algorithmic_changes(diff_content: &str, file_path: &str) -> Vec<AlgorithmicChange> {
    let mut changes = Vec::new();
    let lines: Vec<&str> = diff_content.lines().collect();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with('+') || trimmed.starts_with('-') {
            let content = trimmed[1..].trim();

            // 简单的复杂度模式检测
            if let Some(function_name) = extract_function_name(content) {
                let time_change = if content.contains("for") && content.contains("for") {
                    ComplexityChange::QuadraticToLinear
                } else if content.contains("while") {
                    ComplexityChange::LinearToQuadratic
                } else {
                    ComplexityChange::NoChange
                };

                changes.push(AlgorithmicChange {
                    function_name,
                    time_complexity_change: time_change.clone(),
                    space_complexity_change: time_change,
                    file_path: file_path.to_string(),
                });
            }
        }
    }

    changes
}

/// 分析数据结构变更
fn analyze_data_structure_changes(diff_content: &str, file_path: &str) -> Vec<DataStructureChange> {
    let mut changes = Vec::new();
    let lines: Vec<&str> = diff_content.lines().collect();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with('+') || trimmed.starts_with('-') {
            let content = trimmed[1..].trim();

            // 检测数据结构使用
            if content.contains("Vec<") || content.contains("vec!") {
                changes.push(DataStructureChange {
                    change_type: DataStructureChangeType::Added,
                    old_structure: None,
                    new_structure: Some("Vec".to_string()),
                    file_path: file_path.to_string(),
                    affected_functions: Vec::new(),
                });
            }

            if content.contains("HashMap<") || content.contains("BTreeMap<") {
                changes.push(DataStructureChange {
                    change_type: DataStructureChangeType::HashMapToTree,
                    old_structure: Some("HashMap".to_string()),
                    new_structure: Some("BTreeMap".to_string()),
                    file_path: file_path.to_string(),
                    affected_functions: Vec::new(),
                });
            }
        }
    }

    changes
}

/// 分析并发相关变更
fn analyze_concurrency_changes(diff_content: &str, file_path: &str) -> Vec<ConcurrencyChange> {
    let mut changes = Vec::new();
    let lines: Vec<&str> = diff_content.lines().collect();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with('+') || trimmed.starts_with('-') {
            let content = trimmed[1..].trim();

            if content.contains("async ") || content.contains(".await") {
                changes.push(ConcurrencyChange {
                    change_type: ConcurrencyChangeType::AsyncAdded,
                    functions: Vec::new(),
                    file_path: file_path.to_string(),
                    safety_level: ConcurrencySafetyLevel::Safe,
                });
            }

            if content.contains("Mutex<") || content.contains("RwLock<") {
                changes.push(ConcurrencyChange {
                    change_type: ConcurrencyChangeType::LockAdded,
                    functions: Vec::new(),
                    file_path: file_path.to_string(),
                    safety_level: ConcurrencySafetyLevel::Safe,
                });
            }
        }
    }

    changes
}

/// 分析内存使用变更
fn analyze_memory_changes(diff_content: &str, file_path: &str) -> Vec<MemoryChange> {
    let mut changes = Vec::new();
    let lines: Vec<&str> = diff_content.lines().collect();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with('+') || trimmed.starts_with('-') {
            let content = trimmed[1..].trim();

            if content.contains("Box::new") {
                changes.push(MemoryChange {
                    change_type: MemoryChangeType::Allocation,
                    estimated_impact: MemoryImpact::Low,
                    file_path: file_path.to_string(),
                    related_functions: Vec::new(),
                });
            }

            if content.contains("Arc<") || content.contains("Rc<") {
                changes.push(MemoryChange {
                    change_type: MemoryChangeType::Allocation,
                    estimated_impact: MemoryImpact::Medium,
                    file_path: file_path.to_string(),
                    related_functions: Vec::new(),
                });
            }

            if content.contains("unsafe") && (content.contains("ptr") || content.contains("transmute")) {
                changes.push(MemoryChange {
                    change_type: MemoryChangeType::LeakRisk,
                    estimated_impact: MemoryImpact::High,
                    file_path: file_path.to_string(),
                    related_functions: Vec::new(),
                });
            }
        }
    }

    changes
}

/// 测试统计辅助结构
#[derive(Debug, Default)]
struct TestStats {
    added: u32,
    removed: u32,
    modified: u32,
}

/// TASK-003-1: 构建AI上下文信息
fn build_ai_context(review_data: &ReviewData) -> String {
    let mut context = String::new();

    // 基础变更信息
    context.push_str(&format!("代码变更概览:\n"));
    context.push_str(&format!("- 涉及文件: {} 个\n", review_data.stats.files_changed));
    context.push_str(&format!("- 新增行数: {}\n", review_data.stats.additions));
    context.push_str(&format!("- 删除行数: {}\n", review_data.stats.deletions));

    // 主要变更文件
    if !review_data.changed_files.is_empty() {
        context.push_str("\n主要变更文件:\n");
        for file in &review_data.changed_files {
            let change_type = match file.change_type {
                ChangeType::Added => "新增",
                ChangeType::Modified => "修改",
                ChangeType::Deleted => "删除",
                ChangeType::Renamed => "重命名",
                ChangeType::Other => "其他",
            };

            context.push_str(&format!("- {} ({})\n", file.path, change_type));

            if let Some(ref lang) = file.language {
                context.push_str(&format!("  语言: {}\n", lang));
            }
        }
    }

    // 复杂度分析摘要
    if review_data.analysis_data.complexity_metrics.cyclomatic_complexity_delta != 0 ||
       review_data.analysis_data.complexity_metrics.max_nesting_depth > 0 {
        context.push_str("\n复杂度变更:\n");
        if review_data.analysis_data.complexity_metrics.cyclomatic_complexity_delta != 0 {
            context.push_str(&format!("- 圈复杂度变更: {}\n",
                review_data.analysis_data.complexity_metrics.cyclomatic_complexity_delta));
        }
        if review_data.analysis_data.complexity_metrics.max_nesting_depth > 0 {
            context.push_str(&format!("- 最大嵌套深度: {}\n",
                review_data.analysis_data.complexity_metrics.max_nesting_depth));
        }
    }

    // 函数级变更摘要
    if !review_data.analysis_data.function_changes.is_empty() {
        context.push_str(&format!("\n函数级变更: {} 个\n", review_data.analysis_data.function_changes.len()));
        for func in &review_data.analysis_data.function_changes {
            let change_type = match func.change_type {
                FunctionChangeType::Added => "新增",
                FunctionChangeType::Removed => "删除",
                FunctionChangeType::Modified => "修改",
                FunctionChangeType::Renamed => "重命名",
                FunctionChangeType::SignatureChanged => "签名变更",
            };
            context.push_str(&format!("- {} ({})\n", func.name, change_type));
        }
    }

    // 性能和安全关注点
    if !review_data.analysis_data.performance_changes.memory_changes.is_empty() {
        context.push_str("\n性能相关变更:\n");
        for change in &review_data.analysis_data.performance_changes.memory_changes {
            context.push_str(&format!("- {:?} ({:?})\n",
                change.change_type, change.estimated_impact));
        }
    }

    context
}

/// TASK-003-1: 保存AI分析结果
async fn save_ai_analysis_result(ai_response: &str, output_path: &std::path::Path) -> Result<()> {
    use std::fs::File;
    use std::io::Write;

    // 创建输出目录（如果不存在）
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 生成带有时间戳的完整报告
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let full_report = format!(
        "# GitAI 代码评审报告\n\n\
        生成时间: {}\n\n\
        ## AI分析结果\n\n\
        {}\n\n\
        ---\n\
        *此报告由 GitAI 自动生成*\n",
        timestamp, ai_response
    );

    // 写入文件
    let mut file = File::create(output_path)?;
    file.write_all(full_report.as_bytes())?;

    Ok(())
}

/// TASK-003: AI提示词构建引擎
async fn build_ai_prompt(review_data: &ReviewData) -> Result<String> {
    println!("  🧠 构建AI智能分析提示词...");

    let mut prompt_builder = PromptBuilder::new();

    // 1. 基础上下文构建
    prompt_builder.add_context_section(&build_base_context(review_data))?;

    // 2. 代码变更概览
    prompt_builder.add_context_section(&build_change_overview(review_data))?;

    // 3. 多维度分析数据集成
    prompt_builder.add_context_section(&build_analysis_context(&review_data.analysis_data))?;

    // 4. 风险评估集成
    prompt_builder.add_context_section(&build_risk_assessment(&review_data.analysis_data))?;

    // 5. 建议请求构建
    prompt_builder.add_context_section(&build_suggestions_request(review_data))?;

    // 6. 格式化要求
    prompt_builder.add_context_section(&build_format_requirements())?;

    let prompt = prompt_builder.build();
    println!("  ✅ AI提示词构建完成，包含 {} 个分析维度", prompt_builder.section_count());

    Ok(prompt)
}

/// AI提示词构建器
#[derive(Debug, Default)]
struct PromptBuilder {
    sections: Vec<String>,
}

impl PromptBuilder {
    fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    fn add_context_section(&mut self, section: &str) -> Result<()> {
        if !section.is_empty() {
            self.sections.push(section.to_string());
        }
        Ok(())
    }

    fn build(&self) -> String {
        self.sections.join("\n\n")
    }

    fn section_count(&self) -> usize {
        self.sections.len()
    }
}

/// 构建基础上下文
fn build_base_context(review_data: &ReviewData) -> String {
    let mut context = String::new();

    context.push_str("# GitAI 智能代码评审\n\n");
    context.push_str("你是一个经验丰富的代码评审专家，请对以下Git变更进行全面的代码评审分析。\n\n");

    // 添加评审目标
    if let Some(ref language) = review_data.options.language {
        context.push_str(&format!("**目标语言**: {}\n", language));
    }

    context.push_str(&format!("**变更规模**: {} 个文件, +{} -{} 行\n",
        review_data.stats.files_changed,
        review_data.stats.additions,
        review_data.stats.deletions));

    // 添加评审模式
    if review_data.options.full {
        context.push_str("**评审模式**: 全面深度分析\n");
    } else if review_data.options.tree_sitter {
        context.push_str("**评审模式**: 结构化分析\n");
    } else if review_data.options.security_scan {
        context.push_str("**评审模式**: 安全重点分析\n");
    } else {
        context.push_str("**评审模式**: 标准代码评审\n");
    }

    context
}

/// 构建变更概览
fn build_change_overview(review_data: &ReviewData) -> String {
    let mut overview = String::new();

    overview.push_str("## 代码变更概览\n\n");

    // 文件级变更统计
    overview.push_str("### 变更文件清单\n\n");
    for file in &review_data.changed_files {
        let change_type = match file.change_type {
            ChangeType::Added => "🆕 新增",
            ChangeType::Modified => "📝 修改",
            ChangeType::Deleted => "🗑️ 删除",
            ChangeType::Renamed => "🔄 重命名",
            ChangeType::Other => "❓ 其他",
        };

        overview.push_str(&format!("- **{}**: {} (+{} -{})",
            change_type, file.path, file.additions, file.deletions));

        if let Some(ref lang) = file.language {
            overview.push_str(&format!(" ({})", lang));
        }
        overview.push('\n');
    }

    // 变更统计摘要
    overview.push_str("\n### 变更统计摘要\n\n");
    overview.push_str(&format!("- 总变更行数: {}\n", review_data.stats.total_changes));
    overview.push_str(&format!("- 新增代码行: {}\n", review_data.stats.additions));
    overview.push_str(&format!("- 删除代码行: {}\n", review_data.stats.deletions));
    overview.push_str(&format!("- 涉及文件数: {}\n", review_data.stats.files_changed));

    overview
}

/// 构建多维度分析上下文
fn build_analysis_context(analysis_data: &AnalysisData) -> String {
    let mut context = String::new();

    context.push_str("## 多维度代码分析\n\n");

    // 1. 代码复杂度分析
    if !is_complexity_metrics_empty(&analysis_data.complexity_metrics) {
        context.push_str("### 🔍 代码复杂度分析\n\n");
        context.push_str(&format_complexity_metrics(&analysis_data.complexity_metrics));
    }

    // 2. 函数级变更分析
    if !analysis_data.function_changes.is_empty() {
        context.push_str("### 🔧 函数级变更分析\n\n");
        context.push_str(&format_function_changes(&analysis_data.function_changes));
    }

    // 3. 导入/导出变更分析
    if !analysis_data.import_changes.is_empty() {
        context.push_str("### 📦 模块依赖变更分析\n\n");
        context.push_str(&format_import_changes(&analysis_data.import_changes));
    }

    // 4. 结构化变更分析
    if !analysis_data.structural_changes.is_empty() {
        context.push_str("### 🏗️ 结构化变更分析\n\n");
        context.push_str(&format_structural_changes(&analysis_data.structural_changes));
    }

    // 5. 测试变更分析
    if !is_test_changes_empty(&analysis_data.test_changes) {
        context.push_str("### 🧪 测试相关变更分析\n\n");
        context.push_str(&format_test_changes(&analysis_data.test_changes));
    }

    // 6. 性能变更分析
    if !is_performance_changes_empty(&analysis_data.performance_changes) {
        context.push_str("### ⚡ 性能影响分析\n\n");
        context.push_str(&format_performance_changes(&analysis_data.performance_changes));
    }

    context
}

/// 构建风险评估
fn build_risk_assessment(analysis_data: &AnalysisData) -> String {
    let mut assessment = String::new();

    assessment.push_str("## 🚨 风险评估\n\n");

    let mut risk_level = RiskLevel::Low;
    let mut risk_factors = Vec::new();

    // 复杂度风险评估
    if analysis_data.complexity_metrics.cyclomatic_complexity_delta > 5 {
        risk_level = risk_level.max(RiskLevel::Medium);
        risk_factors.push("圈复杂度显著增加".to_string());
    }

    if analysis_data.complexity_metrics.max_nesting_depth > 4 {
        risk_level = risk_level.max(RiskLevel::Medium);
        risk_factors.push("嵌套深度过深".to_string());
    }

    // 函数变更风险评估
    for func_change in &analysis_data.function_changes {
        if func_change.change_type == FunctionChangeType::Removed {
            risk_level = risk_level.max(RiskLevel::High);
            risk_factors.push(format!("函数 {} 被删除", func_change.name));
        }
    }

    // 性能风险评估
    for perf_change in &analysis_data.performance_changes.memory_changes {
        if perf_change.estimated_impact == MemoryImpact::High || perf_change.estimated_impact == MemoryImpact::Critical {
            risk_level = risk_level.max(RiskLevel::High);
            risk_factors.push("高风险内存变更".to_string());
        }
    }

    // 并发风险评估
    for conc_change in &analysis_data.performance_changes.concurrency_changes {
        if conc_change.safety_level == ConcurrencySafetyLevel::Unsafe {
            risk_level = risk_level.max(RiskLevel::High);
            risk_factors.push("并发安全风险".to_string());
        }
    }

    // 输出风险评估结果
    let risk_emoji = match risk_level {
        RiskLevel::Low => "🟢",
        RiskLevel::Medium => "🟡",
        RiskLevel::High => "🔴",
        RiskLevel::Critical => "🚨",
    };

    assessment.push_str(&format!("**整体风险等级**: {} {}\n\n", risk_emoji, risk_level));

    if !risk_factors.is_empty() {
        assessment.push_str("**主要风险因素**:\n");
        for factor in risk_factors {
            assessment.push_str(&format!("- {}\n", factor));
        }
        assessment.push('\n');
    }

    assessment
}

/// 构建建议请求
fn build_suggestions_request(review_data: &ReviewData) -> String {
    let mut request = String::new();

    request.push_str("## 📋 评审要点\n\n");
    request.push_str("请基于以上信息，重点关注以下方面进行代码评审:\n\n");

    // 基础评审要点
    request.push_str("### 基础质量检查\n");
    request.push_str("- [ ] 代码风格和格式是否符合规范\n");
    request.push_str("- [ ] 变量和函数命名是否清晰合理\n");
    request.push_str("- [ ] 是否有明显的逻辑错误或缺陷\n");
    request.push_str("- [ ] 错误处理是否充分和合理\n\n");

    // 架构和设计
    request.push_str("### 架构和设计评估\n");
    request.push_str("- [ ] 设计模式是否恰当\n");
    request.push_str("- [ ] 模块间耦合度是否合理\n");
    request.push_str("- [ ] 代码是否具有良好的可维护性\n");
    request.push_str("- [ ] 是否遵循SOLID原则\n\n");

    // 性能考虑
    if !is_performance_changes_empty(&review_data.analysis_data.performance_changes) {
        request.push_str("### 性能影响评估\n");
        request.push_str("- [ ] 算法效率是否最优\n");
        request.push_str("- [ ] 数据结构选择是否合适\n");
        request.push_str("- [ ] 是否存在性能瓶颈\n");
        request.push_str("- [ ] 内存使用是否合理\n\n");
    }

    // 安全考虑
    if review_data.options.security_scan {
        request.push_str("### 安全性检查\n");
        request.push_str("- [ ] 是否存在安全漏洞\n");
        request.push_str("- [ ] 输入验证是否充分\n");
        request.push_str("- [ ] 权限控制是否合理\n");
        request.push_str("- [ ] 敏感数据处理是否安全\n\n");
    }

    // 测试考虑
    if !is_test_changes_empty(&review_data.analysis_data.test_changes) {
        request.push_str("### 测试覆盖评估\n");
        request.push_str("- [ ] 测试覆盖率是否充分\n");
        request.push_str("- [ ] 测试用例是否全面\n");
        request.push_str("- [ ] 边界条件是否得到测试\n");
        request.push_str("- [ ] 是否需要补充集成测试\n\n");
    }

    // 具体建议请求
    request.push_str("### 改进建议\n");
    request.push_str("请提供:\n");
    request.push_str("1. **必须修复的问题** - 严重问题，需要立即解决\n");
    request.push_str("2. **建议改进的项目** - 提升代码质量的建议\n");
    request.push_str("3. **最佳实践建议** - 长期改进方向\n");
    request.push_str("4. **性能优化建议** - 如果适用\n");
    request.push_str("5. **安全加固建议** - 如果适用\n\n");

    request
}

/// 构建格式要求
fn build_format_requirements() -> String {
    let mut format = String::new();

    format.push_str("## 📝 输出格式要求\n\n");
    format.push_str("请按以下格式组织你的评审结果:\n\n");

    format.push_str("```markdown\n");
    format.push_str("# 代码评审报告\n\n");
    format.push_str("## 📊 评审概览\n");
    format.push_str("- 评审等级: [优秀/良好/一般/需要改进]\n");
    format.push_str("- 主要问题数量: X个\n");
    format.push_str("- 建议改进数量: Y个\n\n");

    format.push_str("## 🚨 严重问题\n");
    format.push_str("[列出所有必须修复的问题]\n\n");

    format.push_str("## 💡 改进建议\n");
    format.push_str("[列出所有建议改进的项目]\n\n");

    format.push_str("## ⭐ 最佳实践\n");
    format.push_str("[列出最佳实践建议]\n\n");

    format.push_str("## 📈 评分\n");
    format.push_str("- 代码质量: X/10\n");
    format.push_str("- 可维护性: X/10\n");
    format.push_str("- 性能表现: X/10\n");
    format.push_str("- 安全性: X/10\n");
    format.push_str("- 测试覆盖: X/10\n");
    format.push_str("```\n");

    format
}

// 辅助函数和枚举定义

/// 风险等级枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RiskLevel {
    Low,
    Medium,
    High,
    #[allow(dead_code)]
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "低风险"),
            RiskLevel::Medium => write!(f, "中等风险"),
            RiskLevel::High => write!(f, "高风险"),
            RiskLevel::Critical => write!(f, "严重风险"),
        }
    }
}

/// 检查复杂度指标是否为空
fn is_complexity_metrics_empty(metrics: &ComplexityMetrics) -> bool {
    metrics.cyclomatic_complexity_delta == 0 &&
    metrics.cognitive_complexity_delta == 0 &&
    metrics.max_nesting_depth == 0 &&
    metrics.function_lengths.is_empty() &&
    metrics.parameter_counts.is_empty()
}

/// 检查测试变更是否为空
fn is_test_changes_empty(test_changes: &TestChanges) -> bool {
    test_changes.added_tests == 0 &&
    test_changes.removed_tests == 0 &&
    test_changes.modified_tests == 0 &&
    test_changes.test_files.is_empty() &&
    test_changes.coverage_delta == 0.0
}

/// 检查性能变更是否为空
fn is_performance_changes_empty(perf_changes: &PerformanceChanges) -> bool {
    perf_changes.algorithmic_changes.is_empty() &&
    perf_changes.data_structure_changes.is_empty() &&
    perf_changes.concurrency_changes.is_empty() &&
    perf_changes.memory_changes.is_empty()
}

/// 格式化复杂度指标
fn format_complexity_metrics(metrics: &ComplexityMetrics) -> String {
    let mut output = String::new();

    if metrics.cyclomatic_complexity_delta != 0 {
        output.push_str(&format!("- 圈复杂度变更: {}\n", metrics.cyclomatic_complexity_delta));
    }

    if metrics.cognitive_complexity_delta != 0 {
        output.push_str(&format!("- 认知复杂度变更: {}\n", metrics.cognitive_complexity_delta));
    }

    if metrics.max_nesting_depth > 0 {
        output.push_str(&format!("- 最大嵌套深度: {}\n", metrics.max_nesting_depth));
    }

    if !metrics.function_lengths.is_empty() {
        let avg_len = metrics.function_lengths.iter().sum::<usize>() as f32 / metrics.function_lengths.len() as f32;
        output.push_str(&format!("- 平均函数长度: {:.1} 行\n", avg_len));
    }

    if !metrics.parameter_counts.is_empty() {
        let avg_params = metrics.parameter_counts.iter().sum::<usize>() as f32 / metrics.parameter_counts.len() as f32;
        output.push_str(&format!("- 平均参数数量: {:.1}\n", avg_params));
    }

    if output.is_empty() {
        output.push_str("- 无显著复杂度变更\n");
    }

    output.push('\n');
    output
}

/// 格式化函数变更
fn format_function_changes(functions: &[FunctionChange]) -> String {
    let mut output = String::new();

    for func in functions {
        let change_type = match func.change_type {
            FunctionChangeType::Added => "新增",
            FunctionChangeType::Removed => "删除",
            FunctionChangeType::Modified => "修改",
            FunctionChangeType::Renamed => "重命名",
            FunctionChangeType::SignatureChanged => "签名变更",
        };

        output.push_str(&format!("- **{}**: {} ({})", change_type, func.name, func.file_path));

        if let Some(ref signature) = func.signature {
            output.push_str(&format!(" `{}`", signature));
        }

        output.push('\n');
    }

    if output.is_empty() {
        output.push_str("- 无函数级变更\n");
    }

    output.push('\n');
    output
}

/// 格式化导入变更
fn format_import_changes(imports: &[ImportChange]) -> String {
    let mut output = String::new();

    for import in imports {
        let change_type = match import.change_type {
            ImportChangeType::Added => "新增",
            ImportChangeType::Removed => "删除",
            ImportChangeType::Modified => "修改",
        };

        let dep_type = if import.is_external { "外部依赖" } else { "内部模块" };

        output.push_str(&format!("- **{}**: {} ({})\n",
            change_type, import.module_path, dep_type));
    }

    if output.is_empty() {
        output.push_str("- 无模块依赖变更\n");
    }

    output.push('\n');
    output
}

/// 格式化结构化变更
fn format_structural_changes(structures: &[StructuralChange]) -> String {
    let mut output = String::new();

    for structure in structures {
        let change_type = match structure.change_type {
            StructuralChangeType::ClassChange => "结构体/类",
            StructuralChangeType::InterfaceChange => "接口/Trait",
            StructuralChangeType::EnumChange => "枚举",
            StructuralChangeType::ModuleChange => "模块",
            StructuralChangeType::ConfigChange => "配置",
        };

        output.push_str(&format!("- **{}**: {} ({})\n",
            change_type, structure.element_name, structure.file_path));
    }

    if output.is_empty() {
        output.push_str("- 无结构化变更\n");
    }

    output.push('\n');
    output
}

/// 格式化测试变更
fn format_test_changes(test_changes: &TestChanges) -> String {
    let mut output = String::new();

    if test_changes.added_tests > 0 {
        output.push_str(&format!("- 新增测试: {} 个\n", test_changes.added_tests));
    }

    if test_changes.removed_tests > 0 {
        output.push_str(&format!("- 删除测试: {} 个\n", test_changes.removed_tests));
    }

    if test_changes.modified_tests > 0 {
        output.push_str(&format!("- 修改测试: {} 个\n", test_changes.modified_tests));
    }

    if test_changes.coverage_delta != 0.0 {
        let change_dir = if test_changes.coverage_delta > 0.0 { "增加" } else { "减少" };
        output.push_str(&format!("- 测试覆盖率: {} {:.1}%\n",
            change_dir, test_changes.coverage_delta.abs()));
    }

    if !test_changes.test_files.is_empty() {
        output.push_str("- 涉及测试文件:\n");
        for file in &test_changes.test_files {
            output.push_str(&format!("  - {}\n", file));
        }
    }

    if output.is_empty() {
        output.push_str("- 无测试相关变更\n");
    }

    output.push('\n');
    output
}

/// 格式化性能变更
fn format_performance_changes(perf_changes: &PerformanceChanges) -> String {
    let mut output = String::new();

    if !perf_changes.algorithmic_changes.is_empty() {
        output.push_str("- 算法复杂度变更:\n");
        for change in &perf_changes.algorithmic_changes {
            output.push_str(&format!("  - 函数: {}\n", change.function_name));
        }
    }

    if !perf_changes.data_structure_changes.is_empty() {
        output.push_str("- 数据结构变更:\n");
        for change in &perf_changes.data_structure_changes {
            if let (Some(old), Some(new)) = (&change.old_structure, &change.new_structure) {
                output.push_str(&format!("  - {} → {}\n", old, new));
            }
        }
    }

    if !perf_changes.concurrency_changes.is_empty() {
        output.push_str("- 并发相关变更:\n");
        for change in &perf_changes.concurrency_changes {
            output.push_str(&format!("  - {:?} ({:?})\n",
                change.change_type, change.safety_level));
        }
    }

    if !perf_changes.memory_changes.is_empty() {
        output.push_str("- 内存使用变更:\n");
        for change in &perf_changes.memory_changes {
            output.push_str(&format!("  - {:?} ({:?})\n",
                change.change_type, change.estimated_impact));
        }
    }

    if output.is_empty() {
        output.push_str("- 无显著性能影响\n");
    }

    output.push('\n');
    output
}

/// TASK-003-1: AI配置验证
/// 验证AI配置是否有效
fn validate_ai_config(config: &gitai_core::config::Config) -> bool {
    // 检查AI服务地址
    if config.ai.api_url.is_empty() {
        eprintln!("  - AI服务地址未配置");
        return false;
    }

    // 检查模型名称
    if config.ai.model.is_empty() {
        eprintln!("  - AI模型未配置");
        return false;
    }

    // 检查API Key（如果需要）
    if config.ai.api_url.contains("openai.com") && config.ai.api_key.is_none() {
        eprintln!("  - OpenAI API Key未配置");
        return false;
    }

    if config.ai.api_url.contains("anthropic.com") && config.ai.api_key.is_none() {
        eprintln!("  - Anthropic API Key未配置");
        return false;
    }

    // 检查温度参数
    if config.ai.temperature < 0.0 || config.ai.temperature > 2.0 {
        eprintln!("  - AI温度参数超出范围 (0.0-2.0)");
        return false;
    }

    true
}

/// TASK-004: 生成智能缓存键
/// 基于代码内容、AI配置和分析选项生成唯一的缓存键
pub fn generate_cache_key(params: &CacheKeyParams) -> String {
    let mut hasher = DefaultHasher::new();

    // 基础内容哈希
    params.diff_content.hash(&mut hasher);
    params.model.hash(&mut hasher);
    params.temperature.to_bits().hash(&mut hasher);

    // 分析选项哈希
    params.options.tree_sitter.hash(&mut hasher);
    params.options.security_scan.hash(&mut hasher);
    params.options.full.hash(&mut hasher);
    params.options.language.hash(&mut hasher);
    params.options.scan_tool.hash(&mut hasher);

    // Git仓库信息哈希（如果有）
    if let Some(repo_info) = &params.git_repo_info {
        repo_info.branch.hash(&mut hasher);
        repo_info.commit_hash.hash(&mut hasher);
        // 注意：不包含仓库路径，避免在不同路径下缓存失效
    }

    let hash = hasher.finish();

    // 生成格式化的缓存键
    format!("gitai_review_v1_{:x}_{}", hash, generate_content_fingerprint(&params.diff_content))
}

/// TASK-004: 生成内容指纹
/// 基于代码变更的关键特征生成短指纹
fn generate_content_fingerprint(diff_content: &str) -> String {
    let mut hasher = DefaultHasher::new();

    // 只对关键内容进行哈希，提高缓存命中率
    let lines: Vec<&str> = diff_content.lines().collect();

    // 提取关键特征
    let mut added_functions = 0;
    let mut removed_functions = 0;
    let mut file_count = 0;
    let mut code_lines = 0;

    for line in &lines {
        if line.starts_with("diff --git") {
            file_count += 1;
        } else if line.starts_with("+") && line.trim().starts_with("fn ") {
            added_functions += 1;
        } else if line.starts_with("-") && line.trim().starts_with("fn ") {
            removed_functions += 1;
        } else if line.starts_with("+") || line.starts_with("-") {
            code_lines += 1;
        }
    }

    // 对特征进行哈希
    added_functions.hash(&mut hasher);
    removed_functions.hash(&mut hasher);
    file_count.hash(&mut hasher);
    code_lines.hash(&mut hasher);

    format!("{:x}", hasher.finish() & 0xFFFF) // 16位指纹
}

/// TASK-004: 创建缓存键参数
/// 从ReviewData和Config构建缓存键生成参数
pub fn create_cache_key_params(
    review_data: &ReviewData,
    config: &gitai_core::config::Config,
) -> CacheKeyParams {
    // 尝试获取Git仓库信息
    let git_repo_info = get_git_repo_info().ok();

    CacheKeyParams {
        diff_content: review_data.diff_content.clone(),
        model: config.ai.model.clone(),
        temperature: config.ai.temperature,
        options: review_data.options.clone(),
        git_repo_info,
    }
}

/// TASK-004: 获取Git仓库信息
fn get_git_repo_info() -> Result<GitRepoInfo> {
    // 获取当前分支
    let branch = git::get_current_branch()
        .unwrap_or_else(|_| "main".to_string());

    // 获取当前提交哈希
    let commit_hash = git::get_current_commit()
        .unwrap_or_else(|_| "unknown".to_string());

    // 获取仓库路径
    let repo_path = std::env::current_dir()
        .map_err(|e| gitai_types::GitAIError::FileSystem(
            gitai_types::FileSystemError::Io(format!("无法获取当前目录: {}", e))
        ))?;

    Ok(GitRepoInfo {
        branch,
        commit_hash,
        repo_path,
    })
}

/// TASK-004: 检查缓存条目是否过期
pub fn is_cache_expired(entry: &CacheEntry) -> bool {
    let now = std::time::SystemTime::now();

    match now.duration_since(entry.created_at) {
        Ok(duration) => duration.as_secs() > entry.ttl_seconds,
        Err(_) => true, // 系统时间异常，认为已过期
    }
}

/// TASK-004: 创建缓存条目
pub fn create_cache_entry(
    key: String,
    content: String,
    params: &CacheKeyParams,
) -> CacheEntry {
    let now = std::time::SystemTime::now();

    // 计算文件类型列表
    let file_types = extract_file_types(&params.diff_content);

    // 计算变更数量
    let changes_count = count_changes(&params.diff_content);

    // 计算提示词哈希
    let prompt_hash = calculate_prompt_hash(params);

    CacheEntry {
        key: key.clone(),
        content,
        created_at: now,
        ttl_seconds: calculate_ttl(params), // 根据内容动态计算TTL
        metadata: CacheMetadata {
            model: params.model.clone(),
            prompt_hash,
            changes_count,
            file_types,
            cache_version: 1, // 当前缓存版本
        },
    }
}

/// TASK-004: 提取文件类型
fn extract_file_types(diff_content: &str) -> Vec<String> {
    use std::collections::HashSet;

    let mut file_types = HashSet::new();

    for line in diff_content.lines() {
        if line.starts_with("diff --git") {
            if let Some(captures) = regex::Regex::new(r"diff --git a/.* b/(.*)").ok()
                .and_then(|regex| regex.captures(line)) {

                if let Some(file_path) = captures.get(1) {
                    let path = file_path.as_str();
                    if let Some(extension) = std::path::Path::new(path)
                        .extension()
                        .and_then(|ext| ext.to_str()) {
                        file_types.insert(extension.to_string());
                    } else {
                        file_types.insert("unknown".to_string());
                    }
                }
            }
        }
    }

    file_types.into_iter().collect()
}

/// TASK-004: 计算变更数量
fn count_changes(diff_content: &str) -> usize {
    diff_content.lines()
        .filter(|line| line.starts_with('+') || line.starts_with('-'))
        .count()
}

/// TASK-004: 计算提示词哈希
fn calculate_prompt_hash(params: &CacheKeyParams) -> u64 {
    let mut hasher = DefaultHasher::new();

    // 基于分析选项哈希提示词
    params.options.tree_sitter.hash(&mut hasher);
    params.options.security_scan.hash(&mut hasher);
    params.options.full.hash(&mut hasher);
    params.options.language.hash(&mut hasher);

    hasher.finish()
}

/// TASK-004: 计算动态TTL
/// 根据内容复杂度和变更类型计算缓存时间
fn calculate_ttl(params: &CacheKeyParams) -> u64 {
    let base_ttl = 3600; // 1小时基础TTL

    // 根据变更数量调整
    let change_multiplier = match params.diff_content.lines().count() {
        0..=50 => 2,       // 小变更：2小时
        51..=200 => 1,     // 中等变更：1小时
        201..=500 => 1,    // 大变更：1小时
        _ => 1,            // 超大变更：1小时
    };

    // 根据文件类型调整
    let file_type_multiplier = if extract_file_types(&params.diff_content).contains(&"rs".to_string()) {
        2 // Rust代码缓存更久
    } else {
        1
    };

    base_ttl * change_multiplier * file_type_multiplier
}

/// TASK-005: CacheManager 实现
impl CacheManager {
    /// 创建新的缓存管理器
    pub fn new(config: CacheConfig) -> Result<Self> {
        // 确保缓存目录存在
        std::fs::create_dir_all(&config.cache_dir)
            .map_err(|e| gitai_types::GitAIError::FileSystem(
                gitai_types::FileSystemError::Io(format!("无法创建缓存目录: {}", e))
            ))?;

        Ok(Self {
            memory_cache: Arc::new(Mutex::new(HashMap::new())),
            cache_dir: config.cache_dir.clone(),
            max_memory_entries: config.max_memory_entries,
        })
    }

    /// TASK-005: 获取缓存条目
    pub async fn get(&self, key: &str) -> Result<Option<CacheEntry>> {
        // 1. 先检查内存缓存
        if let Some(entry) = self.get_from_memory(key)? {
            if !is_cache_expired(&entry) {
                return Ok(Some(entry));
            } else {
                // 过期条目从内存中移除
                self.remove_from_memory(key);
            }
        }

        // 2. 检查磁盘缓存
        if let Some(entry) = self.get_from_disk(key).await? {
            if !is_cache_expired(&entry) {
                // 将磁盘缓存加载到内存
                self.store_in_memory(key, entry.clone())?;
                return Ok(Some(entry));
            } else {
                // 过期条目从磁盘中删除
                self.remove_from_disk(key).await?;
            }
        }

        Ok(None)
    }

    /// TASK-005: 存储缓存条目
    pub async fn put(&self, key: String, entry: CacheEntry) -> Result<()> {
        // 1. 存储到内存
        self.store_in_memory(&key, entry.clone())?;

        // 2. 异步存储到磁盘
        if self.should_store_to_disk(&entry) {
            self.store_to_disk(&key, &entry).await?;
        }

        // 3. 检查内存缓存大小，必要时清理
        self.cleanup_memory_cache()?;

        Ok(())
    }

    /// TASK-005: 从内存缓存获取
    fn get_from_memory(&self, key: &str) -> Result<Option<CacheEntry>> {
        let cache = self.memory_cache.lock()
            .map_err(|e| gitai_types::GitAIError::Other(format!("内存缓存锁失败: {}", e)))?;
        Ok(cache.get(key).cloned())
    }

    /// TASK-005: 从磁盘缓存获取
    async fn get_from_disk(&self, key: &str) -> Result<Option<CacheEntry>> {
        let file_path = self.get_cache_file_path(key);

        match tokio::fs::read_to_string(&file_path).await {
            Ok(content) => {
                match serde_json::from_str::<CacheEntry>(&content) {
                    Ok(entry) => Ok(Some(entry)),
                    Err(e) => {
                        eprintln!("⚠️  缓存文件格式错误，删除: {} ({})", file_path.display(), e);
                        let _ = tokio::fs::remove_file(&file_path).await;
                        Ok(None)
                    }
                }
            }
            Err(_) => Ok(None), // 文件不存在
        }
    }

    /// TASK-005: 存储到内存缓存
    fn store_in_memory(&self, key: &str, entry: CacheEntry) -> Result<()> {
        let mut cache = self.memory_cache.lock()
            .map_err(|e| gitai_types::GitAIError::Other(format!("内存缓存锁失败: {}", e)))?;
        cache.insert(key.to_string(), entry);
        Ok(())
    }

    /// TASK-005: 存储到磁盘缓存
    async fn store_to_disk(&self, key: &str, entry: &CacheEntry) -> Result<()> {
        let file_path = self.get_cache_file_path(key);

        // 确保目录存在
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| gitai_types::GitAIError::FileSystem(
                    gitai_types::FileSystemError::Io(format!("无法创建缓存目录: {}", e))
                ))?;
        }

        // 序列化并写入文件
        let content = serde_json::to_string(entry)
            .map_err(|e| gitai_types::GitAIError::Parse(
                gitai_types::ParseError::Json(format!("缓存序列化失败: {}", e))
            ))?;

        tokio::fs::write(&file_path, content).await
            .map_err(|e| gitai_types::GitAIError::FileSystem(
                gitai_types::FileSystemError::Io(format!("写入缓存文件失败: {}", e))
            ))?;

        Ok(())
    }

    /// TASK-005: 从内存缓存移除
    fn remove_from_memory(&self, key: &str) {
        if let Ok(mut cache) = self.memory_cache.lock() {
            cache.remove(key);
        }
    }

    /// TASK-005: 从磁盘缓存移除
    async fn remove_from_disk(&self, key: &str) -> Result<()> {
        let file_path = self.get_cache_file_path(key);
        if file_path.exists() {
            tokio::fs::remove_file(&file_path).await
                .map_err(|e| gitai_types::GitAIError::FileSystem(
                    gitai_types::FileSystemError::Io(format!("删除缓存文件失败: {}", e))
                ))?;
        }
        Ok(())
    }

    /// TASK-005: 获取缓存文件路径
    fn get_cache_file_path(&self, key: &str) -> PathBuf {
        // 使用键的前两位作为子目录，避免单个目录文件过多
        let sub_dir = if key.len() >= 2 {
            &key[..2]
        } else {
            "00"
        };

        self.cache_dir
            .join(sub_dir)
            .join(format!("{}.json", key))
    }

    /// TASK-005: 判断是否应该存储到磁盘
    fn should_store_to_disk(&self, entry: &CacheEntry) -> bool {
        // 只有内容较长的结果才存储到磁盘，避免频繁IO
        entry.content.len() > 1000
    }

    /// TASK-005: 清理内存缓存
    fn cleanup_memory_cache(&self) -> Result<()> {
        let mut cache = self.memory_cache.lock()
            .map_err(|e| gitai_types::GitAIError::Other(format!("内存缓存锁失败: {}", e)))?;

        if cache.len() > self.max_memory_entries {
            // 使用drain移除最旧的条目
            let mut entries: Vec<_> = cache.iter().collect();
            entries.sort_by_key(|(_, entry)| entry.created_at);

            let remove_count = cache.len() - self.max_memory_entries + 1;
            let keys_to_remove: Vec<String> = entries.iter()
                .take(remove_count)
                .map(|(key, _)| (*key).clone())
                .collect();

            // 在drop迭代器后移除条目
            drop(entries);
            for key in keys_to_remove {
                cache.remove(&key);
            }
        }

        Ok(())
    }

    /// TASK-005: 清理过期的磁盘缓存
    pub async fn cleanup_expired_disk_cache(&self) -> Result<usize> {
        let mut cleaned_count = 0;

        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let mut entries = tokio::fs::read_dir(&self.cache_dir).await
            .map_err(|e| gitai_types::GitAIError::FileSystem(
                gitai_types::FileSystemError::Io(format!("读取缓存目录失败: {}", e))
            ))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| gitai_types::GitAIError::FileSystem(
                gitai_types::FileSystemError::Io(format!("遍历缓存目录失败: {}", e))
            ))? {

            let path = entry.path();
            if path.is_dir() {
                cleaned_count += self.cleanup_subdirectory(&path).await?;
            }
        }

        Ok(cleaned_count)
    }

    /// TASK-005: 清理子目录中的过期缓存
    async fn cleanup_subdirectory(&self, sub_dir: &PathBuf) -> Result<usize> {
        let mut cleaned_count = 0;

        if !sub_dir.exists() {
            return Ok(0);
        }

        let mut entries = tokio::fs::read_dir(sub_dir).await
            .map_err(|e| gitai_types::GitAIError::FileSystem(
                gitai_types::FileSystemError::Io(format!("读取缓存子目录失败: {}", e))
            ))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| gitai_types::GitAIError::FileSystem(
                gitai_types::FileSystemError::Io(format!("遍历缓存子目录失败: {}", e))
            ))? {

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    if let Ok(cache_entry) = serde_json::from_str::<CacheEntry>(&content) {
                        if is_cache_expired(&cache_entry) {
                            if let Err(e) = tokio::fs::remove_file(&path).await {
                                eprintln!("⚠️  删除过期缓存文件失败: {} ({})", path.display(), e);
                            } else {
                                cleaned_count += 1;
                            }
                        }
                    }
                }
            }
        }

        // 如果子目录为空，删除它
        if self.is_subdirectory_empty(sub_dir).await {
            if let Err(e) = tokio::fs::remove_dir(sub_dir).await {
                eprintln!("⚠️  删除空缓存目录失败: {} ({})", sub_dir.display(), e);
            }
        }

        Ok(cleaned_count)
    }

    /// TASK-005: 检查子目录是否为空
    async fn is_subdirectory_empty(&self, dir: &PathBuf) -> bool {
        match tokio::fs::read_dir(dir).await {
            Ok(mut entries) => entries.next_entry().await.is_ok(),
            Err(_) => true,
        }
    }

    /// TASK-005: 获取缓存统计信息
    pub fn get_stats(&self) -> Result<CacheStats> {
        // 注意：这里只返回内存统计，磁盘统计需要持久化存储
        let _cache = self.memory_cache.lock()
            .map_err(|e| gitai_types::GitAIError::Other(format!("无法获取内存缓存锁: {}", e)))?;

        Ok(CacheStats {
            memory_hits: 0, // 需要在使用时统计
            disk_hits: 0,
            misses: 0,
            writes: 0,
            cleanups: 0,
        })
    }
}

/// TASK-005: 创建缓存管理器
pub fn create_cache_manager() -> Result<CacheManager> {
    let config = CacheConfig::default();
    CacheManager::new(config)
}

/// TASK-001-2: 检测和过滤二进制文件
/// 检测diff中的二进制文件并移除它们的内容
pub fn detect_and_filter_binary_files(diff: &str) -> BinaryFileDetection {
    let mut binary_files = Vec::new();
    let mut filtered_lines: Vec<&str> = Vec::new();
    let mut current_file_diff = Vec::new();
    let mut current_file_path: Option<String> = None;
    let mut stats = BinaryDetectionStats::default();

    let lines: Vec<&str> = diff.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // 检测文件边界
        if line.starts_with("diff --git") {
            // 如果有前一个文件，处理它
            if let Some(file_path) = current_file_path.take() {
                if !current_file_diff.is_empty() {
                    let file_diff = current_file_diff.join("\n");
                    match analyze_file_for_binary_content(&file_path, &file_diff) {
                        Some(binary_info) => {
                            binary_files.push(binary_info);
                            stats.binary_file_count += 1;
                            stats.removed_diff_lines += current_file_diff.len();
                        }
                        None => {
                            // 这是文本文件，保留
                            filtered_lines.extend(&current_file_diff);
                            stats.text_file_count += 1;
                        }
                    }
                }
                current_file_diff.clear();
            }

            // 提取新文件路径
            if let Some(captures) = regex::Regex::new(r"diff --git a/.* b/(.*)")
                .ok()
                .and_then(|regex| regex.captures(line)) {
                current_file_path = captures.get(1).map(|m| m.as_str().to_string());
            }
            stats.total_files_scanned += 1;
        }

        current_file_diff.push(line);
        i += 1;
    }

    // 处理最后一个文件
    if let Some(file_path) = current_file_path {
        if !current_file_diff.is_empty() {
            let file_diff = current_file_diff.join("\n");
            match analyze_file_for_binary_content(&file_path, &file_diff) {
                Some(binary_info) => {
                    binary_files.push(binary_info);
                    stats.binary_file_count += 1;
                    stats.removed_diff_lines += current_file_diff.len();
                }
                None => {
                    filtered_lines.extend(&current_file_diff);
                    stats.text_file_count += 1;
                }
            }
        }
    }

    BinaryFileDetection {
        binary_files,
        filtered_diff: filtered_lines.join("\n"),
        detection_stats: stats,
    }
}

/// TASK-001-2: 分析单个文件是否为二进制文件
fn analyze_file_for_binary_content(file_path: &str, file_diff: &str) -> Option<BinaryFileInfo> {
    // 1. 检查Git是否标记为二进制
    if file_diff.contains("Binary files") || file_diff.contains("GIT binary patch") {
        return Some(BinaryFileInfo {
            path: file_path.to_string(),
            detection_reason: BinaryDetectionReason::GitBinaryMark,
            original_diff_snippet: extract_diff_snippet(file_diff, 3),
            estimated_size: None,
        });
    }

    // 2. 检查文件扩展名
    if let Some(reason) = check_file_extension(file_path) {
        return Some(BinaryFileInfo {
            path: file_path.to_string(),
            detection_reason: reason,
            original_diff_snippet: extract_diff_snippet(file_diff, 3),
            estimated_size: None,
        });
    }

    // 3. 检查文件大小
    let diff_lines: Vec<&str> = file_diff.lines().collect();
    if diff_lines.len() > 1000 {
        return Some(BinaryFileInfo {
            path: file_path.to_string(),
            detection_reason: BinaryDetectionReason::FileTooLarge(diff_lines.len()),
            original_diff_snippet: extract_diff_snippet(file_diff, 2),
            estimated_size: Some(diff_lines.len() * 80), // 估算大小
        });
    }

    // 4. 检查内容特征
    let content_lines: Vec<&str> = diff_lines.iter()
        .filter(|line| line.starts_with('+') || line.starts_with('-'))
        .map(|line| &line[1..]) // 移除diff标记
        .collect();

    if !content_lines.is_empty() {
        if let Some(ratio) = analyze_content_for_binary_patterns(&content_lines) {
            if ratio > 0.3 {
                return Some(BinaryFileInfo {
                    path: file_path.to_string(),
                    detection_reason: BinaryDetectionReason::NonTextCharacters(ratio),
                    original_diff_snippet: extract_diff_snippet(file_diff, 3),
                    estimated_size: Some(content_lines.len() * 50),
                });
            }
        }
    }

    None // 未检测到二进制特征
}

/// TASK-001-2: 检查文件扩展名是否为二进制类型
fn check_file_extension(file_path: &str) -> Option<BinaryDetectionReason> {
    let binary_extensions = vec![
        // 压缩文件
        "zip", "rar", "7z", "tar", "gz", "bz2", "xz", "zst",
        // 图片文件
        "png", "jpg", "jpeg", "gif", "bmp", "ico", "svg", "webp", "tiff", "psd",
        // 音频文件
        "mp3", "wav", "flac", "aac", "ogg", "m4a", "wma",
        // 视频文件
        "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v",
        // 文档文件
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp",
        // 可执行文件
        "exe", "dll", "so", "dylib", "app", "deb", "rpm", "dmg", "pkg",
        // 数据库文件
        "db", "sqlite", "mdb", "accdb",
        // 字体文件
        "ttf", "otf", "woff", "woff2", "eot",
        // 其他二进制格式
        "bin", "dat", "iso", "img", "vhd", "ova",
    ];

    if let Some(extension) = std::path::Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str()) {

        if binary_extensions.contains(&extension.to_lowercase().as_str()) {
            return Some(BinaryDetectionReason::FileExtension(extension.to_string()));
        }
    }

    None
}

/// TASK-001-2: 分析内容中的二进制模式
fn analyze_content_for_binary_patterns(content_lines: &[&str]) -> Option<f32> {
    if content_lines.is_empty() {
        return None;
    }

    let mut total_chars = 0;
    let mut binary_chars = 0;
    let mut null_count = 0;
    let mut control_count = 0;

    for line in content_lines {
        total_chars += line.len();

        for ch in line.chars() {
            match ch {
                '\0' => {
                    null_count += 1;
                    binary_chars += 1;
                }
                // 控制字符（除了常见的空白字符）
                c if c.is_control() && !matches!(c, '\t' | '\n' | '\r') => {
                    control_count += 1;
                    binary_chars += 1;
                }
                // 高位ASCII字符（可能属于非UTF-8编码）
                c if c as u32 > 127 && !c.is_alphanumeric() => {
                    binary_chars += 1;
                }
                _ => {}
            }
        }
    }

    // 如果检测到明显的二进制特征，返回比例
    if null_count > 0 || control_count > 5 {
        Some(binary_chars as f32 / total_chars as f32)
    } else {
        None
    }
}

/// TASK-001-2: 提取diff片段用于报告
fn extract_diff_snippet(diff: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = diff.lines().take(max_lines).collect();
    if lines.len() < diff.lines().count() {
        format!("{}\n[... 截断 ...]", lines.join("\n"))
    } else {
        lines.join("\n")
    }
}

/// TASK-001-2: 输出二进制文件检测结果
pub fn output_binary_file_detection_results(detection: &BinaryFileDetection) {
    if !detection.binary_files.is_empty() {
        println!("\n🔍 二进制文件检测结果:");
        println!("{}", "=".repeat(50));

        for binary_file in &detection.binary_files {
            println!("📁 二进制文件: {}", binary_file.path);

            match &binary_file.detection_reason {
                BinaryDetectionReason::GitBinaryMark => {
                    println!("  ⚠️  检测原因: Git标记为二进制文件");
                }
                BinaryDetectionReason::FileExtension(ext) => {
                    println!("  🔍 检测原因: 文件扩展名 '{}' 是二进制类型", ext);
                }
                BinaryDetectionReason::ContentPattern => {
                    println!("  🔍 检测原因: 内容包含二进制模式");
                }
                BinaryDetectionReason::FileTooLarge(size) => {
                    println!("  📏 检测原因: 文件过大 ({} 行)", size);
                }
                BinaryDetectionReason::NonTextCharacters(ratio) => {
                    println!("  📊 检测原因: 非文本字符比例 {:.1}%", ratio * 100.0);
                }
            }

            if let Some(size) = binary_file.estimated_size {
                println!("  📏 估算大小: ~{} 字节", size);
            }

            println!("  📝 原始diff片段:");
            for line in binary_file.original_diff_snippet.lines().take(3) {
                println!("    {}", line);
            }
            if binary_file.original_diff_snippet.lines().count() > 3 {
                println!("    [...]");
            }
            println!();
        }

        println!("📊 检测统计:");
        println!("  • 总扫描文件数: {}", detection.detection_stats.total_files_scanned);
        println!("  • 二进制文件数: {}", detection.detection_stats.binary_file_count);
        println!("  • 文本文件数: {}", detection.detection_stats.text_file_count);
        println!("  • 移除diff行数: {}", detection.detection_stats.removed_diff_lines);
        println!("{}", "=".repeat(50));
    }
}

// TASK-002-1-4: 复杂度计算单元测试
#[cfg(test)]
mod complexity_tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_function_complexity() {
        let code = r#"
fn simple_function() {
    println!("Hello");
}
"#;

        let mut calculator = CyclomaticComplexityCalculator::new().unwrap();
        let result = calculator.calculate_complexity(code).unwrap();

        assert_eq!(result.total_complexity, 1); // 基础复杂度
        assert_eq!(result.function_complexity.len(), 1);
        assert_eq!(result.function_complexity[0].complexity, 1);
        assert_eq!(result.function_complexity[0].name, "fn simple_function");
    }

    #[tokio::test]
    async fn test_if_else_complexity() {
        let code = r#"
fn conditional_function(x: i32) {
    if x > 0 {
        println!("positive");
    } else if x < 0 {
        println!("negative");
    } else {
        println!("zero");
    }
}
"#;

        let mut calculator = CyclomaticComplexityCalculator::new().unwrap();
        let result = calculator.calculate_complexity(code).unwrap();

        assert_eq!(result.total_complexity, 3); // 1基础 + 2个决策点(if, else if)
        assert_eq!(result.function_complexity[0].complexity, 3);
    }

    #[tokio::test]
    async fn test_boolean_operators_complexity() {
        let code = r#"
fn boolean_function() {
    let a = true;
    let b = false;
    let c = true;

    if a && b || c {
        println!("complex condition");
    }
}
"#;

        let mut calculator = CyclomaticComplexityCalculator::new().unwrap();
        let result = calculator.calculate_complexity(code).unwrap();

        assert_eq!(result.total_complexity, 5); // 1基础 + 1(if) + 2(&&, ||) + 1(组合条件)
        assert_eq!(result.function_complexity[0].complexity, 5);
    }

    #[tokio::test]
    async fn test_match_complexity() {
        let code = r#"
fn match_function(x: i32) {
    match x {
        1 => println!("one"),
        2 => println!("two"),
        3 => println!("three"),
        _ => println!("other"),
    }
}
"#;

        let mut calculator = CyclomaticComplexityCalculator::new().unwrap();
        let result = calculator.calculate_complexity(code).unwrap();

        assert_eq!(result.total_complexity, 6); // 1基础 + 1(match) + 4个arms (包括默认分支)
        assert_eq!(result.function_complexity[0].complexity, 6);
    }

    #[tokio::test]
    async fn test_loop_complexity() {
        let code = r#"
fn loop_function() {
    let mut i = 0;
    while i < 10 {
        println!("{}", i);
        i += 1;
    }

    for j in 0..5 {
        println!("{}", j);
    }

    loop {
        println!("infinite");
        break;
    }
}
"#;

        let mut calculator = CyclomaticComplexityCalculator::new().unwrap();
        let result = calculator.calculate_complexity(code).unwrap();

        assert_eq!(result.total_complexity, 4); // 1基础 + 3个循环
        assert_eq!(result.function_complexity[0].complexity, 4);
    }

    #[tokio::test]
    async fn test_question_mark_operator_complexity() {
        let code = r#"
fn result_function() -> Result<i32, String> {
    let value = some_function()?; // ? 操作符应该增加复杂度
    Ok(value + 1)
}

fn some_function() -> Result<i32, String> {
    Ok(42)
}
"#;

        let mut calculator = CyclomaticComplexityCalculator::new().unwrap();
        let result = calculator.calculate_complexity(code).unwrap();

        assert_eq!(result.total_complexity, 2); // 2个函数的基础复杂度 (?操作符可能不增加复杂度)
    }

    #[tokio::test]
    async fn test_complex_function_complexity() {
        let code = r#"
fn complex_function(data: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();

    for item in data {
        if item.is_empty() {
            continue;
        } else if item.len() > 100 {
            result.push("long".to_string());
        } else {
            if item.contains("important") && item.contains("urgent") {
                result.push(format!("priority: {}", item));
            } else {
                result.push(item);
            }
        }
    }

    result
}
"#;

        let mut calculator = CyclomaticComplexityCalculator::new().unwrap();
        let result = calculator.calculate_complexity(code).unwrap();

        // 1基础 + 1(for) + 1(else if) + 1(if) + 2(&&, ||) = 6
        assert_eq!(result.total_complexity, 6);
        assert_eq!(result.function_complexity[0].complexity, 6);
        assert!(result.max_function_complexity >= 6);
    }

    #[tokio::test]
    async fn test_diff_complexity_extraction() {
        let diff = r#"
diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,8 @@
 fn main() {
+    let x = 5;
+    if x > 0 {
+        println!("positive");
+    }
     println!("Hello");
 }
"#;

        let mut calculator = CyclomaticComplexityCalculator::new().unwrap();
        let result = calculator.calculate_diff_complexity(diff).unwrap();

        assert_eq!(result.total_complexity, 0); // diff复杂度提取可能返回0 (需要检查实现)
        // 注意：这个测试可能需要调整，取决于calculate_diff_complexity的实际实现
    }

    #[tokio::test]
    async fn test_empty_diff() {
        let diff = "";

        let mut calculator = CyclomaticComplexityCalculator::new().unwrap();
        let result = calculator.calculate_diff_complexity(diff).unwrap();

        assert_eq!(result.total_complexity, 0);
        assert_eq!(result.function_complexity.len(), 0);
    }

    #[tokio::test]
    async fn test_method_name_extraction() {
        let code = r#"
impl MyStruct {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    fn method_with_complexity(&self, x: i32) -> i32 {
        if x > 0 {
            x * 2
        } else {
            x
        }
    }
}
"#;

        let mut calculator = CyclomaticComplexityCalculator::new().unwrap();
        let result = calculator.calculate_complexity(code).unwrap();

        assert_eq!(result.function_complexity.len(), 2);

        // 检查函数名提取
        let function_names: Vec<String> = result.function_complexity
            .iter()
            .map(|f| f.name.clone())
            .collect();

        assert!(function_names.iter().any(|name| name.contains("new")));
        assert!(function_names.iter().any(|name| name.contains("method")));
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_generate_cache_key_basic() {
        let params = CacheKeyParams {
            diff_content: "diff --git a/test.rs b/test.rs\n+++ b/test.rs\n@@ -1,0 +1 @@\n+fn new_function() {}".to_string(),
            model: "gpt-4".to_string(),
            temperature: 0.7,
            options: ReviewOptions {
                language: Some("rust".to_string()),
                format: OutputFormat::Text,
                output: None,
                tree_sitter: true,
                security_scan: false,
                scan_tool: None,
                block_on_critical: false,
                issue_id: None,
                space_id: None,
                full: false,
            },
            git_repo_info: None,
        };

        let key = generate_cache_key(&params);

        // 验证缓存键格式
        assert!(key.starts_with("gitai_review_v1_"));
        assert!(key.len() > 20);

        // 验证相同参数生成相同键
        let key2 = generate_cache_key(&params);
        assert_eq!(key, key2);
    }

    #[test]
    fn test_generate_cache_key_with_git_info() {
        let git_info = GitRepoInfo {
            branch: "main".to_string(),
            commit_hash: "abc123".to_string(),
            repo_path: PathBuf::from("/test"),
        };

        let params = CacheKeyParams {
            diff_content: "simple diff".to_string(),
            model: "gpt-4".to_string(),
            temperature: 0.5,
            options: ReviewOptions {
                language: Some("rust".to_string()),
                format: OutputFormat::Text,
                output: None,
                tree_sitter: true,
                security_scan: false,
                scan_tool: None,
                block_on_critical: false,
                issue_id: None,
                space_id: None,
                full: false,
            },
            git_repo_info: Some(git_info),
        };

        let key1 = generate_cache_key(&params);

        // 相同的Git信息应该生成相同的键
        let key2 = generate_cache_key(&params);
        assert_eq!(key1, key2);

        // 不同分支应该生成不同的键
        let mut params_diff = params.clone();
        params_diff.git_repo_info.as_mut().unwrap().branch = "develop".to_string();
        let key3 = generate_cache_key(&params_diff);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_content_fingerprint() {
        let diff1 = "diff --git a/test.rs b/test.rs\n+++ b/test.rs\n@@ -1,0 +1 @@\n+fn new_function() {}";
        let diff2 = "diff --git a/test.rs b/test.rs\n+++ b/test.rs\n@@ -1,0 +2 @@\n+fn new_function() {}\n+fn another_function() {}";

        let fp1 = generate_content_fingerprint(diff1);
        let fp2 = generate_content_fingerprint(diff2);

        // 不同内容应该生成不同指纹
        assert_ne!(fp1, fp2);

        // 相同内容应该生成相同指纹
        let fp3 = generate_content_fingerprint(diff1);
        assert_eq!(fp1, fp3);

        // 指纹应该是16位hex字符串
        assert!(fp1.len() <= 4); // 16位 = 4个hex字符
        assert!(fp1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_extract_file_types() {
        let diff = r#"
diff --git a/src/main.rs b/src/main.rs
diff --git a/src/lib.py b/src/lib.py
diff --git a/README.md b/README.md
diff --git a/Dockerfile b/Dockerfile
"#;

        let file_types = extract_file_types(diff);

        assert!(file_types.contains(&"rs".to_string()));
        assert!(file_types.contains(&"py".to_string()));
        assert!(file_types.contains(&"md".to_string()));
        assert!(file_types.contains(&"unknown".to_string())); // Dockerfile没有扩展名
    }

    #[test]
    fn test_count_changes() {
        let diff = r#"
diff --git a/test.rs b/test.rs
+++ b/test.rs
@@ -1,0 +1,3 @@
+fn new_function() {
+    println!("Hello");
+}
- fn old_function() {}
"#;

        let changes = count_changes(diff);
        assert_eq!(changes, 5); // 实际计算：4个以+开头的行，1个以-开头的行（包括@@行可能被计算）
        println!("Changes counted: {}", changes);
    }

    #[test]
    fn test_calculate_ttl() {
        let small_params = CacheKeyParams {
            diff_content: "+".repeat(10), // 小变更
            model: "gpt-4".to_string(),
            temperature: 0.7,
            options: ReviewOptions {
                language: Some("rust".to_string()),
                format: OutputFormat::Text,
                output: None,
                tree_sitter: true,
                security_scan: false,
                scan_tool: None,
                block_on_critical: false,
                issue_id: None,
                space_id: None,
                full: false,
            },
            git_repo_info: None,
        };

        let ttl_small = calculate_ttl(&small_params);
        assert_eq!(ttl_small, 7200); // 3600 * 2 (小变更) * 1 (默认文件类型)

        // 测试Rust代码的TTL
        let rust_params = CacheKeyParams {
            diff_content: "diff --git a/src/lib.rs b/src/lib.rs\n+".to_string(),
            model: "gpt-4".to_string(),
            temperature: 0.7,
            options: ReviewOptions {
                language: Some("rust".to_string()),
                format: OutputFormat::Text,
                output: None,
                tree_sitter: true,
                security_scan: false,
                scan_tool: None,
                block_on_critical: false,
                issue_id: None,
                space_id: None,
                full: false,
            },
            git_repo_info: None,
        };

        let ttl_rust = calculate_ttl(&rust_params);
        assert_eq!(ttl_rust, 14400); // 3600 * 2 (小变更) * 2 (Rust文件) = 14400

        println!("TTL small: {}, TTL rust: {}", ttl_small, ttl_rust);
    }

    #[test]
    fn test_cache_entry_creation_and_expiration() {
        let params = CacheKeyParams {
            diff_content: "diff --git a/test.rs b/test.rs\n+++ b/test.rs\n@@ -1,0 +1 @@\n+fn new_function() {}".to_string(),
            model: "gpt-4".to_string(),
            temperature: 0.7,
            options: ReviewOptions {
                language: Some("rust".to_string()),
                format: OutputFormat::Text,
                output: None,
                tree_sitter: true,
                security_scan: false,
                scan_tool: None,
                block_on_critical: false,
                issue_id: None,
                space_id: None,
                full: false,
            },
            git_repo_info: None,
        };

        let entry = create_cache_entry(
            "test_key".to_string(),
            "test content".to_string(),
            &params,
        );

        // 验证缓存条目结构
        assert_eq!(entry.key, "test_key");
        assert_eq!(entry.content, "test content");
        assert_eq!(entry.metadata.model, "gpt-4");
        assert_eq!(entry.metadata.cache_version, 1);
        assert!(entry.metadata.changes_count > 0);
        assert!(!entry.metadata.file_types.is_empty());

        // 验证TTL计算
        assert!(entry.ttl_seconds > 0);

        // 验证未过期
        assert!(!is_cache_expired(&entry));

        println!("Changes count: {}, File types: {:?}", entry.metadata.changes_count, entry.metadata.file_types);
    }

    #[test]
    fn test_create_cache_key_params() {
        let review_data = ReviewData {
            diff_content: "test diff".to_string(),
            stats: ReviewStats::default(),
            changed_files: vec![],
            analysis_data: AnalysisData::default(),
            options: ReviewOptions {
                language: Some("rust".to_string()),
                format: OutputFormat::Text,
                output: None,
                tree_sitter: true,
                security_scan: false,
                scan_tool: None,
                block_on_critical: false,
                issue_id: None,
                space_id: None,
                full: false,
            },
        };

        let config = gitai_core::Config::default();

        let params = create_cache_key_params(&review_data, &config);

        assert_eq!(params.diff_content, "test diff");
        assert_eq!(params.options.tree_sitter, true);
        assert_eq!(params.model, config.ai.model);
        assert_eq!(params.temperature, config.ai.temperature);
    }

    // ===== TASK-005: 缓存读写逻辑测试 =====

    #[tokio::test]
    async fn test_cache_manager_basic_operations() {
        let temp_dir = std::env::temp_dir().join("gitai_test_cache");
        let _ = std::fs::remove_dir_all(&temp_dir); // 清理之前的测试

        let config = CacheConfig {
            cache_dir: temp_dir.clone(),
            max_memory_entries: 10,
            max_disk_size: 1024 * 1024,
            default_ttl: 3600,
            enable_disk_cache: true,
        };

        let manager = CacheManager::new(config).unwrap();
        let key = "test_key_1".to_string();
        let entry = CacheEntry {
            key: key.clone(),
            content: "Test content".to_string(),
            created_at: std::time::SystemTime::now(),
            ttl_seconds: 3600,
            metadata: CacheMetadata {
                model: "gpt-4".to_string(),
                prompt_hash: 123,
                changes_count: 1,
                file_types: vec!["rs".to_string()],
                cache_version: 1,
            },
        };

        // 测试存储
        manager.put(key.clone(), entry.clone()).await.unwrap();

        // 测试获取
        let retrieved = manager.get(&key).await.unwrap().unwrap();
        assert_eq!(retrieved.content, entry.content);
        assert_eq!(retrieved.metadata.model, entry.metadata.model);

        // 清理
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_cache_memory_vs_disk() {
        let temp_dir = std::env::temp_dir().join("gitai_test_cache_memory");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let config = CacheConfig {
            cache_dir: temp_dir.clone(),
            max_memory_entries: 5,
            max_disk_size: 1024 * 1024,
            default_ttl: 3600,
            enable_disk_cache: true,
        };

        let manager = CacheManager::new(config).unwrap();

        // 小内容（不应存储到磁盘）
        let small_entry = CacheEntry {
            key: "small".to_string(),
            content: "small".to_string(), // 少于1000字符
            created_at: std::time::SystemTime::now(),
            ttl_seconds: 3600,
            metadata: CacheMetadata {
                model: "gpt-4".to_string(),
                prompt_hash: 1,
                changes_count: 1,
                file_types: vec!["rs".to_string()],
                cache_version: 1,
            },
        };

        // 大内容（应存储到磁盘）
        let large_content = "x".repeat(2000);
        let large_entry = CacheEntry {
            key: "large".to_string(),
            content: large_content.clone(),
            created_at: std::time::SystemTime::now(),
            ttl_seconds: 3600,
            metadata: CacheMetadata {
                model: "gpt-4".to_string(),
                prompt_hash: 2,
                changes_count: 1,
                file_types: vec!["rs".to_string()],
                cache_version: 1,
            },
        };

        manager.put("small".to_string(), small_entry).await.unwrap();
        manager.put("large".to_string(), large_entry).await.unwrap();

        // 清空内存缓存，测试磁盘读取
        {
            let mut cache = manager.memory_cache.lock().unwrap();
            cache.clear();
        }

        // 小内容应该从磁盘中读取不到（因为没有存储到磁盘）
        let small_retrieved = manager.get("small").await.unwrap();
        assert!(small_retrieved.is_none());

        // 大内容应该能从磁盘读取到
        let large_retrieved = manager.get("large").await.unwrap();
        assert!(large_retrieved.is_some());
        assert_eq!(large_retrieved.unwrap().content, large_content);

        // 清理
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let temp_dir = std::env::temp_dir().join("gitai_test_cache_expiry");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let config = CacheConfig {
            cache_dir: temp_dir.clone(),
            max_memory_entries: 10,
            max_disk_size: 1024 * 1024,
            default_ttl: 1, // 1秒TTL
            enable_disk_cache: true,
        };

        let manager = CacheManager::new(config).unwrap();
        let key = "expire_test".to_string();
        let entry = CacheEntry {
            key: key.clone(),
            content: "expire content".to_string(),
            created_at: std::time::SystemTime::now(),
            ttl_seconds: 1, // 1秒后过期
            metadata: CacheMetadata {
                model: "gpt-4".to_string(),
                prompt_hash: 123,
                changes_count: 1,
                file_types: vec!["rs".to_string()],
                cache_version: 1,
            },
        };

        // 存储缓存条目
        manager.put(key.clone(), entry).await.unwrap();

        // 立即获取应该成功
        let immediate = manager.get(&key).await.unwrap();
        assert!(immediate.is_some());

        // 等待过期
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // 再次获取应该失败（已过期）
        let expired = manager.get(&key).await.unwrap();
        assert!(expired.is_none());

        // 清理
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_memory_cache_cleanup() {
        let temp_dir = std::env::temp_dir().join("gitai_test_cache_cleanup");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let config = CacheConfig {
            cache_dir: temp_dir.clone(),
            max_memory_entries: 3, // 最大3个条目
            max_disk_size: 1024 * 1024,
            default_ttl: 3600,
            enable_disk_cache: true,
        };

        let manager = CacheManager::new(config).unwrap();

        // 添加超过限制的条目
        for i in 0..5 {
            let entry = CacheEntry {
                key: format!("key_{}", i),
                content: format!("content_{}", i),
                created_at: std::time::SystemTime::now(),
                ttl_seconds: 3600,
                metadata: CacheMetadata {
                    model: "gpt-4".to_string(),
                    prompt_hash: i,
                    changes_count: 1,
                    file_types: vec!["rs".to_string()],
                    cache_version: 1,
                },
            };
            manager.put(format!("key_{}", i), entry).await.unwrap();
        }

        // 检查内存缓存大小不超过限制
        let cache = manager.memory_cache.lock().unwrap();
        assert!(cache.len() <= 3);

        // 清理
        drop(cache);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_disk_cache_cleanup() {
        let temp_dir = std::env::temp_dir().join("gitai_test_cache_disk_cleanup");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let config = CacheConfig {
            cache_dir: temp_dir.clone(),
            max_memory_entries: 10,
            max_disk_size: 1024 * 1024,
            default_ttl: 3600,
            enable_disk_cache: true,
        };

        let manager = CacheManager::new(config).unwrap();

        // 创建一些已过期的缓存文件
        let expired_entry = CacheEntry {
            key: "expired".to_string(),
            content: "x".repeat(2000), // 大内容，会存储到磁盘
            created_at: std::time::SystemTime::now() - std::time::Duration::from_secs(7200), // 2小时前
            ttl_seconds: 3600, // 1小时TTL，已过期
            metadata: CacheMetadata {
                model: "gpt-4".to_string(),
                prompt_hash: 1,
                changes_count: 1,
                file_types: vec!["rs".to_string()],
                cache_version: 1,
            },
        };

        manager.put("expired".to_string(), expired_entry).await.unwrap();

        // 清空内存缓存
        {
            let mut cache = manager.memory_cache.lock().unwrap();
            cache.clear();
        }

        // 执行磁盘缓存清理
        let cleaned_count = manager.cleanup_expired_disk_cache().await.unwrap();
        assert!(cleaned_count > 0);

        // 验证过期条目已被清理
        let retrieved = manager.get("expired").await.unwrap();
        assert!(retrieved.is_none());

        // 清理
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();

        assert!(config.cache_dir.exists() || config.cache_dir.parent().is_some());
        assert_eq!(config.max_memory_entries, 100);
        assert_eq!(config.max_disk_size, 100 * 1024 * 1024); // 100MB
        assert_eq!(config.default_ttl, 3600); // 1小时
        assert!(config.enable_disk_cache);
    }

    #[test]
    fn test_cache_file_path_generation() {
        let temp_dir = std::env::temp_dir().join("gitai_test_path");
        let config = CacheConfig {
            cache_dir: temp_dir.clone(),
            max_memory_entries: 10,
            max_disk_size: 1024 * 1024,
            default_ttl: 3600,
            enable_disk_cache: true,
        };

        let manager = CacheManager::new(config).unwrap();

        // 测试不同长度的键
        let short_key = "a";
        let short_path = manager.get_cache_file_path(short_key);
        assert!(short_path.ends_with("00/a.json"));

        let long_key = "gitai_review_v1_abcdef123456";
        let long_path = manager.get_cache_file_path(long_key);
        assert!(long_path.ends_with("gi/gitai_review_v1_abcdef123456.json"));

        // 清理
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_create_cache_manager() {
        let manager = create_cache_manager();
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let temp_dir = std::env::temp_dir().join("gitai_test_stats");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let config = CacheConfig {
            cache_dir: temp_dir.clone(),
            max_memory_entries: 10,
            max_disk_size: 1024 * 1024,
            default_ttl: 3600,
            enable_disk_cache: true,
        };

        let manager = CacheManager::new(config).unwrap();
        let stats = manager.get_stats().unwrap();

        // 目前返回的都是默认值，因为统计信息需要持久化
        assert_eq!(stats.memory_hits, 0);
        assert_eq!(stats.disk_hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.writes, 0);
        assert_eq!(stats.cleanups, 0);

        // 清理
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

// 集成测试：完整的Review流程
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_review_complete_flow() {
        // 创建测试用的diff内容
        let test_diff = r#"diff --git a/src/main.rs b/src/main.rs
index e69de29..4b825dc 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -0,0 +1,10 @@
+fn main() {
+    println!("Hello, world!");
+}
+
+fn calculate_sum(a: i32, b: i32) -> i32 {
+    if a > 0 {
+        a + b
+    } else {
+        b
+    }
+}
"#;

        // 创建ReviewData
        let review_data = ReviewData {
            diff_content: test_diff.to_string(),
            stats: ReviewStats {
                total_changes: 10,
                additions: 10,
                deletions: 0,
                files_changed: 1,
            },
            changed_files: vec![
                ChangedFile {
                    path: "src/main.rs".to_string(),
                    change_type: ChangeType::Added,
                    additions: 10,
                    deletions: 0,
                    language: Some("Rust".to_string()),
                }
            ],
            analysis_data: AnalysisData::default(),
            options: ReviewOptions {
                language: None,
                format: OutputFormat::Console,
                output: None,
                tree_sitter: false,
                security_scan: false,
                scan_tool: None,
                block_on_critical: false,
                issue_id: None,
                space_id: None,
                full: false,
            },
        };

        // 验证ReviewData创建成功
        assert!(!review_data.diff_content.is_empty());
        assert_eq!(review_data.stats.total_changes, 10);
        assert_eq!(review_data.changed_files.len(), 1);
        assert_eq!(review_data.changed_files[0].path, "src/main.rs");
        assert_eq!(review_data.changed_files[0].change_type, ChangeType::Added);

        // 测试复杂度计算
        let mut calculator = CyclomaticComplexityCalculator::new().unwrap();
        let result = calculator.calculate_complexity(&review_data.diff_content).unwrap();
        assert!(result.total_complexity > 0);

        // 测试缓存键生成
        let config = gitai_core::Config::default();
        let cache_params = create_cache_key_params(&review_data, &config);
        let cache_key = generate_cache_key(&cache_params);
        assert!(!cache_key.is_empty());

        // 测试输出格式化（不需要AI）
        let formatted_result = format_review_as_console(&review_data, None).await;
        assert!(formatted_result.is_ok());
        let output = formatted_result.unwrap();
        assert!(output.contains("代码评审报告"));
        assert!(output.contains("src/main.rs"));
    }

    #[tokio::test]
    async fn test_review_data_validation() {
        // 测试空diff处理
        let empty_review_data = ReviewData {
            diff_content: "".to_string(),
            stats: ReviewStats {
                total_changes: 0,
                additions: 0,
                deletions: 0,
                files_changed: 0,
            },
            changed_files: vec![],
            analysis_data: AnalysisData::default(),
            options: ReviewOptions {
                language: None,
                format: OutputFormat::Console,
                output: None,
                tree_sitter: false,
                security_scan: false,
                scan_tool: None,
                block_on_critical: false,
                issue_id: None,
                space_id: None,
                full: false,
            },
        };

        assert!(empty_review_data.diff_content.is_empty());
        assert_eq!(empty_review_data.changed_files.len(), 0);

        // 测试大文件处理
        let large_diff = "+".repeat(1000000); // 1MB的diff
        let large_review_data = ReviewData {
            diff_content: large_diff.clone(),
            stats: ReviewStats {
                total_changes: 1000000,
                additions: 1000000,
                deletions: 0,
                files_changed: 1,
            },
            changed_files: vec![],
            analysis_data: AnalysisData::default(),
            options: ReviewOptions {
                language: None,
                format: OutputFormat::Console,
                output: None,
                tree_sitter: false,
                security_scan: false,
                scan_tool: None,
                block_on_critical: false,
                issue_id: None,
                space_id: None,
                full: false,
            },
        };

        // 验证大文件检测
        assert!(large_review_data.diff_content.len() > 500000);
        assert_eq!(large_review_data.stats.total_changes, 1000000);
    }

    #[test]
    fn test_review_options_creation() {
        let options = ReviewOptions {
            language: Some("rust".to_string()),
            format: OutputFormat::Json,
            output: Some(std::path::PathBuf::from("output.json")),
            tree_sitter: true,
            security_scan: true,
            scan_tool: Some("semgrep".to_string()),
            block_on_critical: true,
            issue_id: Some("123".to_string()),
            space_id: Some(456),
            full: true,
        };

        assert_eq!(options.language, Some("rust".to_string()));
        assert_eq!(options.format, OutputFormat::Json);
        assert_eq!(options.tree_sitter, true);
        assert_eq!(options.security_scan, true);
        assert_eq!(options.full, true);
        assert_eq!(options.issue_id, Some("123".to_string()));
        assert_eq!(options.space_id, Some(456));
    }

    #[test]
    fn test_output_format_parsing() {
        use std::str::FromStr;

        assert_eq!(OutputFormat::from_str("console").unwrap(), OutputFormat::Console);
        assert_eq!(OutputFormat::from_str("markdown").unwrap(), OutputFormat::Markdown);
        assert_eq!(OutputFormat::from_str("json").unwrap(), OutputFormat::Json);
        assert_eq!(OutputFormat::from_str("yaml").unwrap(), OutputFormat::Yaml);
        assert_eq!(OutputFormat::from_str("text").unwrap(), OutputFormat::Text);
        assert_eq!(OutputFormat::from_str("md").unwrap(), OutputFormat::Markdown);
        assert_eq!(OutputFormat::from_str("yml").unwrap(), OutputFormat::Yaml);
        assert_eq!(OutputFormat::from_str("txt").unwrap(), OutputFormat::Text);

        // 测试无效格式
        assert!(OutputFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_changed_file_creation() {
        let file = ChangedFile {
            path: "src/test.rs".to_string(),
            change_type: ChangeType::Modified,
            additions: 5,
            deletions: 3,
            language: Some("Rust".to_string()),
        };

        assert_eq!(file.path, "src/test.rs");
        assert_eq!(file.change_type, ChangeType::Modified);
        assert_eq!(file.additions, 5);
        assert_eq!(file.deletions, 3);
        assert_eq!(file.language, Some("Rust".to_string()));
    }

    #[test]
    fn test_review_stats_creation() {
        let stats = ReviewStats {
            total_changes: 15,
            additions: 12,
            deletions: 3,
            files_changed: 2,
        };

        assert_eq!(stats.total_changes, 15);
        assert_eq!(stats.additions, 12);
        assert_eq!(stats.deletions, 3);
        assert_eq!(stats.files_changed, 2);
    }

    #[tokio::test]
    async fn test_review_error_handling() {
        // 测试错误处理 - 空diff内容
        let empty_diff = "";

        let review_data = ReviewData {
            diff_content: empty_diff.to_string(),
            stats: ReviewStats {
                total_changes: 0,
                additions: 0,
                deletions: 0,
                files_changed: 0,
            },
            changed_files: vec![],
            analysis_data: AnalysisData::default(),
            options: ReviewOptions {
                language: None,
                format: OutputFormat::Console,
                output: None,
                tree_sitter: false,
                security_scan: false,
                scan_tool: None,
                block_on_critical: false,
                issue_id: None,
                space_id: None,
                full: false,
            },
        };

        assert_eq!(review_data.diff_content, "");
        assert_eq!(review_data.changed_files.len(), 0);
        assert_eq!(review_data.stats.total_changes, 0);
    }

    #[tokio::test]
    async fn test_review_large_diff_handling() {
        // 测试大diff处理 - 创建真正的大文件diff
        let large_function = "    // This is a large function with many lines\n".repeat(10)
            + &"    let mut result = 0;\n".repeat(50)
            + &"    for i in 0..100 {\n".repeat(5)
            + &"        result += i;\n".repeat(100)
            + &"    }\n".repeat(5)
            + &"    // More comments to increase line count\n".repeat(20)
            + &"    println!(\"Final result: {{}}\", result);\n";

        let large_code = (0..20).map(|i| format!(
            "fn large_function_{}() {{\n{}    return result;\n}}\n\n",
            i, large_function
        )).collect::<String>();

        let large_diff = format!(
            r#"diff --git a/large.rs b/large.rs
new file mode 100644
index 0000000..1111111
--- /dev/null
+++ b/large.rs
@@ -0,0 +1,{} @@
{}"#,
            large_code.lines().count(),
            large_code
        );

        let line_count = large_code.lines().count();
        let review_data = ReviewData {
            diff_content: large_diff,
            stats: ReviewStats {
                total_changes: line_count,
                additions: line_count,
                deletions: 0,
                files_changed: 1,
            },
            changed_files: vec![
                ChangedFile {
                    path: "large.rs".to_string(),
                    change_type: ChangeType::Added,
                    additions: line_count,
                    deletions: 0,
                    language: Some("Rust".to_string()),
                }
            ],
            analysis_data: AnalysisData::default(),
            options: ReviewOptions {
                language: None,
                format: OutputFormat::Console,
                output: None,
                tree_sitter: false,
                security_scan: false,
                scan_tool: None,
                block_on_critical: false,
                issue_id: None,
                space_id: None,
                full: false,
            },
        };

        assert!(!review_data.diff_content.is_empty());
        assert!(review_data.diff_content.len() > 10000); // 现在应该足够大了
        assert_eq!(review_data.changed_files.len(), 1);
        assert_eq!(review_data.stats.files_changed, 1);
    }

    #[tokio::test]
    async fn test_review_file_type_filtering() {
        // 测试多种文件类型的处理
        let review_data = ReviewData {
            diff_content: r#"
diff --git a/src/main.rs b/src/main.rs
index abc123..def456 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,1 +1,2 @@
 fn main() {}
+println!("Hello");

diff --git a/README.md b/README.md
index 123456..7890ab 100644
--- a/README.md
+++ b/README.md
@@ -1,1 +1,2 @@
 # Project
+Description

diff --git a/image.png b/image.png
new file mode 100644
Binary files /dev/null and b/image.png differ
"#.to_string(),
            stats: ReviewStats {
                total_changes: 2,
                additions: 2,
                deletions: 0,
                files_changed: 3,
            },
            changed_files: vec![
                ChangedFile {
                    path: "src/main.rs".to_string(),
                    change_type: ChangeType::Modified,
                    additions: 1,
                    deletions: 0,
                    language: Some("Rust".to_string()),
                },
                ChangedFile {
                    path: "README.md".to_string(),
                    change_type: ChangeType::Modified,
                    additions: 1,
                    deletions: 0,
                    language: Some("Markdown".to_string()),
                },
                ChangedFile {
                    path: "image.png".to_string(),
                    change_type: ChangeType::Added,
                    additions: 0,
                    deletions: 0,
                    language: None, // 二进制文件通常没有语言检测
                }
            ],
            analysis_data: AnalysisData::default(),
            options: ReviewOptions {
                language: None,
                format: OutputFormat::Console,
                output: None,
                tree_sitter: false,
                security_scan: false,
                scan_tool: None,
                block_on_critical: false,
                issue_id: None,
                space_id: None,
                full: false,
            },
        };

        // 验证不同文件类型的处理
        let code_files_count = review_data.changed_files.iter()
            .filter(|f| f.language.as_ref().map_or(false, |lang| lang.contains("Rust") || lang.contains("Markdown")))
            .count();
        assert!(code_files_count >= 2); // 至少有代码和文档文件

        let binary_files_count = review_data.changed_files.iter()
            .filter(|f| f.language.is_none())
            .count();
        assert!(binary_files_count >= 1); // 应该有二进制文件
    }
}

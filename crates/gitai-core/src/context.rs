// GitAI 统一操作上下文
// 消除重复定义，提供一致的数据结构

use gitai_types::{GitInfo, RiskLevel, Severity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 统一操作上下文 - 所有 GitAI 操作的核心数据结构
///
/// 遵循 "Fix the data structures, not the symptoms" 原则
/// 这个结构体包含了所有操作需要的数据，避免了在函数间重复传递大量参数
#[derive(Debug, Clone)]
pub struct OperationContext {
    /// 代码变更内容（git diff）
    pub diff: String,

    /// Git 信息（可选）
    pub git_info: Option<GitInfo>,

    /// 相关的 Issue 列表
    pub issues: Vec<Issue>,

    /// 结构分析信息（Tree-sitter 分析结果）
    pub structural_info: Option<StructuralSummary>,

    /// 架构影响分析结果
    pub architectural_impact: Option<ArchitecturalImpact>,

    /// 依赖图（简化版）
    pub dependency_graph: Option<DependencyGraph>,

    /// 影响范围（传播分析）
    pub impact_scope: Option<ImpactScope>,

    /// 级联效应列表
    pub cascade_effects: Vec<CascadeEffect>,

    /// 操作特定的选项
    pub options: OperationOptions,

    /// 额外元数据
    pub metadata: HashMap<String, String>,
}

/// 操作选项 - 统一所有操作的配置选项
#[derive(Debug, Clone, Default)]
pub struct OperationOptions {
    // 通用选项
    /// 仅演示，不产生副作用
    pub dry_run: bool,
    /// 输出详细日志
    pub verbose: bool,
    /// 指定语言（可选）
    pub language: Option<String>,
    /// 输出路径（可选）
    pub output: Option<PathBuf>,
    /// 输出格式（可选）
    pub format: Option<String>,

    // Issue 相关
    /// 关联的 Issue ID 列表
    pub issue_ids: Vec<String>,
    /// 是否执行偏离度分析
    pub deviation_analysis: bool,

    // 分析选项
    /// 是否启用 Tree-sitter 分析
    pub tree_sitter: bool,
    /// 是否启用安全扫描
    pub security_scan: bool,
    /// 指定扫描工具（可选）
    pub scan_tool: Option<String>,
    /// 是否执行架构分析
    pub architectural_analysis: bool,
    /// 是否执行依赖分析
    pub dependency_analysis: bool,

    // 评审选项
    /// 遇到严重问题是否阻塞
    pub block_on_critical: bool,
    /// 是否包含建议输出
    pub include_suggestions: bool,

    // 提交选项
    /// 提交信息（可选）
    pub message: Option<String>,
    /// 是否添加所有变更
    pub add_all: bool,
    /// 提交前是否先评审
    pub review_before_commit: bool,

    // 性能选项
    /// 是否启用缓存
    pub cache_enabled: bool,
    /// 超时时间（秒，可选）
    pub timeout: Option<u64>,
    /// 最大分析深度（可选）
    pub max_depth: Option<usize>,
}

/// Issue 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Issue 唯一标识
    pub id: String,
    /// 标题
    pub title: String,
    /// 描述
    pub description: String,
    /// 状态
    pub status: String,
    /// 优先级（可选）
    pub priority: Option<String>,
    /// 链接地址
    pub url: String,
    /// 标签
    pub labels: Vec<String>,
    /// 指派人（可选）
    pub assignee: Option<String>,
}

/// 结构化摘要（简化版，实际实现在 gitai-analysis 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralSummary {
    /// 函数信息
    pub functions: Vec<FunctionInfo>,
    /// 类/结构体信息
    pub classes: Vec<ClassInfo>,
    /// 导入信息
    pub imports: Vec<String>,
    /// 复杂度指标
    pub complexity: ComplexityMetrics,
}

/// 函数信息（结构化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    /// 函数名
    pub name: String,
    /// 参数列表
    pub parameters: Vec<String>,
    /// 返回类型（可选）
    pub return_type: Option<String>,
    /// 起始行
    pub line_start: usize,
    /// 结束行
    pub line_end: usize,
}

/// 类/结构体信息（结构化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInfo {
    /// 类/结构体名
    pub name: String,
    /// 方法列表
    pub methods: Vec<String>,
    /// 字段列表
    pub fields: Vec<String>,
    /// 起始行
    pub line_start: usize,
    /// 结束行
    pub line_end: usize,
}

/// 复杂度指标（结构化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// 圈复杂度
    pub cyclomatic: usize,
    /// 认知复杂度
    pub cognitive: usize,
    /// 代码行数
    pub lines_of_code: usize,
}

/// 架构影响（简化版，实际实现在 gitai-analysis 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalImpact {
    /// 破坏性变更列表
    pub breaking_changes: Vec<BreakingChangeInfo>,
    /// 风险级别
    pub risk_level: RiskLevel,
    /// 受影响的组件
    pub affected_components: Vec<String>,
}

/// 破坏性变更信息（结构化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChangeInfo {
    /// 组件名称
    pub component: String,
    /// 变更描述
    pub description: String,
    /// 风险级别
    pub risk_level: RiskLevel,
}

/// 依赖图（简化版，实际实现在 gitai-analysis 中）
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// 节点集合
    pub nodes: Vec<String>,
    /// 边集合（from, to）
    pub edges: Vec<(String, String)>,
}

/// 影响范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactScope {
    /// 直接受影响的组件
    pub directly_affected: Vec<String>,
    /// 间接受影响的组件
    pub indirectly_affected: Vec<String>,
    /// 影响半径（0-1）
    pub impact_radius: f32,
}

/// 级联效应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeEffect {
    /// 源组件
    pub source: String,
    /// 目标组件列表
    pub targets: Vec<String>,
    /// 效应类型
    pub effect_type: String,
    /// 严重程度
    pub severity: Severity,
}

impl OperationContext {
    /// 创建新的操作上下文
    pub fn new() -> Self {
        Self {
            diff: String::new(),
            git_info: None,
            issues: Vec::new(),
            structural_info: None,
            architectural_impact: None,
            dependency_graph: None,
            impact_scope: None,
            cascade_effects: Vec::new(),
            options: OperationOptions::default(),
            metadata: HashMap::new(),
        }
    }

    /// Builder 模式方法
    pub fn with_diff(mut self, diff: String) -> Self {
        self.diff = diff;
        self
    }

    /// 设置 Git 信息（builder）
    pub fn with_git_info(mut self, info: GitInfo) -> Self {
        self.git_info = Some(info);
        self
    }

    /// 设置 Issue 列表（builder）
    pub fn with_issues(mut self, issues: Vec<Issue>) -> Self {
        self.issues = issues;
        self
    }

    /// 设置结构化摘要（builder）
    pub fn with_structural_info(mut self, info: StructuralSummary) -> Self {
        self.structural_info = Some(info);
        self
    }

    /// 设置架构影响分析（builder）
    pub fn with_architectural_impact(mut self, impact: ArchitecturalImpact) -> Self {
        self.architectural_impact = Some(impact);
        self
    }

    /// 设置依赖图（builder）
    pub fn with_dependency_graph(mut self, graph: DependencyGraph) -> Self {
        self.dependency_graph = Some(graph);
        self
    }

    /// 设置影响范围（builder）
    pub fn with_impact_scope(mut self, scope: ImpactScope) -> Self {
        self.impact_scope = Some(scope);
        self
    }

    /// 设置级联效应（builder）
    pub fn with_cascade_effects(mut self, effects: Vec<CascadeEffect>) -> Self {
        self.cascade_effects = effects;
        self
    }

    /// 设置操作选项（builder）
    pub fn with_options(mut self, options: OperationOptions) -> Self {
        self.options = options;
        self
    }

    /// 添加额外元数据（builder）
    pub fn add_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// 检查方法
    pub fn has_changes(&self) -> bool {
        !self.diff.trim().is_empty()
    }

    /// 是否包含关联的 Issue
    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    /// 是否需要 Issue 上下文（根据选项）
    pub fn needs_issue_context(&self) -> bool {
        !self.options.issue_ids.is_empty() || self.options.deviation_analysis
    }

    /// 是否存在高风险（High/Critical）
    pub fn has_high_risk(&self) -> bool {
        self.architectural_impact
            .as_ref()
            .map(|impact| matches!(impact.risk_level, RiskLevel::High | RiskLevel::Critical))
            .unwrap_or(false)
    }

    /// 获取 Issue 上下文字符串
    pub fn issue_context(&self) -> String {
        if self.issues.is_empty() {
            return String::new();
        }

        self.issues
            .iter()
            .map(|issue| {
                format!(
                    "Issue #{}: {}\n描述: {}\n状态: {}\n优先级: {}\n链接: {}\n",
                    issue.id,
                    issue.title,
                    issue.description,
                    issue.status,
                    issue.priority.as_deref().unwrap_or("未设置"),
                    issue.url
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for OperationContext {
    fn default() -> Self {
        Self::new()
    }
}

impl OperationOptions {
    /// 从提交参数创建选项
    pub fn for_commit(
        message: Option<String>,
        issue_ids: Vec<String>,
        add_all: bool,
        review: bool,
        tree_sitter: bool,
        dry_run: bool,
    ) -> Self {
        Self {
            message,
            issue_ids,
            add_all,
            review_before_commit: review,
            tree_sitter,
            dry_run,
            ..Default::default()
        }
    }

    /// 从评审参数创建选项
    #[allow(clippy::too_many_arguments)]
    pub fn for_review(
        language: Option<String>,
        format: Option<String>,
        output: Option<PathBuf>,
        tree_sitter: bool,
        security_scan: bool,
        scan_tool: Option<String>,
        block_on_critical: bool,
        issue_ids: Vec<String>,
        deviation_analysis: bool,
    ) -> Self {
        Self {
            language,
            format,
            output,
            tree_sitter,
            security_scan,
            scan_tool,
            block_on_critical,
            issue_ids,
            deviation_analysis,
            ..Default::default()
        }
    }

    /// 从分析参数创建选项
    pub fn for_analysis(
        issue_ids: Vec<String>,
        deviation_analysis: bool,
        security_scan: bool,
        architectural_analysis: bool,
        dependency_analysis: bool,
    ) -> Self {
        Self {
            issue_ids,
            deviation_analysis,
            security_scan,
            architectural_analysis,
            dependency_analysis,
            tree_sitter: true, // 分析默认启用 tree-sitter
            ..Default::default()
        }
    }

    /// 从扫描参数创建选项
    pub fn for_scan(
        scan_tool: Option<String>,
        output: Option<PathBuf>,
        format: Option<String>,
        timeout: Option<u64>,
    ) -> Self {
        Self {
            scan_tool,
            output,
            format,
            timeout,
            security_scan: true,
            ..Default::default()
        }
    }
}

/// 解析 Issue ID 字符串为列表
pub fn parse_issue_ids(issue_id: Option<String>) -> Vec<String> {
    issue_id
        .map(|id| {
            id.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

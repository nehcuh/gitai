// GitAI 统一操作上下文
// 消除重复定义，提供一致的数据结构

use gitai_types::{GitInfo, Finding, Severity, RiskLevel};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;

/// 统一操作上下文 - 所有 GitAI 操作的核心数据结构
/// 
/// 遵循 "Fix the data structures, not the symptoms" 原则
/// 这个结构体包含了所有操作需要的数据，避免了在函数间重复传递大量参数
#[derive(Debug, Clone)]
pub struct OperationContext {
    /// 代码变更内容
    pub diff: String,
    
    /// Git 信息
    pub git_info: Option<GitInfo>,
    
    /// 相关的 Issue 列表
    pub issues: Vec<Issue>,
    
    /// 结构分析信息（Tree-sitter 分析结果）
    pub structural_info: Option<StructuralSummary>,
    
    /// 架构影响分析结果
    pub architectural_impact: Option<ArchitecturalImpact>,
    
    /// 依赖图
    pub dependency_graph: Option<DependencyGraph>,
    
    /// 影响范围
    pub impact_scope: Option<ImpactScope>,
    
    /// 级联效应
    pub cascade_effects: Vec<CascadeEffect>,
    
    /// 操作特定的选项
    pub options: OperationOptions,
    
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 操作选项 - 统一所有操作的配置选项
#[derive(Debug, Clone, Default)]
pub struct OperationOptions {
    // 通用选项
    pub dry_run: bool,
    pub verbose: bool,
    pub language: Option<String>,
    pub output: Option<PathBuf>,
    pub format: Option<String>,
    
    // Issue 相关
    pub issue_ids: Vec<String>,
    pub deviation_analysis: bool,
    
    // 分析选项
    pub tree_sitter: bool,
    pub security_scan: bool,
    pub scan_tool: Option<String>,
    pub architectural_analysis: bool,
    pub dependency_analysis: bool,
    
    // 评审选项
    pub block_on_critical: bool,
    pub include_suggestions: bool,
    
    // 提交选项
    pub message: Option<String>,
    pub add_all: bool,
    pub review_before_commit: bool,
    
    // 性能选项
    pub cache_enabled: bool,
    pub timeout: Option<u64>,
    pub max_depth: Option<usize>,
}

/// Issue 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: Option<String>,
    pub url: String,
    pub labels: Vec<String>,
    pub assignee: Option<String>,
}

/// 结构化摘要（简化版，实际实现在 gitai-analysis 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralSummary {
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub imports: Vec<String>,
    pub complexity: ComplexityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
    pub line_start: usize,
    pub line_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInfo {
    pub name: String,
    pub methods: Vec<String>,
    pub fields: Vec<String>,
    pub line_start: usize,
    pub line_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    pub cyclomatic: usize,
    pub cognitive: usize,
    pub lines_of_code: usize,
}

/// 架构影响（简化版，实际实现在 gitai-analysis 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalImpact {
    pub breaking_changes: Vec<BreakingChangeInfo>,
    pub risk_level: RiskLevel,
    pub affected_components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChangeInfo {
    pub component: String,
    pub description: String,
    pub risk_level: RiskLevel,
}

/// 依赖图（简化版，实际实现在 gitai-analysis 中）
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>,
}

/// 影响范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactScope {
    pub directly_affected: Vec<String>,
    pub indirectly_affected: Vec<String>,
    pub impact_radius: f32,
}

/// 级联效应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeEffect {
    pub source: String,
    pub targets: Vec<String>,
    pub effect_type: String,
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
    
    pub fn with_git_info(mut self, info: GitInfo) -> Self {
        self.git_info = Some(info);
        self
    }
    
    pub fn with_issues(mut self, issues: Vec<Issue>) -> Self {
        self.issues = issues;
        self
    }
    
    pub fn with_structural_info(mut self, info: StructuralSummary) -> Self {
        self.structural_info = Some(info);
        self
    }
    
    pub fn with_architectural_impact(mut self, impact: ArchitecturalImpact) -> Self {
        self.architectural_impact = Some(impact);
        self
    }
    
    pub fn with_dependency_graph(mut self, graph: DependencyGraph) -> Self {
        self.dependency_graph = Some(graph);
        self
    }
    
    pub fn with_impact_scope(mut self, scope: ImpactScope) -> Self {
        self.impact_scope = Some(scope);
        self
    }
    
    pub fn with_cascade_effects(mut self, effects: Vec<CascadeEffect>) -> Self {
        self.cascade_effects = effects;
        self
    }
    
    pub fn with_options(mut self, options: OperationOptions) -> Self {
        self.options = options;
        self
    }
    
    pub fn add_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// 检查方法
    pub fn has_changes(&self) -> bool {
        !self.diff.trim().is_empty()
    }
    
    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }
    
    pub fn needs_issue_context(&self) -> bool {
        !self.options.issue_ids.is_empty() || self.options.deviation_analysis
    }
    
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
        
        self.issues.iter()
            .map(|issue| format!(
                "Issue #{}: {}\n描述: {}\n状态: {}\n优先级: {}\n链接: {}\n",
                issue.id, issue.title, issue.description, issue.status,
                issue.priority.as_deref().unwrap_or("未设置"), issue.url
            ))
            .collect::<Vec<_>>()
            .join("\n")
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

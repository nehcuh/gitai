// GitAI 操作上下文
// 统一所有操作的数据传递，消除重复配置和参数传递混乱

use crate::architectural_impact::ArchitecturalImpactAnalysis;
use crate::config::Config;
use crate::tree_sitter::StructuralSummary;
use std::path::PathBuf;

// Re-export Issue from devops when available, or define a stub
#[cfg(feature = "devops")]
pub use crate::devops::Issue;

// Define a stub Issue type when devops feature is not enabled
#[cfg(not(feature = "devops"))]
#[derive(Debug, Clone)]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: Option<String>,
    pub url: String,
    pub ai_context: Option<String>,
}

/// 统一操作上下文 - Linus式数据结构优先设计
///
/// 这个结构体包含了所有GitAI操作需要的数据，避免了在函数间重复传递
/// 大量参数的混乱。遵循"Fix the data structures, not the symptoms"的原则。
#[derive(Debug, Clone)]
pub struct OperationContext {
    /// 应用配置
    pub config: Config,

    /// 代码变更内容
    pub diff: String,

    /// 相关的Issue列表
    pub issues: Vec<Issue>,

    /// Tree-sitter结构分析结果
    pub structural_info: Option<StructuralSummary>,

    /// 架构影响分析结果
    pub architectural_impact: Option<ArchitecturalImpactAnalysis>,

    /// 操作特定的选项
    pub options: OperationOptions,
}

/// 操作选项 - 统一所有操作的配置选项
///
/// 之前分散在CommitConfig, ReviewConfig, AnalysisConfig中的选项
/// 现在统一在这里，消除重复和不一致
#[derive(Debug, Clone, Default)]
pub struct OperationOptions {
    // 通用选项
    pub dry_run: bool,
    pub language: Option<String>,
    pub output: Option<PathBuf>,
    pub issue_ids: Vec<String>,

    // 分析选项
    pub tree_sitter: bool,
    pub security_scan: bool,
    pub scan_tool: Option<String>,
    pub deviation_analysis: bool,

    // 评审选项
    pub format: Option<String>,
    pub block_on_critical: bool,

    // 提交选项
    pub message: Option<String>,
    pub add_all: bool,
    pub review_before_commit: bool,
}

impl OperationContext {
    /// 创建新的操作上下文
    pub fn new(config: Config) -> Self {
        Self {
            config,
            diff: String::new(),
            issues: Vec::new(),
            structural_info: None,
            architectural_impact: None,
            options: OperationOptions::default(),
        }
    }

    /// 设置代码变更
    pub fn with_diff(mut self, diff: String) -> Self {
        self.diff = diff;
        self
    }

    /// 设置相关Issue
    pub fn with_issues(mut self, issues: Vec<Issue>) -> Self {
        self.issues = issues;
        self
    }

    /// 设置结构分析信息
    pub fn with_structural_info(mut self, info: StructuralSummary) -> Self {
        self.structural_info = Some(info);
        self
    }

    /// 设置架构影响分析信息
    pub fn with_architectural_impact(mut self, impact: ArchitecturalImpactAnalysis) -> Self {
        self.architectural_impact = Some(impact);
        self
    }

    /// 设置操作选项
    pub fn with_options(mut self, options: OperationOptions) -> Self {
        self.options = options;
        self
    }

    /// 是否有变更需要处理
    pub fn has_changes(&self) -> bool {
        !self.diff.trim().is_empty()
    }

    /// 是否有相关Issue
    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    /// 是否需要Issue上下文
    pub fn needs_issue_context(&self) -> bool {
        !self.options.issue_ids.is_empty() || self.options.deviation_analysis
    }

    /// 获取Issue上下文字符串
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

impl OperationOptions {
    /// 从提交参数创建选项
    pub fn for_commit(
        message: Option<String>,
        issue_id: Option<String>,
        add_all: bool,
        review: bool,
        tree_sitter: bool,
        dry_run: bool,
    ) -> Self {
        Self {
            message,
            issue_ids: parse_issue_ids(issue_id),
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
        format: String,
        output: Option<PathBuf>,
        tree_sitter: bool,
        security_scan: bool,
        scan_tool: Option<String>,
        block_on_critical: bool,
        issue_id: Option<String>,
        deviation_analysis: bool,
    ) -> Self {
        Self {
            language,
            format: Some(format),
            output,
            tree_sitter,
            security_scan,
            scan_tool,
            block_on_critical,
            issue_ids: parse_issue_ids(issue_id),
            deviation_analysis,
            ..Default::default()
        }
    }

    /// 从分析参数创建选项
    pub fn for_analysis(
        issue_id: Option<String>,
        deviation_analysis: bool,
        security_scan: bool,
    ) -> Self {
        Self {
            issue_ids: parse_issue_ids(issue_id),
            deviation_analysis,
            security_scan,
            ..Default::default()
        }
    }
}

/// 解析issue ID字符串为列表
fn parse_issue_ids(issue_id: Option<String>) -> Vec<String> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    if let Some(ids) = issue_id {
        for raw in ids.split(',') {
            let mut s = raw.trim().to_string();
            if s.is_empty() {
                continue;
            }
            if !s.starts_with('#') {
                s = format!("#{s}");
            }
            if seen.insert(s.clone()) {
                out.push(s);
            }
        }
    }
    out
}

use crate::architectural_impact::ArchitecturalImpact;
use crate::architectural_impact::{CascadeEffect, DependencyGraph, ImpactScope};
use gitai_core::Config;
use gitai_core::context::Issue; // Always use from context module, which handles the conditional compilation
use crate::tree_sitter::StructuralSummary;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 统一操作上下文 - Linus式数据结构优先设计
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
    pub architectural_impact: Option<ArchitecturalImpact>,
    /// 依赖图（从结构化摘要构建）
    pub dependency_graph: Option<DependencyGraph>,
    /// 影响范围（传播分析结果）
    pub impact_scope: Option<ImpactScope>,
    /// 级联效应
    pub cascade_effects: Vec<CascadeEffect>,
    /// 操作特定的选项
    pub options: OperationOptions,
}

/// 操作选项 - 统一所有操作的配置选项
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
            dependency_graph: None,
            impact_scope: None,
            cascade_effects: Vec::new(),
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
    pub fn with_architectural_impact(mut self, impact: ArchitecturalImpact) -> Self {
        self.architectural_impact = Some(impact);
        self
    }

    /// 设置依赖图
    pub fn with_dependency_graph(mut self, graph: DependencyGraph) -> Self {
        self.dependency_graph = Some(graph);
        self
    }

    /// 设置影响范围
    pub fn with_impact_scope(mut self, scope: ImpactScope) -> Self {
        self.impact_scope = Some(scope);
        self
    }

    /// 设置级联效应
    pub fn with_cascade_effects(mut self, effects: Vec<CascadeEffect>) -> Self {
        self.cascade_effects = effects;
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

/// 分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub review_result: String,
    pub security_findings: Vec<SecurityFinding>,
    pub deviation_analysis: Option<DeviationAnalysis>,
    /// 影响范围的Markdown报告（若已计算）
    pub impact_markdown: Option<String>,
    /// 级联效应数量（若已计算）
    pub cascade_effects_count: Option<usize>,
    /// 影响范围分析结果
    pub impact_scope: Option<ImpactScope>,
    /// 架构影响分析结果
    pub architectural_impact: Option<ArchitecturalImpact>,
}

/// 安全发现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub title: String,
    pub file_path: String,
    pub line: usize,
    pub severity: String,
    pub rule_id: String,
    pub code_snippet: Option<String>,
}

/// 偏离度分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviationAnalysis {
    pub requirement_coverage: f32, // 需求覆盖率 0.0-1.0
    pub quality_score: f32,        // 质量评分 0.0-1.0
    pub deviations: Vec<Deviation>,
    pub suggestions: Vec<String>,
}

/// 偏离项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deviation {
    pub type_: String,       // 偏离类型
    pub description: String, // 描述
    pub severity: String,    // 严重程度
    pub suggestion: String,  // 建议
}

/// 代码分析器 - 使用统一的OperationContext
pub struct Analyzer;

impl Analyzer {
    /// 执行完整分析 - 使用统一上下文
    pub async fn analyze(
        context: &OperationContext,
    ) -> Result<AnalysisResult, Box<dyn std::error::Error + Send + Sync>> {
        let security_findings = if context.options.security_scan {
            Self::analyze_security(context).await?
        } else {
            Vec::new()
        };

        let review_result = Self::analyze_review(context, &security_findings).await?;

        let deviation_analysis = if context.options.deviation_analysis && context.has_issues() {
            Some(Self::analyze_deviation(context).await?)
        } else {
            None
        };

        // 构建影响报告（如果有上下文中的影响范围）
        let (impact_markdown, cascade_effects_count) = Self::build_impact_metadata(context);

        Ok(AnalysisResult {
            review_result,
            security_findings,
            deviation_analysis,
            impact_markdown,
            cascade_effects_count,
            impact_scope: context.impact_scope.clone(),
            architectural_impact: context.architectural_impact.clone(),
        })
    }

    /// 分析代码评审
    async fn analyze_review(
        context: &OperationContext,
        security_findings: &[SecurityFinding],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // 准备tree-sitter结构信息
        let tree_sitter_info = if let Some(ref structural_info) = context.structural_info {
            serde_json::to_string(structural_info).unwrap_or_default()
        } else {
            "无结构分析信息".to_string()
        };

        // 准备安全扫描结果
        let security_scan_results = if security_findings.is_empty() {
            "✅ 未发现安全问题".to_string()
        } else {
            let mut result = String::new();
            for finding in security_findings {
                result.push_str(&format!(
                    "- {} ({}) ({})\n",
                    finding.title, finding.file_path, finding.rule_id
                ));
            }
            result
        };

        // 准备DevOps Issue上下文
        let devops_issue_context = if context.has_issues() {
            context.issue_context()
        } else {
            "无相关Issue上下文".to_string()
        };

        // 尝试使用模板，如果失败则降级为硬编码提示词
        log::debug!("准备使用模板进行AI评审");

        #[cfg(feature = "ai")]
        {
            // 依赖洞察：在 Analyzer 流程中暂无依赖图，传入空文本（留给 review --full 路径）
            match crate::ai::review_code_with_template(
                &context.config,
                &context.diff,
                Some(&tree_sitter_info),
                &security_scan_results,
                &devops_issue_context,
                "",
            )
            .await
            {
                Ok(result) => {
                    log::debug!("模板AI调用成功，结果长度: {}", result.len());
                    Ok(result)
                }
                Err(template_error) => {
                    log::warn!("使用模板失败，降级为硬编码提示词: {template_error}");

                    // 降级为原有的硬编码逻辑
                    let mut prompt = format!(
                        "请评审以下代码变更，重点关注代码质量、安全性、性能等方面：\n\n代码变更：\n{}",
                        context.diff
                    );

                    // 添加结构分析信息
                    if context.structural_info.is_some() {
                        prompt.push_str(&format!("\n\n{tree_sitter_info}"));
                    }

                    if context.has_issues() {
                        prompt
                            .push_str(&format!("\n\n相关Issue信息：\n{}", context.issue_context()));
                    }

                    if context.options.deviation_analysis && context.has_issues() {
                        prompt.push_str("\n\n请特别分析以下方面：\n");
                        prompt.push_str("1. 代码变更是否完全解决了Issue中描述的问题？\n");
                        prompt.push_str("2. 是否存在偏离需求的情况？\n");
                        prompt.push_str("3. 代码实现是否符合Issue的优先级和重要性？\n");
                        prompt.push_str("4. 是否引入了与Issue无关的代码变更？\n");
                        prompt.push_str("5. 代码质量是否满足生产环境要求？\n");
                        prompt.push_str("\n请提供偏离度分析报告，包括符合度和改进建议。");
                    }

                    match crate::ai::call_ai(&context.config, &prompt).await {
                        Ok(result) => Ok(result),
                        Err(e) => Err(format!("AI服务错误: {e}").into()),
                    }
                }
            }
        }

        #[cfg(not(feature = "ai"))]
        {
            // 在未启用 AI 时返回基于静态信息的评审摘要
            let mut summary = String::new();
            summary.push_str("[AI未启用] 基于静态信息的代码评审摘要\n\n");
            summary.push_str("结构分析信息:\n");
            summary.push_str(&tree_sitter_info);
            summary.push_str("\n\n安全扫描结果:\n");
            summary.push_str(&security_scan_results);
            if !devops_issue_context.is_empty() {
                summary.push_str("\n\n相关Issue信息:\n");
                summary.push_str(&devops_issue_context);
            }
            Ok(summary)
        }
    }

    /// 分析安全问题
    async fn analyze_security(
        context: &OperationContext,
    ) -> Result<Vec<SecurityFinding>, Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(feature = "security")]
        {
            let current_dir = std::env::current_dir()?;
            match crate::scan::run_opengrep_scan(&context.config, &current_dir, None, None, false) {
                Ok(result) => {
                    let findings: Vec<SecurityFinding> = result
                        .findings
                        .into_iter()
                        .map(|f| SecurityFinding {
                            title: f.title,
                            file_path: f.file_path.display().to_string(),
                            line: f.line,
                            severity: f.severity,
                            rule_id: f.rule_id.unwrap_or_else(|| "unknown".to_string()),
                            code_snippet: f.code_snippet,
                        })
                        .collect();
                    Ok(findings)
                }
                Err(e) => {
                    eprintln!("⚠️ 安全扫描失败: {e}");
                    Ok(Vec::new())
                }
            }
        }

        #[cfg(not(feature = "security"))]
        {
            let _ = context; // silence unused parameter when security feature is disabled
            log::debug!("Security scan feature is not enabled");
            Ok(Vec::new())
        }
    }

    /// 分析偏离度
    async fn analyze_deviation(
        context: &OperationContext,
    ) -> Result<DeviationAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            "分析以下代码变更与Issue需求的符合度，提供偏离度分析：\n\nIssue信息：\n{}\n\n代码变更：\n{}\n\n请分析：\n1. 需求覆盖率 (0.0-1.0)\n2. 质量评分 (0.0-1.0)\n3. 具体偏离项\n4. 改进建议",
            context.issue_context(),
            context.diff
        );

        #[cfg(feature = "ai")]
        {
            match crate::ai::call_ai(&context.config, &prompt).await {
                Ok(ai_response) => {
                    // 简化的解析逻辑 - 实际应该更robust
                    let requirement_coverage =
                        extract_score(&ai_response, "需求覆盖率").unwrap_or(0.8);
                    let quality_score = extract_score(&ai_response, "质量评分").unwrap_or(0.7);

                    Ok(DeviationAnalysis {
                        requirement_coverage,
                        quality_score,
                        deviations: vec![],
                        suggestions: vec![ai_response.to_string()],
                    })
                }
                Err(e) => Err(format!("AI服务错误: {e}").into()),
            }
        }

        #[cfg(not(feature = "ai"))]
        {
            // 在未启用 AI 时返回一个保守的默认偏离度分析
            Ok(DeviationAnalysis {
                requirement_coverage: 0.7,
                quality_score: 0.7,
                deviations: vec![],
                suggestions: vec![
                    "AI未启用：返回默认偏离度分析。请启用 ai 功能以获得更准确结果。".to_string(),
                ],
            })
        }
    }
}

impl Analyzer {
    /// 根据上下文构建影响范围的Markdown与级联效应计数
    fn build_impact_metadata(context: &OperationContext) -> (Option<String>, Option<usize>) {
        if let (Some(graph), Some(scope)) = (&context.dependency_graph, &context.impact_scope) {
            let md = crate::architectural_impact::generate_markdown_report(scope, Some(graph));
            let cascades_count = Some(context.cascade_effects.len());
            (Some(md), cascades_count)
        } else {
            (None, None)
        }
    }
}

/// 从AI响应中提取分数
fn extract_score(response: &str, key: &str) -> Option<f32> {
    // 简单的实现，实际应该用更复杂的解析
    response
        .lines()
        .find(|line| line.contains(key))
        .and_then(|line| line.split(':').nth(1))
        .and_then(|s| s.trim().parse::<f32>().ok())
}

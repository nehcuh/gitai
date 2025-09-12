// review 执行器模块
// 负责执行评审流程的核心逻辑

use super::types::{ReviewConfig, ReviewResult};
use gitai_core::config::Config;

/// 执行评审流程（控制台输出）
pub async fn execute_review(
    config: &Config,
    review_config: ReviewConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let result = execute_review_with_result(config, review_config).await?;

    // 打印结果到控制台
    println!("\n🤖 AI 代码评审结果:");
    println!("{}", "=".repeat(80));
    println!("{}", result.summary);

    if !result.findings.is_empty() {
        println!("\n🔒 发现的问题:");
        for finding in &result.findings {
            println!("  ⚠️  {}", finding.title);
        }
    }

    if !result.recommendations.is_empty() {
        println!("\n💡 改进建议:");
        for rec in &result.recommendations {
            println!("  • {rec}");
        }
    }

    // 依赖分析洞察（若有）
    if result.details.contains_key("dep_nodes") {
        println!("\n🔗 依赖分析洞察:");
        if let (Some(nodes), Some(edges)) = (
            result.details.get("dep_nodes"),
            result.details.get("dep_edges"),
        ) {
            println!("  图规模: {nodes} 节点 / {edges} 边");
        }
        if let Some(avg) = result.details.get("dep_avg_degree") {
            println!("  平均度: {avg}");
        }
        if let Some(cn) = result.details.get("dep_critical_nodes") {
            println!("  关键节点数: {cn}");
        }
        if let Some(mapped) = result.details.get("dep_changed_nodes_mapped") {
            println!("  映射变更节点: {mapped}");
        }
        if let Some(scope) = result.details.get("dep_impact_scope_count") {
            println!("  影响范围节点: {scope}");
        }
        if let Some(top_pr) = result.details.get("dep_top_pagerank") {
            println!("  PageRank Top: {top_pr}");
        }
        if let Some(top_imp) = result.details.get("dep_top_impacted") {
            println!("  影响度 Top: {top_imp}");
        }
    }

    println!("{}", "=".repeat(80));
    Ok(())
}

/// 执行评审流程并返回结构化结果
pub async fn execute_review_with_result(
    config: &Config,
    review_config: ReviewConfig,
) -> Result<ReviewResult, Box<dyn std::error::Error + Send + Sync>> {
    // 获取代码变更
    // 优先获取当前变更，如果没有则尝试获取最后一次提交
    // 这样 MCP 调用时即使没有新变更也可以分析最近的提交
    let diff = match gitai_core::git_impl::get_all_diff() {
        Ok(d) => d,
        Err(_) => {
            // 如果没有当前变更，尝试获取最后一次提交
            match gitai_core::git_impl::get_last_commit_diff() {
                Ok(last_diff) if !last_diff.trim().is_empty() => {
                    format!("## 最后一次提交的变更 (Last Commit):\n{last_diff}")
                }
                Ok(_) => {
                    // 最后一次提交为空
                    return Ok(ReviewResult {
                        success: true,
                        message: "没有检测到代码变更".to_string(),
                        summary: "没有需要评审的代码变更".to_string(),
                        details: std::collections::HashMap::new(),
                        findings: Vec::new(),
                        score: Some(100),
                        recommendations: Vec::new(),
                    });
                }
                Err(e) => {
                    // 无法获取任何 diff，可能是新仓库或空仓库，或 MCP 服务未在仓库目录执行
                    log::warn!("无法获取代码变更: {e}");

                    // 在 details 中附带当前工作目录，方便排查
                    let mut details = std::collections::HashMap::new();
                    if let Ok(cwd) = std::env::current_dir() {
                        details.insert("cwd".to_string(), cwd.display().to_string());
                    }

                    return Ok(ReviewResult {
                        success: true,
                        message: "无法获取代码变更，可能是新仓库、空仓库，或 MCP 服务不在 Git 仓库目录中运行".to_string(),
                        summary: "没有可用的代码变更进行评审".to_string(),
                        details,
                        findings: Vec::new(),
                        score: None,
                        recommendations: vec![
                            "请确保仓库中至少有一个提交".to_string(),
                            "或者添加一些代码变更后再进行评审".to_string(),
                            "如果通过 MCP 调用，请在参数中设置 path 指向仓库根目录，或在仓库根目录启动 MCP 服务".to_string(),
                        ],
                    });
                }
            }
        }
    };

    // 如果没有变更，返回空结果
    if diff.trim().is_empty() {
        return Ok(ReviewResult {
            success: true,
            message: "没有检测到代码变更".to_string(),
            summary: "没有需要评审的代码变更".to_string(),
            details: std::collections::HashMap::new(),
            findings: Vec::new(),
            score: Some(100),
            recommendations: Vec::new(),
        });
    }

    // 生成缓存键
    let cache_key = super::cache::build_cache_key(&diff, &review_config);

    // 检查缓存
    if let Some(cached_result) = super::cache::check_cache(&cache_key)? {
        println!("📦 使用缓存的评审结果");
        return Ok(ReviewResult {
            success: true,
            message: "代码评审完成（缓存）".to_string(),
            summary: cached_result,
            details: std::collections::HashMap::new(),
            findings: Vec::new(),
            score: Some(85),
            recommendations: Vec::new(),
        });
    }

    // 检查暂存状态与未跟踪文件、提交基线
    let has_unstaged = gitai_core::git_impl::has_unstaged_changes().unwrap_or(false);
    let has_staged = gitai_core::git_impl::has_staged_changes().unwrap_or(false);
    let has_untracked = gitai_core::git_impl::has_untracked_changes().unwrap_or(false);
    let has_commits = gitai_core::git_impl::has_any_commit();

    if has_unstaged || has_untracked {
        if has_unstaged {
            println!("💡 提示：检测到未暂存的代码变更");
            println!("   使用 `git add .` 暂存所有变更，或使用 `git add <file>` 暂存特定文件");
        }
        if has_untracked {
            println!("💡 提示：检测到未跟踪的新文件");
            println!("   使用 `git add <file>` 开始跟踪这些文件");
        }
        if has_staged {
            println!("   当前已暂存的变更也会被评审");
        }
        if !has_commits {
            println!("   ⚠️ 当前仓库还没有任何提交（建议尽快 `git commit -m \"<msg>\"`）");
        }
        println!("   📝 GitAI将分析所有变更（已暂存 + 未暂存 + 未跟踪）");
        println!();
    } else if has_staged {
        println!("✅ 已暂存的代码准备就绪");
        println!("   📝 GitAI将分析已暂存的变更");
    } else if !has_commits {
        println!("💡 提示：仓库没有任何提交。请先进行一次提交以建立基线：");
        println!("   git add -A && git commit -m \"init\"");
    } else {
        println!("🔍 检查未推送的提交...");
        println!("   📝 GitAI将分析最近的提交变更");
    }

    // 如果启用了 tree-sitter 分析
    let mut structural_summary = None;
    if review_config.tree_sitter {
        println!("🌳 使用 Tree-sitter 进行结构分析...");
        structural_summary =
            super::analyzer::perform_structural_analysis(&diff, &review_config.language).await?;

        if let Some(ref summary) = structural_summary {
            // 根据是否为多语言模式显示不同的统计信息
            if summary.is_multi_language() {
                println!("  ✅ 多语言结构分析完成");
                for (lang, lang_summary) in &summary.language_summaries {
                    println!(
                        "    🗺️ {}: {} 函数, {} 类, {} 注释",
                        lang,
                        lang_summary.functions.len(),
                        lang_summary.classes.len(),
                        lang_summary.comments.len()
                    );
                }
            } else {
                println!("  ✅ 结构分析完成");
                println!(
                    "    📋 {}: {} 函数, {} 类, {} 注释",
                    summary.language,
                    summary.functions.len(),
                    summary.classes.len(),
                    summary.comments.len()
                );
            }
        }
    }

    // 执行架构影响分析
    let architectural_impact =
        super::analyzer::perform_architectural_impact_analysis(&diff).await?;

    // 依赖分析与 PageRank（受 deviation_analysis 控制）
    let mut dep_details: Vec<(String, String)> = Vec::new();
    let mut dep_score_penalty: u8 = 0;
    let mut extra_findings: Vec<super::types::Finding> = Vec::new();
    let mut dep_prompt: Option<String> = None;
    if review_config.full || review_config.deviation_analysis {
        println!("🔗 正在进行依赖图与 PageRank 分析...");
        match crate::architectural_impact::graph_export::build_global_dependency_graph(
            std::path::Path::new("."),
        )
        .await
        {
            Ok(mut graph) => {
                // 计算 PageRank 并统计
                let pagerank = graph.calculate_pagerank(0.85, 20, 1e-4);
                let stats = graph.get_statistics();
                let critical = graph.identify_critical_nodes(0.15);
                dep_details.push(("dep_nodes".to_string(), stats.node_count.to_string()));
                dep_details.push(("dep_edges".to_string(), stats.edge_count.to_string()));
                dep_details.push((
                    "dep_avg_degree".to_string(),
                    format!("{:.2}", stats.avg_degree),
                ));
                dep_details.push(("dep_critical_nodes".to_string(), critical.len().to_string()));

                // Top PageRank 节点
                let mut pr_vec: Vec<(String, f32)> = pagerank.into_iter().collect();
                pr_vec.sort_by(|a, b| b.1.total_cmp(&a.1));
                let top_pr: Vec<String> = pr_vec
                    .iter()
                    .take(5)
                    .map(|(id, score)| format!("{id}:{score:.3}"))
                    .collect();
                if !top_pr.is_empty() {
                    dep_details.push(("dep_top_pagerank".to_string(), top_pr.join(", ")));
                }

                // 将变更映射到图并评估影响
                if let Some(ref impact) = architectural_impact {
                    let mut changed_ids = Vec::new();
                    let mut local_critical_hits = 0usize;
                    for fc in &impact.function_changes {
                        let id = format!("func:{}::{}", fc.file_path, fc.name);
                        if graph.nodes.contains_key(&id) {
                            changed_ids.push(id);
                        }
                    }
                    for sc in &impact.struct_changes {
                        let id = format!("class:{}::{}", sc.file_path, sc.name);
                        if graph.nodes.contains_key(&id) {
                            changed_ids.push(id);
                        }
                    }

                    let mut impacted: std::collections::HashMap<String, f32> =
                        std::collections::HashMap::new();
                    let mut impacted_set: std::collections::HashSet<String> =
                        std::collections::HashSet::new();
                    let mut critical_hits = 0usize;

                    for id in &changed_ids {
                        let cent = graph.calculate_centrality(id);
                        if cent > 0.15 {
                            critical_hits += 1;
                            local_critical_hits += 1;
                            if let Some(node) = graph.nodes.get(id) {
                                extra_findings.push(super::types::Finding {
                                    title: format!("关键节点变更: {id}"),
                                    severity: super::types::Severity::High,
                                    file_path: Some(node.metadata.file_path.clone()),
                                    line: Some(node.metadata.start_line),
                                    column: None,
                                    code_snippet: None,
                                    message: format!("变更影响关键节点，中心性 {cent:.3}"),
                                    rule_id: None,
                                    recommendation: Some("考虑回归测试与影响面评估".to_string()),
                                });
                            }
                        }
                        // 影响范围（BFS）
                        let scope = graph.calculate_impact_scope(id, 3);
                        for (nid, _) in scope {
                            impacted_set.insert(nid);
                        }
                        // 加权影响（考虑重要性与边权重）
                        for (nid, score) in graph.calculate_weighted_impact(id, 1.0, 0.85, 0.05) {
                            let prev = impacted.get(&nid).copied().unwrap_or(0.0);
                            if score > prev {
                                impacted.insert(nid, score);
                            }
                        }
                    }

                    if !changed_ids.is_empty() {
                        dep_details.push((
                            "dep_changed_nodes_mapped".to_string(),
                            changed_ids.len().to_string(),
                        ));
                    }
                    if !impacted_set.is_empty() {
                        dep_details.push((
                            "dep_impact_scope_count".to_string(),
                            impacted_set.len().to_string(),
                        ));
                    }
                    if !impacted.is_empty() {
                        let mut list: Vec<(String, f32)> = impacted.into_iter().collect();
                        list.sort_by(|a, b| b.1.total_cmp(&a.1));
                        let top_imp: Vec<String> = list
                            .iter()
                            .take(5)
                            .map(|(id, score)| format!("{id}:{score:.3}"))
                            .collect();
                        dep_details.push(("dep_top_impacted".to_string(), top_imp.join(", ")));
                    }

                    if critical_hits > 0 {
                        dep_details.push((
                            "dep_critical_impacts".to_string(),
                            critical_hits.to_string(),
                        ));
                        dep_score_penalty = dep_score_penalty
                            .saturating_add((critical_hits as u8).saturating_mul(5));
                    }

                    // 生成用于 AI 的依赖洞察文本（仅在 full 模式下拼入 prompt）
                    if review_config.full {
                        let mut lines = Vec::new();
                        lines.push(format!(
                            "图规模: {} 节点 / {} 边，平均度 {:.2}",
                            stats.node_count, stats.edge_count, stats.avg_degree
                        ));
                        lines.push(format!(
                            "关键节点数: {}，命中关键变更: {}",
                            critical.len(),
                            local_critical_hits
                        ));
                        if !top_pr.is_empty() {
                            lines.push(format!("PageRank Top: {}", top_pr.join(", ")));
                        }
                        if let Some(last) = dep_details
                            .iter()
                            .find(|(k, _)| k == "dep_top_impacted")
                            .map(|(_, v)| v.clone())
                        {
                            if !last.is_empty() {
                                lines.push(format!("影响度 Top: {last}"));
                            }
                        }
                        dep_prompt = Some(lines.join("\n"));
                    }
                }
            }
            Err(e) => {
                println!("  ⚠️ 依赖图构建失败: {e}");
            }
        }
    }

    // 如果启用了安全扫描
    #[cfg(feature = "security")]
    let mut security_findings: Vec<super::types::Finding> = Vec::new();
    #[cfg(not(feature = "security"))]
    let security_findings: Vec<super::types::Finding> = Vec::new();
    #[cfg(feature = "security")]
    if review_config.security_scan {
        println!("🔒 正在进行安全扫描...");
        let scan_result = crate::scan::run_opengrep_scan(
            config,
            std::path::Path::new("."),
            None,
            Some(60),
            false,
        )?;

        if !scan_result.findings.is_empty() {
            println!("  ⚠️  发现 {} 个安全问题", scan_result.findings.len());
            security_findings.extend(scan_result.findings.into_iter().map(Into::into));
        } else {
            println!("  ✅ 未发现安全问题");
        }
    }

    // 调用 AI 进行评审
    #[cfg(feature = "ai")]
    println!("🤖 正在调用 AI 进行代码评审...");
    #[cfg(not(feature = "ai"))]
    println!("🤖 AI 功能未启用，使用基础规则生成结果...");

    let mut prompt = format!("请对以下代码变更进行详细评审：\n\n{diff}\n\n");

    if let Some(ref summary) = structural_summary {
        prompt.push_str(&format!("\n结构分析结果：\n{summary:#?}\n"));
    }

    if let Some(ref dep_txt) = dep_prompt {
        prompt.push_str("\n依赖图关键洞察：\n");
        prompt.push_str(dep_txt);
        prompt.push('\n');
    }

    prompt.push_str("请提供：\n");
    prompt.push_str("1. 代码质量评估\n");
    prompt.push_str("2. 潜在问题和风险\n");
    prompt.push_str("3. 改进建议\n");
    prompt.push_str("4. 总体评分（1-100）\n");

    // 在存在 Issue 或启用偏离度分析时，注入 DevOps Issue 上下文
    let devops_issue_context = {
        #[cfg(feature = "devops")]
        {
            let mut s = String::new();
            if (!review_config.issue_ids.is_empty()) || review_config.deviation_analysis {
                if let Some(ref devops_cfg) = config.devops {
                    let client = crate::devops::DevOpsClient::new(devops_cfg.clone());
                    match client
                        .get_issues_with_space(
                            &review_config.issue_ids,
                            review_config.space_id.or(devops_cfg.space_id),
                        )
                        .await
                    {
                        Ok(issues) => {
                            if !issues.is_empty() {
                                use std::fmt::Write as _;
                                for issue in &issues {
                                    if let Some(ref ctx) = issue.ai_context {
                                        // 优先使用为 AI 准备的上下文摘要
                                        let _ = writeln!(&mut s, "{ctx}\n");
                                    } else {
                                        let _ = write!(
                                            &mut s,
                                            "#{} [{}] {}\n优先级: {}\n指派: {}\n标签: {}\n链接: {}\n\n",
                                            issue.id,
                                            issue.status,
                                            issue.title,
                                            issue.priority.as_deref().unwrap_or("未设置"),
                                            issue.assignee.as_deref().unwrap_or("未指派"),
                                            if issue.labels.is_empty() {
                                                "无".to_string()
                                            } else {
                                                issue.labels.join(", ")
                                            },
                                            issue.url
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("获取 DevOps Issue 失败: {e}");
                        }
                    }
                }
            }
            s
        }
        #[cfg(not(feature = "devops"))]
        {
            String::new()
        }
    };

    // 如果有 DevOps 上下文，将其追加到 prompt（非模板路径）
    if !devops_issue_context.is_empty() {
        prompt.push_str("\n相关 Issue 上下文：\n");
        prompt.push_str(&devops_issue_context);
        prompt.push('\n');
        // 当存在 Issue 上下文时，追加偏离度分析指引
        prompt.push_str("\n请基于上述 Issue 上下文，对以下方面进行偏离度分析：\n");
        prompt.push_str("1. 代码变更是否覆盖 Issue 中的关键任务点与验收标准；\n");
        prompt.push_str("2. 是否存在与 Issue 无关的改动或偏离预期的实现；\n");
        prompt.push_str("3. 给出偏离项清单与建议收敛方案；\n");
    }

    let ai_response = {
        #[cfg(feature = "ai")]
        {
            crate::ai::call_ai(config, &prompt).await?
        }
        #[cfg(not(feature = "ai"))]
        {
            // 退化为基础总结
            let mut summary_text = String::new();
            summary_text.push_str("AI 功能未启用。以下为基础分析汇总：\n");
            if let Some(ref summary_struct) = structural_summary {
                summary_text.push_str(&format!("结构分析结果：\n{summary_struct:#?}\n"));
            }
            if let Some(ref impact) = architectural_impact {
                summary_text.push_str(&format!(
                    "架构影响：风险={}\n受影响模块：{}\n破坏性变更数：{}\n",
                    impact.impact_summary.risk_level,
                    impact.impact_summary.affected_modules.join(", "),
                    impact.impact_summary.breaking_changes.len()
                ));
            }
            if !devops_issue_context.is_empty() {
                summary_text.push_str("相关 Issue 上下文：\n");
                summary_text.push_str(&devops_issue_context);
                summary_text.push('\n');
            }
            summary_text
        }
    };

    // 解析 AI 响应并构建结果
    let mut details = std::collections::HashMap::new();
    details.insert("review_result".to_string(), ai_response.clone());

    if review_config.tree_sitter {
        details.insert("tree_sitter".to_string(), "true".to_string());
    }

    // 添加架构影响分析结果
    if let Some(ref impact) = architectural_impact {
        let total_changes = impact.function_changes.len()
            + impact.struct_changes.len()
            + impact.interface_changes.len();
        details.insert("total_changes".to_string(), total_changes.to_string());
        details.insert(
            "breaking_changes_count".to_string(),
            impact.impact_summary.breaking_changes.len().to_string(),
        );
        details.insert(
            "affected_modules".to_string(),
            impact.impact_summary.affected_modules.join(", "),
        );

        // 添加风险级别
        details.insert(
            "risk_level".to_string(),
            impact.impact_summary.risk_level.clone(),
        );
    }

    // 合并依赖分析详情
    for (k, v) in dep_details {
        details.insert(k, v);
    }

    // 添加安全扫描结果
    if !security_findings.is_empty() {
        details.insert(
            "security_findings_count".to_string(),
            security_findings.len().to_string(),
        );
    }

    // 简单的评分提取（尝试从 AI 响应中找到数字）
    let mut score = extract_score_from_response(&ai_response).unwrap_or(85);

    // 根据安全问题调整评分
    if !security_findings.is_empty() {
        let critical_count = security_findings
            .iter()
            .filter(|f| {
                matches!(
                    f.severity,
                    super::types::Severity::Critical | super::types::Severity::High
                )
            })
            .count();
        score = score.saturating_sub((critical_count * 10) as u8);
    }
    // 根据依赖分析调整评分
    if dep_score_penalty > 0 {
        score = score.saturating_sub(dep_score_penalty);
    }

    // 保存缓存
    super::cache::save_cache(&cache_key, &ai_response, &review_config.language)?;

    // 合并发现（安全 + 依赖分析）
    let mut combined_findings = security_findings;
    combined_findings.extend(extra_findings);

    Ok(ReviewResult {
        success: true,
        message: "代码评审完成".to_string(),
        summary: ai_response,
        details,
        findings: combined_findings,
        score: Some(score),
        recommendations: Vec::new(),
    })
}

/// 从 AI 响应中提取评分
fn extract_score_from_response(response: &str) -> Option<u8> {
    // 简单的正则匹配，寻找类似 "评分: 85" 或 "Score: 85" 的模式
    for line in response.lines() {
        if line.contains("评分") || line.contains("Score") || line.contains("score") {
            // 提取数字
            for word in line.split_whitespace() {
                if let Ok(num) = word.trim_matches(|c: char| !c.is_numeric()).parse::<u8>() {
                    if num <= 100 {
                        return Some(num);
                    }
                }
            }
        }
    }
    None
}

/// 评审执行器（已弃用）
#[deprecated(note = "Use static functions execute_review and execute_review_with_result instead")]
pub struct ReviewExecutor {
    config: Config,
}

#[allow(deprecated)]
impl ReviewExecutor {
    #[deprecated(note = "Use static functions instead")]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    #[deprecated(note = "Use execute_review static function instead")]
    pub async fn execute(
        &self,
        review_config: ReviewConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        execute_review(&self.config, review_config).await
    }

    #[deprecated(note = "Use execute_review_with_result static function instead")]
    pub async fn execute_with_result(
        &self,
        review_config: ReviewConfig,
    ) -> Result<ReviewResult, Box<dyn std::error::Error + Send + Sync>> {
        execute_review_with_result(&self.config, review_config).await
    }
}

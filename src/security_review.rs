use crate::config::Config;
use crate::security_insights::{InsightCategory, SecurityInsight, SecurityInsights, Severity};
use crate::tree_sitter::{SupportedLanguage, TreeSitterManager};
use std::path::Path;

/// AI时代安全评审器
///
/// 专注于AI Vibe Coding时代的开发安全：
/// - 架构一致性
/// - 需求符合度  
/// - 模式合规性
/// - 安全边界保护
pub struct SecurityReviewer {
    _config: Config,
    _tree_sitter_manager: Option<TreeSitterManager>,
    _security_insights: SecurityInsights,
}

/// 评审结果
#[derive(Debug, Clone)]
pub struct SecurityReviewResult {
    pub insights: Vec<SecurityInsight>,
    pub summary: ReviewSummary,
    pub recommendations: Vec<String>,
}

/// 评审摘要
#[derive(Debug, Clone)]
pub struct ReviewSummary {
    pub total_insights: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub info_count: usize,
    pub architecture_score: f32,
    pub requirement_coverage: f32,
    pub overall_assessment: String,
}

impl SecurityReviewer {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let tree_sitter_manager = match TreeSitterManager::new().await {
            Ok(manager) => Some(manager),
            Err(_) => None,
        };

        let security_insights = SecurityInsights::new(config.clone()).await?;

        Ok(Self {
            _config: config,
            _tree_sitter_manager: tree_sitter_manager,
            _security_insights: security_insights,
        })
    }

    /// 执行安全评审
    pub async fn review_changes(
        &mut self,
        diff: &str,
        issue_ids: &[String],
        security_scan: bool,
    ) -> Result<SecurityReviewResult, Box<dyn std::error::Error + Send + Sync>> {
        println!("🔍 正在进行AI时代安全评审...");

        let mut all_insights = Vec::new();

        // 1. 分析变更的文件
        let changed_files = self.parse_changed_files(diff);

        // 2. 获取Issue上下文
        let issue_context = if !issue_ids.is_empty() {
            self.get_issue_context(issue_ids).await?
        } else {
            String::new()
        };

        // 3. 对每个变更文件进行安全分析
        for (file_path, _file_diff) in changed_files {
            println!("📁 分析文件: {}", file_path);

            // 检测文件语言
            let language = self.detect_language(&file_path);

            // 提取完整的文件内容进行分析
            if let Some(full_content) = self.get_file_content(&file_path) {
                let insights = self
                    ._security_insights
                    .analyze_code(
                        &full_content,
                        language,
                        &file_path,
                        if issue_context.is_empty() {
                            None
                        } else {
                            Some(&issue_context)
                        },
                    )
                    .await?;

                all_insights.extend(insights);
            }

            // 4. 使用Tree-sitter进行深入结构分析（如果可用）
            if let Some(full_content) = self.get_file_content(&file_path) {
                if let Some(ref mut manager) = self._tree_sitter_manager {
                    if let Ok(structural_summary) =
                        manager.analyze_structure(&full_content, language)
                    {
                        all_insights.extend(
                            self.analyze_structural_patterns(&structural_summary, &file_path),
                        );
                    }
                }
            }
        }

        // 5. 安全扫描（如果启用）
        if security_scan {
            if let Ok(scan_results) = self.perform_security_scan().await {
                all_insights.extend(scan_results);
            }
        }

        // 6. 生成评审摘要和建议
        let summary = self.generate_summary(&all_insights);
        let recommendations = self.generate_recommendations(&all_insights);

        Ok(SecurityReviewResult {
            insights: all_insights,
            summary,
            recommendations,
        })
    }

    /// 解析变更的文件
    fn parse_changed_files(&self, diff: &str) -> Vec<(String, String)> {
        let mut files = Vec::new();
        let mut current_file = String::new();
        let mut current_diff = String::new();

        for line in diff.lines() {
            if line.starts_with("diff --git") {
                // 保存上一个文件
                if !current_file.is_empty() && !current_diff.is_empty() {
                    files.push((current_file.clone(), current_diff.clone()));
                }

                // 解析新文件名
                if let Some(file_path) = line.split_whitespace().nth(2) {
                    current_file = file_path[2..].to_string(); // 移除 "b/" 前缀
                }
                current_diff.clear();
            } else if !current_file.is_empty() {
                current_diff.push_str(line);
                current_diff.push('\n');
            }
        }

        // 添加最后一个文件
        if !current_file.is_empty() && !current_diff.is_empty() {
            files.push((current_file, current_diff));
        }

        files
    }

    /// 检测文件语言
    fn detect_language(&self, file_path: &str) -> SupportedLanguage {
        let path = Path::new(file_path);
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            SupportedLanguage::from_extension(ext).unwrap_or(SupportedLanguage::Rust)
        } else {
            SupportedLanguage::Rust // 默认
        }
    }

    /// 获取文件内容
    fn get_file_content(&self, file_path: &str) -> Option<String> {
        use std::fs;
        use std::path::Path;

        let path = Path::new(file_path);
        if path.exists() {
            fs::read_to_string(path).ok()
        } else {
            None
        }
    }

    /// 获取Issue上下文
    async fn get_issue_context(
        &self,
        issue_ids: &[String],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if issue_ids.is_empty() {
            return Ok(String::new());
        }

        if let Some(devops_config) = &self._config.devops {
            let devops_client = crate::devops::DevOpsClient::new(devops_config.clone());
            let issues = devops_client.get_issues(issue_ids).await?;

            let context = issues
                .iter()
                .map(|issue| {
                    format!(
                        "Issue #{}: {}\n描述: {}\n状态: {}\n优先级: {}\n",
                        issue.id,
                        issue.title,
                        issue.description,
                        issue.status,
                        issue.priority.as_deref().unwrap_or("未设置")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\n");

            Ok(context)
        } else {
            Ok("DevOps配置未启用，无法获取Issue上下文".to_string())
        }
    }

    /// 分析结构模式
    fn analyze_structural_patterns(
        &self,
        summary: &crate::tree_sitter::StructuralSummary,
        file_path: &str,
    ) -> Vec<SecurityInsight> {
        let mut insights = Vec::new();

        // 基于结构分析的洞察
        if summary.functions.len() > 20 {
            insights.push(SecurityInsight {
                category: InsightCategory::ArchitecturalConsistency,
                severity: Severity::Medium,
                title: "函数过多".to_string(),
                description: format!(
                    "文件包含{}个函数，可能职责过于分散",
                    summary.functions.len()
                ),
                suggestion: "考虑按功能拆分文件或模块".to_string(),
                file_path: Some(file_path.to_string()),
                line_range: None,
            });
        }

        if summary.classes.len() > 5 {
            insights.push(SecurityInsight {
                category: InsightCategory::ArchitecturalConsistency,
                severity: Severity::Medium,
                title: "类过多".to_string(),
                description: format!(
                    "文件包含{}个类，可能违反单一职责原则",
                    summary.classes.len()
                ),
                suggestion: "考虑重新设计，每个文件应该只包含一个主要类".to_string(),
                file_path: Some(file_path.to_string()),
                line_range: None,
            });
        }

        // 检查复杂度
        for func in &summary.functions {
            let line_count = func.line_end.saturating_sub(func.line_start);
            if line_count > 50 {
                insights.push(SecurityInsight {
                    category: InsightCategory::PatternCompliance,
                    severity: Severity::Medium,
                    title: "函数过长".to_string(),
                    description: format!("函数{}有{}行，过于复杂", func.name, line_count),
                    suggestion: "考虑拆分函数或提取子函数".to_string(),
                    file_path: Some(file_path.to_string()),
                    line_range: Some((func.line_start, func.line_end)),
                });
            }

            if func.parameters.len() > 5 {
                insights.push(SecurityInsight {
                    category: InsightCategory::PatternCompliance,
                    severity: Severity::Low,
                    title: "参数过多".to_string(),
                    description: format!("函数{}有{}个参数", func.name, func.parameters.len()),
                    suggestion: "考虑使用参数对象或配置结构体".to_string(),
                    file_path: Some(file_path.to_string()),
                    line_range: Some((func.line_start, func.line_end)),
                });
            }
        }

        insights
    }

    /// 执行安全扫描
    async fn perform_security_scan(
        &self,
    ) -> Result<Vec<SecurityInsight>, Box<dyn std::error::Error + Send + Sync>> {
        let current_dir = std::env::current_dir()?;
        match crate::scan::run_opengrep_scan(&self._config, &current_dir, None, None, false) {
            Ok(result) => {
                let insights: Vec<SecurityInsight> = result
                    .findings
                    .into_iter()
                    .map(|f| {
                        let title = f.title.clone();
                        SecurityInsight {
                            category: InsightCategory::BoundaryProtection,
                            severity: match f.severity {
                                crate::scan::Severity::Error => Severity::Critical,
                                crate::scan::Severity::Warning => Severity::High,
                                crate::scan::Severity::Info => Severity::Medium,
                            },
                            title,
                            description: format!("发现安全问题: {}", f.title),
                            suggestion: "请修复此安全问题".to_string(),
                            file_path: Some(f.file_path.display().to_string()),
                            line_range: Some((f.line, f.line)),
                        }
                    })
                    .collect();
                Ok(insights)
            }
            Err(_) => Ok(Vec::new())
        }
    }

    /// 生成评审摘要
    fn generate_summary(&self, insights: &[SecurityInsight]) -> ReviewSummary {
        let mut critical_count = 0;
        let mut high_count = 0;
        let mut medium_count = 0;
        let mut low_count = 0;
        let mut info_count = 0;

        for insight in insights {
            match insight.severity {
                Severity::Critical => critical_count += 1,
                Severity::High => high_count += 1,
                Severity::Medium => medium_count += 1,
                Severity::Low => low_count += 1,
                Severity::Info => info_count += 1,
            }
        }

        let total_insights = insights.len();
        let architecture_score = if total_insights > 0 {
            let non_critical_high = total_insights - critical_count - high_count;
            (non_critical_high as f32 / total_insights as f32) * 100.0
        } else {
            100.0
        };

        let requirement_coverage = if total_insights > 0 {
            let requirement_issues = insights
                .iter()
                .filter(|i| i.category == InsightCategory::RequirementDeviation)
                .count();
            if requirement_issues == 0 {
                100.0
            } else {
                ((total_insights - requirement_issues) as f32 / total_insights as f32) * 100.0
            }
        } else {
            100.0
        };

        let overall_assessment = if critical_count > 0 {
            "存在严重安全问题，建议立即修复".to_string()
        } else if high_count > 0 {
            "存在高风险问题，建议尽快修复".to_string()
        } else if medium_count > 0 {
            "存在中等风险问题，建议修复".to_string()
        } else if low_count > 0 {
            "存在低风险问题，可以考虑优化".to_string()
        } else {
            "代码质量良好，符合AI时代开发安全标准".to_string()
        };

        ReviewSummary {
            total_insights,
            critical_count,
            high_count,
            medium_count,
            low_count,
            info_count,
            architecture_score,
            requirement_coverage,
            overall_assessment,
        }
    }

    /// 生成建议
    fn generate_recommendations(&self, insights: &[SecurityInsight]) -> Vec<String> {
        let mut recommendations = Vec::new();

        // 基于洞察类别生成建议
        let by_category: std::collections::HashMap<_, Vec<_>> =
            insights
                .iter()
                .fold(std::collections::HashMap::new(), |mut map, insight| {
                    map.entry(insight.category.clone())
                        .or_insert_with(Vec::new)
                        .push(insight);
                    map
                });

        if let Some(arch_insights) = by_category.get(&InsightCategory::ArchitecturalConsistency) {
            if !arch_insights.is_empty() {
                recommendations.push("🏗️ 建议重新审视代码架构，确保符合项目设计模式".to_string());
            }
        }

        if let Some(req_insights) = by_category.get(&InsightCategory::RequirementDeviation) {
            if !req_insights.is_empty() {
                recommendations
                    .push("📋 建议仔细对照Issue需求，确保代码实现完全符合要求".to_string());
            }
        }

        if let Some(pattern_insights) = by_category.get(&InsightCategory::PatternCompliance) {
            if !pattern_insights.is_empty() {
                recommendations.push("🔧 建议遵循项目的编码规范和最佳实践".to_string());
            }
        }

        if let Some(boundary_insights) = by_category.get(&InsightCategory::BoundaryProtection) {
            if !boundary_insights.is_empty() {
                recommendations.push("🛡️ 建议加强安全边界检查，避免潜在的安全风险".to_string());
            }
        }

        if recommendations.is_empty() {
            recommendations.push("✅ 代码质量良好，继续保持当前的编码标准".to_string());
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_reviewer_creation() {
        let config = Config::load().unwrap_or_default();
        let result = SecurityReviewer::new(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_changed_files() {
        let config = Config::load().unwrap_or_default();
        let reviewer = SecurityReviewer {
            _config: config,
            _tree_sitter_manager: None,
            _security_insights: SecurityInsights::new(Config::default()).await.unwrap(),
        };

        let diff = r#"
diff --git a/src/test.rs b/src/test.rs
index abc123..def456 100644
--- a/src/test.rs
+++ b/src/test.rs
@@ -1 +1,2 @@
 fn main() {
+    println!("Hello");
}
diff --git a/src/another.rs b/src/another.rs
new file mode 100644
index 000000..123456
--- /dev/null
+++ b/src/another.rs
@@ -0,0 +1 @@
+fn test() {}
        "#;

        let files = reviewer.parse_changed_files(diff);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].0, "src/test.rs");
        assert_eq!(files[1].0, "src/another.rs");
    }

    #[tokio::test]
    async fn test_detect_language() {
        let config = Config::load().unwrap_or_default();
        let reviewer = SecurityReviewer {
            _config: config,
            _tree_sitter_manager: None,
            _security_insights: SecurityInsights::new(Config::default()).await.unwrap(),
        };

        assert_eq!(reviewer.detect_language("test.rs"), SupportedLanguage::Rust);
        assert_eq!(
            reviewer.detect_language("test.py"),
            SupportedLanguage::Python
        );
        assert_eq!(
            reviewer.detect_language("test.js"),
            SupportedLanguage::JavaScript
        );
    }
}

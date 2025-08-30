use crate::config::Config;
use crate::prompt_engine::PromptEngine;
use crate::tree_sitter::SupportedLanguage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI时代开发安全洞察器
///
/// 在AI Vibe Coding时代，开发安全的含义已经扩展：
/// - 传统安全：漏洞防护、数据安全
/// - AI时代新安全：需求一致性、架构合规性、上下文保持
pub struct SecurityInsights {
    _config: Config,
    _prompt_engine: PromptEngine,
}

/// 安全洞察结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInsight {
    pub category: InsightCategory,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub suggestion: String,
    pub file_path: Option<String>,
    pub line_range: Option<(usize, usize)>,
}

/// 洞察类别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InsightCategory {
    /// 架构一致性 - AI生成的代码是否符合项目架构
    ArchitecturalConsistency,
    /// 需求偏离 - 代码是否真正解决了Issue需求
    RequirementDeviation,
    /// 依赖安全 - 新代码是否引入了危险的依赖关系
    DependencySafety,
    /// 模式合规 - 代码是否符合项目的惯用模式
    PatternCompliance,
    /// 边界保护 - 代码是否越过了安全边界
    BoundaryProtection,
}

/// 严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl SecurityInsights {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let prompt_engine = PromptEngine::new(&config).await?;
        Ok(Self {
            _config: config,
            _prompt_engine: prompt_engine,
        })
    }

    /// 分析代码的安全洞察
    pub async fn analyze_code(
        &self,
        code: &str,
        language: SupportedLanguage,
        file_path: &str,
        issue_context: Option<&str>,
    ) -> Result<Vec<SecurityInsight>, Box<dyn std::error::Error + Send + Sync>> {
        let mut insights = Vec::new();

        // 1. 架构一致性分析
        insights.extend(
            self.analyze_architectural_consistency(code, language, file_path)
                .await?,
        );

        // 2. 需求偏离分析（如果有Issue上下文）
        if let Some(issue_ctx) = issue_context {
            insights.extend(
                self.analyze_requirement_deviation(code, issue_ctx, file_path)
                    .await?,
            );
        }

        // 3. 模式合规性检查
        insights.extend(
            self.analyze_pattern_compliance(code, language, file_path)
                .await?,
        );

        // 4. 基础安全边界检查
        insights.extend(
            self.analyze_boundary_protection(code, language, file_path)
                .await?,
        );

        Ok(insights)
    }

    /// 架构一致性分析
    async fn analyze_architectural_consistency(
        &self,
        code: &str,
        language: SupportedLanguage,
        file_path: &str,
    ) -> Result<Vec<SecurityInsight>, Box<dyn std::error::Error + Send + Sync>> {
        let mut context = HashMap::new();
        context.insert("language".to_string(), language.name().to_string());
        context.insert("code".to_string(), code.to_string());

        let prompt = self
            ._prompt_engine
            .render("architectural_analysis", &context)
            .map_err(|e| format!("架构分析提示词渲染失败: {}", e))?;

        #[cfg(feature = "ai")]
        {
            match crate::ai::call_ai(&self._config, &prompt).await {
                Ok(response) => {
                    self.parse_architectural_response(&response, file_path)
                        .await
                }
                Err(e) => {
                    log::warn!("架构一致性AI分析失败: {}", e);
                    Ok(Vec::new())
                }
            }
        }
        #[cfg(not(feature = "ai"))]
        {
            log::debug!("AI feature disabled: skip architectural consistency analysis");
            Ok(Vec::new())
        }
    }

    /// 解析架构分析响应
    async fn parse_architectural_response(
        &self,
        response: &str,
        file_path: &str,
    ) -> Result<Vec<SecurityInsight>, Box<dyn std::error::Error + Send + Sync>> {
        let mut insights = Vec::new();

        // 简化的JSON解析，不依赖复杂结构体
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(issues) = json.get("issues").and_then(|v| v.as_array()) {
                for issue in issues {
                    if let (Some(title), Some(description), Some(severity)) = (
                        issue.get("title").and_then(|v| v.as_str()),
                        issue.get("description").and_then(|v| v.as_str()),
                        issue.get("severity").and_then(|v| v.as_str()),
                    ) {
                        insights.push(SecurityInsight {
                            category: InsightCategory::ArchitecturalConsistency,
                            severity: parse_severity(severity),
                            title: title.to_string(),
                            description: description.to_string(),
                            suggestion: issue
                                .get("suggestion")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            file_path: Some(file_path.to_string()),
                            line_range: None,
                        });
                    }
                }
            }
        }

        Ok(insights)
    }

    /// 需求偏离分析
    async fn analyze_requirement_deviation(
        &self,
        code: &str,
        issue_context: &str,
        file_path: &str,
    ) -> Result<Vec<SecurityInsight>, Box<dyn std::error::Error + Send + Sync>> {
        let (issue_description, acceptance_criteria) = self.parse_issue_context(issue_context);

        let mut context = HashMap::new();
        context.insert("issue_description".to_string(), issue_description);
        context.insert("acceptance_criteria".to_string(), acceptance_criteria);
        context.insert("language".to_string(), "unknown".to_string());
        context.insert("code".to_string(), code.to_string());

        let prompt = self
            ._prompt_engine
            .render("requirement_validation", &context)
            .map_err(|e| format!("需求验证提示词渲染失败: {}", e))?;

        #[cfg(feature = "ai")]
        {
            match crate::ai::call_ai(&self._config, &prompt).await {
                Ok(response) => self.parse_requirement_response(&response, file_path).await,
                Err(e) => {
                    log::warn!("需求偏离AI分析失败: {}", e);
                    Ok(Vec::new())
                }
            }
        }
        #[cfg(not(feature = "ai"))]
        {
            log::debug!("AI feature disabled: skip requirement deviation analysis");
            Ok(Vec::new())
        }
    }

    /// 解析需求分析响应
    async fn parse_requirement_response(
        &self,
        response: &str,
        file_path: &str,
    ) -> Result<Vec<SecurityInsight>, Box<dyn std::error::Error + Send + Sync>> {
        let mut insights = Vec::new();

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
            // 检查覆盖率
            if let Some(coverage) = json.get("coverage").and_then(|v| v.as_f64()) {
                if coverage < 0.8 {
                    insights.push(SecurityInsight {
                        category: InsightCategory::RequirementDeviation,
                        severity: Severity::High,
                        title: "需求覆盖率不足".to_string(),
                        description: format!("代码只覆盖了{:.0}%的Issue需求", coverage * 100.0),
                        suggestion: "请确保代码实现完全覆盖Issue的所有需求点".to_string(),
                        file_path: Some(file_path.to_string()),
                        line_range: None,
                    });
                }
            }

            // 解析偏离项
            if let Some(deviations) = json.get("deviations").and_then(|v| v.as_array()) {
                for deviation in deviations {
                    if let (Some(title), Some(description), Some(severity)) = (
                        deviation.get("title").and_then(|v| v.as_str()),
                        deviation.get("description").and_then(|v| v.as_str()),
                        deviation.get("severity").and_then(|v| v.as_str()),
                    ) {
                        insights.push(SecurityInsight {
                            category: InsightCategory::RequirementDeviation,
                            severity: parse_severity(severity),
                            title: title.to_string(),
                            description: description.to_string(),
                            suggestion: deviation
                                .get("suggestion")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            file_path: Some(file_path.to_string()),
                            line_range: None,
                        });
                    }
                }
            }
        }

        Ok(insights)
    }

    /// 解析Issue上下文
    fn parse_issue_context(&self, issue_context: &str) -> (String, String) {
        let lines: Vec<&str> = issue_context.lines().collect();
        let mut description = String::new();
        let mut criteria = String::new();
        let mut in_criteria = false;

        for line in lines {
            let line = line.trim();
            if line.starts_with("需求：") || line.starts_with("需求:") {
                description = line[3..].trim().to_string();
            } else if line.starts_with("验收标准：") || line.starts_with("验收标准:") {
                in_criteria = true;
                criteria = line[5..].trim().to_string();
            } else if in_criteria
                && (line.starts_with('-') || line.starts_with('•') || line.starts_with('*'))
            {
                if !criteria.is_empty() {
                    criteria.push('\n');
                }
                criteria.push_str(line.trim_start_matches(['-', '•', '*']).trim());
            }
        }

        (description, criteria)
    }

    /// 模式合规性检查
    async fn analyze_pattern_compliance(
        &self,
        code: &str,
        _language: SupportedLanguage,
        file_path: &str,
    ) -> Result<Vec<SecurityInsight>, Box<dyn std::error::Error + Send + Sync>> {
        let mut insights = Vec::new();

        // 基于语言的静态模式检查
        let line_count = code.lines().count();
        let lines: Vec<&str> = code.lines().collect();

        // 检查过长的函数
        if line_count > 100 {
            insights.push(SecurityInsight {
                category: InsightCategory::PatternCompliance,
                severity: Severity::Medium,
                title: "代码过长".to_string(),
                description: format!("文件有{}行，超过了推荐的100行限制", line_count),
                suggestion: "考虑将长文件拆分为多个模块或类".to_string(),
                file_path: Some(file_path.to_string()),
                line_range: None,
            });
        }

        // 检查重复代码模式
        let mut consecutive_similar_lines = 0;
        for (i, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            // 检查是否有相似的重复行
            if i > 0 && lines[i - 1].trim() == line.trim() {
                consecutive_similar_lines += 1;
            } else {
                consecutive_similar_lines = 0;
            }

            if consecutive_similar_lines >= 3 {
                insights.push(SecurityInsight {
                    category: InsightCategory::PatternCompliance,
                    severity: Severity::Low,
                    title: "可能的重复代码".to_string(),
                    description: format!("在第{}行附近发现重复的代码模式", i + 1),
                    suggestion: "考虑将重复代码提取为函数或常量".to_string(),
                    file_path: Some(file_path.to_string()),
                    line_range: Some((i.saturating_sub(3), i + 1)),
                });
                break;
            }
        }

        // 检查TODO注释
        for (i, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();
            if lower_line.contains("todo")
                || lower_line.contains("fixme")
                || lower_line.contains("hack")
            {
                insights.push(SecurityInsight {
                    category: InsightCategory::PatternCompliance,
                    severity: Severity::Info,
                    title: "待办事项标记".to_string(),
                    description: format!("第{}行包含TODO/FIXME/HACK标记", i + 1),
                    suggestion: "请在提交前处理这些待办事项".to_string(),
                    file_path: Some(file_path.to_string()),
                    line_range: Some((i + 1, i + 1)),
                });
            }
        }

        Ok(insights)
    }

    /// 边界保护检查
    async fn analyze_boundary_protection(
        &self,
        code: &str,
        language: SupportedLanguage,
        file_path: &str,
    ) -> Result<Vec<SecurityInsight>, Box<dyn std::error::Error + Send + Sync>> {
        let mut insights = Vec::new();

        let lines: Vec<&str> = code.lines().collect();

        // 检查潜在的安全风险模式
        let risk_patterns = match language {
            SupportedLanguage::JavaScript | SupportedLanguage::TypeScript => {
                vec![
                    ("eval(", "使用eval()可能存在安全风险"),
                    ("innerHTML", "直接操作innerHTML可能导致XSS攻击"),
                    ("document.write", "使用document.write可能存在安全风险"),
                ]
            }
            SupportedLanguage::Python => {
                vec![
                    ("eval(", "使用eval()可能存在安全风险"),
                    ("exec(", "使用exec()可能存在安全风险"),
                    ("subprocess.run", "使用subprocess需要注意命令注入"),
                ]
            }
            SupportedLanguage::Java => {
                vec![
                    (
                        "Runtime.getRuntime().exec",
                        "使用Runtime.exec需要注意命令注入",
                    ),
                    ("Class.forName", "动态类加载可能存在安全风险"),
                ]
            }
            _ => vec![],
        };

        for (pattern, description) in risk_patterns {
            for (i, line) in lines.iter().enumerate() {
                if line.contains(pattern) {
                    insights.push(SecurityInsight {
                        category: InsightCategory::BoundaryProtection,
                        severity: Severity::Medium,
                        title: "潜在的安全风险".to_string(),
                        description: format!("第{}行: {}", i + 1, description),
                        suggestion: "请确保这是必要的，并考虑更安全的替代方案".to_string(),
                        file_path: Some(file_path.to_string()),
                        line_range: Some((i + 1, i + 1)),
                    });
                }
            }
        }

        Ok(insights)
    }
}

/// 解析严重程度
fn parse_severity(severity_str: &str) -> Severity {
    match severity_str.to_lowercase().as_str() {
        "critical" => Severity::Critical,
        "high" => Severity::High,
        "medium" => Severity::Medium,
        "low" => Severity::Low,
        "info" => Severity::Info,
        _ => Severity::Medium,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_severity() {
        assert_eq!(parse_severity("critical"), Severity::Critical);
        assert_eq!(parse_severity("high"), Severity::High);
        assert_eq!(parse_severity("medium"), Severity::Medium);
        assert_eq!(parse_severity("low"), Severity::Low);
        assert_eq!(parse_severity("info"), Severity::Info);
        assert_eq!(parse_severity("unknown"), Severity::Medium);
    }

    #[tokio::test]
    async fn test_analyze_pattern_compliance() {
        let config = Config::load().unwrap_or_default();
        let insights = SecurityInsights::new(config).await.unwrap();

        let long_code = "fn main() {\n".repeat(150);
        let result = insights
            .analyze_pattern_compliance(&long_code, SupportedLanguage::Rust, "test.rs")
            .await
            .unwrap();

        assert!(!result.is_empty());
        assert!(result.iter().any(|i| i.title.contains("过长")));
    }

    #[tokio::test]
    async fn test_analyze_boundary_protection() {
        let config = Config::load().unwrap_or_default();
        let insights = SecurityInsights::new(config).await.unwrap();

        let js_code = r#"
        function risky() {
            eval(userInput);
            document.getElementById('output').innerHTML = userContent;
        }
        "#;

        let result = insights
            .analyze_boundary_protection(js_code, SupportedLanguage::JavaScript, "test.js")
            .await
            .unwrap();

        assert!(!result.is_empty());
        assert!(result.iter().any(|i| i.description.contains("eval")));
        assert!(result.iter().any(|i| i.description.contains("innerHTML")));
    }
}

use crate::config::Config;
use crate::devops::Issue;
use serde::{Deserialize, Serialize};

/// 分析配置 - 消除嵌套条件
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub issue_ids: Vec<String>,
    pub deviation_analysis: bool,
    pub security_scan: bool,
}

impl AnalysisConfig {
    pub fn from_flags(
        issue_id: Option<String>,
        deviation_analysis: bool,
        security_scan: bool,
    ) -> Self {
        let issue_ids = issue_id
            .map(|ids| ids.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();
        
        Self {
            issue_ids,
            deviation_analysis,
            security_scan,
        }
    }
    
    pub fn needs_issue_context(&self) -> bool {
        !self.issue_ids.is_empty() || self.deviation_analysis
    }
    
    pub fn deviation_analysis(&self) -> bool {
        self.deviation_analysis
    }
    
    pub fn has_issues(&self) -> bool {
        !self.issue_ids.is_empty()
    }
}

/// 分析上下文 - 包含所有必要信息
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    pub diff: String,
    pub issues: Vec<Issue>,
    pub config: AnalysisConfig,
    pub structural_info: Option<String>,
}

impl AnalysisContext {
    pub fn new(diff: String, issues: Vec<Issue>, config: AnalysisConfig) -> Self {
        Self { 
            diff, 
            issues, 
            config, 
            structural_info: None,
        }
    }
    
    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }
    
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
    
    /// 添加结构分析信息
    pub fn add_structural_info(&mut self, info: String) {
        self.structural_info = Some(info);
    }
    
    /// 获取结构分析信息
    pub fn structural_info(&self) -> &Option<String> {
        &self.structural_info
    }
}

/// 分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub review_result: String,
    pub security_findings: Vec<SecurityFinding>,
    pub deviation_analysis: Option<DeviationAnalysis>,
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
    pub requirement_coverage: f32,  // 需求覆盖率 0.0-1.0
    pub quality_score: f32,        // 质量评分 0.0-1.0
    pub deviations: Vec<Deviation>,
    pub suggestions: Vec<String>,
}

/// 偏离项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deviation {
    pub type_: String,      // 偏离类型
    pub description: String, // 描述
    pub severity: String,    // 严重程度
    pub suggestion: String,  // 建议
}

/// 分析器 - 专门负责代码分析
pub struct Analyzer {
    config: Config,
}

impl Analyzer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    /// 执行完整分析
    pub async fn analyze(&self, context: AnalysisContext) -> Result<AnalysisResult, Box<dyn std::error::Error + Send + Sync>> {
        let review_result = self.analyze_review(&context).await?;
        let security_findings = if context.config.security_scan {
            self.analyze_security().await?
        } else {
            Vec::new()
        };
        let deviation_analysis = if context.config.deviation_analysis && context.has_issues() {
            Some(self.analyze_deviation(&context).await?)
        } else {
            None
        };
        
        Ok(AnalysisResult {
            review_result,
            security_findings,
            deviation_analysis,
        })
    }
    
    /// 分析代码评审
    async fn analyze_review(&self, context: &AnalysisContext) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // 准备tree-sitter结构信息
        let tree_sitter_info = if let Some(ref structural_info) = context.structural_info {
            structural_info.clone()
        } else {
            "无结构分析信息".to_string()
        };
        
        // 尝试使用模板，如果失败则降级为硬编码提示词
        match crate::ai::review_code_with_template(&self.config, &context.diff, Some(&tree_sitter_info)).await {
            Ok(result) => Ok(result),
            Err(template_error) => {
                log::warn!("使用模板失败，降级为硬编码提示词: {}", template_error);
                
                // 降级为原有的硬编码逻辑
                let mut prompt = format!(
                    "请评审以下代码变更，重点关注代码质量、安全性、性能等方面：\n\n代码变更：\n{}",
                    context.diff
                );
                
                // 添加结构分析信息
                if let Some(ref structural_info) = context.structural_info {
                    prompt.push_str(&format!("\n\n{}", structural_info));
                }
                
                if context.has_issues() {
                    prompt.push_str(&format!("\n\n相关Issue信息：\n{}", context.issue_context()));
                }
                
                if context.config.deviation_analysis() && context.has_issues() {
                    prompt.push_str("\n\n请特别分析以下方面：\n");
                    prompt.push_str("1. 代码变更是否完全解决了Issue中描述的问题？\n");
                    prompt.push_str("2. 是否存在偏离需求的情况？\n");
                    prompt.push_str("3. 代码实现是否符合Issue的优先级和重要性？\n");
                    prompt.push_str("4. 是否引入了与Issue无关的代码变更？\n");
                    prompt.push_str("5. 代码质量是否满足生产环境要求？\n");
                    prompt.push_str("\n请提供偏离度分析报告，包括符合度和改进建议。");
                }
                
                crate::ai::call_ai(&self.config, &prompt).await
            }
        }
    }
    
    /// 分析安全问题
    async fn analyze_security(&self) -> Result<Vec<SecurityFinding>, Box<dyn std::error::Error + Send + Sync>> {
        let current_dir = std::env::current_dir()?;
        match crate::scan::run_opengrep_scan(&self.config, &current_dir, None, None, false) {
            Ok(result) => {
                let findings: Vec<SecurityFinding> = result.findings.into_iter().map(|f| SecurityFinding {
                    title: f.title,
                    file_path: f.file_path.display().to_string(),
                    line: f.line,
                    severity: format!("{:?}", f.severity),
                    rule_id: f.rule_id,
                    code_snippet: f.code_snippet,
                }).collect();
                Ok(findings)
            }
            Err(e) => {
                eprintln!("⚠️ 安全扫描失败: {}", e);
                Ok(Vec::new())
            }
        }
    }
    
    /// 分析偏离度
    async fn analyze_deviation(&self, context: &AnalysisContext) -> Result<DeviationAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            "分析以下代码变更与Issue需求的符合度，提供偏离度分析：\n\nIssue信息：\n{}\n\n代码变更：\n{}\n\n请分析：\n1. 需求覆盖率 (0.0-1.0)\n2. 质量评分 (0.0-1.0)\n3. 具体偏离项\n4. 改进建议",
            context.issue_context(),
            context.diff
        );
        
        let ai_response = crate::ai::call_ai(&self.config, &prompt).await?;
        
        // 简化的解析逻辑 - 实际应该更robust
        let requirement_coverage = extract_score(&ai_response, "需求覆盖率").unwrap_or(0.8);
        let quality_score = extract_score(&ai_response, "质量评分").unwrap_or(0.7);
        
        Ok(DeviationAnalysis {
            requirement_coverage,
            quality_score,
            deviations: vec![],
            suggestions: vec![ai_response],
        })
    }
}

/// 从AI响应中提取分数
fn extract_score(response: &str, key: &str) -> Option<f32> {
    // 简单的实现，实际应该用更复杂的解析
    response.lines()
        .find(|line| line.contains(key))
        .and_then(|line| line.split(':').nth(1))
        .and_then(|s| s.trim().parse::<f32>().ok())
}
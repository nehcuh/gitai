// review 执行器模块
// 负责执行评审流程的核心逻辑

use crate::config::Config;
use super::types::{ReviewConfig, ReviewResult};

/// 执行评审流程（控制台输出）
pub async fn execute_review(config: &Config, review_config: ReviewConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
            println!("  • {}", rec);
        }
    }
    
    println!("{}", "=".repeat(80));
    Ok(())
}

/// 执行评审流程并返回结构化结果
pub async fn execute_review_with_result(config: &Config, review_config: ReviewConfig) -> Result<ReviewResult, Box<dyn std::error::Error + Send + Sync>> {
    // 临时实现：返回占位结果
    Ok(ReviewResult {
        success: true,
        message: "代码评审完成".to_string(),
        summary: "评审功能正在重构中，将很快完成迁移".to_string(),
        details: std::collections::HashMap::new(),
        findings: Vec::new(),
        score: Some(80),
        recommendations: vec!["模块化重构正在进行中".to_string()],
    })
}

/// 评审执行器（已弃用）
#[deprecated(note = "Use static functions execute_review and execute_review_with_result instead")]
pub struct ReviewExecutor {
    config: Config,
}

impl ReviewExecutor {
    #[deprecated(note = "Use static functions instead")]
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    #[deprecated(note = "Use execute_review static function instead")]
    pub async fn execute(&self, review_config: ReviewConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        execute_review(&self.config, review_config).await
    }
    
    #[deprecated(note = "Use execute_review_with_result static function instead")]
    pub async fn execute_with_result(&self, review_config: ReviewConfig) -> Result<ReviewResult, Box<dyn std::error::Error + Send + Sync>> {
        execute_review_with_result(&self.config, review_config).await
    }
}

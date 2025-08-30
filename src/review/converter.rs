// review 转换器模块
// 负责将分析结果转换为评审结果

use super::types::{ReviewResult, ReviewConfig};
use crate::analysis::AnalysisResult;

/// 将分析结果转换为评审结果
pub fn convert_analysis_result(result: &AnalysisResult, _config: &ReviewConfig) -> ReviewResult {
    ReviewResult {
        success: true,
        message: "转换完成".to_string(),
        summary: result.review_result.clone(),
        details: std::collections::HashMap::new(),
        findings: Vec::new(),
        score: Some(75),
        recommendations: Vec::new(),
    }
}

/// 将分析结果转换为评审结果（带严重问题检查）
pub fn convert_analysis_result_with_critical_check(result: &AnalysisResult, config: &ReviewConfig) -> ReviewResult {
    let review_result = convert_analysis_result(result, config);
    
    // TODO: 实现严重问题检查逻辑
    
    review_result
}

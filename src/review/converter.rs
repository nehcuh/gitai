// review 转换器模块
// 负责将分析结果转换为评审结果

use super::types::{ReviewConfig, ReviewResult};
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
pub fn convert_analysis_result_with_critical_check(
    result: &AnalysisResult,
    config: &ReviewConfig,
) -> ReviewResult {
    // TODO: 实现严重问题检查逻辑

    convert_analysis_result(result, config)
}

// ===============
// 类型适配/转换
// ===============

#[cfg(feature = "security")]
impl From<crate::scan::Finding> for super::types::Finding {
    fn from(f: crate::scan::Finding) -> Self {
        super::types::Finding {
            title: f.title,
            severity: f
                .severity
                .as_str()
                .parse()
                .unwrap_or(super::types::Severity::Info),
            file_path: Some(f.file_path.to_string_lossy().to_string()),
            line: Some(f.line),
            column: Some(f.column),
            code_snippet: f.code_snippet,
            message: f.message,
            rule_id: f.rule_id,
            recommendation: f.remediation,
        }
    }
}

/// 批量转换扫描结果为评审发现
#[cfg(feature = "security")]
pub fn convert_scan_findings(findings: Vec<crate::scan::Finding>) -> Vec<super::types::Finding> {
    findings.into_iter().map(Into::into).collect()
}

// review 类型定义模块
// 所有 review 相关的数据结构定义

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 评审结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    /// 是否成功
    pub success: bool,
    /// 结果消息
    pub message: String,
    /// 简要摘要
    pub summary: String,
    /// 详细信息
    pub details: HashMap<String, String>,
    /// 发现的问题
    pub findings: Vec<Finding>,
    /// 评分 (可选)
    pub score: Option<u8>,
    /// 建议列表
    pub recommendations: Vec<String>,
}

/// 发现的问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// 问题描述
    pub title: String,
    /// 文件路径
    pub file_path: Option<String>,
    /// 行号
    pub line: Option<u32>,
    /// 严重程度
    pub severity: Severity,
    /// 详细描述
    pub description: String,
    /// 代码片段
    pub code_snippet: Option<String>,
}

/// 严重程度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// 评审配置
#[derive(Debug, Clone)]
pub struct ReviewConfig {
    pub language: Option<String>,
    pub format: String,
    pub output: Option<PathBuf>,
    pub tree_sitter: bool,
    pub security_scan: bool,
    pub scan_tool: Option<String>,
    pub block_on_critical: bool,
    pub issue_ids: Vec<String>,
    pub deviation_analysis: bool,
}

impl ReviewConfig {
    pub fn from_args(
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
        let issue_ids = issue_id
            .map(|ids| ids.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();
        
        // 当指定了 scan_tool 时自动启用 security_scan
        let security_scan = security_scan || scan_tool.is_some();
        
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
        }
    }
    
    pub fn needs_issue_context(&self) -> bool {
        !self.issue_ids.is_empty() || self.deviation_analysis
    }
    
    pub fn deviation_analysis(&self) -> bool {
        self.deviation_analysis
    }
}

/// 简化的Review缓存
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewCache {
    pub timestamp: u64,
    pub diff_hash: String,
    pub review_result: String,
    pub language: Option<String>,
}

impl ReviewCache {
    pub fn new(diff_hash: &str, review_result: String, language: Option<String>) -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            diff_hash: diff_hash.to_string(),
            review_result,
            language,
        }
    }
    
    pub fn is_expired(&self, max_age_seconds: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now.saturating_sub(self.timestamp) > max_age_seconds
    }
}

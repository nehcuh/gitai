//
// 提供安全扫描功能的 MCP 服务实现

use crate::{config::Config, mcp::*};
use log::{debug, error, info};
use rmcp::model::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// 扫描服务
pub struct ScanService {
    config: Config,
}

impl ScanService {
    /// 创建新的扫描服务
    pub fn new(config: Config) -> McpResult<Self> {
        Ok(Self { config })
    }
}

/// 扫描服务参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanParams {
    /// 要扫描的路径
    pub path: String,
    /// 扫描工具 (可选，默认 opengrep；security 等同于 opengrep)
    #[serde(default)]
    pub tool: Option<String>,
    /// 超时时间（秒）(可选，默认 300)
    #[serde(default)]
    pub timeout: Option<u64>,
    /// 语言过滤 (可选；默认自动检测并使用多语言规则)
    #[serde(default)]
    pub lang: Option<String>,
}

/// 扫描发现问题的严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// 错误级别问题
    Error,
    /// 警告级别问题
    Warning,
    /// 信息级别问题
    Info,
}

/// 扫描发现的问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// 问题标题
    pub title: String,
    /// 文件路径
    pub file_path: String,
    /// 行号
    pub line: u32,
    /// 严重程度
    pub severity: Severity,
    /// 规则ID
    pub rule_id: String,
    /// 问题描述
    pub description: String,
    /// 修复建议
    pub suggestion: Option<String>,
    /// 代码片段
    pub code_snippet: Option<String>,
}

/// 扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// 是否成功
    pub success: bool,
    /// 消息
    pub message: String,
    /// 总共发现的问题数量
    pub total_issues: usize,
    /// 错误级别问题数量
    pub error_count: usize,
    /// 警告级别问题数量
    pub warning_count: usize,
    /// 信息级别问题数量
    pub info_count: usize,
    /// 发现的问题列表
    pub findings: Vec<Finding>,
    /// 详细信息
    pub details: HashMap<String, String>,
    /// 执行时间（秒）
    pub execution_time: f64,
    /// 扫描路径
    pub path: String,
    /// 扫描工具
    pub tool: String,
    /// 扫描时间
    pub timestamp: i64,
}

#[async_trait::async_trait]
impl GitAiMcpService for ScanService {
    fn name(&self) -> &str {
        "scan"
    }

    fn description(&self) -> &str {
        "执行安全扫描，支持多语言自动检测与可选语言过滤"
    }

    fn tools(&self) -> Vec<Tool> {
        vec![Tool {
            name: "execute_scan".into(),
            description: "执行安全扫描，支持多语言自动检测与可选语言过滤".into(),
            input_schema: Arc::new(self.get_scan_schema()),
        }]
    }

    async fn handle_tool_call(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> McpResult<serde_json::Value> {
        match name {
            "execute_scan" => {
                let params: ScanParams =
                    serde_json::from_value(arguments).map_err(|e| parse_error("scan", e))?;
                debug!("扫描参数: {:?}", params);

                let result = self.execute_scan(params).await?;
                let result_json =
                    serde_json::to_value(result).map_err(|e| serialize_error("scan", e))?;
                Ok(result_json)
            }
            _ => Err(invalid_parameters_error(format!("Unknown tool: {}", name))),
        }
    }
}

impl ScanService {
    fn get_scan_schema(&self) -> serde_json::Map<String, serde_json::Value> {
        let mut schema = serde_json::Map::new();
        schema.insert("type".to_string(), serde_json::json!("object"));
        schema.insert(
            "properties".to_string(),
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "要扫描的路径"
                },
                "tool": {
                    "type": "string",
                    "description": "扫描工具 (可选，默认 opengrep；security 等同于 opengrep)",
                    "enum": ["opengrep", "security"]
                },
                "timeout": {
                    "type": "integer",
                    "description": "超时时间（秒）(可选，默认 300)"
                },
                "lang": {
                    "type": "string",
                    "description": "语言过滤 (可选；默认自动检测并使用多语言规则)"
                }
            }),
        );
        schema.insert("required".to_string(), serde_json::json!(["path"]));
        schema
    }

    async fn execute_scan(&self, params: ScanParams) -> McpResult<ScanResult> {
        info!("🔍 开始安全扫描: {}", params.path);

        #[cfg(feature = "security")]
        {
            // 检查 OpenGrep 是否已安装
            if !gitai_security::is_opengrep_installed() {
                error!("❌ OpenGrep 未安装，无法执行扫描");
                return Err(execution_failed_error(
                    "OpenGrep is not installed. Please run 'gitai scan --auto-install' first.",
                ));
            }

            // 配置扫描参数
            let config = &self.config;
            let path = Path::new(&params.path);
            let timeout = params.timeout.unwrap_or(300);
            let lang = params.lang.as_deref();

            // 执行扫描
            info!("🔄 运行 OpenGrep 扫描...");
            let result = gitai_security::run_opengrep_scan(config, path, lang, Some(timeout), true)
                .map_err(|e| execution_failed_error(format!("Failed to run scan: {}", e)))?;

            // 转换扫描结果
            let result = self.convert_real_scan_result(result, &params);
            info!("✅ 安全扫描完成: {}", params.path);
            Ok(result)
        }

        #[cfg(not(feature = "security"))]
        {
            info!("⚠️ 安全扫描功能未启用，返回模拟结果");
            Ok(self.create_mock_scan_result(params))
        }
    }

    #[cfg(feature = "security")]
    fn convert_real_scan_result(
        &self,
        scan_result: gitai_security::ScanResult,
        params: &ScanParams,
    ) -> ScanResult {
        let mut findings = Vec::new();

        for finding in &scan_result.findings {
            findings.push(Finding {
                title: finding.title.clone(),
                file_path: finding.file_path.to_string_lossy().to_string(),
                line: finding.line as u32,
                severity: match finding.severity.as_str() {
                    "ERROR" | "error" => Severity::Error,
                    "WARNING" | "warning" => Severity::Warning,
                    "INFO" | "info" => Severity::Info,
                    _ => Severity::Info,
                },
                rule_id: finding
                    .rule_id
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                description: "发现安全问题的代码段".to_string(),
                suggestion: None,
                code_snippet: finding.code_snippet.clone(),
            });
        }

        let mut details = HashMap::new();
        details.insert("tool".to_string(), scan_result.tool.clone());
        details.insert("version".to_string(), scan_result.version.clone());
        details.insert(
            "execution_time".to_string(),
            format!("{:.2}s", scan_result.execution_time),
        );

        // 添加规则信息到详情中
        let has_valid_rules = if let Some(rules_info) = &scan_result.rules_info {
            details.insert(
                "total_rules".to_string(),
                rules_info.total_rules.to_string(),
            );
            details.insert("rules_dir".to_string(), rules_info.dir.clone());
            true
        } else {
            details.insert("rules_summary".to_string(), "未加载规则".to_string());
            false
        };

        // 对问题进行分类计数
        let error_count = findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count();
        let warning_count = findings
            .iter()
            .filter(|f| f.severity == Severity::Warning)
            .count();
        let info_count = findings
            .iter()
            .filter(|f| f.severity == Severity::Info)
            .count();

        let success = scan_result.error.is_none();

        let message = if let Some(error) = &scan_result.error {
            format!("扫描失败: {}", error)
        } else if findings.is_empty() && has_valid_rules {
            "恭喜！没有发现安全问题".to_string()
        } else if findings.is_empty() && !has_valid_rules {
            "没有发现问题，但规则可能未正确加载".to_string()
        } else {
            format!(
                "发现 {} 个问题 ({} 错误, {} 警告, {} 信息)",
                findings.len(),
                error_count,
                warning_count,
                info_count
            )
        };

        ScanResult {
            success,
            message,
            total_issues: findings.len(),
            error_count,
            warning_count,
            info_count,
            findings,
            details,
            execution_time: scan_result.execution_time,
            path: params.path.clone(),
            tool: scan_result.tool,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs() as i64,
        }
    }

    #[cfg(not(feature = "security"))]
    fn create_mock_scan_result(&self, params: ScanParams) -> ScanResult {
        let mut details = HashMap::new();
        let tool_name = params
            .tool
            .clone()
            .unwrap_or_else(|| "opengrep".to_string());
        details.insert("tool".to_string(), tool_name.clone());
        details.insert("version".to_string(), "mock".to_string());
        details.insert("execution_time".to_string(), "0.1s".to_string());
        details.insert(
            "note".to_string(),
            "安全扫描功能未启用，这是一个模拟结果".to_string(),
        );

        ScanResult {
            success: true,
            message: "安全扫描功能未启用。要启用实际扫描功能，请使用 --features security 重新编译"
                .to_string(),
            total_issues: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            findings: Vec::new(),
            details,
            execution_time: 0.1,
            tool: tool_name,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs() as i64,
        }
    }
}

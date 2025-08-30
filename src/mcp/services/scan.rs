// MCP Scan 服务
//
// 提供安全扫描功能的 MCP 服务实现

use crate::{config::Config, mcp::*, scan};
use log::{debug, error, info, warn};
use rmcp::model::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Scan 服务
pub struct ScanService {
    config: Config,
    default_tool: String,
    default_timeout: u64,
}

impl ScanService {
    /// 创建新的 Scan 服务
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let (default_tool, default_timeout) = if let Some(mcp_config) = &config.mcp {
            if let Some(scan_config) = &mcp_config.services.scan {
                (
                    scan_config.default_tool.clone(),
                    scan_config.default_timeout,
                )
            } else {
                ("opengrep".to_string(), 300)
            }
        } else {
            ("opengrep".to_string(), 300)
        };

        Ok(Self {
            config,
            default_tool,
            default_timeout,
        })
    }

    /// 执行扫描
    async fn execute_scan(
        &self,
        params: ScanParams,
    ) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("🔍 开始安全扫描: {}", params.path);
        let tool = params.tool.unwrap_or_else(|| self.default_tool.clone());
        let timeout = params.timeout.unwrap_or(self.default_timeout);

        // 智能路径解析：处理相对路径和绝对路径
        let path = if Path::new(&params.path).is_absolute() {
            // 如果是绝对路径，直接使用
            PathBuf::from(&params.path)
        } else {
            // 如果是相对路径，尝试多种解析策略
            let relative_path = Path::new(&params.path);

            // 策略1：相对于当前工作目录
            let cwd_path = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(relative_path);

            if cwd_path.exists() {
                cwd_path
            } else {
                // 策略2：相对于用户主目录的 Projects 目录（常见的项目目录）
                if let Some(home) = dirs::home_dir() {
                    let home_projects_path = home.join("Projects").join(relative_path);
                    if home_projects_path.exists() {
                        home_projects_path
                    } else {
                        // 策略3：如果路径看起来像 ../xxx，尝试从 gitai 项目目录解析
                        if params.path.starts_with("../") {
                            let gitai_path = home.join("Projects/gitai").join(relative_path);
                            if gitai_path.exists() {
                                gitai_path
                            } else {
                                // 回退到原始路径
                                PathBuf::from(&params.path)
                            }
                        } else {
                            // 回退到原始路径
                            PathBuf::from(&params.path)
                        }
                    }
                } else {
                    // 无法获取主目录，使用原始路径
                    PathBuf::from(&params.path)
                }
            }
        };

        debug!(
            "📋 扫描参数: 工具={}, 超时={}s, 解析后路径={}",
            tool,
            timeout,
            path.display()
        );

        // 验证路径是否存在
        if !path.exists() {
            error!(
                "❌ 扫描路径不存在: {} (解析后: {})",
                params.path,
                path.display()
            );
            return Err(format!(
                "扫描路径不存在: {} (解析后: {})",
                params.path,
                path.display()
            )
            .into());
        }

        // 使用真实的扫描逻辑
        let scan_result = match tool.as_str() {
            "opengrep" => {
                debug!("🛡️  使用 OpenGrep 扫描工具");
                let lang = params.lang.as_deref();
                if let Some(ref lang) = lang {
                    debug!("🌐 语言过滤: {}", lang);
                }

                // 确保扫描工具已安装（与 CLI 逻辑一致）
                if !scan::is_opengrep_installed() {
                    error!("❌ OpenGrep 未安装");
                    return Err("OpenGrep 未安装，请先安装: cargo install opengrep".into());
                }

                // MCP 服务不应自动更新规则，避免超时
                // 规则更新应由用户通过 'gitai update' 命令显式触发
                debug!("🔄 使用现有扫描规则...");
                // let updater = AutoUpdater::new(self.config.clone());
                // if let Err(e) = updater.update_scan_rules().await {
                //     warn!("⚠️ 规则更新失败: {}", e);
                //     // 不返回错误，继续使用现有规则
                // }

                // 使用与 CLI 完全一致的调用方式
                let include_version = false; // 不获取版本信息以提高性能

                debug!(
                    "🔍 开始扫描: path={:?}, lang={:?}, timeout={:?}",
                    path,
                    lang,
                    Some(timeout)
                );
                let result = scan::run_opengrep_scan(
                    &self.config,
                    &path,
                    lang,
                    Some(timeout),
                    include_version,
                );

                match &result {
                    Ok(scan_result) => {
                        debug!(
                            "✅ 扫描成功: findings={}, error={:?}",
                            scan_result.findings.len(),
                            scan_result.error
                        );
                        if let Some(ref error) = scan_result.error {
                            warn!("⚠️ 扫描完成但有错误: {}", error);
                        }
                    }
                    Err(e) => {
                        error!("❌ 扫描失败: {}", e);
                        return Err(format!("扫描失败: {}", e).into());
                    }
                }

                result?
            }
            _ => {
                error!("❌ 不支持的扫描工具: {}", tool);
                return Err(format!("不支持的扫描工具: {}", tool).into());
            }
        };

        debug!("📊 扫描结果: 发现 {} 个问题", scan_result.findings.len());

        // 转换扫描结果
        let result = self.convert_scan_result(scan_result);
        info!("✅ 安全扫描完成: {}", params.path);
        Ok(result)
    }

    fn convert_scan_result(&self, scan_result: scan::ScanResult) -> ScanResult {
        let mut findings = Vec::new();

        for finding in scan_result.findings {
            findings.push(Finding {
                title: finding.title,
                file_path: finding.file_path.to_string_lossy().to_string(),
                line: finding.line,
                severity: match finding.severity.as_str() {
                    "ERROR" | "error" => Severity::Error,
                    "WARNING" | "warning" => Severity::Warning,
                    "INFO" | "info" => Severity::Info,
                    _ => Severity::Info,
                },
                rule_id: finding.rule_id.unwrap_or_else(|| "unknown".to_string()),
                description: format!("发现安全问题的代码段"),
                suggestion: None,
                code_snippet: finding.code_snippet,
            });
        }

        let mut details = HashMap::new();
        details.insert("tool".to_string(), scan_result.tool);
        details.insert("version".to_string(), scan_result.version);
        details.insert(
            "execution_time".to_string(),
            format!("{:.2}s", scan_result.execution_time),
        );

        if let Some(rules_info) = scan_result.rules_info {
            details.insert(
                "total_rules".to_string(),
                rules_info.total_rules.to_string(),
            );
            details.insert("rules_dir".to_string(), rules_info.dir);
        }

        let findings_count = findings.len();
        let severity_counts = self.count_by_severity(&findings);

        // 改进成功判断逻辑：只要能得到扫描结果就算成功
        // stderr 输出不应该导致扫描失败
        let success = scan_result.error.is_none() || !findings.is_empty();

        ScanResult {
            success,
            message: if let Some(error) = scan_result.error {
                // 如果有发现但也有错误，说明扫描部分成功
                if !findings.is_empty() {
                    format!(
                        "扫描完成，发现 {} 个问题（有警告: {}）",
                        findings_count, error
                    )
                } else {
                    format!("扫描完成，但有错误: {}", error)
                }
            } else {
                format!("扫描完成，发现 {} 个问题", findings_count)
            },
            findings,
            summary: ScanSummary {
                total_findings: findings_count,
                by_severity: severity_counts,
                execution_time: scan_result.execution_time,
            },
            details,
        }
    }

    fn count_by_severity(&self, findings: &[Finding]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        counts.insert("error".to_string(), 0);
        counts.insert("warning".to_string(), 0);
        counts.insert("info".to_string(), 0);

        for finding in findings {
            let key = match finding.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "info",
            };
            *counts.get_mut(key).unwrap() += 1;
        }

        counts
    }
}

#[async_trait::async_trait]
impl crate::mcp::GitAiMcpService for ScanService {
    fn name(&self) -> &str {
        "scan"
    }

    fn description(&self) -> &str {
        "执行安全扫描，支持多种扫描工具和语言过滤"
    }

    fn tools(&self) -> Vec<Tool> {
        vec![Tool {
            name: "execute_scan".to_string().into(),
            description: self.description().to_string().into(),
            input_schema: Arc::new(
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "要扫描的路径"
                        },
                        "tool": {
                            "type": "string",
                            "enum": ["opengrep"],
                            "description": "扫描工具 (可选，默认 opengrep)"
                        },
                        "lang": {
                            "type": "string",
                            "description": "语言过滤 (可选，如 rust, python, java)"
                        },
                        "timeout": {
                            "type": "integer",
                            "description": "超时时间（秒）(可选，默认 300)"
                        }
                    },
                    "required": ["path"]
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        }]
    }

    async fn handle_tool_call(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> crate::mcp::McpResult<serde_json::Value> {
        match name {
            "execute_scan" => {
                let params: ScanParams = serde_json::from_value(arguments)
                    .map_err(|e| crate::mcp::parse_error("scan", e))?;

                let result = self
                    .execute_scan(params)
                    .await
                    .map_err(|e| crate::mcp::execution_error("Scan", e))?;

                Ok(serde_json::to_value(result)
                    .map_err(|e| crate::mcp::serialize_error("scan", e))?)
            }
            _ => Err(invalid_parameters_error(format!("Unknown tool: {}", name))),
        }
    }
}

/// Scan 参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanParams {
    /// 扫描路径
    pub path: String,
    /// 扫描工具
    pub tool: Option<String>,
    /// 语言过滤
    pub lang: Option<String>,
    /// 超时时间
    pub timeout: Option<u64>,
}

/// Scan 结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// 是否成功
    pub success: bool,
    /// 结果消息
    pub message: String,
    /// 发现的问题
    pub findings: Vec<Finding>,
    /// 扫描摘要
    pub summary: ScanSummary,
    /// 详细信息
    pub details: HashMap<String, String>,
}

/// 扫描摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    /// 总发现数
    pub total_findings: usize,
    /// 按严重程度统计
    pub by_severity: HashMap<String, usize>,
    /// 执行时间
    pub execution_time: f64,
}

/// 安全问题发现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// 问题标题
    pub title: String,
    /// 文件路径
    pub file_path: String,
    /// 行号
    pub line: usize,
    /// 严重程度
    pub severity: Severity,
    /// 规则ID
    pub rule_id: String,
    /// 描述
    pub description: String,
    /// 修复建议
    pub suggestion: Option<String>,
    /// 代码片段
    pub code_snippet: Option<String>,
}

/// 严重程度
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

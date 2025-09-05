// MCP Review 服务
//
// 提供代码评审功能的 MCP 服务实现

use crate::{config::Config, mcp::*, review};
use rmcp::model::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Review 服务
pub struct ReviewService {
    config: Config,
    default_config: review::ReviewConfig,
}

impl ReviewService {
    /// 创建新的 Review 服务
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let default_config = if let Some(mcp_config) = &config.mcp {
            if let Some(review_config) = &mcp_config.services.review {
                review::ReviewConfig {
                    language: None,
                    format: review_config.default_format.clone(),
                    output: None,
                    tree_sitter: review_config.default_tree_sitter,
                    security_scan: review_config.default_security_scan,
                    scan_tool: None,
                    block_on_critical: false,
                    issue_ids: Vec::new(),
                    space_id: None,
                    deviation_analysis: false,
                    full: false,
                }
            } else {
                Self::default_review_config()
            }
        } else {
            Self::default_review_config()
        };

        Ok(Self {
            config,
            default_config,
        })
    }

    fn default_review_config() -> review::ReviewConfig {
        review::ReviewConfig {
            language: None,
            format: "text".to_string(),
            output: None,
            tree_sitter: false,
            security_scan: false,
            scan_tool: None,
            block_on_critical: false,
            issue_ids: Vec::new(),
            space_id: None,
            deviation_analysis: false,
            full: false,
        }
    }

    /// 执行代码评审
    async fn execute_review(
        &self,
        params: ReviewParams,
    ) -> Result<ReviewResult, Box<dyn std::error::Error + Send + Sync>> {
        // 如果指定了 path，则临时切换工作目录
        let orig_dir = std::env::current_dir().ok();
        let mut changed_dir = false;
        if let Some(ref p) = params.path {
            if !p.is_empty() {
                if let Err(e) = std::env::set_current_dir(p) {
                    log::warn!("无法切换到指定路径 '{}': {}", p, e);
                } else {
                    changed_dir = true;
                }
            }
        }

        // 构建评审配置
        let mut review_config = self.default_config.clone();

        // 应用参数覆盖
        if let Some(tree_sitter) = params.tree_sitter {
            review_config.tree_sitter = tree_sitter;
        }

        if let Some(security_scan) = params.security_scan {
            review_config.security_scan = security_scan;
        }

        if let Some(issue_ids) = params.issue_ids {
            review_config.issue_ids = issue_ids;
        }

        if let Some(space_id) = params.space_id {
            review_config.space_id = Some(space_id);
        }

        if let Some(scan_tool) = params.scan_tool {
            review_config.scan_tool = Some(scan_tool);
        }

        if let Some(deviation_analysis) = params.deviation_analysis {
            review_config.deviation_analysis = deviation_analysis;
        }

        if let Some(format) = params.format {
            review_config.format = format;
        }

        // 保存 tree_sitter 配置，因为 review_config 会被移动
        let tree_sitter_enabled = review_config.tree_sitter;

        // 执行评审
        let exec_res = review::execute_review_with_result(&self.config, review_config).await;

        // 恢复工作目录
        if changed_dir {
            if let Some(orig) = orig_dir {
                let _ = std::env::set_current_dir(orig);
            }
        }

        let review_result = exec_res?;

        // 转换为 MCP 使用的 ReviewResult 格式
        let mut details = review_result.details;

        // 检查是否有 Tree-sitter 多语言分析结果
        if tree_sitter_enabled {
            // 检查 details 中是否有多语言相关信息
            if let Some(tree_sitter_flag) = details.get("tree_sitter") {
                if tree_sitter_flag == "true" {
                    // 尝试推断是否为多语言项目
                    let has_multiple_langs = details
                        .keys()
                        .any(|k| k.contains("_functions") || k.contains("_classes"))
                        && details.keys().filter(|k| k.ends_with("_functions")).count() > 1;

                    if has_multiple_langs {
                        details.insert("analysis_mode".to_string(), "multi-language".to_string());

                        // 提取语言列表
                        let languages: Vec<String> = details
                            .keys()
                            .filter_map(|k| {
                                if k.ends_with("_functions") {
                                    Some(k.trim_end_matches("_functions").to_string())
                                } else {
                                    None
                                }
                            })
                            .collect();

                        if !languages.is_empty() {
                            details.insert("detected_languages".to_string(), languages.join(", "));
                            details
                                .insert("language_count".to_string(), languages.len().to_string());
                        }
                    } else {
                        details.insert("analysis_mode".to_string(), "single-language".to_string());
                    }
                }
            }
        }

        // 增强消息以体现多语言分析
        let enhanced_message =
            if details.get("analysis_mode") == Some(&"multi-language".to_string()) {
                if let Some(langs) = details.get("detected_languages") {
                    format!("{} (多语言项目：{})", review_result.message, langs)
                } else {
                    format!("{} (多语言项目)", review_result.message)
                }
            } else {
                review_result.message
            };

        Ok(ReviewResult {
            success: review_result.success,
            message: enhanced_message,
            details,
            findings: review_result
                .findings
                .into_iter()
                .map(|f| Finding {
                    title: f.title,
                    file_path: f.file_path,
                    line: f.line,
                    severity: match f.severity {
                        review::types::Severity::Critical => Severity::Error,
                        review::types::Severity::High => Severity::Error,
                        review::types::Severity::Medium => Severity::Warning,
                        review::types::Severity::Low => Severity::Warning,
                        review::types::Severity::Info => Severity::Info,
                        review::types::Severity::Error => Severity::Error,
                        review::types::Severity::Warning => Severity::Warning,
                    },
                    description: f.message,
                    suggestion: f.recommendation,
                })
                .collect(),
            score: review_result.score,
            recommendations: review_result.recommendations,
        })
    }
}

#[async_trait::async_trait]
impl crate::mcp::GitAiMcpService for ReviewService {
    fn name(&self) -> &str {
        "review"
    }

    fn description(&self) -> &str {
        "执行代码评审，支持多语言 Tree-sitter 结构分析、安全扫描和 Issue 关联"
    }

    fn tools(&self) -> Vec<Tool> {
        vec![Tool {
            name: "execute_review".to_string().into(),
            description: "执行代码评审，支持多语言项目（Rust、Java、Python、JavaScript、TypeScript、Go、C、C++）的 Tree-sitter 结构分析、安全扫描和 Issue 关联".to_string().into(),
            input_schema: Arc::new(
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "可选：仓库根路径（当 MCP 服务运行目录不是仓库根时需指定）"
                        },
                        "tree_sitter": {
                            "type": "boolean",
                            "description": "是否启用 Tree-sitter 多语言结构分析 (可选，默认 false)。支持自动检测和分析多种编程语言"
                        },
                        "security_scan": {
                            "type": "boolean",
                            "description": "是否启用安全扫描 (可选，默认 false)"
                        },
                        "issue_ids": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "关联的 Issue ID 列表 (可选，空数组表示不关联；提供后将隐式启用偏离度分析)"
                        },
                        "space_id": {
                            "type": "integer",
                            "description": "Coding 空间（项目）ID（可选；提供则覆盖配置 devops.space_id）"
                        },
                        "scan_tool": {
                            "type": "string",
                            "description": "使用的扫描工具 (可选，如 opengrep)"
                        },
                        "deviation_analysis": {
                            "type": "boolean",
                            "description": "是否进行偏差分析 (可选，默认 false；若未提供则在存在 issue_ids 时自动启用)"
                        },
                        "format": {
                            "type": "string",
                            "enum": ["text", "json", "markdown"],
                            "description": "输出格式 (可选，默认 text)"
                        }
                    },
                    "required": []
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
            "execute_review" => {
                let params: ReviewParams = serde_json::from_value(arguments)
                    .map_err(|e| crate::mcp::parse_error("review", e))?;

                let result = self
                    .execute_review(params)
                    .await
                    .map_err(|e| crate::mcp::execution_error("Review", e))?;

                Ok(serde_json::to_value(result)
                    .map_err(|e| crate::mcp::serialize_error("review", e))?)
            }
            _ => Err(invalid_parameters_error(format!("Unknown tool: {}", name))),
        }
    }
}

/// Review 参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewParams {
    /// 可选：指定仓库根路径（当 MCP 服务运行目录不是仓库根时需指定）
    pub path: Option<String>,
    /// 可选：Coding 空间（项目）ID；如提供将覆盖配置中的 devops.space_id
    pub space_id: Option<u64>,
    /// 是否启用 Tree-sitter 结构分析
    pub tree_sitter: Option<bool>,
    /// 是否启用安全扫描  
    pub security_scan: Option<bool>,
    /// 关联的 Issue ID 列表（传入后将隐式启用偏离度分析）
    pub issue_ids: Option<Vec<String>>,
    /// 使用的扫描工具
    pub scan_tool: Option<String>,
    /// 是否进行偏差分析（可选，若未提供则在存在 issue_ids 时自动启用）
    pub deviation_analysis: Option<bool>,
    /// 输出格式
    pub format: Option<String>,
}

/// Review 结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    /// 是否成功
    pub success: bool,
    /// 结果消息
    pub message: String,
    /// 详细信息
    pub details: HashMap<String, String>,
    /// 发现的问题
    pub findings: Vec<Finding>,
    /// 评分 (可选)
    pub score: Option<u8>,
    /// 建议列表
    pub recommendations: Vec<String>,
}

/// 问题发现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// 问题标题
    pub title: String,
    /// 严重程度
    pub severity: Severity,
    /// 文件路径
    pub file_path: Option<String>,
    /// 行号
    pub line: Option<usize>,
    /// 描述
    pub description: String,
    /// 修复建议
    pub suggestion: Option<String>,
}

/// 严重程度
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

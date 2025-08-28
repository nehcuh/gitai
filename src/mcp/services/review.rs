// MCP Review 服务
//
// 提供代码评审功能的 MCP 服务实现

use crate::{config::Config, review, mcp::*};
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
                    deviation_analysis: false,
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
            deviation_analysis: false,
        }
    }

    /// 执行代码评审
    async fn execute_review(&self, params: ReviewParams) -> Result<ReviewResult, Box<dyn std::error::Error + Send + Sync>> {
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
        
        if let Some(scan_tool) = params.scan_tool {
            review_config.scan_tool = Some(scan_tool);
        }
        
        if let Some(deviation_analysis) = params.deviation_analysis {
            review_config.deviation_analysis = deviation_analysis;
        }
        
        if let Some(format) = params.format {
            review_config.format = format;
        }

        // 执行评审
        let executor = review::ReviewExecutor::new(self.config.clone());
        
        // 由于原始 execute 方法没有返回值，我们需要适配
        // 这里我们创建一个自定义的执行器来捕获结果
        let result = self.execute_review_with_result(&executor, review_config).await?;
        
        Ok(result)
    }

    async fn execute_review_with_result(
        &self,
        executor: &review::ReviewExecutor,
        config: review::ReviewConfig,
    ) -> Result<ReviewResult, Box<dyn std::error::Error + Send + Sync>> {
        // 使用真实的业务逻辑执行评审
        let review_result = executor.execute_with_result(config).await?;
        
        // 转换为 MCP 使用的 ReviewResult 格式
        Ok(ReviewResult {
            success: review_result.success,
            message: review_result.message,
            details: review_result.details,
            findings: review_result.findings.into_iter().map(|f| Finding {
                title: f.title,
                file_path: f.file_path,
                line: f.line.map(|l| l as usize),
                severity: match f.severity {
                    review::Severity::Error => Severity::Error,
                    review::Severity::Warning => Severity::Warning,
                    review::Severity::Info => Severity::Info,
                },
                description: f.description,
                suggestion: None,
            }).collect(),
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
        "执行代码评审，支持 Tree-sitter 结构分析、安全扫描和 Issue 关联"
    }

    fn tools(&self) -> Vec<Tool> {
        vec![Tool {
            name: "execute_review".to_string().into(),
            description: self.description().to_string().into(),
            input_schema: Arc::new(serde_json::json!({
                "type": "object",
                "properties": {
                    "tree_sitter": {
                        "type": "boolean",
                        "description": "是否启用 Tree-sitter 结构分析 (可选，默认 false)"
                    },
                    "security_scan": {
                        "type": "boolean", 
                        "description": "是否启用安全扫描 (可选，默认 false)"
                    },
                    "issue_ids": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "关联的 Issue ID 列表 (可选，空数组表示不关联)"
                    },
                    "scan_tool": {
                        "type": "string",
                        "description": "使用的扫描工具 (可选，如 opengrep)"
                    },
                    "deviation_analysis": {
                        "type": "boolean",
                        "description": "是否进行偏差分析 (可选，默认 false)"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["text", "json", "markdown"],
                        "description": "输出格式 (可选，默认 text)"
                    }
                },
                "required": []
            }).as_object().unwrap().clone()),
        }]
    }

    async fn handle_tool_call(&self, name: &str, arguments: serde_json::Value) -> crate::mcp::McpResult<serde_json::Value> {
        match name {
            "execute_review" => {
                let params: ReviewParams = serde_json::from_value(arguments)
                    .map_err(|e| invalid_parameters_error(format!("Failed to parse review parameters: {}", e)))?;
                
                let result = self.execute_review(params).await
                    .map_err(|e| execution_failed_error(format!("Review execution failed: {}", e)))?;
                
                Ok(serde_json::to_value(result)
                    .map_err(|e| execution_failed_error(format!("Failed to serialize review result: {}", e)))?)
            }
            _ => Err(invalid_parameters_error(format!("Unknown tool: {}", name))),
        }
    }
}

/// Review 参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewParams {
    /// 是否启用 Tree-sitter 结构分析
    pub tree_sitter: Option<bool>,
    /// 是否启用安全扫描  
    pub security_scan: Option<bool>,
    /// 关联的 Issue ID 列表
    pub issue_ids: Option<Vec<String>>,
    /// 使用的扫描工具
    pub scan_tool: Option<String>,
    /// 是否进行偏差分析
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
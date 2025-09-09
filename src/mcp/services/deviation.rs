// 提供基于 DevOps Issue 的偏离度分析能力

use crate::{config::Config, mcp::*};
use rmcp::model::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// 偏差分析服务
pub struct DeviationService {
    #[allow(dead_code)] // Used in conditional compilation
    config: Config,
}

impl DeviationService {
    /// 创建新的偏差分析服务
    pub fn new(config: Config) -> McpResult<Self> {
        Ok(Self { config })
    }
}

/// 偏差分析服务参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviationParams {
    /// Issue ID 列表
    pub issue_ids: Vec<String>,
    /// 差异内容
    #[serde(default)]
    pub diff: Option<String>,
}

/// 偏差分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviationResult {
    /// 匹配度（0-100）
    pub match_score: i32,
    /// 偏离原因
    pub deviation_reasons: Vec<String>,
    /// 匹配到的议题 ID
    pub matched_issues: Vec<String>,
    /// 未匹配到的议题 ID
    pub unmatched_issues: Vec<String>,
    /// 详细信息
    pub details: HashMap<String, String>,
    /// 是否需要关注
    pub needs_attention: bool,
}

#[async_trait::async_trait]
impl GitAiMcpService for DeviationService {
    fn name(&self) -> &str {
        "deviation"
    }

    fn description(&self) -> &str {
        "分析代码变更与 DevOps Issue 的偏离度"
    }

    fn tools(&self) -> Vec<Tool> {
        vec![Tool {
            name: "analyze_deviation".into(),
            description: "分析代码变更与 DevOps Issue 的偏离度".into(),
            input_schema: Arc::new(self.get_schema()),
        }]
    }

    async fn handle_tool_call(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> McpResult<serde_json::Value> {
        match name {
            "analyze_deviation" => {
                let params: DeviationParams =
                    serde_json::from_value(arguments).map_err(|e| parse_error("deviation", e))?;

                let result = self.analyze_deviation(params).await?;
                Ok(serde_json::to_value(result).map_err(|e| serialize_error("deviation", e))?)
            }
            _ => Err(invalid_parameters_error(format!("Unknown tool: {}", name))),
        }
    }
}

impl DeviationService {
    fn get_schema(&self) -> serde_json::Map<String, serde_json::Value> {
        let mut schema = serde_json::Map::new();
        schema.insert("type".to_string(), serde_json::json!("object"));
        schema.insert(
            "properties".to_string(),
            serde_json::json!({
                "issue_ids": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    }
                },
                "diff": {
                    "type": "string"
                }
            }),
        );
        schema.insert("required".to_string(), serde_json::json!(["issue_ids"]));
        schema
    }

    async fn analyze_deviation(&self, params: DeviationParams) -> McpResult<DeviationResult> {
        info!("🔍 分析偏差度: {:?}", params.issue_ids);

        #[cfg(feature = "devops")]
        {
            // 这里应该是实际的偏差分析逻辑
            let devops_cfg = self
                .config
                .devops
                .clone()
                .ok_or_else(|| configuration_error("DevOps 未启用或未配置".to_string()))?;

            // 检查参数
            if params.issue_ids.is_empty() {
                return Err(invalid_parameters_error("缺少 issue_ids 参数"));
            }

            // 调用 DevOps API 获取 issues
            let client = crate::devops::DevOpsClient::new(devops_cfg.clone());
            let _issues = client.get_issues(&params.issue_ids).await?;

            // 计算匹配度和偏离度
            // 这里应该有复杂的分析逻辑

            // 返回结果
            Ok(DeviationResult {
                match_score: 85,
                deviation_reasons: vec!["有部分 Issue 要求未完全满足".to_string()],
                matched_issues: params.issue_ids.clone(),
                unmatched_issues: Vec::new(),
                details: HashMap::new(),
                needs_attention: false,
            })
        }

        #[cfg(not(feature = "devops"))]
        {
            info!("⚠️ DevOps 功能未启用，返回模拟结果");

            // 返回模拟结果
            let mut details = HashMap::new();
            details.insert(
                "note".to_string(),
                "DevOps 功能未启用，这是一个模拟结果".to_string(),
            );

            if params.issue_ids.is_empty() {
                return Err(invalid_parameters_error("缺少 issue_ids 参数"));
            }

            Ok(DeviationResult {
                match_score: 50,
                deviation_reasons: vec!["DevOps 功能未启用，无法进行实际分析".to_string()],
                matched_issues: params.issue_ids.clone(),
                unmatched_issues: Vec::new(),
                details,
                needs_attention: false,
            })
        }
    }
}

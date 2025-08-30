// MCP Deviation 服务
// 提供基于 DevOps Issue 的偏离度分析能力

use crate::{config::Config, mcp::*, devops};
use rmcp::model::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct DeviationService {
    config: Config,
}

impl DeviationService {
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self { config })
    }

    async fn analyze_deviation(&self, params: DeviationParams) -> Result<DeviationResult, Box<dyn std::error::Error + Send + Sync>> {
        // 校验依赖
        let devops_cfg = self
            .config
            .devops
            .as_ref()
            .ok_or_else(|| configuration_error("DevOps 未启用或未配置".to_string()))?;

        if params.issue_ids.is_empty() {
            return Err(invalid_parameters_error("缺少 issue_ids 参数").into());
        }

        // 拉取 issues
        let client = devops::DevOpsClient::new(devops_cfg.clone());
        let issues = client.get_issues(&params.issue_ids).await?;

        // 计算一个非常简单的“覆盖率”：基于关键词匹配
        let diff_lc = params.diff.unwrap_or_default().to_lowercase();
        let mut matched: Vec<String> = Vec::new();
        let mut unmatched: Vec<String> = Vec::new();

        for issue in &issues {
            // 基于标题和标签做关键词
            let mut keywords = vec![issue.title.to_lowercase()];
            keywords.extend(issue.labels.iter().map(|l| l.to_lowercase()));
            let mut any = false;
            for kw in keywords {
                if !kw.is_empty() && diff_lc.contains(&kw) {
                    matched.push(kw);
                    any = true;
                }
            }
            if !any {
                unmatched.push(format!("#{} {}", issue.id, issue.title));
            }
        }

        let total = (matched.len() + unmatched.len()).max(1) as f64;
        let coverage = (matched.len() as f64 / total * 100.0).round() as u8;

        Ok(DeviationResult {
            success: true,
            message: "偏离度分析完成".to_string(),
            coverage_score: coverage,
            matched_keywords: matched,
            unmatched_targets: unmatched,
        })
    }
}

#[async_trait::async_trait]
impl crate::mcp::GitAiMcpService for DeviationService {
    fn name(&self) -> &str { "deviation" }

    fn description(&self) -> &str { "基于 DevOps Issue 的偏离度分析服务" }

    fn tools(&self) -> Vec<Tool> {
        vec![Tool {
            name: "analyze_deviation".to_string().into(),
            description: "分析代码变更与 DevOps Issue 的偏离度".to_string().into(),
            input_schema: Arc::new(
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "issue_ids": {"type": "array", "items": {"type": "string"}},
                        "diff": {"type": "string"}
                    },
                    "required": ["issue_ids"]
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        }]
    }

    async fn handle_tool_call(&self, name: &str, arguments: serde_json::Value) -> crate::mcp::McpResult<serde_json::Value> {
        match name {
            "analyze_deviation" => {
                let params: DeviationParams = serde_json::from_value(arguments)
                    .map_err(|e| crate::mcp::parse_error("deviation", e))?;
                let result = self
                    .analyze_deviation(params)
                    .await
                    .map_err(|e| crate::mcp::execution_error("Deviation", e))?;
                Ok(serde_json::to_value(result)
                    .map_err(|e| crate::mcp::serialize_error("deviation", e))?)
            }
            _ => Err(invalid_parameters_error(format!("Unknown tool: {}", name))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviationParams {
    pub issue_ids: Vec<String>,
    pub diff: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviationResult {
    pub success: bool,
    pub message: String,
    pub coverage_score: u8,
    pub matched_keywords: Vec<String>,
    pub unmatched_targets: Vec<String>,
}


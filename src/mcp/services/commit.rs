// MCP Commit 服务
//
// 提供智能提交功能的 MCP 服务实现

use crate::{commit, config::Config, mcp::*};
use rmcp::model::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Commit 服务
pub struct CommitService {
    config: Config,
    default_config: commit::CommitConfig,
}

impl CommitService {
    /// Creates a new CommitService initialized from the given configuration.
    ///
    /// If `config.mcp.services.commit` is present, its default flags (`default_add_all`,
    /// `default_review`, `default_tree_sitter`) are used to build the service's default
    /// `commit::CommitConfig`. Otherwise a baseline default configuration is used.
    /// The resulting service contains the provided `config` and a `default_config`
    /// with `space_id` set to `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct a Config (application-specific) and create the service.
    /// let cfg = Config::default(); // replace with real construction
    /// let svc = CommitService::new(cfg).expect("failed to create CommitService");
    /// ```
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let default_config = if let Some(mcp_config) = &config.mcp {
            if let Some(commit_config) = &mcp_config.services.commit {
                commit::CommitConfig {
                    message: None,
                    issue_ids: Vec::new(),
                    space_id: None,
                    add_all: commit_config.default_add_all,
                    review: commit_config.default_review,
                    tree_sitter: commit_config.default_tree_sitter,
                    dry_run: false,
                }
            } else {
                Self::default_commit_config()
            }
        } else {
            Self::default_commit_config()
        };

        Ok(Self {
            config,
            default_config,
        })
    }

    /// Returns a default `commit::CommitConfig` with all fields set to their initial values.
    ///
    /// The produced config has:
    /// - `message: None`
    /// - `issue_ids: []`
    /// - `space_id: None`
    /// - `add_all: false`
    /// - `review: false`
    /// - `tree_sitter: false`
    /// - `dry_run: false`
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = default_commit_config();
    /// assert!(cfg.message.is_none());
    /// assert!(cfg.issue_ids.is_empty());
    /// assert!(cfg.space_id.is_none());
    /// assert_eq!(cfg.add_all, false);
    /// assert_eq!(cfg.review, false);
    /// assert_eq!(cfg.tree_sitter, false);
    /// assert_eq!(cfg.dry_run, false);
    /// ```
    fn default_commit_config() -> commit::CommitConfig {
        commit::CommitConfig {
            message: None,
            issue_ids: Vec::new(),
            space_id: None,
            add_all: false,
            review: false,
            tree_sitter: false,
            dry_run: false,
        }
    }

    /// 执行提交
    async fn execute_commit(
        &self,
        params: CommitParams,
    ) -> Result<CommitResult, Box<dyn std::error::Error + Send + Sync>> {
        // 构建提交配置
        let mut commit_config = self.default_config.clone();

        // 应用参数覆盖
        if let Some(message) = params.message {
            commit_config.message = Some(message);
        }

        if let Some(issue_ids) = params.issue_ids {
            commit_config.issue_ids = issue_ids;
        }

        if let Some(add_all) = params.add_all {
            commit_config.add_all = add_all;
        }

        if let Some(review) = params.review {
            commit_config.review = review;
        }

        if let Some(tree_sitter) = params.tree_sitter {
            commit_config.tree_sitter = tree_sitter;
        }

        if let Some(dry_run) = params.dry_run {
            commit_config.dry_run = dry_run;
        }

        // 执行提交
        let commit_result = commit::execute_commit_with_result(&self.config, commit_config).await?;

        // 转换为 MCP 使用的 CommitResult 格式
        let result = CommitResult {
            success: commit_result.success,
            message: commit_result.message,
            commit_hash: commit_result.commit_hash,
            changes_count: commit_result.changes_count,
            review_results: commit_result.review_results.map(|r| ReviewResults {
                score: if r.critical_issues > 0 {
                    Some(30)
                } else if r.issues_found > 0 {
                    Some(60)
                } else {
                    Some(80)
                },
                findings: (0..r.issues_found)
                    .map(|i| format!("Issue {}", i + 1))
                    .collect(),
                recommendations: if let Some(report) = r.report {
                    vec![report]
                } else {
                    vec![]
                },
            }),
            details: commit_result.details,
        };

        Ok(result)
    }
}

#[async_trait::async_trait]
impl crate::mcp::GitAiMcpService for CommitService {
    fn name(&self) -> &str {
        "commit"
    }

    fn description(&self) -> &str {
        "执行智能提交，支持 AI 生成提交信息、代码评审和 Issue 关联"
    }

    fn tools(&self) -> Vec<Tool> {
        vec![Tool {
            name: "execute_commit".to_string().into(),
            description: self.description().to_string().into(),
            input_schema: Arc::new(
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "提交信息 (可选，未指定时由 AI 生成)"
                        },
                        "issue_ids": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "关联的 Issue ID 列表 (可选，空数组表示不关联)"
                        },
                        "add_all": {
                            "type": "boolean",
                            "description": "是否添加所有修改的文件 (可选，默认 false)"
                        },
                        "review": {
                            "type": "boolean",
                            "description": "是否在提交前进行代码评审 (可选，默认 false)"
                        },
                        "tree_sitter": {
                            "type": "boolean",
                            "description": "是否在评审中启用 Tree-sitter 分析 (可选，默认 false)"
                        },
                        "dry_run": {
                            "type": "boolean",
                            "description": "是否试运行，不实际提交 (可选，默认 false)"
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
            "execute_commit" => {
                let params: CommitParams = serde_json::from_value(arguments)
                    .map_err(|e| crate::mcp::parse_error("commit", e))?;

                let result = self
                    .execute_commit(params)
                    .await
                    .map_err(|e| crate::mcp::execution_error("Commit", e))?;

                Ok(serde_json::to_value(result)
                    .map_err(|e| crate::mcp::serialize_error("commit", e))?)
            }
            _ => Err(invalid_parameters_error(format!("Unknown tool: {}", name))),
        }
    }
}

/// Commit 参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitParams {
    /// 提交信息
    pub message: Option<String>,
    /// 关联的 Issue ID 列表
    pub issue_ids: Option<Vec<String>>,
    /// 是否添加所有文件
    pub add_all: Option<bool>,
    /// 是否进行代码评审
    pub review: Option<bool>,
    /// 是否启用 Tree-sitter 分析
    pub tree_sitter: Option<bool>,
    /// 是否试运行
    pub dry_run: Option<bool>,
}

/// Commit 结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitResult {
    /// 是否成功
    pub success: bool,
    /// 结果消息
    pub message: String,
    /// 提交哈希 (成功时)
    pub commit_hash: Option<String>,
    /// 变更文件数量
    pub changes_count: usize,
    /// 评审结果 (如果启用了评审)
    pub review_results: Option<ReviewResults>,
    /// 详细信息
    pub details: HashMap<String, String>,
}

/// 评审结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResults {
    /// 评分 (0-100)
    pub score: Option<u8>,
    /// 主要发现
    pub findings: Vec<String>,
    /// 建议
    pub recommendations: Vec<String>,
}

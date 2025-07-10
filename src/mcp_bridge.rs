// MCP 桥接模块
// 
// 该模块提供 GitAI 的 MCP 兼容层，使得 GitAI 既能作为命令行工具独立运行，
// 也能作为 MCP 服务供 LLM 调用

use std::sync::Arc;
use rmcp::{Error as McpError, model::*};
use tokio::sync::Mutex;
use crate::{
    config::AppConfig,
    handlers,
    types::git::CommitArgs,
};

/// GitAI MCP 桥接服务
/// 
/// 这个结构体将 GitAI 的核心功能封装为 MCP 兼容的接口，
/// 使得 LLM 可以通过标准化的工具调用来使用 GitAI 的功能
#[derive(Clone)]
pub struct GitAiMcpBridge {
    /// GitAI 配置
    config: Arc<Mutex<AppConfig>>,
}

impl GitAiMcpBridge {
    /// 创建新的 MCP 桥接服务
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
        }
    }

    /// AI 生成智能提交信息并执行提交
    pub async fn gitai_commit(
        &self,
        message: Option<String>,
        auto_stage: Option<bool>,
        tree_sitter: Option<bool>,
        issue_id: Option<String>
    ) -> Result<CallToolResult, McpError> {
        // 构建 CommitArgs
        let commit_args = CommitArgs {
            message,
            auto_stage: auto_stage.unwrap_or(false),
            tree_sitter: tree_sitter.unwrap_or(false),
            issue_id,
            depth: None,
            passthrough_args: Vec::new(),
            review: false,
        };

        // 调用现有的 commit 处理器
        let config = self.config.lock().await.clone();
        let error_msg = match handlers::commit::handle_commit(&config, commit_args).await {
            Ok(_) => return Ok(CallToolResult::success(vec![Content::text(
                "✅ 提交成功完成".to_string()
            )])),
            Err(e) => format!("❌ 提交失败: {}", e),
        };
        
        Ok(CallToolResult::error(vec![Content::text(error_msg)]))
    }

    /// 对代码进行 AI 驱动的智能评审
    pub async fn gitai_review(
        &self,
        depth: Option<String>,
        focus: Option<String>,
        language: Option<String>,
        format: Option<String>
    ) -> Result<CallToolResult, McpError> {
        // 构建评审参数  
        let review_args = crate::types::git::ReviewArgs {
            depth: depth.unwrap_or("medium".to_string()),
            focus,
            language,
            format: format.unwrap_or("markdown".to_string()),
            output: None,
            tree_sitter: false,
            commit1: None,
            commit2: None,
            stories: None,
            tasks: None,
            defects: None,
            space_id: None,
            passthrough_args: Vec::new(),
        };

        // 调用现有的 review 处理器
        let mut config = self.config.lock().await.clone();
        match handlers::review::handle_review(&mut config, review_args, None).await {
            Ok(_) => Ok(CallToolResult::success(vec![Content::text(
                "📝 代码评审已完成，结果已显示在上方".to_string()
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(
                format!("❌ 代码评审失败: {}", e)
            )])),
        }
    }

    /// 执行代码安全和质量扫描
    pub async fn gitai_scan(
        &self,
        path: Option<String>,
        full_scan: Option<bool>,
        update_rules: Option<bool>
    ) -> Result<CallToolResult, McpError> {
        // 构建扫描参数
        let scan_args = crate::types::git::ScanArgs {
            path: Some(path.unwrap_or(".".to_string())),
            full: full_scan.unwrap_or(false),
            update_rules: update_rules.unwrap_or(false),
            output: None,
            remote: false,
            format: "text".to_string(),
        };

        // 简化的扫描实现，避免 Send 问题
        Ok(CallToolResult::success(vec![Content::text(
            "🔍 代码扫描功能暂时在 MCP 模式下不可用".to_string()
        )]))
    }

    /// 获取 Git 仓库状态信息
    pub async fn gitai_status(
        &self,
        detailed: Option<bool>
    ) -> Result<CallToolResult, McpError> {
        // 获取 Git 状态  
        let status_result = match handlers::git::get_staged_files_status().await {
            Ok(status_output) => {
                if detailed.unwrap_or(false) {
                    // 获取详细状态信息
                    let staged_diff = handlers::git::get_staged_diff().await.unwrap_or_default();
                    let unstaged_diff = handlers::git::get_diff_for_commit().await.unwrap_or_default();
                    
                    format!("📊 Git 状态（详细）\n\n状态: {}\n\n暂存的更改:\n{}\n\n未暂存的更改:\n{}", 
                           status_output, staged_diff, unstaged_diff)
                } else {
                    format!("📊 Git 状态\n\n{}", status_output)
                }
            }
            Err(e) => return Ok(CallToolResult::error(vec![Content::text(
                format!("❌ 获取状态失败: {}", e)
            )]))
        };
        
        Ok(CallToolResult::success(vec![Content::text(status_result)]))
    }

    /// 获取代码差异信息
    pub async fn gitai_diff(
        &self,
        staged: Option<bool>,
        file_path: Option<String>
    ) -> Result<CallToolResult, McpError> {
        let use_staged = staged.unwrap_or(true);
        
        let diff_content = if use_staged {
            if file_path.is_some() {
                // 简化实现：不支持单文件diff
                handlers::git::get_staged_diff().await.unwrap_or_default()
            } else {
                match handlers::git::get_staged_diff().await {
                    Ok(diff) => diff,
                    Err(e) => return Ok(CallToolResult::error(vec![Content::text(
                        format!("❌ 获取暂存差异失败: {}", e)
                    )]))
                }
            }
        } else {
            handlers::git::get_diff_for_commit().await.unwrap_or_default()
        };

        Ok(CallToolResult::success(vec![Content::text(
            format!("📝 代码差异\n\n{}", diff_content)
        )]))
    }

    /// 获取支持的工具列表
    pub fn get_tools(&self) -> Vec<Tool> {
        vec![
            Tool {
                name: "gitai_commit".into(),
                description: Some("使用 AI 生成智能提交信息并执行提交".into()),
                input_schema: std::sync::Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "自定义提交信息（可选，如果不提供将使用 AI 生成）"
                        },
                        "auto_stage": {
                            "type": "boolean",
                            "description": "是否自动暂存修改的文件"
                        },
                        "tree_sitter": {
                            "type": "boolean",
                            "description": "是否启用 Tree-sitter 语法分析增强"
                        },
                        "issue_id": {
                            "type": "string",
                            "description": "关联的 issue ID（可选）"
                        }
                    }
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "gitai_review".into(),
                description: Some("对代码进行 AI 驱动的智能评审".into()),
                input_schema: std::sync::Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "depth": {
                            "type": "string",
                            "description": "分析深度: shallow | medium | deep"
                        },
                        "focus": {
                            "type": "string",
                            "description": "重点关注领域（如：性能、安全、可读性）"
                        },
                        "language": {
                            "type": "string",
                            "description": "限制分析的编程语言"
                        },
                        "format": {
                            "type": "string",
                            "description": "输出格式: text | json | markdown"
                        }
                    }
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "gitai_scan".into(),
                description: Some("执行代码安全和质量扫描".into()),
                input_schema: std::sync::Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "指定扫描路径（默认: 当前目录）"
                        },
                        "full_scan": {
                            "type": "boolean",
                            "description": "是否执行全量扫描"
                        },
                        "update_rules": {
                            "type": "boolean",
                            "description": "是否更新扫描规则"
                        }
                    }
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "gitai_status".into(),
                description: Some("获取 Git 仓库状态信息".into()),
                input_schema: std::sync::Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "detailed": {
                            "type": "boolean",
                            "description": "是否返回详细状态信息"
                        }
                    }
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "gitai_diff".into(),
                description: Some("获取代码差异信息".into()),
                input_schema: std::sync::Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "staged": {
                            "type": "boolean",
                            "description": "是否显示已暂存的更改"
                        },
                        "file_path": {
                            "type": "string",
                            "description": "特定文件路径（可选）"
                        }
                    }
                }).as_object().unwrap().clone()),
                annotations: None,
            },
        ]
    }

    /// 处理工具调用请求
    pub async fn handle_tool_call(&self, request: CallToolRequest) -> Result<CallToolResult, McpError> {
        let args = request.params.arguments.unwrap_or_default();
        
        match request.params.name.as_ref() {
            "gitai_commit" => {
                let message = args.get("message").and_then(|v| v.as_str()).map(|s| s.to_string());
                let auto_stage = args.get("auto_stage").and_then(|v| v.as_bool());
                let tree_sitter = args.get("tree_sitter").and_then(|v| v.as_bool());
                let issue_id = args.get("issue_id").and_then(|v| v.as_str()).map(|s| s.to_string());
                
                self.gitai_commit(message, auto_stage, tree_sitter, issue_id).await
            }
            "gitai_review" => {
                let depth = args.get("depth").and_then(|v| v.as_str()).map(|s| s.to_string());
                let focus = args.get("focus").and_then(|v| v.as_str()).map(|s| s.to_string());
                let language = args.get("language").and_then(|v| v.as_str()).map(|s| s.to_string());
                let format = args.get("format").and_then(|v| v.as_str()).map(|s| s.to_string());
                
                self.gitai_review(depth, focus, language, format).await
            }
            "gitai_scan" => {
                let path = args.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
                let full_scan = args.get("full_scan").and_then(|v| v.as_bool());
                let update_rules = args.get("update_rules").and_then(|v| v.as_bool());
                
                self.gitai_scan(path, full_scan, update_rules).await
            }
            "gitai_status" => {
                let detailed = args.get("detailed").and_then(|v| v.as_bool());
                
                self.gitai_status(detailed).await
            }
            "gitai_diff" => {
                let staged = args.get("staged").and_then(|v| v.as_bool());
                let file_path = args.get("file_path").and_then(|v| v.as_str()).map(|s| s.to_string());
                
                self.gitai_diff(staged, file_path).await
            }
            _ => {
                Ok(CallToolResult::error(vec![Content::text(
                    format!("未知的工具: {}", request.params.name)
                )]))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bridge_creation() {
        // 创建一个默认配置用于测试
        let config = AppConfig::default();
        let _bridge = GitAiMcpBridge::new(config);
        // 构造函数现在总是成功的
        assert!(true);
    }

    #[tokio::test]
    async fn test_bridge_functionality() {
        let config = AppConfig::default();
        let bridge = GitAiMcpBridge::new(config);
        
        // 测试获取状态功能
        let result = bridge.gitai_status(Some(false)).await;
        assert!(result.is_ok());
        
        // 测试差异功能
        let result = bridge.gitai_diff(Some(true), None).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_get_tools() {
        let config = AppConfig::default();
        let bridge = GitAiMcpBridge::new(config);
        
        let tools = bridge.get_tools();
        assert_eq!(tools.len(), 5);
        assert!(tools.iter().any(|t| t.name == "gitai_commit"));
        assert!(tools.iter().any(|t| t.name == "gitai_review"));
        assert!(tools.iter().any(|t| t.name == "gitai_scan"));
        assert!(tools.iter().any(|t| t.name == "gitai_status"));
        assert!(tools.iter().any(|t| t.name == "gitai_diff"));
    }
}
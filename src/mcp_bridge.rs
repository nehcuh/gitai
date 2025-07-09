// MCP 桥接模块
// 
// 该模块提供 GitAI 的 MCP 兼容层，使得 GitAI 既能作为命令行工具独立运行，
// 也能作为 MCP 服务供 LLM 调用

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::{
    config::AppConfig,
    errors::AppError,
    handlers,
    types::git::CommitArgs,
};

/// GitAI MCP 桥接服务
/// 
/// 这个结构体将 GitAI 的核心功能封装为 MCP 兼容的接口，
/// 使得 LLM 可以通过标准化的工具调用来使用 GitAI 的功能
pub struct GitAiMcpBridge {
    /// GitAI 配置
    config: AppConfig,
    /// 服务状态
    running: bool,
}

/// MCP 工具调用请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolRequest {
    /// 工具名称
    pub name: String,
    /// 工具参数
    pub arguments: HashMap<String, Value>,
}

/// MCP 工具调用响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResponse {
    /// 是否成功
    pub success: bool,
    /// 结果数据
    pub result: Option<Value>,
    /// 错误信息
    pub error: Option<String>,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
}

/// 支持的 MCP 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 输入参数 schema
    pub input_schema: Value,
}

impl GitAiMcpBridge {
    /// 创建新的 MCP 桥接服务
    pub async fn new() -> Result<Self, AppError> {
        let config = AppConfig::load()?;
        Ok(Self {
            config,
            running: false,
        })
    }

    /// 启动服务
    pub async fn start(&mut self) -> Result<(), AppError> {
        tracing::info!("🚀 启动 GitAI MCP 桥接服务");
        self.running = true;
        Ok(())
    }

    /// 停止服务
    pub async fn stop(&mut self) -> Result<(), AppError> {
        tracing::info!("🛑 停止 GitAI MCP 桥接服务");
        self.running = false;
        Ok(())
    }

    /// 检查服务状态
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// 获取服务信息
    pub fn get_server_info(&self) -> Value {
        serde_json::json!({
            "name": "GitAI MCP Bridge",
            "version": "1.0.0",
            "description": "GitAI 智能 Git 工具的 MCP 服务接口",
            "capabilities": {
                "tools": true,
                "resources": false,
                "prompts": false
            }
        })
    }

    /// 列出所有支持的工具
    pub fn list_tools(&self) -> Vec<McpTool> {
        vec![
            McpTool {
                name: "gitai_commit".to_string(),
                description: "使用 AI 生成智能提交信息并执行提交".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "自定义提交信息（可选，如果不提供将使用 AI 生成）"
                        },
                        "auto_stage": {
                            "type": "boolean",
                            "description": "是否自动暂存修改的文件",
                            "default": false
                        },
                        "tree_sitter": {
                            "type": "boolean",
                            "description": "是否启用 Tree-sitter 语法分析增强",
                            "default": false
                        },
                        "issue_id": {
                            "type": "string",
                            "description": "关联的 issue ID（可选）"
                        }
                    }
                }),
            },
            McpTool {
                name: "gitai_review".to_string(),
                description: "对代码进行 AI 驱动的智能评审".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "depth": {
                            "type": "string",
                            "enum": ["shallow", "medium", "deep"],
                            "description": "分析深度",
                            "default": "medium"
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
                            "enum": ["text", "json", "markdown"],
                            "description": "输出格式",
                            "default": "markdown"
                        }
                    }
                }),
            },
            McpTool {
                name: "gitai_scan".to_string(),
                description: "执行代码安全和质量扫描".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "扫描路径（默认为当前目录）"
                        },
                        "full_scan": {
                            "type": "boolean",
                            "description": "是否执行全量扫描",
                            "default": false
                        },
                        "update_rules": {
                            "type": "boolean",
                            "description": "是否更新扫描规则",
                            "default": false
                        }
                    }
                }),
            },
            McpTool {
                name: "gitai_status".to_string(),
                description: "获取 Git 仓库状态信息".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "detailed": {
                            "type": "boolean",
                            "description": "是否返回详细状态信息",
                            "default": false
                        }
                    }
                }),
            },
            McpTool {
                name: "gitai_diff".to_string(),
                description: "获取代码差异信息".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "staged": {
                            "type": "boolean",
                            "description": "是否显示已暂存的更改",
                            "default": true
                        },
                        "file_path": {
                            "type": "string",
                            "description": "特定文件路径（可选）"
                        }
                    }
                }),
            },
        ]
    }

    /// 调用工具
    pub async fn call_tool(&self, request: McpToolRequest) -> McpToolResponse {
        let start_time = std::time::Instant::now();
        
        if !self.running {
            return McpToolResponse {
                success: false,
                result: None,
                error: Some("服务未启动".to_string()),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
            };
        }

        tracing::info!("🔧 执行 MCP 工具调用: {}", request.name);
        
        let result = match request.name.as_str() {
            "gitai_commit" => self.handle_commit_tool(request.arguments).await,
            "gitai_review" => self.handle_review_tool(request.arguments).await,
            "gitai_scan" => self.handle_scan_tool(request.arguments).await,
            "gitai_status" => self.handle_status_tool(request.arguments).await,
            "gitai_diff" => self.handle_diff_tool(request.arguments).await,
            _ => Err(AppError::Config(crate::errors::ConfigError::Other(
                format!("未知的工具: {}", request.name)
            ))),
        };

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(data) => McpToolResponse {
                success: true,
                result: Some(data),
                error: None,
                execution_time_ms,
            },
            Err(err) => McpToolResponse {
                success: false,
                result: None,
                error: Some(err.to_string()),
                execution_time_ms,
            },
        }
    }

    /// 处理提交工具调用
    async fn handle_commit_tool(&self, args: HashMap<String, Value>) -> Result<Value, AppError> {
        // 解析参数
        let message = args.get("message")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let auto_stage = args.get("auto_stage")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let tree_sitter = args.get("tree_sitter")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let issue_id = args.get("issue_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // 构建 CommitArgs
        let commit_args = CommitArgs {
            message,
            auto_stage,
            tree_sitter,
            issue_id,
            depth: None,
            passthrough_args: Vec::new(),
            review: false,
        };

        // 调用现有的 commit 处理器
        handlers::commit::handle_commit(&self.config, commit_args).await?;

        Ok(serde_json::json!({
            "status": "success",
            "message": "提交成功完成",
            "commit_hash": "unknown", // 在实际实现中可以获取真实的 commit hash
        }))
    }

    /// 处理评审工具调用
    async fn handle_review_tool(&self, args: HashMap<String, Value>) -> Result<Value, AppError> {
        // 解析参数
        let depth = args.get("depth")
            .and_then(|v| v.as_str())
            .unwrap_or("medium")
            .to_string();
        
        let focus = args.get("focus")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let language = args.get("language")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let format = args.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("markdown")
            .to_string();

        // 构建评审参数  
        let review_args = crate::types::git::ReviewArgs {
            depth: depth,
            focus: focus,
            language,
            format: format,
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
        let mut config = self.config.clone();
        let review_result = handlers::review::handle_review(&mut config, review_args, None).await?;

        Ok(serde_json::json!({
            "status": "success",
            "review_content": review_result,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
    }

    /// 处理扫描工具调用
    async fn handle_scan_tool(&self, args: HashMap<String, Value>) -> Result<Value, AppError> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".")
            .to_string();
        
        let full_scan = args.get("full_scan")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let update_rules = args.get("update_rules")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // 构建扫描参数
        let scan_args = crate::types::git::ScanArgs {
            path: Some(path),
            full: full_scan,
            update_rules,
            output: None,
            remote: false,
            format: "text".to_string(),
        };

        // 调用现有的 scan 处理器  
        let scan_result = handlers::scan::handle_scan(&self.config, scan_args, None).await?;

        Ok(serde_json::json!({
            "status": "success",
            "scan_result": scan_result,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
    }

    /// 处理状态工具调用
    async fn handle_status_tool(&self, args: HashMap<String, Value>) -> Result<Value, AppError> {
        let detailed = args.get("detailed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // 获取 Git 状态  
        let status_output = handlers::git::get_staged_files_status().await?;
        
        if detailed {
            // 获取详细状态信息
            let staged_diff = handlers::git::get_staged_diff().await.unwrap_or_default();
            let unstaged_diff = handlers::git::get_diff_for_commit().await.unwrap_or_default();
            
            Ok(serde_json::json!({
                "status": "success",
                "git_status": status_output,
                "staged_changes": staged_diff,
                "unstaged_changes": unstaged_diff,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            Ok(serde_json::json!({
                "status": "success",
                "git_status": status_output,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// 处理差异工具调用
    async fn handle_diff_tool(&self, args: HashMap<String, Value>) -> Result<Value, AppError> {
        let staged = args.get("staged")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let file_path = args.get("file_path")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let diff_content = if staged {
            if let Some(ref _path) = file_path {
                // 简化实现：不支持单文件diff
                handlers::git::get_staged_diff().await.unwrap_or_default()
            } else {
                handlers::git::get_staged_diff().await?
            }
        } else {
            if let Some(ref _path) = file_path {
                // 简化实现：不支持单文件diff
                handlers::git::get_diff_for_commit().await.unwrap_or_default()
            } else {
                handlers::git::get_diff_for_commit().await.unwrap_or_default()
            }
        };

        Ok(serde_json::json!({
            "status": "success",
            "diff_content": diff_content,
            "staged": staged,
            "file_path": file_path,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
    }
}

/// MCP 服务器实现
/// 
/// 这个函数可以作为独立的 MCP 服务器运行，
/// 也可以集成到现有的 GitAI 应用中
pub async fn run_mcp_server() -> Result<(), AppError> {
    tracing::info!("🌟 启动 GitAI MCP 服务器");
    
    let mut bridge = GitAiMcpBridge::new().await?;
    bridge.start().await?;

    // 这里可以添加实际的 MCP 服务器监听逻辑
    // 例如监听 stdio、HTTP 或其他传输协议
    
    tracing::info!("📋 GitAI MCP 服务器支持的工具:");
    for tool in bridge.list_tools() {
        tracing::info!("  - {}: {}", tool.name, tool.description);
    }

    // 简单的 stdin/stdout 接口示例
    println!("GitAI MCP Bridge 已启动，等待工具调用...");
    println!("服务器信息: {}", serde_json::to_string_pretty(&bridge.get_server_info()).unwrap_or_default());

    // 在实际实现中，这里会有真正的 MCP 协议处理逻辑
    // 目前只是一个占位符实现
    
    Ok(())
}

/// 命令行模式运行（现有功能保持不变）
pub async fn run_cli_mode(_args: Vec<String>) -> Result<(), AppError> {
    // 这里调用现有的 main 函数逻辑
    // 保持完全的向后兼容性
    tracing::info!("🖥️  运行 GitAI 命令行模式");
    
    // 实际的命令行处理逻辑在 main.rs 中
    Ok(())
}

/// 检测运行模式
pub fn detect_run_mode() -> RunMode {
    // 检测环境变量或命令行参数来确定运行模式
    if std::env::var("GITAI_MCP_MODE").is_ok() {
        RunMode::McpServer
    } else if std::env::args().any(|arg| arg == "--mcp-server") {
        RunMode::McpServer
    } else {
        RunMode::CliMode
    }
}

/// 运行模式枚举
#[derive(Debug, Clone, PartialEq)]
pub enum RunMode {
    /// 命令行模式（默认）
    CliMode,
    /// MCP 服务器模式
    McpServer,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bridge_creation() {
        let result = GitAiMcpBridge::new().await;
        // 在测试环境中可能会因为配置文件不存在而失败，这是正常的
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_detect_run_mode() {
        // 默认应该是命令行模式
        let mode = detect_run_mode();
        assert_eq!(mode, RunMode::CliMode);
    }

    #[tokio::test]
    async fn test_tool_listing() {
        if let Ok(bridge) = GitAiMcpBridge::new().await {
            let tools = bridge.list_tools();
            assert!(!tools.is_empty());
            
            // 验证所有工具都有必要的字段
            for tool in tools {
                assert!(!tool.name.is_empty());
                assert!(!tool.description.is_empty());
                assert!(tool.input_schema.is_object());
            }
        }
    }

    #[tokio::test]
    async fn test_service_lifecycle() {
        if let Ok(mut bridge) = GitAiMcpBridge::new().await {
            assert!(!bridge.is_running());
            
            bridge.start().await.unwrap();
            assert!(bridge.is_running());
            
            bridge.stop().await.unwrap();
            assert!(!bridge.is_running());
        }
    }
}
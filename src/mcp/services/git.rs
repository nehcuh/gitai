// Git 原生能力 MCP 服务
// 
// 封装 git 原生命令为 MCP 工具，提供完整的 git 操作能力

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use serde::{Deserialize, Serialize};
use crate::mcp::{McpService, RmcpResult, RmcpError};
use rmcp::model::{Tool, Resource};
use rmcp::handler::server::{ServerHandler, ToolCallParams, ToolCallResult};

/// Git MCP 服务
pub struct GitService {
    name: String,
    version: String,
    description: String,
    repository_path: Option<PathBuf>,
    is_running: bool,
}

/// Git 操作参数
#[derive(Debug, Serialize, Deserialize)]
pub struct GitOperationParams {
    /// Git 命令
    pub command: String,
    /// 命令参数
    pub args: Vec<String>,
    /// 工作目录（可选）
    pub working_dir: Option<String>,
}

/// Git 操作结果
#[derive(Debug, Serialize, Deserialize)]
pub struct GitOperationResult {
    /// 是否成功
    pub success: bool,
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 退出码
    pub exit_code: Option<i32>,
}

/// Git 状态信息
#[derive(Debug, Serialize, Deserialize)]
pub struct GitStatus {
    /// 当前分支
    pub current_branch: Option<String>,
    /// 是否有未提交的更改
    pub has_changes: bool,
    /// 修改的文件
    pub modified_files: Vec<String>,
    /// 新增的文件
    pub added_files: Vec<String>,
    /// 删除的文件
    pub deleted_files: Vec<String>,
    /// 未跟踪的文件
    pub untracked_files: Vec<String>,
}

impl GitService {
    /// 创建新的 Git 服务
    pub fn new(repository_path: Option<PathBuf>) -> Self {
        Self {
            name: "gitai-git-service".to_string(),
            version: "1.0.0".to_string(),
            description: "GitAI Git 原生能力服务".to_string(),
            repository_path,
            is_running: false,
        }
    }

    /// 执行 git 命令
    fn execute_git_command(&self, command: &str, args: &[String], working_dir: Option<&str>) -> GitOperationResult {
        let mut cmd = Command::new("git");
        cmd.arg(command);
        cmd.args(args);

        // 设置工作目录
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        } else if let Some(repo_path) = &self.repository_path {
            cmd.current_dir(repo_path);
        }

        match cmd.output() {
            Ok(output) => GitOperationResult {
                success: output.status.success(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code(),
            },
            Err(e) => GitOperationResult {
                success: false,
                stdout: String::new(),
                stderr: format!("执行 git 命令失败: {}", e),
                exit_code: None,
            },
        }
    }

    /// 获取 git 状态
    fn get_git_status(&self, working_dir: Option<&str>) -> RmcpResult<GitStatus> {
        // 获取当前分支
        let branch_result = self.execute_git_command("branch", &["--show-current".to_string()], working_dir);
        let current_branch = if branch_result.success {
            let branch_name = branch_result.stdout.trim();
            if branch_name.is_empty() { None } else { Some(branch_name.to_string()) }
        } else {
            None
        };

        // 获取状态信息
        let status_result = self.execute_git_command("status", &["--porcelain".to_string()], working_dir);
        if !status_result.success {
            return Err(RmcpError::internal_error(format!("获取 git 状态失败: {}", status_result.stderr)));
        }

        let mut modified_files = Vec::new();
        let mut added_files = Vec::new();
        let mut deleted_files = Vec::new();
        let mut untracked_files = Vec::new();

        for line in status_result.stdout.lines() {
            if line.len() < 3 {
                continue;
            }

            let status_chars = &line[0..2];
            let file_path = &line[3..];

            match status_chars {
                " M" | "M " | "MM" => modified_files.push(file_path.to_string()),
                "A " | "AM" => added_files.push(file_path.to_string()),
                " D" | "D " => deleted_files.push(file_path.to_string()),
                "??" => untracked_files.push(file_path.to_string()),
                _ => {}
            }
        }

        let has_changes = !modified_files.is_empty() || !added_files.is_empty() || 
                         !deleted_files.is_empty() || !untracked_files.is_empty();

        Ok(GitStatus {
            current_branch,
            has_changes,
            modified_files,
            added_files,
            deleted_files,
            untracked_files,
        })
    }

    /// 获取提供的工具列表
    fn get_tools(&self) -> Vec<Tool> {
        vec![
            Tool {
                name: "git_execute".to_string(),
                description: "执行任意 git 命令".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "Git 命令（不包含 'git' 前缀）"
                        },
                        "args": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "命令参数"
                        },
                        "working_dir": {
                            "type": "string",
                            "description": "工作目录（可选）"
                        }
                    },
                    "required": ["command"]
                }),
            },
            Tool {
                name: "git_status".to_string(),
                description: "获取 git 仓库状态信息".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "working_dir": {
                            "type": "string",
                            "description": "工作目录（可选）"
                        }
                    }
                }),
            },
            Tool {
                name: "git_log".to_string(),
                description: "获取 git 提交历史".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": {
                            "type": "integer",
                            "description": "限制显示的提交数量",
                            "default": 10
                        },
                        "oneline": {
                            "type": "boolean",
                            "description": "是否使用简洁格式",
                            "default": true
                        },
                        "working_dir": {
                            "type": "string",
                            "description": "工作目录（可选）"
                        }
                    }
                }),
            },
            Tool {
                name: "git_diff".to_string(),
                description: "显示文件差异".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file": {
                            "type": "string",
                            "description": "特定文件路径（可选）"
                        },
                        "staged": {
                            "type": "boolean",
                            "description": "是否显示暂存区差异",
                            "default": false
                        },
                        "working_dir": {
                            "type": "string",
                            "description": "工作目录（可选）"
                        }
                    }
                }),
            },
            Tool {
                name: "git_add".to_string(),
                description: "添加文件到暂存区".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "files": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "要添加的文件路径列表"
                        },
                        "all": {
                            "type": "boolean",
                            "description": "是否添加所有修改的文件",
                            "default": false
                        },
                        "working_dir": {
                            "type": "string",
                            "description": "工作目录（可选）"
                        }
                    }
                }),
            },
            Tool {
                name: "git_commit".to_string(),
                description: "提交更改".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "提交消息"
                        },
                        "amend": {
                            "type": "boolean",
                            "description": "是否修改上次提交",
                            "default": false
                        },
                        "working_dir": {
                            "type": "string",
                            "description": "工作目录（可选）"
                        }
                    },
                    "required": ["message"]
                }),
            },
            Tool {
                name: "git_branch".to_string(),
                description: "分支操作".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["list", "create", "delete", "checkout"],
                            "description": "分支操作类型"
                        },
                        "branch_name": {
                            "type": "string",
                            "description": "分支名称（create、delete、checkout 时必需）"
                        },
                        "force": {
                            "type": "boolean",
                            "description": "是否强制操作",
                            "default": false
                        },
                        "working_dir": {
                            "type": "string",
                            "description": "工作目录（可选）"
                        }
                    },
                    "required": ["action"]
                }),
            },
            Tool {
                name: "git_remote".to_string(),
                description: "远程仓库操作".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["list", "add", "remove", "set-url"],
                            "description": "远程仓库操作类型"
                        },
                        "name": {
                            "type": "string",
                            "description": "远程仓库名称"
                        },
                        "url": {
                            "type": "string",
                            "description": "远程仓库 URL"
                        },
                        "working_dir": {
                            "type": "string",
                            "description": "工作目录（可选）"
                        }
                    },
                    "required": ["action"]
                }),
            },
            Tool {
                name: "git_push".to_string(),
                description: "推送到远程仓库".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "remote": {
                            "type": "string",
                            "description": "远程仓库名称",
                            "default": "origin"
                        },
                        "branch": {
                            "type": "string",
                            "description": "分支名称（可选）"
                        },
                        "force": {
                            "type": "boolean",
                            "description": "是否强制推送",
                            "default": false
                        },
                        "working_dir": {
                            "type": "string",
                            "description": "工作目录（可选）"
                        }
                    }
                }),
            },
            Tool {
                name: "git_pull".to_string(),
                description: "从远程仓库拉取".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "remote": {
                            "type": "string",
                            "description": "远程仓库名称",
                            "default": "origin"
                        },
                        "branch": {
                            "type": "string",
                            "description": "分支名称（可选）"
                        },
                        "rebase": {
                            "type": "boolean",
                            "description": "是否使用 rebase",
                            "default": false
                        },
                        "working_dir": {
                            "type": "string",
                            "description": "工作目录（可选）"
                        }
                    }
                }),
            },
        ]
    }
}

impl McpService for GitService {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn start_sync(&mut self) -> RmcpResult<()> {
        // 检查 git 是否可用
        let result = Command::new("git").arg("--version").output();
        match result {
            Ok(output) if output.status.success() => {
                self.is_running = true;
                tracing::info!("Git 服务启动成功");
                Ok(())
            }
            Ok(_) => {
                Err(RmcpError::internal_error("Git 命令不可用".to_string()))
            }
            Err(e) => {
                Err(RmcpError::internal_error(format!("启动 Git 服务失败: {}", e)))
            }
        }
    }

    fn stop_sync(&mut self) -> RmcpResult<()> {
        self.is_running = false;
        tracing::info!("Git 服务已停止");
        Ok(())
    }

    fn health_check_sync(&self) -> RmcpResult<bool> {
        Ok(self.is_running)
    }

    fn get_handler_info(&self) -> String {
        "GitServiceHandler".to_string()
    }
}

/// Git 服务的 MCP 处理器
pub struct GitServiceHandler {
    service: GitService,
}

impl GitServiceHandler {
    pub fn new(repository_path: Option<PathBuf>) -> Self {
        Self {
            service: GitService::new(repository_path),
        }
    }
}

#[async_trait::async_trait]
impl ServerHandler for GitServiceHandler {
    async fn list_tools(&self) -> Result<Vec<Tool>, rmcp::service::ServiceError> {
        Ok(self.service.get_tools())
    }

    async fn list_resources(&self) -> Result<Vec<Resource>, rmcp::service::ServiceError> {
        // Git 服务主要提供工具，不提供资源
        Ok(vec![])
    }

    async fn call_tool(&self, name: &str, params: ToolCallParams) -> Result<ToolCallResult, rmcp::service::ServiceError> {
        match name {
            "git_execute" => {
                let git_params: GitOperationParams = serde_json::from_value(params.arguments)
                    .map_err(|e| rmcp::service::ServiceError::invalid_params(format!("参数解析失败: {}", e)))?;
                
                let result = self.service.execute_git_command(
                    &git_params.command,
                    &git_params.args,
                    git_params.working_dir.as_deref(),
                );
                
                Ok(ToolCallResult {
                    content: vec![rmcp::model::Content::Text {
                        text: serde_json::to_string_pretty(&result)
                            .map_err(|e| rmcp::service::ServiceError::internal_error(format!("序列化结果失败: {}", e)))?,
                    }],
                    is_error: !result.success,
                })
            }
            "git_status" => {
                let params: serde_json::Value = params.arguments;
                let working_dir = params.get("working_dir").and_then(|v| v.as_str());
                
                let status = self.service.get_git_status(working_dir)
                    .map_err(|e| rmcp::service::ServiceError::internal_error(format!("获取 git 状态失败: {:?}", e)))?;
                
                Ok(ToolCallResult {
                    content: vec![rmcp::model::Content::Text {
                        text: serde_json::to_string_pretty(&status)
                            .map_err(|e| rmcp::service::ServiceError::internal_error(format!("序列化状态失败: {}", e)))?,
                    }],
                    is_error: false,
                })
            }
            "git_log" => {
                let params: serde_json::Value = params.arguments;
                let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(10);
                let oneline = params.get("oneline").and_then(|v| v.as_bool()).unwrap_or(true);
                let working_dir = params.get("working_dir").and_then(|v| v.as_str());
                
                let mut args = vec![format!("-{}", limit)];
                if oneline {
                    args.push("--oneline".to_string());
                }
                
                let result = self.service.execute_git_command("log", &args, working_dir);
                
                Ok(ToolCallResult {
                    content: vec![rmcp::model::Content::Text {
                        text: serde_json::to_string_pretty(&result)
                            .map_err(|e| rmcp::service::ServiceError::internal_error(format!("序列化结果失败: {}", e)))?,
                    }],
                    is_error: !result.success,
                })
            }
            "git_diff" => {
                let params: serde_json::Value = params.arguments;
                let file = params.get("file").and_then(|v| v.as_str());
                let staged = params.get("staged").and_then(|v| v.as_bool()).unwrap_or(false);
                let working_dir = params.get("working_dir").and_then(|v| v.as_str());
                
                let mut args = Vec::new();
                if staged {
                    args.push("--staged".to_string());
                }
                if let Some(f) = file {
                    args.push(f.to_string());
                }
                
                let result = self.service.execute_git_command("diff", &args, working_dir);
                
                Ok(ToolCallResult {
                    content: vec![rmcp::model::Content::Text {
                        text: serde_json::to_string_pretty(&result)
                            .map_err(|e| rmcp::service::ServiceError::internal_error(format!("序列化结果失败: {}", e)))?,
                    }],
                    is_error: !result.success,
                })
            }
            "git_add" => {
                let params: serde_json::Value = params.arguments;
                let files = params.get("files")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>())
                    .unwrap_or_default();
                let all = params.get("all").and_then(|v| v.as_bool()).unwrap_or(false);
                let working_dir = params.get("working_dir").and_then(|v| v.as_str());
                
                let args = if all {
                    vec!["-A".to_string()]
                } else if files.is_empty() {
                    return Err(rmcp::service::ServiceError::invalid_params("必须指定文件或使用 all=true".to_string()));
                } else {
                    files
                };
                
                let result = self.service.execute_git_command("add", &args, working_dir);
                
                Ok(ToolCallResult {
                    content: vec![rmcp::model::Content::Text {
                        text: serde_json::to_string_pretty(&result)
                            .map_err(|e| rmcp::service::ServiceError::internal_error(format!("序列化结果失败: {}", e)))?,
                    }],
                    is_error: !result.success,
                })
            }
            "git_commit" => {
                let params: serde_json::Value = params.arguments;
                let message = params.get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| rmcp::service::ServiceError::invalid_params("缺少提交消息".to_string()))?;
                let amend = params.get("amend").and_then(|v| v.as_bool()).unwrap_or(false);
                let working_dir = params.get("working_dir").and_then(|v| v.as_str());
                
                let mut args = vec!["-m".to_string(), message.to_string()];
                if amend {
                    args.insert(0, "--amend".to_string());
                }
                
                let result = self.service.execute_git_command("commit", &args, working_dir);
                
                Ok(ToolCallResult {
                    content: vec![rmcp::model::Content::Text {
                        text: serde_json::to_string_pretty(&result)
                            .map_err(|e| rmcp::service::ServiceError::internal_error(format!("序列化结果失败: {}", e)))?,
                    }],
                    is_error: !result.success,
                })
            }
            "git_branch" => {
                let params: serde_json::Value = params.arguments;
                let action = params.get("action")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| rmcp::service::ServiceError::invalid_params("缺少操作类型".to_string()))?;
                let branch_name = params.get("branch_name").and_then(|v| v.as_str());
                let force = params.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
                let working_dir = params.get("working_dir").and_then(|v| v.as_str());
                
                let (command, args) = match action {
                    "list" => ("branch", vec![]),
                    "create" => {
                        let name = branch_name.ok_or_else(|| rmcp::service::ServiceError::invalid_params("创建分支需要分支名称".to_string()))?;
                        ("branch", vec![name.to_string()])
                    }
                    "delete" => {
                        let name = branch_name.ok_or_else(|| rmcp::service::ServiceError::invalid_params("删除分支需要分支名称".to_string()))?;
                        let flag = if force { "-D" } else { "-d" };
                        ("branch", vec![flag.to_string(), name.to_string()])
                    }
                    "checkout" => {
                        let name = branch_name.ok_or_else(|| rmcp::service::ServiceError::invalid_params("切换分支需要分支名称".to_string()))?;
                        ("checkout", vec![name.to_string()])
                    }
                    _ => return Err(rmcp::service::ServiceError::invalid_params(format!("不支持的分支操作: {}", action))),
                };
                
                let result = self.service.execute_git_command(command, &args, working_dir);
                
                Ok(ToolCallResult {
                    content: vec![rmcp::model::Content::Text {
                        text: serde_json::to_string_pretty(&result)
                            .map_err(|e| rmcp::service::ServiceError::internal_error(format!("序列化结果失败: {}", e)))?,
                    }],
                    is_error: !result.success,
                })
            }
            "git_remote" => {
                let params: serde_json::Value = params.arguments;
                let action = params.get("action")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| rmcp::service::ServiceError::invalid_params("缺少操作类型".to_string()))?;
                let name = params.get("name").and_then(|v| v.as_str());
                let url = params.get("url").and_then(|v| v.as_str());
                let working_dir = params.get("working_dir").and_then(|v| v.as_str());
                
                let args = match action {
                    "list" => vec!["-v".to_string()],
                    "add" => {
                        let name = name.ok_or_else(|| rmcp::service::ServiceError::invalid_params("添加远程仓库需要名称".to_string()))?;
                        let url = url.ok_or_else(|| rmcp::service::ServiceError::invalid_params("添加远程仓库需要 URL".to_string()))?;
                        vec!["add".to_string(), name.to_string(), url.to_string()]
                    }
                    "remove" => {
                        let name = name.ok_or_else(|| rmcp::service::ServiceError::invalid_params("删除远程仓库需要名称".to_string()))?;
                        vec!["remove".to_string(), name.to_string()]
                    }
                    "set-url" => {
                        let name = name.ok_or_else(|| rmcp::service::ServiceError::invalid_params("设置远程仓库 URL 需要名称".to_string()))?;
                        let url = url.ok_or_else(|| rmcp::service::ServiceError::invalid_params("设置远程仓库 URL 需要 URL".to_string()))?;
                        vec!["set-url".to_string(), name.to_string(), url.to_string()]
                    }
                    _ => return Err(rmcp::service::ServiceError::invalid_params(format!("不支持的远程仓库操作: {}", action))),
                };
                
                let result = self.service.execute_git_command("remote", &args, working_dir);
                
                Ok(ToolCallResult {
                    content: vec![rmcp::model::Content::Text {
                        text: serde_json::to_string_pretty(&result)
                            .map_err(|e| rmcp::service::ServiceError::internal_error(format!("序列化结果失败: {}", e)))?,
                    }],
                    is_error: !result.success,
                })
            }
            "git_push" => {
                let params: serde_json::Value = params.arguments;
                let remote = params.get("remote").and_then(|v| v.as_str()).unwrap_or("origin");
                let branch = params.get("branch").and_then(|v| v.as_str());
                let force = params.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
                let working_dir = params.get("working_dir").and_then(|v| v.as_str());
                
                let mut args = vec![remote.to_string()];
                if let Some(b) = branch {
                    args.push(b.to_string());
                }
                if force {
                    args.insert(0, "--force".to_string());
                }
                
                let result = self.service.execute_git_command("push", &args, working_dir);
                
                Ok(ToolCallResult {
                    content: vec![rmcp::model::Content::Text {
                        text: serde_json::to_string_pretty(&result)
                            .map_err(|e| rmcp::service::ServiceError::internal_error(format!("序列化结果失败: {}", e)))?,
                    }],
                    is_error: !result.success,
                })
            }
            "git_pull" => {
                let params: serde_json::Value = params.arguments;
                let remote = params.get("remote").and_then(|v| v.as_str()).unwrap_or("origin");
                let branch = params.get("branch").and_then(|v| v.as_str());
                let rebase = params.get("rebase").and_then(|v| v.as_bool()).unwrap_or(false);
                let working_dir = params.get("working_dir").and_then(|v| v.as_str());
                
                let mut args = vec![];
                if rebase {
                    args.push("--rebase".to_string());
                }
                args.push(remote.to_string());
                if let Some(b) = branch {
                    args.push(b.to_string());
                }
                
                let result = self.service.execute_git_command("pull", &args, working_dir);
                
                Ok(ToolCallResult {
                    content: vec![rmcp::model::Content::Text {
                        text: serde_json::to_string_pretty(&result)
                            .map_err(|e| rmcp::service::ServiceError::internal_error(format!("序列化结果失败: {}", e)))?,
                    }],
                    is_error: !result.success,
                })
            }
            _ => Err(rmcp::service::ServiceError::method_not_found(format!("未知工具: {}", name))),
        }
    }

    async fn read_resource(&self, _uri: &str) -> Result<String, rmcp::service::ServiceError> {
        Err(rmcp::service::ServiceError::method_not_found("Git 服务不提供资源读取功能".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_repo() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();
        
        // 初始化 git 仓库
        std::process::Command::new("git")
            .args(&["init"])
            .current_dir(&repo_path)
            .output()
            .unwrap();
            
        // 配置用户信息
        std::process::Command::new("git")
            .args(&["config", "user.name", "Test User"])
            .current_dir(&repo_path)
            .output()
            .unwrap();
            
        std::process::Command::new("git")
            .args(&["config", "user.email", "test@example.com"])
            .current_dir(&repo_path)
            .output()
            .unwrap();
        
        (temp_dir, repo_path)
    }

    #[test]
    fn test_git_service_creation() {
        let service = GitService::new(None);
        assert_eq!(service.name(), "gitai-git-service");
        assert_eq!(service.version(), "1.0.0");
        assert!(!service.is_running);
    }

    #[test]
    fn test_git_service_start_stop() {
        let mut service = GitService::new(None);
        
        // 启动服务
        let result = service.start_sync();
        assert!(result.is_ok());
        assert!(service.is_running);
        
        // 停止服务
        let result = service.stop_sync();
        assert!(result.is_ok());
        assert!(!service.is_running);
    }

    #[test]
    fn test_git_execute_command() {
        let (_temp_dir, repo_path) = setup_test_repo();
        let service = GitService::new(Some(repo_path.clone()));
        
        // 测试 git 版本命令
        let result = service.execute_git_command("--version", &[], None);
        assert!(result.success);
        assert!(result.stdout.contains("git version"));
    }

    #[test]
    fn test_git_status() {
        let (_temp_dir, repo_path) = setup_test_repo();
        let service = GitService::new(Some(repo_path.clone()));
        
        // 创建一个测试文件
        std::fs::write(repo_path.join("test.txt"), "Hello World").unwrap();
        
        let status = service.get_git_status(None).unwrap();
        assert!(status.has_changes);
        assert_eq!(status.untracked_files.len(), 1);
        assert_eq!(status.untracked_files[0], "test.txt");
    }

    #[test]
    fn test_git_tools_list() {
        let service = GitService::new(None);
        let tools = service.get_tools();
        
        assert_eq!(tools.len(), 10);
        assert!(tools.iter().any(|t| t.name == "git_execute"));
        assert!(tools.iter().any(|t| t.name == "git_status"));
        assert!(tools.iter().any(|t| t.name == "git_commit"));
    }

    #[tokio::test]
    async fn test_git_service_handler() {
        let (_temp_dir, repo_path) = setup_test_repo();
        let handler = GitServiceHandler::new(Some(repo_path));
        
        // 测试列出工具
        let tools = handler.list_tools().await.unwrap();
        assert!(!tools.is_empty());
        
        // 测试列出资源
        let resources = handler.list_resources().await.unwrap();
        assert!(resources.is_empty()); // Git 服务不提供资源
    }
}
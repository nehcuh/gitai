// GitAI 增强 Git MCP 服务
// 
// 封装 GitAI 增强的 git 功能为 MCP 工具，提供智能化的 git 操作能力
// 对于已被 GitAI 增强的功能，调用 GitAI 实现；对于原生功能，调用原生 git
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::mcp::{McpService, RmcpResult};
use crate::config::AppConfig;
use crate::handlers::commit::handle_commit;
use crate::handlers::review::handle_review_with_output_in_dir;
use crate::utils::{construct_commit_args, construct_review_args};
use rmcp::model::Tool;
use crate::mcp::rmcp_compat::{
    ServiceError, ToolBuilder, CompatServerHandler, 
    create_param,
};

/// GitAI 增强 Git MCP 服务
pub struct GitService {
    name: String,
    version: String,
    description: String,
    repository_path: Option<PathBuf>,
    is_running: bool,
    config: Arc<AppConfig>,
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
    /// 创建新的 GitAI 增强 Git 服务
    pub fn new(repository_path: Option<PathBuf>, config: Arc<AppConfig>) -> Self {
        Self {
            name: "gitai-git-service".to_string(),
            version: "1.0.0".to_string(),
            description: "GitAI 增强 Git 服务".to_string(),
            repository_path,
            is_running: false,
            config,
        }
    }

    /// 执行 GitAI 增强的 git 命令或原生 git 命令
    async fn execute_gitai_command(&self, command: &str, args: &[String], working_dir: Option<&str>) -> GitOperationResult {
        // 检查是否是被 GitAI 增强的命令
        match command {
            "commit" => self.execute_gitai_commit(args, working_dir).await,
            "review" => self.execute_gitai_review(args, working_dir).await,
            _ => self.execute_git_command(command, args, working_dir),
        }
    }

    /// 执行 GitAI 智能提交
    async fn execute_gitai_commit(&self, args: &[String], working_dir: Option<&str>) -> GitOperationResult {
        // 切换到指定工作目录
        let original_dir = std::env::current_dir().ok();
        if let Some(dir) = working_dir {
            if let Err(e) = std::env::set_current_dir(dir) {
                return GitOperationResult {
                    success: false,
                    stdout: String::new(),
                    stderr: format!("无法切换到目录 {}: {}", dir, e),
                    exit_code: Some(1),
                };
            }
        } else if let Some(repo_path) = &self.repository_path {
            if let Err(e) = std::env::set_current_dir(repo_path) {
                return GitOperationResult {
                    success: false,
                    stdout: String::new(),
                    stderr: format!("无法切换到仓库目录: {}", e),
                    exit_code: Some(1),
                };
            }
        }

        // 构造 GitAI commit 参数
        let mut gitai_args = vec!["commit".to_string()];
        gitai_args.extend(args.iter().cloned());
        let commit_args = construct_commit_args(&gitai_args);

        // 调用 GitAI commit 功能
        let result = handle_commit(&self.config, commit_args).await;
        
        // 恢复原始目录
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(original);
        }

        match result {
            Ok(_) => GitOperationResult {
                success: true,
                stdout: "GitAI 智能提交成功完成".to_string(),
                stderr: String::new(),
                exit_code: Some(0),
            },
            Err(e) => GitOperationResult {
                success: false,
                stdout: String::new(),
                stderr: format!("GitAI 提交失败: {:?}", e),
                exit_code: Some(1),
            },
        }
    }

    /// 执行 GitAI 代码评审
    async fn execute_gitai_review(&self, args: &[String], working_dir: Option<&str>) -> GitOperationResult {
        // 切换到指定工作目录
        let original_dir = std::env::current_dir().ok();
        if let Some(dir) = working_dir {
            if let Err(e) = std::env::set_current_dir(dir) {
                return GitOperationResult {
                    success: false,
                    stdout: String::new(),
                    stderr: format!("无法切换到目录 {}: {}", dir, e),
                    exit_code: Some(1),
                };
            }
        } else if let Some(repo_path) = &self.repository_path {
            if let Err(e) = std::env::set_current_dir(repo_path) {
                return GitOperationResult {
                    success: false,
                    stdout: String::new(),
                    stderr: format!("无法切换到仓库目录: {}", e),
                    exit_code: Some(1),
                };
            }
        }

        // 构造 GitAI review 参数
        let mut gitai_args = vec!["review".to_string()];
        gitai_args.extend(args.iter().cloned());
        let review_args = construct_review_args(&gitai_args);

        // 调用 GitAI review 功能，直接返回输出
        let mut config = (*self.config).clone();
        let result = handle_review_with_output_in_dir(&mut config, review_args, working_dir).await;
        
        // 恢复原始目录
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(original);
        }

        match result {
            Ok(output) => GitOperationResult {
                success: true,
                stdout: output,
                stderr: String::new(),
                exit_code: Some(0),
            },
            Err(e) => GitOperationResult {
                success: false,
                stdout: String::new(),
                stderr: format!("GitAI 代码评审失败: {:?}", e),
                exit_code: Some(1),
            },
        }
    }

    /// 执行原生 git 命令
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
            return Err(ServiceError::internal_error(format!("获取 git 状态失败: {}", status_result.stderr)).into());
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
        use std::collections::HashMap;
        
        vec![
            ToolBuilder::new("git_execute")
                .description("执行 GitAI 增强的 git 命令（对于 commit、review 等会使用 GitAI 功能）")
                .with_simple_schema({
                    let mut props = HashMap::new();
                    props.insert("command".to_string(), create_param("string", "Git 命令（不包含 'git' 前缀），如 commit、review 会使用 GitAI 增强功能"));
                    props.insert("args".to_string(), create_param("array", "命令参数"));
                    props.insert("working_dir".to_string(), create_param("string", "工作目录（可选）"));
                    props
                })
                .build(),
            ToolBuilder::new("gitai_commit")
                .description("使用 GitAI 智能提交功能（AI 生成提交信息、Tree-sitter 分析、代码评审集成）")
                .with_simple_schema({
                    let mut props = HashMap::new();
                    props.insert("message".to_string(), create_param("string", "自定义提交信息（可选，如果不提供将由 AI 生成）"));
                    props.insert("auto_stage".to_string(), create_param("boolean", "是否自动暂存所有修改的文件"));
                    props.insert("tree_sitter".to_string(), create_param("boolean", "是否启用 Tree-sitter 分析"));
                    props.insert("depth".to_string(), create_param("string", "分析深度"));
                    props.insert("issue_id".to_string(), create_param("string", "工单号（可选）"));
                    props.insert("review".to_string(), create_param("boolean", "是否集成代码评审结果"));
                    props.insert("working_dir".to_string(), create_param("string", "工作目录（可选）"));
                    props
                })
                .build(),
            ToolBuilder::new("gitai_review")
                .description("使用 GitAI 代码评审功能（AI 驱动的代码质量分析）")
                .with_simple_schema({
                    let mut props = HashMap::new();
                    props.insert("stories".to_string(), create_param("array", "用户故事列表（可选）"));
                    props.insert("tasks".to_string(), create_param("array", "任务列表（可选）"));
                    props.insert("defects".to_string(), create_param("array", "缺陷列表（可选）"));
                    props.insert("space_id".to_string(), create_param("string", "工作空间ID（可选）"));
                    props.insert("files".to_string(), create_param("array", "要评审的文件列表（可选，默认评审所有更改的文件）"));
                    props.insert("working_dir".to_string(), create_param("string", "工作目录（可选）"));
                    props
                })
                .build(),
            ToolBuilder::new("git_status")
                .description("获取 git 仓库状态信息")
                .with_simple_schema({
                    let mut props = HashMap::new();
                    props.insert("working_dir".to_string(), create_param("string", "工作目录（可选）"));
                    props
                })
                .build(),
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
                Err(ServiceError::internal_error("Git 命令不可用".to_string()).into())
            }
            Err(e) => {
                Err(ServiceError::internal_error(format!("启动 Git 服务失败: {}", e)).into())
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
    pub fn new(repository_path: Option<PathBuf>, config: Arc<AppConfig>) -> Self {
        Self {
            service: GitService::new(repository_path, config),
        }
    }
}

impl CompatServerHandler for GitServiceHandler {
    fn get_server_info(&self) -> rmcp::model::ServerInfo {
        use rmcp::model::{Implementation, ServerCapabilities, ProtocolVersion};
        
        rmcp::model::ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::default(),
            server_info: Implementation {
                name: "gitai-git-service".to_string(),
                version: "1.0.0".to_string(),
            },
            instructions: None,
        }
    }

    fn list_tools(&self) -> Vec<rmcp::model::Tool> {
        self.service.get_tools()
    }

    fn list_resources(&self) -> Vec<rmcp::model::Resource> {
        // Git 服务主要提供工具，不提供资源
        vec![]
    }

    fn call_tool(&self, name: &str, params: serde_json::Value) -> Result<serde_json::Value, ServiceError> {
        // 由于原本的实现是异步的，我们需要在这里处理同步调用
        // 对于这个简化版本，我们先实现基础功能
        match name {
            "git_execute" => {
                let git_params: GitOperationParams = serde_json::from_value(params)
                    .map_err(|e| ServiceError::invalid_params(format!("参数解析失败: {}", e)))?;
                
                let result = self.service.execute_git_command(
                    &git_params.command,
                    &git_params.args,
                    git_params.working_dir.as_deref(),
                );
                
                serde_json::to_value(&result)
                    .map_err(|e| ServiceError::internal_error(format!("序列化结果失败: {}", e)))
            }
            "git_status" => {
                let working_dir = params.get("working_dir").and_then(|v| v.as_str());
                
                let status = self.service.get_git_status(working_dir)
                    .map_err(|e| ServiceError::internal_error(format!("获取 git 状态失败: {:?}", e)))?;
                
                serde_json::to_value(&status)
                    .map_err(|e| ServiceError::internal_error(format!("序列化状态失败: {}", e)))
            }
            _ => Err(ServiceError::method_not_found(format!("未知工具: {}", name))),
        }
    }

    fn read_resource(&self, _uri: &str) -> Result<String, ServiceError> {
        Err(ServiceError::method_not_found("Git 服务不提供资源读取功能".to_string()))
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

    fn create_test_config() -> Arc<AppConfig> {
        use crate::config::{AIConfig, TreeSitterConfig, LanguageConfig, ReviewConfig, ScanConfig};
        use std::collections::HashMap;
        
        let mut prompts = HashMap::new();
        prompts.insert(
            "commit-generator".to_string(),
            "Generate a professional commit message".to_string(),
        );
        
        Arc::new(AppConfig {
            ai: AIConfig {
                api_url: "http://localhost:11434/v1/chat/completions".to_string(),
                model_name: "test-model".to_string(),
                temperature: 0.7,
                api_key: None,
            },
            tree_sitter: TreeSitterConfig {
                enabled: true,
                cache_enabled: true,
                languages: vec!["rust".to_string(), "javascript".to_string()],
            },
            review: ReviewConfig {
                auto_save: true,
                storage_path: "~/.gitai/review_results".to_string(),
                format: "markdown".to_string(),
                max_age_hours: 168,
                include_in_commit: true,
            },
            account: None,
            language: LanguageConfig::default(),
            scan: ScanConfig::default(),
            prompts,
        })
    }

    #[test]
    fn test_git_service_creation() {
        let config = create_test_config();
        let service = GitService::new(None, config);
        assert_eq!(service.name(), "gitai-git-service");
        assert_eq!(service.version(), "1.0.0");
        assert!(!service.is_running);
    }

    #[test]
    fn test_git_service_start_stop() {
        let config = create_test_config();
        let mut service = GitService::new(None, config);
        
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
        let config = create_test_config();
        let service = GitService::new(Some(repo_path.clone()), config);
        
        // 测试 git 版本命令
        let result = service.execute_git_command("--version", &[], None);
        assert!(result.success);
        assert!(result.stdout.contains("git version"));
    }

    #[test]
    fn test_git_status() {
        let (_temp_dir, repo_path) = setup_test_repo();
        let config = create_test_config();
        let service = GitService::new(Some(repo_path.clone()), config);
        
        // 创建一个测试文件
        std::fs::write(repo_path.join("test.txt"), "Hello World").unwrap();
        
        let status = service.get_git_status(None).unwrap();
        assert!(status.has_changes);
        assert_eq!(status.untracked_files.len(), 1);
        assert_eq!(status.untracked_files[0], "test.txt");
    }

    #[test]
    fn test_git_tools_list() {
        let config = create_test_config();
        let service = GitService::new(None, config);
        let tools = service.get_tools();
        
        assert!(tools.len() >= 3); // 至少应该有 git_execute, gitai_commit, gitai_review
        assert!(tools.iter().any(|t| t.name == "git_execute"));
        assert!(tools.iter().any(|t| t.name == "gitai_commit"));
        assert!(tools.iter().any(|t| t.name == "gitai_review"));
    }

    #[tokio::test]
    async fn test_git_service_handler() {
        let (_temp_dir, repo_path) = setup_test_repo();
        let config = create_test_config();
        let handler = GitServiceHandler::new(Some(repo_path), config);
        
        // 测试列出工具
        let tools = handler.list_tools().await.unwrap();
        assert!(!tools.is_empty());
        
        // 测试列出资源
        let resources = handler.list_resources().await.unwrap();
        assert!(resources.is_empty()); // Git 服务不提供资源
    }
}
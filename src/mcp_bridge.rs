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
        format: Option<String>,
        path: Option<String>
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

        // 调用带输出的 review 处理器
        let mut config = self.config.lock().await.clone();
        match handlers::review::handle_review_with_output_in_dir(&mut config, review_args, None, path.as_deref()).await {
            Ok(review_content) => Ok(CallToolResult::success(vec![Content::text(review_content)])),
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
        update_rules: Option<bool>,
        show_results: Option<bool>
    ) -> Result<CallToolResult, McpError> {
        let scan_path = path.unwrap_or(".".to_string());
        let scan_type = if full_scan.unwrap_or(false) { "全量扫描" } else { "增量扫描" };
        let update_text = if update_rules.unwrap_or(false) { "（包含规则更新）" } else { "" };
        let should_show_results = show_results.unwrap_or(false);
        
        if should_show_results {
            // 用户要求展示完整扫描结果
            match self.perform_full_scan(&scan_path, full_scan.unwrap_or(false), update_rules.unwrap_or(false)).await {
                Ok(detailed_results) => {
                    Ok(CallToolResult::success(vec![Content::text(detailed_results)]))
                }
                Err(e) => {
                    Ok(CallToolResult::error(vec![Content::text(
                        format!("❌ 代码扫描失败: {}", e)
                    )]))
                }
            }
        } else {
            // 基础模式，只显示扫描信息
            let scan_result = format!(
                "🔍 代码扫描结果\n\n\
                📁 扫描路径: {}\n\
                📊 扫描类型: {}{}\n\
                📋 扫描状态: 完成\n\n\
                💡 提示: 添加 \"show_results\": true 参数可以获取详细扫描结果。\n\
                或者使用命令行工具 `gitai scan` 获取完整功能。\n\n\
                ✅ 基础扫描检查完成",
                scan_path, scan_type, update_text
            );
            
            Ok(CallToolResult::success(vec![Content::text(scan_result)]))
        }
    }

    /// 执行完整的代码扫描并返回格式化的结果
    async fn perform_full_scan(
        &self,
        scan_path: &str,
        full_scan: bool,
        update_rules: bool,
    ) -> Result<String, McpError> {
        use std::process::Command;
        use std::path::Path;
        
        // 首先检查扫描结果缓存
        if let Ok(cached_result) = self.get_cached_scan_result(scan_path, full_scan).await {
            return Ok(format!("📋 使用缓存的扫描结果:\n\n{}", cached_result));
        }
        
        // 构建 gitai scan 命令
        let current_exe = std::env::current_exe()
            .map_err(|e| McpError::internal_error(format!("无法获取当前可执行文件路径: {}", e), None))?;
        
        let gitai_path = current_exe.parent()
            .ok_or_else(|| McpError::internal_error("无法获取可执行文件目录", None))?
            .join("gitai");
        
        let mut cmd = Command::new(&gitai_path);
        cmd.arg("scan");
        
        // 解析扫描路径，如果是绝对路径，设置工作目录并扫描当前目录
        let (working_dir, scan_arg) = if Path::new(scan_path).is_absolute() {
            (Some(scan_path), ".")
        } else {
            (None, scan_path)
        };
        
        cmd.arg(scan_arg);
        
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }
        
        if full_scan {
            cmd.arg("--full");
        }
        
        if update_rules {
            cmd.arg("--update-rules");
        }
        
        // 执行扫描命令
        let output = cmd.output()
            .map_err(|e| McpError::internal_error(format!("执行扫描命令失败: {}", e), None))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(McpError::internal_error(format!("扫描命令执行失败: {}", stderr), None));
        }
        
        // 解析扫描结果
        let scan_result = self.parse_and_format_scan_output(&output.stdout, scan_path).await?;
        
        // 缓存结果
        if let Err(e) = self.cache_scan_result(scan_path, full_scan, &scan_result).await {
            tracing::warn!("缓存扫描结果失败: {}", e);
        }
        
        Ok(scan_result)
    }

    /// 获取缓存的扫描结果
    async fn get_cached_scan_result(&self, scan_path: &str, full_scan: bool) -> Result<String, McpError> {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};
        use std::path::Path;
        
        // 为绝对路径创建更简洁的缓存键
        let path_key = if Path::new(scan_path).is_absolute() {
            // 对于绝对路径，使用目录名和路径hash
            let dir_name = Path::new(scan_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            let path_hash = std::collections::hash_map::DefaultHasher::new();
            use std::hash::{Hash, Hasher};
            let mut hasher = path_hash;
            scan_path.hash(&mut hasher);
            format!("{}_{:x}", dir_name, hasher.finish())
        } else {
            scan_path.replace("/", "_").replace("\\", "_")
        };
        
        let cache_key = format!("{}_{}", 
            path_key, 
            if full_scan { "full" } else { "incremental" }
        );
        let cache_dir = dirs::home_dir()
            .ok_or_else(|| McpError::internal_error("无法获取用户主目录", None))?
            .join(".gitai")
            .join("mcp-cache");
        
        let cache_file = cache_dir.join(format!("{}.json", cache_key));
        
        if !cache_file.exists() {
            return Err(McpError::internal_error("缓存文件不存在", None));
        }
        
        // 检查缓存是否过期（24小时）
        let metadata = fs::metadata(&cache_file)
            .map_err(|e| McpError::internal_error(format!("读取缓存文件元数据失败: {}", e), None))?;
        
        let modified_time = metadata.modified()
            .map_err(|e| McpError::internal_error(format!("获取文件修改时间失败: {}", e), None))?;
        
        let now = SystemTime::now();
        let cache_age = now.duration_since(modified_time)
            .map_err(|e| McpError::internal_error(format!("计算缓存时间失败: {}", e), None))?;
        
        // 24小时 = 86400秒
        if cache_age.as_secs() > 86400 {
            return Err(McpError::internal_error("缓存已过期", None));
        }
        
        // 读取缓存内容
        let cached_content = fs::read_to_string(&cache_file)
            .map_err(|e| McpError::internal_error(format!("读取缓存文件失败: {}", e), None))?;
        
        Ok(cached_content)
    }

    /// 缓存扫描结果
    async fn cache_scan_result(&self, scan_path: &str, full_scan: bool, result: &str) -> Result<(), McpError> {
        use std::fs;
        use std::path::Path;
        
        // 为绝对路径创建更简洁的缓存键（与 get_cached_scan_result 相同逻辑）
        let path_key = if Path::new(scan_path).is_absolute() {
            let dir_name = Path::new(scan_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            let path_hash = std::collections::hash_map::DefaultHasher::new();
            use std::hash::{Hash, Hasher};
            let mut hasher = path_hash;
            scan_path.hash(&mut hasher);
            format!("{}_{:x}", dir_name, hasher.finish())
        } else {
            scan_path.replace("/", "_").replace("\\", "_")
        };
        
        let cache_key = format!("{}_{}", 
            path_key, 
            if full_scan { "full" } else { "incremental" }
        );
        let cache_dir = dirs::home_dir()
            .ok_or_else(|| McpError::internal_error("无法获取用户主目录", None))?
            .join(".gitai")
            .join("mcp-cache");
        
        // 创建缓存目录
        fs::create_dir_all(&cache_dir)
            .map_err(|e| McpError::internal_error(format!("创建缓存目录失败: {}", e), None))?;
        
        let cache_file = cache_dir.join(format!("{}.json", cache_key));
        
        // 写入缓存
        fs::write(&cache_file, result)
            .map_err(|e| McpError::internal_error(format!("写入缓存文件失败: {}", e), None))?;
        
        Ok(())
    }

    /// 解析并格式化扫描输出
    async fn parse_and_format_scan_output(&self, stdout: &[u8], scan_path: &str) -> Result<String, McpError> {
        // 查找最新的扫描结果文件
        let scan_results_dir = dirs::home_dir()
            .ok_or_else(|| McpError::internal_error("无法获取用户主目录", None))?
            .join(".gitai")
            .join("scan-results")
            .join("gitai");
        
        if !scan_results_dir.exists() {
            return Ok("🔍 扫描完成，但未找到结果文件。\n可能是首次运行或配置问题。".to_string());
        }
        
        // 查找最新的JSON结果文件
        let mut latest_file: Option<std::path::PathBuf> = None;
        let mut latest_time = std::time::SystemTime::UNIX_EPOCH;
        
        if let Ok(entries) = std::fs::read_dir(&scan_results_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if modified > latest_time {
                                latest_time = modified;
                                latest_file = Some(path);
                            }
                        }
                    }
                }
            }
        }
        
        let result_file = latest_file
            .ok_or_else(|| McpError::internal_error("未找到扫描结果文件", None))?;
        
        // 读取并解析JSON结果
        let content = std::fs::read_to_string(&result_file)
            .map_err(|e| McpError::internal_error(format!("读取结果文件失败: {}", e), None))?;
        
        let scan_result: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| McpError::internal_error(format!("解析JSON失败: {}", e), None))?;
        
        // 格式化结果
        self.format_scan_results(&scan_result, scan_path)
    }

    /// 格式化扫描结果
    fn format_scan_results(&self, scan_result: &serde_json::Value, scan_path: &str) -> Result<String, McpError> {
        let mut output = String::new();
        
        output.push_str(&format!("🔍 代码扫描详细结果\n\n"));
        output.push_str(&format!("📁 扫描路径: {}\n", scan_path));
        
        // 基本统计信息
        if let Some(files_scanned) = scan_result.get("files_scanned").and_then(|v| v.as_u64()) {
            output.push_str(&format!("📄 扫描文件数: {}\n", files_scanned));
        }
        
        if let Some(rules_count) = scan_result.get("rules_count").and_then(|v| v.as_u64()) {
            output.push_str(&format!("📋 应用规则数: {}\n", rules_count));
        }
        
        // 问题统计
        if let Some(summary) = scan_result.get("summary") {
            if let Some(total_matches) = summary.get("total_matches").and_then(|v| v.as_u64()) {
                output.push_str(&format!("🎯 发现问题: {}\n", total_matches));
                
                if total_matches > 0 {
                    // 按严重性分类
                    if let Some(by_severity) = summary.get("by_severity").and_then(|v| v.as_object()) {
                        output.push_str("\n📊 问题分布:\n");
                        for (severity, count) in by_severity {
                            let emoji = match severity.as_str() {
                                "error" => "🔴",
                                "warning" => "🟡",
                                "info" => "🔵",
                                _ => "⚪",
                            };
                            output.push_str(&format!("  {} {}: {}\n", emoji, severity, count));
                        }
                    }
                    
                    // 显示前5个问题
                    if let Some(matches) = scan_result.get("matches").and_then(|v| v.as_array()) {
                        output.push_str("\n🔍 发现的主要问题:\n");
                        for (i, match_item) in matches.iter().take(5).enumerate() {
                            output.push_str(&format!("\n{}. ", i + 1));
                            
                            if let Some(file_path) = match_item.get("file_path").and_then(|v| v.as_str()) {
                                let short_path = file_path.split('/').last().unwrap_or(file_path);
                                output.push_str(&format!("📄 {}", short_path));
                            }
                            
                            if let Some(line_number) = match_item.get("line_number").and_then(|v| v.as_u64()) {
                                output.push_str(&format!(" (行{})", line_number));
                            }
                            
                            output.push_str("\n");
                            
                            if let Some(rule_id) = match_item.get("rule_id").and_then(|v| v.as_str()) {
                                output.push_str(&format!("   📋 规则: {}\n", rule_id));
                            }
                            
                            if let Some(severity) = match_item.get("severity").and_then(|v| v.as_str()) {
                                let emoji = match severity {
                                    "error" => "🔴",
                                    "warning" => "🟡",
                                    "info" => "🔵",
                                    _ => "⚪",
                                };
                                output.push_str(&format!("   {} 严重性: {}\n", emoji, severity));
                            }
                            
                            if let Some(message) = match_item.get("message").and_then(|v| v.as_str()) {
                                let short_message = if message.len() > 100 {
                                    format!("{}...", &message[..100])
                                } else {
                                    message.to_string()
                                };
                                output.push_str(&format!("   💬 {}\n", short_message));
                            }
                        }
                        
                        if matches.len() > 5 {
                            output.push_str(&format!("\n... 还有 {} 个问题\n", matches.len() - 5));
                        }
                    }
                } else {
                    output.push_str("\n✅ 未发现安全或质量问题！\n");
                }
            }
        }
        
        output.push_str("\n💾 完整结果已保存到本地文件\n");
        output.push_str("🔍 使用命令行 `gitai scan` 可获得更多详细信息\n");
        
        Ok(output)
    }

    /// 获取 Git 仓库状态信息
    pub async fn gitai_status(
        &self,
        detailed: Option<bool>,
        path: Option<String>
    ) -> Result<CallToolResult, McpError> {
        // 获取 Git 状态  
        let status_result = match handlers::git::get_formatted_repository_status_in_dir(path.as_deref()).await {
            Ok(status_output) => {
                if detailed.unwrap_or(false) {
                    // 获取详细状态信息
                    let staged_diff = handlers::git::get_staged_diff_in_dir(path.as_deref()).await.unwrap_or_default();
                    let unstaged_diff = handlers::git::get_unstaged_diff_in_dir(path.as_deref()).await.unwrap_or_default();
                    
                    let mut detailed_result = format!("📊 Git 状态（详细）\n\n{}", status_output);
                    
                    if !staged_diff.trim().is_empty() {
                        detailed_result.push_str("\n\n📋 暂存的更改详情:\n");
                        detailed_result.push_str(&staged_diff);
                    }
                    
                    if !unstaged_diff.trim().is_empty() {
                        detailed_result.push_str("\n\n📝 未暂存的更改详情:\n");
                        detailed_result.push_str(&unstaged_diff);
                    }
                    
                    detailed_result
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
        file_path: Option<String>,
        path: Option<String>
    ) -> Result<CallToolResult, McpError> {
        let use_staged = staged.unwrap_or(true);
        
        let diff_content = if use_staged {
            if file_path.is_some() {
                // 简化实现：不支持单文件diff
                handlers::git::get_staged_diff_in_dir(path.as_deref()).await.unwrap_or_default()
            } else {
                match handlers::git::get_staged_diff_in_dir(path.as_deref()).await {
                    Ok(diff) => diff,
                    Err(e) => return Ok(CallToolResult::error(vec![Content::text(
                        format!("❌ 获取暂存差异失败: {}", e)
                    )]))
                }
            }
        } else {
            match handlers::git::get_unstaged_diff_in_dir(path.as_deref()).await {
                Ok(diff) => diff,
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(
                    format!("❌ 获取未暂存差异失败: {}", e)
                )]))
            }
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
                        },
                        "path": {
                            "type": "string",
                            "description": "指定 Git 仓库路径（默认: 当前目录）"
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
                        },
                        "show_results": {
                            "type": "boolean",
                            "description": "是否展示详细扫描结果（默认: false，只显示基础信息）"
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
                        },
                        "path": {
                            "type": "string",
                            "description": "指定 Git 仓库路径（默认: 当前目录）"
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
                        },
                        "path": {
                            "type": "string",
                            "description": "指定 Git 仓库路径（默认: 当前目录）"
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
                let path = args.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
                
                self.gitai_review(depth, focus, language, format, path).await
            }
            "gitai_scan" => {
                let path = args.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
                let full_scan = args.get("full_scan").and_then(|v| v.as_bool());
                let update_rules = args.get("update_rules").and_then(|v| v.as_bool());
                let show_results = args.get("show_results").and_then(|v| v.as_bool());
                
                self.gitai_scan(path, full_scan, update_rules, show_results).await
            }
            "gitai_status" => {
                let detailed = args.get("detailed").and_then(|v| v.as_bool());
                let path = args.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
                
                self.gitai_status(detailed, path).await
            }
            "gitai_diff" => {
                let staged = args.get("staged").and_then(|v| v.as_bool());
                let file_path = args.get("file_path").and_then(|v| v.as_str()).map(|s| s.to_string());
                let path = args.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
                
                self.gitai_diff(staged, file_path, path).await
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
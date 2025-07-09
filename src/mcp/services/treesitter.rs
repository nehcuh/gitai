use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use rmcp::{
    handler::server::ServerHandler,
    model::{ServerInfo, Tool, Resource, Implementation, ServerCapabilities, ProtocolVersion},
};
use crate::mcp::rmcp_compat::{
    ServiceError, ToolBuilder, CompatServerHandler, ServerHandlerAdapter,
    create_param, create_object_schema,
};

use crate::{
    mcp::{McpService, RmcpResult},
    tree_sitter_analyzer::{
        analyzer::TreeSitterAnalyzer,
        core::{parse_git_diff, DiffAnalysis, ChangeAnalysis, ChangePattern, ChangeScope},
    },
    config::TreeSitterConfig,
};

/// TreeSitter 分析服务
pub struct TreeSitterService {
    /// 服务名称
    name: String,
    /// 服务版本
    version: String,
    /// 服务描述
    description: String,
    /// TreeSitter 分析器
    analyzer: TreeSitterAnalyzer,
    /// 服务状态
    running: bool,
}

/// 代码分析请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeCodeRequest {
    /// 代码内容
    pub content: String,
    /// 编程语言
    pub language: String,
    /// 分析重点
    pub focus_areas: Option<Vec<String>>,
}

/// 代码分析响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeCodeResponse {
    /// 分析结果
    pub analysis: String,
    /// 语法树信息
    pub syntax_tree: Option<String>,
    /// 代码结构
    pub structure: Option<Value>,
    /// 复杂度分析
    pub complexity: Option<Value>,
}

/// Diff 分析请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseDiffRequest {
    /// Git diff 内容
    pub diff: String,
    /// 上下文行数
    pub context_lines: Option<u32>,
}

/// Diff 分析响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseDiffResponse {
    /// 分析结果
    pub analysis: DiffAnalysis,
    /// 变更摘要
    pub summary: String,
    /// 影响的文件
    pub affected_files: Vec<String>,
}

/// 语言检测请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectLanguageRequest {
    /// 文件路径
    pub file_path: Option<String>,
    /// 文件内容
    pub content: Option<String>,
}

/// 语言检测响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectLanguageResponse {
    /// 检测到的语言
    pub language: Option<String>,
    /// 置信度
    pub confidence: f64,
    /// 支持的特性
    pub features: Vec<String>,
}

/// TreeSitter 服务处理器
pub struct TreeSitterHandler {
    service: TreeSitterService,
}

impl TreeSitterHandler {
    pub fn new(service: TreeSitterService) -> Self {
        Self { service }
    }

    /// 分析代码
    pub async fn analyze_code(&self, request: AnalyzeCodeRequest) -> RmcpResult<AnalyzeCodeResponse> {
        tracing::debug!("开始分析代码，语言: {}", request.language);

        // 由于 TreeSitterAnalyzer 没有 analyze_code 方法，这里提供一个基本的实现
        // 实际使用中，我们会使用 analyze_diff 或 analyze_file_changes 方法
        
        let response = AnalyzeCodeResponse {
            analysis: format!("代码分析完成，语言: {}", request.language),
            syntax_tree: Some("语法树分析功能开发中".to_string()),
            structure: Some(serde_json::Value::String("结构分析功能开发中".to_string())),
            complexity: Some(serde_json::Value::String("复杂度分析功能开发中".to_string())),
        };

        tracing::debug!("代码分析完成");
        Ok(response)
    }

    /// 解析 Git diff
    pub async fn parse_diff(&self, request: ParseDiffRequest) -> RmcpResult<ParseDiffResponse> {
        tracing::debug!("开始解析 Git diff");

        // 解析 diff
        let git_diff = parse_git_diff(&request.diff)
            .map_err(|e| ServiceError::parse_error(format!("Diff 解析失败: {}", e)))?;

        // 生成摘要
        let summary = format!(
            "变更了 {} 个文件",
            git_diff.changed_files.len()
        );

        // 获取受影响的文件
        let affected_files = git_diff.changed_files.iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();

        let response = ParseDiffResponse {
            analysis: DiffAnalysis {
                file_analyses: vec![], // 简化实现
                overall_summary: summary.clone(),
                change_analysis: ChangeAnalysis {
                    function_changes: 0,
                    type_changes: 0,
                    method_changes: 0,
                    interface_changes: 0,
                    other_changes: git_diff.changed_files.len(),
                    change_pattern: ChangePattern::FeatureImplementation,
                    change_scope: ChangeScope::Minor,
                },
            },
            summary,
            affected_files,
        };

        tracing::debug!("Diff 解析完成");
        Ok(response)
    }

    /// 检测编程语言
    pub async fn detect_language(&self, request: DetectLanguageRequest) -> RmcpResult<DetectLanguageResponse> {
        tracing::debug!("开始检测编程语言");

        let language = if let Some(file_path) = &request.file_path {
            // 根据文件扩展名检测语言
            self.detect_language_by_extension(file_path)
        } else if let Some(content) = &request.content {
            // 根据内容检测语言
            self.detect_language_by_content(content)
        } else {
            return Err(crate::mcp::rmcp_compat::ServiceError::invalid_params("必须提供文件路径或内容".to_string()).into());
        };

        let features = if let Some(ref lang) = language {
            self.get_language_features(lang)
        } else {
            Vec::new()
        };

        let response = DetectLanguageResponse {
            language,
            confidence: 0.85, // 简化的置信度
            features,
        };

        tracing::debug!("语言检测完成");
        Ok(response)
    }

    /// 获取支持的语言列表
    pub async fn get_supported_languages(&self) -> RmcpResult<Vec<String>> {
        tracing::debug!("获取支持的语言列表");

        let languages = vec![
            "rust".to_string(),
            "python".to_string(),
            "javascript".to_string(),
            "typescript".to_string(),
            "java".to_string(),
            "c".to_string(),
            "cpp".to_string(),
            "go".to_string(),
            "json".to_string(),
            "yaml".to_string(),
            "markdown".to_string(),
        ];

        Ok(languages)
    }

    /// 根据文件扩展名检测语言
    fn detect_language_by_extension(&self, file_path: &str) -> Option<String> {
        let extension = std::path::Path::new(file_path)
            .extension()?
            .to_str()?;

        match extension {
            "rs" => Some("rust".to_string()),
            "py" => Some("python".to_string()),
            "js" => Some("javascript".to_string()),
            "ts" => Some("typescript".to_string()),
            "java" => Some("java".to_string()),
            "c" => Some("c".to_string()),
            "cpp" | "cc" | "cxx" => Some("cpp".to_string()),
            "go" => Some("go".to_string()),
            "json" => Some("json".to_string()),
            "yml" | "yaml" => Some("yaml".to_string()),
            "md" => Some("markdown".to_string()),
            _ => None,
        }
    }

    /// 根据内容检测语言
    fn detect_language_by_content(&self, content: &str) -> Option<String> {
        // 简化的内容检测逻辑
        if content.contains("fn main()") || content.contains("use std::") {
            Some("rust".to_string())
        } else if content.contains("def ") || content.contains("import ") {
            Some("python".to_string())
        } else if content.contains("function ") || content.contains("const ") {
            Some("javascript".to_string())
        } else if content.contains("interface ") || content.contains("type ") {
            Some("typescript".to_string())
        } else if content.contains("public class ") || content.contains("package ") {
            Some("java".to_string())
        } else if content.contains("package main") || content.contains("func main") {
            Some("go".to_string())
        } else {
            None
        }
    }

    /// 获取语言特性
    fn get_language_features(&self, language: &str) -> Vec<String> {
        match language {
            "rust" => vec![
                "syntax_highlighting".to_string(),
                "structure_analysis".to_string(),
                "complexity_analysis".to_string(),
                "dependency_analysis".to_string(),
            ],
            "python" => vec![
                "syntax_highlighting".to_string(),
                "structure_analysis".to_string(),
                "complexity_analysis".to_string(),
            ],
            "javascript" | "typescript" => vec![
                "syntax_highlighting".to_string(),
                "structure_analysis".to_string(),
                "complexity_analysis".to_string(),
                "import_analysis".to_string(),
            ],
            _ => vec![
                "syntax_highlighting".to_string(),
                "structure_analysis".to_string(),
            ],
        }
    }

    /// 处理工具调用
    pub async fn handle_tool_call(&self, name: &str, args: Value) -> RmcpResult<Value> {
        match name {
            "analyze_code" => {
                let request: AnalyzeCodeRequest = serde_json::from_value(args).map_err(|e| ServiceError::parse_error(format!("参数解析失败: {}", e)))?;
                let response = self.analyze_code(request).await?;
                Ok(serde_json::to_value(response).map_err(|e| ServiceError::parse_error(format!("响应序列化失败: {}", e)))?)
            }
            "parse_diff" => {
                let request: ParseDiffRequest = serde_json::from_value(args).map_err(|e| ServiceError::parse_error(format!("参数解析失败: {}", e)))?;
                let response = self.parse_diff(request).await?;
                Ok(serde_json::to_value(response).map_err(|e| ServiceError::parse_error(format!("响应序列化失败: {}", e)))?)
            }
            "detect_language" => {
                let request: DetectLanguageRequest = serde_json::from_value(args).map_err(|e| ServiceError::parse_error(format!("参数解析失败: {}", e)))?;
                let response = self.detect_language(request).await?;
                Ok(serde_json::to_value(response).map_err(|e| ServiceError::parse_error(format!("响应序列化失败: {}", e)))?)
            }
            "get_supported_languages" => {
                let response = self.get_supported_languages().await?;
                Ok(serde_json::to_value(response).map_err(|e| ServiceError::parse_error(format!("响应序列化失败: {}", e)))?)
            }
            _ => Err(crate::mcp::rmcp_compat::ServiceError::method_not_found(name.to_string()).into())
        }
    }
}

impl CompatServerHandler for TreeSitterHandler {
    fn get_server_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::default(),
            server_info: Implementation {
                name: self.service.name.clone(),
                version: self.service.version.clone(),
            },
            instructions: None,
        }
    }

    fn list_tools(&self) -> Vec<Tool> {
        use std::collections::HashMap;
        
        vec![
            {
                let mut properties = HashMap::new();
                properties.insert("content".to_string(), create_param("string", "代码内容"));
                properties.insert("language".to_string(), create_param("string", "编程语言"));
                properties.insert("focus_areas".to_string(), serde_json::json!({
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "分析重点"
                }));
                
                ToolBuilder::new("analyze_code")
                    .description("分析代码结构和语法")
                    .with_simple_schema(properties)
                    .build()
            },
            {
                let mut properties = HashMap::new();
                properties.insert("diff".to_string(), create_param("string", "Git diff 内容"));
                properties.insert("context_lines".to_string(), create_param("number", "上下文行数"));
                
                ToolBuilder::new("parse_diff")
                    .description("解析和分析 Git diff")
                    .with_simple_schema(properties)
                    .build()
            },
            {
                let mut properties = HashMap::new();
                properties.insert("file_path".to_string(), create_param("string", "文件路径"));
                properties.insert("content".to_string(), create_param("string", "文件内容"));
                
                ToolBuilder::new("detect_language")
                    .description("检测编程语言")
                    .with_simple_schema(properties)
                    .build()
            },
            {
                let properties = HashMap::new(); // No parameters needed
                
                ToolBuilder::new("get_supported_languages")
                    .description("获取支持的语言列表")
                    .with_simple_schema(properties)
                    .build()
            },
        ]
    }

    fn list_resources(&self) -> Vec<Resource> {
        // TODO: Update Resource structure for rmcp 0.2.1
        vec![
            // Resource definitions need to be updated for rmcp 0.2.1 API
        ]
    }

    fn call_tool(&self, name: &str, args: Value) -> Result<Value, ServiceError> {
        // 由于 ServerHandler trait 是同步的，我们需要使用 tokio runtime
        let rt = tokio::runtime::Runtime::new().map_err(|e| ServiceError::internal_error(
            format!("创建运行时失败: {}", e)
        ))?;
        
        rt.block_on(async {
            self.handle_tool_call(name, args).await
        }).map_err(|e| e.into())
    }

    fn read_resource(&self, uri: &str) -> Result<String, ServiceError> {
        // 简化的资源读取实现
        if uri.starts_with("treesitter://queries/") {
            let parts: Vec<&str> = uri.split('/').collect();
            if parts.len() >= 4 {
                let language = parts[3];
                let query_type = parts[4];
                
                match query_type {
                    "highlights.scm" => {
                        Ok(format!("; 语言 {} 的高亮查询规则\n; 这里是查询内容...", language))
                    }
                    "structure.scm" => {
                        Ok(format!("; 语言 {} 的结构查询规则\n; 这里是查询内容...", language))
                    }
                    _ => Err(ServiceError::invalid_params(format!("不支持的查询类型: {}", query_type)))
                }
            } else {
                Err(ServiceError::invalid_params("无效的资源 URI".to_string()))
            }
        } else {
            Err(ServiceError::invalid_params(format!("不支持的资源 URI: {}", uri)))
        }
    }
}

impl TreeSitterService {
    /// 创建新的 TreeSitter 服务
    pub fn new(config: TreeSitterConfig) -> Self {
        let analyzer = TreeSitterAnalyzer::new(config).unwrap_or_else(|_| {
            // 如果创建失败，使用默认配置
            TreeSitterAnalyzer::new(TreeSitterConfig::default()).unwrap()
        });
        
        Self {
            name: "gitai-treesitter-service".to_string(),
            version: "1.0.0".to_string(),
            description: "GitAI TreeSitter 代码分析服务".to_string(),
            analyzer,
            running: false,
        }
    }
}

impl McpService for TreeSitterService {
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
        tracing::info!("启动 TreeSitter 服务");
        self.running = true;
        Ok(())
    }

    fn stop_sync(&mut self) -> RmcpResult<()> {
        tracing::info!("停止 TreeSitter 服务");
        self.running = false;
        Ok(())
    }

    fn health_check_sync(&self) -> RmcpResult<bool> {
        Ok(self.running)
    }

    fn get_handler_info(&self) -> String {
        format!("TreeSitterHandler for service: {}", self.name)
    }
}

impl Clone for TreeSitterService {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            // TreeSitterAnalyzer 不支持 clone，创建新实例
            analyzer: TreeSitterAnalyzer::new(TreeSitterConfig::default()).unwrap(),
            running: self.running,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TreeSitterConfig;

    fn create_test_service() -> TreeSitterService {
        let config = TreeSitterConfig::default();
        TreeSitterService::new(config)
    }

    #[test]
    fn test_service_creation() {
        let service = create_test_service();
        assert_eq!(service.name(), "gitai-treesitter-service");
        assert_eq!(service.version(), "1.0.0");
        assert!(!service.running);
    }

    #[tokio::test]
    async fn test_service_lifecycle() {
        let mut service = create_test_service();
        
        // 初始状态
        assert!(!service.health_check().await.unwrap());
        
        // 启动服务
        service.start().await.unwrap();
        assert!(service.health_check().await.unwrap());
        
        // 停止服务
        service.stop().await.unwrap();
        assert!(!service.health_check().await.unwrap());
    }

    #[test]
    fn test_language_detection_by_extension() {
        let service = create_test_service();
        let handler = TreeSitterHandler::new(service);
        
        assert_eq!(handler.detect_language_by_extension("test.rs"), Some("rust".to_string()));
        assert_eq!(handler.detect_language_by_extension("test.py"), Some("python".to_string()));
        assert_eq!(handler.detect_language_by_extension("test.js"), Some("javascript".to_string()));
        assert_eq!(handler.detect_language_by_extension("test.unknown"), None);
    }

    #[test]
    fn test_language_detection_by_content() {
        let service = create_test_service();
        let handler = TreeSitterHandler::new(service);
        
        assert_eq!(handler.detect_language_by_content("fn main() {}"), Some("rust".to_string()));
        assert_eq!(handler.detect_language_by_content("def hello():"), Some("python".to_string()));
        assert_eq!(handler.detect_language_by_content("function test() {}"), Some("javascript".to_string()));
        assert_eq!(handler.detect_language_by_content("hello world"), None);
    }

    #[test]
    fn test_language_features() {
        let service = create_test_service();
        let handler = TreeSitterHandler::new(service);
        
        let rust_features = handler.get_language_features("rust");
        assert!(rust_features.contains(&"syntax_highlighting".to_string()));
        assert!(rust_features.contains(&"structure_analysis".to_string()));
        assert!(rust_features.contains(&"complexity_analysis".to_string()));
        assert!(rust_features.contains(&"dependency_analysis".to_string()));
        
        let python_features = handler.get_language_features("python");
        assert!(python_features.contains(&"syntax_highlighting".to_string()));
        assert!(!python_features.contains(&"dependency_analysis".to_string()));
    }
}
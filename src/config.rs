use std::path::PathBuf;
use serde::Deserialize;

/// 应用配置
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// AI配置
    pub ai: AiConfig,
    /// 扫描配置
    pub scan: ScanConfig,
    /// DevOps配置
    pub devops: Option<DevOpsConfig>,
    /// 语言配置
    pub language: Option<String>,
    /// MCP配置
    pub mcp: Option<McpConfig>,
}

/// AI配置
#[derive(Debug, Clone, Deserialize)]
pub struct AiConfig {
    pub api_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub temperature: f32,
}

impl AiConfig {
    /// 验证 AI 配置
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        // 验证 API URL
        if self.api_url.trim().is_empty() {
            return Err("AI API URL 不能为空".into());
        }
        
        // 验证 URL 格式
        if !self.api_url.starts_with("http://") && !self.api_url.starts_with("https://") {
            return Err(format!("AI API URL 格式无效: {}", self.api_url).into());
        }
        
        // 验证模型名称
        if self.model.trim().is_empty() {
            return Err("AI 模型名称不能为空".into());
        }
        
        // 验证温度参数
        if self.temperature < 0.0 || self.temperature > 1.0 {
            return Err(format!("AI 温度参数必须在 0.0 到 1.0 之间，当前值: {}", self.temperature).into());
        }
        
        Ok(())
    }
}

/// 扫描配置
#[derive(Debug, Clone, Deserialize)]
pub struct ScanConfig {
    /// 默认扫描路径
    pub default_path: Option<String>,
    /// 超时时间（秒）
    pub timeout: u64,
    /// 并发数
    pub jobs: usize,
    /// 规则目录（可选）。未设置时默认使用 ~/.cache/gitai/rules
    pub rules_dir: Option<String>,
}

impl ScanConfig {
    /// 验证扫描配置
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        // 验证超时时间
        if self.timeout == 0 {
            return Err("扫描超时时间不能为 0".into());
        }
        
        if self.timeout > 3600 {
            return Err("扫描超时时间不能超过 3600 秒（1小时）".into());
        }
        
        // 验证并发数
        if self.jobs == 0 {
            return Err("扫描并发数不能为 0".into());
        }
        
        if self.jobs > 32 {
            return Err("扫描并发数不能超过 32".into());
        }
        
        // 验证默认路径（如果存在）
        if let Some(ref path) = self.default_path {
            if path.trim().is_empty() {
                return Err("扫描默认路径不能为空字符串".into());
            }
        }
        
        // 验证规则目录（如果存在）
        if let Some(ref rules_dir) = self.rules_dir {
            if rules_dir.trim().is_empty() {
                return Err("规则目录不能为空字符串".into());
            }
        }
        
        Ok(())
    }
}

/// DevOps配置
#[derive(Debug, Clone, Deserialize)]
pub struct DevOpsConfig {
    /// 平台类型 (coding, github, gitlab)
    pub platform: String,
    /// API基础URL
    pub base_url: String,
    /// 认证令牌
    pub token: String,
    /// 项目标识
    pub project: Option<String>,
    /// 超时时间（秒）
    pub timeout: u64,
    /// 重试次数
    pub retry_count: u32,
}

/// MCP配置
#[derive(Debug, Clone, Deserialize)]
pub struct McpConfig {
    /// 是否启用MCP服务
    pub enabled: bool,
    /// 服务器配置
    pub server: McpServerConfig,
    /// 服务配置
    pub services: McpServicesConfig,
}

impl McpConfig {
    /// 验证 MCP 配置
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        // 验证服务器配置
        self.server.validate()?;
        
        // 验证服务配置
        self.services.validate()?;
        
        Ok(())
    }
}

/// MCP服务器配置
#[derive(Debug, Clone, Deserialize)]
pub struct McpServerConfig {
    /// 传输协议 (stdio, tcp, sse)
    pub transport: String,
    /// 监听地址 (tcp/sse)
    pub listen_addr: Option<String>,
    /// 服务名称
    pub name: String,
    /// 服务版本
    pub version: String,
}

impl McpServerConfig {
    /// 验证 MCP 服务器配置
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        // 验证传输协议
        match self.transport.as_str() {
            "stdio" | "tcp" | "sse" => {}
            _ => return Err(format!("不支持的传输协议: {}，支持的协议: stdio, tcp, sse", self.transport).into()),
        }
        
        // 验证监听地址（如果需要）
        if (self.transport == "tcp" || self.transport == "sse") && self.listen_addr.is_none() {
            return Err(format!("传输协议为 {} 时必须指定监听地址", self.transport).into());
        }
        
        // 验证服务名称
        if self.name.trim().is_empty() {
            return Err("MCP 服务名称不能为空".into());
        }
        
        // 验证服务版本
        if self.version.trim().is_empty() {
            return Err("MCP 服务版本不能为空".into());
        }
        
        // 验证监听地址格式（如果存在）
        if let Some(ref addr) = self.listen_addr {
            if addr.trim().is_empty() {
                return Err("监听地址不能为空字符串".into());
            }
            
            // 简单的地址格式验证
            if self.transport == "tcp" && !addr.contains(':') {
                return Err("TCP 监听地址必须包含端口号，例如: 127.0.0.1:8080".into());
            }
        }
        
        Ok(())
    }
}

/// MCP服务配置
#[derive(Debug, Clone, Deserialize)]
pub struct McpServicesConfig {
    /// 启用的服务列表
    pub enabled: Vec<String>,
    /// 服务特定配置
    pub review: Option<McpReviewConfig>,
    pub commit: Option<McpCommitConfig>,
    pub scan: Option<McpScanConfig>,
    pub analysis: Option<McpAnalysisConfig>,
}

impl McpServicesConfig {
    /// 验证 MCP 服务配置
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        // 验证启用的服务
        let valid_services = ["review", "commit", "scan", "analysis"];
        for service in &self.enabled {
            if !valid_services.contains(&service.as_str()) {
                return Err(format!("不支持的 MCP 服务: {}，支持的服务: {:?}", service, valid_services).into());
            }
        }
        
        // 验证服务特定配置
        if let Some(ref review) = self.review {
            review.validate()?;
        }
        
        if let Some(ref commit) = self.commit {
            commit.validate()?;
        }
        
        if let Some(ref scan) = self.scan {
            scan.validate()?;
        }
        
        if let Some(ref analysis) = self.analysis {
            analysis.validate()?;
        }
        
        Ok(())
    }
}

/// MCP Review服务配置
#[derive(Debug, Clone, Deserialize)]
pub struct McpReviewConfig {
    /// 默认启用tree-sitter分析
    pub default_tree_sitter: bool,
    /// 默认启用安全扫描
    pub default_security_scan: bool,
    /// 默认输出格式
    pub default_format: String,
}

impl McpReviewConfig {
    /// 验证 MCP Review 服务配置
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        // 验证输出格式
        let valid_formats = ["text", "json", "markdown"];
        if !valid_formats.contains(&self.default_format.as_str()) {
            return Err(format!("不支持的输出格式: {}，支持的格式: {:?}", self.default_format, valid_formats).into());
        }
        
        Ok(())
    }
}

/// MCP Commit服务配置
#[derive(Debug, Clone, Deserialize)]
pub struct McpCommitConfig {
    /// 默认启用代码评审
    pub default_review: bool,
    /// 默认启用tree-sitter分析
    pub default_tree_sitter: bool,
    /// 默认自动添加所有文件
    pub default_add_all: bool,
}

impl McpCommitConfig {
    /// 验证 MCP Commit 服务配置
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        // Commit 配置目前都是布尔值，不需要额外验证
        Ok(())
    }
}

/// MCP Scan服务配置
#[derive(Debug, Clone, Deserialize)]
pub struct McpScanConfig {
    /// 默认扫描工具
    pub default_tool: String,
    /// 默认超时时间
    pub default_timeout: u64,
}

impl McpScanConfig {
    /// 验证 MCP Scan 服务配置
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        // 验证扫描工具
        if self.default_tool.trim().is_empty() {
            return Err("扫描工具名称不能为空".into());
        }
        
        // 验证超时时间
        if self.default_timeout == 0 {
            return Err("扫描超时时间不能为 0".into());
        }
        
        if self.default_timeout > 3600 {
            return Err("扫描超时时间不能超过 3600 秒（1小时）".into());
        }
        
        Ok(())
    }
}

/// MCP Analysis服务配置
#[derive(Debug, Clone, Deserialize)]
pub struct McpAnalysisConfig {
    /// 默认输出详细程度
    pub verbosity: u32,
}

impl McpAnalysisConfig {
    /// 验证 MCP Analysis 服务配置
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        // 验证详细程度
        if self.verbosity > 2 {
            return Err("输出详细程度不能超过 2".into());
        }
        
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ai: AiConfig {
                api_url: "http://localhost:11434/v1/chat/completions".to_string(),
                model: "qwen2.5:32b".to_string(),
                api_key: None,
                temperature: 0.3,
            },
            scan: ScanConfig {
                default_path: None,
                timeout: 300,
                jobs: 0, // 0 表示不强制设置并发，使用 OpenGrep 默认
                rules_dir: None,
            },
            devops: None,
            language: None,
            mcp: Some(McpConfig {
                enabled: false,
                server: McpServerConfig {
                    transport: "stdio".to_string(),
                    listen_addr: Some("127.0.0.1:8080".to_string()),
                    name: "gitai".to_string(),
                    version: "0.1.0".to_string(),
                },
                services: McpServicesConfig {
                    enabled: vec!["review".to_string(), "commit".to_string(), "scan".to_string(), "analysis".to_string()],
                    review: Some(McpReviewConfig {
                        default_tree_sitter: false,
                        default_security_scan: false,
                        default_format: "text".to_string(),
                    }),
                    commit: Some(McpCommitConfig {
                        default_review: false,
                        default_tree_sitter: false,
                        default_add_all: false,
                    }),
                    scan: Some(McpScanConfig {
                        default_tool: "opengrep".to_string(),
                        default_timeout: 300,
                    }),
                    analysis: Some(McpAnalysisConfig {
                        verbosity: 1,
                    }),
                },
            }),
        }
    }
}

impl Config {
    /// 加载配置
    pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let config_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("gitai")
            .join("config.toml");
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            config.validate()?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }
    
    /// 验证配置
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        // 验证 AI 配置
        self.ai.validate()?;
        
        // 验证扫描配置
        self.scan.validate()?;
        
        // 验证 MCP 配置
        if let Some(mcp) = &self.mcp {
            mcp.validate()?;
        }
        
        Ok(())
    }
}
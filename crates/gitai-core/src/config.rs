// GitAI 配置模块
// TODO: 从 src/config.rs 迁移

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// GitAI 主配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// AI 相关配置
    pub ai: AiConfig,
    /// 安全扫描相关配置
    pub scan: ScanConfig,
    /// DevOps 平台集成配置（可选）
    pub devops: Option<DevOpsConfig>,
    /// MCP（模型上下文协议）配置（可选）
    pub mcp: Option<McpConfig>,
}

/// AI 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// AI 服务地址
    pub api_url: String,
    /// 默认模型名称
    pub model: String,
    /// AI 服务 API Key（可选）
    pub api_key: Option<String>,
    /// 默认采样温度（0-1）
    pub temperature: f32,
}

/// 安全扫描配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// 默认扫描路径
    pub default_path: PathBuf,
    /// 扫描超时时间（秒）
    pub timeout: u64,
    /// 并行任务数
    pub jobs: usize,
    /// 规则目录（可选）
    pub rules_dir: Option<String>,
}

/// DevOps 平台集成配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevOpsConfig {
    /// 平台名称（coding/github/gitlab等）
    pub platform: String,
    /// 平台基础地址
    pub base_url: String,
    /// 访问令牌（可选）
    pub token: Option<String>,
    /// 项目标识（可选）
    pub project: Option<String>,
    /// 空间 ID（仅 Coding 平台使用）
    pub space_id: Option<u64>,
    /// 请求超时时间（秒）
    pub timeout: u64,
}

/// MCP（模型上下文协议）配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// 是否启用 MCP
    pub enabled: bool,
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
                default_path: PathBuf::from("."),
                timeout: 300,
                jobs: 4,
                rules_dir: None,
            },
            devops: None,
            mcp: Some(McpConfig { enabled: true }),
        }
    }
}

impl Config {
    /// 从默认路径加载配置
    pub fn load() -> gitai_types::Result<Self> {
        let config_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("gitai")
            .join("config.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .map_err(|e| gitai_types::GitAIError::Other(format!("Failed to read config file: {}", e)))?;
            let config: Config = toml::from_str(&content)
                .map_err(|e| gitai_types::GitAIError::Other(format!("Failed to parse config file: {}", e)))?;
            Ok(config)
        } else {
            // 如果配置文件不存在，使用默认配置
            Ok(Self::default())
        }
    }
}

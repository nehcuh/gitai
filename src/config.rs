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
}

/// AI配置
#[derive(Debug, Clone, Deserialize)]
pub struct AiConfig {
    pub api_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub temperature: f32,
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
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }
}
use std::collections::HashMap;
use std::path::PathBuf;
use serde::Deserialize;

/// 应用配置 - 简化为一个结构体
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// AI配置
    pub ai: AiConfig,
    /// 扫描配置
    pub scan: ScanConfig,
    /// 提示词
    #[allow(dead_code)]
    pub prompts: HashMap<String, String>,
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
    #[allow(dead_code)]
    pub default_tool: String,
    #[allow(dead_code)]
    pub codeql_language: String,
    #[allow(dead_code)]
    pub enable_cache: bool,
    #[allow(dead_code)]
    pub cache_dir: PathBuf,
    /// Semgrep专用配置
    #[allow(dead_code)]
    pub semgrep_timeout: u64,
    #[allow(dead_code)]
    pub semgrep_concurrency: usize,
    #[allow(dead_code)]
    pub semgrep_exclude_patterns: Vec<String>,
    /// CodeQL专用配置
    #[allow(dead_code)]
    pub codeql_database_timeout: u64,
    #[allow(dead_code)]
    pub codeql_query_timeout: u64,
    #[allow(dead_code)]
    pub codeql_security_only: bool,
    #[allow(dead_code)]
    pub codeql_memory_limit: usize,
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
                default_tool: "opengrep".to_string(),
                codeql_language: "auto".to_string(),
                enable_cache: true,
                cache_dir: dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".cache")
                    .join("gitai"),
                semgrep_timeout: 300,
                semgrep_concurrency: 4,
                semgrep_exclude_patterns: vec![
                    "*.test.*".to_string(),
                    "*/tests/*".to_string(),
                    "*/node_modules/*".to_string(),
                    "*/target/*".to_string(),
                ],
                codeql_database_timeout: 30,
                codeql_query_timeout: 15,
                codeql_security_only: true,
                codeql_memory_limit: 2048,
            },
            prompts: HashMap::new(),
        }
    }
}

impl Config {
    /// 加载配置
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
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
// GitAI 配置模块
// TODO: 从 src/config.rs 迁移

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// GitAI 主配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ai: AIConfig,
    pub scan: ScanConfig,
    pub devops: Option<DevOpsConfig>,
    pub mcp: Option<MCPConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub api_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub default_path: PathBuf,
    pub timeout: u64,
    pub jobs: usize,
    pub rules_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevOpsConfig {
    pub platform: String,
    pub base_url: String,
    pub token: Option<String>,
    pub project: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    pub enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ai: AIConfig {
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
            mcp: Some(MCPConfig {
                enabled: true,
            }),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // TODO: 实际加载逻辑
        Ok(Self::default())
    }
}

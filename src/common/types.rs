use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 命令输出结果
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: std::process::ExitStatus,
}

/// 支持的语言枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupportedLanguage {
    Chinese,
    English,
    Auto,
}

impl Default for SupportedLanguage {
    fn default() -> Self {
        Self::Auto
    }
}

impl SupportedLanguage {
    /// Get the language code string
    pub fn code(&self) -> &'static str {
        match self {
            SupportedLanguage::English => "en",
            SupportedLanguage::Chinese => "zh",
            SupportedLanguage::Auto => "auto",
        }
    }

    /// Parse language from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Some(SupportedLanguage::English),
            "zh" | "chinese" | "zh-cn" | "zh_cn" => Some(SupportedLanguage::Chinese),
            "auto" => Some(SupportedLanguage::Auto),
            _ => None,
        }
    }

    /// Get system default language
    pub fn system_default() -> Self {
        // Try to detect system language, fallback to English
        if let Ok(lang) = std::env::var("LANG") {
            if lang.starts_with("zh") {
                return SupportedLanguage::Chinese;
            }
        }
        SupportedLanguage::English
    }
}

impl std::fmt::Display for SupportedLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedLanguage::Chinese => write!(f, "zh"),
            SupportedLanguage::English => write!(f, "en"),
            SupportedLanguage::Auto => write!(f, "auto"),
        }
    }
}

impl std::str::FromStr for SupportedLanguage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "zh" | "chinese" | "中文" => Ok(SupportedLanguage::Chinese),
            "en" | "english" | "英文" => Ok(SupportedLanguage::English),
            "auto" | "自动" => Ok(SupportedLanguage::Auto),
            _ => Err(format!("不支持的语言: {}。支持的值: zh, en, auto", s)),
        }
    }
}

/// AI 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }
}

/// 分析深度级别
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisDepth {
    Shallow,
    Medium,
    Deep,
}

impl Default for AnalysisDepth {
    fn default() -> Self {
        Self::Medium
    }
}

impl std::fmt::Display for AnalysisDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalysisDepth::Shallow => write!(f, "shallow"),
            AnalysisDepth::Medium => write!(f, "medium"),
            AnalysisDepth::Deep => write!(f, "deep"),
        }
    }
}

impl std::str::FromStr for AnalysisDepth {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "shallow" => Ok(AnalysisDepth::Shallow),
            "medium" => Ok(AnalysisDepth::Medium),
            "deep" => Ok(AnalysisDepth::Deep),
            _ => Err(format!("不支持的分析深度: {}。支持的值: shallow, medium, deep", s)),
        }
    }
}

/// 通用的键值对配置
pub type ConfigMap = HashMap<String, String>;
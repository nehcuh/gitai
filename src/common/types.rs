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
    /// Returns the default supported language, which is `Auto`.
    fn default() -> Self {
        Self::Auto
    }
}

impl SupportedLanguage {
    /// Returns the language code corresponding to the supported language.
    ///
    /// The returned code is "en" for English, "zh" for Chinese, and "auto" for automatic detection.
    ///
    /// # Examples
    ///
    /// ```
    /// let lang = SupportedLanguage::English;
    /// assert_eq!(lang.code(), "en");
    /// ```
    pub fn code(&self) -> &'static str {
        match self {
            SupportedLanguage::English => "en",
            SupportedLanguage::Chinese => "zh",
            SupportedLanguage::Auto => "auto",
        }
    }

    /// Parses a string into a `SupportedLanguage` variant.
    ///
    /// Returns `Some(SupportedLanguage)` if the input matches a supported language code or name (case-insensitive), or `None` if unsupported.
    ///
    /// Recognized values include "en", "english", "zh", "chinese", "zh-cn", "zh_cn", and "auto".
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Some(SupportedLanguage::English),
            "zh" | "chinese" | "zh-cn" | "zh_cn" => Some(SupportedLanguage::Chinese),
            "auto" => Some(SupportedLanguage::Auto),
            _ => None,
        }
    }

    /// Returns the system's default language based on the `LANG` environment variable.
    ///
    /// If the `LANG` environment variable starts with "zh", returns `SupportedLanguage::Chinese`.
    /// Otherwise, returns `SupportedLanguage::English`. If the variable is not set, defaults to English.
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
    /// Formats the `SupportedLanguage` as its corresponding language code ("zh", "en", or "auto").
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::SupportedLanguage;
    /// assert_eq!(format!("{}", SupportedLanguage::Chinese), "zh");
    /// assert_eq!(format!("{}", SupportedLanguage::English), "en");
    /// assert_eq!(format!("{}", SupportedLanguage::Auto), "auto");
    /// ```
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

    /// Parses a string into a `SupportedLanguage` variant.
    ///
    /// Accepts language codes or names in English or Chinese (e.g., "zh", "chinese", "中文", "en", "english", "英文", "auto", "自动").
    /// Returns an error message in Chinese if the input is not supported.
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
    /// Creates a new chat message with the "user" role and the specified content.
    ///
    /// # Examples
    ///
    /// ```
    /// let msg = ChatMessage::user("Hello!");
    /// assert_eq!(msg.role, "user");
    /// assert_eq!(msg.content, "Hello!");
    /// ```
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    /// Creates a new chat message with the role set to "assistant".
    ///
    /// # Examples
    ///
    /// ```
    /// let msg = ChatMessage::assistant("How can I help you?");
    /// assert_eq!(msg.role, "assistant");
    /// assert_eq!(msg.content, "How can I help you?");
    /// ```
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }

    /// Creates a chat message with the "system" role and the specified content.
    ///
    /// # Examples
    ///
    /// ```
    /// let msg = ChatMessage::system("System initialization complete.");
    /// assert_eq!(msg.role, "system");
    /// assert_eq!(msg.content, "System initialization complete.");
    /// ```
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
    /// Returns the default analysis depth, which is `Medium`.
    fn default() -> Self {
        Self::Medium
    }
}

impl std::fmt::Display for AnalysisDepth {
    /// Formats the `AnalysisDepth` as a lowercase string ("shallow", "medium", or "deep").
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::common::types::AnalysisDepth;
    /// use std::fmt::Write;
    ///
    /// let depth = AnalysisDepth::Deep;
    /// let mut s = String::new();
    /// write!(&mut s, "{}", depth).unwrap();
    /// assert_eq!(s, "deep");
    /// ```
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

    /// Parses a string into an `AnalysisDepth` variant.
    ///
    /// Returns an error message in Chinese if the input does not match "shallow", "medium", or "deep" (case-insensitive).
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::AnalysisDepth;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(AnalysisDepth::from_str("shallow").unwrap(), AnalysisDepth::Shallow);
    /// assert_eq!(AnalysisDepth::from_str("MEDIUM").unwrap(), AnalysisDepth::Medium);
    /// assert!(AnalysisDepth::from_str("unknown").is_err());
    /// ```
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
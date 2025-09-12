//! AI服务接口定义

use crate::domain_errors::DomainError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI模型类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIModel {
    /// OpenAI GPT-3.5 系列
    GPT35,
    /// OpenAI GPT-4 系列
    GPT4,
    /// OpenAI GPT-4 Turbo 变体
    GPT4Turbo,
    /// Anthropic Claude 3 系列
    Claude3,
    /// 自定义模型标识
    Custom(String),
}

/// AI配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    /// 访问 AI 服务所需的 API Key
    pub api_key: String,
    /// 自定义服务地址（可选）
    pub base_url: Option<String>,
    /// 使用的模型类型
    pub model: AIModel,
    /// 响应最大 token 数（可选）
    pub max_tokens: Option<u32>,
    /// 采样温度（可选，0-1）
    pub temperature: Option<f32>,
    /// 请求超时时间（秒，可选）
    pub timeout: Option<u64>,
}

/// AI请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequest {
    /// 提示词（完整请求内容）
    pub prompt: String,
    /// 额外上下文（键值对，可选）
    pub context: Option<HashMap<String, String>>,
    /// 覆盖默认最大 token（可选）
    pub max_tokens: Option<u32>,
    /// 覆盖默认温度（可选）
    pub temperature: Option<f32>,
}

/// AI响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    /// 模型回复的文本内容
    pub content: String,
    /// 计费与用量信息（可选）
    pub usage: Option<UsageInfo>,
    /// 实际返回的模型名称
    pub model: String,
}

/// 使用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    /// 提示词占用的 token 数
    pub prompt_tokens: u32,
    /// 补全内容占用的 token 数
    pub completion_tokens: u32,
    /// 总 token 数
    pub total_tokens: u32,
}

/// AI服务接口
#[async_trait]
pub trait AIService: Send + Sync {
    /// 发送AI请求
    async fn request(
        &self,
        request: AIRequest,
    ) -> std::result::Result<AIResponse, crate::domain_errors::DomainError>;

    /// 批量发送AI请求
    async fn batch_request(&self, requests: Vec<AIRequest>)
        -> Result<Vec<AIResponse>, DomainError>;

    /// 检查服务健康状态
    async fn health_check(&self) -> std::result::Result<bool, crate::domain_errors::DomainError>;
}

/// AI服务提供者
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// 创建AI服务
    fn create_service(&self, config: AIConfig) -> Result<Box<dyn AIService>, DomainError>;

    /// 支持的模型列表
    fn supported_models(&self) -> Vec<AIModel>;
}

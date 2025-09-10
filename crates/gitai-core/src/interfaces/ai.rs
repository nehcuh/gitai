//! AI服务接口定义

use crate::domain_errors::DomainError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI模型类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIModel {
    GPT35,
    GPT4,
    GPT4Turbo,
    Claude3,
    Custom(String),
}

/// AI配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: AIModel,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub timeout: Option<u64>,
}

/// AI请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequest {
    pub prompt: String,
    pub context: Option<HashMap<String, String>>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

/// AI响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub content: String,
    pub usage: Option<UsageInfo>,
    pub model: String,
}

/// 使用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// AI服务接口
#[async_trait]
pub trait AIService: Send + Sync {
    /// 发送AI请求
    async fn request(&self, request: AIRequest) -> std::result::Result<AIResponse, crate::domain_errors::DomainError>;

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

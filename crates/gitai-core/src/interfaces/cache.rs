//! 缓存服务接口定义

use crate::domain_errors::DomainError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_size: Option<usize>,
    pub ttl: Option<Duration>,
    pub namespace: Option<String>,
}

/// 缓存值包装
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheValue {
    pub data: Vec<u8>,
    pub expires_at: Option<i64>,
}

/// 缓存服务接口
#[async_trait]
pub trait CacheService: Send + Sync {
    /// 获取缓存值
    async fn get(&self, key: &str) -> Result<Option<CacheValue>, DomainError>;

    /// 设置缓存值
    async fn set(&self, key: &str, value: CacheValue) -> std::result::Result<(), crate::domain_errors::DomainError>;

    /// 设置带TTL的缓存值
    async fn set_with_ttl(
        &self,
        key: &str,
        value: Vec<u8>,
        ttl: Duration,
    ) -> std::result::Result<(), crate::domain_errors::DomainError>;

    /// 删除缓存值
    async fn delete(&self, key: &str) -> std::result::Result<bool, crate::domain_errors::DomainError>;

    /// 批量删除（支持通配符）
    async fn delete_pattern(&self, pattern: &str) -> std::result::Result<usize, crate::domain_errors::DomainError>;

    /// 清空缓存
    async fn clear(&self) -> std::result::Result<(), crate::domain_errors::DomainError>;

    /// 检查key是否存在
    async fn exists(&self, key: &str) -> std::result::Result<bool, crate::domain_errors::DomainError>;

    /// 获取缓存统计信息
    async fn stats(&self) -> std::result::Result<CacheStats, crate::domain_errors::DomainError>;
}

/// 缓存统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_keys: usize,
    pub total_size: usize,
    pub hit_rate: f64,
    pub miss_rate: f64,
}

/// 缓存服务提供者
#[async_trait]
pub trait CacheProvider: Send + Sync {
    /// 创建缓存服务
    fn create_service(&self, config: CacheConfig) -> Result<Box<dyn CacheService>, DomainError>;

    /// 支持的缓存类型
    fn cache_types(&self) -> Vec<String>;
}

//! 基础设施层
//! 
//! 提供具体的技术实现，包括：
//! - 依赖注入容器
//! - 外部服务适配器
//! - 数据持久化
//! - 配置管理

// 容器实现
pub mod container;

// 重新导出API
pub use container::{ServiceContainer, ContainerError, ServiceLifetime};
pub use container::{ServiceProvider, ErasedServiceProvider};
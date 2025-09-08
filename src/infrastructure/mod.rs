//! 基础设施层
//!
//! 提供具体的技术实现，包括：
//! - 依赖注入容器
//! - 外部服务适配器
//! - 数据持久化
//! - 配置管理

// 容器实现
pub mod container;

// 重新导出API（切换为 v2 实现，保留生命周期与 Provider 接口自 v1）
pub use container::v2::{ContainerError, ServiceContainer};
pub use container::{ServiceLifetime};
pub use container::{ErasedServiceProvider, ServiceProvider};

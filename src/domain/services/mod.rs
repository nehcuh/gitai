//! 领域服务层
//!
//! 提供业务逻辑和领域服务实现

// 应用服务
pub mod application;

// 配置服务已迁移到crates/gitai-core
// pub mod config;

// 工作流服务
pub mod workflow;

// 重新导出主要服务
pub use application::ApplicationService;
// ConfigurationService从gitai-core导出
pub use gitai_core::services::config::ConfigurationService;
pub use workflow::WorkflowService;

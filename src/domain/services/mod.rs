//! 领域服务层
//!
//! 提供业务逻辑和领域服务实现

// 应用服务
pub mod application;

// 配置服务
pub mod config;

// 工作流服务
pub mod workflow;

// 重新导出主要服务
pub use application::ApplicationService;
pub use config::ConfigurationService;
pub use workflow::WorkflowService;
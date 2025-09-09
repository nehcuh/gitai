//! GitAI 命令处理器模块
//!
//! 该模块包含了所有 GitAI 命令的具体处理逻辑，每个命令都有对应的处理器模块。

pub mod commit;
pub mod config;
pub mod features;
pub mod git;
pub mod graph;
pub mod init;
pub mod prompts;
pub mod review;

#[cfg(feature = "mcp")]
pub mod mcp;

#[cfg(feature = "metrics")]
pub mod metrics;

#[cfg(feature = "security")]
pub mod scan;

#[cfg(feature = "update-notifier")]
pub mod update;

/// 命令处理器的通用结果类型
pub type HandlerResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

/// 命令处理器特征
///
/// 所有命令处理器都应实现此特征以保证接口一致性
#[async_trait::async_trait]
pub trait CommandHandler {
    /// 处理命令
    async fn handle(&self, command: &gitai::args::Command, config: Option<&gitai::config::Config>) -> HandlerResult<()>;
    
    /// 获取处理器名称
    fn name(&self) -> &'static str;
    
    /// 验证命令参数是否有效
    fn validate(&self, command: &gitai::args::Command) -> HandlerResult<()> {
        let _ = command; // 默认实现不做验证
        Ok(())
    }
}

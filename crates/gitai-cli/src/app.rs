//! GitAI CLI Application Structure

use crate::args::{Args, Command};
use gitai_core::config::Config;

type CliResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

/// GitAI CLI 应用程序主入口点
pub struct CliApp {
    args: Args,
    config: Option<Config>,
}

impl CliApp {
    /// 创建新的 CLI 应用程序实例
    pub fn new(args: Args) -> Self {
        Self { args, config: None }
    }

    /// 初始化配置
    ///
    /// Init 命令不需要配置文件，其他命令需要先加载配置
    pub async fn initialize(&mut self) -> CliResult<()> {
        // Init 命令不需要配置文件
        if matches!(&self.args.command, Command::Init { .. }) {
            return Ok(());
        }

        // 加载配置文件
        match Config::load() {
            Ok(config) => {
                log::debug!("配置文件加载成功");
                self.config = Some(config);
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ 配置加载失败: {e}");
                eprintln!("💡 提示: 请检查 ~/.config/gitai/config.toml 文件");
                eprintln!("💡 可以使用 'gitai init' 初始化配置");
                Err(format!("配置加载失败: {e}").into())
            }
        }
    }

    /// 运行 CLI 应用程序
    pub async fn run(&self) -> CliResult<()> {
        match &self.args.command {
            Command::Init { .. } => {
                crate::handlers::init::handle_command(&self.args.command).await
            }
            Command::Review { .. } => {
                let config = self.config.as_ref().unwrap();
                crate::handlers::review::handle_command(config, &self.args.command).await
            }
            #[cfg(feature = "security")]
            Command::Scan { .. } | Command::ScanHistory { .. } => {
                let config = self.config.as_ref().unwrap();
                crate::handlers::scan::handle_command(config, &self.args.command).await
            }
            #[cfg(not(feature = "security"))]
            Command::Scan { .. } | Command::ScanHistory { .. } => {
                eprintln!("❌ 安全扫描功能未启用");
                eprintln!("💡 请使用包含 'security' 功能的构建版本");
                Err("功能未启用".into())
            }
            Command::Commit { .. } => {
                let config = self.config.as_ref().unwrap();
                crate::handlers::commit::handle_command(config, &self.args.command).await
            }
            Command::Prompts { .. } => {
                let config = self.config.as_ref().unwrap();
                crate::handlers::prompts::handle_command(config, &self.args.command).await
            }
            // TODO: 更新功能暂时禁用，待 feature 重新启用
            // #[cfg(feature = "update-notifier")]
            Command::Update { .. } => {
                eprintln!("❌ 更新功能未启用");
                eprintln!("💡 请使用包含 'update-notifier' 功能的构建版本");
                Err("功能未启用".into())
            }
            Command::Git(..) => {
                let config = self.config.as_ref().unwrap();
                crate::handlers::git::handle_command(config, &self.args.command, &self.args).await
            }
            // TODO: MCP 功能暂时禁用，待 feature 重新启用
            // #[cfg(feature = "mcp")]
            Command::Mcp { .. } => {
                eprintln!("❌ MCP 服务器功能未启用");
                eprintln!("💡 请使用包含 'mcp' 功能的构建版本");
                Err("功能未启用".into())
            }
            Command::Config { .. } => {
                let config = self.config.as_ref().unwrap();
                crate::handlers::config::handle_command(config, &self.args.command, self.args.offline)
                    .await
            }
            // TODO: 度量功能暂时禁用，待 feature 重新启用
            // #[cfg(feature = "metrics")]
            Command::Metrics { .. } => {
                eprintln!("❌ 度量功能未启用");
                eprintln!("💡 请使用包含 'metrics' 功能的构建版本");
                Err("功能未启用".into())
            }
            Command::Graph { .. } => {
                crate::handlers::graph::handle_command(&self.args.command).await
            }
            Command::Features { .. } => {
                crate::handlers::features::handle_command(&self.args.command).await
            }
        }
    }
}
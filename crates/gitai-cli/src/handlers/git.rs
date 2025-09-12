//! Git 命令处理器
//!
//! 处理通用 Git 命令

use crate::args::{Args, Command};

type HandlerResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

/// 处理 git 命令
pub async fn handle_command(
    _config: &gitai_core::config::Config,
    command: &Command,
    args: &Args,
) -> HandlerResult<()> {
    match command {
        Command::Git(git_args) => {
            // 默认不启用AI解释；--ai 显式开启；--noai 可显式关闭（当外部别名强制开启时）
            let use_ai = args.ai && !args.noai;

            // 执行Git命令
            let output = gitai_core::git::run_git(git_args)?;
            print!("{output}");

            // TODO: AI 功能暂时禁用，待 feature 重新启用
            // #[cfg(feature = "ai")]
            {
                if use_ai {
                    // TODO: 实现 AI 解释逻辑
                    println!("\n🤖 AI解释功能正在开发中...");
                }
            }

            Ok(())
        }
        _ => Err("Invalid command for git handler".into()),
    }
}

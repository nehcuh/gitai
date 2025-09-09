//! Commit 命令处理器
//!
//! 处理智能提交相关的命令

use super::HandlerResult;
use gitai::args::Command;
use gitai::config::Config;
use gitai::commit;

/// 处理 commit 命令
pub async fn handle_command(config: &Config, command: &Command) -> HandlerResult<()> {
    match command {
        Command::Commit {
            message,
            issue_id,
            space_id,
            all,
            review,
            tree_sitter,
            dry_run,
        } => {
            let commit_config = commit::CommitConfig::from_args(
                message.clone(),
                issue_id.clone(),
                *space_id,
                *all,
                *review,
                *tree_sitter,
                *dry_run,
            );
            
            commit::execute_commit(config, commit_config).await?;
            Ok(())
        }
        _ => Err("Invalid command for commit handler".into())
    }
}

//! Review 命令处理器
//!
//! 处理代码评审相关的命令

use super::HandlerResult;
use gitai::args::Command;
use gitai::config::Config;
use gitai::review;

/// 处理 review 命令
pub async fn handle_command(config: &Config, command: &Command) -> HandlerResult<()> {
    match command {
        Command::Review {
            language,
            format,
            output,
            tree_sitter,
            security_scan,
            scan_tool,
            block_on_critical,
            issue_id,
            space_id,
            full,
            ..
        } => {
            let review_config = review::ReviewConfig::from_args(
                language.clone(),
                format.clone(),
                output.clone(),
                *tree_sitter,
                *security_scan,
                scan_tool.clone(),
                *block_on_critical,
                issue_id.clone(),
                *space_id,
                *full,
            );

            review::execute_review(config, review_config).await?;
            Ok(())
        }
        _ => Err(anyhow::anyhow!("Invalid command for review handler").into()),
    }
}

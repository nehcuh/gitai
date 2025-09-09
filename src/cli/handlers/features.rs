//! Features 命令处理器
//!
//! 处理功能特性显示相关的命令

use super::HandlerResult;
use gitai::args::Command;
use gitai::features;

/// 处理 features 命令
pub async fn handle_command(command: &Command) -> HandlerResult<()> {
    match command {
        Command::Features { format } => {
            features::display_features(format);
            Ok(())
        }
        _ => Err("Invalid command for features handler".into())
    }
}

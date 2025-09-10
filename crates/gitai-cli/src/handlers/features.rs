//! Features 命令处理器
//!
//! 处理功能特性显示相关的命令

use crate::args::Command;

// 简单的功能显示函数
pub fn display_features(format: &str) {
    match format {
        "json" => {
            println!("{}", serde_json::json!({
                "features": {
                    "security": cfg!(feature = "security"),
                    "full-analysis": cfg!(feature = "full-analysis"),
                    "minimal": cfg!(feature = "minimal")
                }
            }));
        }
        "table" => {
            println!("📋 可用功能:");
            println!("┌─────────────────┬─────────┐");
            println!("│ 功能             │ 状态    │");
            println!("├─────────────────┼─────────┤");
            println!("│ 安全扫描         │ {}      │", if cfg!(feature = "security") { "✅" } else { "❌" });
            println!("│ 完整分析         │ {}      │", if cfg!(feature = "full-analysis") { "✅" } else { "❌" });
            println!("│ 最小配置         │ {}      │", if cfg!(feature = "minimal") { "✅" } else { "❌" });
            println!("└─────────────────┴─────────┘");
        }
        _ => {
            println!("🎯 GitAI 功能特性:");
            println!("  🔒 安全扫描: {}", if cfg!(feature = "security") { "已启用" } else { "未启用" });
            println!("  📊 完整分析: {}", if cfg!(feature = "full-analysis") { "已启用" } else { "未启用" });
            println!("  ⚡ 最小配置: {}", if cfg!(feature = "minimal") { "已启用" } else { "未启用" });
        }
    }
}

type HandlerResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

/// 处理 features 命令
pub async fn handle_command(command: &Command) -> HandlerResult<()> {
    match command {
        Command::Features { format } => {
            display_features(format);
            Ok(())
        }
        _ => Err("Invalid command for features handler".into()),
    }
}
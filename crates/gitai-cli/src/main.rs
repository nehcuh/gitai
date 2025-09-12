//! GitAI CLI Entry Point

#![allow(
    clippy::uninlined_format_args,
    clippy::collapsible_else_if,
    clippy::wildcard_in_or_patterns,
    clippy::too_many_arguments,
    clippy::unnecessary_map_or
)]

use gitai_cli::args::Args;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

fn init_logger() {
    use std::io::Write;

    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format(|buf, record| {
            let level_style = match record.level() {
                log::Level::Error => "\x1b[31m", // 红色
                log::Level::Warn => "\x1b[33m",  // 黄色
                log::Level::Info => "\x1b[32m",  // 绿色
                log::Level::Debug => "\x1b[36m", // 青色
                log::Level::Trace => "\x1b[90m", // 灰色
            };

            writeln!(
                buf,
                "{}{} [{}] {}",
                level_style,
                chrono::Local::now().format("%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();

    let args = Args::parse();
    let mut app = gitai_cli::CliApp::new(args);

    // Initialize configuration if needed
    app.initialize().await?;

    // Run the application
    app.run().await
}

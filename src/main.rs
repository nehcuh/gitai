mod core;

use crate::core::config::AppConfig;
use crate::core::errors::AppError;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let _config = match AppConfig::load() { // Prefix with underscore to mark as unused
        Ok(config) => config,
        Err(e) => return Err(AppError::Config(e)),
    };

    // tracing::debug!("AI setting\n: api_url: {}", &_config.ai.api_url);
    // // Obtain cmd args
    // let args: Vec<String> = std::env::args().collect();

    // // Default behavior should be gitie --ai
    // if args.len() <= 1 {
    //     passthrough_to_git(&[])?;
    //     return Ok(());
    // }

    // // Filter tree-sitter related arguments, ensure clean git commands
    // let filtered_args = filter_tree_sitter_args(&args[1..]);

    // // Check if contains `review` argument
    // // Notice: review is a new command beyond git command supported arguments
    // if filtered_args.contains(&"review".to_string())
    //     && filtered_args.iter().all(|a| a != "--help" && a != "-h")
    // {
    //     tracing::info!("检测到review命令");

    //     // Refactor review comand arg to use clap parse
    //     let mut review_args_vec = vec!["gitai".to_string(), "review".to_string()];

    //     // Obtain arguments after review
    //     let review_index = filtered_args
    //         .iter()
    //         .position(|a| a == "review")
    //         .unwrap_or(0);
    //     if review_index + 1 < filtered_args.len() {
    //         review_args_vec.extend_from_slice(&filtered_args[review_index + 1..]);
    //     }

    //     tracing::debug!("重构的review命令: {:?}", review_args_vec);

    //     if let Ok(parsed_args) = GitaiArgs::try_parse_from(&review_args_vec) {
    //         match parsed_args.command {
    //             GitaiSubCommand::Review(review_args) => {
    //                 return handle_review(review_args, &config).await;
    //             }
    //             _ => {}
    //         }
    //     } else {
    //         tracing::warn!("解析review命令失败");
    //         // 创建默认的ReviewArgs
    //         let default_review_args = ReviewArgs {
    //             depth: "normal".to_string(),
    //             focus: None,
    //             lang: None,
    //             format: "text".to_string(),
    //             output: None,
    //             tree_sitter: false,
    //             no_tree_sitter: false,
    //             review_ts: false,
    //             passthrough_args: vec![],
    //             commit1: None,
    //             commit2: None,
    //         };
    //         return handle_review(default_review_args, &config).await;
    //     }
    // }
    Ok(())
}

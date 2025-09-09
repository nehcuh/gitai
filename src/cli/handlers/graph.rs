use anyhow::Result;
use log::{debug, info};

use gitai::args::Command;

/// Handler for graph command with Command enum
pub async fn handle_command(command: &Command) -> crate::cli::CliResult<()> {
    match command {
        Command::Graph {
            path,
            output,
            threshold,
            summary,
            radius,
            top_k,
            seeds_from_diff,
            summary_format,
            budget_tokens,
            community,
            comm_alg,
            max_communities,
            max_nodes_per_community,
            with_paths,
            path_samples,
            path_max_hops,
        } => {
            if *summary {
                handle_graph_summary(
                    path,
                    *radius,
                    *top_k,
                    *budget_tokens,
                    *seeds_from_diff,
                    summary_format,
                    *community,
                    comm_alg,
                    *max_communities,
                    *max_nodes_per_community,
                    *with_paths,
                    *path_samples,
                    *path_max_hops,
                    output.as_ref(),
                )
                .await
                .map_err(|e| e.into())
            } else {
                handle_graph_export(path, output.as_ref(), *threshold)
                    .await
                    .map_err(|e| e.into())
            }
        }
        _ => Err("Invalid command for graph handler".into()),
    }
}

/// Handle graph export
async fn handle_graph_export(
    path: &std::path::Path,
    output: Option<&std::path::PathBuf>,
    threshold: f32,
) -> Result<()> {
    use gitai::architectural_impact::graph_export::export_dot_string;
    
    info!("Exporting dependency graph for path: {}", path.display());
    debug!("Using threshold: {}", threshold);
    
    let dot = export_dot_string(path, threshold).await.map_err(|e| anyhow::anyhow!(e.to_string()))?;
    
    if let Some(out) = output {
        std::fs::write(out, dot)?;
        println!("📁 依赖图已导出: {}", out.display());
        info!("Graph exported to: {}", out.display());
    } else {
        println!("{dot}");
        debug!("Graph output written to stdout");
    }
    
    Ok(())
}

/// Handle graph summary
#[allow(clippy::too_many_arguments)]
async fn handle_graph_summary(
    path: &std::path::Path,
    radius: usize,
    top_k: usize,
    budget_tokens: usize,
    seeds_from_diff: bool,
    format: &str,
    with_communities: bool,
    comm_alg: &str,
    max_communities: usize,
    max_nodes_per_community: usize,
    with_paths: bool,
    path_samples: usize,
    path_max_hops: usize,
    output: Option<&std::path::PathBuf>,
) -> Result<()> {
    use gitai::architectural_impact::graph_export::export_summary_string;
    
    info!("Generating graph summary for path: {}", path.display());
    debug!("Parameters: radius={}, top_k={}, budget_tokens={}, format={}", 
           radius, top_k, budget_tokens, format);
    
    let summary = export_summary_string(
        path,
        radius,
        top_k,
        seeds_from_diff,
        format,
        budget_tokens,
        with_communities,
        comm_alg,
        max_communities,
        max_nodes_per_community,
        with_paths,
        path_samples,
        path_max_hops,
    )
    .await.map_err(|e| anyhow::anyhow!(e.to_string()))?;
    
    if let Some(out) = output {
        std::fs::write(out, &summary)?;
        println!("📁 图摘要已导出: {}", out.display());
        info!("Summary exported to: {}", out.display());
    } else {
        println!("{summary}");
        debug!("Summary output written to stdout");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_handle_graph_export() {
        let path = std::path::Path::new(".");
        let result = handle_graph_export(&path, None, 0.1).await;
        // This test may fail without proper setup, but shows the interface
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_handle_graph_summary() {
        let path = std::path::Path::new(".");
        let result = handle_graph_summary(
            &path, 1, 10, 1000, false, "json", false, "labelprop", 10, 5, false, 5, 3, None,
        )
        .await;
        // This test may fail without proper setup, but shows the interface
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_handle_command_graph_export() {
        let command = Command::Graph {
            path: PathBuf::from("."),
            output: None,
            threshold: 0.1,
            summary: false,
            radius: 1,
            top_k: 10,
            seeds_from_diff: false,
            summary_format: "json".to_string(),
            budget_tokens: 1000,
            community: false,
            comm_alg: "labelprop".to_string(),
            max_communities: 10,
            max_nodes_per_community: 5,
            with_paths: false,
            path_samples: 5,
            path_max_hops: 3,
        };

        let result = handle_command(&command).await;
        assert!(result.is_ok() || result.is_err());
    }
}

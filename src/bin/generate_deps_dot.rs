use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use gitai::architectural_impact::dependency_graph::DotOptions;
use gitai::architectural_impact::graph_export::build_global_dependency_graph;

fn is_code_file(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "rs" | "java" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "cc" | "cxx" => true,
        _ => false,
    }
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if !dir.exists() { return; }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 跳过常见的无关目录
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    if [".git", "target", "node_modules", ".cache", ".idea", ".vscode"].contains(&name) {
                        continue;
                    }
                }
                collect_files(&path, out);
            } else if is_code_file(&path) {
                out.push(path);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();
    let scan_dir = if args.len() > 1 { &args[1] } else { "src" };
    let out_file = if args.len() > 2 { &args[2] } else { "deps.dot" };

    println!("Scanning directory: {}", scan_dir);

    let mut files = Vec::new();
    collect_files(Path::new(scan_dir), &mut files);
    files.sort();

println!("Found {} code files", files.len()); // count still printed above from collect_files

let global_graph = build_global_dependency_graph(Path::new(scan_dir)).await?;

    // 高亮关键节点（基于简单中心性阈值 0.15）
    let critical: Vec<String> = global_graph
        .identify_critical_nodes(0.15)
        .into_iter()
        .map(|(id, _)| id.clone())
        .collect();

let options = DotOptions {
        include_weights: true,
        highlight_nodes: critical,
    };

    // Re-build graph via builder already done; here we compute critical again
    use gitai::architectural_impact::graph_export::build_global_dependency_graph as _;
    // Write file
    build_global_dependency_graph(Path::new(scan_dir)).await?; // no-op to ensure await context
    // We need the graph from earlier scope; recompute critical via options already set.
    // For simplicity, rebuild and write using options
    let graph = build_global_dependency_graph(Path::new(scan_dir)).await?;
    graph.write_dot_file(out_file, Some(&options))?;
    println!("DOT written to {}", out_file);

    Ok(())
}


use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use gitai::architectural_impact::dependency_graph::{DependencyGraph, DotOptions};
use gitai::tree_sitter::{SupportedLanguage, TreeSitterManager};

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

    println!("Found {} code files", files.len());

    let mut manager = TreeSitterManager::new().await?;
    let mut global_graph = DependencyGraph::new();

    // 暂存跨文件调用以便后处理
    struct PendingCall { file_path: String, line: usize, callee: String }
    let mut pending_calls: Vec<PendingCall> = Vec::new();

    for path in files {
        let path_str = path.to_string_lossy().to_string();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let Some(lang) = SupportedLanguage::from_extension(&ext) else {
            continue;
        };
        let Ok(code) = fs::read_to_string(&path) else { continue };

        match manager.analyze_structure(&code, lang) {
            Ok(summary) => {
                // 收集调用信息（用于跨文件解析）
                for call in &summary.calls {
                    pending_calls.push(PendingCall { file_path: path_str.clone(), line: call.line, callee: call.callee.clone() });
                }

                let sub_graph = DependencyGraph::from_structural_summary(&summary, &path_str);
                // 合并节点
                for (id, node) in sub_graph.nodes.into_iter() {
                    global_graph.nodes.entry(id).or_insert(node);
                }
                // 合并边
                global_graph.edges.extend(sub_graph.edges.into_iter());
            }
            Err(e) => {
                eprintln!("Failed to analyze {}: {}", path_str, e);
            }
        }
    }

    // 先重建邻接表以保证基础索引
    global_graph.rebuild_adjacency_lists();

    // 基于唯一函数名解析跨文件调用
    let before_edges = global_graph.edges.len();
    for pc in pending_calls {
        global_graph.add_resolved_call(&pc.file_path, pc.line, &pc.callee);
    }
    let after_edges = global_graph.edges.len();
    println!("Added {} cross-file call edges", after_edges.saturating_sub(before_edges));

    // 变更后重建邻接表
    global_graph.rebuild_adjacency_lists();

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

    global_graph.write_dot_file(out_file, Some(&options))?;
    println!("DOT written to {}", out_file);

    Ok(())
}


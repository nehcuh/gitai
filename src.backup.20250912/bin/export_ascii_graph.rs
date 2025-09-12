use std::collections::HashMap;
use std::env;
use std::path::Path;

use gitai::architectural_impact::dependency_graph::{DependencyGraph, EdgeType, NodeType};
use gitai::architectural_impact::graph_export::build_global_dependency_graph;

fn convert_to_ascii(graph: &DependencyGraph, verbosity: u32) -> String {
    // Stable short IDs for nodes
    let mut node_ids: Vec<&String> = graph.nodes.keys().collect();
    node_ids.sort();
    let mut id_map: HashMap<String, String> = HashMap::new();
    for (i, id) in node_ids.iter().enumerate() {
        id_map.insert((**id).clone(), format!("N{}", i + 1));
    }

    let mut out = String::new();
    out.push_str("# Dependency Graph (ASCII)\n");
    out.push_str(&format!(
        "nodes: {}, edges: {}\n",
        graph.nodes.len(),
        graph.edges.len()
    ));
    let stats = graph.get_statistics();
    out.push_str(&format!(
        "avg_degree: {:.2}, cycles: {}, critical: {}\n\n",
        stats.avg_degree, stats.cycles_count, stats.critical_nodes_count
    ));

    out.push_str("[Nodes]\n");
    let mut nodes_sorted: Vec<(
        &String,
        &gitai::architectural_impact::dependency_graph::Node,
    )> = graph.nodes.iter().collect();
    nodes_sorted.sort_by(|a, b| a.0.cmp(b.0));
    for (id, node) in nodes_sorted {
        let short = id_map.get(id).cloned().unwrap_or_else(|| id.clone());
        let label = match &node.node_type {
            NodeType::Function(f) => format!("fn {}()", f.name),
            NodeType::Class(c) => format!("class {}", c.name),
            NodeType::Module(m) => format!("mod {}", m.name),
            NodeType::File(f) => format!("file {}", f.path),
        };
        if verbosity >= 2 {
            out.push_str(&format!(
                "  {short}: {label}  [loc={}:{}..{}, score={:.2}]\n",
                node.metadata.file_path,
                node.metadata.start_line,
                node.metadata.end_line,
                node.importance_score
            ));
        } else {
            out.push_str(&format!("  {short}: {label}\n"));
        }
    }

    out.push_str("\n[Edges]\n");
    let mut edges_sorted = graph.edges.clone();
    edges_sorted.sort_by(|a, b| {
        let c = a.from.cmp(&b.from);
        if c == std::cmp::Ordering::Equal {
            a.to.cmp(&b.to)
        } else {
            c
        }
    });
    for e in edges_sorted {
        let from_s = id_map
            .get(&e.from)
            .cloned()
            .unwrap_or_else(|| e.from.clone());
        let to_s = id_map.get(&e.to).cloned().unwrap_or_else(|| e.to.clone());
        let etype = match e.edge_type {
            EdgeType::Calls => "CALLS",
            EdgeType::Imports => "IMPORTS",
            EdgeType::Exports => "EXPORTS",
            EdgeType::Inherits => "INHERITS",
            EdgeType::Implements => "IMPLEMENTS",
            EdgeType::Uses => "USES",
            EdgeType::References => "REFS",
            EdgeType::Contains => "CONTAINS",
            EdgeType::DependsOn => "DEPENDS",
        };
        if verbosity >= 2 {
            let mut meta = String::new();
            if let Some(m) = &e.metadata {
                if let Some(ref notes) = m.notes {
                    if !notes.is_empty() {
                        meta.push_str(&format!(" notes={notes}"));
                    }
                }
                if let Some(cc) = m.call_count {
                    meta.push_str(&format!(" calls={cc}"));
                }
                if m.is_strong_dependency {
                    meta.push_str(" strong");
                }
            }
            out.push_str(&format!(
                "  {from_s} -[{etype} w={:.2}]{meta}-> {to_s}\n",
                e.weight
            ));
        } else {
            out.push_str(&format!("  {from_s} -[{etype}]-> {to_s}\n"));
        }
    }

    out
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = env::args().skip(1).collect::<Vec<String>>();
    // Defaults
    let mut path = ".".to_string();
    let mut verbosity: u32 = 1;

    // Very simple args: export_ascii_graph [path] [--verbosity N]
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--verbosity" | "-v" => {
                if i + 1 < args.len() {
                    if let Ok(v) = args[i + 1].parse::<u32>() {
                        verbosity = v;
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            s => {
                if !s.starts_with('-') {
                    path = s.to_string();
                }
                i += 1;
            }
        }
    }

    let graph = build_global_dependency_graph(Path::new(&path)).await?;
    let ascii = convert_to_ascii(&graph, verbosity);
    println!("{ascii}");
    Ok(())
}

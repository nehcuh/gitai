use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::architectural_impact::dependency_graph::{DependencyGraph, DotOptions, NodeType};
use crate::tree_sitter::{SupportedLanguage, TreeSitterManager};

fn is_code_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()).unwrap_or(""),
        "rs" | "java"
            | "py"
            | "js"
            | "ts"
            | "go"
            | "c"
            | "cpp"
            | "cc"
            | "cxx"
            | "hpp"
            | "hxx"
            | "h"
    )
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if !dir.exists() {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 跳过常见的无关目录
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    if [
                        ".git",
                        "target",
                        "node_modules",
                        ".cache",
                        ".idea",
                        ".vscode",
                        "vendor",
                        "build",
                    ]
                    .contains(&name)
                    {
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

/// 从给定目录构建全局依赖图（跨文件调用会在后处理阶段尽力解析）
pub async fn build_global_dependency_graph(
    scan_dir: &Path,
) -> Result<DependencyGraph, Box<dyn std::error::Error + Send + Sync>> {
    let mut files = Vec::new();
    collect_files(scan_dir, &mut files);
    files.sort();

    let mut manager = TreeSitterManager::new().await?;
    let mut global_graph = DependencyGraph::new();

    // 暂存跨文件调用以便后处理
    struct PendingCall {
        file_path: String,
        line: usize,
        callee: String,
    }
    let mut pending_calls: Vec<PendingCall> = Vec::new();

    for path in files {
        let path_str = path.to_string_lossy().to_string();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let Some(lang) = SupportedLanguage::from_extension(&ext) else {
            continue;
        };
        let Ok(code) = fs::read_to_string(&path) else {
            continue;
        };

        match manager.analyze_structure(&code, lang) {
            Ok(summary) => {
                // 收集调用信息（用于跨文件解析）
                for call in &summary.calls {
                    pending_calls.push(PendingCall {
                        file_path: path_str.clone(),
                        line: call.line,
                        callee: call.callee.clone(),
                    });
                }

                let sub_graph = DependencyGraph::from_structural_summary(&summary, &path_str);
                // 合并节点
                for (id, node) in sub_graph.nodes.into_iter() {
                    global_graph.nodes.entry(id).or_insert(node);
                }
                // 合并边
                global_graph.edges.extend(sub_graph.edges.into_iter());
            }
            Err(_e) => {
                // 忽略单文件失败，继续其它文件
            }
        }
    }

    // 先重建邻接表以保证基础索引
    global_graph.rebuild_adjacency_lists();

    // 基于唯一函数名解析跨文件调用
    for pc in pending_calls {
        global_graph.add_resolved_call(&pc.file_path, pc.line, &pc.callee);
    }

    // 变更后重建邻接表
    global_graph.rebuild_adjacency_lists();

    Ok(global_graph)
}

/// 导出 DOT 文本（含高亮关键节点）
pub async fn export_dot_string(
    scan_dir: &Path,
    highlight_threshold: f32,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let graph = build_global_dependency_graph(scan_dir).await?;
    let critical: Vec<String> = graph
        .identify_critical_nodes(highlight_threshold)
        .into_iter()
        .map(|(id, _)| id.clone())
        .collect();
    let options = DotOptions {
        include_weights: true,
        highlight_nodes: critical,
    };
    Ok(graph.to_dot(Some(&options)))
}

/// 调用链节点信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct CallNodeInfo {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
}

/// 调用链路径
#[derive(Debug, Clone, serde::Serialize)]
pub struct CallChain {
    pub nodes: Vec<CallNodeInfo>,
}

/// 查询函数调用链
/// direction: "downstream" (默认，获取被调用方路径) 或 "upstream"（获取调用方路径）
pub async fn query_call_chain(
    scan_dir: &Path,
    start: &str,
    end: Option<&str>,
    direction: &str,
    max_depth: usize,
    max_paths: usize,
) -> Result<Vec<CallChain>, Box<dyn std::error::Error + Send + Sync>> {
    let graph = build_global_dependency_graph(scan_dir).await?;

    // 构建仅包含 Calls 的邻接表
    let mut forward: HashMap<String, Vec<String>> = HashMap::new();
    let mut reverse: HashMap<String, Vec<String>> = HashMap::new();
    for e in &graph.edges {
        if let crate::architectural_impact::dependency_graph::EdgeType::Calls = e.edge_type {
            forward
                .entry(e.from.clone())
                .or_default()
                .push(e.to.clone());
            reverse
                .entry(e.to.clone())
                .or_default()
                .push(e.from.clone());
        }
    }

    // 根据函数名定位起点/终点候选
    let mut start_ids: Vec<String> = Vec::new();
    let end_name = end.map(|s| s.to_string());
    for (id, node) in &graph.nodes {
        if let NodeType::Function(f) = &node.node_type {
            if f.name == start {
                start_ids.push(id.clone());
            }
        }
    }
    if start_ids.is_empty() {
        return Ok(vec![]); // 无起点
    }

    // DFS 搜索路径
    let mut results: Vec<Vec<String>> = Vec::new();
    let target_name = end_name.as_deref();

    for sid in start_ids {
        let mut stack: Vec<(String, Vec<String>)> = vec![(sid.clone(), vec![sid.clone()])];
        while let Some((node_id, path)) = stack.pop() {
            if results.len() >= max_paths {
                break;
            }
            // 匹配终点（如果指定）
            if let Some(target) = target_name {
                if let Some(node) = graph.nodes.get(&node_id) {
                    if let NodeType::Function(f) = &node.node_type {
                        if f.name == target {
                            results.push(path.clone());
                            continue;
                        }
                    }
                }
            }
            // 深度限制
            if path.len() > max_depth {
                continue;
            }
            // 获取下一步邻居
            let neighbors = match direction {
                "upstream" => reverse.get(&node_id),
                _ => forward.get(&node_id), // 默认 downstream
            };
            if let Some(neigh) = neighbors {
                for nxt in neigh {
                    if !path.contains(nxt) {
                        // 避免环
                        let mut new_path = path.clone();
                        new_path.push(nxt.clone());
                        stack.push((nxt.clone(), new_path));
                    }
                }
            }
        }
    }

    // 如果没有指定终点，收集到达边界的路径（叶子/深度上限）已在上面添加
    if target_name.is_none() && results.is_empty() {
        // 兜底：从任一起点进行有限层次遍历，收集边界路径
        let sid_opt = graph
            .nodes
            .iter()
            .find_map(|(id, node)| match &node.node_type {
                NodeType::Function(f) if f.name == start => Some(id.clone()),
                _ => None,
            });
        if let Some(sid) = sid_opt {
            let mut queue: Vec<(String, Vec<String>)> = vec![(sid.clone(), vec![sid.clone()])];
            while let Some((node_id, path)) = queue.pop() {
                if results.len() >= max_paths {
                    break;
                }
                if path.len() > max_depth {
                    results.push(path);
                    continue;
                }
                let neighbors = match direction {
                    "upstream" => reverse.get(&node_id),
                    _ => forward.get(&node_id),
                };
                let mut extended = false;
                if let Some(neigh) = neighbors {
                    for nxt in neigh {
                        if !path.contains(nxt) {
                            let mut new_path = path.clone();
                            new_path.push(nxt.clone());
                            queue.push((nxt.clone(), new_path));
                            extended = true;
                        }
                    }
                }
                if !extended {
                    results.push(path);
                }
            }
        }
    }

    // 去重和裁剪
    results.sort();
    results.dedup();
    if results.len() > max_paths {
        results.truncate(max_paths);
    }

    // 转换为富节点信息
    let chains: Vec<CallChain> = results
        .into_iter()
        .map(|path_ids| {
            let mut nodes_info = Vec::new();
            for nid in path_ids {
                if let Some(node) = graph.nodes.get(&nid) {
                    if let NodeType::Function(f) = &node.node_type {
                        nodes_info.push(CallNodeInfo {
                            id: nid.clone(),
                            name: f.name.clone(),
                            file_path: node.metadata.file_path.clone(),
                            line_start: node.metadata.start_line,
                            line_end: node.metadata.end_line,
                        });
                    }
                }
            }
            CallChain { nodes: nodes_info }
        })
        .collect();

    Ok(chains)
}

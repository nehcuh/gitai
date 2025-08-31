use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::architectural_impact::dependency_graph::{DependencyGraph, DotOptions, NodeType};
use crate::tree_sitter::{SupportedLanguage, TreeSitterManager};
use crate::git;

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

/// 导出 LLM 友好的图摘要（v1：在 v0 基础上可选社区压缩，文本/JSON）
pub async fn export_summary_string(
    scan_dir: &Path,
    radius: usize,
    top_k: usize,
    seeds_from_diff: bool,
    format: &str,
    _budget_tokens: usize,
    with_communities: bool,
    comm_alg: &str,
    max_communities: usize,
    max_nodes_per_community: usize,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut graph = build_global_dependency_graph(scan_dir).await?;

    // 计算 PageRank 以填充 importance_score
    let _pr = graph.calculate_pagerank(0.85, 20, 1e-6);

    // 种子：从 git diff 推导（按文件）
    let mut seed_ids: Vec<String> = Vec::new();
    if seeds_from_diff {
        let mut changed_files = std::collections::HashSet::new();
        // 已暂存
        if let Ok(staged) = git::run_git(&[
            "diff".to_string(),
            "--cached".to_string(),
            "--name-only".to_string(),
        ]) {
            for line in staged.lines() {
                if !line.trim().is_empty() {
                    changed_files.insert(line.trim().to_string());
                }
            }
        }
        // 未暂存
        if let Ok(unstaged) = git::run_git(&["diff".to_string(), "--name-only".to_string()]) {
            for line in unstaged.lines() {
                if !line.trim().is_empty() {
                    changed_files.insert(line.trim().to_string());
                }
            }
        }
        // 从节点元数据匹配文件
        for (id, node) in &graph.nodes {
            // 仅匹配文件/函数/类的 file_path
            let fp = &node.metadata.file_path;
            if changed_files.iter().any(|p| fp.ends_with(p)) {
                seed_ids.push(id.clone());
            }
        }
        // 去重与限量
        seed_ids.sort();
        seed_ids.dedup();
        if seed_ids.len() > 200 {
            seed_ids.truncate(200);
        }
    }

    // 如果没有种子，则选择 Top-K 全局节点作为参考（退化处理）
    if seed_ids.is_empty() {
        let mut ids: Vec<(String, f32)> = graph
            .nodes
            .iter()
            .map(|(id, n)| (id.clone(), n.importance_score))
            .collect();
        ids.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        for (id, _) in ids.into_iter().take(20) {
            seed_ids.push(id);
        }
    }

    // 从种子出发做有限半径的邻域采样（双向）
    let mut kept: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut queue: std::collections::VecDeque<(String, usize)> = std::collections::VecDeque::new();
    for sid in &seed_ids {
        kept.insert(sid.clone());
        queue.push_back((sid.clone(), 0));
    }
    while let Some((nid, d)) = queue.pop_front() {
        if d >= radius { continue; }
        // 正向依赖
        for dep in graph.get_dependencies(&nid) {
            if kept.insert((*dep).clone()) {
                queue.push_back(((*dep).clone(), d + 1));
            }
        }
        // 反向依赖
        for dep in graph.get_dependents(&nid) {
            if kept.insert((*dep).clone()) {
                queue.push_back(((*dep).clone(), d + 1));
            }
        }
    }

    // Top-K 重要节点（限制在 kept 子图内）
    let mut top: Vec<(String, f32)> = kept
        .iter()
        .filter_map(|id| graph.nodes.get(id).map(|n| (id.clone(), n.importance_score)))
        .collect();
    top.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    if top.len() > top_k { top.truncate(top_k); }

    // 统计
    let stats = graph.get_statistics();

    // Seeds 预览（最多 20）
    let mut seeds_preview: Vec<String> = Vec::new();
    for sid in seed_ids.iter().take(20) {
        if let Some(n) = graph.nodes.get(sid) {
            let label = match &n.node_type {
                NodeType::Function(f) => format!("fn {}()", f.name),
                NodeType::Class(c) => format!("class {}", c.name),
                NodeType::Module(m) => format!("mod {}", m.name),
                NodeType::File(f) => format!("file {}", f.path),
            };
            seeds_preview.push(label);
        }
    }

    // 可选：社区检测与压缩
    let mut communities_out: Vec<(String, usize, Vec<String>)> = Vec::new();
    let mut comm_edges_out: Vec<(String, String, usize, f32)> = Vec::new();
    if with_communities {
        // 目前仅支持 labelprop；其他值退化到 labelprop
        let _alg = comm_alg.to_lowercase();
        let labels = label_propagation_communities(&graph, 10);

        // 按社区聚合节点
        let mut buckets: HashMap<String, Vec<String>> = HashMap::new();
        for (nid, lab) in &labels {
            buckets.entry(lab.clone()).or_default().push(nid.clone());
        }

        // 计算社区统计与样本
        let mut bucket_vec: Vec<(String, Vec<String>)> = buckets.into_iter().collect();
        bucket_vec.sort_by_key(|(_, nodes)| std::cmp::Reverse(nodes.len()));

        for (comm_id, nodes) in bucket_vec.into_iter().take(max_communities) {
            // 选择前 N 个代表节点（按 importance_score）
            let mut samples: Vec<(String, f32, String)> = nodes
                .iter()
                .filter_map(|id| graph.nodes.get(id).map(|n| {
                    let label = match &n.node_type {
                        NodeType::Function(f) => format!("fn {}()", f.name),
                        NodeType::Class(c) => format!("class {}", c.name),
                        NodeType::Module(m) => format!("mod {}", m.name),
                        NodeType::File(f) => format!("file {}", f.path),
                    };
                    (id.clone(), n.importance_score, label)
                }))
                .collect();
            samples.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            let sample_labels: Vec<String> = samples
                .into_iter()
                .take(max_nodes_per_community)
                .map(|(_, _, lbl)| lbl)
                .collect();
            communities_out.push((comm_id, nodes.len(), sample_labels));
        }

        // 聚合跨社区边（有向）
        let mut edge_buckets: HashMap<(String, String), (usize, f32)> = HashMap::new();
        for e in &graph.edges {
            if let (Some(ls), Some(lt)) = (labels.get(&e.from), labels.get(&e.to)) {
                if ls != lt {
                    let key = (ls.clone(), lt.clone());
                    let entry = edge_buckets.entry(key).or_insert((0, 0.0));
                    entry.0 += 1;
                    entry.1 += e.weight;
                }
            }
        }
        let mut edge_vec: Vec<((String, String), (usize, f32))> = edge_buckets.into_iter().collect();
        edge_vec.sort_by(|a, b| b.1.0.cmp(&a.1.0));
        for ((src, dst), (cnt, wsum)) in edge_vec.into_iter().take(max_communities * 2) {
            comm_edges_out.push((src, dst, cnt, wsum));
        }
    }

    if format == "json" {
        #[derive(serde::Serialize)]
        struct Summary<'a> {
            graph_stats: &'a crate::architectural_impact::dependency_graph::GraphStatistics,
            seeds_preview: Vec<String>,
            top_nodes: Vec<(String, f32)>,
            kept_nodes: usize,
            radius: usize,
            truncated: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            communities: Option<Vec<CommunitySummaryOut>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            community_edges: Option<Vec<CommunityEdgeOut>>,
        }
        #[derive(serde::Serialize)]
        struct CommunitySummaryOut {
            id: String,
            size: usize,
            samples: Vec<String>,
        }
        #[derive(serde::Serialize)]
        struct CommunityEdgeOut {
            src: String,
            dst: String,
            edges: usize,
            weight_sum: f32,
        }
        let top_out = top
            .iter()
            .map(|(id, score)| (id.clone(), *score))
            .collect::<Vec<_>>();
        let comm_json = if with_communities {
            Some(
                communities_out
                    .iter()
                    .map(|(id, size, samples)| CommunitySummaryOut {
                        id: id.clone(),
                        size: *size,
                        samples: samples.clone(),
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        };
        let edges_json = if with_communities {
            Some(
                comm_edges_out
                    .iter()
                    .map(|(s, d, c, w)| CommunityEdgeOut {
                        src: s.clone(),
                        dst: d.clone(),
                        edges: *c,
                        weight_sum: *w,
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        };
        let s = Summary {
            graph_stats: &stats,
            seeds_preview,
            top_nodes: top_out,
            kept_nodes: kept.len(),
            radius,
            truncated: false,
            communities: comm_json,
            community_edges: edges_json,
        };
        return Ok(serde_json::to_string_pretty(&s)?);
    }

    // 文本格式
    let mut out = String::new();
    out.push_str("📊 Graph Summary (v1)\n");
    out.push_str(&format!(
        "nodes: {}, edges: {}, avg_degree: {:.2}, components: {}\n",
        stats.node_count, stats.edge_count, stats.avg_degree, stats.cycles_count
    ));
    out.push_str(&format!("seeds_preview (<=20): {}\n", seeds_preview.join(", ")));
    out.push_str(&format!("kept_nodes (radius={}): {}\n", radius, kept.len()));
    out.push_str("top_nodes (by PageRank):\n");
    for (i, (id, score)) in top.iter().take(10).enumerate() {
        let label = graph.nodes.get(id).map(|n| match &n.node_type {
            NodeType::Function(f) => format!("fn {}()", f.name),
            NodeType::Class(c) => format!("class {}", c.name),
            NodeType::Module(m) => format!("mod {}", m.name),
            NodeType::File(f) => format!("file {}", f.path),
        }).unwrap_or_else(|| id.clone());
        out.push_str(&format!("  {}. {} (pr={:.5})\n", i + 1, label, score));
    }

    if with_communities {
        out.push_str("\n🧩 communities (labelprop):\n");
        for (i, (cid, size, samples)) in communities_out.iter().enumerate() {
            out.push_str(&format!(
                "  C{:02} [{}] size={} samples: {}\n",
                i + 1,
                cid,
                size,
                samples.iter().take(max_nodes_per_community).cloned().collect::<Vec<_>>().join(", ")
            ));
        }
        if !comm_edges_out.is_empty() {
            out.push_str("  cross-community edges (top):\n");
            for (src, dst, cnt, wsum) in comm_edges_out.iter().take(20) {
                out.push_str(&format!("    {} -> {}: edges={} w_sum={:.2}\n", src, dst, cnt, wsum));
            }
        }
    }

    Ok(out)
}

/// Label Propagation 社区检测（无权重简化版）
fn label_propagation_communities(
    graph: &DependencyGraph,
    max_iters: usize,
) -> HashMap<String, String> {
    use rand::{rngs::StdRng, SeedableRng};
    use rand::seq::SliceRandom;

    // 初始标签：每个节点的标签为自身ID
    let mut labels: HashMap<String, String> = graph
        .nodes
        .keys()
        .map(|id| (id.clone(), id.clone()))
        .collect();

    // 预构建无向邻居列表（依赖+被依赖）
    let mut neighbors: HashMap<String, Vec<String>> = HashMap::new();
    for id in graph.nodes.keys() {
        let mut neigh = Vec::new();
        for d in graph.get_dependencies(id) { neigh.push((*d).clone()); }
        for d in graph.get_dependents(id) { neigh.push((*d).clone()); }
        neigh.sort();
        neigh.dedup();
        neighbors.insert(id.clone(), neigh);
    }

    let mut node_order: Vec<String> = graph.nodes.keys().cloned().collect();
    let mut rng = StdRng::seed_from_u64(42);

    for _ in 0..max_iters {
        let mut changed = 0u32;
        node_order.shuffle(&mut rng);
        for id in node_order.iter() {
            let neigh = neighbors.get(id).cloned().unwrap_or_default();
            if neigh.is_empty() { continue; }
            let mut freq: HashMap<String, usize> = HashMap::new();
            for n in neigh.iter() {
                if let Some(l) = labels.get(n) {
                    *freq.entry(l.clone()).or_insert(0) += 1;
                }
            }
            if let Some((&_max_count, candidates)) = freq.values().max().map(|m| (m, freq.iter().filter(|(_, &v)| v == *m).collect::<Vec<_>>())) {
                // 选择字典序最小的标签以保证确定性
                let mut best = candidates
                    .into_iter()
                    .map(|(k, _)| (*k).clone())
                    .collect::<Vec<_>>();
                best.sort();
                let new_label = best.into_iter().next().unwrap();
                if labels.get(id).map(|l| l != &new_label).unwrap_or(true) {
                    labels.insert(id.clone(), new_label);
                    changed += 1;
                }
            }
        }
        if changed == 0 { break; }
    }

    labels
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

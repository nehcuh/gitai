# MCP Graph Summarization API

本规范定义 MCP 服务提供的图摘要接口（summarize）及下钻子命令，便于客户端在有限 token 预算内逐步获取依赖图关键信息。

## summarize

请求参数（建议默认值）：
- radius: usize (default: 1)
- top_k: usize (default: 200)
- with_communities: bool (default: true)
- with_paths: bool (default: true)
- path_samples: usize (default: 5)
- path_max_hops: usize (default: 5)
- budget_tokens: usize (default: 3000)
- include_filters: [string]（glob）
- exclude_filters: [string]（glob; 默认排除 tests/**, vendor/**, generated/**, examples/**）
- scope: enum { SeedOnly, Module, Community, Full } （default: Community）
- language_filters: [string]（可选）
- format: enum { summary, json, dot }（default: summary）

响应（与当前实现对齐的字段说明）：
- graph_stats: { node_count, edge_count, avg_degree, cycles_count, critical_nodes_count }
- seeds_preview: [string...]（节点label，限量）
- top_nodes: [[node_id, pr_score] ...]（Top-K 子集）
- kept_nodes: number（摘要诱导子图保留的节点数）
- radius: number（摘要半径）
- truncated: bool（若发生预算裁剪/降级则为 true）
- communities?: [{ id: string, size: number, samples: [string...] } ...]（v1）
- community_edges?: [{ src: string, dst: string, edges: number, weight_sum: number } ...]（v1）
- path_examples?: [[string...] ...]（v2；每条为节点label序列，Calls-only）

说明：
- 若 format=json，返回上述结构的 JSON；若 format=dot，返回裁剪后的诱导子图 DOT 文本（严格限量）。
- 节点 label 取决于类型：fn name() / class Name / mod name / file path。

示例（JSON）：
```json
{
  "graph_stats": {"node_count": 1107, "edge_count": 2813, "avg_degree": 5.08, "cycles_count": 26, "critical_nodes_count": 0},
  "seeds_preview": ["fn dfs_paths()", "class ProjectInsights"],
  "top_nodes": [["func:src/..::dfs_paths", 0.0103], ["func:src/..::validate", 0.0089]],
  "kept_nodes": 116,
  "radius": 1,
  "truncated": false,
  "communities": [
    {"id": "file:./src/architectural_impact/dependency_graph.rs", "size": 64, "samples": ["fn dfs_cycle_detection()", "fn add_edge()"]}
  ],
  "community_edges": [
    {"src": "file:./src/a.rs", "dst": "file:./src/b.rs", "edges": 12, "weight_sum": 9.5}
  ],
  "path_examples": [
    ["fn start()", "fn parse()", "fn validate()"],
    ["fn run()", "fn handle()"]
  ]
}
```

## 子命令（按需下钻）
- get_node_details(node_id) → 基本信息、度、所在社区、样例引用
- get_paths(src, dst, limit, max_hops) → 若干最短路径（限量）
- get_community(community_id) → 社区摘要与 Top 节点
- expand(node_id, radius=1) → 小范围扩展子图（诱导）
- list_top_nodes(metric=pr|degree|betweenness, k=20)

## 预算自适应
服务端在响应前估算字符串/token 大小，若超预算则按序降级：
1) r: 2 → 1（重新计算 kept/top）
2) top_k: 300 → 200 → 150（截断）
3) communities: Top-10 → Top-5（截断）
4) path_samples（截断；当前实现不缩减 path_max_hops）
5) seeds 输出截断（保留计数）

说明：当发生任何一次降级时，响应中的 `truncated` 字段为 true。

## MCP 工具：summarize_graph（已实装）
- 所属服务：analysis
- 工具名：summarize_graph
- 参数映射：
  - path → scan_dir
  - radius → radius（默认 1）
  - top_k → top_k（默认 200）
  - seeds_from_diff → seeds_from_diff（默认 false）
  - format → "json" | "text"（默认 json）
  - budget_tokens → budget_tokens（默认 3000）
  - community → with_communities（默认 false）
  - comm_alg → comm_alg（当前仅支持 "labelprop"）
  - max_communities → max_communities（默认 50）
  - max_nodes_per_community → max_nodes_per_community（默认 10）
  - with_paths → with_paths（默认 false）
  - path_samples → path_samples（默认 5）
  - path_max_hops → path_max_hops（默认 5）

示例（MCP tools/call 请求）：
```json
{
  "name": "summarize_graph",
  "arguments": {
    "path": ".",
    "radius": 1,
    "top_k": 200,
    "seeds_from_diff": false,
    "format": "json",
    "budget_tokens": 2000,
    "community": true,
    "comm_alg": "labelprop",
    "max_communities": 20,
    "max_nodes_per_community": 8,
    "with_paths": true,
    "path_samples": 5,
    "path_max_hops": 5
  }
}
```

示例响应（节选）：
```json
{
  "graph_stats": {"node_count": 1107, "edge_count": 2813, "avg_degree": 5.08, "cycles_count": 26, "critical_nodes_count": 0},
  "seeds_preview": ["fn dfs_paths()", "class ProjectInsights"],
  "top_nodes": [["func:src/..::dfs_paths", 0.0103], ["func:src/..::validate", 0.0089]],
  "kept_nodes": 116,
  "radius": 1,
  "truncated": true,
  "communities": [
    {"id": "file:./src/architectural_impact/dependency_graph.rs", "size": 64, "samples": ["fn dfs_cycle_detection()", "fn add_edge()"]}
  ],
  "community_edges": [
    {"src": "file:./src/a.rs", "dst": "file:./src/b.rs", "edges": 12, "weight_sum": 9.5}
  ],
  "path_examples": [
    ["fn start()", "fn parse()", "fn validate()"],
    ["fn run()", "fn handle()"]
  ]
}
```

## 错误与诊断
- invalid_filter / budget_too_small / radius_too_large / no_seeds
- stats 字段包含实际裁剪信息，便于客户端提示“可下钻获取更多”


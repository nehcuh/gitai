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

响应（summary 格式，示例字段）：
- graph_stats: { nodes, edges, avg_degree, components }
- seeds_preview: [{ id, label, module }...]（限量）
- top_nodes: [{ id, label, module, pr_score }...]（Top-10）
- communities: [{ id, name, size, cross_edges, samples: [node_label...] }...]（Top-10）
- cross_edges_summary: [{ src_comm, dst_comm, types: { import: n, call: m, write: k } }...]
- impacted_summary: [{ module/community, size, ratio } ...]
- path_examples: [[node_label; <= L] ...]（N 条示例）
- truncated: bool（是否因预算裁剪）

说明：若 format=json，返回上述结构的 JSON；若 format=dot，返回裁剪后的诱导子图 DOT 文本（严格限量）。

## 子命令（按需下钻）
- get_node_details(node_id) → 基本信息、度、所在社区、样例引用
- get_paths(src, dst, limit, max_hops) → 若干最短路径（限量）
- get_community(community_id) → 社区摘要与 Top 节点
- expand(node_id, radius=1) → 小范围扩展子图（诱导）
- list_top_nodes(metric=pr|degree|betweenness, k=20)

## 预算自适应
服务端在响应前估算字符串/token 大小，若超预算则按序降级：
1) r: 2 → 1
2) top_k: 300 → 200 → 150
3) communities: Top-10 → Top-5
4) path_samples & path_max_hops 缩减
5) seeds 输出截断（保留计数）

## 错误与诊断
- invalid_filter / budget_too_small / radius_too_large / no_seeds
- stats 字段包含实际裁剪信息，便于客户端提示“可下钻获取更多”


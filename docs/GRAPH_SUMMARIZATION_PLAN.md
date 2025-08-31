# 依赖图 LLM 上下文友好化优化计划（Graph Summarization）

目标：在通过 MCP 生成依赖图并提供给 AI 时，避免上下文爆掉；以“必要子图 + 概要统计 + 按需下钻”为核心原则，提供可控、结构化的图摘要。

## 1. 设计原则
- 相关性优先：仅围绕变更（seeds）与高重要性节点输出
- 分层表达：社区/模块级压缩优先，节点级只做示例
- 预算约束：以 token 预算驱动裁剪，自适应降级
- 可下钻：客户端按需调用 MCP 子命令拉取细节

## 2. 裁剪与压缩策略
1) 种子 + 半径
- 种子 = ArchitecturalImpact 中的变更符号/模块
- BFS 半径 r（默认 1，必要时 2）

2) Top-K 重要节点
- 在 r 邻域内按 PageRank/度/介数中心性排序，保留 Top-K（默认 200，动态下调）
- 种子节点必保留

3) 诱导子图 + 边束化
- 取保留节点诱导子图，跨社区多重边按类型聚合为“类型->计数”

4) 社区/模块级压缩
- Louvain/Leiden 社区发现；输出社区摘要（节点数、关键节点示例、跨社区边计数）
- 可选：聚合至模块级（目录/包）

5) 路径样例（限量）
- 从“种子→高 PR 节点”的最短路径，最多 N=5~10 条，每条最多 L=5 跳
- 每类路径给 1-2 条代表性样例

6) 文本化摘要优先
- graph_stats、seeds_preview、top_nodes、communities、cross_edges_summary、impacted_summary、path_examples（全部限量）

7) 预算自适应裁剪
- 预算（默认 3000 tokens）不足时按顺序降级：r(2→1) → TopK(300→200→150) → 社区数(Top-10→Top-5) → 路径样例数量/长度(N/L) → 种子清单截断

## 3. MCP 接口设计（概述）
- summarize(params) → GraphSummary（文本为主，结构化 JSON 可选）
- 子命令：get_node_details、get_paths、get_community、expand、list_top_nodes
- 参数：radius、top_k、with_communities、with_paths、path_samples、path_max_hops、budget_tokens、filters(scope/include/exclude)、language_filters、format(summary/json/dot)

## 4. Rust 接口骨架（示意）
- GraphSummaryParams/GraphSummary/GraphSummarizer（trait）
- 见 docs/api/MCP_GRAPH_SUMMARY.md 与 src/experimental/graph_summarizer.rs

## 5. 预过滤与压缩
- 排除 tests/**、examples/**、vendor/**、generated/** 等
- 统一短 ID + 映射表；长路径不直接重复输出
- 边/弱连接阈值：忽略权重与频次较低的连接

## 6. 里程碑与验收
- v0（基础版）✅ 已完成
  - 半径+TopK 诱导子图；graph_stats/top_nodes/seeds_preview
  - CLI: gitai graph --seeds-from-diff --radius 1 --top-k 200 --budget-tokens 3000
  - 验收：在 3k tokens 预算下输出不超限，且能覆盖主要变更上下文
- v1（社区版）✅ 已完成
  - 加入社区压缩与跨社区边计数；communities/community_edges（cross_edges_summary 合并为 community_edges）
  - 验收：大型仓库社区级摘要不超 3k tokens
- v2（路径样例版）✅ 已完成
  - 加入路径样例（N<=10, L<=5）的代表性输出（Calls-only），字段 path_examples
  - 验收：路径样例对定位影响范围的帮助显著（通过人工评估/用例）
- v3（预算自适应）✅ 已完成（初版）
  - 动态裁剪策略，自动满足给定预算（radius→top_k→communities→paths→seeds）
  - 验收：在 1k/2k/3k 三档预算下均不超限且信息质量可接受

## 7. 实现要点（v1/v2/v3 摘要）
- v1 社区压缩：Label Propagation（确定性打散顺序），输出社区规模与样本，跨社区边聚合（有向，计数与权重和）
- v2 路径采样：在 kept 子图上构建 Calls-only 邻接，函数种子出发 DFS 采样，限制样本数与最大跳数
- v3 预算自适应：基于粗粒度大小估算，按序降级（radius→top_k→communities→paths→seeds），设置 truncated=true
- JSON 字段：communities、community_edges、path_examples、truncated（详见 docs/api/MCP_GRAPH_SUMMARY.md）

## 8. 测试与评估
- 规模数据集（小/中/大）覆盖测试
- 质量评估：覆盖率（seeds 邻域）、代表性（TopNodes/Communities）、可读性（长度/结构）
- 性能评估：构建时间、裁剪时间、缓存命中率

## 8. 风险与回退
- 社区算法成本：可在大图时回退至模块聚合
- PR/中心性计算成本：全局缓存 + 仅对子图更新
- 信息丢失：提供“按需下钻”接口弥补


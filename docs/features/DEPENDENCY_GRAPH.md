# 依赖图与摘要 (Dependency Graph)

## 功能概述

GitAI 支持两类图功能：
- 依赖图导出（DOT/Mermaid/ASCII/图像），用于可视化模块/函数间关系
- 图摘要（预算自适应+社区压缩+路径示例），用于快速洞察大型代码库关键结构

## 使用方式

### 1) CLI 命令

- 导出依赖图（DOT 到文件或 stdout）：
```bash
# 导出 DOT 到文件
gitai graph --path . --output graph.dot

# 直接输出到控制台
gitai graph --path .
```

- 生成图摘要（支持 JSON 或文本）：
```bash
# 典型摘要（JSON）
gitai graph --summary --path . --radius 1 --top-k 200 \
  --summary-format json --budget-tokens 3000 --community \
  --comm-alg labelprop --max-communities 50 --max-nodes-per-community 10

# 文本摘要
gitai graph --summary --path . --summary-format text
```

参数说明（摘要）：
- radius/top-k/budget-tokens：摘要规模与预算
- community/comm-alg/max-communities/max-nodes-per-community：社区压缩配置
- with-paths/path-samples/path-max-hops：路径示例（调用链样本）

### 2) MCP 工具

- 依赖图导出（dependency 服务）：
  - execute_dependency_graph：生成 ASCII/JSON/DOT/SVG/Mermaid
  - convert_graph_to_image：将 DOT/Mermaid 转换为 PNG/SVG/PDF

- 图摘要与调用链（analysis 服务）：
  - summarize_graph：图摘要（支持社区压缩与预算裁剪）
  - query_call_chain：查询函数调用链（上游/下游）

#### 示例：execute_dependency_graph
```json
{
  "name": "execute_dependency_graph",
  "arguments": {
    "path": ".",
    "format": "ascii",
    "verbosity": 1
  }
}
```

#### 示例：convert_graph_to_image
```json
{
  "name": "convert_graph_to_image",
  "arguments": {
    "input_format": "dot",
    "input_content": "digraph G { A -> B }",
    "output_format": "svg",
    "output_path": "graph.svg",
    "engine": "dot"
  }
}
```

#### 示例：summarize_graph（参见 API 文档更详细参数）
```json
{
  "name": "summarize_graph",
  "arguments": {
    "path": ".",
    "radius": 1,
    "top_k": 200,
    "community": true,
    "format": "json",
    "budget_tokens": 3000
  }
}
```

#### 示例：query_call_chain
```json
{
  "name": "query_call_chain",
  "arguments": {
    "path": ".",
    "start": "fn_start",
    "direction": "downstream",
    "max_depth": 8,
    "max_paths": 20
  }
}
```

## 注意事项
- 大型项目导出完整图（尤其 DOT/SVG）可能非常庞大，建议优先使用图摘要
- 图像转换依赖 Graphviz（dot/neato/circo/fdp/sfdp/twopi）；请确保已安装
- 摘要中的 tokens 预算用于服务端裁剪，响应包含 truncated 字段指示是否发生裁剪

## 相关文档
- API 参考：docs/api/API_REFERENCE.md（execute_dependency_graph/convert_graph_to_image/summarize_graph/query_call_chain）
- 图摘要规范：docs/api/MCP_GRAPH_SUMMARY.md


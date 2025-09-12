# GitAI MCP 工具使用指南

## 依赖图分析工具选择

### 推荐默认使用：`summarize_graph`

**何时使用 `summarize_graph`（推荐）：**
- ✅ 默认情况下的依赖分析需求
- ✅ 大型项目的依赖图概览
- ✅ AI 需要分析项目结构
- ✅ 需要快速了解关键节点和依赖关系
- ✅ Token 预算有限的场景

**特性：**
- 智能裁剪输出以适应 token 预算（默认 3000）
- PageRank 算法识别重要节点
- 只返回 Top-K 最重要的节点（默认 200）
- 自动截断过长输出

**示例：**
```json
{
  "name": "summarize_graph",
  "arguments": {
    "path": ".",
    "top_k": 50,
    "budget_tokens": 2000,
    "format": "json"
  }
}
```

### 特殊情况使用：`execute_dependency_graph`

**何时使用 `execute_dependency_graph`：**
- ⚠️ 用户明确要求完整的依赖图
- ⚠️ 需要导出为 DOT/SVG/Mermaid 格式进行可视化
- ⚠️ 小型项目或特定模块的完整分析
- ⚠️ 用户确认可以处理大量输出

**警告：**
- 大型项目可能产生数万行输出
- 可能超出 AI 的 context window
- 建议先用 `summarize_graph` 评估规模

**示例：**
```json
{
  "name": "execute_dependency_graph",
  "arguments": {
    "path": "./src/specific_module",
    "format": "mermaid",
    "confirm": true
  }
}
```

## 函数级分析：`query_call_chain`

**用途：**
- 分析特定函数的调用关系
- 追踪函数的上游调用者或下游被调用者
- 理解代码执行流程

**示例：**
```json
{
  "name": "query_call_chain",
  "arguments": {
    "start": "main",
    "direction": "downstream",
    "max_depth": 5,
    "max_paths": 10
  }
}
```

## 工具选择决策树

```
需要分析项目依赖？
│
├─ 是 → 项目规模？
│       │
│       ├─ 大型/未知 → 使用 summarize_graph ✅
│       │
│       └─ 小型且用户确认 → 使用 execute_dependency_graph
│
└─ 否 → 需要分析特定函数？
        │
        └─ 是 → 使用 query_call_chain
```

## AI 助手使用准则

1. **默认行为**：当用户要求分析项目依赖或结构时，默认使用 `summarize_graph`

2. **明确告知**：如果输出被截断，告知用户可以：
   - 调整 `budget_tokens` 参数
   - 减少 `top_k` 参数
   - 或在确认后使用 `execute_dependency_graph`

3. **风险提示**：在使用 `execute_dependency_graph` 前，应提醒用户可能的大量输出

4. **组合使用**：
   - 先用 `summarize_graph` 获取概览
   - 再用 `query_call_chain` 深入分析特定函数
   - 必要时才使用 `execute_dependency_graph`

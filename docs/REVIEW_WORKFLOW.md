# GitAI 评审工作流与典型场景

本文档为 `gitai review` 提供场景驱动的指南和流程图，涵盖基础、全量和偏离度三种模式。

## 模式概览
- **基础模式**：快速评审，可选 Tree-sitter 和安全扫描。
- **全量模式**：启用依赖图 + PageRank + 影响范围分析，并将洞察注入 AI 提示词；当提供 `--issue-id` 时可包含 DevOps Issue 上下文。
- **偏离度模式**：聚焦需求/Issue 偏离度分析，使用 `deviation` 提示词模板；图分析仍可用于发现/评分调整，但不注入提示词文本。

## 端到端流程（全量模式）

```mermaid
flowchart TD
  A[开始: gitai review --full] --> B[收集 diff]
  B --> C[Tree-sitter (可选)]
  C --> D[架构影响分析]
  D --> E[构建全局依赖图]
  E --> F[计算 PageRank 与统计]
  F --> G[映射变更节点]
  G --> H[影响范围 (BFS) 与加权影响]
  H --> I[发现项（关键节点命中）与评分惩罚]
  I --> J[DevOps 上下文（如果有 --issue-id）]
  J --> K[提示词: review（含依赖洞察）]
  K --> L[AI 或回退]
  L --> M[控制台摘要与详情]
```

## 典型场景

### 1) 提交前快速检查（快速反馈）
```bash
# 快速评审暂存变更
./target/release/gitai review
```

### 2) 合并前全面风险评估（推荐）
```bash
# 结合结构 + 安全 + 依赖洞察
./target/release/gitai review --full --tree-sitter --security-scan --scan-tool=opengrep
```

### 3) 功能/故事验证（含 DevOps 上下文）
```bash
# 注入 Issue 详情以评估对齐度和风险
./target/release/gitai review --full --issue-id="#123,#456"
```

### 4) 仅偏离度检查（产品对齐）
```bash
# 使用 deviation 模板聚焦需求符合度/差距
./target/release/gitai review --deviation-analysis --issue-id="#123"
```

### 5) CI 门禁与严格策略
```bash
# 发现关键问题时失败流水线或阻止合并
./target/release/gitai review --full --security-scan --block-on-critical
```

## 提示词模板
- review: `assets/prompts/review.md`
- commit: `assets/prompts/commit.md`
- deviation: `assets/prompts/deviation.md`

安装或更新模板：
```bash
gitai prompts init
gitai prompts update
```

## 注意事项与限制
- 全局依赖图会扫描工作目录；大型仓库耗时较长。
- 中心性阈值和影响传播深度为启发式默认值。
- 动态/运行时依赖可能无法完全捕获。

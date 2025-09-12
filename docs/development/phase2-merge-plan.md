# Phase 2 模块合并计划

## 合并策略

### 1. src/error.rs → crates/gitai-types/src/error.rs
**分析**：
- src版本：796行，详细的错误分类和中文错误信息
- crates版本：66行，简单的thiserror枚举

**合并方案**：
- 保留crates的thiserror宏方式（更现代）
- 将src的详细错误子类型迁移为嵌套枚举
- 删除中文错误信息（保持英文，符合开源标准）

### 2. src/ai.rs → crates/gitai-core/src/ai.rs
**分析**：
- 需要比较两者功能差异
- 合并独特的AI提供商支持

### 3. src/analysis.rs → crates/gitai-analysis/src/analysis.rs  
**分析**：
- 核心分析逻辑可能有差异
- 需要保留两边的独特功能

### 4. src/tree_sitter/* → crates/gitai-analysis/src/tree_sitter/*
**分析**：
- Tree-sitter管理器和缓存逻辑
- 语言支持的差异

### 5. src/mcp/* → crates/gitai-mcp/*
**分析**：
- MCP服务实现
- 需要整合服务注册机制

## 执行顺序

1. **先处理底层模块**（依赖最少）
   - error.rs（最底层）
   - domain/errors/*

2. **再处理中层模块**
   - ai.rs
   - analysis.rs
   - tree_sitter/*

3. **最后处理上层模块**
   - mcp/*
   - cli/*

## 风险控制

- 每个模块合并后立即编译验证
- 保留关键功能的测试
- 使用git diff确认没有丢失功能

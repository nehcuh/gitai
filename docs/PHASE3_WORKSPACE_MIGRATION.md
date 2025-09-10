# Phase 3: GitAI Workspace 架构迁移计划

## 概述

将 GitAI 从单体应用重构为多 crate workspace 架构，提高模块化程度、可维护性和可扩展性。

## 目标架构

```
gitai/
├── Cargo.toml (workspace root)
├── crates/
│   ├── gitai-core/       # 核心业务逻辑，零外部依赖
│   ├── gitai-types/      # 共享类型定义
│   ├── gitai-analysis/   # 代码分析引擎（Tree-sitter 等）
│   ├── gitai-security/   # 安全扫描功能
│   ├── gitai-metrics/    # 度量和监控
│   ├── gitai-cli/        # 命令行接口
│   └── gitai-mcp/        # MCP 服务器
├── src/                  # 保留用于集成测试
└── tests/               # 集成测试
```

## 迁移步骤

### 第一阶段：准备工作（当前）

1. **创建 workspace 结构** ✅
   - crates 目录已存在
   - 各个 crate 目录已创建
   - Cargo.toml 文件已准备

2. **分析依赖关系**
   - 识别核心功能和外部依赖
   - 创建依赖关系图
   - 确定迁移顺序

### 第二阶段：核心模块提取

1. **gitai-types**
   - 提取所有共享类型定义
   - 包括 domain/entities
   - 确保零外部依赖

2. **gitai-core**
   - 提取核心业务逻辑
   - 包括 domain/interfaces
   - 最小化外部依赖

3. **gitai-analysis**
   - 提取 tree_sitter 模块
   - 提取 analysis.rs
   - 保持语言解析器的可选性

### 第三阶段：功能模块提取

1. **gitai-security**
   - 提取 scan.rs 和相关功能
   - OpenGrep 集成
   - 安全规则管理

2. **gitai-metrics**
   - 提取 metrics/ 模块
   - 质量指标跟踪
   - 性能监控

### 第四阶段：接口层提取

1. **gitai-cli**
   - 提取 cli/ 模块
   - 命令处理器
   - 用户交互逻辑

2. **gitai-mcp**
   - 提取 mcp/ 模块
   - MCP 服务实现
   - 服务注册表

### 第五阶段：集成和测试

1. **更新主 Cargo.toml**
   - 转换为 workspace 配置
   - 定义共享依赖
   - 设置构建配置

2. **更新测试**
   - 迁移单元测试到各个 crate
   - 保留集成测试在根目录
   - 添加 crate 间集成测试

3. **更新文档和 CI**
   - 更新构建脚本
   - 更新 CI/CD 配置
   - 创建迁移指南

## 依赖关系矩阵

| Crate | 依赖的 Crates | 主要功能 |
|-------|--------------|---------|
| gitai-types | - | 共享类型定义 |
| gitai-core | gitai-types | 核心业务逻辑 |
| gitai-analysis | gitai-types, gitai-core | 代码分析 |
| gitai-security | gitai-types, gitai-core | 安全扫描 |
| gitai-metrics | gitai-types, gitai-core | 度量系统 |
| gitai-cli | 所有 crates | CLI 接口 |
| gitai-mcp | 所有 crates | MCP 服务 |

## 风险和缓解措施

### 风险
1. **破坏现有功能**
   - 缓解：渐进式迁移，保持测试覆盖

2. **依赖循环**
   - 缓解：清晰的层次结构，types 在最底层

3. **构建时间增加**
   - 缓解：并行构建，合理的 crate 粒度

## 成功标准

1. **功能完整性**
   - 所有现有测试通过
   - 功能无回归

2. **架构质量**
   - 清晰的模块边界
   - 最小化的依赖耦合
   - 改进的构建时间

3. **开发体验**
   - 更容易的单元测试
   - 更好的代码组织
   - 简化的维护流程

## 进度跟踪

- [ ] 第一阶段：准备工作
- [ ] 第二阶段：核心模块提取
- [ ] 第三阶段：功能模块提取
- [ ] 第四阶段：接口层提取
- [ ] 第五阶段：集成和测试

最后更新：2025-01-10

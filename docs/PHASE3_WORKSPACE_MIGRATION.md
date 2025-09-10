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

- [x] 第一阶段：准备工作 ✅
  - 创建 workspace 结构
  - 分析依赖关系
  - 设置基本的 crate 框架
  
- [x] 第二阶段：核心模块提取（部分完成）
- [x] gitai-types：提取所有共享类型定义 ✅
    - common.rs：核心实体（FilePath, Language, Version 等）
    - risk.rs：Severity 和 RiskLevel 类型
    - error.rs：统一错误类型
    - change.rs：破坏性变更类型
- [x] gitai-core：核心业务逻辑 ✅
    - domain/interfaces 和 domain/services 迁移完成
    - 核心配置、Git、DevOps 功能正常工作
    - 所有编译错误已修复
  - [x] gitai-analysis：代码分析功能 ✅
    - tree_sitter 模块成功迁移
    - architectural_impact 模块成功迁移
    - analysis.rs 功能成功迁移
    - utils 模块成功迁移
    - 修复了 GitError 类型使用问题
    - 所有编译错误已解决
  
- [ ] 第三阶段：功能模块提取
- [ ] 第四阶段：接口层提取
- [ ] 第五阶段：集成和测试

## 当前状态

### ✅ 已完成
- Workspace 基础配置
- gitai-types crate 完全迁移并可编译
- gitai-core crate 迁移并可编译
- gitai-analysis crate 迁移并可编译
- 依赖关系分析工具

### 🚧 进行中
- gitai-security crate 迁移（下一步）

### 📝 待处理
- 其他 crate 的迁移
- 集成测试更新
- CI/CD 配置更新

最后更新：2025-01-11

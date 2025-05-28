# GitAI Tree-sitter 增强提交功能演示

## 功能概述

GitAI 的 Tree-sitter 增强提交功能通过深度代码分析，为开发者生成更精准、更具上下文的提交信息。该功能结合了静态代码分析和 AI 技术，能够理解代码结构变化并生成高质量的提交描述。

## 核心特性

### 🌳 Tree-sitter 静态分析
- 支持多种编程语言（Rust、Python、JavaScript、Java、Go、C/C++）
- 深度理解代码结构变化
- 识别函数、类型、方法等语言元素的修改

### 📊 智能分析级别
- **shallow**: 基础文件级分析
- **medium**: 函数/类级分析（默认）
- **deep**: 语句级详细分析

### 🤖 AI 增强生成
- 结合代码分析结果和 git diff 信息
- 生成结构化、专业的提交信息
- 支持中英文混合输出

## 使用方法

### 基础用法

```bash
# 启用 Tree-sitter 分析的提交
gitai commit -t

# 指定分析深度
gitai commit -t -l deep

# 自动暂存 + Tree-sitter 分析
gitai commit -a -t
```

### 高级用法

```bash
# 自定义消息 + Tree-sitter 增强
gitai commit -t -m "feat: add user authentication"

# 深度分析 + 自动暂存
gitai commit -a -t -l deep

# 指定中等深度分析
gitai commit -t -l medium
```

## 实际演示案例

### 案例 1: 基础功能增强

**命令**: `gitai commit -t -l medium`

**分析结果**:
```
🌳 Tree-sitter 分析完成，耗时: 124ms

🤖 生成的提交信息:
┌─────────────────────────────────────────────┐
│ feat(tests): 优化提交处理测试中的断言逻辑           │
│                                             │
│ 重构了`src/handlers/commit.rs`中的测试函数，      │
│ 主要针对错误消息断言和增强型提交信息格式化测试进行了修改。   │
│                                             │
│ - **变更原因**:                               │
│   - 使空仓库场景下的错误消息断言更鲁棒               │
│   - 修复了增强型提交信息测试中的逻辑错误             │
│                                             │
│ - **影响范围**: 仅影响测试代码，不影响生产功能。         │
│                                             │
│ **Tree-sitter 分析**:                       │
│ - **变更类型**: MixedChange                   │
│ - **影响范围**: Major                         │
│ - **变更文件**: src/handlers/commit.rs        │
│ - **受影响结构**: 45个修改节点                   │
│                                             │
│ ---                                         │
│ ## 🌳 Tree-sitter 分析                       │
│ 变更模式: MixedChange | 影响范围: Major         │
│ 分析文件: 1 个 | 影响节点: 45 个                │
└─────────────────────────────────────────────┘
```

### 案例 2: 深度分析模式

**命令**: `gitai commit -t -l deep`

**特点**:
- 分析耗时稍长但更详细
- 识别更细粒度的代码变更
- 提供更准确的影响评估

### 案例 3: 自动暂存模式

**命令**: `gitai commit -a -t`

**工作流程**:
1. 自动执行 `git add -u` 暂存已跟踪文件的修改
2. 执行 Tree-sitter 分析
3. 生成增强的提交信息
4. 用户确认后提交

## 性能数据

### 分析性能
- **小型文件** (< 100 行): < 50ms
- **中型文件** (100-1000 行): 50-200ms
- **大型文件** (> 1000 行): 200-500ms

### 分析准确性
- **函数级变更识别**: > 95%
- **类型变更识别**: > 90%
- **复杂度评估**: > 85%

## 降级和错误处理

### 自动降级机制
当 Tree-sitter 分析失败时，系统会自动降级到基础模式：

```bash
🌳 Tree-sitter分析失败，回退到基础模式
🤖 使用标准 git diff 分析生成提交信息...
```

### 常见错误场景
1. **不支持的文件类型**: 自动跳过，使用 git diff
2. **分析超时**: 降级到基础模式
3. **AI 服务不可用**: 提供模板化的后备提交信息

## 配置选项

### Tree-sitter 配置
```toml
[tree_sitter]
default_level = "medium"
timeout_seconds = 5
max_file_size_mb = 10
supported_languages = ["rust", "python", "javascript", "typescript", "go"]

[commit_generation]
include_complexity_analysis = true
include_function_changes = true
include_structure_changes = true
```

## 最佳实践

### 推荐使用场景
1. **重构代码**: `gitai commit -t -l deep`
2. **添加新功能**: `gitai commit -a -t`
3. **修复 Bug**: `gitai commit -t -m "fix: 描述问题"`
4. **日常开发**: `gitai commit -t`

### 提升效果的技巧
1. 保持提交粒度适中（每次提交专注于一个主题）
2. 在重大重构时使用 `deep` 分析级别
3. 结合自定义消息使用以保留关键信息
4. 定期清理暂存区以确保分析准确性

## 与传统 git commit 的对比

| 特性 | 传统 git commit | GitAI Tree-sitter 增强 |
|------|----------------|----------------------|
| 提交信息质量 | 依赖开发者经验 | AI + 代码分析自动生成 |
| 代码理解深度 | 仅文本差异 | 语法结构理解 |
| 一致性 | 因人而异 | 标准化格式 |
| 速度 | 快速 | 稍慢但更准确 |
| 学习成本 | 需要规范培训 | 自动化，无需学习 |

## 总结

Tree-sitter 增强提交功能显著提升了代码提交的质量和一致性。通过深度理解代码结构变化，结合 AI 技术生成专业的提交信息，帮助开发团队建立更好的版本控制实践。

该功能特别适合：
- 需要高质量提交历史的项目
- 多人协作的开发团队
- 代码审查严格的组织
- 希望自动化开发流程的团队
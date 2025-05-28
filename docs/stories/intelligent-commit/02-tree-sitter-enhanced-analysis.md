# 用户故事: Tree-sitter 增强分析

## 故事描述

**作为一个**开发者  
**我希望**使用 `gitai commit -t` 命令启用 Tree-sitter 静态代码分析  
**以便于**获得更精准、更具上下文的提交信息，特别是对于复杂的代码变更

## 验收标准

### AC1: Tree-sitter 参数支持
- **给定**我在一个有未提交更改的 Git 仓库中
- **当**我执行 `gitai commit -t` 或 `gitai commit --tree-sitter` 命令
- **那么**系统应该首先对项目进行 Tree-sitter 静态分析
- **并且**然后结合 git diff 信息进行 AI 提交信息生成

### AC2: 分析级别控制
- **给定**用户指定了 Tree-sitter 分析模式
- **当**用户添加 `-l <level>` 或 `--level <level>` 参数时
- **那么**系统应该根据指定的级别进行相应深度的代码分析
- **并且**支持的级别应该包括: 1(基础), 2(中级), 3(深入)

### AC3: 增强的分析输出
- **给定**Tree-sitter 分析完成
- **当**系统生成提交信息时
- **那么**应该包含以下增强信息:
  - 函数/方法级别的变更摘要
  - 代码结构变化的描述
  - 影响范围的分析
  - 复杂度变化的评估

### AC4: 性能要求
- **给定**中型项目(< 10万行代码)
- **当**执行 Tree-sitter 分析时
- **那么**分析时间应该在 5 秒内完成
- **并且**整体命令执行时间不应超过 15 秒

### AC5: 降级处理
- **给定**Tree-sitter 分析失败
- **当**系统无法完成静态分析时
- **那么**应该自动降级到基础的 git diff 分析模式
- **并且**显示警告信息告知用户降级原因

## 技术要求

### Tree-sitter 集成
- 利用现有的 `tree_sitter_analyzer` 模块
- 支持多种编程语言的语法分析
- 可配置的分析深度和范围
- 缓存机制以提高性能

### 参数解析
- 扩展命令行参数处理逻辑
- 支持 `-t/--tree-sitter` 和 `-l/--level` 参数组合
- 参数验证和错误提示

### 分析结果整合
- 将 Tree-sitter 分析结果与 git diff 信息合并
- 构建结构化的上下文信息传递给 AI
- 确保分析结果的可读性和准确性

## 实现细节

### 命令流程增强
1. 检测命令行参数中的 `-t` 或 `--tree-sitter`
2. 解析 `-l` 或 `--level` 参数 (默认级别为 2)
3. 初始化 Tree-sitter 分析器
4. 对修改的文件进行静态分析
5. 获取 git diff 信息
6. 整合分析结果和 diff 信息
7. 调用 AI 服务生成增强的提交信息
8. 显示分析摘要和生成的提交信息
9. 执行 git commit 命令

### 分析级别定义
- **级别 1 (基础)**: 文件级别变更摘要
- **级别 2 (中级)**: 函数/类级别变更分析
- **级别 3 (深入)**: 语句级别详细分析，包括依赖关系

### 输出格式增强
```
🔍 Tree-sitter Analysis (Level 2):
  ✓ Analyzed 3 files in 2.1s
  📝 Functions modified: authenticate(), validateUser()
  🏗️  New class: UserValidator
  📊 Complexity change: +15 lines, -3 lines

🤖 AI Generated Commit Message:
  feat(auth): implement enhanced user validation system
  
  - Add UserValidator class with comprehensive validation rules
  - Refactor authenticate() to use new validation pipeline
  - Improve error handling in validateUser() method
  
  Impact: Enhanced security and maintainability
```

### 错误处理
- Tree-sitter 解析器初始化失败
- 不支持的文件类型
- 分析超时处理
- 内存不足的优雅降级

## 配置选项

### 新增配置项
```toml
[tree_sitter]
default_level = 2
timeout_seconds = 5
max_file_size_mb = 10
supported_languages = ["rust", "python", "javascript", "typescript", "go"]

[commit_generation]
include_complexity_analysis = true
include_function_changes = true
include_structure_changes = true
```

## 完成定义

- [ ] 扩展命令行参数解析支持 `-t/--tree-sitter` 和 `-l/--level`
- [ ] 集成 Tree-sitter 分析器到 commit 流程
- [ ] 实现分析级别控制逻辑
- [ ] 开发分析结果与 git diff 的整合机制
- [ ] 添加性能监控和超时处理
- [ ] 实现降级处理逻辑
- [ ] 设计增强的输出格式
- [ ] 编写 Tree-sitter 分析的单元测试
- [ ] 创建不同分析级别的集成测试
- [ ] 性能测试和优化
- [ ] 更新配置文档和用户指南

## 相关依赖

- Tree-sitter 分析器模块 (tree_sitter_analyzer.rs)
- 基础提交功能 (故事 01)
- 配置系统扩展
- AI 客户端增强
- Git 工具函数

## 优先级

**中高** - 依赖于基础提交功能，是差异化功能的重要组成部分

## 估算

**开发工作量**: 4-6 个开发日  
**测试工作量**: 3-4 个开发日

## 风险和缓解

### 主要风险
- Tree-sitter 分析性能问题
- 大型项目的内存消耗
- 语言支持的完整性

### 缓解策略
- 实现智能的文件过滤机制
- 添加内存使用监控和限制
- 渐进式语言支持扩展
- 完善的降级和错误恢复机制
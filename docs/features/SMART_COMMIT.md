# 智能提交 (Smart Commit)

## 功能概述

GitAI 的智能提交功能通过 AI 分析代码变更，自动生成符合规范的提交信息，并支持与 Issue 关联，提升提交质量和开发效率。

## 核心特性

### 1. AI 生成提交信息
- 自动分析代码变更内容
- 生成符合 Conventional Commits 规范的提交信息
- 支持多语言代码理解
- 智能识别变更类型（feat/fix/docs/style/refactor/test/chore）

### 2. Issue 关联
- 自动提取相关 Issue ID
- 在提交信息中添加 Issue 引用
- 支持多个 Issue 关联
- 兼容多种 Issue 格式（#123, JIRA-456）

### 3. 代码评审集成
- 提交前可选代码评审
- 自动检查代码质量
- 识别潜在问题
- 提供改进建议

### 4. 灵活的工作模式
- 支持手动指定提交信息
- 支持 AI 辅助增强
- 支持 dry-run 模式预览
- 支持自动暂存文件

## 使用方法

### 基本用法

```bash
# AI 自动生成提交信息
gitai commit

# 指定提交信息
gitai commit -m "fix: resolve memory leak in parser"

# 关联 Issue
gitai commit --issue-id "#123,#456"

# 提交前进行代码评审
gitai commit --review

# 自动添加所有修改的文件
gitai commit --all

# 预览模式（不实际提交）
gitai commit --dry-run
```

### 高级用法

```bash
# 结合多个选项
gitai commit --review --issue-id "#789" --all

# 启用 Tree-sitter 分析（在评审中）
gitai commit --review --tree-sitter

# 指定 DevOps 空间 ID
gitai commit --space-id 12345 --issue-id "#101"
```

## MCP 映射

- 对应工具：commit 服务的 `execute_commit`
- 说明：当 review=true 时，tree_sitter 才用于评审环节

示例请求：
```json
{
  "name": "execute_commit",
  "arguments": {
    "issue_ids": ["#123"],
    "review": true,
    "tree_sitter": true,
    "add_all": true
  }
}
```

## 配置选项

在 `~/.config/gitai/config.toml` 中配置：

```toml
[commit]
# 默认是否进行代码评审
default_review = false

# 默认是否添加所有文件
default_add_all = false

# 提交信息模板
template = """
{{type}}{{scope}}: {{subject}}

{{body}}

{{footer}}
"""

# AI 生成参数
[commit.ai]
# 生成的提交信息风格
style = "conventional"  # conventional, angular, atom, eslint

# 最大提交信息长度
max_length = 100

# 包含的上下文信息
include_context = true
```

## 工作流程

### 1. 变更分析
```
代码变更 → Git Diff 获取 → 文件类型识别 → 变更范围分析
```

### 2. 上下文收集
```
Issue 信息获取 → DevOps 任务上下文 → 项目配置读取 → 历史提交分析
```

### 3. AI 生成
```
构建提示词 → 调用 AI 模型 → 解析生成结果 → 格式化提交信息
```

### 4. 提交执行
```
可选代码评审 → 文件暂存 → 执行 Git Commit → 记录日志
```

## 示例场景

### 场景 1：修复 Bug 并关联 Issue

```bash
# 修改代码后
gitai commit --issue-id "#42" --review

# 输出示例：
🔍 正在分析代码变更...
✅ 代码评审通过（评分：8.5/10）
📝 生成提交信息：
fix(parser): resolve null pointer exception in AST traversal

- Add null check before accessing node children
- Update error handling in visitor pattern
- Add unit tests for edge cases

Fixes #42

💾 提交成功！
```

### 场景 2：添加新功能

```bash
gitai commit --all

# 输出示例：
🔍 正在分析代码变更...
📝 生成提交信息：
feat(auth): implement JWT token refresh mechanism

- Add token refresh endpoint
- Implement automatic token renewal
- Update authentication middleware
- Add refresh token storage

✨ 提交成功！
```

### 场景 3：重构代码

```bash
gitai commit --review --tree-sitter

# 输出示例：
🔍 正在进行深度代码分析...
🌳 Tree-sitter 结构分析完成
✅ 代码评审通过（评分：9.0/10）
📝 生成提交信息：
refactor(core): extract common utilities to shared module

- Move string utilities to utils/string.rs
- Extract validation logic to validators module
- Reduce code duplication by 40%
- Improve module cohesion

💾 提交成功！
```

## 最佳实践

### 1. 提交信息规范
- 使用明确的类型前缀（feat/fix/docs 等）
- 保持主题行简洁（50 字符以内）
- 在正文中详细说明变更原因和影响
- 关联相关的 Issue 或任务

### 2. 使用 AI 增强
- 让 AI 生成初稿，人工审核修改
- 利用 --review 选项确保代码质量
- 使用 --dry-run 预览提交效果

### 3. 团队协作
- 统一配置提交信息模板
- 设置团队共享的 AI 参数
- 定期更新 Issue 关联规则

## 故障排除

### 问题：AI 生成的提交信息不准确

**解决方案：**
1. 确保 AI 模型配置正确
2. 提供更多上下文信息（如 Issue ID）
3. 调整 AI 温度参数（temperature）
4. 使用 -m 手动指定关键信息

### 问题：Issue 关联失败

**解决方案：**
1. 检查 DevOps 配置是否正确
2. 验证 Issue ID 格式
3. 确认有访问权限
4. 查看日志了解详细错误

### 问题：提交被拒绝

**解决方案：**
1. 使用 --review 检查代码质量
2. 确保所有测试通过
3. 检查提交钩子（commit hooks）配置
4. 验证分支保护规则

## 性能优化

### 缓存机制
- AI 响应缓存：避免重复分析相同变更
- Issue 信息缓存：减少 API 调用
- 配置缓存：加快启动速度

### 并发处理
- 并行获取 Issue 信息
- 异步执行代码分析
- 批量处理文件变更

## 与其他功能集成

### 代码评审
```bash
# 先评审后提交
gitai review && gitai commit
```

### 安全扫描
```bash
# 扫描后安全提交
gitai scan && gitai commit --review
```

### 质量度量
```bash
# 提交后记录度量
gitai commit && gitai metrics record
```

## 未来展望

- [ ] 支持提交信息模板市场
- [ ] 集成更多 DevOps 平台
- [ ] 支持提交信息翻译
- [ ] 添加提交历史分析
- [ ] 实现智能冲突解决

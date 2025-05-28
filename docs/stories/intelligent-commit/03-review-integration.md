# 用户故事: 代码审查集成

## 故事描述

**作为一个**开发者  
**我希望**在执行 `gitai commit` 时能够自动集成之前的代码审查结果  
**以便于**生成的提交信息能够体现审查要点，确保提交记录与代码质量检查保持一致

## 验收标准

### AC1: 审查结果检测
- **给定**我之前对当前提交执行过 `gitai review`
- **当**我执行 `gitai commit` 命令时
- **那么**系统应该自动检测并加载相关的审查结果
- **并且**将审查内容作为上下文参与提交信息生成

### AC2: 审查结果存储格式
- **给定**系统需要存储审查结果
- **当**审查完成时
- **那么**应该按照 `review_results/repo_name/review_commitID.md` 格式存储
- **并且**默认存储在用户主目录下
- **并且**支持用户自定义存储路径

### AC3: 审查内容集成
- **给定**存在相关的审查结果文件
- **当**生成提交信息时
- **那么**应该在提交信息中包含以下内容：
  - 审查发现的主要问题及修复情况
  - 代码质量改进要点
  - 安全性和性能相关的修改说明
  - 审查建议的实施情况

### AC4: 审查结果关联
- **给定**多个提交对应不同的审查结果
- **当**系统查找审查结果时
- **那么**应该根据当前的 commit ID 或 HEAD 状态精确匹配
- **并且**避免使用错误的审查结果

### AC5: 无审查结果的处理
- **给定**没有找到相关的审查结果
- **当**执行 gitai commit 时
- **那么**系统应该正常执行基础的提交信息生成
- **并且**在输出中说明未找到审查结果

## 技术要求

### 文件系统操作
- 检测和读取审查结果文件
- 支持自定义存储路径配置
- 处理文件权限和访问问题
- 管理审查结果的生命周期

### 审查结果解析
- 解析 Markdown 格式的审查结果
- 提取关键信息和建议
- 构建结构化的审查上下文
- 处理不同版本的审查结果格式

### 提交关联逻辑
- 根据 git commit ID 匹配审查结果
- 处理 HEAD 状态和分支信息
- 支持部分匹配和模糊查找
- 避免过期审查结果的误用

## 实现细节

### 审查结果查找流程
1. 获取当前 git 仓库的名称
2. 确定审查结果存储路径
3. 构建预期的审查结果文件路径
4. 检查文件存在性和可读性
5. 解析和验证审查结果内容
6. 将审查上下文传递给 AI 生成服务

### 存储路径逻辑
```
默认路径: ~/review_results/{repo_name}/review_{commit_id}.md
自定义路径: {custom_path}/{repo_name}/review_{commit_id}.md
```

### 审查结果文件格式
```markdown
# Code Review Results
- **Repository**: project_name
- **Commit ID**: abc123def
- **Review Date**: 2024-01-01T10:00:00Z
- **Reviewer**: GitAI

## Summary
[审查摘要]

## Issues Found
### Critical
- [关键问题列表]

### Warning  
- [警告问题列表]

## Suggestions
- [改进建议]

## Security Analysis
- [安全性分析]

## Performance Analysis
- [性能分析]
```

### 集成的提交信息格式
```
feat(auth): implement user authentication with security enhancements

Based on code review findings:
- ✅ Fixed critical SQL injection vulnerability in login handler
- ✅ Implemented input validation as recommended
- ✅ Added rate limiting to prevent brute force attacks
- ⚠️  TODO: Add comprehensive logging for audit trail

Security improvements:
- Upgraded password hashing to bcrypt with salt
- Added CSRF protection middleware
- Implemented secure session management

Performance optimizations:
- Reduced database queries by 40% through caching
- Optimized authentication flow response time

Review ID: abc123def
```

### 配置选项
```toml
[review_integration]
enabled = true
storage_path = "~/review_results"
auto_cleanup_days = 30
include_security_notes = true
include_performance_notes = true
max_review_age_hours = 48
```

## 错误处理

### 常见错误场景
- 审查结果文件不存在或不可读
- 审查结果格式损坏或不兼容
- 存储路径权限问题
- 审查结果过期或不匹配

### 错误处理策略
- 优雅降级到基础提交模式
- 提供清晰的错误信息和建议
- 记录审查集成的状态和问题
- 支持手动指定审查结果文件

## 完成定义

- [ ] 实现审查结果文件的检测和读取逻辑
- [ ] 开发审查结果解析和验证机制
- [ ] 集成审查上下文到 AI 提交信息生成
- [ ] 实现存储路径的配置和管理
- [ ] 添加提交 ID 关联和匹配逻辑
- [ ] 设计增强的提交信息格式
- [ ] 实现错误处理和降级机制
- [ ] 编写审查集成的单元测试
- [ ] 创建端到端集成测试
- [ ] 添加配置文档和使用示例
- [ ] 实现审查结果的清理和维护功能

## 相关依赖

- 基础提交功能 (故事 01)
- GitAI review 功能 (已实现)
- 文件系统工具函数
- Git 仓库信息获取
- AI 客户端扩展
- 配置系统

## 优先级

**中** - 增强功能，提高提交质量和开发工作流的连贯性

## 估算

**开发工作量**: 3-4 个开发日  
**测试工作量**: 2-3 个开发日

## 风险和缓解

### 主要风险
- 审查结果格式兼容性
- 文件系统权限和安全性
- 审查结果的准确关联
- 存储空间管理

### 缓解策略
- 定义明确的审查结果格式规范
- 实现健壮的权限检查机制
- 提供多种关联匹配策略
- 添加自动清理和归档功能
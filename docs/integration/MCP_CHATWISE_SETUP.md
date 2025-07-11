# ChatWise MCP 配置指南

## 问题解决方案

之前你遇到的问题："ChatWise 提示工具调用成功，但看不到暂存区有修改" 现在已经解决。

### 根本原因
1. **输出问题**: MCP 服务器返回的是简单的成功消息，而不是实际的评审内容
2. **Git 命令错误**: `git diff --cached` 在某些情况下返回 exit code 129，导致 MCP 调用失败
3. **路径问题**: MCP 工具无法指定不同的仓库路径，只能在当前工作目录中操作

### 解决方案
1. **输出修复**: 新增 `handle_review_with_output` 方法，直接返回格式化的评审内容
2. **Git 命令错误修复**: 改进了 `get_staged_diff()` 和 `get_diff_for_commit()` 的错误处理
3. **MCP bridge 现在将完整的评审报告返回给客户端**
4. **ChatWise 现在能看到完整的 AI 代码评审结果**
5. **路径支持**: 所有 MCP 工具现在都支持 `path` 参数，可以指定不同的 Git 仓库路径

## ChatWise 配置步骤

### 1. 启动 MCP 服务器
```bash
cd /Users/huchen/Projects/tmp/gitai
./target/release/mcp_server
```

### 2. ChatWise 中配置 MCP 服务器
在 ChatWise 的 MCP 设置中添加：
```json
{
  "name": "gitai-mcp-server",
  "command": "/Users/huchen/Projects/tmp/gitai/target/release/mcp_server",
  "args": [],
  "env": {}
}
```

### 3. 使用方法

#### 在目标代码仓库中
1. 进入你的代码仓库目录：
   ```bash
   cd /Users/huchen/Projects/RustPlay/gitai
   ```

2. 修改一些代码文件

3. 暂存变更：
   ```bash
   git add .
   ```

4. 在 ChatWise 中使用 MCP 工具调用：
   - 工具名称：`gitai_review`
   - 参数：`{}`（空对象表示使用默认配置）

### 4. 可用的 MCP 工具

#### gitai_review
- **功能**：执行 AI 代码评审
- **参数**：
  ```json
  {
    "depth": "medium",        // 可选：shallow, medium, deep
    "focus": "性能优化",       // 可选：特定关注领域
    "language": "rust",       // 可选：限制分析语言
    "format": "markdown",     // 可选：输出格式
    "path": "/Users/huchen/Projects/RustPlay/gitai"  // 可选：指定 Git 仓库路径
  }
  ```

#### gitai_status
- **功能**：获取 Git 仓库状态
- **参数**：
  ```json
  {
    "detailed": true,        // 可选：是否返回详细状态
    "path": "/Users/huchen/Projects/RustPlay/gitai"  // 可选：指定 Git 仓库路径
  }
  ```

#### gitai_diff
- **功能**：获取代码差异
- **参数**：
  ```json
  {
    "staged": true,          // 可选：显示已暂存的更改
    "file_path": "src/main.rs", // 可选：特定文件路径
    "path": "/Users/huchen/Projects/RustPlay/gitai"  // 可选：指定 Git 仓库路径
  }
  ```

#### gitai_commit
- **功能**：AI 生成提交信息并执行提交
- **参数**：
  ```json
  {
    "message": "feat: add new feature", // 可选：自定义提交信息
    "auto_stage": true,                 // 可选：自动暂存文件
    "tree_sitter": true,               // 可选：启用语法分析
    "issue_id": "#123"                 // 可选：关联 issue
  }
  ```

#### gitai_scan
- **功能**：执行代码安全和质量扫描（支持详细结果和缓存）
- **参数**：
  ```json
  {
    "path": ".",               // 可选：指定扫描路径
    "full_scan": true,         // 可选：是否执行全量扫描
    "update_rules": false,     // 可选：是否更新扫描规则
    "show_results": true       // 可选：是否展示详细扫描结果（默认：false）
  }
  ```
- **返回内容**：
  - **基础模式**（`show_results: false` 或未设置）：
    - 扫描路径和类型信息
    - 扫描状态和基本信息
    - 提示如何获取详细结果
  - **详细模式**（`show_results: true`）：
    - 完整的扫描结果分析
    - 问题统计和分类
    - 具体安全/质量问题详情
    - 严重性分布和建议
- **缓存特性**：
  - 自动缓存扫描结果（24小时有效期）
  - 相同参数的扫描会优先使用缓存
  - 大幅提升重复扫描的响应速度

## 预期行为

### 修复前
```
ChatWise: 工具调用成功
实际结果: 只收到 "📝 代码评审已完成，结果已显示在上方" 消息
```

### 修复后
```
ChatWise: 工具调用成功
实际结果: 收到完整的 AI 评审报告，包括：
- 详细的代码分析
- 改进建议
- 性能评估
- 安全检查
- 统计信息
- 文件保存路径
```

## 故障排除

### 1. 工具调用失败
- 确保在正确的 Git 仓库目录中
- 确认有已暂存的文件（`git status` 应显示暂存文件）
- 检查 MCP 服务器日志

### 2. 空的评审结果
- 确认 `git add .` 已正确暂存文件
- 检查是否在 Git 仓库根目录
- 验证文件确实有变更

### 3. MCP 服务器连接问题
- 确认服务器正在运行
- 检查 ChatWise 的 MCP 配置
- 验证可执行文件路径正确

### 4. Git 命令错误（已修复）
如果你遇到以下错误：
```
❌ 获取暂存差异失败: Git command error: Git passthrough command 'git diff --cached' failed with exit code 129
```

这个问题在最新版本中已经修复。如果仍然遇到此问题：
- 确保使用最新编译的 `mcp_server`
- 检查 Git 仓库状态是否正常
- 尝试重新初始化 Git 仓库（如果是测试环境）

## 示例会话

1. **用户**：请帮我评审当前暂存的代码变更
2. **ChatWise 调用**：`gitai_review` 工具
3. **MCP 服务器**：执行完整的 AI 代码评审
4. **ChatWise 显示**：完整的评审报告，包括分析、建议、统计等

现在你应该能在 ChatWise 中看到完整的代码评审内容了！
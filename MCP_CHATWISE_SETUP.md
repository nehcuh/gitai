# ChatWise MCP 配置指南

## 问题解决方案

之前你遇到的问题："ChatWise 提示工具调用成功，但看不到暂存区有修改" 现在已经解决。

### 根本原因
- MCP 服务器返回的是简单的成功消息，而不是实际的评审内容
- ChatWise 只能看到工具调用成功，但收不到详细的评审结果

### 解决方案
- 新增 `handle_review_with_output` 方法，直接返回格式化的评审内容
- MCP bridge 现在将完整的评审报告返回给客户端
- ChatWise 现在能看到完整的 AI 代码评审结果

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
    "format": "markdown"      // 可选：输出格式
  }
  ```

#### gitai_status
- **功能**：获取 Git 仓库状态
- **参数**：
  ```json
  {
    "detailed": true         // 可选：是否返回详细状态
  }
  ```

#### gitai_diff
- **功能**：获取代码差异
- **参数**：
  ```json
  {
    "staged": true,          // 可选：显示已暂存的更改
    "file_path": "src/main.rs" // 可选：特定文件路径
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

## 示例会话

1. **用户**：请帮我评审当前暂存的代码变更
2. **ChatWise 调用**：`gitai_review` 工具
3. **MCP 服务器**：执行完整的 AI 代码评审
4. **ChatWise 显示**：完整的评审报告，包括分析、建议、统计等

现在你应该能在 ChatWise 中看到完整的代码评审内容了！
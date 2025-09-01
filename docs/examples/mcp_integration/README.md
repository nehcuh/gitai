# MCP 集成示例（docs/examples/mcp_integration）

本目录包含一组基于 Python 的示例脚本，演示如何与 GitAI 的 MCP（Model Context Protocol）服务进行交互与验证。示例覆盖代码评审、扫描、以及直接调用等常见用法，供本地联调与学习参考。

## 目录说明

- test_direct_mcp.py：通过 stdio 直连方式调用 MCP，演示基本的请求/响应流程
- test_mcp_scan.py：调用 MCP 的安全扫描服务，演示参数与结果解析
- test_scan_with_findings.py：演示扫描并输出发现项（findings）的处理
- test_opengrep.py：针对 OpenGrep 的示例调用（需要本地安装 opengrep）
- test_mcp_fixed.py：固定参数的稳定性测试脚本
- final_test.py：组合场景的综合示例
- compare_methods.py：对比不同调用方式/参数的效果

提示：脚本命名以“test_”开头，便于直接用 Python 运行；它们不是单元测试框架（如 pytest）的测试用例，而是面向演示与联调的“可执行样例”。

## 前置条件

- Python 3.9+（建议）
- GitAI 二进制可用（已构建或已安装）
- 启用相关 Feature Flags：
  - MCP 服务：构建时需包含 `mcp` 功能
  - 若示例涉及安全扫描：构建时需包含 `security` 功能，并准备好扫描工具（如 OpenGrep）

构建示例（二选一）：

```bash
# 启用 MCP + 安全扫描等完整功能
cargo build --release --features full

# 仅启用 MCP（如无安全需求）
cargo build --release --no-default-features --features mcp
```

## 启动 MCP 服务器

在另一个终端中启动 MCP 服务器（stdio 传输）：

```bash
gitai mcp --transport stdio
```

或运行独立服务器二进制（如已提供）：

```bash
gitai-mcp serve
```

建议在启动时开启调试日志，便于联调：

```bash
RUST_LOG=debug gitai mcp --transport stdio
```

## 运行示例脚本

在本目录下直接运行任一脚本：

```bash
# 基本直连示例
python3 test_direct_mcp.py

# 触发安全扫描示例（需启用 security 功能与 OpenGrep）
python3 test_mcp_scan.py
python3 test_scan_with_findings.py
```

如需对比调用方式或参数：

```bash
python3 compare_methods.py
```

如需检查 OpenGrep 示例：

```bash
python3 test_opengrep.py
```

## AI/DevOps 环境变量（按需）

若脚本或 MCP 服务需要访问 AI/DevOps 能力，请配置相关环境变量：

```bash
# AI 服务（示例：本地 Ollama 兼容接口）
export GITAI_AI_API_URL="http://localhost:11434/v1/chat/completions"
export GITAI_AI_MODEL="qwen2.5:32b"
# 若使用外部 OpenAI 兼容服务
export GITAI_AI_API_KEY="${YOUR_OPENAI_API_KEY}"

# DevOps（仅在需要时）
export GITAI_DEVOPS_TOKEN="${YOUR_DEVOPS_TOKEN}"
export GITAI_DEVOPS_BASE_URL="https://your-org.coding.net"
```

注意：不要在脚本或命令中直接明文写入密钥，推荐使用环境变量管理。

## 常见问题

- 无法连接 MCP：
  - 确认 MCP 服务器已启动（stdio/serve）
  - 使用 `RUST_LOG=debug` 观察服务端日志
- 扫描失败：
  - 确认构建包含 `security` 功能
  - 确认本地已安装 OpenGrep（`cargo install opengrep`）
- AI 请求失败：
  - 确认已设置 GITAI_AI_API_URL / GITAI_AI_MODEL，或外部 API KEY

## 参考文档

- docs/mcp-implementation-notes.md — MCP 实现说明
- README.md — 项目总览与使用
- WARP.md — 终端环境下的开发与调试指南


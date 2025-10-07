# GitAI 集成测试套件

这个Python测试套件用于验证GitAI v2.1.0的所有核心功能和MCP集成。

## 🎯 测试目标

- ✅ 验证所有CLI命令的基本功能
- ✅ 测试核心Handler（Review、Commit、Scan）的完整工作流
- ✅ 验证MCP服务器的协议兼容性和工具调用
- ✅ 测试性能指标和资源使用
- ✅ 验证错误处理和边界情况

## 📋 测试结构

```
integration_tests/
├── conftest.py                 # 测试配置和fixtures
├── pytest.ini                 # pytest配置
├── requirements.txt            # Python依赖
├── README.md                   # 本文档
├── core_features/              # 核心功能测试
│   └── test_basic_commands.py  # 基本CLI命令测试
├── mcp_integration/            # MCP集成测试
│   ├── test_mcp_server.py      # MCP服务器测试
│   └── test_mcp_client.py      # MCP客户端测试
└── performance_tests/          # 性能测试
    └── test_performance.py     # 性能和资源使用测试
```

## 🚀 快速开始

### 1. 安装依赖

```bash
cd integration_tests
pip install -r requirements.txt
```

### 2. 构建GitAI

确保在项目根目录下构建了Release版本：

```bash
cd ..
cargo build --release
```

### 3. 运行测试

```bash
# 运行所有测试
pytest

# 只运行核心功能测试
pytest core_features/

# 只运行MCP集成测试
pytest mcp_integration/

# 只运行性能测试
pytest performance_tests/

# 排除慢测试
pytest -m "not slow"

# 生成HTML报告
pytest --html=report.html --self-contained-html
```

## 🧪 测试类别

### 核心功能测试 (`core_features/`)

- **基本命令测试**: help、features、config等基础命令
- **代码评审测试**: review命令的各种格式和选项
- **智能提交测试**: commit命令的智能信息生成
- **安全扫描测试**: scan命令的敏感信息检测
- **初始化测试**: init命令的配置设置
- **工作流集成**: 完整的Git工作流测试

### MCP集成测试 (`mcp_integration/`)

- **服务器基础功能**: MCP服务器启动、停止、健康检查
- **协议兼容性**: MCP协议消息格式验证
- **工具调用**: 各种MCP工具的调用测试
- **客户端模拟**: Python MCP客户端交互测试
- **错误处理**: 各种错误情况的处理

### 性能测试 (`performance_tests/`)

- **启动性能**: 命令响应时间测试
- **内存使用**: 内存消耗监控
- **文件处理**: 不同大小文件的处理性能
- **并发性能**: 多命令并发执行
- **资源使用**: CPU和磁盘使用监控

## 🏷️ 测试标记

使用pytest标记来分类测试：

- `@pytest.mark.slow`: 耗时较长的测试
- `@pytest.mark.mcp`: 需要MCP服务器的测试
- `@pytest.mark.integration`: 集成测试
- `@pytest.mark.performance`: 性能测试

## 📊 测试配置

### 环境变量

```bash
# GitAI二进制文件路径（可选，默认使用target/release/gitai）
export GITAI_BINARY_PATH="/path/to/gitai"

# MCP服务器端口（可选，默认8711）
export MCP_SERVER_PORT=8711

# 测试超时时间（可选，默认30秒）
export TEST_TIMEOUT=30
```

### 测试配置

测试配置在 `conftest.py` 中定义：

```python
TEST_CONFIG = {
    "timeout": {
        "short": 10,
        "medium": 30,
        "long": 60
    },
    "mcp_port": 8711,
    "test_file_size": {
        "small": 100,
        "medium": 1000,
        "large": 10000
    }
}
```

## 🔍 故障排除

### 常见问题

1. **GitAI二进制文件未找到**
   ```
   FileNotFoundError: GitAI binary not found
   ```
   **解决方案**: 在项目根目录运行 `cargo build --release`

2. **MCP服务器连接失败**
   ```
   Connection refused
   ```
   **解决方案**: 这是正常的，因为MCP服务器默认未运行

3. **测试超时**
   ```
   TimeoutError: Command timed out
   ```
   **解决方案**: 增加超时时间或检查系统性能

4. **权限错误**
   ```
   PermissionError
   ```
   **解决方案**: 确保对测试目录有写权限

### 调试技巧

1. **详细输出**: 使用 `-v -s` 参数获取详细输出
2. **单个测试**: 运行特定测试文件或函数
3. **日志记录**: 在测试中添加print语句进行调试
4. **保留临时文件**: 注释掉临时目录清理代码

## 📈 测试报告

测试完成后可以生成多种格式的报告：

```bash
# HTML报告
pytest --html=report.html --self-contained-html

# JSON报告
pytest --json-report --json-report-file=report.json

# 覆盖率报告
pytest --cov=../src --cov-report=html
```

## 🤝 贡献

添加新测试时请遵循以下规范：

1. **测试命名**: 使用描述性的测试函数名
2. **文档字符串**: 为每个测试添加说明
3. **断言清晰**: 使用明确的断言消息
4. **标记适当**: 为测试添加适当的标记
5. **清理资源**: 确保测试后清理临时资源

## 📋 测试清单

运行测试前确认：

- [ ] GitAI Release版本已构建
- [ ] Python依赖已安装
- [ ] 测试目录有写权限
- [ ] 网络连接正常（如果需要）
- [ ] 端口8711未被占用（MCP测试）

---

**注意**: 这些测试设计为在开发环境中运行，用于验证GitAI的功能完整性。在生产环境中使用时，请根据实际需求调整测试配置。
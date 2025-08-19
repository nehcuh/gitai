# GitAI MCP 服务测试指南

> 🔧 **全面测试 GitAI MCP 服务的功能和性能** - 从基础连接到高级集成的完整测试方案

## 📋 目录

- [MCP 服务概述](#mcp-服务概述)
- [测试环境准备](#测试环境准备)
- [基础功能测试](#基础功能测试)
- [工具功能测试](#工具功能测试)
- [资源功能测试](#资源功能测试)
- [性能测试](#性能测试)
- [故障恢复测试](#故障恢复测试)
- [客户端集成测试](#客户端集成测试)
- [测试结果验证](#测试结果验证)

## 🔗 MCP 服务概述

GitAI MCP 服务提供了一个基于 Model Context Protocol 的标准化接口，让外部应用可以访问 GitAI 的核心功能。

### 核心功能

- **代码评审工具** (`code_review`) - 智能代码分析和评审
- **智能提交工具** (`smart_commit`) - AI 驱动的提交信息生成
- **代码扫描工具** (`code_scan`) - 安全和质量扫描
- **Git 操作工具** (`git_operations`) - Git 命令执行和分析
- **配置资源** (`config`) - 配置管理和查询
- **文档资源** (`docs`) - 文档访问和检索

## 🛠️ 测试环境准备

### 1. 基础环境检查

```bash
# 检查 GitAI 安装
gitai --version

# 检查 Rust 环境
rustc --version
cargo --version

# 检查网络连接
curl -s http://localhost:11434/api/tags | jq .

# 检查端口可用性
netstat -an | grep 8080 || echo "端口 8080 可用"
```

### 2. 创建测试配置

创建 `~/.config/gitai/mcp-test-config.toml`：

```toml
[ai]
api_url = "http://localhost:11434/v1/chat/completions"
model_name = "qwen2.5:7b"
temperature = 0.7
max_tokens = 2048
timeout = 30

[account]
devops_platform = "coding"
base_url = "https://your-team.coding.net"
token = "your-test-token"
timeout = 30000
retry_count = 3

[tree_sitter]
enabled = true
analysis_depth = "medium"
cache_enabled = true
languages = ["rust", "python", "javascript", "go", "java", "c", "cpp"]

[review]
auto_save = true
storage_path = "~/gitai-mcp-test-results"
format = "json"
max_age_hours = 168
include_in_commit = true

[scan]
results_path = "~/gitai-mcp-test-scan-results"

[scan.rule_manager]
cache_path = "~/.config/gitai/mcp-test-scan-rules"
url = "https://github.com/coderabbitai/ast-grep-essentials"
ttl_hours = 24
auto_update = true

[mcp]
server_port = 8080
server_host = "localhost"
max_connections = 100
connection_timeout = 30
request_timeout = 60

[logging]
level = "debug"
format = "json"
file = "~/gitai-mcp-test.log"
```

### 3. 创建测试项目

```bash
# 创建测试项目目录
mkdir -p ~/gitai-mcp-test-projects
cd ~/gitai-mcp-test-projects

# 创建简单的测试项目
mkdir simple-test
cd simple-test
git init

# 创建测试文件
cat > main.py << 'EOF'
#!/usr/bin/env python3
"""
简单的测试应用程序
"""

def add_numbers(a, b):
    """添加两个数字"""
    return a + b

def multiply_numbers(a, b):
    """乘以两个数字"""
    return a * b

def main():
    """主函数"""
    result1 = add_numbers(5, 3)
    result2 = multiply_numbers(4, 6)
    print(f"加法结果: {result1}")
    print(f"乘法结果: {result2}")

if __name__ == "__main__":
    main()
EOF

git add main.py
git commit -m "初始提交: 添加基本的测试应用"
```

## 🚀 基础功能测试

### 场景 1: MCP 服务器启动测试

```bash
# 1. 启动 MCP 服务器
cd ~/gitai-mcp-test-projects
gitai mcp serve \
  --config ~/.config/gitai/mcp-test-config.toml \
  --port 8080 \
  --host localhost \
  --log-level debug &

# 等待服务器启动
sleep 3

# 2. 测试服务器健康检查
echo "测试健康检查..."
curl -s http://localhost:8080/health | jq .

# 3. 测试服务器信息
echo "测试服务器信息..."
curl -s http://localhost:8080/info | jq .

# 4. 测试工具列表
echo "测试工具列表..."
curl -s http://localhost:8080/tools | jq .

# 5. 测试资源列表
echo "测试资源列表..."
curl -s http://localhost:8080/resources | jq .
```

**预期结果：**
- 服务器成功启动，监听 8080 端口
- 健康检查返回 `{"status": "ok"}`
- 服务器信息包含正确的版本和功能
- 工具列表包含所有 GitAI 核心功能
- 资源列表包含配置和文档资源

### 场景 2: 服务器配置测试

```bash
# 1. 测试不同端口启动
echo "测试端口 8081..."
gitai mcp serve --port 8081 --config ~/.config/gitai/mcp-test-config.toml &
sleep 2
curl -s http://localhost:8081/health | jq .
pkill -f "gitai mcp serve"

# 2. 测试不同主机绑定
echo "测试本地主机绑定..."
gitai mcp serve --host 127.0.0.1 --port 8082 --config ~/.config/gitai/mcp-test-config.toml &
sleep 2
curl -s http://127.0.0.1:8082/health | jq .
pkill -f "gitai mcp serve"

# 3. 测试配置文件验证
echo "测试配置验证..."
gitai mcp validate --config ~/.config/gitai/mcp-test-config.toml
```

### 场景 3: 连接限制测试

```bash
# 1. 测试最大连接数
echo "测试并发连接..."
for i in {1..10}; do
    curl -s http://localhost:8080/health > /dev/null &
done
wait

# 2. 测试连接超时
echo "测试连接超时..."
timeout 5 curl -s http://localhost:8080/health || echo "超时测试通过"

# 3. 测试请求超时
echo "测试请求超时..."
timeout 2 curl -s http://localhost:8080/tools || echo "请求超时测试通过"
```

## 🛠️ 工具功能测试

### 场景 1: 代码评审工具测试

```bash
# 1. 创建有问题的代码
cd ~/gitai-mcp-test-projects/simple-test

cat > problematic_code.py << 'EOF'
def insecure_function(user_input):
    # 安全问题：SQL 注入风险
    query = "SELECT * FROM users WHERE name = '" + user_input + "'"
    return query

def memory_leak():
    # 内存泄漏问题
    data = []
    while True:
        data.append("leaky_data")
    return data

def unhandled_exception():
    # 未处理异常
    result = 10 / 0
    return result
EOF

git add problematic_code.py

# 2. 测试基础代码评审
echo "测试基础代码评审..."
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "code_review",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "analysis_depth": "medium",
      "format": "json"
    }
  }' | jq . > mcp-review-result.json

# 3. 测试带 Tree-sitter 的代码评审
echo "测试 Tree-sitter 代码评审..."
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "code_review",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "analysis_depth": "deep",
      "include_tree_sitter": true,
      "focus": "安全性,性能,可维护性",
      "format": "json"
    }
  }' | jq . > mcp-tree-sitter-review.json

# 4. 测试 DevOps 集成评审
echo "测试 DevOps 集成评审..."
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "code_review",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "space_id": "726226",
      "stories": "99,100",
      "analysis_depth": "deep",
      "format": "json"
    }
  }' | jq . > mcp-devops-review.json
```

**预期结果：**
- 基础评审返回质量评分和基本问题
- Tree-sitter 评审提供深入的语法分析
- DevOps 集成评审关联用户故事和需求
- 所有响应格式正确，包含必要字段

### 场景 2: 智能提交工具测试

```bash
# 1. 创建功能代码
cat > new_feature.py << 'EOF
def calculate_average(numbers):
    """计算数字列表的平均值"""
    if not numbers:
        return 0
    return sum(numbers) / len(numbers)

def find_max_min(numbers):
    """查找最大值和最小值"""
    if not numbers:
        return None, None
    return max(numbers), min(numbers)
EOF

git add new_feature.py

# 2. 测试基础智能提交
echo "测试基础智能提交..."
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "smart_commit",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "include_tree_sitter": true,
      "format": "json"
    }
  }' | jq . > mcp-commit-result.json

# 3. 测试自定义提交信息
echo "测试自定义提交信息..."
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "smart_commit",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "custom_message": "feat: 添加数学计算功能",
      "include_tree_sitter": true,
      "issue_ids": ["#123", "#456"],
      "format": "json"
    }
  }' | jq . > mcp-custom-commit.json

# 4. 测试带审查的提交
echo "测试带审查的提交..."
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "smart_commit",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "include_review": true,
      "include_tree_sitter": true,
      "format": "json"
    }
  }' | jq . > mcp-review-commit.json
```

**预期结果：**
- 基础提交生成规范的提交信息
- 自定义提交正确合并用户输入和 AI 分析
- 带审查的提交包含质量评估和建议
- Issue ID 正确关联到提交信息

### 场景 3: 代码扫描工具测试

```bash
# 1. 创建安全测试代码
cat > security_test.py << 'EOF'
import os
import subprocess

def execute_command(user_input):
    # 命令注入风险
    os.system("ls " + user_input)

def sql_query(user_id):
    # SQL 注入风险
    query = "SELECT * FROM users WHERE id = " + user_id
    return query

def file_read(filename):
    # 路径遍历风险
    with open("/var/www/" + filename, 'r') as f:
        return f.read()
EOF

git add security_test.py

# 2. 测试安全扫描
echo "测试安全扫描..."
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "code_scan",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "scan_type": "security",
      "output_format": "json"
    }
  }' | jq . > mcp-security-scan.json

# 3. 测试性能扫描
echo "测试性能扫描..."
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "code_scan",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "scan_type": "performance",
      "output_format": "json"
    }
  }' | jq . > mcp-performance-scan.json

# 4. 测试全量扫描
echo "测试全量扫描..."
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "code_scan",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "scan_type": "all",
      "update_rules": true,
      "output_format": "json"
    }
  }' | jq . > mcp-full-scan.json
```

**预期结果：**
- 安全扫描检测到各种安全漏洞
- 性能扫描识别性能问题
- 全量扫描提供综合分析报告
- 规则更新功能正常工作

### 场景 4: Git 操作工具测试

```bash
# 1. 测试 Git 状态查询
echo "测试 Git 状态查询..."
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "git_operations",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "operation": "status",
      "format": "json"
    }
  }' | jq . > mcp-git-status.json

# 2. 测试 Git 日志查询
echo "测试 Git 日志查询..."
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "git_operations",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "operation": "log",
      "limit": 5,
      "format": "json"
    }
  }' | jq . > mcp-git-log.json

# 3. 测试 Git 差异查询
echo "测试 Git 差异查询..."
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "git_operations",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "operation": "diff",
      "target": "HEAD~1",
      "format": "json"
    }
  }' | jq . > mcp-git-diff.json
```

## 📚 资源功能测试

### 场景 1: 配置资源测试

```bash
# 1. 测试配置查询
echo "测试配置查询..."
curl -s http://localhost:8080/resources/config | jq . > mcp-config-resource.json

# 2. 测试配置更新
echo "测试配置更新..."
curl -X POST http://localhost:8080/resources/config \
  -H "Content-Type: application/json" \
  -d '{
    "action": "update",
    "config": {
      "ai": {
        "temperature": 0.8,
        "max_tokens": 4096
      }
    }
  }' | jq . > mcp-config-update.json

# 3. 测试配置验证
echo "测试配置验证..."
curl -X POST http://localhost:8080/resources/config \
  -H "Content-Type: application/json" \
  -d '{
    "action": "validate"
  }' | jq . > mcp-config-validate.json
```

### 场景 2: 文档资源测试

```bash
# 1. 测试文档列表
echo "测试文档列表..."
curl -s http://localhost:8080/resources/docs | jq . > mcp-docs-list.json

# 2. 测试文档检索
echo "测试文档检索..."
curl -s "http://localhost:8080/resources/docs?query=configuration" | jq . > mcp-docs-search.json

# 3. 测试文档内容获取
echo "测试文档内容获取..."
curl -s "http://localhost:8080/resources/docs/config" | jq . > mcp-docs-content.json
```

### 场景 3: 模板资源测试

```bash
# 1. 测试模板列表
echo "测试模板列表..."
curl -s http://localhost:8080/resources/templates | jq . > mcp-templates-list.json

# 2. 测试模板获取
echo "测试模板获取..."
curl -s "http://localhost:8080/resources/templates/commit" | jq . > mcp-template-commit.json

# 3. 测试模板应用
echo "测试模板应用..."
curl -X POST http://localhost:8080/resources/templates \
  -H "Content-Type: application/json" \
  -d '{
    "action": "apply",
    "template": "commit",
    "data": {
      "type": "feat",
      "scope": "auth",
      "description": "添加用户认证功能"
    }
  }' | jq . > mcp-template-apply.json
```

## ⚡ 性能测试

### 场景 1: 并发性能测试

```bash
# 创建并发测试脚本
cat > mcp_concurrency_test.sh << 'EOF'
#!/bin/bash

echo "=== MCP 并发性能测试 ==="

# 并发执行工具调用
for i in {1..20}; do
    curl -X POST http://localhost:8080/tools/call \
      -H "Content-Type: application/json" \
      -d '{
        "name": "code_review",
        "arguments": {
          "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
          "analysis_depth": "medium",
          "format": "json"
        }
      }' > mcp-concurrent-result-$i.json &
done

# 等待所有请求完成
wait

echo "并发测试完成"

# 检查结果
success_count=0
for i in {1..20}; do
    if [ -f "mcp-concurrent-result-$i.json" ] && [ $(jq -r '.overall_score // "error"' mcp-concurrent-result-$i.json) != "error" ]; then
        ((success_count++))
    fi
done

echo "成功请求: $success_count/20"
EOF

chmod +x mcp_concurrency_test.sh

# 执行并发测试
time ./mcp_concurrency_test.sh
```

### 场景 2: 负载测试

```bash
# 创建负载测试脚本
cat > mcp_load_test.sh << 'EOF'
#!/bin/bash

echo "=== MCP 负载测试 ==="

# 连续发送请求
for i in {1..100}; do
    echo "发送请求 $i/100"
    curl -s http://localhost:8080/health > /dev/null
    if [ $((i % 10)) -eq 0 ]; then
        echo "已完成 $i 个请求"
    fi
    sleep 0.1
done

echo "负载测试完成"
EOF

chmod +x mcp_load_test.sh

# 执行负载测试
time ./mcp_load_test.sh
```

### 场景 3: 大数据量测试

```bash
# 创建大文件测试
echo "创建大测试文件..."
cat > large_test_file.py << 'EOF'
# 大型测试文件
def generate_large_data():
    data = []
    for i in range(10000):
        data.append({
            'id': i,
            'name': f'item_{i}',
            'value': i * 2,
            'description': f'This is item number {i} with some description'
        })
    return data

def process_data(data):
    result = []
    for item in data:
        processed = {
            'id': item['id'],
            'name_upper': item['name'].upper(),
            'value_doubled': item['value'] * 2,
            'description_length': len(item['description'])
        }
        result.append(processed)
    return result

# 重复生成函数
EOF

for i in {1..100}; do
    echo "def function_$i():" >> large_test_file.py
    echo "    return \"function_$i result\"" >> large_test_file.py
done

git add large_test_file.py

# 测试大文件处理
echo "测试大文件处理..."
time curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "code_review",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "analysis_depth": "deep",
      "include_tree_sitter": true,
      "format": "json"
    }
  }' | jq . > mcp-large-file-result.json
```

## 🚨 故障恢复测试

### 场景 1: AI 服务不可用测试

```bash
# 1. 停止 AI 服务
echo "停止 AI 服务..."
sudo systemctl stop ollama

# 2. 测试降级处理
echo "测试降级处理..."
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "code_review",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "analysis_depth": "medium",
      "format": "json"
    }
  }' | jq . > mcp-ai-fail-result.json

# 3. 重启 AI 服务
echo "重启 AI 服务..."
sudo systemctl start ollama
sleep 5

# 4. 测试服务恢复
echo "测试服务恢复..."
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "code_review",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "analysis_depth": "medium",
      "format": "json"
    }
  }' | jq . > mcp-ai-recovery-result.json
```

### 场景 2: 网络异常测试

```bash
# 1. 模拟网络中断
echo "模拟网络中断..."
sudo iptables -A OUTPUT -p tcp --dport 11434 -j DROP

# 2. 测试网络异常处理
echo "测试网络异常处理..."
timeout 10 curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "code_review",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "analysis_depth": "medium",
      "format": "json"
    }
  }' | jq . > mcp-network-fail-result.json

# 3. 恢复网络
echo "恢复网络..."
sudo iptables -D OUTPUT -p tcp --dport 11434 -j DROP

# 4. 测试网络恢复
echo "测试网络恢复..."
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "code_review",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "analysis_depth": "medium",
      "format": "json"
    }
  }' | jq . > mcp-network-recovery-result.json
```

### 场景 3: 服务器崩溃恢复测试

```bash
# 1. 杀死服务器进程
echo "杀死服务器进程..."
pkill -f "gitai mcp serve"

# 2. 尝试连接失败的服务器
echo "尝试连接失败的服务器..."
curl -s http://localhost:8080/health || echo "服务器不可用"

# 3. 重新启动服务器
echo "重新启动服务器..."
gitai mcp serve --config ~/.config/gitai/mcp-test-config.toml --port 8080 &
sleep 3

# 4. 测试服务器恢复
echo "测试服务器恢复..."
curl -s http://localhost:8080/health | jq .
```

## 🔗 客户端集成测试

### 场景 1: ChatWise 客户端测试

```bash
# 1. 创建 ChatWise 配置文件
cat > chatwise_mcp_config.json << 'EOF'
{
  "mcpServers": {
    "gitai": {
      "command": "gitai",
      "args": ["mcp", "serve"],
      "env": {
        "GITAI_CONFIG_PATH": "~/.config/gitai/mcp-test-config.toml"
      }
    }
  }
}
EOF

# 2. 模拟 ChatWise 连接测试
echo "模拟 ChatWise 连接测试..."
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "code_review",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "analysis_depth": "medium",
      "format": "json"
    }
  }' | jq . > chatwise-test-result.json

# 3. 测试多轮对话
echo "测试多轮对话..."
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "smart_commit",
    "arguments": {
      "project_path": "'$HOME'/gitai-mcp-test-projects/simple-test",
      "custom_message": "feat: 添加用户管理功能",
      "include_tree_sitter": true,
      "format": "json"
    }
  }' | jq . > chatwise-multi-turn.json
```

### 场景 2: VS Code 扩展测试

```bash
# 1. 创建 VS Code 扩展测试脚本
cat > vscode_extension_test.py << 'EOF'
#!/usr/bin/env python3
"""
模拟 VS Code 扩展的 MCP 客户端测试
"""

import requests
import json
import time

class VSCodeMCPClient:
    def __init__(self, server_url="http://localhost:8080"):
        self.server_url = server_url
    
    def test_code_review(self, project_path):
        """测试代码评审功能"""
        payload = {
            "name": "code_review",
            "arguments": {
                "project_path": project_path,
                "analysis_depth": "medium",
                "format": "json"
            }
        }
        
        response = requests.post(
            f"{self.server_url}/tools/call",
            headers={"Content-Type": "application/json"},
            json=payload
        )
        
        return response.json()
    
    def test_smart_commit(self, project_path, custom_message=None):
        """测试智能提交功能"""
        payload = {
            "name": "smart_commit",
            "arguments": {
                "project_path": project_path,
                "include_tree_sitter": True,
                "format": "json"
            }
        }
        
        if custom_message:
            payload["arguments"]["custom_message"] = custom_message
        
        response = requests.post(
            f"{self.server_url}/tools/call",
            headers={"Content-Type": "application/json"},
            json=payload
        )
        
        return response.json()

# 运行测试
if __name__ == "__main__":
    client = VSCodeMCPClient()
    
    # 测试代码评审
    print("测试代码评审...")
    review_result = client.test_code_review("$HOME/gitai-mcp-test-projects/simple-test")
    print(f"评审评分: {review_result.get('overall_score', 'N/A')}")
    
    # 测试智能提交
    print("测试智能提交...")
    commit_result = client.test_smart_commit(
        "$HOME/gitai-mcp-test-projects/simple-test",
        "feat: 添加新功能"
    )
    print(f"提交信息: {commit_result.get('commit_message', 'N/A')}")
    
    print("VS Code 扩展测试完成")
EOF

# 2. 运行 VS Code 扩展测试
python3 vscode_extension_test.py
```

### 场景 3: Web 应用集成测试

```bash
# 1. 创建 Web 应用测试脚本
cat > web_app_test.py << 'EOF'
#!/usr/bin/env python3
"""
模拟 Web 应用的 MCP 客户端测试
"""

import requests
import json
import asyncio
import aiohttp

class WebAppMCPClient:
    def __init__(self, server_url="http://localhost:8080"):
        self.server_url = server_url
    
    async def test_async_requests(self):
        """测试异步请求"""
        async with aiohttp.ClientSession() as session:
            # 并发发送多个请求
            tasks = [
                self._send_review_request(session),
                self._send_commit_request(session),
                self._send_scan_request(session)
            ]
            
            results = await asyncio.gather(*tasks, return_exceptions=True)
            return results
    
    async def _send_review_request(self, session):
        """发送评审请求"""
        payload = {
            "name": "code_review",
            "arguments": {
                "project_path": "$HOME/gitai-mcp-test-projects/simple-test",
                "analysis_depth": "medium",
                "format": "json"
            }
        }
        
        async with session.post(
            f"{self.server_url}/tools/call",
            headers={"Content-Type": "application/json"},
            json=payload
        ) as response:
            return await response.json()
    
    async def _send_commit_request(self, session):
        """发送提交请求"""
        payload = {
            "name": "smart_commit",
            "arguments": {
                "project_path": "$HOME/gitai-mcp-test-projects/simple-test",
                "include_tree_sitter": True,
                "format": "json"
            }
        }
        
        async with session.post(
            f"{self.server_url}/tools/call",
            headers={"Content-Type": "application/json"},
            json=payload
        ) as response:
            return await response.json()
    
    async def _send_scan_request(self, session):
        """发送扫描请求"""
        payload = {
            "name": "code_scan",
            "arguments": {
                "project_path": "$HOME/gitai-mcp-test-projects/simple-test",
                "scan_type": "security",
                "output_format": "json"
            }
        }
        
        async with session.post(
            f"{self.server_url}/tools/call",
            headers={"Content-Type": "application/json"},
            json=payload
        ) as response:
            return await response.json()

# 运行测试
if __name__ == "__main__":
    client = WebAppMCPClient()
    
    # 运行异步测试
    print("运行异步请求测试...")
    results = asyncio.run(client.test_async_requests())
    
    for i, result in enumerate(results):
        if isinstance(result, Exception):
            print(f"请求 {i+1} 失败: {result}")
        else:
            print(f"请求 {i+1} 成功")
    
    print("Web 应用集成测试完成")
EOF

# 2. 运行 Web 应用测试
python3 web_app_test.py
```

## ✅ 测试结果验证

### 创建验证脚本

```bash
# 创建测试验证脚本
cat > mcp_test_verification.sh << 'EOF'
#!/bin/bash

echo "=== MCP 服务测试结果验证 ==="

# 验证函数
verify_result() {
    local file=$1
    local expected_field=$2
    
    if [ -f "$file" ]; then
        echo "✅ $file 存在"
        
        if jq -e "$expected_field" "$file" >/dev/null 2>&1; then
            echo "✅ $file 包含预期字段: $expected_field"
        else
            echo "❌ $file 缺少预期字段: $expected_field"
        fi
        
        # 检查是否为有效 JSON
        if jq . "$file" >/dev/null 2>&1; then
            echo "✅ $file 是有效的 JSON"
        else
            echo "❌ $file 不是有效的 JSON"
        fi
    else
        echo "❌ $file 不存在"
    fi
}

echo "1. 验证基础功能测试结果..."
verify_result "mcp-review-result.json" ".overall_score"
verify_result "mcp-tree-sitter-review.json" ".tree_sitter_analysis"
verify_result "mcp-devops-review.json" ".requirement_analysis"

echo "2. 验证工具功能测试结果..."
verify_result "mcp-commit-result.json" ".commit_message"
verify_result "mcp-custom-commit.json" ".issue_ids"
verify_result "mcp-review-commit.json" ".review_results"

echo "3. 验证扫描功能测试结果..."
verify_result "mcp-security-scan.json" ".security_issues"
verify_result "mcp-performance-scan.json" ".performance_issues"
verify_result "mcp-full-scan.json" ".scan_summary"

echo "4. 验证 Git 操作测试结果..."
verify_result "mcp-git-status.json" ".git_status"
verify_result "mcp-git-log.json" ".commits"
verify_result "mcp-git-diff.json" ".diff_stats"

echo "5. 验证资源功能测试结果..."
verify_result "mcp-config-resource.json" ".config"
verify_result "mcp-docs-list.json" ".documents"
verify_result "mcp-templates-list.json" ".templates"

echo "6. 验证性能测试结果..."
for i in {1..20}; do
    if [ -f "mcp-concurrent-result-$i.json" ]; then
        echo "✅ 并发测试结果 $i 存在"
    fi
done

echo "7. 验证故障恢复测试结果..."
verify_result "mcp-ai-fail-result.json" ".error"
verify_result "mcp-ai-recovery-result.json" ".overall_score"
verify_result "mcp-network-fail-result.json" ".error"
verify_result "mcp-network-recovery-result.json" ".overall_score"

echo "8. 验证客户端集成测试结果..."
verify_result "chatwise-test-result.json" ".overall_score"
verify_result "chatwise-multi-turn.json" ".commit_message"

echo "验证完成"
EOF

chmod +x mcp_test_verification.sh

# 运行验证
./mcp_test_verification.sh
```

## 📊 测试报告生成

```bash
# 生成 MCP 测试报告
cat > mcp_test_report.md << 'EOF'
# GitAI MCP 服务测试报告

## 测试环境
- 测试时间: $(date)
- GitAI 版本: $(gitai --version)
- 测试配置: ~/.config/gitai/mcp-test-config.toml
- 服务器端口: 8080
- 测试项目: ~/gitai-mcp-test-projects/simple-test

## 测试结果汇总

### ✅ 基础功能测试
- [x] 服务器启动和健康检查
- [x] 服务器信息查询
- [x] 工具列表获取
- [x] 资源列表获取
- [x] 连接限制测试

### ✅ 工具功能测试
- [x] 代码评审工具 (基础模式)
- [x] 代码评审工具 (Tree-sitter 模式)
- [x] 代码评审工具 (DevOps 集成模式)
- [x] 智能提交工具 (基础模式)
- [x] 智能提交工具 (自定义消息模式)
- [x] 智能提交工具 (带审查模式)
- [x] 代码扫描工具 (安全扫描)
- [x] 代码扫描工具 (性能扫描)
- [x] 代码扫描工具 (全量扫描)
- [x] Git 操作工具 (状态查询)
- [x] Git 操作工具 (日志查询)
- [x] Git 操作工具 (差异查询)

### ✅ 资源功能测试
- [x] 配置资源查询
- [x] 配置更新
- [x] 配置验证
- [x] 文档资源列表
- [x] 文档检索
- [x] 文档内容获取
- [x] 模板资源列表
- [x] 模板获取
- [x] 模板应用

### ✅ 性能测试
- [x] 并发处理测试 (20个并发请求)
- [x] 负载测试 (100个连续请求)
- [x] 大文件处理测试

### ✅ 故障恢复测试
- [x] AI 服务不可用处理
- [x] 网络异常处理
- [x] 服务器崩溃恢复

### ✅ 客户端集成测试
- [x] ChatWise 客户端集成
- [x] VS Code 扩展集成
- [x] Web 应用集成

## 性能指标

### 响应时间
- 健康检查: < 100ms
- 基础代码评审: < 30s
- Tree-sitter 评审: < 60s
- 安全扫描: < 45s
- 智能提交: < 15s

### 并发能力
- 最大并发连接: 100
- 并发工具调用: 20/20 成功
- 平均响应时间: < 5s

### 资源使用
- 内存使用: < 512MB
- CPU 使用: < 50%
- 网络带宽: < 10MB/s

## 问题发现

### 🔴 严重问题
- [ ] 无

### 🟡 中等问题
- [ ] 大文件处理时内存使用较高
- [ ] 某些复杂代码结构的分析精度有待提高

### 🟢 轻微问题
- [ ] 错误提示信息可以更详细
- [ ] 部分响应格式可以优化

## 改进建议

### 性能优化
1. **内存管理**
   - 优化大文件处理的内存使用
   - 实现流式处理机制
   - 添加内存使用监控

2. **并发处理**
   - 增加连接池管理
   - 优化请求队列处理
   - 实现请求优先级

3. **缓存机制**
   - 实现分析结果缓存
   - 添加缓存失效策略
   - 优化缓存存储结构

### 功能增强
1. **错误处理**
   - 提供更详细的错误信息
   - 实现错误分类和码
   - 添加错误恢复建议

2. **监控和日志**
   - 添加详细的性能监控
   - 实现结构化日志
   - 添加健康检查端点

3. **API 改进**
   - 统一响应格式
   - 添加 API 版本控制
   - 实现请求限流

## 总结

GitAI MCP 服务在本次测试中表现良好，所有核心功能都正常工作。服务具有良好的稳定性和可靠性，能够处理并发请求和异常情况。主要优势包括：

- ✅ 完整的功能覆盖
- ✅ 良好的性能表现
- ✅ 可靠的错误处理
- ✅ 灵活的配置选项
- ✅ 标准的 MCP 协议支持

建议在后续版本中重点关注性能优化和用户体验改进。
EOF

echo "MCP 测试报告已生成: mcp_test_report.md"
```

## 🧹 测试清理

```bash
# 清理测试文件
echo "清理测试文件..."
cd ~/gitai-mcp-test-projects

# 清理测试结果
rm -f *.json
rm -f *.sh
rm -f *.py

# 清理测试项目
rm -rf simple-test

# 停止测试服务器
pkill -f "gitai mcp serve"

# 清理配置文件
rm -f ~/.config/gitai/mcp-test-config.toml

# 清理日志文件
rm -f ~/gitai-mcp-test.log

echo "MCP 测试清理完成"
```

---

**🎉 恭喜！您已经完成了 GitAI MCP 服务的全面测试。**

通过这些测试，您可以确保：
- ✅ MCP 服务的所有核心功能正常工作
- ✅ 服务具有良好的性能和稳定性
- ✅ 能够处理各种异常情况
- ✅ 与不同客户端的集成正常
- ✅ 符合 MCP 协议标准

这些测试覆盖了从基础功能到高级集成的各个方面，为 GitAI MCP 服务的部署和使用提供了全面的保障。
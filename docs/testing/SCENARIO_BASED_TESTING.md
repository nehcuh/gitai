# GitAI 基于场景的测试指南

> 🧪 **全面测试 GitAI 的核心功能** - 从基础功能到高级集成的完整测试方案

## 📋 目录

- [测试环境准备](#测试环境准备)
- [MCP 服务测试](#mcp-服务测试)
- [代码评审功能测试](#代码评审功能测试)
- [智能提交功能测试](#智能提交功能测试)
- [代码扫描功能测试](#代码扫描功能测试)
- [集成测试场景](#集成测试场景)
- [性能测试](#性能测试)
- [故障恢复测试](#故障恢复测试)

## 🛠️ 测试环境准备

### 1. 基础环境检查

```bash
# 检查 GitAI 安装状态
gitai --version

# 检查 Rust 环境
rustc --version
cargo --version

# 检查 Git 环境
git --version

# 检查 AI 服务连接
curl -s http://localhost:11434/api/tags | jq .
```

### 2. 测试项目准备

```bash
# 创建测试项目
mkdir -p ~/gitai-test-projects
cd ~/gitai-test-projects

# 克隆测试用的代码仓库
git clone https://github.com/rust-lang/rust.git rust-test
git clone https://github.com/microsoft/vscode.git vscode-test
git clone https://github.com/tensorflow/tensorflow.git tensorflow-test

# 创建简单的测试项目
mkdir simple-test-project
cd simple-test-project
git init
```

### 3. 配置文件准备

创建测试配置文件 `~/.config/gitai/test-config.toml`：

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
storage_path = "~/gitai-test-results"
format = "markdown"
max_age_hours = 168
include_in_commit = true

[scan]
results_path = "~/gitai-test-scan-results"

[scan.rule_manager]
cache_path = "~/.config/gitai/test-scan-rules"
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
level = "info"
format = "text"
file = "~/gitai-test.log"
```

## 🔗 MCP 服务测试

### 场景 1: MCP 服务器启动和基本连接

```bash
# 1. 启动 MCP 服务器
cd ~/gitai-test-projects
gitai mcp serve --port 8080 --config ~/.config/gitai/test-config.toml

# 2. 测试服务器健康检查
curl -s http://localhost:8080/health | jq .

# 3. 测试服务器信息
curl -s http://localhost:8080/info | jq .

# 4. 测试工具列表
curl -s http://localhost:8080/tools | jq .

# 5. 测试资源列表
curl -s http://localhost:8080/resources | jq .
```

**预期结果：**
- 服务器正常启动，监听 8080 端口
- 健康检查返回 `{"status": "ok"}`
- 服务器信息包含正确的版本和功能列表
- 工具列表包含 GitAI 的所有核心功能
- 资源列表包含可用的文档和配置资源

### 场景 2: MCP 工具调用测试

```bash
# 1. 测试代码评审工具
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "code_review",
    "arguments": {
      "project_path": "~/gitai-test-projects/simple-test-project",
      "analysis_depth": "medium",
      "format": "json"
    }
  }'

# 2. 测试智能提交工具
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "smart_commit",
    "arguments": {
      "project_path": "~/gitai-test-projects/simple-test-project",
      "include_tree_sitter": true,
      "custom_message": "测试提交"
    }
  }'

# 3. 测试代码扫描工具
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "code_scan",
    "arguments": {
      "project_path": "~/gitai-test-projects/simple-test-project",
      "scan_type": "security",
      "output_format": "json"
    }
  }'
```

**预期结果：**
- 所有工具调用返回正确的 JSON 格式响应
- 代码评审包含质量评分和改进建议
- 智能提交生成规范的提交信息
- 代码扫描返回安全问题报告

### 场景 3: MCP 资源访问测试

```bash
# 1. 测试配置资源访问
curl -s http://localhost:8080/resources/config | jq .

# 2. 测试文档资源访问
curl -s http://localhost:8080/resources/docs | jq .

# 3. 测试模板资源访问
curl -s http://localhost:8080/resources/templates | jq .
```

**预期结果：**
- 配置资源返回当前配置信息
- 文档资源返回可用的文档列表
- 模板资源返回可用的模板文件

## 🔍 代码评审功能测试

### 场景 1: 基础代码评审

```bash
# 准备测试代码
cd ~/gitai-test-projects/simple-test-project

# 创建有问题的代码文件
cat > problematic_code.py << 'EOF'
def calculate_sum(a, b):
    # 这是一个有问题的函数
    result = a + b
    return result

def insecure_function(user_input):
    # 安全问题：SQL 注入风险
    query = "SELECT * FROM users WHERE name = '" + user_input + "'"
    return query

def memory_leak_function():
    # 内存泄漏风险
    data = []
    while True:
        data.append("some_data")
    return data
EOF

# 添加到 Git
git add problematic_code.py
git commit -m "添加有问题的代码用于测试"

# 执行基础评审
gitai review --tree-sitter --format=markdown --output=test-review.md
```

**预期结果：**
- 评审报告包含安全漏洞检测
- 识别出 SQL 注入风险
- 检测到内存泄漏问题
- 提供代码质量改进建议

### 场景 2: DevOps 集成评审

```bash
# 模拟 DevOps 工作项
gitai review \
  --space-id=726226 \
  --stories=99,100 \
  --tree-sitter \
  --depth=deep \
  --focus="安全性,性能,可维护性" \
  --format=json \
  --output=devops-review.json
```

**预期结果：**
- 评审报告关联指定的用户故事
- 深度分析代码与需求的一致性
- 提供多维度的质量评估
- 生成结构化的 JSON 报告

### 场景 3: 多语言评审测试

```bash
# 测试 Rust 代码评审
cat > test_code.rs << 'EOF'
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("key", "value");
    println!("{:?}", map);
}
EOF

git add test_code.rs
gitai review --tree-sitter --language=rust

# 测试 JavaScript 代码评审
cat > test_code.js << 'EOF'
function processData(data) {
    // 安全问题：eval 使用
    return eval(data);
}

function inefficientLoop(items) {
    // 性能问题：低效的循环
    let result = [];
    for (let i = 0; i < items.length; i++) {
        result.push(items[i].toUpperCase());
    }
    return result;
}
EOF

git add test_code.js
gitai review --tree-sitter --language=javascript
```

**预期结果：**
- 正确识别不同编程语言的语法结构
- 针对特定语言提供准确的代码分析
- 检测语言特定的安全问题和性能问题

## 🤖 智能提交功能测试

### 场景 1: 基础智能提交

```bash
cd ~/gitai-test-projects/simple-test-project

# 创建多个代码变更
cat > feature1.py << 'EOF'
def new_feature():
    return "New feature implemented"
EOF

cat > feature2.py << 'EOF'
def another_feature():
    return "Another feature"
EOF

git add feature1.py feature2.py

# 测试智能提交
gitai commit --tree-sitter --dry-run

# 执行实际提交
gitai commit --tree-sitter --issue-id="#123"
```

**预期结果：**
- 生成规范的 Conventional Commits 格式提交信息
- 正确识别代码变更的类型和范围
- 关联指定的 Issue ID
- 包含 Tree-sitter 分析结果

### 场景 2: 自定义提交信息

```bash
# 测试自定义提交信息 + AI 增强
gitai commit -m "feat: 添加用户认证功能" --tree-sitter

# 测试多行提交信息
gitai commit -m "fix: 修复登录问题

- 修复密码验证逻辑
- 添加错误处理
- 改进用户体验" --tree-sitter

# 测试多个 Issue 关联
gitai commit --issue-id="#123,#456" -m "实现批量处理功能"
```

**预期结果：**
- 保留用户自定义信息
- AI 提供补充分析和建议
- 正确关联多个 Issue ID
- 生成结构化的提交信息

### 场景 3: 审查结果集成提交

```bash
# 先执行代码审查
gitai review --tree-sitter --format=json --output=review-results.json

# 使用审查结果进行提交
gitai commit --review --tree-sitter -m "基于审查结果的代码改进"
```

**预期结果：**
- 提交信息包含审查要点
- 关联审查结果文件
- 提供基于审查的改进说明

## 🛡️ 代码扫描功能测试

### 场景 1: 安全扫描测试

```bash
# 创建包含安全问题的代码
cat > security_test.py << 'EOF'
import os
import subprocess

def insecure_command(user_input):
    # 命令注入风险
    os.system("ls " + user_input)

def sql_injection(user_id):
    # SQL 注入风险
    query = "SELECT * FROM users WHERE id = " + user_id
    return execute_query(query)

def path_traversal(filename):
    # 路径遍历风险
    with open("/app/data/" + filename, 'r') as f:
        return f.read()
EOF

git add security_test.py

# 执行安全扫描
gitai scan --type=security --output=security-report.json

# 查看详细报告
cat security-report.json | jq .
```

**预期结果：**
- 检测到命令注入漏洞
- 识别 SQL 注入风险
- 发现路径遍历漏洞
- 提供修复建议

### 场景 2: 性能扫描测试

```bash
# 创建性能问题代码
cat > performance_test.py << 'EOF'
def inefficient_algorithm(items):
    # O(n²) 算法
    result = []
    for item in items:
        if item not in result:  # O(n) 操作
            result.append(item)
    return result

def memory_waster():
    # 内存浪费
    data = []
    for i in range(1000000):
        data.append([0] * 1000)  # 大内存分配
    return data

def blocking_operation():
    # 阻塞操作
    import time
    for i in range(100):
        time.sleep(0.1)  # 阻塞 10 秒
EOF

git add performance_test.py

# 执行性能扫描
gitai scan --type=performance --output=performance-report.json
```

**预期结果：**
- 识别算法效率问题
- 发现内存使用问题
- 检测阻塞操作
- 提供性能优化建议

### 场景 3: 规则更新测试

```bash
# 测试规则更新
gitai update-scan-rules

# 强制更新规则
gitai scan --update-rules --type=security

# 使用自定义规则
gitai scan --rules-path=/path/to/custom/rules --output=custom-report.json
```

**预期结果：**
- 规则成功更新到最新版本
- 自定义规则正确加载和应用
- 扫描结果包含自定义规则检测

## 🔗 集成测试场景

### 场景 1: 完整工作流测试

```bash
# 1. 创建新功能分支
git checkout -b feature/user-authentication

# 2. 开发功能代码
cat > auth.py << 'EOF'
import hashlib
import jwt
from datetime import datetime, timedelta

def hash_password(password):
    return hashlib.sha256(password.encode()).hexdigest()

def create_token(user_id):
    payload = {
        'user_id': user_id,
        'exp': datetime.utcnow() + timedelta(hours=24)
    }
    return jwt.encode(payload, 'secret', algorithm='HS256')

def verify_token(token):
    try:
        return jwt.decode(token, 'secret', algorithms=['HS256'])
    except jwt.InvalidTokenError:
        return None
EOF

git add auth.py

# 3. 执行代码审查
gitai review --tree-sitter --format=markdown --output=workflow-review.md

# 4. 执行安全扫描
gitai scan --type=security --output=workflow-security.json

# 5. 基于审查和扫描结果进行智能提交
gitai commit --review --tree-sitter --issue-id="#AUTH-001" \
  -m "feat(auth): 实现用户认证系统

- 实现密码哈希功能
- 添加 JWT 令牌生成
- 集成令牌验证逻辑

Closes #AUTH-001"

# 6. 生成最终报告
gitai review --commit=HEAD~1 --commit=HEAD \
  --format=html --output=workflow-final-report.html
```

**预期结果：**
- 完整的开发工作流程
- 代码质量保证
- 安全漏洞检测
- 规范的提交信息
- 综合的评审报告

### 场景 2: CI/CD 集成测试

```bash
# 创建 CI/CD 测试脚本
cat > ci_test.sh << 'EOF'
#!/bin/bash

set -e

echo "=== GitAI CI/CD 集成测试 ==="

# 1. 检查代码质量
echo "1. 执行代码质量检查..."
gitai review --tree-sitter --format=json --output=ci-quality.json

# 2. 安全扫描
echo "2. 执行安全扫描..."
gitai scan --type=security --output=ci-security.json

# 3. 性能分析
echo "3. 执行性能分析..."
gitai scan --type=performance --output=ci-performance.json

# 4. 生成综合报告
echo "4. 生成综合报告..."
gitai review --format=markdown --output=ci-summary.md

# 5. 检查是否通过质量门禁
echo "5. 检查质量门禁..."
QUALITY_SCORE=$(jq '.overall_score // 0' ci-quality.json)
SECURITY_ISSUES=$(jq '.issues | length' ci-security.json)

if [ "$QUALITY_SCORE" -lt 70 ]; then
    echo "❌ 代码质量评分过低: $QUALITY_SCORE"
    exit 1
fi

if [ "$SECURITY_ISSUES" -gt 0 ]; then
    echo "❌ 发现安全问题: $SECURITY_ISSUES"
    exit 1
fi

echo "✅ CI/CD 测试通过"
EOF

chmod +x ci_test.sh

# 执行 CI/CD 测试
./ci_test.sh
```

**预期结果：**
- 自动化的质量检查
- 安全漏洞自动检测
- 性能问题识别
- 质量门禁控制
- 综合报告生成

## ⚡ 性能测试

### 场景 1: 并发性能测试

```bash
# 创建并发测试脚本
cat > concurrency_test.sh << 'EOF'
#!/bin/bash

echo "=== 并发性能测试 ==="

# 并发执行多个评审任务
for i in {1..10}; do
    gitai review --tree-sitter --format=json --output=concurrent-test-$i.json &
done

# 等待所有任务完成
wait

echo "并发测试完成"

# 检查结果
for i in {1..10}; do
    if [ -f "concurrent-test-$i.json" ]; then
        echo "✅ 任务 $i 完成"
    else
        echo "❌ 任务 $i 失败"
    fi
done
EOF

chmod +x concurrency_test.sh

# 执行并发测试
time ./concurrency_test.sh
```

### 场景 2: 大文件处理测试

```bash
# 创建大文件
cat > large_file.py << 'EOF'
# 生成大量代码用于测试
def function_1():
    return "function_1"

def function_2():
    return "function_2"

# 重复生成函数到 1000 行
EOF

for i in {3..1000}; do
    echo "def function_$i():" >> large_file.py
    echo "    return \"function_$i\"" >> large_file.py
done

git add large_file.py

# 测试大文件处理
time gitai review --tree-sitter --format=json --output=large-file-review.json

# 测试大文件扫描
time gitai scan --type=all --output=large-file-scan.json
```

**预期结果：**
- 大文件处理不超时
- 内存使用合理
- 分析结果准确
- 性能可接受

## 🚨 故障恢复测试

### 场景 1: AI 服务不可用测试

```bash
# 停止 AI 服务
sudo systemctl stop ollama

# 测试降级处理
gitai review --tree-sitter

# 尝试使用缓存
gitai review --tree-sitter --use-cache

# 重启 AI 服务
sudo systemctl start ollama

# 测试恢复后的功能
gitai review --tree-sitter
```

**预期结果：**
- 优雅处理服务不可用
- 合理的错误提示
- 缓存功能正常工作
- 服务恢复后功能正常

### 场景 2: 网络异常测试

```bash
# 模拟网络中断
sudo iptables -A OUTPUT -p tcp --dport 11434 -j DROP

# 测试网络异常处理
gitai review --tree-sitter

# 恢复网络
sudo iptables -D OUTPUT -p tcp --dport 11434 -j DROP

# 测试恢复后的功能
gitai review --tree-sitter
```

### 场景 3: 配置错误测试

```bash
# 创建错误配置
cat > broken_config.toml << 'EOF'
[ai]
api_url = "http://invalid-url:11434/v1/chat/completions"
model_name = "invalid-model"
EOF

# 测试错误配置处理
gitai --config broken_config.toml review

# 测试配置验证
gitai config --validate --config broken_config.toml
```

## 📊 测试结果验证

### 测试检查清单

```bash
# 创建测试验证脚本
cat > test_verification.sh << 'EOF'
#!/bin/bash

echo "=== GitAI 测试结果验证 ==="

# 检查测试结果文件
check_file() {
    if [ -f "$1" ]; then
        echo "✅ $1 存在"
        if [ $(jq . "$1" 2>/dev/null || echo "invalid") != "invalid" ]; then
            echo "✅ $1 是有效的 JSON"
        else
            echo "⚠️  $1 不是 JSON 格式"
        fi
    else
        echo "❌ $1 不存在"
    fi
}

# 验证各类测试结果
echo "1. 验证评审结果..."
check_file "test-review.md"
check_file "devops-review.json"
check_file "workflow-review.md"
check_file "ci-quality.json"

echo "2. 验证扫描结果..."
check_file "security-report.json"
check_file "performance-report.json"
check_file "workflow-security.json"
check_file "ci-security.json"

echo "3. 验证并发测试结果..."
for i in {1..10}; do
    check_file "concurrent-test-$i.json"
done

echo "4. 验证大文件处理结果..."
check_file "large-file-review.json"
check_file "large-file-scan.json"

echo "验证完成"
EOF

chmod +x test_verification.sh

# 执行验证
./test_verification.sh
```

## 🎯 测试报告生成

```bash
# 生成综合测试报告
cat > test_report.md << 'EOF'
# GitAI 功能测试报告

## 测试环境
- 测试时间: $(date)
- GitAI 版本: $(gitai --version)
- Rust 版本: $(rustc --version)
- AI 服务状态: $(curl -s http://localhost:11434/api/tags | jq -r '.models[0].name // "不可用"')

## 测试结果汇总

### ✅ 通过的测试
- [x] MCP 服务启动和连接
- [x] 基础代码评审功能
- [x] 智能提交功能
- [x] 安全扫描功能
- [x] 性能扫描功能
- [x] 多语言支持
- [x] DevOps 集成
- [x] 并发处理
- [x] 大文件处理

### ⚠️ 需要注意的问题
- [ ] 某些复杂代码结构的分析精度
- [ ] 超大文件的内存使用优化
- [ ] 网络异常的恢复时间

### 📊 性能指标
- 平均评审时间: TBD
- 并发处理能力: TBD
- 内存使用峰值: TBD
- 错误率: TBD

## 建议和改进

1. **性能优化**
   - 优化大文件处理算法
   - 改进并发处理机制
   - 减少内存使用

2. **功能增强**
   - 增加更多编程语言支持
   - 改进 AI 分析精度
   - 添加更多扫描规则

3. **用户体验**
   - 改进错误提示信息
   - 优化配置文件格式
   - 增加可视化报告
EOF

echo "测试报告已生成: test_report.md"
```

## 🔚 测试清理

```bash
# 清理测试文件
echo "清理测试文件..."
rm -f ~/gitai-test-projects/simple-test-project/*.json
rm -f ~/gitai-test-projects/simple-test-project/*.md
rm -f ~/gitai-test-projects/simple-test-project/test_*
rm -f ~/gitai-test-projects/simple-test-project/broken_config.toml
rm -f ~/gitai-test-projects/simple-test-project/ci_test.sh
rm -f ~/gitai-test-projects/simple-test-project/concurrency_test.sh
rm -f ~/gitai-test-projects/simple-test-project/test_verification.sh

# 清理测试结果
rm -rf ~/gitai-test-results
rm -rf ~/gitai-test-scan-results

echo "测试清理完成"
```

---

**🎉 恭喜！您已经完成了 GitAI 的全面功能测试。**

这些测试场景涵盖了 GitAI 的所有核心功能，包括：
- ✅ MCP 服务的完整功能测试
- ✅ 代码评审的多场景测试
- ✅ 智能提交的各种模式测试
- ✅ 代码扫描的安全和性能测试
- ✅ 集成工作流测试
- ✅ 性能和故障恢复测试

通过这些测试，您可以确保 GitAI 在您的环境中正常工作，并了解各项功能的性能表现。
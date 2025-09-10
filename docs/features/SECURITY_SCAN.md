# 安全扫描 (Security Scan)

## 功能概述

GitAI 的安全扫描功能通过集成 OpenGrep（Semgrep）等工具，对代码进行静态安全分析（SAST），自动检测潜在的安全漏洞和代码质量问题。

## 核心特性

### 1. 多语言支持
- 支持 30+ 种编程语言
- 自动检测项目语言
- 语言特定的规则集
- 跨语言漏洞检测

### 2. 规则管理
- 内置数千条安全规则
- 支持自定义规则
- 自动规则更新
- 规则分类和优先级

### 3. 智能分析
- 污点分析（Taint Analysis）
- 数据流分析
- 控制流分析
- 模式匹配

### 4. 集成能力
- 与代码评审集成
- CI/CD 管道集成
- IDE 插件支持
- API 接口

## 使用方法

### 基本用法

```bash
# 扫描当前目录
gitai scan

# 扫描指定路径
gitai scan /path/to/project

# 指定语言规则
gitai scan --lang java

# 自动安装 OpenGrep
gitai scan --auto-install

# 更新规则库
gitai scan --update-rules

# 设置超时时间
gitai scan --timeout 600

# 输出 JSON 格式
gitai scan --format json

# 不保存历史记录
gitai scan --no-history
```

### 高级用法

```bash
# 使用多个语言规则
gitai scan --lang "java,python,javascript"

# 并行扫描提升性能
gitai scan --jobs 8

# 只扫描特定严重级别
gitai scan --severity high,critical

# 排除特定路径
gitai scan --exclude "*/test/*,*/vendor/*"

# 使用自定义规则
gitai scan --rules /path/to/custom/rules
```

## 配置选项

在 `~/.config/gitai/config.toml` 中配置：

```toml
[scan]
# 默认扫描路径
default_path = "."

# 超时时间（秒）
timeout = 300

# 并行任务数
jobs = 4

# 默认输出格式
default_format = "text"  # text, json, sarif

# 自动保存历史
save_history = true

# 规则更新检查间隔（天）
rule_update_interval = 7

[scan.rules]
# 规则源
sources = [
    "https://github.com/returntocorp/semgrep-rules",
    "https://your-org.com/custom-rules"
]

# 启用的规则集
enabled_rulesets = [
    "security",
    "best-practices",
    "performance"
]

# 排除的规则
exclude_rules = [
    "generic.secrets.security.detected-private-key"
]
```

## 扫描规则分类

### 安全漏洞
- **注入攻击**：SQL、NoSQL、LDAP、XPath 注入
- **XSS**：跨站脚本攻击
- **CSRF**：跨站请求伪造
- **认证授权**：弱密码、硬编码凭证
- **加密问题**：弱加密算法、不安全的随机数
- **路径遍历**：目录遍历、文件包含
- **反序列化**：不安全的反序列化

### 代码质量
- **错误处理**：未捕获的异常、错误泄露
- **资源管理**：内存泄漏、未关闭的资源
- **并发问题**：竞态条件、死锁
- **性能问题**：N+1 查询、低效算法

### 合规性
- **隐私保护**：GDPR、CCPA 合规
- **许可证**：开源许可证合规
- **行业标准**：OWASP、CWE、SANS

## 工作流程

### 1. 环境准备
```
检查 OpenGrep → 自动安装（可选）→ 更新规则（可选）→ 语言检测
```

### 2. 规则选择
```
语言规则匹配 → 自定义规则加载 → 规则优先级排序 → 规则去重
```

### 3. 扫描执行
```
文件遍历 → 并行分析 → 模式匹配 → 结果聚合
```

### 4. 结果处理
```
严重级别分类 → 去重和分组 → 格式化输出 → 历史记录保存
```

## 示例场景

### 场景 1：Java 项目安全扫描

```bash
gitai scan --lang java --severity high,critical

# 输出示例：
🔍 正在扫描 Java 项目...
📋 使用规则集：java-security (487 条规则)
⚡ 并行执行（4 个任务）...

发现 3 个安全问题：

🔴 高危：SQL 注入漏洞
   文件：src/main/java/UserDao.java:42
   代码：String query = "SELECT * FROM users WHERE id = " + userId;
   建议：使用参数化查询防止 SQL 注入

🔴 高危：硬编码密码
   文件：src/main/java/Config.java:15
   代码：private static final String PASSWORD = "admin123";
   建议：使用环境变量或密钥管理系统

🟡 中危：不安全的随机数生成
   文件：src/main/java/TokenGenerator.java:28
   代码：Random random = new Random();
   建议：使用 SecureRandom 生成安全随机数

扫描完成！发现 2 个高危，1 个中危问题
```

### 场景 2：多语言项目扫描

```bash
gitai scan --auto-install --update-rules

# 输出示例：
🔧 正在安装 OpenGrep...
✅ OpenGrep 安装成功！
🔄 正在更新规则库...
✅ 规则库已更新（新增 127 条规则）

🔍 自动检测到项目语言：
   - Python (45%)
   - JavaScript (30%)
   - Go (25%)

⚡ 开始多语言扫描...

Python 扫描结果：
   ✅ 未发现安全问题

JavaScript 扫描结果：
   🟡 2 个中危问题（XSS 风险）

Go 扫描结果：
   🟡 1 个中危问题（错误处理）

总计：3 个问题需要关注
```

### 场景 3：CI/CD 集成

```yaml
# .github/workflows/security.yml
name: Security Scan

on: [push, pull_request]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Install GitAI
        run: cargo install gitai
      
      - name: Run Security Scan
        run: |
          gitai scan --format json --output scan-results.json
          
      - name: Upload Results
        uses: actions/upload-artifact@v2
        with:
          name: security-scan
          path: scan-results.json
```

## 自定义规则

### 规则格式（YAML）

```yaml
rules:
  - id: custom-sql-injection
    pattern: |
      $QUERY = "..." + $USER_INPUT
    message: 潜在的 SQL 注入漏洞
    languages: [java, csharp]
    severity: ERROR
    metadata:
      category: security
      cwe: CWE-89
      owasp: A03:2021
    fix: |
      使用参数化查询：
      PreparedStatement stmt = connection.prepareStatement("SELECT * FROM users WHERE id = ?");
      stmt.setString(1, userId);
```

### 规则测试

```bash
# 测试自定义规则
gitai scan --rules ./custom-rules.yaml --test

# 验证规则语法
gitai scan --rules ./custom-rules.yaml --validate
```

## 性能优化

### 1. 并行扫描
```bash
# 使用 8 个并行任务
gitai scan --jobs 8
```

### 2. 增量扫描
```bash
# 只扫描变更的文件
gitai scan --incremental
```

### 3. 缓存优化
- 规则缓存：避免重复解析
- 结果缓存：跳过未变更文件
- AST 缓存：复用语法树

## 结果分析

### 严重级别
- 🔴 **Critical**：立即修复，生产环境风险
- 🔴 **High**：高优先级，潜在严重影响
- 🟡 **Medium**：中等优先级，应当修复
- 🔵 **Low**：低优先级，建议改进
- ⚪ **Info**：信息提示，最佳实践

### 误报处理

```python
# nosemgrep: rule-id
vulnerable_code()  # 这行会被忽略

# nosemgrep
entire_function()  # 整个函数被忽略
```

### 批量忽略

```yaml
# .semgrepignore
# 测试文件
**/test/**
**/tests/**
**/*_test.go

# 第三方库
vendor/
node_modules/

# 生成的代码
**/*_generated.go
**/*.pb.go
```

## MCP 映射

- 对应工具：scan 服务的 `execute_scan`
- 参数：path（必填）、tool（opengrep/security，可选，默认 opengrep）、timeout、lang
- 注意：MCP 服务不会自动更新规则。若扫描结果显示未加载有效规则，请配置规则来源或先通过 CLI 更新规则。

规则配置建议：
- 通过环境变量提供规则包（推荐）：
-  - export GITAI_RULES_URL="https://github.com/opengrep/opengrep-rules/archive/refs/heads/main.tar.gz"
  - 然后重试 MCP 扫描
- 或使用 CLI 一次性更新：
  - gitai scan --update-rules --auto-install
- 或将规则手动放置到目录：
  - ~/.cache/gitai/rules/opengrep-rules-main/java（及其他语言子目录）

示例请求：
```json
{
  "name": "execute_scan",
  "arguments": {
    "path": ".",
    "tool": "opengrep",
    "timeout": 300
  }
}
```

## 与其他功能集成

### 代码评审集成
```bash
# 在代码评审中包含安全扫描
gitai review --security-scan
```

### 提交前检查
```bash
# Git hook 集成
#!/bin/bash
gitai scan --severity high,critical || exit 1
```

### 度量跟踪
```bash
# 记录安全度量
gitai scan && gitai metrics record --type security
```

## 故障排除

### 问题：OpenGrep 未找到

**解决方案：**
```bash
# 自动安装
gitai scan --auto-install

# 手动安装
pip install semgrep
# 或
brew install semgrep
```

### 问题：扫描超时

**解决方案：**
1. 增加超时时间：`--timeout 1200`
2. 减少扫描范围：排除大文件或目录
3. 增加并行度：`--jobs 16`

### 问题：规则更新失败

**解决方案：**
1. 检查网络连接
2. 手动下载规则包
3. 使用镜像源
4. 配置代理

## 最佳实践

### 1. 定期扫描
- 每次提交前扫描
- 每日全量扫描
- 发布前深度扫描

### 2. 规则管理
- 根据项目定制规则
- 定期更新规则库
- 记录误报和例外

### 3. 团队协作
- 共享自定义规则
- 统一忽略配置
- 安全问题跟踪

## 未来展望

- [ ] 支持更多扫描工具（Snyk、Checkmarx）
- [ ] AI 辅助的漏洞修复建议
- [ ] 实时扫描（文件保存时）
- [ ] 依赖项漏洞扫描
- [ ] 容器镜像扫描

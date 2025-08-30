# GitAI 功能门控（Feature Flags）指南

GitAI 支持通过功能标志来定制构建，允许您根据需求选择包含或排除特定功能，从而优化二进制文件大小和编译时间。

## 目录

- [功能标志概览](#功能标志概览)
- [预定义功能集](#预定义功能集)
- [语言支持功能](#语言支持功能)
- [构建示例](#构建示例)
- [二进制文件大小对比](#二进制文件大小对比)
- [使用构建脚本](#使用构建脚本)
- [查看启用的功能](#查看启用的功能)

## 功能标志概览

| 功能标志 | 描述 | 默认启用 |
|---------|------|---------|
| `ai` | AI 功能（代码解释、智能提交等） | ✅ |
| `security` | 安全扫描功能 | ❌ |
| `mcp` | MCP 服务器支持 | ❌ |
| `metrics` | 代码质量度量功能 | ❌ |
| `devops` | DevOps 集成功能 | ❌ |
| `update-notifier` | 自动更新通知 | ❌ |

## 预定义功能集

| 功能集 | 包含功能 | 用途 |
|-------|---------|-----|
| `minimal` | 仅核心功能 | 最小化构建，基础 Git 操作 |
| `default` | `ai` + 常用语言支持 | 标准版本，推荐日常使用 |
| `full` | 所有功能 | 完整功能集，包含所有语言和功能 |

## 语言支持功能

Tree-sitter 语言解析器可以单独启用或禁用：

| 功能标志 | 支持语言 | 文件扩展名 |
|---------|---------|-----------|
| `tree-sitter-rust` | Rust | `.rs` |
| `tree-sitter-python` | Python | `.py`, `.pyi` |
| `tree-sitter-javascript` | JavaScript | `.js`, `.mjs`, `.cjs` |
| `tree-sitter-typescript` | TypeScript | `.ts`, `.tsx` |
| `tree-sitter-go` | Go | `.go` |
| `tree-sitter-java` | Java | `.java` |
| `tree-sitter-c` | C | `.c`, `.h` |
| `tree-sitter-cpp` | C++ | `.cpp`, `.cc`, `.cxx`, `.hpp` |
| `tree-sitter-all` | 所有语言 | - |

## 构建示例

### 最小构建（仅核心功能）

```bash
cargo build --release --no-default-features --features minimal
```

生成约 **10MB** 的二进制文件，仅包含基础功能。

### 仅 Rust 语言支持

```bash
cargo build --release --no-default-features --features tree-sitter-rust
```

生成约 **11MB** 的二进制文件，仅支持 Rust 代码分析。

### Web 开发版本

```bash
cargo build --release --no-default-features --features "tree-sitter-javascript,tree-sitter-typescript"
```

适合前端开发者使用。

### AI 增强版

```bash
cargo build --release --no-default-features --features "ai,tree-sitter-rust,tree-sitter-python"
```

包含 AI 功能和常用语言支持。

### 完整功能集

```bash
cargo build --release --features full
```

生成约 **22MB** 的二进制文件，包含所有功能。

## 二进制文件大小对比

基于 macOS ARM64 平台的实测数据：

| 构建配置 | 文件大小 | 相对默认版本 |
|---------|---------|-------------|
| minimal | 10MB | -17% |
| rust-only | 11MB | -8% |
| default | 12MB | 基准 |
| full | 22MB | +83% |

## 使用构建脚本

我们提供了便捷的构建脚本来生成各种预配置版本：

### 构建所有变体

```bash
./scripts/build-variants.sh
```

该脚本会：
1. 清理旧的构建
2. 构建各种预定义变体
3. 生成大小对比报告
4. 创建安装脚本

构建完成后，所有变体将保存在 `dist/` 目录中。

### 安装特定变体

```bash
cd dist
./install.sh
```

按提示选择要安装的版本。

## 查看启用的功能

构建完成后，可以在本地查看当前二进制启用的功能列表：

```bash
gitai features --format table   # 表格
# 或者
gitai features --format json    # JSON
```

使用构建脚本生成的变体，脚本会将每个变体的功能列表保存到 dist/gitai-<variant>.features.txt，方便快速查看。

## 开发建议

### 为新项目选择合适的功能集

- **个人项目**：使用 `minimal` 或特定语言版本
- **团队项目**：使用 `default` 版本
- **企业环境**：使用 `full` 或 `security` + `devops` 组合
- **CI/CD 环境**：使用 `minimal` + 必需功能

### 条件编译最佳实践

在代码中使用功能标志：

```rust
#[cfg(feature = "ai")]
pub mod ai;

#[cfg(feature = "security")]
pub mod scan;
```

在 `Cargo.toml` 中定义功能：

```toml
[features]
default = ["ai", "tree-sitter-rust", "tree-sitter-python"]
minimal = []
full = ["ai", "security", "mcp", "metrics", "devops", "tree-sitter-all"]
```

## 性能影响

功能门控不仅影响二进制大小，还影响：

- **编译时间**：更少的功能 = 更快的编译
- **内存占用**：更少的功能 = 更低的内存使用
- **启动时间**：更小的二进制 = 更快的启动

## 故障排除

### 编译错误：未找到模块

如果遇到类似 `use of unresolved module` 的错误，确保：

1. 启用了相应的功能标志
2. 代码中有正确的条件编译标记

### 运行时错误：功能不可用

某些命令可能需要特定功能：

- `gitai scan` 需要 `security` 功能
- `gitai mcp` 需要 `mcp` 功能
- AI 相关功能需要 `ai` 功能

## 贡献指南

添加新功能时，请：

1. 在 `Cargo.toml` 中定义功能标志
2. 使用条件编译保护相关代码
3. 更新本文档
4. 测试各种功能组合的编译

## 相关文档

- [架构重构文档](./ARCHITECTURE_REFACTOR.md)
- [README](../README.md)
- [构建脚本](../scripts/build-variants.sh)

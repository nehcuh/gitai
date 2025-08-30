# GitAI Workspace 迁移指南

## 概述

本文档指导如何将 GitAI 从单一 crate 结构迁移到 Workspace 多 crate 结构。

## 新的项目结构

```
gitai/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── gitai-types/              # 共享类型定义
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── gitai-core/               # 核心功能
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs        # 从 src/config.rs 迁移
│   │       ├── context.rs       # 从 src/context.rs 迁移
│   │       ├── git.rs           # 从 src/git.rs 迁移
│   │       ├── ai.rs            # 从 src/ai.rs 迁移
│   │       ├── cache/           # 缓存功能模块
│   │       └── error.rs         # 错误处理
│   ├── gitai-analysis/           # 代码分析引擎
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── tree_sitter/     # 从 src/tree_sitter/ 迁移
│   │       ├── dependency_graph/ # 从 src/architectural_impact/dependency_graph.rs 迁移
│   │       ├── ast_comparison/  # 从 src/architectural_impact/ast_comparison.rs 迁移
│   │       └── impact/          # 影响分析
│   ├── gitai-security/           # 安全扫描
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── scan.rs          # 从 src/scan.rs 迁移
│   │       ├── opengrep.rs      # OpenGrep 集成
│   │       └── rules.rs         # 规则管理
│   ├── gitai-metrics/            # 度量和报告
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── collector.rs     # 从 src/metrics/ 迁移
│   │       ├── analyzer.rs      # 分析器
│   │       └── reporter.rs      # 报告生成
│   ├── gitai-cli/                # CLI 应用
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs          # 从 src/main.rs 迁移
│   │       ├── args.rs          # 从 src/args.rs 迁移
│   │       ├── commands/        # 命令实现
│   │       │   ├── init.rs      # 从 src/config_init.rs 迁移
│   │       │   ├── review.rs    # 从 src/review.rs 迁移
│   │       │   ├── commit.rs    # 从 src/commit.rs 迁移
│   │       │   └── scan.rs      # 扫描命令
│   │       └── output/          # 输出格式化
│   └── gitai-mcp/                # MCP 服务器
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs          # 从 src/bin/gitai-mcp.rs 迁移
│           ├── lib.rs
│           ├── server.rs        # 从 src/mcp/mod.rs 迁移
│           └── services/        # 从 src/mcp/services/ 迁移
```

## 迁移步骤

### Phase 1: 准备工作（当前阶段）

1. ✅ 创建 Workspace 结构
2. ✅ 创建 gitai-types crate
3. ✅ 创建各个 crate 的基础配置

### Phase 2: 代码迁移

#### 2.1 迁移共享类型到 gitai-types

```bash
# 需要迁移的类型定义
- src/scan.rs -> Severity, Finding
- src/architectural_impact/mod.rs -> RiskLevel, BreakingChange, ImpactLevel
- src/project_insights.rs -> NodeType, DependencyType
- src/error.rs -> GitAIError
```

#### 2.2 迁移核心功能到 gitai-core

```bash
# 迁移文件
cp src/config.rs crates/gitai-core/src/
cp src/context.rs crates/gitai-core/src/
cp src/git.rs crates/gitai-core/src/
cp src/ai.rs crates/gitai-core/src/
cp src/devops.rs crates/gitai-core/src/
```

更新导入：
```rust
// 旧的导入
use crate::config::Config;

// 新的导入
use gitai_core::config::Config;
```

#### 2.3 迁移分析功能到 gitai-analysis

```bash
# 迁移 tree-sitter 模块
cp -r src/tree_sitter crates/gitai-analysis/src/

# 迁移依赖图和影响分析
cp src/architectural_impact/dependency_graph.rs crates/gitai-analysis/src/dependency_graph/
cp src/architectural_impact/ast_comparison.rs crates/gitai-analysis/src/ast_comparison/
cp src/architectural_impact/impact_propagation.rs crates/gitai-analysis/src/impact/
```

#### 2.4 迁移安全扫描到 gitai-security

```bash
# 迁移扫描模块
cp src/scan.rs crates/gitai-security/src/
cp src/security_insights.rs crates/gitai-security/src/
```

#### 2.5 迁移度量功能到 gitai-metrics

```bash
# 迁移度量模块
cp -r src/metrics/* crates/gitai-metrics/src/
```

#### 2.6 迁移 CLI 到 gitai-cli

```bash
# 迁移主程序和命令
cp src/main.rs crates/gitai-cli/src/
cp src/args.rs crates/gitai-cli/src/

# 迁移命令实现
mkdir -p crates/gitai-cli/src/commands
cp src/config_init.rs crates/gitai-cli/src/commands/init.rs
cp src/review.rs crates/gitai-cli/src/commands/review.rs
cp src/commit.rs crates/gitai-cli/src/commands/commit.rs
```

#### 2.7 迁移 MCP 服务器到 gitai-mcp

```bash
# 迁移 MCP 实现
cp src/bin/gitai-mcp.rs crates/gitai-mcp/src/main.rs
cp -r src/mcp/* crates/gitai-mcp/src/
```

### Phase 3: 更新依赖关系

#### 3.1 更新内部依赖

所有 crate 之间的依赖关系：
```toml
# gitai-core/Cargo.toml
[dependencies]
gitai-types = { path = "../gitai-types" }

# gitai-analysis/Cargo.toml
[dependencies]
gitai-types = { path = "../gitai-types" }
gitai-core = { path = "../gitai-core" }

# gitai-cli/Cargo.toml
[dependencies]
gitai-types = { path = "../gitai-types" }
gitai-core = { path = "../gitai-core" }
gitai-analysis = { path = "../gitai-analysis" }
gitai-security = { path = "../gitai-security" }
gitai-metrics = { path = "../gitai-metrics" }
```

#### 3.2 更新导入路径

```rust
// 旧的导入
use crate::scan::{ScanResult, Finding};
use crate::review::ReviewResult;

// 新的导入
use gitai_security::scan::{ScanResult, Finding};
use gitai_cli::commands::review::ReviewResult;
```

### Phase 4: 测试和验证

#### 4.1 编译测试

```bash
# 编译整个 workspace
cargo build --workspace

# 编译特定 crate
cargo build -p gitai-core
cargo build -p gitai-cli

# 编译时使用不同的 feature
cargo build -p gitai-analysis --features all-languages
cargo build -p gitai-cli --features minimal
```

#### 4.2 运行测试

```bash
# 运行所有测试
cargo test --workspace

# 运行特定 crate 的测试
cargo test -p gitai-core
cargo test -p gitai-types

# 运行特定 feature 的测试
cargo test -p gitai-analysis --features all-languages
```

#### 4.3 检查二进制大小

```bash
# 完整功能版本
cargo build --release
ls -lh target/release/gitai

# 最小功能版本
cargo build --release -p gitai-cli --features minimal
ls -lh target/release/gitai
```

### Phase 5: 更新配置文件

#### 5.1 更新根 Cargo.toml

```toml
[workspace]
members = [
    "crates/gitai-types",
    "crates/gitai-core",
    "crates/gitai-analysis",
    "crates/gitai-security",
    "crates/gitai-metrics",
    "crates/gitai-cli",
    "crates/gitai-mcp",
]
resolver = "2"

# 删除旧的 [package] 和 [[bin]] 部分
```

#### 5.2 更新 CI/CD 配置

更新 GitHub Actions 或其他 CI 配置以支持 Workspace：

```yaml
# .github/workflows/ci.yml
- name: Build workspace
  run: cargo build --workspace --all-features

- name: Test workspace
  run: cargo test --workspace --all-features

- name: Build release binaries
  run: |
    cargo build --release -p gitai-cli
    cargo build --release -p gitai-mcp
```

## Feature Gates 使用指南

### 编译不同版本

```bash
# 最小版本（仅基础功能）
cargo build --release -p gitai-cli --no-default-features --features minimal

# 仅支持特定语言
cargo build --release -p gitai-analysis --no-default-features --features "rust,python"

# 完整版本
cargo build --release --workspace --all-features
```

### 功能组合示例

```toml
# 用户自定义配置示例
[dependencies.gitai-cli]
path = "crates/gitai-cli"
default-features = false
features = ["security", "metrics"]  # 不包含 full-analysis

[dependencies.gitai-analysis]
path = "crates/gitai-analysis"
default-features = false
features = ["rust", "javascript"]  # 仅支持 Rust 和 JavaScript
```

## 性能优化建议

1. **并行编译**: Workspace 支持并行编译多个 crate
2. **增量编译**: 只重新编译修改的 crate
3. **Feature 优化**: 根据需求选择功能，减少编译时间
4. **缓存共享**: Workspace 中的 crate 共享依赖缓存

## 故障排除

### 常见问题

**Q: 编译错误 "unresolved import"**
A: 检查是否正确更新了导入路径，从 `crate::` 改为对应的 crate 名称

**Q: 找不到类型定义**
A: 确保在 Cargo.toml 中添加了 `gitai-types` 依赖

**Q: Feature 不生效**
A: 检查是否在正确的 crate 中启用了 feature

**Q: 二进制文件变大**
A: 使用 `--no-default-features` 并手动选择需要的 feature

## 回滚计划

如果迁移出现严重问题：

1. 保留原始 `src/` 目录的备份
2. 恢复原始 `Cargo.toml`
3. 删除 `crates/` 目录
4. 重新编译项目

## 验证清单

- [ ] 所有 crate 都能独立编译
- [ ] 所有测试通过
- [ ] CLI 功能正常
- [ ] MCP 服务器正常启动
- [ ] Feature gates 正确工作
- [ ] 二进制大小符合预期
- [ ] 编译时间有所改善
- [ ] 文档已更新

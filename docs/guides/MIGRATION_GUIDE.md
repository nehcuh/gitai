# GitAI 重构迁移指南

## 概述

本指南详细说明了如何从当前的 GitAI 架构迁移到新的重构版本。重构主要涉及模块重组、翻译系统改进、以及配置管理优化。

## 迁移前准备

### 1. 备份当前代码

```bash
# 创建备份分支
git checkout -b backup-before-refactor

# 推送备份
git push origin backup-before-refactor

# 切换到重构分支
git checkout feat/ast-grep-integration
```

### 2. 评估当前配置

```bash
# 检查当前配置
ls ~/.config/gitai/
cat ~/.config/gitai/config.toml
```

## 分阶段迁移计划

### Phase 1: 基础架构迁移

#### 1.1 模块重组

**当前结构**:
```
src/
├── handlers/
├── ast_grep_analyzer/
├── clients/
├── types/
└── utils/
```

**新结构**:
```
src/
├── cli/
├── core/
├── analysis/
├── translation/
├── devops/
├── config/
└── common/
```

**迁移步骤**:

1. 创建新的模块目录结构
2. 逐步迁移现有代码到新模块
3. 更新模块间的依赖关系

#### 1.2 翻译系统迁移

**当前位置**: `src/ast_grep_analyzer/translation/`
**新位置**: `src/translation/`

**迁移步骤**:

1. **创建新的翻译模块**
   ```bash
   mkdir -p src/translation
   ```

2. **迁移核心翻译功能**
   ```rust
   // 从 ast_grep_analyzer/translation/ 迁移到 translation/
   // 主要文件:
   // - translator.rs
   // - cache_manager.rs
   // - manager.rs
   ```

3. **集成AI提示词**
   ```rust
   // 新的翻译器将使用 assets/translator.md
   pub struct PromptManager {
       base_prompt: String,
   }
   
   impl PromptManager {
       pub fn load_from_file(path: &Path) -> Result<Self, PromptError> {
           let content = std::fs::read_to_string(path)?;
           Ok(Self { base_prompt: content })
       }
   }
   ```

#### 1.3 配置系统迁移

**配置文件更新**:

1. **备份现有配置**
   ```bash
   cp ~/.config/gitai/config.toml ~/.config/gitai/config.toml.backup
   ```

2. **使用新配置格式**
   ```toml
   # 新配置格式 (参考 config.refactored.example.toml)
   [ai]
   api_url = "http://localhost:1234/v1/chat/completions"
   model_name = "deepseek-r1-0528-qwen3-8b-awq"
   
   [translation]
   enabled = true
   use_ai = true
   default_language = "auto"
   prompt_file = "assets/translator.md"
   
   [ast_grep]
   enabled = true
   analysis_depth = "medium"
   ```

### Phase 2: 核心功能迁移

#### 2.1 CLI 模块迁移

**当前**: `main.rs` 中的参数解析
**新架构**: `src/cli/` 模块

**迁移步骤**:

1. **创建 CLI 模块**
   ```rust
   // src/cli/mod.rs
   pub mod parser;
   pub mod commands;
   pub mod help;
   ```

2. **迁移参数解析逻辑**
   ```rust
   // src/cli/parser.rs
   pub fn parse_args() -> Result<AppArgs, ParseError> {
       // 从 main.rs 迁移参数解析逻辑
   }
   ```

3. **简化 main.rs**
   ```rust
   // 新的 main.rs 将更加简洁
   #[tokio::main]
   async fn main() -> Result<(), AppError> {
       let args = cli::parse_args()?;
       let config = config::load()?;
       app::run(args, config).await
   }
   ```

#### 2.2 核心业务逻辑迁移

**Git 操作迁移**:
```rust
// 从 handlers/git.rs 迁移到 core/git/
// 保持接口兼容性
```

**AI 集成迁移**:
```rust
// 从 handlers/ai.rs 迁移到 core/ai/
// 统一 AI 客户端接口
```

#### 2.3 AST-Grep 分析迁移

**当前**: `src/ast_grep_analyzer/`
**新位置**: `src/analysis/ast_grep/`

**迁移步骤**:

1. **保持核心分析逻辑**
   ```rust
   // analysis/ast_grep/analyzer.rs
   // 保持现有的分析功能
   ```

2. **分离翻译依赖**
   ```rust
   // 移除对 translation 模块的直接依赖
   // 通过依赖注入使用翻译服务
   ```

### Phase 3: 测试和验证

#### 3.1 测试迁移

**现有测试**:
```
tests/
├── integration_commit_test.rs
├── translation_integration.rs
└── commit_untracked.rs
```

**迁移步骤**:

1. **更新测试导入**
   ```rust
   // 更新模块导入路径
   use gitai::translation::Translator;
   use gitai::core::git::GitOperations;
   ```

2. **添加新的集成测试**
   ```rust
   // tests/refactored_integration.rs
   // 测试新架构的集成功能
   ```

#### 3.2 功能验证

**验证清单**:

- [ ] CLI 命令正常工作
- [ ] 配置文件正确加载
- [ ] 翻译功能正常
- [ ] AI 集成正常
- [ ] AST-Grep 分析正常
- [ ] DevOps 集成正常

**验证脚本**:
```bash
#!/bin/bash
echo "开始功能验证..."

# 测试基本命令
cargo run -- --help
cargo run -- review
cargo run -- commit
cargo run -- scan src/

# 测试翻译功能
cargo run -- --lang=zh review
cargo run -- --lang=en commit

# 运行测试套件
cargo test

echo "验证完成！"
```

## 配置迁移详细步骤

### 1. 自动迁移脚本

```rust
// 创建配置迁移工具
pub fn migrate_config() -> Result<(), MigrationError> {
    let old_config = load_old_config()?;
    let new_config = convert_to_new_format(old_config)?;
    save_new_config(new_config)?;
    backup_old_config()?;
    Ok(())
}
```

### 2. 手动迁移步骤

1. **tree_sitter 配置转换**
   ```toml
   # 旧配置
   [tree_sitter]
   enabled = false
   
   # 新配置
   [ast_grep]
   enabled = true
   ```

2. **翻译配置增强**
   ```toml
   # 新增翻译配置
   [translation]
   enabled = true
   use_ai = true
   default_language = "auto"
   prompt_file = "assets/translator.md"
   ```

## 兼容性保证

### 1. 向后兼容性

- CLI 接口保持不变
- 主要功能行为一致
- 配置文件自动迁移

### 2. 渐进式迁移

```rust
// 支持旧配置格式
impl AppConfig {
    pub fn load_with_migration() -> Result<Self, ConfigError> {
        match Self::load() {
            Ok(config) => Ok(config),
            Err(_) => {
                // 尝试加载旧格式并迁移
                let old_config = OldConfig::load()?;
                let new_config = old_config.migrate()?;
                new_config.save()?;
                Ok(new_config)
            }
        }
    }
}
```

## 性能优化

### 1. 模块加载优化

```rust
// 延迟加载非关键模块
use std::sync::LazyLock;

static TRANSLATOR: LazyLock<Translator> = LazyLock::new(|| {
    Translator::new().expect("Failed to initialize translator")
});
```

### 2. 缓存策略改进

```rust
// 统一缓存管理
pub struct CacheManager {
    translation_cache: TranslationCache,
    analysis_cache: AnalysisCache,
}
```

## 故障排除

### 常见问题

1. **模块导入错误**
   ```bash
   # 解决方案: 更新 Cargo.toml 和 mod.rs
   ```

2. **配置加载失败**
   ```bash
   # 解决方案: 检查配置文件格式
   ```

3. **翻译功能异常**
   ```bash
   # 解决方案: 检查 AI 服务配置
   ```

### 调试技巧

```bash
# 启用详细日志
RUST_LOG=debug cargo run -- review

# 检查配置加载
RUST_LOG=gitai::config=debug cargo run -- --help
```

## 迁移后验证

### 1. 功能测试

```bash
# 完整测试套件
cargo test --all

# 集成测试
cargo test --test integration_commit_test
```

### 2. 性能基准测试

```bash
# 运行性能测试
cargo bench

# 比较迁移前后性能
```

## 回滚计划

如果迁移过程中出现问题，可以使用以下回滚策略：

```bash
# 回滚到迁移前状态
git checkout backup-before-refactor

# 恢复配置文件
cp ~/.config/gitai/config.toml.backup ~/.config/gitai/config.toml
```

## 总结

这个迁移指南提供了从当前架构到新架构的完整迁移路径。通过分阶段实施，可以最大程度地降低迁移风险，确保功能的连续性和稳定性。

重构后的架构将提供更好的模块化、可维护性和扩展性，为未来的功能开发奠定坚实基础。
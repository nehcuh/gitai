# AST-grep Core 0.38.6 API 研究报告

## 研究概述

本报告详细研究了 `ast-grep-core` 0.38.6 版本的 API 使用方式，包括如何正确创建语言实例、解析源代码为 AST、应用规则进行模式匹配以及 `SerializableRuleCore` 和 `RuleConfig` 的正确使用方式。

## 主要发现

### 1. 项目依赖配置

项目已正确配置了 ast-grep 相关依赖：
```toml
ast-grep-config = "0.38.6"
ast-grep-core = "0.38.6"
```

### 2. Language 创建方式

通过研究发现，`Language` 在 ast-grep-core 中是一个 trait，而不是具体的类型。正确的使用方式涉及：

- `Language` trait 定义了与 tree-sitter 语言交互的接口
- 实际的语言实现通过 `TSLanguage` 结构体提供
- 语言实例通过特定的构造函数创建，如 `language_rust()`, `language_javascript()` 等

### 3. 源代码解析 API

AST 解析主要通过以下方式：
- 使用 `Language::ast_grep(source_code)` 方法解析源代码
- 返回 `Doc` 类型，提供 AST 操作接口
- 支持多种编程语言的解析

### 4. 规则配置和使用

#### SerializableRuleCore
- 用于序列化和反序列化规则配置
- 通过 YAML 格式定义规则
- 包含规则的核心逻辑定义

#### RuleConfig
- 需要泛型参数 `<L: Language>`
- 通过 `RuleConfig::try_from()` 方法创建
- 用于实际的模式匹配操作

### 5. 规则 YAML 格式

标准的 ast-grep 规则格式：
```yaml
id: rule-name
language: rust
rule:
  pattern: println!($ARGS)
message: "Found println macro"
severity: info
```

支持复杂规则：
```yaml
id: complex-rule
language: rust
rule:
  any:
    - pattern: $FUNC($ARGS)
    - pattern: $VAR = $VALUE
utils:
  helper_rule:
    pattern: "fn $NAME() { $BODY }"
    has:
      pattern: "#[test]"
message: "Found function call or assignment"
severity: warning
```

## API 使用示例

### 1. 基本的规则解析

```rust
use ast_grep_config::SerializableRuleCore;
use serde_yaml;

let rule_yaml = r#"
id: test-rule
language: rust
rule:
  pattern: println!($MSG)
message: "Found println macro"
severity: info
"#;

let rule_core: SerializableRuleCore = serde_yaml::from_str(rule_yaml)?;
```

### 2. 语言支持枚举

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SupportedLanguage {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Java,
    C,
    Cpp,
    Go,
    Html,
    Css,
}

impl SupportedLanguage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
            // ... 其他语言
        }
    }
}
```

### 3. AST-grep 引擎包装器

```rust
pub struct AstGrepEngine {
    rules: HashMap<String, AstGrepRule>,
}

impl AstGrepEngine {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule_yaml: &str) -> Result<String, AppError> {
        let rule_core: SerializableRuleCore = serde_yaml::from_str(rule_yaml)?;
        // 处理规则...
    }

    pub fn find_matches_simple(
        &self,
        source_code: &str,
        language: SupportedLanguage,
        file_path: &str,
    ) -> Result<Vec<AstMatch>, AppError> {
        // 简化的模式匹配实现
    }
}
```

## 当前实现状态

### 已实现功能

1. **规则解析和管理**
   - YAML 规则配置解析
   - 规则元数据提取（id、language、severity、message）
   - 规则存储和管理

2. **语言支持**
   - 支持 10+ 种编程语言
   - 语言标识符转换
   - 文件扩展名到语言的映射

3. **简化的模式匹配**
   - 基于文本搜索的回退实现
   - 支持常见模式检测（println!、console.log、print等）

4. **规则模板**
   - JavaScript console.log 检测
   - Rust println! 检测
   - Python print 检测
   - TODO 注释检测

### 待完善功能

1. **真正的 AST 模式匹配**
   - 目前使用简化的文本匹配作为回退
   - 需要集成完整的 ast-grep-core Language API

2. **复杂规则支持**
   - utils 部分的处理
   - 组合规则（any、all、not）
   - 关系规则（inside、has、follows、precedes）

3. **高级功能**
   - 代码变换和重写
   - Meta 变量提取
   - 上下文感知匹配

## 集成建议

### 短期方案
使用当前的简化实现，支持基本的模式检测，可以满足基本的代码扫描需求。

### 长期方案
1. 深入研究 ast-grep-core 的 Language trait 实现
2. 集成 ast-grep-language crate 获取预定义语言支持
3. 实现完整的 AST 模式匹配功能
4. 支持复杂规则和代码变换

## 测试验证

创建的集成模块包含完整的测试套件：
- 语言转换测试
- 规则创建和解析测试
- 简化模式匹配测试
- 规则模板验证测试

## 结论

通过深入研究 ast-grep-core 0.38.6 API，我们了解了：

1. **正确的语言创建方式**：通过 Language trait 和 TSLanguage 结构体
2. **源代码解析**：使用 `Language::ast_grep()` 方法
3. **规则应用**：通过 RuleConfig 和 SerializableRuleCore 配合使用
4. **YAML 配置格式**：标准的 ast-grep 规则定义结构

当前实现提供了一个可工作的基础，可以逐步增强以支持更高级的 AST 模式匹配功能。集成代码位于 `/src/ast_grep_integration.rs`，提供了完整的 API 包装和使用示例。

---

**文件位置**：
- 集成代码：`/src/ast_grep_integration.rs`
- 依赖配置：`Cargo.toml`
- 现有扫描器：`/src/scanner.rs`（包含注释掉的 ast-grep 集成代码）

**下一步行动**：
1. 修复项目中的配置结构问题
2. 逐步替换简化实现为完整的 AST 模式匹配
3. 扩展规则库和模板
4. 完善错误处理和性能优化
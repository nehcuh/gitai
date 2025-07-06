# AST-Grep 集成任务交接文档

## 1. 最终目标 (Overall Goal)

集成 `ast-grep` 工具，为 `gitai` 提供强大的本地代码扫描能力。

核心需求如下：
- **集成 `ast-grep` 库**: 以便在项目内部调用其扫描接口。
- **规则管理**:
    - 能够自动从远程仓库 (`https://github.com/coderabbitai/ast-grep-essentials`) 下载和更新扫描规则。
    - 规则应被缓存在本地 (`~/.config/gitai/scan-rules` 或用户自定义路径)。
    - 缓存应有过期机制（TTL），并支持强制更新。
- **扫描机制**:
    - 支持增量扫描（默认，基于 `git diff`）和全量扫描。
    - 用户可以指定扫描路径，若不指定则扫描整个项目。
    - 尊重 `.gitignore` 配置。
- **远程扫描支持 (框架)**:
    - 支持将代码发送到远程扫描系统进行分析的配置选项（作为扩展点，初期无需具体实现）。
- **结果保存**:
    - 扫描结果应被保存到 `~/.gitai/scan-results` 目录。
    - 结果文件应以 `/<仓库名>/<commit_id>.json` 的格式进行组织。

## 2. 已完成的任务 (Completed Tasks)

当前工作在一个独立��分支 `feature/ast-grep-integration` 上。

**第一阶段：基础架构与配置**
- ✅ **依赖集成**: 已使用 `cargo search` 确认并添加了最新版本的 `ast-grep-core` (0.38.6), `ast-grep-config` (0.38.6), 和 `git2` (0.20.2) 到 `Cargo.toml`。
- ✅ **配置模型**: 已在 `src/config.rs` 中定义了新的配置结构体 `ScanConfig`, `RuleManagerConfig`, `RemoteScanConfig` 及其默认值。
- ✅ **CLI 骨架**:
    - 在 `src/types/git.rs` 中定义了 `ScanArgs` 结构体，用于解析 `scan` 命令的参数。
    - 在 `src/main.rs` 中添加了 `scan` 命令的入口逻辑。
    - 在 `src/utils.rs` 中添加了 `construct_scan_args` 函数。
- ✅ **处理器骨架**: 已创建 `src/handlers/scan.rs` 文件，并定义了 `handle_scan` 函数的骨架。

**第二阶段：规则管理器 (Rule Manager)**
- ✅ **模块创建**: 已创建 `src/rule_manager/mod.rs` 文件，并定义了 `RuleManager` 结构体。
- ✅ **缓存逻辑**: `RuleManager::new` 方法能够解析缓存路径并按需创建目录。
- ✅ **网络逻辑**: `RuleManager` 中已实现 `download_rules` 异步方法，用于从 GitHub API 获取并下载所有 `.yml` 规则文件。
- ✅ **主逻辑**: `RuleManager` 中已实现 `get_rule_paths` 方法，该方法能够处理缓存有效期（TTL）和强制更新逻辑，并返回可��的本地规则文件路径列表。

## 3. 未完成的任务 (Pending Tasks)

- **本地扫描引擎 (`LocalScanner`)**:
    - [ ] 创建 `src/scanner.rs` 模块。
    - [ ] 实现增量扫描逻辑（获取 `git diff` 并确定待扫描文件）。
    - [ ] 实现全量扫描逻辑（遍历项目文件并过滤掉 `.gitignore` 中的条目）。
    - [ ] 集成 `ast-grep-core`，编写调用其 API 对文件列表执行扫描的核心函数。

- **整合与输出**:
    - [ ] 在 `handle_scan` 函数中，调用 `LocalScanner` 执行扫描。
    - [ ] 格式化 `ast-grep` 的扫描结果。
    - [ ] 实现结果保存逻辑，按 `~/.gitai/scan-results/<仓库名>/<commit_id>.json` 格式保存。

- **远程扫描框架**:
    - [ ] 定义 `RemoteScanner` trait。
    - [ ] 提供一个占位的空实现。

## 4. 当前遇到的困难 (Current Blocker)

**核心问题**: 持续的模块可见性问题导致编译失败。

- **错误信息**: `error[E0432]: unresolved import 'crate::rule_manager'`
- **发生位置**: `src/handlers/scan.rs`
- **问题描述**: 尽管 `rule_manager` 模块已创建，并且在 `main.rs` 或 `lib.rs` 中进行了声明，但 `handlers` 模块下的 `scan.rs` 始终无法通过 `use crate::rule_manager::RuleManager;` 找到该模块。
- **已尝试的方案**:
    1. 在 `lib.rs` 中声明 `pub mod rule_manager;`，在 `scan.rs` 中使用 `use crate::rule_manager::RuleManager;`。
    2. 在 `main.rs` 中声明 `mod rule_manager;`，在 `scan.rs` 中使用 `use crate::rule_manager::RuleManager;`。
    3. 在 `scan.rs` 中尝试使用 `use gitai::rule_manager::RuleManager;`。

这些尝试均未解决编译错误。问题根源很可能在于当前项目混合了 `lib.rs` 和 `main.rs` 两个 crate 根，导致模块路径解析变得复杂。接手的开发者需要优先梳理项目的模块结构，确保 `handlers` 模块可以正确地访问到 `rule_manager` 模块。

---
希望这份文档能够帮助下一位开发者顺利接手。

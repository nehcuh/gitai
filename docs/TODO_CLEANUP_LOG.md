# TODO 清理日志

> 生成时间: 2025-01-08  
> 分支: feature/code-cleanup

## 清理的 TODO 项目

### 1. scanner.rs - ast-grep 集成相关

**位置**: `src/scanner.rs`  
**状态**: 暂时保留，需要后续修复

**TODO 项目**:
1. `Line 29-30`: Re-enable Parsed(SerializableRuleCore) when ast-grep integration is fixed
2. `Line 38-39`: Re-enable Parsed rule display in Debug implementation  
3. `Line 199`: Implement true ast-grep pattern matching
4. `Line 204`: Implement complex rule matching with utils support
5. `Line 209-214`: Re-enable parsed rule matching functionality
6. `Line 246`: Implement true ast-grep pattern matching in apply_pattern_matching
7. `Line 317`: Fix ast-grep API usage - currently disabled due to compilation errors

**原因**: ast-grep 库 API 变更导致编译错误，当前使用 regex 作为临时方案

**计划**: 在后续 Sprint 中专门修复 ast-grep 集成问题

### 2. 测试代码中的 TODO

**位置**: `src/handlers/commit.rs:727`  
**内容**: 测试字符串中包含 "// TODO: implement"  
**状态**: 保留，这是测试数据的一部分

### 3. ast_grep_integration.rs 中的 TODO 规则

**位置**: `src/ast_grep_integration.rs`  
**内容**: TODO comments detection rule  
**状态**: 保留，这是功能性的规则定义

## 清理策略

### 已清理的项目
1. ✅ 移除未使用变量 `count` in scanner.rs:297
2. ✅ 添加 `#[allow(dead_code)]` 标记到未使用字段
3. ✅ 修复 clippy 警告：
   - 空行警告 (ai.rs)
   - else-if 块简化 (commit.rs)
   - 嵌套 if 简化 (help.rs)
   - map_or 简化 (config.rs)

### 保留的项目及原因
1. **ast-grep 相关 TODO**: 需要专门的 Sprint 来修复 API 兼容性
2. **功能性 TODO**: 作为规则定义的一部分，不是代码问题
3. **测试数据 TODO**: 测试用例中的示例数据

### 下一步计划
1. 在后续的重构 Sprint 中专门处理 ast-grep 集成
2. 将注释掉的代码移动到专门的 feature branch
3. 建立 TODO 项目跟踪机制

## 编译状态
清理后的编译警告数量：
- 清理前：8+ 个警告
- 清理后：预期 <3 个警告（主要是 ast-grep 相关的已知问题）

---

*此日志记录了代码清理过程中的决策和保留的技术债务，为后续重构提供参考。*
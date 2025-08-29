# Phase 1: 基础架构影响分析 - 任务追踪

## 概述
实现最基本的变更前后 AST 对比和架构影响识别，让 AI 能够理解代码变更的架构影响。

## 任务清单

### 1.1 创建架构影响分析模块 (Day 1-2)

#### ✅ 任务 1.1.1: 创建新模块 (2小时)
- [x] 创建 `src/architectural_impact/mod.rs`
- [x] 定义核心数据结构
- [x] 添加必要的依赖项到 Cargo.toml

**具体步骤:**
```bash
# 创建模块目录
mkdir src/architectural_impact
touch src/architectural_impact/mod.rs
```

**预期输出:** 
- 基础的架构影响分析数据结构定义
- 模块导入配置完成

#### ✅ 任务 1.1.2: 实现 AST 对比引擎 (4小时)
- [x] 创建 `src/architectural_impact/ast_comparison.rs`
- [x] 实现基本的结构对比逻辑
- [x] 函数签名变化检测
- [x] 类/结构体变化检测

**具体步骤:**
- 比较两个 `StructuralSummary`
- 识别新增/删除/修改的函数
- 识别新增/删除/修改的类型

**预期输出:**
```rust
pub fn compare_structural_summaries(
    before: &StructuralSummary,
    after: &StructuralSummary,
) -> ArchitecturalImpactAnalysis
```

### 1.2 集成到现有评审流程 (Day 3)

#### ✅ 任务 1.2.1: 实现变更前代码状态获取 (4小时)
- [x] 创建 `get_ast_before_changes()` 函数
- [x] 从 git diff 中提取变更前的代码
- [x] 对变更前代码进行 Tree-sitter 分析

**技术挑战:**
- 如何从 git diff 重建变更前的完整代码状态？
- 如何高效地对变更前代码进行 AST 分析？

**解决方案:**
```rust
// 方案1: 使用 git show 获取变更前文件
fn get_file_content_before_changes(file_path: &str, commit: &str) -> Result<String>

// 方案2: 从 diff 中逆向重建代码
fn reconstruct_before_state(diff: &str) -> Result<HashMap<String, String>>
```

#### ✅ 任务 1.2.2: 修改 review.rs 集成架构影响分析 (3小时)
- [x] 在 `perform_structural_analysis` 前添加变更前分析
- [x] 调用架构影响分析
- [x] 集成架构影响到 AI 上下文

**修改点:**
```rust
// 在 review.rs 的 perform_structural_analysis 函数中
if review_config.tree_sitter {
    let before_ast = get_ast_before_changes(&diff, &language)?;
    let after_ast = analyze_current_state(&code_content, supported_lang)?;
    
    if let (Some(before), Some(after)) = (before_ast, after_ast) {
        let impact = analyze_architectural_impact(&before, &after);
        context.push_str(&impact.to_ai_context());
    }
}
```

### 1.3 实现最小可用版本 (Day 4-5)

#### ✅ 任务 1.3.1: 基础破坏性变更检测 (4小时)
- [x] 创建 `src/architectural_impact/breaking_changes.rs`
- [x] 函数签名变化检测
- [x] API 删除检测
- [x] 公共接口修改检测

**检测逻辑:**
```rust
pub enum BreakingChangeType {
    FunctionSignatureChanged,
    FunctionRemoved,
    FunctionAdded,
    VisibilityChanged,
    ParameterCountChanged,
    ReturnTypeChanged,
}

pub fn detect_breaking_changes(
    before: &[FunctionInfo],
    after: &[FunctionInfo],
) -> Vec<BreakingChange>
```

#### ✅ 任务 1.3.2: 简单风险评估 (3小时)
- [x] 创建 `src/architectural_impact/risk_assessment.rs`
- [x] 实现基于变更类型的风险评级
- [x] 影响范围估算

**风险评估规则:**
```rust
pub enum RiskLevel {
    Critical,  // API 删除、重大签名变更
    High,      // 参数数量变更、可见性变更
    Medium,    // 参数类型变更
    Low,       // 新增 API、注释变更
}
```

#### ✅ 任务 1.3.3: AI 友好输出格式 (2小时)
- [x] 创建 `src/architectural_impact/ai_context.rs`
- [x] 实现结构化文本输出
- [x] 确保 AI 可读性

**输出格式设计:**
```markdown
## 架构影响分析

### 🚨 高风险变更 (1个)
- **API 删除**: 函数 `deprecated_function()` 已被移除
  - 影响范围: 高 (可能被外部模块调用)
  - 建议: 检查调用方，提供迁移指南

### ⚠️ 中风险变更 (2个)
- **函数签名变更**: `parse_config(path: &str)` → `parse_config(path: &Path, options: ConfigOptions)`
  - 影响范围: 中等
  - 建议: 考虑保留向后兼容版本
```

### 1.4 测试和验证 (Day 6-7)

#### ⏳ 任务 1.4.1: 单元测试 (4小时)
- [ ] 测试 AST 对比逻辑
- [ ] 测试破坏性变更检测
- [ ] 测试风险评估算法

**测试用例设计:**
```rust
#[test]
fn test_function_signature_change_detection() {
    let before = create_test_summary_with_function("foo", vec!["String"], Some("i32"));
    let after = create_test_summary_with_function("foo", vec!["String", "bool"], Some("i32"));
    
    let analysis = compare_structural_summaries(&before, &after);
    assert_eq!(analysis.breaking_changes.len(), 1);
    assert!(matches!(analysis.breaking_changes[0].change_type, 
                    BreakingChangeType::ParameterCountChanged));
}
```

#### ⏳ 任务 1.4.2: 集成测试 (4小时)
- [ ] 在 GitAI 项目本身测试
- [ ] 验证 AI 理解架构影响信息
- [ ] 收集真实使用反馈

**测试方法:**
1. 在 GitAI 项目中做一个实际的代码变更
2. 运行 `gitai review --tree-sitter`
3. 观察架构影响分析是否准确
4. 检查 AI 是否在评审中提及架构影响

## 实现优先级

### 🚀 立即开始 (今天)
1. **任务 1.1.1** - 创建基础模块结构 (最简单)
2. **任务 1.1.2** - 实现基础 AST 对比 (核心功能)

### 📅 本周内完成
- 所有 Phase 1 任务
- 基本的架构影响分析可用
- 在真实项目中验证效果

## 技术决策

### 数据结构设计
```rust
// src/architectural_impact/mod.rs
pub struct ArchitecturalImpactAnalysis {
    pub breaking_changes: Vec<BreakingChange>,
    pub risk_level: RiskLevel,
    pub summary: String,
    pub ai_context: String,
}

pub struct BreakingChange {
    pub change_type: BreakingChangeType,
    pub component: String,
    pub description: String,
    pub impact_level: ImpactLevel,
    pub suggestions: Vec<String>,
}
```

### 集成策略
1. **非破坏性集成**: 不修改现有 `StructuralSummary` 结构
2. **可选功能**: 只在启用 Tree-sitter 时进行架构影响分析
3. **渐进增强**: 先实现基础功能，后续逐步完善

### 性能考虑
- **缓存**: 对相同的代码状态缓存 AST 分析结果
- **增量分析**: 只分析实际发生变更的文件
- **异步处理**: 变更前代码分析可以并行进行

## 成功标准

### MVP 完成标准
- [ ] 能检测到函数签名的基本变更
- [ ] 能生成 AI 可理解的架构影响描述
- [ ] 在 GitAI 项目中产生有意义的分析结果
- [ ] 分析时间控制在合理范围内（<10秒）

### 质量标准
- [ ] 单元测试覆盖率 > 80%
- [ ] 不破坏现有功能
- [ ] AI 能在评审中有效利用架构影响信息
- [ ] 误报率控制在可接受范围内

## 风险识别

### 技术风险
1. **获取变更前代码状态困难** 
   - 缓解: 使用 `git show HEAD~1:file` 命令
2. **AST 对比复杂度高**
   - 缓解: 先实现简单版本，逐步完善
3. **性能影响**
   - 缓解: 并行处理，智能缓存

### 实现风险
1. **AI 无法理解输出格式**
   - 缓解: 基于现有 prompts 设计输出格式
2. **误报率过高**
   - 缓解: 保守的风险评估策略

## 下一步行动

### 🎯 今天的目标
1. ✅ 创建文档和计划（已完成）
2. ⏳ 开始任务 1.1.1 - 创建基础模块结构
3. ⏳ 开始任务 1.1.2 - 设计核心数据结构

### 📋 明天的计划
1. 完成 AST 对比引擎实现
2. 开始集成到 review.rs
3. 实现变更前代码状态获取

---

**记住我们的目标**: 让 AI 在 Vibe Coding 中能够理解 "这个代码变更会对项目架构产生什么影响"！

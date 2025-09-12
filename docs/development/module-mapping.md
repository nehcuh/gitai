# GitAI 模块映射分析

生成日期: 2025-01-12

## 概览

项目当前存在两套并行架构，需要将src/逐步迁移到crates/。

## 详细映射表

### 核心模块

| src/模块 | crates/对应模块 | 状态 | 行动 |
|----------|----------------|------|------|
| src/config.rs | crates/gitai-core/src/config.rs | ✅ 重复 | 删除src版本 |
| src/git.rs | crates/gitai-core/src/git_impl.rs | ✅ 重复 | 删除src版本 |
| src/error.rs | crates/gitai-types/src/error.rs | ⚠️ 部分重复 | 合并后删除 |
| src/context.rs | crates/gitai-core/src/context.rs | 🔄 需迁移 | 迁移到core |
| src/ai.rs | crates/gitai-core/src/ai.rs | ⚠️ 部分重复 | 合并功能 |

### 分析模块

| src/模块 | crates/对应模块 | 状态 | 行动 |
|----------|----------------|------|------|
| src/analysis.rs | crates/gitai-analysis/src/analysis.rs | ⚠️ 部分重复 | 合并功能 |
| src/tree_sitter/* | crates/gitai-analysis/src/tree_sitter/* | ⚠️ 部分重复 | 合并功能 |
| src/architectural_impact/* | crates/gitai-analysis/src/architectural_impact/* | ⚠️ 部分重复 | 合并功能 |
| src/review/* | crates/gitai-analysis/src/review/* | 🔄 需迁移 | 迁移到analysis |

### 功能模块

| src/模块 | crates/对应模块 | 状态 | 行动 |
|----------|----------------|------|------|
| src/commit.rs | crates/gitai-core/src/commit.rs | 🔄 需迁移 | 迁移到core |
| src/scan.rs | crates/gitai-security/src/scanner.rs | 🔄 需迁移 | 迁移到security |
| src/metrics/* | crates/gitai-metrics/* | 🔄 需迁移 | 迁移到metrics |
| src/mcp/* | crates/gitai-mcp/* | ⚠️ 部分重复 | 合并功能 |
| src/devops.rs | crates/gitai-adapters/src/devops.rs | 🔄 需迁移 | 迁移到adapters |

### CLI相关

| src/模块 | crates/对应模块 | 状态 | 行动 |
|----------|----------------|------|------|
| src/args.rs | crates/gitai-cli/src/args.rs | 🔄 需迁移 | 迁移到cli |
| src/cli/* | crates/gitai-cli/src/* | ⚠️ 部分重复 | 合并功能 |
| src/main.rs | - | ✅ 保留 | 作为二进制入口 |
| src/lib.rs | - | ✅ 保留 | 重构为facade |

### Domain层（DDD架构）

| src/模块 | crates/对应模块 | 状态 | 行动 |
|----------|----------------|------|------|
| src/domain/interfaces/config.rs | crates/gitai-core/src/interfaces/config.rs | ✅ 重复 | 删除src版本 |
| src/domain/services/config.rs | crates/gitai-core/src/services/config.rs | ✅ 重复 | 删除src版本 |
| src/domain/entities/* | crates/gitai-types/src/* | 🔄 需迁移 | 迁移到types |
| src/domain/errors/* | crates/gitai-types/src/error.rs | ⚠️ 部分重复 | 合并到types |

### 基础设施层

| src/模块 | crates/对应模块 | 状态 | 行动 |
|----------|----------------|------|------|
| src/infrastructure/container/* | - | 🆕 独特 | 评估是否需要 |
| src/utils/* | crates/gitai-core/src/utils/* | 🔄 需迁移 | 迁移到core |

### 支持模块

| src/模块 | crates/对应模块 | 状态 | 行动 |
|----------|----------------|------|------|
| src/prompts.rs | crates/gitai-core/src/prompts.rs | 🔄 需迁移 | 迁移到core |
| src/resource_manager.rs | crates/gitai-core/src/resource_manager.rs | 🔄 需迁移 | 迁移到core |
| src/config_init.rs | crates/gitai-cli/src/handlers/init.rs | ⚠️ 部分重复 | 合并到cli |
| src/features.rs | crates/gitai-cli/src/features.rs | 🔄 需迁移 | 迁移到cli |
| src/update/* | crates/gitai-cli/src/update/* | 🔄 需迁移 | 迁移到cli |

## 统计分析

### 按状态分类
- ✅ **重复可删除**: 8个文件
- ⚠️ **部分重复需合并**: 10个文件
- 🔄 **需迁移**: 15个文件
- 🆕 **独特功能**: 2个文件
- ✅ **需保留**: 2个文件（main.rs, lib.rs）

### 按目标crate分类
- **gitai-core**: 12个文件
- **gitai-analysis**: 8个文件
- **gitai-cli**: 6个文件
- **gitai-types**: 4个文件
- **gitai-security**: 2个文件
- **gitai-metrics**: 2个文件
- **gitai-mcp**: 2个文件
- **gitai-adapters**: 1个文件

## 迁移优先级

### Phase 1: 清理重复（立即）
1. 删除所有标记为"✅ 重复"的文件
2. 更新导入路径
3. 验证编译

### Phase 2: 合并部分重复（本周）
1. 比较功能差异
2. 合并独特功能到crates
3. 删除src版本

### Phase 3: 迁移独特功能（下周）
1. 按依赖顺序迁移
2. 先迁移底层模块（types, utils）
3. 再迁移上层模块（cli, features）

### Phase 4: 重构lib.rs（2周后）
1. 将lib.rs改为facade模式
2. 只暴露crates的公共接口
3. 删除对src/内部模块的依赖

## 风险评估

### 高风险
- src/lib.rs被广泛使用，需谨慎重构
- src/main.rs依赖src/lib.rs

### 中风险
- 部分模块可能有未识别的依赖
- 测试覆盖不足可能导致功能丢失

### 低风险
- 重复文件删除相对安全
- 工具类模块迁移影响较小

## 执行计划

### 今日任务
1. ✅ 创建模块映射表
2. [ ] 删除8个明确重复的文件
3. [ ] 更新相关导入路径
4. [ ] 运行测试验证

### 本周目标
- 完成Phase 1和Phase 2
- 减少src/文件数量50%
- Box<dyn Error>减少到300个以下

### 成功指标
- [ ] src/目录文件 < 50个
- [ ] 所有测试通过
- [ ] 无新增编译警告
- [ ] 项目完成度 > 45%

---

*"Simplicity is the ultimate sophistication."* - Leonardo da Vinci

# GitAI 架构清理计划

## 现状分析

### 重复架构问题

项目中存在两套并行的架构：

```
旧架构 (src/)                 新架构 (crates/)
├── src/                      ├── crates/
│   ├── domain/               │   ├── gitai-core/
│   │   ├── interfaces/       │   │   ├── interfaces/
│   │   ├── services/         │   │   ├── services/
│   │   └── errors/           │   │   └── domain_errors.rs
│   ├── cli/                  │   ├── gitai-cli/
│   │   └── handlers/         │   │   └── handlers/
│   ├── config.rs             │   └── gitai-types/
│   └── git.rs                        └── error.rs
```

### 重复文件映射

| 旧文件 (src/) | 新文件 (crates/) | 状态 |
|---------------|------------------|------|
| src/config.rs | crates/gitai-core/src/config.rs | 重复 |
| src/domain/interfaces/config.rs | crates/gitai-core/src/interfaces/config.rs | 重复 |
| src/domain/services/config.rs | crates/gitai-core/src/services/config.rs | 重复 |
| src/cli/handlers/config.rs | crates/gitai-cli/src/handlers/config.rs | 重复 |
| src/git.rs | crates/gitai-core/src/git_impl.rs | 几乎相同 |
| src/error.rs | crates/gitai-types/src/error.rs | 功能重叠 |

## 清理策略

### 第一阶段：统一配置管理（本周）

1. **保留单一真实源**
   - 主配置：`crates/gitai-core/src/config.rs`
   - 配置接口：`crates/gitai-core/src/interfaces/config.rs`
   - 配置服务：`crates/gitai-core/src/services/config.rs`

2. **删除重复文件**
   ```bash
   # 要删除的文件
   rm src/config.rs
   rm src/domain/interfaces/config.rs
   rm src/domain/services/config.rs
   rm src/cli/handlers/config.rs
   ```

3. **更新导入路径**
   - 将所有 `use crate::config` 改为 `use gitai_core::config`
   - 将所有 `use crate::domain::interfaces::config` 改为 `use gitai_core::interfaces::config`

### 第二阶段：清理Git模块（下周）

1. **统一Git功能**
   - 保留：`crates/gitai-core/src/git_impl.rs`
   - 删除：`src/git.rs`
   - 创建：`crates/gitai-core/src/git.rs` 作为公共接口

2. **迁移差异功能**
   - 比较两个文件的差异
   - 将独特功能迁移到新文件
   - 删除重复代码

### 第三阶段：完整架构迁移（2周内）

1. **模块迁移顺序**
   ```
   优先级1：核心功能
   - src/analysis.rs → crates/gitai-analysis/
   - src/review/ → crates/gitai-analysis/src/review/
   - src/commit.rs → crates/gitai-core/src/commit.rs
   
   优先级2：工具功能
   - src/scan.rs → crates/gitai-security/src/scan.rs
   - src/metrics/ → crates/gitai-metrics/
   - src/prompts.rs → crates/gitai-core/src/prompts.rs
   
   优先级3：支持功能
   - src/tree_sitter/ → crates/gitai-analysis/src/tree_sitter/
   - src/mcp/ → crates/gitai-mcp/
   - src/devops.rs → crates/gitai-adapters/src/devops.rs
   ```

2. **依赖关系调整**
   ```toml
   # 根Cargo.toml调整为只依赖crates
   [dependencies]
   gitai-core = { path = "crates/gitai-core" }
   gitai-cli = { path = "crates/gitai-cli" }
   gitai-analysis = { path = "crates/gitai-analysis" }
   # 移除对src/的直接依赖
   ```

## 执行计划

### 立即行动（今天）

1. **备份现有代码**
   ```bash
   cp -r src/ src.backup/
   ```

2. **创建迁移脚本**
   ```bash
   scripts/migrate-config.sh
   scripts/update-imports.sh
   ```

3. **逐步迁移**
   - 每次迁移一个模块
   - 确保编译通过
   - 运行测试验证

### 风险控制

1. **分支策略**
   - 在新分支上进行迁移
   - 每个模块迁移后创建提交
   - 完成后合并到主分支

2. **回滚计划**
   - 保留src.backup/目录
   - 使用git进行版本控制
   - 记录每步操作

3. **验证清单**
   - [ ] 编译通过
   - [ ] 测试通过
   - [ ] 功能正常
   - [ ] 无新增警告

## 预期收益

1. **代码量减少**
   - 预计减少30%重复代码
   - 从110个文件减少到约70个

2. **维护性提升**
   - 单一事实来源
   - 清晰的模块边界
   - 统一的错误处理

3. **性能改善**
   - 减少编译时间
   - 减少二进制大小
   - 更好的代码复用

## 成功标准

- [ ] 所有重复config.rs文件已删除
- [ ] src/目录文件数量 < 20个
- [ ] 无编译错误
- [ ] Box<dyn Error>使用量 < 200个
- [ ] 项目完成度 > 50%

## 时间线

| 日期 | 任务 | 目标 |
|------|------|------|
| 2025-01-12 | 配置清理 | 删除重复config.rs |
| 2025-01-13 | Git模块统一 | 合并git.rs |
| 2025-01-14 | 核心模块迁移 | 迁移analysis、review |
| 2025-01-15 | 工具模块迁移 | 迁移scan、metrics |
| 2025-01-19 | 完成迁移 | src/目录清理完成 |

---

*"Perfection is achieved not when there is nothing more to add, but when there is nothing left to take away."* - Antoine de Saint-Exupéry

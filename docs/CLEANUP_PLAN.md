# GitAI 清理与对齐计划（提案）

目标：在不破坏用户空间前提下，对齐文档与实现、明确边界、恢复关键能力（Tree‑sitter），并建立稳定回归路径。

## 阶段 0：冻结（1 天）
- 原则：不再引入新特性，只做整理与对齐；每次改动必须通过构建并跑核心回归。
- 落地：
  - 现状文档：docs/ARCHITECTURE.md（已添加）
  - 回归手册：docs/REGRESSION.md（已添加）

## 阶段 1：文档与 CLI 对齐（1 天）
- README 审核：
  - 将 Tree‑sitter 标注为“实验/逐步恢复”，避免承诺未实现路径（若你坚持“已存在”，则同步恢复实现，见阶段 2）
  - Prompts.update 标注为“占位/规划中”
- CLI 帮助：
  - review --tree-sitter 的描述改为“启用语法结构摘要（实验）”
  - scan 帮助中明确 --lang/--timeout/--no-history/--benchmark 的作用

验收：
- README 中所有示例命令可以原样执行
- CLI --help 输出与 README 一致

## 阶段 2：恢复 Tree‑sitter（最小可用，2-3 天）
- 目标：当传入 --tree-sitter 时，在评审 prompt 中追加“结构化摘要”，先支持 Java，一门跑通；可扩展到其它语言。
- 方案：
  - 新增模块 ts：
    - summarize_files(lang, files) -> String
    - 实现：基于 tree-sitter-java，提取 class/interface 名、method 名、变更函数列表、计数
  - 在 review.execute 流程：
    - 获取 diff 后，提取变更文件列表
    - 若 --tree-sitter，则对选中文件调用 summarize_files，将摘要附加到 diff 尾部
    - 缓存 key 包含 tree_sitter 开关（现已包含 diff 内容，等价满足，但建议纳入 flag）
- 不做：深度数据流/跨文件语义；仅作为 prompt 的结构辅助。

验收：
- gitai review --tree-sitter 对 Java 仓库能附加“结构摘要”段
- 禁用时无任何额外开销；启用时对性能影响小于 +200ms（demo 规模）

## 阶段 3：Prompts.update 对齐（1-2 天）
- 目标：让 prompts update 可从“可信源”拉取或升级本地模板。
- 方案：
  - 支持两类源：
    - git 仓库 URL（浅克隆或下载归档）
    - HTTP 归档（tar.gz/zip）
  - 具备：重试、校验（sha256 或 etag）、失败降级
- 验收：
  - gitai prompts update 成功拉取/更新两个模板文件；失败有清晰提示

## 阶段 4：回归自动化（并行进行）
- 将 docs/REGRESSION.md 的关键用例脚本化（scripts/regression.sh）
- 在 PR 模板中添加“回归表单”

## 阶段 5：性能基线（可选）
- 提供 scripts/bench_scan.sh，固定 --lang/--timeout/--benchmark 条件，输出对比表
- 可选：--raw 模式（直通 opengrep stdout），进一步缩小差距

## 优先级与排期建议
- P0：阶段 1（对齐），阶段 2（Tree‑sitter 最小可用）
- P1：阶段 3（Prompts.update），阶段 4（回归自动化）
- P2：阶段 5（性能基线与 --raw）

## 变更管控
- 每次改动：
  - 必须通过 cargo build --release
  - 必须执行 docs/REGRESSION.md 中“快速冒烟”小节
  - 涉及 scan 的，跑一轮 hyperfine --warmup 3 的对比（记录到 PR）

## 风险与回退
- 风险：Tree‑sitter 解析带来的平台依赖/构建失败
  - 回退策略：保留 feature flag（编译开关/运行开关均可），默认关闭
- 风险：Prompts.update 源不稳定
  - 回退策略：失败不阻断主流程，仅在 update 命令中反馈


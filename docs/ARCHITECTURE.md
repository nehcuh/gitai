# GitAI 架构概览（当前状态）

版本范围：v0.1.x（本仓库现状），目标是“零破坏、可用为先”，避免文档与实现漂移。

## 1. 模块与职责

- src/main.rs
  - 启动、解析 CLI、加载配置、路由到各子命令
- src/args.rs
  - CLI 参数定义（Review/Scan/ScanHistory/Prompts/Commit/Update/Git 代理）
- src/config.rs
  - 配置结构体与加载（~/.config/gitai/config.toml）
- src/git.rs
  - Git 子进程封装（run_git、diff/status/add/commit），禁用 pager 保证非交互稳定
- src/scan.rs
  - OpenGrep 集成：规则目录选择（--lang 优先/自动检测作为后备）、构建命令、运行与 JSON 解析、扫描历史持久化、安装帮助
- src/review.rs
  - 评审主流程：获取 diff → 暂存提示 → 缓存 → 获取 Issue 上下文 → 调用 Analyzer → 保存缓存 → 输出（可阻断严重问题）
- src/analysis.rs
  - Analyzer：
    - analyze_review：组装 prompt 调用 AI（当前不含 Tree‑sitter 结构化摘要）
    - analyze_security：按需调用 scan 进行安全扫描（OpenGrep）
    - analyze_deviation：根据 Issue 上下文做偏离度报告（AI）
- src/commit.rs
  - 智能提交：收集 diff/Issue → 生成提交信息（或使用用户 -m）→ 可选评审 → git add/commit
- src/devops.rs
  - DevOps 客户端（Coding/GitHub）拉取 Issue 简要信息
- src/update/*
  - 自动更新器：规则包下载/解压/元信息写入、prompts（占位）、版本检查与通知
- src/ai.rs
  - AI 请求（OpenAI 兼容），system+user 双消息，支持 api_key/temperature

## 2. CLI 契约（核心）

- gitai review
  - 关键开关：
    - --security-scan（在评审过程中触发 OpenGrep）
    - --block-on-critical（Error 级别时非零退出）
    - --tree-sitter（当前文档存在，但实现未接入；需恢复）
    - --format/--output
  - 默认不破坏：无扫描、无阻断，纯文本输出至 stdout
- gitai scan
  - 关键开关：--lang、--no-history、--timeout、--format、--auto-install、--update-rules
  - 默认：不写历史（已提供 --no-history 以便基准），JSON 解析整块 results
- gitai commit
  - -m 与 AI 互斥优先：有 -m 则格式化+拼接 Issue 前缀；无 -m 才调用 AI 生成
  - --review 可在提交前做一次评审摘要（不阻断）
- gitai <git 子命令>
  - 默认直通原生输出；--ai 时追加“🤖 AI解释”；失败保持非零退出
- gitai update / prompts
  - update：规则/提示词/版本检查；prompts 的 update 为占位

## 3. 数据与状态

- 配置：~/.config/gitai/config.toml
- 规则：~/.cache/gitai/rules（附 .rules.meta）、语言子目录（java/python/...）
- 扫描历史：~/.cache/gitai/scan_history/scan_<tool>_<ts>.json（可用 --no-history 禁用）
- 评审缓存：~/.cache/gitai/review_cache/review_<cache_key>.json
- Prompts：~/.config/gitai/prompts/*.md（init 时生成默认模板）

## 4. 关键流程

- Review 流：
  1) diff = git.get_all_diff()
  2) 暂存状态提示（非强制）
  3) cache_key = md5(diff + language + security_scan + deviation + issue_ids)
  4) 若命中缓存 → 输出 → 结束
  5) issues = DevOps.get_issues(issue_ids)（可选）
  6) analysis = Analyzer.analyze(context)
     - review_result = AI 文本
     - security_findings = run_opengrep_scan（按需）
     - deviation_analysis = AI 提取（按需）
  7) 保存缓存 + 输出 + （可选阻断）

- Scan 流：
  1) 选择规则目录：--lang 优先；否则自动统计主语言 → 子目录；不存在则回退根目录
  2) 构建 opengrep 参数（--json/--quiet/--timeout）+ 可选 --jobs
  3) 执行并解析 results 数组 → 写历史（可禁用）→ 输出（text 或 json）

- Commit 流：
  1) diff/issue 上下文 → AI 生成或使用 -m
  2) 可选评审摘要（不阻断）
  3) git add/commit

- Git 代理：
  1) 运行原生命令捕获 stdout/stderr+退出码 → 直通 → 可选 AI 解释 → 保持退出码

## 5. 行为契约（Never break userspace）

- 不替换 git，也不默认改写输出或退出码
- 所有增强（AI解释、阻断、扫描）均显式开关
- 错误时保留原始 stderr，并保持非零退出码

## 6. 性能与可用性

- 封装层对 opengrep 的开销：< 1s（当前基准约 1.28x 原生）；提供 --lang/--no-history/--timeout/--benchmark 以对齐条件
- 复杂命令的分页/交互风险：禁用 pager 保证非交互输出

## 7. 风险与已知差异

- Tree‑sitter：CLI 有开关，但未接入 Analyzer；文档存在超前描述
- Prompts.update：占位未实现真实源
- README 与实现细节：已做一轮对齐，但仍需二次审视（尤其 Tree‑sitter 章节）

## 8. 设计原则

- 简洁优先：最少分支、明确数据流
- 配置可选，约定优于配置；出现配置缺失时给出清晰提示
- 不引入交互式 CLI；保持脚本友好

## 9. 后续演进（简述）

- 恢复并接入 Tree‑sitter（最小可用：结构摘要注入 prompt）
- 完成 prompts.update 真实源同步（git/URL），含重试与校验
- 加 smoke tests 与基准章节的自动化脚本


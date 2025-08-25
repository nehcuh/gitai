# GitAI 回归测试手册

> 目标：每次改动后，2-10 分钟内验证关键功能，确保“Never break userspace”。

## 0. 前置条件
- Mac/Linux/WSL 任一环境
- Rust 工具链（用于构建与自动安装 OpenGrep）
- 可用的 AI 服务（可用本地 Ollama 或任意 OpenAI 兼容端点），按 README 配置 ~/.config/gitai/config.toml
- 可选测试仓库：java-sec-code（或任意含已知问题的演示仓库）

## 1. 快速冒烟（2 分钟）

- 构建
```bash path=null start=null
cargo build
```
- 直通 Git（默认不解释）
```bash path=null start=null
./target/debug/gitai status
```
- AI 解释（显式开关）
```bash path=null start=null
./target/debug/gitai --ai status
```
- Prompts 基础
```bash path=null start=null
./target/debug/gitai prompts init
./target/debug/gitai prompts list
./target/debug/gitai prompts show --name commit-generator
```

预期：
- 构建成功
- status 正常输出；--ai 模式在原始输出后追加“🤖 AI解释”
- prompts 目录初始化并可列出/查看模板

## 2. 扫描（OpenGrep 集成与规则加载）

- 自动安装与更新规则
```bash path=null start=null
./target/debug/gitai scan --auto-install --update-rules
```
- 扫描已知问题项目（示例：java-sec-code）
```bash path=null start=null
# 将路径替换为你的本地项目路径
./target/debug/gitai scan -p ../java-sec-code
```
- 导出 JSON 结果
```bash path=null start=null
./target/debug/gitai scan -p ../java-sec-code --format=json --output=scan-java.json
```
- 交叉验证（直接调用 OpenGrep，应与上面结果一致量级）
```bash path=null start=null
opengrep --json --quiet --timeout=300 --jobs=4 \
  --config "$HOME/.cache/gitai/rules/java" ../java-sec-code | head -n 5
```

预期：
- 自动安装成功（如未安装 cargo 或 PATH 未配置，会给出明确指引）
- 对 java-sec-code 能发现较多问题（非 0 条）；文本模式显示前 5 条；JSON 模式包含完整 results 数组
- 直接调用 OpenGrep（指定 --config=规则目录下的语言子目录）能输出与工具近似的 findings 数量级

常见坑位与判定：
- 若报“unknown option '--rules'”：说明错误地使用了旧参数，应统一使用 --config=...
- 若 JSON 输出 error 含 "Invalid rule schema"：代表选择了包含非规则 YAML 的目录，需切换到具体语言子目录（如 rules/java）

## 3. 评审（Review）与缓存

- 准备：确保仓库有暂存或未暂存变更
```bash path=null start=null
# 示例：制造一个小改动
echo "// noop" >> src/tmp_noop.rs
```
- 基础评审
```bash path=null start=null
./target/debug/gitai review --format=text
```
- 命中缓存（重复执行，应打印“使用缓存的评审结果”提示）
```bash path=null start=null
./target/debug/gitai review --format=text
```
- 触发不同缓存键（增加安全扫描或带 Issue 上下文）
```bash path=null start=null
./target/debug/gitai review --security-scan
./target/debug/gitai review --issue-id="#123,#456"
```
- 严重问题阻断（需确实检测到 ERROR 级别安全问题）
```bash path=null start=null
./target/debug/gitai review --security-scan --block-on-critical || echo "blocked as expected"
```

预期：
- 第一次评审输出分析内容；第二次提示“使用缓存”；添加 --security-scan 或 --issue-id 时应重新计算缓存
- 存在 ERROR 级别时 --block-on-critical 返回非零退出（并打印阻断提示）

## 4. 提交（Commit）

- 智能提交（不实际提交）
```bash path=null start=null
./target/debug/gitai commit --dry-run --issue-id="#123" -m "feat: demo"
```
- 结合评审（不实际提交）
```bash path=null start=null
./target/debug/gitai commit --dry-run --review --issue-id="#123"
```

预期：
- 控制台打印最终提交信息（含 Issue 前缀）；--review 会先输出简短评审摘要

## 5. Git 代理零破坏验证

- 直通（默认）
```bash path=null start=null
./target/debug/gitai log --oneline -5
```
- AI 解释（显式）
```bash path=null start=null
./target/debug/gitai --ai log --oneline -5
```
- 错误场景（应保留原始错误并追加解释，且保持非零退出码）
```bash path=null start=null
./target/debug/gitai --ai rebase definitely-not-a-branch || echo "non-zero as expected"
```

预期：
- 非 AI 模式仅输出原始 Git 结果
- AI 模式先原始输出，再“🤖 AI解释”；失败保持非零退出

## 6. Prompts 管理

- 初始化/更新/查看
```bash path=null start=null
./target/debug/gitai prompts init
./target/debug/gitai prompts update
./target/debug/gitai prompts list
./target/debug/gitai prompts show --name review
```

预期：
- init 在 ~/.config/gitai/prompts 写入默认模板（若不存在）
- update 正常返回成功/失败信息（当前为占位更新逻辑，未来会接入真实源）

## 7. 更新检查（规则/提示词/版本）

- 检查（文本）
```bash path=null start=null
./target/debug/gitai update --check
```
- 检查（JSON）
```bash path=null start=null
./target/debug/gitai update --check --format=json
```

预期：
- 文本模式列出需要更新的组件；JSON 模式包含 ready 字段与 rules_info 段

## 8. 可选：Git hooks 冒烟

- pre-commit（仅示例，按需启用）
```bash path=null start=null
echo 'gitai review --security-scan --block-on-critical' > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```
提交时：若扫描出现 ERROR 级别应阻断。

## 9. 故障排除

- OpenGrep 未安装或 PATH 未配置
  - 提示按文档安装 Rust 与 cargo，或 export PATH="$HOME/.cargo/bin:$PATH"
- 扫描无发现
  - 检查是否正确选择了语言规则目录（当前默认会根据扫描路径语言自动选择；也可手动验证：
```bash path=null start=null
opengrep --json --quiet --timeout=300 --jobs=4 \
  --config "$HOME/.cache/gitai/rules/<lang>" <project-dir> | head -n 5
```
- AI 解释无输出
  - 检查 ~/.config/gitai/config.toml 的 [ai] 配置，或使用更轻量的模型

## 10. 基准测试（性能对齐原生 OpenGrep）

- 预热缓存，确保条件公平
```bash
hyperfine --warmup 3 \
  'opengrep --json --quiet --timeout=300 --config=$HOME/.cache/gitai/rules/java ../java-sec-code >/dev/null' \
  './target/release/gitai scan -p ../java-sec-code --lang=java --no-history --format=json >/dev/null'
```
- 可选：使用 --benchmark 快捷开关（等价于禁用历史与跳过版本查询），并直通超时参数
```bash
./target/release/gitai scan -p ../java-sec-code --lang=java --benchmark --timeout=300 --format=json >/dev/null
```

预期：
- 两者处于同等条件下，时间接近；若仍有明显差距，检查是否存在规则选择差异、I/O 干扰或第一次运行冷缓存影响（可增大 --warmup 次数）。

## 11. CI 建议

- 最小阻断：在 PR 流水线中运行
```bash path=null start=null
./gitai scan -p . --format=json --output=scan.json || true
./gitai review --format=json --output=review.json || true
# 若需要阻断高危：
./gitai review --security-scan --block-on-critical
```

---

维护原则：
- 默认“直通”原生行为，增强功能全部显式开启
- 文档中的每条命令都必须在当前版本可直接运行
- 若有行为变更，先更新本文档与 README，再合并代码


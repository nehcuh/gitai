# Workspace Migration Plan (Proposal)

Goal
- Adopt a Cargo workspace layout to decouple core, CLI, MCP, security, metrics, and types into separate crates, improving build times, dependency management, and modularity.

Current State
- The repository already contains crates/* with per-crate Cargo.toml files using `workspace.*` fields, but the root Cargo.toml is a monolithic crate (package = gitai).
- Mixed usage may cause duplicated dependencies and longer build times.

Minimal Migration Steps
1) Introduce a root [workspace] Cargo.toml
   - Create `Cargo.toml` at repo root with `[workspace]` members:
     - members = ["crates/*", "."] (temporarily include the root package to support a gradual migration)
   - Move common dependency versions to `[workspace.dependencies]` for unification.

2) Extract root package into crates/gitai-cli and crates/gitai-core if not already aligned
   - If the root package contains CLI and library, split into `gitai-cli` (bin) and `gitai-core` (lib).
   - Adjust paths in other crates to depend on `gitai-core`.

3) Convert bin targets
   - Move `src/main.rs` into `crates/gitai-cli/src/main.rs` (if not already in place).
   - Move `src/bin/gitai-mcp.rs` into `crates/gitai-mcp/src/main.rs`.

4) Unify dependencies
   - In the root workspace Cargo.toml, define pinned versions in `[workspace.dependencies]` for:
     - tokio, serde (+derive), clap (+derive), reqwest, lru, md5, walkdir, tempfile, log, tracing, tracing-subscriber, chrono, semver.
   - Remove duplicated version specs from per-crate Cargo.toml files to inherit workspace versions.

5) CI updates
   - Use `--workspace` in build/test/lint commands.
   - Ensure feature flags are scoped per crate (e.g., cli enables features of analysis/security crates via dependency features).

6) Backward-compatibility validation
   - Run full test suite: `cargo test --all-features --workspace`.
   - Run benches: `cargo bench --workspace` (optional in CI).
   - Verify binary targets `gitai` and `gitai-mcp` still build and run.

Roll-back Plan
- Keep the root package functional during steps 1-2. If any step fails, revert the member list and crate moves.
- Use a feature-flagged branch; merge after passing CI and benchmarks.

Risks and Mitigations
- Risk: Feature and optional-dependency mismatches across crates.
  - Mitigation: Centralize features in CLI and MCP crates; document in each crate README.
- Risk: CI breakage due to new workspace commands.
  - Mitigation: Stage CI changes, keep default build as safety net until stabilized.

Next Steps
- Confirm scope: full workspace migration vs. partial unification (dependencies only).
- If approved, I can open a PR implementing step 1 (root workspace file + dep unification map), then proceed incrementally.


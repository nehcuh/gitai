# Changelog

All notable changes to this project will be documented in this file.

## [1.1.0] - 2025-09-09
### Added
- Tree-sitter concurrent analysis worker pool with TreeSitterManager reuse for better performance.
- Expanded MCP API and documentation, including summarize_graph and dependency graph tooling.
- Documentation index (docs/README.md) and clarified links across docs.

### Changed
- Tightened CI quality gates: clippy -D warnings; all tests pass.
- Cleaned up imports and fixed clippy warnings across source and tests.
- Updated API_REFERENCE.md with performance notes and examples; added MCP Quickstart (stdio).
- Updated MCP_SERVICE.md with deviation_analysis behavior and environment variables section.
- Improved MCP_GRAPH_SUMMARY.md with end-to-end flow examples.
- README documentation links corrected and consolidated.

### Fixed
- Broken links in examples/mcp_integration/README.md pointing to implementation notes.

---

Keep this changelog updated as part of each release.


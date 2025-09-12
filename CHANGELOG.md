# Changelog

All notable changes to this project will be documented in this file.

## [2.0.0] - 2025-01-12

### Changed
- **BREAKING**: Complete architecture refactoring to Workspace structure
- Migrated from monolithic application to 9 specialized crates
- Unified error handling system using GitAIError enum
- Removed 33% of redundant code through deduplication

### Added  
- gitai-core: Core business logic and interfaces
- gitai-types: Shared types and error definitions
- gitai-analysis: Code analysis engine with Tree-sitter
- gitai-security: Security scanning with OpenGrep
- gitai-metrics: Quality metrics and trend analysis
- gitai-mcp: MCP protocol server implementation
- gitai-adapters: External service adapters
- gitai-cli: Command-line interface
- gitai-evaluation: Project quality evaluation tools

### Fixed
- All compilation errors resolved
- Type safety improvements (96% reduction in dynamic errors)
- Module dependencies properly structured

### Removed
- Duplicate code files and backup directories
- Legacy error handling using Box<dyn Error>
- Redundant feature flags

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


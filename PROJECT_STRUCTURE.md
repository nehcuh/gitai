# GitAI Project Structure

This document describes the organized file structure of the GitAI project.

## 📁 Directory Structure

```
gitai/
├── 📚 docs/                    # All documentation
│   ├── guides/                 # User and developer guides
│   ├── architecture/           # Technical architecture docs
│   ├── api/                    # API documentation
│   ├── examples/               # Usage examples and demos
│   ├── assets/                 # Documentation assets (prompts, etc.)
│   ├── prds/                   # Product requirement documents
│   └── stories/                # Feature development stories
│
├── ⚙️ configs/                 # Configuration files and templates
│   ├── templates/              # Configuration templates
│   ├── examples/               # Example configurations
│   └── prompts/                # External prompt configurations
│
├── 🧪 tests/                   # Test files
│   ├── integration_*.rs        # Integration tests
│   └── *.rs                    # Unit tests
│
├── 💻 src/                     # Source code
│   ├── common/                 # Common types, errors, config
│   ├── cli/                    # Command line interface
│   ├── git/                    # Git operations
│   ├── ai/                     # AI integration
│   ├── handlers/               # Command handlers
│   ├── clients/                # External API clients
│   ├── ast_grep_analyzer/      # Code analysis engine
│   ├── types/                  # Legacy type definitions
│   └── utils/                  # Utility functions
│
├── 🎯 target/                  # Rust build artifacts (auto-generated)
│
└── 📋 Project Files            # Core project files
    ├── README.md               # Main project readme
    ├── CLAUDE.md              # Claude development instructions
    ├── Cargo.toml             # Rust project configuration
    ├── Cargo.lock             # Dependency lock file
    └── PROJECT_STRUCTURE.md   # This file
```

## 📚 Documentation Organization

### `/docs/guides/`
- `DEVELOPMENT_GUIDE.md` - Developer setup and contribution guide
- `MIGRATION_GUIDE.md` - Migration guide for refactored codebase
- `TRANSLATION_PROJECT_COMPLETION.md` - Translation system completion guide
- `TRANSLATION_TROUBLESHOOTING.md` - Translation system troubleshooting
- `testing-issue-resolution.md` - Testing and issue resolution guide

### `/docs/architecture/`
- `REFACTORING_PLAN.md` - Overall refactoring strategy
- `REFACTORING_SUMMARY.md` - Refactoring completion summary
- `TRANSLATION_REFACTOR_DESIGN.md` - Translation system architecture
- `AI_ANALYSIS_INTEGRATION.md` - AI analysis integration design
- `NETWORK_COMPATIBILITY.md` - Network and compatibility considerations
- `TRANSLATION_SYSTEM_ARCHITECTURE.md` - Detailed translation architecture

### `/docs/assets/`
- Various prompt templates and configuration examples
- Documentation supporting files

### `/docs/prds/` & `/docs/stories/`
- Product requirement documents
- Feature development stories and specifications

## ⚙️ Configuration Files

### `/configs/templates/`
- `custom_rules.yaml` - Custom AST-grep rules template
- Other configuration templates

### `/configs/prompts/`
- `prompts_config.yaml` - External prompt configurations

### `/configs/examples/`
- Example configuration files for different use cases

## 🧪 Testing

### `/tests/`
- `integration_commit_test.rs` - Commit functionality integration tests
- `translation_integration.rs` - Translation system integration tests
- Other test files

## 💻 Source Code Organization

The source code follows a modular architecture:

- **common/** - Shared types, error handling, and configuration
- **cli/** - Command line interface and argument parsing
- **git/** - Git repository operations and management
- **ai/** - AI service integration and analysis
- **handlers/** - Command handlers for different operations
- **clients/** - External service clients (DevOps platforms, etc.)
- **ast_grep_analyzer/** - Code analysis and AST processing
- **types/** - Legacy type definitions (being phased out)
- **utils/** - Utility functions

## 🚀 Getting Started

1. Read `README.md` for project overview
2. Check `CLAUDE.md` for development instructions
3. Follow `docs/guides/DEVELOPMENT_GUIDE.md` for setup
4. Use `docs/guides/MIGRATION_GUIDE.md` if migrating from older versions

## 📝 Notes

- The project structure has been recently refactored for better organization
- Legacy files and empty directories have been cleaned up
- All documentation is now centralized under `/docs/`
- Configuration files are organized under `/configs/`
- The source code follows a clean modular architecture
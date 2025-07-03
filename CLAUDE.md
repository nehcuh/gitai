# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

GitAI is an AI-powered Git tools suite written in Rust that integrates artificial intelligence into Git workflows. It provides intelligent code review, automated commit message generation, and enhanced Git command interpretation with multi-language support.

### Core Features
- **Intelligent Code Review** (`gitai review`): AI-driven code analysis with AstGrep syntax parsing
- **Smart Commit Assistant** (`gitai commit`): AI-generated commit messages with Tree-sitter enhancement  
- **Intelligent Git Agent** (`--ai` flag): All Git commands enhanced with AI interpretation
- **DevOps Integration**: Automatic work item tracking and requirement consistency analysis
- **Multi-language Translation** (`--lang`): Chinese/English output support for all commands

## Development Commands

### Build and Test
```bash
# Build the project
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration_commit_test
cargo test --test translation_integration
```

### Development Workflow
```bash
# Run in development mode with debug logging
RUST_LOG=debug cargo run -- help

# Test specific commands
cargo run -- review
cargo run -- commit
cargo run -- scan src/
cargo run -- --lang=zh scan src/

# Test with AI features
cargo run -- --ai status
```

### Linting and Formatting
```bash
# Format code
cargo fmt

# Check for linting issues
cargo clippy

# Check for unused dependencies
cargo +nightly udeps  # requires cargo-udeps
```

## Project Structure

The project follows a clean, modular architecture. For detailed structure information, see `PROJECT_STRUCTURE.md`.

### Directory Overview
```
gitai/
├── docs/                     # All documentation
│   ├── guides/              # Development and user guides
│   ├── architecture/        # Technical architecture docs
│   ├── examples/            # Usage examples
│   └── assets/              # Prompt templates and assets
├── configs/                  # Configuration files and templates
│   ├── templates/           # Configuration templates
│   ├── examples/            # Example configurations
│   └── prompts/             # External prompt configurations
├── src/                     # Source code
│   ├── common/              # Shared types, errors, config
│   ├── cli/                 # Command line interface
│   ├── git/                 # Git operations
│   ├── ai/                  # AI integration
│   ├── handlers/            # Command handlers
│   ├── clients/             # External API clients
│   └── ast_grep_analyzer/   # Code analysis engine
└── tests/                   # Test files
```

### Key Components

#### Command Processing Flow
1. **Argument Parsing**: `main.rs` handles CLI arguments and global flags (`--ai`, `--noai`, `--lang`)
2. **Command Routing**: Commands are routed to appropriate handlers (review, commit, scan, etc.)
3. **AI Integration**: When `--ai` flag is used, all Git commands are enhanced with AI interpretation
4. **Language Processing**: `--lang` parameter enables translation for all output

#### Configuration System
- Configuration loaded from `~/.config/gitai/config.toml`
- Supports AI service configuration (OpenAI, Ollama, etc.)
- DevOps platform integration settings
- Translation service configuration

#### AST-Grep Integration
- Multi-language syntax analysis using ast-grep-core
- Supports Rust, Python, JavaScript, Java, Go, C/C++
- Custom rule management and caching
- Performance optimized with parallel processing

#### Translation System
- Real-time translation between Chinese and English
- Caching system for performance optimization
- Configurable translation providers
- Automatic language detection

## Important Development Guidelines

### Error Handling
- Use `AppError` enum for all errors (defined in `errors.rs`)
- Implement proper error context with `thiserror` 
- Log errors appropriately with tracing

### Configuration Management
- Always use `AppConfig::load()` to get configuration
- Command-line arguments can override config file settings
- Handle missing configuration gracefully

### AI Service Integration
- AI calls are asynchronous and should use proper error handling
- Support for multiple AI providers (OpenAI, Ollama, etc.)
- Implement timeout and retry logic for robustness

### Git Operations
- Use the wrapper functions in `handlers/git.rs` for Git operations
- Always handle Git command failures gracefully
- Preserve original Git exit codes when passing through commands

### Translation Features
- Translation is enabled via `--lang` parameter or config
- Use `SupportedLanguage` enum for language specification
- Implement caching for translation performance
- All user-facing output should support translation

### Testing
- Unit tests for individual components
- Integration tests for command flows
- Use `tempfile` for filesystem tests
- Mock external APIs with `httpmock` for testing

### Code Style
- Follow standard Rust conventions
- Use `cargo fmt` for consistent formatting
- Address `cargo clippy` warnings
- Use meaningful variable and function names
- Comment complex logic, especially in AI integration

## DevOps Integration Notes

The project integrates with DevOps platforms (primarily Coding.net) for:
- Work item tracking and requirement analysis
- Automated commit message enhancement with issue IDs
- Code review correlation with business requirements
- Project management workflow integration

When working on DevOps features, ensure proper API client configuration and error handling for network operations.
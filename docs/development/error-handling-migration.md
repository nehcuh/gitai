# Error Handling Migration Guide

## Overview

GitAI is migrating to a unified error handling approach that provides:
- Consistent error types across the codebase
- Rich context information for debugging
- User-friendly error messages
- Structured logging support
- Error recovery suggestions

## New Error Handling APIs

### Core Traits

#### `GitAIErrorExt`
Extension trait for enhanced error handling:

```rust
use gitai::error_ext::GitAIErrorExt;

// Convert any error to GitAIError
let gitai_err = some_error.to_gitai_error();

// Log error with full chain
error.log_error();

// Get error chain for diagnostics
let chain = error.error_chain();

// Check if recoverable
if error.is_recoverable() {
    // Attempt recovery
}

// Get suggested action
if let Some(action) = error.suggested_action() {
    println!("Suggestion: {}", action);
}
```

#### `ResultExt<T>`
Extension trait for Result types:

```rust
use gitai::error_ext::ResultExt;

// Add context to errors
let result = operation()
    .context("Failed to perform operation")?;

// Lazy context (only evaluated on error)
let result = expensive_operation()
    .with_context(|| format!("Failed at step {}", step))?;

// Log and continue on error
let maybe_value = fallible_operation()
    .log_error_and_continue();

// User-friendly message (hides technical details)
let result = complex_operation()
    .user_message("Unable to process request. Please try again.")?;

// Add technical details for debugging
let result = operation()
    .technical_details("Check network configuration")?;
```

### Macros

```rust
// Create GitAIError with message
let err = gitai_error!("Operation failed");
let err = gitai_error!("Failed at step {}", step);

// Add context to results
let result = with_context!(operation(), "Failed to complete");
let result = with_context!(operation(), "Step {} failed", step);
```

## Migration Strategy

### Phase 1: Public API Boundaries
Start with CLI handlers and MCP services - the entry points where errors are handled and displayed to users.

### Phase 2: Core Module Boundaries
Convert core modules (analysis, review, commit, scan) to use unified error handling at their public interfaces.

### Phase 3: Internal Consistency
Gradually convert internal functions to use the new error handling approach.

## Migration Examples

### Before: CLI Handler
```rust
pub async fn handle_review(
    config: &Config,
    args: &ReviewArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let review = review::execute_review(config, args)?;
    println!("{}", review);
    Ok(())
}
```

### After: CLI Handler with Enhanced Error Handling
```rust
use gitai::error_ext::{GitAIErrorExt, ResultExt};

pub async fn handle_review(
    config: &Config,
    args: &ReviewArgs,
) -> gitai::error::Result<()> {
    let review = review::execute_review(config, args)
        .await
        .context("Failed to execute code review")?
        .user_message("Unable to complete code review. Please check your repository state.")?;
    
    println!("{}", review);
    Ok(())
}
```

### Before: MCP Service
```rust
async fn handle_tool_call(
    &self,
    name: &str,
    arguments: Value,
) -> Result<Value, Box<dyn Error>> {
    match name {
        "execute_review" => {
            let result = self.execute_review(arguments)?;
            Ok(result)
        }
        _ => Err("Unknown tool".into())
    }
}
```

### After: MCP Service with Recovery
```rust
use gitai::error_ext::{GitAIErrorExt, ResultExt, ErrorRecovery};

async fn handle_tool_call(
    &self,
    name: &str,
    arguments: Value,
) -> McpResult<Value> {
    match name {
        "execute_review" => {
            let result = self.execute_review(arguments)
                .await
                .with_context(|| format!("Failed to execute tool: {}", name))?;
            
            // Check for recovery on error
            if let Err(ref e) = result {
                if let Some(action) = ErrorRecovery::try_recover(e) {
                    log::info!("Recovery suggested: {:?}", action);
                }
            }
            
            Ok(result?)
        }
        _ => Err(gitai_error!("Unknown tool: {}", name).into())
    }
}
```

## Error Type Mapping

| Old Error Type | New Error Type | Notes |
|----------------|----------------|-------|
| `Box<dyn Error>` | `GitAIError` | Use `.to_gitai_error()` for conversion |
| `anyhow::Error` | `GitAIError` | Use `.to_gitai_error()` or `.context()` |
| `std::io::Error` | `GitAIError` | Automatic conversion via `From` trait |
| Custom errors | `GitAIError` variant | Add new variants as needed |

## Best Practices

### 1. Add Context at Boundaries
```rust
// Good: Add context at module boundaries
pub fn process_files(paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        process_single_file(path)
            .with_context(|| format!("Failed to process: {}", path.display()))?;
    }
    Ok(())
}
```

### 2. Use User-Friendly Messages for CLI
```rust
// Good: Hide technical details from users
let result = complex_ai_operation()
    .user_message("AI service is currently unavailable. Please try again later.")?;
```

### 3. Log Technical Details
```rust
// Good: Log technical details for debugging
match operation() {
    Ok(val) => val,
    Err(e) => {
        e.log_error(); // Logs full error chain
        return Err(e.to_gitai_error());
    }
}
```

### 4. Check for Recoverability
```rust
// Good: Attempt recovery when possible
if let Err(e) = operation() {
    if e.is_recoverable() {
        // Retry or use fallback
        return fallback_operation()
            .context("Fallback also failed")?;
    }
    return Err(e.to_gitai_error());
}
```

### 5. Provide Suggestions
```rust
// Good: Help users resolve issues
if let Err(e) = operation() {
    if let Some(suggestion) = e.suggested_action() {
        eprintln!("Error: {}", e);
        eprintln!("Suggestion: {}", suggestion);
    }
    return Err(e.to_gitai_error());
}
```

## Testing Error Handling

### Unit Tests
```rust
#[test]
fn test_error_with_context() {
    let result: Result<()> = Err(gitai_error!("Base error"));
    let with_context = result.context("Additional context");
    
    assert!(with_context.is_err());
    let err = with_context.unwrap_err();
    assert!(err.to_string().contains("Additional context"));
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_recovery_on_timeout() {
    let result = unreliable_operation()
        .await
        .context("Network operation failed");
    
    if let Err(e) = result {
        assert!(e.is_recoverable());
        assert!(e.suggested_action().is_some());
    }
}
```

## Migration Checklist

- [ ] Phase 1: CLI Handlers
  - [ ] `src/cli/handlers/review.rs`
  - [ ] `src/cli/handlers/commit.rs`
  - [ ] `src/cli/handlers/scan.rs`
  - [ ] `src/cli/handlers/metrics.rs`
  - [ ] `src/cli/handlers/graph.rs`

- [ ] Phase 2: MCP Services
  - [ ] `src/mcp/manager.rs`
  - [ ] `src/mcp/services/review.rs`
  - [ ] `src/mcp/services/commit.rs`
  - [ ] `src/mcp/services/scan.rs`
  - [ ] `src/mcp/services/analysis.rs`

- [ ] Phase 3: Core Modules
  - [ ] `src/review.rs`
  - [ ] `src/commit.rs`
  - [ ] `src/scan.rs`
  - [ ] `src/analysis.rs`

## Common Patterns

### Pattern: Fallback on Error
```rust
let result = primary_operation()
    .or_else(|e| {
        e.log_error();
        fallback_operation()
            .context("Fallback failed after primary operation failed")
    })?;
```

### Pattern: Aggregate Errors
```rust
let mut errors = Vec::new();
for item in items {
    if let Err(e) = process_item(item) {
        errors.push(e.error_chain());
    }
}
if !errors.is_empty() {
    return Err(gitai_error!("Failed to process {} items", errors.len()));
}
```

### Pattern: Conditional Recovery
```rust
loop {
    match operation() {
        Ok(val) => return Ok(val),
        Err(e) if e.is_recoverable() && retries < MAX_RETRIES => {
            retries += 1;
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }
        Err(e) => return Err(e.to_gitai_error()),
    }
}
```

## FAQ

**Q: When should I use `GitAIError` vs `anyhow::Error`?**
A: Use `GitAIError` at public API boundaries and for user-facing errors. Use `anyhow::Error` internally for maximum flexibility.

**Q: How do I add a new error variant?**
A: Add it to the `GitAIError` enum in `src/error.rs`, then update the `From` implementations as needed.

**Q: Should I always add context?**
A: Add context at module boundaries and when the error alone wouldn't provide enough information for debugging.

**Q: How do I handle errors in async code?**
A: The same traits work with async functions. Use `.await?` and add context as normal.

## Resources

- [Error Handling in Rust](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [anyhow Documentation](https://docs.rs/anyhow)
- [Best Practices for Error Handling](https://www.lpalmieri.com/posts/error-handling-rust/)

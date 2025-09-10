#!/bin/bash
# Fix remaining import errors in gitai-analysis crate

CRATE_DIR="/Users/huchen/Projects/gitai/crates/gitai-analysis"

echo "Fixing error imports..."

# Fix error imports
find "$CRATE_DIR/src" -name "*.rs" -type f -exec sed -i '' \
  -e 's/use crate::error::{GitAIError, GitError};/use gitai_types::{GitAIError, DomainError as GitError};/g' \
  -e 's/use crate::error::{GitAIError, Result};/use gitai_types::{GitAIError, Result};/g' \
  -e 's/use crate::error::/use gitai_types::/g' \
  {} \;

# Fix git imports
find "$CRATE_DIR/src" -name "*.rs" -type f -exec sed -i '' \
  -e 's/use crate::git;/use gitai_core::git;/g' \
  -e 's/use crate::git::/use gitai_core::git::/g' \
  {} \;

# Fix the shuffle method in graph_export.rs
sed -i '' '/use rand::seq::SliceRandom;/a\
use rand::prelude::*;' "$CRATE_DIR/src/architectural_impact/graph_export.rs"

# Fix error types in error_handling.rs
sed -i '' \
  -e 's/GitAIError::Unknown/GitAIError::Generic/g' \
  -e 's/ConfigResult/gitai_core::interfaces::ConfigResult/g' \
  -e 's/GitResult/gitai_core::interfaces::GitResult/g' \
  -e 's/AiResult/gitai_core::interfaces::AiResult/g' \
  -e 's/ScanResult/gitai_core::interfaces::ScanResult/g' \
  -e 's/ConfigError::/gitai_core::domain_errors::ConfigError::/g' \
  -e 's/GitError::/gitai_core::domain_errors::GitError::/g' \
  -e 's/AiError::/gitai_core::domain_errors::AiError::/g' \
  -e 's/ScanError::/gitai_core::domain_errors::ScanError::/g' \
  "$CRATE_DIR/src/utils/error_handling.rs"

# Add missing imports to error_handling.rs
sed -i '' '1i\
use gitai_core::domain_errors::{ConfigError, GitError, AiError, ScanError};\
use gitai_core::interfaces::{ConfigResult, GitResult, AiResult, ScanResult};\
' "$CRATE_DIR/src/utils/error_handling.rs"

# Fix the HashMap key type in git_state_analyzer.rs
sed -i '' 's/results.insert(file_path.clone()/results.insert(file_path.to_string()/g' "$CRATE_DIR/src/architectural_impact/git_state_analyzer.rs"

echo "Import error fixes complete!"

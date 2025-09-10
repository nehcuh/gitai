#!/bin/bash
# Fix Result type usage in gitai-core

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CORE_SRC="$PROJECT_ROOT/crates/gitai-core/src"

echo "=== Fixing Result types in gitai-core ==="

# Fix Result<T, DomainError> to std::result::Result<T, DomainError>
find "$CORE_SRC" -name "*.rs" -type f | while read -r file; do
    echo "Fixing: $file"
    
    # Replace Result<..., DomainError> with std::result::Result<..., DomainError>
    sed -i '' 's/-> Result<\([^,>]*\), DomainError>/-> std::result::Result<\1, crate::domain_errors::DomainError>/g' "$file"
    sed -i '' 's/-> Result<\([^,>]*\), ConfigError>/-> std::result::Result<\1, crate::domain_errors::ConfigError>/g' "$file"
    sed -i '' 's/-> Result<\([^,>]*\), GitError>/-> std::result::Result<\1, crate::domain_errors::GitError>/g' "$file"
    sed -i '' 's/-> Result<\([^,>]*\), AiError>/-> std::result::Result<\1, crate::domain_errors::AiError>/g' "$file"
    sed -i '' 's/-> Result<\([^,>]*\), CacheError>/-> std::result::Result<\1, crate::domain_errors::CacheError>/g' "$file"
    sed -i '' 's/-> Result<\([^,>]*\), ScanError>/-> std::result::Result<\1, crate::domain_errors::ScanError>/g' "$file"
done

echo "Result type fixes complete."

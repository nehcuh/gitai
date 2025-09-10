#!/bin/bash
# Update imports in gitai-core crate

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CORE_SRC="$PROJECT_ROOT/crates/gitai-core/src"

echo "=== Updating imports in gitai-core ==="

# Find all Rust files and update imports
find "$CORE_SRC" -name "*.rs" -type f | while read -r file; do
    echo "Updating: $file"
    
    # Update crate:: references to use proper paths
    sed -i '' 's/use crate::domain::entities/use gitai_types/g' "$file"
    sed -i '' 's/use crate::domain::errors/use crate::domain_errors/g' "$file"
    sed -i '' 's/crate::domain::errors::/crate::domain_errors::/g' "$file"
    sed -i '' 's/use crate::domain::interfaces/use crate::interfaces/g' "$file"
    sed -i '' 's/use crate::infrastructure/use crate/g' "$file"
    
    # Update gitai:: references
    sed -i '' 's/use gitai::domain::entities/use gitai_types/g' "$file"
    sed -i '' 's/use gitai::domain::errors/use crate::domain_errors/g' "$file"
    sed -i '' 's/use gitai::domain::interfaces/use crate::interfaces/g' "$file"
    
    # Add gitai_types import if needed
    if grep -q "Severity\|RiskLevel\|Finding\|BreakingChange" "$file" && ! grep -q "use gitai_types" "$file"; then
        # Add import at the beginning of the file after any existing use statements
        awk 'BEGIN{printed=0} /^use / && !printed {print "use gitai_types::*;"; printed=1} {print}' "$file" > "$file.tmp" && mv "$file.tmp" "$file"
    fi
done

echo "Import updates complete."

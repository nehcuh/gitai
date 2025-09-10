#!/bin/bash
# Update imports in gitai-analysis crate to use workspace paths

CRATE_DIR="/Users/huchen/Projects/gitai/crates/gitai-analysis"

echo "Updating imports in gitai-analysis crate..."

# Update references to gitai types
find "$CRATE_DIR/src" -name "*.rs" -type f -exec sed -i '' \
  -e 's/use crate::domain::common::/use gitai_types::/g' \
  -e 's/use crate::domain::entities::/use gitai_types::/g' \
  -e 's/use crate::domain::errors::/use gitai_types::/g' \
  -e 's/use crate::domain::\([^:]*\);/use gitai_types::\1;/g' \
  -e 's/crate::domain::common::/gitai_types::/g' \
  -e 's/crate::domain::entities::/gitai_types::/g' \
  -e 's/crate::domain::errors::/gitai_types::/g' \
  {} \;

# Update references to gitai core interfaces
find "$CRATE_DIR/src" -name "*.rs" -type f -exec sed -i '' \
  -e 's/use crate::domain::interfaces::/use gitai_core::interfaces::/g' \
  -e 's/use crate::domain::services::/use gitai_core::services::/g' \
  -e 's/crate::domain::interfaces::/gitai_core::interfaces::/g' \
  -e 's/crate::domain::services::/gitai_core::services::/g' \
  {} \;

# Update references to other modules within the main crate
find "$CRATE_DIR/src" -name "*.rs" -type f -exec sed -i '' \
  -e 's/use crate::ai::/use gitai_core::interfaces::ai::/g' \
  -e 's/use crate::config::/use gitai_core::/g' \
  -e 's/use crate::git::/use gitai_core::/g' \
  -e 's/use crate::devops::/use gitai_core::/g' \
  {} \;

# Fix any remaining crate:: references that should be local
find "$CRATE_DIR/src" -name "*.rs" -type f -exec sed -i '' \
  -e 's/use crate::tree_sitter::/use crate::tree_sitter::/g' \
  -e 's/use crate::analysis::/use crate::analysis::/g' \
  {} \;

echo "Import updates complete!"

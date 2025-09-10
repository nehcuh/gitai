#!/bin/bash
# Fix imports in gitai-analysis crate

CRATE_DIR="/Users/huchen/Projects/gitai/crates/gitai-analysis"

echo "Fixing imports in gitai-analysis crate..."

# Fix context imports - use from gitai-core
find "$CRATE_DIR/src" -name "*.rs" -type f -exec sed -i '' \
  -e 's/use crate::context::/use gitai_core::context::/g' \
  {} \;

# Fix AI client imports
find "$CRATE_DIR/src" -name "*.rs" -type f -exec sed -i '' \
  -e 's/use crate::ai::/use gitai_core::ai::/g' \
  {} \;

# Update imports in architectural_impact module
find "$CRATE_DIR/src/architectural_impact" -name "*.rs" -type f -exec sed -i '' \
  -e 's/use crate::domain::common::/use gitai_types::/g' \
  -e 's/use crate::domain::entities::/use gitai_types::/g' \
  -e 's/use crate::domain::errors::/use gitai_types::/g' \
  -e 's/use crate::domain::\([^:]*\);/use gitai_types::\1;/g' \
  -e 's/crate::domain::common::/gitai_types::/g' \
  -e 's/crate::domain::entities::/gitai_types::/g' \
  -e 's/crate::domain::errors::/gitai_types::/g' \
  {} \;

# Update imports in utils module
find "$CRATE_DIR/src/utils" -name "*.rs" -type f -exec sed -i '' \
  -e 's/use crate::domain::common::/use gitai_types::/g' \
  -e 's/use crate::domain::entities::/use gitai_types::/g' \
  -e 's/use crate::domain::errors::/use gitai_types::/g' \
  -e 's/use crate::domain::\([^:]*\);/use gitai_types::\1;/g' \
  {} \;

# Fix cross-module imports within architectural_impact
find "$CRATE_DIR/src/architectural_impact" -name "*.rs" -type f -exec sed -i '' \
  -e 's/use crate::architectural_impact::/use crate::architectural_impact::/g' \
  -e 's/use crate::tree_sitter::/use crate::tree_sitter::/g' \
  -e 's/use crate::analysis::/use crate::analysis::/g' \
  -e 's/use crate::utils::/use crate::utils::/g' \
  {} \;

# Fix imports in analysis.rs to use local architectural_impact
find "$CRATE_DIR/src" -name "analysis.rs" -type f -exec sed -i '' \
  -e 's/use crate::architectural_impact::/use crate::architectural_impact::/g' \
  {} \;

echo "Import fixes complete!"

#!/bin/bash
# GitAI Workspace Migration Script
# 用于协助将单体应用迁移到多 crate workspace 架构

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=== GitAI Workspace Migration Script ==="
echo "Project root: $PROJECT_ROOT"
echo ""

# Function to move files preserving directory structure
move_files() {
    local src_pattern="$1"
    local dest_crate="$2"
    local dest_subdir="${3:-}"
    
    echo "Moving $src_pattern to crates/$dest_crate/src/$dest_subdir"
    
    # Create destination directory if needed
    if [ -n "$dest_subdir" ]; then
        mkdir -p "crates/$dest_crate/src/$dest_subdir"
    fi
    
    # Find and move files
    find src -path "$src_pattern" -type f | while read -r file; do
        # Calculate relative path
        rel_path="${file#src/}"
        
        # Determine destination
        if [ -n "$dest_subdir" ]; then
            dest="crates/$dest_crate/src/$dest_subdir/$(basename "$file")"
        else
            dest="crates/$dest_crate/src/$rel_path"
        fi
        
        # Create destination directory
        mkdir -p "$(dirname "$dest")"
        
        # Copy file (use cp for now, can change to mv later)
        echo "  - $file -> $dest"
        cp "$file" "$dest"
    done
}

# Function to update imports in a crate
update_imports() {
    local crate="$1"
    echo "Updating imports in crates/$crate"
    
    # Common import replacements
    find "crates/$crate/src" -name "*.rs" -type f | while read -r file; do
        # Update crate:: to gitai:: for old imports
        sed -i '' 's/use crate::/use gitai::/g' "$file" 2>/dev/null || true
        
        # Update specific module paths
        sed -i '' 's/use gitai::domain::entities/use gitai_types/g' "$file" 2>/dev/null || true
        sed -i '' 's/use gitai::domain::interfaces/use gitai_core::interfaces/g' "$file" 2>/dev/null || true
        sed -i '' 's/use gitai::tree_sitter/use gitai_analysis::tree_sitter/g' "$file" 2>/dev/null || true
        sed -i '' 's/use gitai::scan/use gitai_security::scanner/g' "$file" 2>/dev/null || true
        sed -i '' 's/use gitai::metrics/use gitai_metrics/g' "$file" 2>/dev/null || true
    done
}

# Main migration steps
case "${1:-help}" in
    "prepare")
        echo "Step 1: Preparing workspace structure..."
        
        # Ensure all crate directories exist
        for crate in gitai-types gitai-core gitai-analysis gitai-security gitai-metrics gitai-cli gitai-mcp gitai-bin; do
            mkdir -p "crates/$crate/src"
        done
        
        echo "Workspace structure prepared."
        ;;
        
    "migrate-types")
        echo "Step 2: Migrating types to gitai-types..."
        
        # Move entity types
        move_files "src/domain/entities/*.rs" "gitai-types" "entities"
        
        # Move error types
        cp src/domain/errors.rs crates/gitai-types/src/errors.rs
        cp src/error.rs crates/gitai-types/src/error_core.rs
        
        echo "Types migration complete."
        ;;
        
    "migrate-core")
        echo "Step 3: Migrating core functionality..."
        
        # Move interfaces
        move_files "src/domain/interfaces/*.rs" "gitai-core" "interfaces"
        
        # Move core services
        move_files "src/domain/services/*.rs" "gitai-core" "services"
        
        # Move core modules
        cp src/config.rs crates/gitai-core/src/
        cp src/git.rs crates/gitai-core/src/
        cp src/devops.rs crates/gitai-core/src/
        
        echo "Core migration complete."
        ;;
        
    "migrate-analysis")
        echo "Step 4: Migrating analysis modules..."
        
        # Move tree-sitter modules
        move_files "src/tree_sitter/*" "gitai-analysis" "tree_sitter"
        
        # Move analysis modules
        cp src/analysis.rs crates/gitai-analysis/src/
        cp src/review/* crates/gitai-analysis/src/review/
        
        echo "Analysis migration complete."
        ;;
        
    "migrate-security")
        echo "Step 5: Migrating security modules..."
        
        # Move scan module
        cp src/scan.rs crates/gitai-security/src/scanner.rs
        cp src/security_*.rs crates/gitai-security/src/
        
        echo "Security migration complete."
        ;;
        
    "migrate-metrics")
        echo "Step 6: Migrating metrics modules..."
        
        # Move metrics modules
        move_files "src/metrics/*" "gitai-metrics" ""
        
        echo "Metrics migration complete."
        ;;
        
    "migrate-cli")
        echo "Step 7: Migrating CLI modules..."
        
        # Move CLI modules
        move_files "src/cli/*" "gitai-cli" ""
        cp src/args.rs crates/gitai-cli/src/
        
        echo "CLI migration complete."
        ;;
        
    "migrate-mcp")
        echo "Step 8: Migrating MCP modules..."
        
        # Move MCP modules
        move_files "src/mcp/*" "gitai-mcp" ""
        
        echo "MCP migration complete."
        ;;
        
    "migrate-bin")
        echo "Step 9: Setting up binary crate..."
        
        # Copy main files
        cp src/main.rs crates/gitai-bin/src/
        cp -r src/bin/* crates/gitai-bin/src/bin/ 2>/dev/null || true
        
        echo "Binary crate setup complete."
        ;;
        
    "update-imports")
        echo "Step 10: Updating imports across all crates..."
        
        for crate in gitai-types gitai-core gitai-analysis gitai-security gitai-metrics gitai-cli gitai-mcp gitai-bin; do
            update_imports "$crate"
        done
        
        echo "Import updates complete."
        ;;
        
    "verify")
        echo "Step 11: Verifying build..."
        
        # Try building each crate
        cargo check --workspace
        
        echo "Build verification complete."
        ;;
        
    "all")
        echo "Running all migration steps..."
        
        "$0" prepare
        "$0" migrate-types
        "$0" migrate-core
        "$0" migrate-analysis
        "$0" migrate-security
        "$0" migrate-metrics
        "$0" migrate-cli
        "$0" migrate-mcp
        "$0" migrate-bin
        "$0" update-imports
        "$0" verify
        
        echo "Full migration complete!"
        ;;
        
    *)
        echo "Usage: $0 {prepare|migrate-types|migrate-core|migrate-analysis|migrate-security|migrate-metrics|migrate-cli|migrate-mcp|migrate-bin|update-imports|verify|all}"
        echo ""
        echo "Steps:"
        echo "  prepare         - Create workspace directory structure"
        echo "  migrate-types   - Migrate shared types to gitai-types"
        echo "  migrate-core    - Migrate core functionality to gitai-core"
        echo "  migrate-analysis- Migrate analysis modules to gitai-analysis"
        echo "  migrate-security- Migrate security modules to gitai-security"
        echo "  migrate-metrics - Migrate metrics modules to gitai-metrics"
        echo "  migrate-cli     - Migrate CLI modules to gitai-cli"
        echo "  migrate-mcp     - Migrate MCP modules to gitai-mcp"
        echo "  migrate-bin     - Setup binary crate"
        echo "  update-imports  - Update import statements"
        echo "  verify          - Verify the build"
        echo "  all             - Run all steps"
        exit 1
        ;;
esac

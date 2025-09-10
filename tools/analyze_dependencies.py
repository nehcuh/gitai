#!/usr/bin/env python3
"""
Analyze dependencies between modules in GitAI codebase
"""

import os
import re
from collections import defaultdict
from pathlib import Path
import json

def extract_imports(file_path):
    """Extract use/import statements from a Rust file"""
    imports = []
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
        
    # Find use statements
    use_pattern = r'use\s+(crate::|gitai::|super::|self::)?([a-zA-Z0-9_:]+)'
    for match in re.finditer(use_pattern, content):
        prefix = match.group(1) or ''
        module_path = match.group(2)
        imports.append(f"{prefix}{module_path}")
        
    return imports

def categorize_module(path):
    """Categorize a module based on its path"""
    path_str = str(path)
    
    if '/domain/entities/' in path_str:
        return 'types'
    elif '/domain/interfaces/' in path_str:
        return 'core-interfaces'
    elif '/domain/services/' in path_str:
        return 'core-services'
    elif '/tree_sitter/' in path_str:
        return 'analysis'
    elif '/mcp/' in path_str:
        return 'mcp'
    elif '/cli/' in path_str:
        return 'cli'
    elif '/metrics/' in path_str:
        return 'metrics'
    elif 'scan.rs' in path_str or 'security_' in path_str:
        return 'security'
    elif 'review' in path_str or 'analysis.rs' in path_str:
        return 'analysis'
    elif any(core in path_str for core in ['config.rs', 'git.rs', 'devops.rs', 'context.rs']):
        return 'core'
    else:
        return 'other'

def analyze_dependencies(root_path):
    """Analyze dependencies across all Rust files"""
    dependencies = defaultdict(lambda: defaultdict(set))
    module_categories = {}
    
    for rust_file in Path(root_path).rglob('*.rs'):
        if 'target' in str(rust_file) or 'src.backup' in str(rust_file):
            continue
            
        relative_path = rust_file.relative_to(root_path)
        category = categorize_module(relative_path)
        module_categories[str(relative_path)] = category
        
        try:
            imports = extract_imports(rust_file)
            for imp in imports:
                # Determine dependency category
                if 'domain::entities' in imp or 'gitai_types' in imp:
                    dependencies[category]['types'].add(imp)
                elif 'domain::interfaces' in imp or 'domain::services' in imp:
                    dependencies[category]['core'].add(imp)
                elif 'tree_sitter' in imp:
                    dependencies[category]['analysis'].add(imp)
                elif 'mcp' in imp:
                    dependencies[category]['mcp'].add(imp)
                elif 'scan' in imp or 'security' in imp:
                    dependencies[category]['security'].add(imp)
                elif 'metrics' in imp:
                    dependencies[category]['metrics'].add(imp)
                elif 'cli' in imp:
                    dependencies[category]['cli'].add(imp)
        except Exception as e:
            print(f"Error processing {rust_file}: {e}")
            
    return dependencies, module_categories

def print_migration_order(dependencies):
    """Determine and print suggested migration order"""
    print("\n=== Suggested Migration Order ===\n")
    
    # Count incoming dependencies
    incoming_deps = defaultdict(int)
    for source, targets in dependencies.items():
        for target in targets:
            incoming_deps[target] += len(targets[target])
            
    # Sort by least dependencies first
    ordered = sorted(incoming_deps.items(), key=lambda x: x[1])
    
    print("1. gitai-types (no dependencies)")
    print("2. gitai-core (depends on types)")
    print("3. gitai-analysis (depends on types, core)")
    print("4. gitai-security (depends on types, core)")
    print("5. gitai-metrics (depends on types, core)")
    print("6. gitai-mcp (depends on all)")
    print("7. gitai-cli (depends on all)")
    print("8. gitai-bin (final integration)")

def main():
    root_path = Path(__file__).parent.parent
    
    print("Analyzing GitAI module dependencies...")
    print(f"Root path: {root_path}")
    
    dependencies, module_categories = analyze_dependencies(root_path / "src")
    
    # Print categorized modules
    print("\n=== Module Categories ===")
    categorized = defaultdict(list)
    for module, category in module_categories.items():
        categorized[category].append(module)
        
    for category, modules in sorted(categorized.items()):
        print(f"\n{category}:")
        for module in sorted(modules)[:10]:  # Show first 10
            print(f"  - {module}")
        if len(modules) > 10:
            print(f"  ... and {len(modules) - 10} more")
            
    # Print dependencies
    print("\n\n=== Cross-Category Dependencies ===")
    for source, targets in sorted(dependencies.items()):
        if targets:
            print(f"\n{source} depends on:")
            for target, imports in sorted(targets.items()):
                if imports and target != source:
                    print(f"  - {target}: {len(imports)} imports")
                    
    print_migration_order(dependencies)
    
    # Save to JSON for further analysis
    output = {
        'module_categories': module_categories,
        'dependencies': {k: {k2: list(v2) for k2, v2 in v.items()} for k, v in dependencies.items()}
    }
    
    with open(root_path / 'tools' / 'dependency_analysis.json', 'w') as f:
        json.dump(output, f, indent=2)
        
    print(f"\nDetailed analysis saved to tools/dependency_analysis.json")

if __name__ == "__main__":
    main()

use std::path::Path;

fn main() {
    println!("Checking Tree-sitter queries cache...");
    
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".cache"))
        .join("gitai")
        .join("tree-sitter-queries");
    
    println!("Cache directory: {:?}", cache_dir);
    
    if !cache_dir.exists() {
        println!("Cache directory does not exist!");
        println!("\nTrying alternative location...");
        
        let alt_cache_dir = dirs::home_dir().unwrap()
            .join(".cache")
            .join("gitai")
            .join("tree-sitter");
        
        println!("Alternative directory: {:?}", alt_cache_dir);
        
        if alt_cache_dir.exists() {
            check_directory(&alt_cache_dir);
        } else {
            println!("Alternative directory also does not exist!");
        }
    } else {
        check_directory(&cache_dir);
    }
}

fn check_directory(dir: &Path) {
    println!("\nChecking downloaded files in {:?}:", dir);
    
    for lang in &["rust", "java", "python", "javascript", "go", "c", "cpp", "typescript"] {
        let lang_dir = dir.join(lang);
        if lang_dir.exists() {
            println!("  {} ✓", lang);
            for file in &["highlights.scm", "locals.scm", "injections.scm", "folds.scm", "indents.scm"] {
                let file_path = lang_dir.join(file);
                if file_path.exists() {
                    if let Ok(metadata) = std::fs::metadata(&file_path) {
                        let size = metadata.len();
                        println!("    - {}: {} bytes", file, size);
                    }
                }
            }
        } else {
            println!("  {} ✗ (directory not found)", lang);
        }
    }
}

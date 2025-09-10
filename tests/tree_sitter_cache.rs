use gitai::tree_sitter::{SupportedLanguage, TreeSitterManager};

#[tokio::test]
async fn test_cache_stats_hits_and_misses() {
    // Prefer a supported language; if none available, skip
    let lang = SupportedLanguage::all()
        .into_iter()
        .find(|l| l.language().is_some());

    let Some(lang) = lang else {
        eprintln!("skipped: no tree-sitter language features enabled");
        return;
    };

    let mut mgr = TreeSitterManager::new().await.expect("manager new");

    // Prepare a minimal snippet for the chosen language
    let code = match lang {
        SupportedLanguage::Rust => "fn main() {}\n",
        SupportedLanguage::Java => "public class A { void f() {} }\n",
        SupportedLanguage::C => "int main(){return 0;}\n",
        SupportedLanguage::Cpp => "int main(){return 0;}\n",
        SupportedLanguage::Python => "def f():\n    pass\n",
        SupportedLanguage::Go => "package main\nfunc f(){}\n",
        SupportedLanguage::JavaScript => "function f(){}\n",
        SupportedLanguage::TypeScript => "function f(): void {}\n",
    };

    // Clear any prior cache to make stats deterministic
    let _ = mgr.clear_cache();

    // First analysis should miss cache
    let stats_before = mgr.cache_stats().unwrap_or_default();
    let _ = mgr
        .analyze_structure(code, lang)
        .expect("first analyze should succeed");
    let stats_after_first = mgr.cache_stats().unwrap_or_default();

    // Second analysis should hit cache
    let _ = mgr
        .analyze_structure(code, lang)
        .expect("second analyze should succeed");
    let stats_after_second = mgr.cache_stats().unwrap_or_default();

    // Validate stats progression: +1 miss on first, +1 hit on second
    assert_eq!(stats_after_first.misses, stats_before.misses + 1);
    assert_eq!(stats_after_second.hits, stats_after_first.hits + 1);
}

use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use gitai::tree_sitter::{SupportedLanguage, TreeSitterManager};
use std::sync::Arc;

fn pick_enabled_language() -> Option<SupportedLanguage> {
    SupportedLanguage::all()
        .into_iter()
        .find(|l| l.language().is_some())
}

fn lang_ext(lang: SupportedLanguage) -> &'static str {
    match lang {
        SupportedLanguage::Java => "java",
        SupportedLanguage::Rust => "rs",
        SupportedLanguage::C => "c",
        SupportedLanguage::Cpp => "cpp",
        SupportedLanguage::Python => "py",
        SupportedLanguage::Go => "go",
        SupportedLanguage::JavaScript => "js",
        SupportedLanguage::TypeScript => "ts",
    }
}

fn minimal_code_for(lang: SupportedLanguage) -> &'static str {
    match lang {
        SupportedLanguage::Java => "public class A { void f() {} }\n",
        SupportedLanguage::Rust => "fn main() {}\n",
        SupportedLanguage::C => "int main(){return 0;}\n",
        SupportedLanguage::Cpp => "int main(){return 0;}\n",
        SupportedLanguage::Python => "def f():\n    pass\n",
        SupportedLanguage::Go => "package main\nfunc f(){}\n",
        SupportedLanguage::JavaScript => "function f(){}\n",
        SupportedLanguage::TypeScript => "function f(): void {}\n",
    }
}

fn make_temp_dir() -> PathBuf {
    let mut base = std::env::temp_dir();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    base.push(format!("gitai_tree_sitter_concurrency_{ts}"));
    fs::create_dir_all(&base).expect("failed to create temp dir");
    base
}

fn generate_files(dir: &std::path::Path, lang: SupportedLanguage, count: usize) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let ext = lang_ext(lang);
    let code = minimal_code_for(lang);
    for i in 0..count {
        let path = dir.join(format!("test_{i}.{ext}"));
        fs::write(&path, code).expect("failed to write test file");
        paths.push(path);
    }
    paths
}

#[tokio::test]
async fn test_analyze_files_concurrent_throughput_basic() {
    let Some(lang) = pick_enabled_language() else {
        eprintln!("skipped: no tree-sitter language features enabled");
        return;
    };

    let dir = make_temp_dir();
    let files = generate_files(&dir, lang, 16);

    let manager = TreeSitterManager::new().await.expect("manager new");
    let results = manager
        .analyze_files_concurrent(files.clone(), Some(8))
        .await
        .expect("analyze_files_concurrent");

    // Expect at least some results; ideally equals input count
    assert!(
        !results.is_empty(),
        "expected at least one analyzed file for enabled language"
    );
    // If all succeeded, lengths should match
    if results.len() != files.len() {
        eprintln!(
            "note: not all files returned results ({} of {})",
            results.len(),
            files.len()
        );
    }

    // Sanity: analysis_time should be non-negative
    for r in results {
        assert!(r.analysis_time >= 0.0);
    }
}

#[tokio::test]
async fn test_analyze_directory_concurrent_basic() {
    let Some(lang) = pick_enabled_language() else {
        eprintln!("skipped: no tree-sitter language features enabled");
        return;
    };

    let dir = make_temp_dir();
    let file_count = 12;
    let _files = generate_files(&dir, lang, file_count);

    let manager = TreeSitterManager::new().await.expect("manager new");
    let summary = manager
        .analyze_directory_concurrent(&dir, Some(lang), Some(6))
        .await
        .expect("analyze_directory_concurrent");

    assert!(summary.total_files >= file_count);
    assert!(
        summary.language_statistics.contains_key(lang.name()),
        "missing language stats for {}",
        lang.name()
    );
}

#[tokio::test]
async fn test_analyze_directory_with_filters() {
    let Some(lang) = pick_enabled_language() else {
        eprintln!("skipped: no tree-sitter language features enabled");
        return;
    };

    let dir = make_temp_dir();
    let file_count = 10;
    let _files = generate_files(&dir, lang, file_count);

    let manager = TreeSitterManager::new().await.expect("manager new");

    // include: test_*.ext; exclude: test_1.ext
    let ext = lang_ext(lang);
    let include = [format!("test_*.{ext}")];
    let exclude = [format!("test_1.{ext}")];
    let include_refs: Vec<&str> = include.iter().map(|s| s.as_str()).collect();
    let exclude_refs: Vec<&str> = exclude.iter().map(|s| s.as_str()).collect();

    let summary = manager
        .analyze_directory_concurrent_with_filters(
            &dir,
            Some(lang),
            Some(4),
            Some(&include_refs),
            Some(&exclude_refs),
        )
        .await
        .expect("analyze_directory_concurrent_with_filters");

    assert_eq!(summary.total_files, file_count - 1);
    assert!(summary.language_statistics.contains_key(lang.name()));
}

#[tokio::test]
async fn test_concurrent_calls_safety() {
    let Some(lang) = pick_enabled_language() else {
        eprintln!("skipped: no tree-sitter language features enabled");
        return;
    };

    let dir = make_temp_dir();
    let files = generate_files(&dir, lang, 10);

    let manager = Arc::new(TreeSitterManager::new().await.expect("manager new"));

    let m1 = manager.clone();
    let m2 = manager.clone();

    let files1 = files.clone();
    let files2 = files.clone();

    let t1 = tokio::spawn(async move { m1.analyze_files_concurrent(files1, Some(4)).await });
    let t2 = tokio::spawn(async move { m2.analyze_files_concurrent(files2, Some(4)).await });

    let (r1, r2) = tokio::join!(t1, t2);

    let res1 = r1.expect("join1").expect("res1");
    let res2 = r2.expect("join2").expect("res2");

    assert!(!res1.is_empty());
    assert!(!res2.is_empty());
}

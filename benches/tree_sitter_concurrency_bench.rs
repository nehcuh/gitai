use criterion::{criterion_group, criterion_main, Criterion};
use gitai::tree_sitter::{SupportedLanguage, TreeSitterManager};
use std::path::PathBuf;
use tokio::runtime::Runtime;

fn pick_enabled_language() -> Option<SupportedLanguage> {
    SupportedLanguage::all()
        .into_iter()
        .find(|l| l.language().is_some())
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

fn generate_files(dir: &std::path::Path, lang: SupportedLanguage, count: usize) -> Vec<PathBuf> {
    let mut paths = Vec::with_capacity(count);
    let ext = match lang {
        SupportedLanguage::Java => "java",
        SupportedLanguage::Rust => "rs",
        SupportedLanguage::C => "c",
        SupportedLanguage::Cpp => "cpp",
        SupportedLanguage::Python => "py",
        SupportedLanguage::Go => "go",
        SupportedLanguage::JavaScript => "js",
        SupportedLanguage::TypeScript => "ts",
    };
    let code = minimal_code_for(lang);
    for i in 0..count {
        let path = dir.join(format!("bench_{i}.{ext}"));
        std::fs::write(&path, code).expect("failed to write bench file");
        paths.push(path);
    }
    paths
}

fn bench_tree_sitter_concurrency(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");

    if let Some(lang) = pick_enabled_language() {
        let file_counts = [32usize, 64, 128];
        let worker_counts = [1usize, 2, 4, 8, 16];

        for &fc in &file_counts {
            let tmp = tempfile::tempdir().expect("temp dir");
            let dir = tmp.path().to_path_buf();
            let files = generate_files(&dir, lang, fc);

            for &wc in &worker_counts {
                let bench_id =
                    format!("tree_sitter_analyze_files_concurrent_{fc}files_{wc}workers");
                let files_clone = files.clone();
                c.bench_function(&bench_id, |b| {
                    b.to_async(&rt).iter(|| async {
                        let manager = TreeSitterManager::new().await.expect("manager new");
                        let _ = manager
                            .analyze_files_concurrent(files_clone.clone(), Some(wc))
                            .await
                            .expect("analyze_files_concurrent");
                    })
                });
            }
        }
    } else {
        c.bench_function("tree_sitter_concurrency_skipped", |b| b.iter(|| {}));
    }
}

criterion_group!(benches, bench_tree_sitter_concurrency);
criterion_main!(benches);

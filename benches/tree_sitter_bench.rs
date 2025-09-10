use criterion::{criterion_group, criterion_main, Criterion};
use gitai::tree_sitter::{SupportedLanguage, TreeSitterManager};
use tokio::runtime::Runtime;

fn pick_enabled_language() -> Option<SupportedLanguage> {
    SupportedLanguage::all()
        .into_iter()
        .find(|l| l.language().is_some())
}

fn sample_code(lang: SupportedLanguage) -> &'static str {
    match lang {
        SupportedLanguage::Java => "public class A { void f(){} }\n",
        SupportedLanguage::Rust => "fn main() {}\n",
        SupportedLanguage::C => "int main(){return 0;}\n",
        SupportedLanguage::Cpp => "int main(){return 0;}\n",
        SupportedLanguage::Python => "def f():\n    pass\n",
        SupportedLanguage::Go => "package main\nfunc f(){}\n",
        SupportedLanguage::JavaScript => "function f(){}\n",
        SupportedLanguage::TypeScript => "function f(): void {}\n",
    }
}

fn bench_tree_sitter_analyze(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");

    if let Some(lang) = pick_enabled_language() {
        let code = sample_code(lang);
        c.bench_function("tree_sitter_analyze_structure", |b| {
            b.to_async(&rt).iter(|| async {
                let mut mgr = TreeSitterManager::new().await.expect("manager new");
                let _ = mgr.analyze_structure(code, lang).expect("analyze");
            })
        });
    } else {
        // No enabled tree-sitter language; keep a no-op benchmark so CI passes
        c.bench_function("tree_sitter_analyze_structure_skipped", |b| b.iter(|| {}));
    }
}

criterion_group!(benches, bench_tree_sitter_analyze);
criterion_main!(benches);

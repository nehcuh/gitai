// build.rs
use reqwest::StatusCode;
use reqwest::blocking::get;
use std::{env, fs, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // OUT_DIR 是 Cargo 在编译期提供的环境变量
    // Use project root for queries directory
    // `CARGO_MANIFEST_DIR` points to the crate root where Cargo.toml lives
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let base = Path::new(&manifest_dir).join("queries");
    // 要拉取的各语言仓库和路径映射
    let repos = [
        ("rust", "tree-sitter/tree-sitter-rust"),
        ("javascript", "tree-sitter/tree-sitter-javascript"),
        ("python", "tree-sitter/tree-sitter-python"),
        ("go", "tree-sitter/tree-sitter-go"),
        ("java", "tree-sitter/tree-sitter-java"),
        ("c", "tree-sitter/tree-sitter-c"),
        ("cpp", "tree-sitter/tree-sitter-cpp"),
    ];
    // 官方仓库里常见的 query 文件
    let files = ["highlights.scm", "injections.scm", "locals.scm"];

    for (lang, repo) in repos {
        let target_dir = base.join(lang);
        fs::create_dir_all(&target_dir)?;
        for file in &files {
            // 构造 raw URL
            let url = format!(
                "https://raw.githubusercontent.com/{repo}/master/queries/{file}",
                repo = repo,
                file = file
            );
            let resp = get(&url)?;
            // 404、404 will be skipped
            if resp.status() == StatusCode::OK {
                let text = resp.text()?;
                let dest = target_dir.join(file);
                fs::write(&dest, text)?;
                println!("cargo:rerun-if-changed={}", url);
            }
        }
    }
    Ok(())
}

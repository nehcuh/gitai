use reqwest::StatusCode;
use std::{env, fs, path::Path};

/// 创建备用查询文件，当网络不可用时使用
fn create_fallback_query(dest: &Path, lang: &str, file: &str) {
    let fallback_content = match (lang, file) {
        (_, "injections.scm") | (_, "locals.scm") => {
            // 对于 injections 和 locals，如果下载失败，创建一个空文件
            ""
        }
        ("rust", "highlights.scm") => {
            // 提供基础的 Rust 高亮查询
            r#"
(identifier) @variable
(function_item name: (identifier) @function)
(struct_item name: (type_identifier) @type)
(enum_item name: (type_identifier) @type)
(impl_item type: (type_identifier) @type)
(use_declaration) @keyword
(mod_item name: (identifier) @module)
"#
        }
        ("java", "highlights.scm") => {
            // 提供基础的 Java 高亮查询
            r#"
(identifier) @variable
(method_declaration name: (identifier) @function)
(class_declaration name: (identifier) @type)
(interface_declaration name: (identifier) @type)
(import_declaration) @keyword
(package_declaration) @keyword
"#
        }
        ("python", "highlights.scm") => {
            // 提供基础的 Python 高亮查询
            r#"
(identifier) @variable
(function_definition name: (identifier) @function)
(class_definition name: (identifier) @type)
(import_statement) @keyword
(import_from_statement) @keyword
"#
        }
        ("cpp", "highlights.scm") => {
            // 提供兼容的 C++ 高亮查询，避免模块相关节点
            r#"
(identifier) @variable
(function_declarator declarator: (identifier) @function)
(call_expression function: (identifier) @function)
(type_identifier) @type
(primitive_type) @type.builtin
(number_literal) @number
(string_literal) @string
"#
        }
        ("c", "highlights.scm") => {
            // 提供基础的 C 高亮查询
            r#"
(identifier) @variable
(function_declarator declarator: (identifier) @function)
(call_expression function: (identifier) @function)
(type_identifier) @type
(primitive_type) @type.builtin
(number_literal) @number
(string_literal) @string
"#
        }
        _ => {
            // 通用的最小查询
            r#"
(identifier) @variable
"#
        }
    };
    
    if let Err(e) = fs::write(dest, fallback_content.trim()) {
        println!("cargo:warning=创建备用查询文件失败 {}: {}", dest.display(), e);
    } else {
        println!("cargo:warning=创建备用查询文件: {}", dest.display());
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            let dest = target_dir.join(file);
            
            // 首先检查本地文件是否已存在
            if dest.exists() {
                println!("cargo:warning=本地文件已存在，跳过下载: {}", dest.display());
                println!("cargo:rerun-if-changed={}", dest.display());
                continue;
            }
            
            // 构造 raw URL
            let url = format!(
                "https://raw.githubusercontent.com/{repo}/master/queries/{file}",
                repo = repo,
                file = file
            );
            
            // 尝试网络请求，失败时给出警告但不中断构建
            match reqwest::get(&url).await {
                Ok(resp) => {
                    if resp.status() == StatusCode::OK {
                        match resp.text().await {
                            Ok(text) => {
                                if let Err(e) = fs::write(&dest, text) {
                                    println!("cargo:warning=写入文件失败 {}: {}", dest.display(), e);
                                } else {
                                    println!("cargo:warning=成功下载: {}", dest.display());
                                    println!("cargo:rerun-if-changed={}", url);
                                }
                            }
                            Err(e) => {
                                println!("cargo:warning=读取响应内容失败 {}: {}", url, e);
                            }
                        }
                    } else {
                        println!("cargo:warning=HTTP请求失败 {} (状态码: {})", url, resp.status());
                        create_fallback_query(&dest, lang, file);
                    }
                }
                Err(e) => {
                    println!("cargo:warning=网络请求失败 {}: {}. 如果本地存在备用文件，构建将继续。", url, e);
                    // 检查是否有备用的内嵌查询可以使用
                    create_fallback_query(&dest, lang, file);
                }
            }
        }
    }
    println!("cargo:warning=Tree-sitter 查询文件准备完成");
    Ok(())
}

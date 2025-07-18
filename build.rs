use futures::future::join_all;
use reqwest::{Client, StatusCode};
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::Semaphore;

// 每隔 10 天需要更新一次
const CACHE_EXPIRATION: Duration = Duration::from_secs(864000);

// 开源 ast-grep 代码扫描规则
const AST_GREP_SCAN_RULES: &str = "https://github.com/coderabbitai/ast-grep-essentials.git";

#[derive(Debug)]
pub enum DownloadError {
    NetworkError(reqwest::Error),
    HttpStatus(StatusCode),
    FileWrite(io::Error),
    InvalidContent,
    Timeout,
}

impl From<reqwest::Error> for DownloadError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            DownloadError::Timeout
        } else {
            DownloadError::NetworkError(err)
        }
    }
}

impl From<io::Error> for DownloadError {
    fn from(err: io::Error) -> Self {
        DownloadError::FileWrite(err)
    }
}

struct DownloadTask {
    lang: String,
    file: String,
    url: String,
    dest: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 检查基础缓存目录
    let base_cache_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache")
        .join("gitai");

    // 查询文件目录
    let user_cache_dir = base_cache_dir.join("queries");
    fs::create_dir_all(&user_cache_dir)?;

    // 扫描规则目录
    let scan_rules_dir = base_cache_dir.join("scan-rules");
    fs::create_dir_all(&scan_rules_dir)?;

    // 配置 http 客户端
    let client = Arc::new(
        Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(5)
            .user_agent("gitai-build/1.0")
            .build()?,
    );

    // 定义仓库和文件
    let repos = [
        ("rust", "tree-sitter/tree-sitter-rust"),
        ("javascript", "tree-sitter/tree-sitter-javascript"),
        ("typescript", "tree-sitter/tree-sitter-typescript"),
        ("python", "tree-sitter/tree-sitter-python"),
        ("go", "tree-sitter/tree-sitter-go"),
        ("java", "tree-sitter/tree-sitter-java"),
        ("c", "tree-sitter/tree-sitter-c"),
        ("cpp", "tree-sitter/tree-sitter-cpp"),
        ("bash", "tree-sitter/tree-sitter-bash"),
        ("css", "tree-sitter/tree-sitter-css"),
        ("haskell", "tree-sitter/tree-sitter-haskell"),
        ("html", "tree-sitter/tree-sitter-html"),
        ("jsdoc", "tree-sitter/tree-sitter-jsdoc"),
        ("json", "tree-sitter/tree-sitter-json"),
        ("julia", "tree-sitter/tree-sitter-julia"),
        ("php", "tree-sitter/tree-sitter-php"),
        ("ql", "tree-sitter/tree-sitter-ql"),
        ("regex", "tree-sitter/tree-sitter-regex"),
        ("ruby", "tree-sitter/tree-sitter-ruby"),
        ("scala", "tree-sitter/tree-sitter-scala"),
    ];

    let files = ["highlights.scm", "injections.scm", "locals.scm"];

    // 创建下载任务
    let mut tasks = Vec::new();
    for (lang, repo) in repos {
        let user_target_dir = user_cache_dir.join(lang);
        fs::create_dir_all(&user_target_dir)?;

        for file in files {
            let dest = user_target_dir.join(file);

            if !should_update_file(&dest) {
                println!(
                    "cargo::warning=文件已存在且未过期，跳过: {}",
                    dest.display()
                );
                println!("cargo::rerun-if-changed={}", dest.display());
                continue;
            }

            let url = format!(
                "https://raw.githubusercontent.com/{repo}/master/queries/{file}",
                repo = repo,
                file = file
            );

            tasks.push(DownloadTask {
                lang: lang.to_string(),
                file: file.to_string(),
                url,
                dest,
            });
        }
    }

    // 使用信号量限制并发数
    let semaphore = Arc::new(Semaphore::new(5));

    // 并发执行下载任务
    let mut handles = Vec::new();
    for task in tasks {
        let client = client.clone();
        let semaphore = semaphore.clone();

        handles.push(tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            download_file(&client, task).await
        }));
    }

    // 收集结果
    let results = join_all(handles).await;

    let mut success_count = 0;
    let mut error_count = 0;

    for result in results {
        match result {
            Ok(_) => success_count += 1,
            Err(e) => {
                error_count += 1;
                println!("cargo:warning=任务执行失败: {}", e);
            }
        }
    }

    println!(
        "cargo:warning=下载完成: 成功 {} 个, 失败 {} 个",
        success_count, error_count
    );

    // 下载扫描规则
    download_scan_rules(&scan_rules_dir)?;

    Ok(())
}

async fn download_file(client: &Client, task: DownloadTask) -> Result<(), DownloadError> {
    println!(
        "cargo:warning=开始下载: {} -> {}",
        task.url,
        task.dest.display()
    );

    match client.get(&task.url).send().await {
        Ok(resp) => {
            if resp.status() == StatusCode::OK {
                match resp.text().await {
                    Ok(text) => {
                        if text.trim().is_empty() {
                            return Err(DownloadError::InvalidContent);
                        }

                        if let Some(parent) = task.dest.parent() {
                            fs::create_dir_all(parent)?;
                        }

                        fs::write(&task.dest, text).map_err(DownloadError::FileWrite)?;

                        println!("cargo:warning=成功下载: {}", task.dest.display());
                        println!("cargo:rerun-if-changed={}", task.url);

                        Ok(())
                    }
                    Err(e) => Err(DownloadError::NetworkError(e.into())),
                }
            } else if resp.status() == StatusCode::NOT_FOUND {
                create_fallback_query(&task.dest, &task.lang, &task.file);
                Ok(())
            } else {
                Err(DownloadError::HttpStatus(resp.status()))
            }
        }
        Err(e) => {
            // 网络错误使用回退方案
            println!("cargo::warning=网络错误，使用回退方案: {}", e);
            create_fallback_query(&task.dest, &task.lang, &task.file);
            Ok(())
        }
    }
}

fn should_update_file(path: &Path) -> bool {
    if !path.exists() {
        return true;
    }

    // 检查文件最后修改时间
    if let Ok(metadata) = fs::metadata(path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(age) = SystemTime::now().duration_since(modified) {
                return age > CACHE_EXPIRATION;
            }
        }
    }

    // 检查文件大小 (空文件需要更新)
    if let Ok(size) = fs::metadata(path).map(|m| m.len()) {
        if size == 0 {
            return true;
        }
    }

    false
}

fn create_fallback_query(dest: &Path, lang: &str, file: &str) {
    let fallback_content = match (lang, file) {
        (_, "injections.scm") | (_, "locals.scm") => "",
        ("rust", "highlights.scm") => include_str!("fallback_queries/rust/highlights.scm"),
        ("javascript", "highlights.scm") => {
            include_str!("fallback_queries/javascript/highlights.scm")
        }
        _ => {
            // 通用回退查询
            match lang {
                "python" => {
                    "(identifier) @variable\n(string) @string\n(number) @number\n(function_definition name: (identifier) @function)\n(class_definition name: (identifier) @type)\n(comment) @comment\n"
                }
                "go" => {
                    "(identifier) @variable\n(string_literal) @string\n(int_literal) @number\n(function_declaration name: (identifier) @function)\n(type_spec name: (type_identifier) @type)\n(comment) @comment\n"
                }
                "java" => {
                    "(identifier) @variable\n(string_literal) @string\n(decimal_integer_literal) @number\n(method_declaration name: (identifier) @function)\n(class_declaration name: (identifier) @type)\n(line_comment) @comment\n"
                }
                _ => {
                    "(identifier) @variable\n(string) @string\n(number) @number\n(function) @function\n(type) @type\n(comment) @comment\n"
                }
            }
        }
    };

    if let Some(parent) = dest.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Err(e) = fs::write(dest, fallback_content.trim()) {
        println!("cargo::warning=创建回退文件失败 {}: {}", dest.display(), e);
    } else {
        println!("cargo::warning=创建回退查询文件: {}", dest.display());
    }
}

fn download_scan_rules(scan_rules_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // 检查是否已经存在规则目录
    if scan_rules_dir.exists() && scan_rules_dir.join(".git").exists() {
        println!("cargo:warning=更新现有的扫描规则库...");

        // 获取远程更新
        let status = Command::new("git")
            .arg("-C")
            .arg(scan_rules_dir)
            .arg("fetch")
            .arg("--depth=1")
            .arg("origin")
            .status()?;

        if status.success() {
            // 检查是否需要更新
            let output = Command::new("git")
                .arg("-C")
                .arg(scan_rules_dir)
                .arg("rev-list")
                .arg("--count")
                .arg("HEAD..origin/main")
                .output()?;

            let commits_behind = String::from_utf8_lossy(&output.stdout);
            if commits_behind.trim() != "0" {
                println!("cargo:warning=发现更新，正在拉取...");
                Command::new("git")
                    .arg("-C")
                    .arg(scan_rules_dir)
                    .arg("pull")
                    .arg("--ff-only")
                    .status()?;
                println!("cargo:warning=扫描规则更新成功");
            } else {
                println!("cargo:warning=扫描规则已是最新版本");
            }
        } else {
            println!("cargo:warning=fetch 失败，使用现有规则");
        }
    } else {
        println!("cargo:warning=克隆扫描规则仓库...");
        // 如果目录存在但不是git仓库，先删除
        if scan_rules_dir.exists() {
            let _ = fs::remove_dir_all(scan_rules_dir);
        }

        // 使用浅克隆减少下载量
        let parent_dir = scan_rules_dir
            .parent()
            .ok_or("Invalid scan rules directory path")?;

        let status = Command::new("git")
            .arg("clone")
            .arg("--depth=1")
            .arg("--branch=main")
            .arg(AST_GREP_SCAN_RULES)
            .arg(scan_rules_dir.file_name().unwrap())
            .current_dir(parent_dir)
            .status()?;

        if status.success() {
            println!("cargo:warning=扫描规则克隆成功");
        } else {
            println!("cargo:warning=扫描规则克隆失败，创建基础结构");
            fs::create_dir_all(scan_rules_dir.join("rules"))?;
        }
    }
    Ok(())
}

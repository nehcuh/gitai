use std::path::{Path, PathBuf};

/// 缓存文件更新间隔 (10天)
const CACHE_EXPIRATION: std::time::Duration = std::time::Duration::from_secs(10 * 24 * 60 * 60);

/// 最大并发下载数量
const MAX_CONCURRENT_DOWNLOADS: usize = 5;

/// 扫描规则下载地址
const SCAN_RULES_URL: &str = "https://github.com/coderabbitai/ast-grep-essentials.git";

/// 开源 tree-sitter-lang 规则下载模板
const TREE_SITTER_URL: &str = "https://raw.githubusercontent.com/REPO/master/queries/FILE";

/// 项目版本号
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 构建脚本日志记录宏
macro_rules! build_log {
    (info, $($arg:tt)*) => {
        println!("cargo:warning=[INFO] {}", format!($($arg)*));
    };
    (warn, $($arg:tt)*) => {
        println!("cargo:warning=[WARN] {}", format!($($arg)*));
    };
    (error, $($arg:tt)*) => {
        println!("cargo:warning=[ERROR] {}", format!($($arg)*));
    };
    (debug, $($arg:tt)*) => {
        println!("cargo:warning=[DEBUG] {}", format!($($arg)*));
    };
}

/// 下载过程中可能出现的错误类型
#[derive(Debug)]
enum DownloadError {
    NetworkError(reqwest::Error),
    HttpStatus(reqwest::StatusCode),
    FileWrite(std::io::Error),
    InvalidContent,
    Timeout,
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadError::NetworkError(err) => write!(f, "网络错误: {}", err),
            DownloadError::HttpStatus(status) => write!(f, "HTTP状态错误: {}", status),
            DownloadError::FileWrite(err) => write!(f, "文件写入错误: {}", err),
            DownloadError::InvalidContent => write!(f, "无效内容"),
            DownloadError::Timeout => write!(f, "请求超时"),
        }
    }
}

impl std::error::Error for DownloadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DownloadError::NetworkError(err) => Some(err),
            DownloadError::FileWrite(err) => Some(err),
            _ => None,
        }
    }
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

impl From<std::io::Error> for DownloadError {
    fn from(err: std::io::Error) -> Self {
        DownloadError::FileWrite(err)
    }
}

/// 下载任务信息
#[derive(Debug)]
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
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cache")
        .join("gitai");

    // 配置 AST queries 目录
    let ast_queries_cache_dir = base_cache_dir.join("queries");
    std::fs::create_dir_all(&ast_queries_cache_dir)?;

    // 配置代码扫描规则目录
    let scan_rules_cache_dir = base_cache_dir.join("scan-rules");
    std::fs::create_dir_all(&scan_rules_cache_dir)?;

    // 配置 http 客户端
    let client = std::sync::Arc::new(
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .pool_max_idle_per_host(5)
            .user_agent(&format!("gitai-build/{}", VERSION))
            .build()?,
    );

    // AST 支持的编程语言
    let langs = [
        ("java", "tree-sitter-java"),
        ("rust", "tree-sitter-rust"),
        ("c", "tree-sitter-c"),
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
    let tasks = create_download_tasks(&langs, &files, &ast_queries_cache_dir)?;

    // 使用信号量限制并发数
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_DOWNLOADS));

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
    let results = futures::future::join_all(handles).await;

    let mut success_counts = 0;
    let mut error_counts = 0;

    for result in results {
        match result {
            Ok(_) => success_counts += 1,
            Err(e) => {
                error_counts += 1;
                build_log!(error, "任务执行失败: {}", e);
            }
        }
    }

    build_log!(
        info,
        "下载完成: 成功 {} 个, 失败 {} 个",
        success_counts,
        error_counts
    );

    download_scan_rules(&scan_rules_cache_dir)?;

    Ok(())
}

/// 创建下载任务列表
fn create_download_tasks(
    langs: &[(&str, &str)],
    files: &[&str],
    ast_queries_cache_dir: &Path,
) -> Result<Vec<DownloadTask>, Box<dyn std::error::Error>> {
    let mut tasks = Vec::new();

    for (lang, repo) in langs {
        let target_dir = ast_queries_cache_dir.join(lang);
        std::fs::create_dir_all(&target_dir)?;

        for file in files {
            let target_file = target_dir.join(file);

            if !should_update_file(&target_file) {
                build_log!(debug, "文件未到更新时间，跳过: {:?}", target_file);
                continue;
            }

            let url = TREE_SITTER_URL.replace("REPO", repo).replace("FILE", file);
            tasks.push(DownloadTask {
                lang: lang.to_string(),
                file: file.to_string(),
                url,
                dest: target_file,
            });
        }
    }

    Ok(tasks)
}

/// 下载和管理代码扫描规则
fn download_scan_rules(scan_rules_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // 检查代码扫描规则目录是否已经存在
    if scan_rules_dir.exists() && scan_rules_dir.join(".git").exists() {
        build_log!(info, "更新现有的扫描规则库...");

        // 获取远程更新
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(scan_rules_dir)
            .arg("fetch")
            .arg("--depth=1")
            .arg("origin")
            .status()?;

        if status.success() {
            // 检查是否需要更新
            let output = std::process::Command::new("git")
                .arg("-C")
                .arg(scan_rules_dir)
                .arg("rev-list")
                .arg("--count")
                .arg("HEAD..origin/main")
                .output()?;
            let commits_behind = String::from_utf8_lossy(&output.stdout);
            if commits_behind.trim() != "0" {
                build_log!(info, "发现更新，正在拉取...");
                std::process::Command::new("git")
                    .arg("-C")
                    .arg(scan_rules_dir)
                    .arg("pull")
                    .arg("--ff-only")
                    .status()?;
                build_log!(info, "扫描规则更新成功");
            } else {
                build_log!(info, "扫描规则已是最新版本");
            }
        }
    } else {
        build_log!(info, "克隆扫描规则仓库...");
        // 如果目录存在但是不是 git 仓库，先删除
        if scan_rules_dir.exists() {
            let _ = std::fs::remove_dir_all(scan_rules_dir);
        }

        // 使用浅克隆减少下载量
        let parent_dir = scan_rules_dir
            .parent()
            .ok_or("Invalid scan rules directory path")?;

        let status = std::process::Command::new("git")
            .arg("clone")
            .arg("--depth=1")
            .arg("--branch=main")
            .arg(SCAN_RULES_URL)
            .arg(scan_rules_dir.file_name().unwrap())
            .current_dir(parent_dir)
            .status()?;

        if status.success() {
            build_log!(info, "扫描规则克隆成功");
        } else {
            panic!("无法下载代码扫描规则")
        }
    }
    Ok(())
}

/// 下载单个文件
async fn download_file(client: &reqwest::Client, task: DownloadTask) -> Result<(), DownloadError> {
    build_log!(info, "开始下载: {} -> {}", task.url, task.dest.display());

    match client.get(&task.url).send().await {
        Ok(resp) => {
            if resp.status() == reqwest::StatusCode::OK {
                match resp.text().await {
                    Ok(text) => {
                        if text.trim().is_empty() {
                            return Err(DownloadError::InvalidContent);
                        }

                        if let Some(parent) = task.dest.parent() {
                            std::fs::create_dir_all(parent)?;
                        }

                        std::fs::write(&task.dest, text).map_err(DownloadError::FileWrite)?;
                    }
                    Err(e) => return Err(DownloadError::NetworkError(e)),
                }
            } else if resp.status() == reqwest::StatusCode::NOT_FOUND {
                create_fallback_query(&task.dest, &task.lang, &task.file);
            } else {
                return Err(DownloadError::HttpStatus(resp.status()));
            }
        }
        Err(e) => {
            // 网络错误
            return Err(DownloadError::NetworkError(e));
        }
    }
    Ok(())
}

/// 检查文件是否需要更新
fn should_update_file(file: &Path) -> bool {
    // 文件不存在需要更新
    if !file.exists() {
        return true;
    }

    // 检查文件最后修改时间
    if let Ok(metadata) = std::fs::metadata(file) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(age) = std::time::SystemTime::now().duration_since(modified) {
                return age > CACHE_EXPIRATION;
            }
        }
    }

    false
}

/// 创建回退查询文件
fn create_fallback_query(dest: &Path, lang: &str, file: &str) {
    let fallback_content = match (lang, file) {
        (_, "injections.scm") | (_, "locals.scm") => "",
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
        let _ = std::fs::create_dir_all(parent);
    }

    if let Err(e) = std::fs::write(dest, fallback_content.trim()) {
        build_log!(error, "创建回退文件失败 {}: {}", dest.display(), e);
    } else {
        build_log!(info, "创建回退查询文件: {}", dest.display());
    }
}

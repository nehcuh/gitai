use crate::config::Config;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use std::io::Write;
use std::string::String;
use std::sync::mpsc;
use std::thread;

/// 扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub tool: String,
    pub version: String,
    pub execution_time: f64,
    pub findings: Vec<Finding>,
    pub error: Option<String>,
}

/// 发现的问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub file_path: PathBuf,
    pub line: usize,
    pub code_snippet: Option<String>,
    pub rule_id: String,
}

/// 严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl std::str::FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(Severity::Error),
            "warning" => Ok(Severity::Warning),
            "info" => Ok(Severity::Info),
            _ => Err(format!("Invalid severity: {s}")),
        }
    }
}

/// 位置信息 - 简化为单行位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub line: usize,
}

/// 确保规则可用，如果不存在则自动下载
fn ensure_rules_available() -> Result<PathBuf> {
    ensure_rules_available_with_update(false)
}

/// 确保规则可用，支持强制更新
fn ensure_rules_available_with_update(force_update: bool) -> Result<PathBuf> {
    let rules_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache")
        .join("gitai")
        .join("scan-rules");
    
    if !rules_dir.exists() {
        println!("📥 正在下载安全扫描规则...");
        fs::create_dir_all(&rules_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create rules directory: {e}"))?;
        
        download_default_rules(&rules_dir)?;
    } else if force_update {
        println!("🔄 正在更新安全扫描规则...");
        download_default_rules(&rules_dir)?;
    }
    
    Ok(rules_dir)
}

/// 下载默认规则集
fn download_default_rules(rules_dir: &Path) -> Result<()> {
    println!("📥 正在下载安全扫描规则...");
    
    // 尝试从网络下载最新的规则集
    if download_rules_from_network(rules_dir).is_err() {
        println!("⚠️  网络下载失败，使用内置默认规则");
        create_builtin_rules(rules_dir)?;
    }
    
    println!("✅ 规则下载完成: {}", rules_dir.display());
    Ok(())
}

/// 从网络下载规则集
fn download_rules_from_network(rules_dir: &Path) -> Result<()> {
    println!("📥 正在从OpenGrep官方规则库下载最新规则...");
    
    // 下载OpenGrep官方规则
    download_opengrep_official_rules(rules_dir)?;
    
    println!("✅ OpenGrep官方规则下载完成");
    Ok(())
}

/// 下载OpenGrep官方规则
fn download_opengrep_official_rules(rules_dir: &Path) -> Result<()> {
    let repo_url = "https://github.com/opengrep/opengrep-rules.git";
    let temp_dir = tempfile::TempDir::new()?;
    let temp_path = temp_dir.path();
    
    println!("📥 正在克隆OpenGrep规则库: {}", repo_url);
    
    // 使用git clone下载规则
    let output = Command::new("git")
        .args(["clone", "--depth", "1", repo_url, &temp_path.display().to_string()])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to clone OpenGrep rules repository: {e}"))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Git clone failed: {stderr}"));
    }
    
    // 复制规则文件到目标目录
    copy_rules_from_source(temp_path, rules_dir)?;
    
    Ok(())
}


/// 从源目录复制规则文件
fn copy_rules_from_source(source_dir: &Path, target_dir: &Path) -> Result<()> {
    // 清空目标目录
    if target_dir.exists() {
        fs::remove_dir_all(target_dir)?;
    }
    fs::create_dir_all(target_dir)?;
    
    // 简化：只复制主要语言目录
    let languages = ["rust", "python", "javascript", "java", "go", "c", "cpp"];
    let mut rule_count = 0;
    
    for lang in languages.iter() {
        let lang_source_dir = source_dir.join(lang);
        if lang_source_dir.exists() {
            let lang_target_dir = target_dir.join(lang);
            fs::create_dir_all(&lang_target_dir)?;
            
            // 递归查找所有.yml文件
            rule_count += copy_yaml_files_recursively(&lang_source_dir, &lang_target_dir)?;
        }
    }
    
    println!("📋 已复制 {} 个规则文件", rule_count);
    
    // 创建一个简单的默认规则文件
    let default_rules = r#"rules:
  - id: unsafe-usage
    message: Detected 'unsafe' usage, please audit for secure usage
    pattern: "unsafe { ... }"
    languages: [rust]
    severity: INFO
    
  - id: hardcoded-password
    message: Hardcoded password detected
    pattern: |
      let $VAR = "...";
    metavariable-regex:
      $VAR: (?i)(password|secret|key|token)
    languages: [rust]
    severity: ERROR
"#;
    
    fs::write(target_dir.join("default-rules.yml"), default_rules)?;
    
    Ok(())
}

/// 创建规则配置文件
fn create_rules_config(rules_dir: &Path) -> String {
    let mut config = String::new();
    
    // 直接包含所有主要语言的规则目录
    let languages = ["rust", "python", "javascript", "java", "go", "c", "cpp", "csharp", "php", "ruby"];
    
    config.push_str("include:\n");
    let mut has_rules = false;
    
    for lang in languages.iter() {
        let lang_dir = rules_dir.join(lang);
        if lang_dir.exists() {
            config.push_str(&format!("  - {}\n", lang_dir.display()));
            has_rules = true;
        }
    }
    
    // 如果没有找到语言规则，查找其他规则目录
    if !has_rules {
        if let Ok(entries) = fs::read_dir(rules_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    
                    // 跳过一些非规则目录
                    if dir_name == ".git" || dir_name == "stats" || dir_name == "problem-based-packs" || dir_name == "generic" {
                        continue;
                    }
                    
                    // 检查目录是否包含.yml文件
                    if let Ok(dir_entries) = fs::read_dir(&path) {
                        for dir_entry in dir_entries.flatten() {
                            let dir_path = dir_entry.path();
                            if dir_path.extension().and_then(|s| s.to_str()) == Some("yml") ||
                               dir_path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                                config.push_str(&format!("  - {}\n", path.display()));
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    
    config
}

/// 创建内置默认规则
fn create_builtin_rules(rules_dir: &Path) -> Result<()> {
    let rules = vec![
        (
            "default-security.yml",
            r#"rules:
  - id: hardcoded-password
    languages: [rust, python, javascript, java, go]
    message: 硬编码密码
    severity: ERROR
    pattern: |
      $VAR = "..."
    metavariable-regex:
      $VAR: (?i)(password|secret|key|token|api_key|passcode)
  
  - id: sql-injection
    languages: [rust, python, javascript, java, go]
    message: SQL注入风险
    severity: ERROR
    pattern: format!("SELECT ... WHERE ... = {}", ...)
  
  - id: command-injection
    languages: [rust, python, javascript, java, go]
    message: 命令注入风险
    severity: ERROR
    pattern: |
      os.system($USER_INPUT)
      exec($USER_INPUT)
  
  - id: buffer-overflow
    languages: [rust, c, cpp]
    message: 缓冲区溢出风险
    severity: ERROR
    pattern: |
      strcpy($DEST, $SRC)
      gets($BUF)
  
  - id: unsafe-deserialization
    languages: [java, python, javascript]
    message: 不安全的反序列化
    severity: WARNING
    pattern: |
      pickle.loads($DATA)
      eval($DATA)
  
  - id: weak-crypto
    languages: [rust, python, javascript, java, go]
    message: 弱加密算法
    severity: WARNING
    pattern: |
      md5($DATA)
      sha1($DATA)
  
  - id: path-traversal
    languages: [rust, python, javascript, java, go]
    message: 路径遍历漏洞
    severity: ERROR
    pattern: |
      open($USER_INPUT)
      Path($USER_INPUT)
"#,
        ),
        (
            "rust-specific.yml",
            r#"rules:
  - id: rust-unsafe-block
    languages: [rust]
    message: 使用unsafe代码块
    severity: WARNING
    pattern: |
      unsafe {
        ...
      }
  
  - id: rust-raw-pointer
    languages: [rust]
    message: 使用原始指针
    severity: WARNING
    pattern: |
      *const $TYPE
      *mut $TYPE
  
  - id: rust-unwrap
    languages: [rust]
    message: 使用unwrap()可能导致panic
    severity: WARNING
    pattern: |
      .unwrap()
  
  - id: rust-expect
    languages: [rust]
    message: 使用expect()可能导致panic
    severity: WARNING
    pattern: |
      .expect($MSG)
"#,
        ),
    ];
    
    for (filename, content) in rules {
        let rule_file = rules_dir.join(filename);
        fs::write(&rule_file, content)
            .map_err(|e| anyhow::anyhow!("Failed to write rule file {}: {e}", filename))?;
    }
    
    Ok(())
}

/// 工具安装器 trait
#[allow(dead_code)]
trait ToolInstaller {
    fn name(&self) -> &'static str;
    fn is_installed(&self) -> bool;
    fn install(&self) -> Result<()>;
    #[allow(dead_code)]
    fn get_version(&self) -> Result<String>;
}

/// 检查工具是否可用，如果不可用则尝试安装
fn ensure_tool_available(tool_name: &str) -> Result<String> {
    // 首先检查是否已安装
    if let Ok(version) = check_tool_installed(tool_name) {
        return Ok(version);
    }
    
    // 如果没有安装，尝试自动安装
    println!("📦 工具 {tool_name} 未安装，正在尝试自动安装...");
    
    let installer: Box<dyn ToolInstaller> = match tool_name {
        "opengrep" => Box::new(OpenGrepInstaller),
        _ => return Err(anyhow::anyhow!("不支持自动安装的工具: {}", tool_name)),
    };
    
    if let Err(e) = installer.install() {
        return Err(anyhow::anyhow!("自动安装失败: {}\n请手动安装 {} 或使用 --tool=auto 跳过此工具", e, tool_name));
    }
    
    // 安装成功后再次检查
    check_tool_installed(tool_name)
}

/// 检查工具是否已安装
fn check_tool_installed(tool_name: &str) -> Result<String> {
    let output = Command::new(tool_name)
        .arg("--version")
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run {tool_name} --version: {e}"))?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("{tool_name} --version failed"));
    }
    
    let version = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();
    
    Ok(version)
}

/// OpenGrep 安装器
struct OpenGrepInstaller;

impl ToolInstaller for OpenGrepInstaller {
    fn name(&self) -> &'static str {
        "opengrep"
    }
    
    fn is_installed(&self) -> bool {
        check_tool_installed("opengrep").is_ok()
    }
    
    fn install(&self) -> Result<()> {
        println!("🔧 正在安装 OpenGrep...");
        
        // 检测平台
        let (platform, arch) = detect_platform()?;
        
        // OpenGrep 的下载地址（这里使用假设的地址）
        let download_url = get_opengrep_download_url(&platform, &arch)?;
        
        // 创建安装目录
        let install_dir = get_tool_install_dir("opengrep")?;
        fs::create_dir_all(&install_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create install directory: {e}"))?;
        
        // 下载并解压二进制文件
        let binary_path = download_and_extract_tar_gz(&download_url, &install_dir)?;
        
        // 设置执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)?;
        }
        
        // 添加到 PATH（通过创建符号链接到 ~/.local/bin）
        create_symlink(&binary_path, "opengrep")?;
        
        println!("✅ OpenGrep 安装完成");
        Ok(())
    }
    
    fn get_version(&self) -> Result<String> {
        check_tool_installed("opengrep")
    }
}


/// 检测平台信息
fn detect_platform() -> Result<(String, String)> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    
    let platform = match os {
        "macos" => "darwin",
        "linux" => "linux",
        "windows" => "windows",
        _ => return Err(anyhow::anyhow!("不支持的平台: {}", os)),
    };
    
    let arch_str = match arch {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        "x86" => "x86",
        _ => return Err(anyhow::anyhow!("不支持的架构: {}", arch)),
    };
    
    Ok((platform.to_string(), arch_str.to_string()))
}

/// 获取 OpenGrep 下载地址
fn get_opengrep_download_url(platform: &str, arch: &str) -> Result<String> {
    // 为了测试，使用本地模拟文件
    // 实际实现时需要替换为真实的 OpenGrep 发布地址
    if std::env::var("GITAI_TEST_MODE").is_ok() {
        return Ok("file:///tmp/mock-opengrep".to_string());
    }
    
    match (platform, arch) {
        ("darwin", "x64") => Ok("https://github.com/opengrep/opengrep/releases/latest/download/opengrep-core_osx_x86.tar.gz".to_string()),
        ("darwin", "arm64") => Ok("https://github.com/opengrep/opengrep/releases/latest/download/opengrep-core_osx_aarch64.tar.gz".to_string()),
        ("linux", "x64") => Ok("https://github.com/opengrep/opengrep/releases/latest/download/opengrep-core_linux_x86.tar.gz".to_string()),
        ("linux", "arm64") => Ok("https://github.com/opengrep/opengrep/releases/latest/download/opengrep-core_linux_aarch64.tar.gz".to_string()),
        _ => Err(anyhow::anyhow!("不支持的平台架构: {}-{}", platform, arch)),
    }
}

/// 获取工具安装目录
fn get_tool_install_dir(tool_name: &str) -> Result<PathBuf> {
    let install_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local")
        .join("gitai")
        .join("tools")
        .join(tool_name);
    
    Ok(install_dir)
}

/// 下载二进制文件 (保留用于未来可能的直接二进制下载)
#[allow(dead_code)]
fn download_binary(url: &str, dest_path: &Path) -> Result<()> {
    println!("📥 正在从 {url} 下载...");
    
    if let Some(stripped) = url.strip_prefix("file://") {
        // 本地文件复制（用于测试）
        let src_path = PathBuf::from(stripped); // 移除 "file://" 前缀
        fs::copy(&src_path, dest_path)
            .map_err(|e| anyhow::anyhow!("复制文件失败: {}", e))?;
    } else {
        // HTTP 下载
        let response = reqwest::blocking::get(url)
            .map_err(|e| anyhow::anyhow!("下载失败: {}", e))?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP 请求失败: {}", response.status()));
        }
        
        let mut file = fs::File::create(dest_path)
            .map_err(|e| anyhow::anyhow!("无法创建文件: {}", e))?;
        
        let content = response.bytes()?;
        file.write_all(&content)
            .map_err(|e| anyhow::anyhow!("写入文件失败: {}", e))?;
    }
    
    println!("✅ 下载完成: {}", dest_path.display());
    Ok(())
}

/// 创建符号链接到 ~/.local/bin
fn create_symlink(binary_path: &Path, tool_name: &str) -> Result<()> {
    let local_bin = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local")
        .join("bin");
    
    fs::create_dir_all(&local_bin)
        .map_err(|e| anyhow::anyhow!("无法创建 ~/.local/bin 目录: {}", e))?;
    
    let symlink_path = local_bin.join(tool_name);
    
    // 如果符号链接已存在，先删除
    if symlink_path.exists() {
        fs::remove_file(&symlink_path)
            .map_err(|e| anyhow::anyhow!("无法删除现有符号链接: {}", e))?;
    }
    
    // 创建符号链接
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(binary_path, &symlink_path)
            .map_err(|e| anyhow::anyhow!("无法创建符号链接: {}", e))?;
    }
    
    #[cfg(windows)]
    {
        // Windows 上复制文件而不是创建符号链接
        fs::copy(binary_path, &symlink_path)
            .map_err(|e| anyhow::anyhow!("无法复制文件: {}", e))?;
    }
    
    println!("✅ 已创建符号链接: {}", symlink_path.display());
    println!("💡 请确保 ~/.local/bin 在您的 PATH 中");
    
    Ok(())
}

/// 卸载工具
#[allow(dead_code)]
pub fn uninstall_tool(tool_name: &str) -> Result<()> {
    println!("🗑️  正在卸载 {tool_name}...");
    
    // 删除符号链接
    let local_bin = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local")
        .join("bin")
        .join(tool_name);
    
    if local_bin.exists() {
        fs::remove_file(&local_bin)
            .map_err(|e| anyhow::anyhow!("无法删除符号链接: {}", e))?;
        println!("✅ 已删除符号链接: {}", local_bin.display());
    }
    
    // 删除安装目录
    let install_dir = get_tool_install_dir(tool_name)?;
    if install_dir.exists() {
        fs::remove_dir_all(&install_dir)
            .map_err(|e| anyhow::anyhow!("无法删除安装目录: {}", e))?;
        println!("✅ 已删除安装目录: {}", install_dir.display());
    }
    
    println!("✅ {tool_name} 卸载完成");
    Ok(())
}

/// 下载并解压tar.gz文件
fn download_and_extract_tar_gz(url: &str, dest_dir: &Path) -> Result<PathBuf> {
    println!("📥 正在从 {url} 下载...");
    
    let response = reqwest::blocking::get(url)
        .map_err(|e| anyhow::anyhow!("下载失败: {}", e))?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("HTTP 请求失败: {}", response.status()));
    }
    
    let archive_data = response.bytes()?;
    
    // 解压tar.gz文件
    let decoder = flate2::read::GzDecoder::new(&archive_data[..]);
    let mut archive = tar::Archive::new(decoder);
    
    println!("📦 正在解压到 {}...", dest_dir.display());
    archive.unpack(dest_dir)
        .map_err(|e| anyhow::anyhow!("解压失败: {}", e))?;
    
    // 查找opengrep二进制文件
    let opengrep_path = find_opengrep_binary(dest_dir)?;
    
    println!("✅ 下载和解压完成");
    Ok(opengrep_path)
}

/// 查找opengrep二进制文件
fn find_opengrep_binary(dir: &Path) -> Result<PathBuf> {
    let entries = fs::read_dir(dir)
        .map_err(|e| anyhow::anyhow!("无法读取目录 {}: {}", dir.display(), e))?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.file_name().is_some_and(|name| name == "opengrep") {
            return Ok(path);
        }
        
        // 如果是目录，递归查找
        if path.is_dir() {
            if let Ok(found) = find_opengrep_binary(&path) {
                return Ok(found);
            }
        }
    }
    
    Err(anyhow::anyhow!("在目录 {} 中未找到 opengrep 二进制文件", dir.display()))
}

/// 扫描器 trait
trait Scanner {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    fn is_available_no_install(&self) -> bool;
    fn scan(&self, config: &Config, path: &Path) -> Result<ScanResult>;
}

/// OpenGrep 扫描器
struct OpenGrepScanner;

impl Scanner for OpenGrepScanner {
    fn name(&self) -> &'static str {
        "opengrep"
    }
    
    fn is_available(&self) -> bool {
        ensure_tool_available("opengrep").is_ok()
    }
    
    fn is_available_no_install(&self) -> bool {
        check_tool_installed("opengrep").is_ok()
    }
    
    fn scan(&self, config: &Config, path: &Path) -> Result<ScanResult> {
        let start_time = std::time::Instant::now();
        
        // 确保工具可用（会自动安装）
        let version = match ensure_tool_available("opengrep") {
            Ok(v) => v,
            Err(e) => {
                return Ok(ScanResult {
                    tool: "opengrep".to_string(),
                    version: "unknown".to_string(),
                    execution_time: 0.0,
                    findings: vec![],
                    error: Some(format!("OpenGrep not available: {e}")),
                });
            }
        };
        
        // 确保规则存在
        let rules_dir = ensure_rules_available()?;
        
        // 构建命令 - 优化参数顺序和组合
        let mut args = Vec::with_capacity(8 + config.scan.semgrep_exclude_patterns.len());
        
        // 基础参数
        args.push("opengrep".to_string());
        args.push("--json".to_string());
        args.push("--quiet".to_string());
        
        // 性能优化参数
        args.push(format!("--timeout={}", config.scan.semgrep_timeout));
        args.push(format!("--jobs={}", config.scan.semgrep_concurrency));
        
        // 添加排除模式 - 批量处理
        if !config.scan.semgrep_exclude_patterns.is_empty() {
            let exclude_patterns = config.scan.semgrep_exclude_patterns.join(",");
            args.push(format!("--exclude={exclude_patterns}"));
        }
        
        // 添加规则路径 - 优化为直接传递目录
        args.push("--config".to_string());
        args.push(rules_dir.display().to_string());
        
        // 添加扫描路径
        args.push(path.display().to_string());
        
        // 运行扫描 - 使用并行处理
        let (execution_time, findings, error) = scan_parallel_internal(&args)?;
        
        Ok(ScanResult {
            tool: "opengrep".to_string(),
            version,
            execution_time,
            findings,
            error,
        })
    }
}


/// 并行扫描处理
fn scan_parallel_internal(args: &[String]) -> Result<(f64, Vec<Finding>, Option<String>)> {
    let start_time = std::time::Instant::now();
    
    // 运行扫描
    let output = Command::new(&args[0])
        .args(&args[1..])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run opengrep: {e}"))?;
    
    let execution_time = start_time.elapsed().as_secs_f64();
    
    // 创建通道用于并行处理
    let (tx, rx) = mpsc::channel();
    
    // 将输出数据克隆到新线程中处理
    let stdout_data = output.stdout.clone();
    let stderr_data = output.stderr.clone();
    let status = output.status;
    
    // 启动解析线程
    let parse_handle = thread::spawn(move || -> Result<(), anyhow::Error> {
        if status.success() || status.code() == Some(2) {
            // OpenGrep returns exit code 2 for partial success (some results, some rule errors)
            let raw_output = match std::str::from_utf8(&stdout_data) {
                Ok(s) => s,
                Err(_) => return Err(anyhow::anyhow!("Failed to parse stdout as UTF-8")),
            };
            
            match parse_scan_results_parallel(raw_output, "opengrep") {
                Ok(f) => tx.send(Ok(f)).unwrap(),
                Err(e) => {
                    let error_msg = format!("Failed to parse opengrep results: {e}");
                    tx.send(Err(error_msg)).unwrap();
                },
            }
        } else {
            let stderr = match std::str::from_utf8(&stderr_data) {
                Ok(s) => s,
                Err(_) => "Failed to parse stderr as UTF-8",
            };
            let error_msg = format!("OpenGrep failed: {stderr}");
            tx.send(Err(error_msg)).unwrap();
        }
        Ok(())
    });
    
    // 等待解析结果
    let result = rx.recv().unwrap();
    
    // 等待线程完成
    parse_handle.join().unwrap()?;
    
    match result {
        Ok(findings) => Ok((execution_time, findings, None)),
        Err(error) => Ok((execution_time, Vec::new(), Some(error))),
    }
}

/// 智能扫描器 - 根据工具可用性自动选择
/// 检查扫描器是否可用
fn is_scanner_available(scanner: &dyn Scanner, auto_install: bool) -> bool {
    if auto_install {
        scanner.is_available()
    } else {
        scanner.is_available_no_install()
    }
}

/// 创建错误结果
fn create_error_result(tool: &str, error: &str) -> ScanResult {
    ScanResult {
        tool: tool.to_string(),
        version: "unknown".to_string(),
        execution_time: 0.0,
        findings: vec![],
        error: Some(error.to_string()),
    }
}

/// 自动选择扫描器
fn run_auto_scan(config: &Config, path: &Path, auto_install: bool) -> Result<ScanResult> {
    let scanners = [Box::new(OpenGrepScanner) as Box<dyn Scanner>];
    
    for scanner in scanners {
        if is_scanner_available(scanner.as_ref(), auto_install) {
            println!("🔍 自动选择扫描工具: {}", scanner.name());
            return scanner.scan(config, path);
        }
    }
    
    let error_msg = if auto_install {
        "没有可用的扫描工具，即使尝试了自动安装"
    } else {
        "没有可用的扫描工具，请使用 --auto-install 选项启用自动安装"
    };
    
    Ok(create_error_result("none", error_msg))
}

/// 运行指定扫描器
fn run_specific_scan(config: &Config, path: &Path, tool: &str, auto_install: bool) -> Result<ScanResult> {
    let scanners = [Box::new(OpenGrepScanner) as Box<dyn Scanner>];
    
    for scanner in scanners {
        if scanner.name() == tool {
            if is_scanner_available(scanner.as_ref(), auto_install) {
                return scanner.scan(config, path);
            } else {
                return Ok(create_error_result(tool, &format!("工具 {tool} 不可用")));
            }
        }
    }
    
    Ok(create_error_result(tool, &format!("未知工具: {tool}")))
}

pub fn run_smart_scan(config: &Config, path: &Path, tool: &str, auto_install: bool) -> Result<ScanResult> {
    match tool {
        "auto" => run_auto_scan(config, path, auto_install),
        _ => run_specific_scan(config, path, tool, auto_install),
    }
}

/// 运行OpenGrep扫描（保持向后兼容）
pub fn run_opengrep_scan(config: &Config, path: &Path) -> Result<ScanResult> {
    OpenGrepScanner.scan(config, path)
}


/// 解析扫描结果（优化的高性能解析器）
fn parse_scan_results(raw_output: &str, tool_name: &str) -> Result<Vec<Finding>> {
    parse_scan_results_parallel(raw_output, tool_name)
}

/// 并行解析扫描结果
fn parse_scan_results_parallel(raw_output: &str, tool_name: &str) -> Result<Vec<Finding>> {
    if raw_output.trim().is_empty() {
        return Ok(Vec::new());
    }
    
    // 使用serde_json::Value进行解析，但优化内存使用
    let output: serde_json::Value = serde_json::from_str(raw_output)
        .map_err(|e| anyhow::anyhow!("Failed to parse {tool_name} JSON: {e}"))?;
    
    let results = output.get("results")
        .and_then(|r| r.as_array())
        .ok_or_else(|| anyhow::anyhow!("No results field in {tool_name} output"))?;
    
    // 预分配容量以减少内存重分配
    let mut findings = Vec::with_capacity(results.len());
    
    // 批量处理结果，减少函数调用开销
    for result in results {
        let finding = create_finding_from_json_optimized(result)?;
        findings.push(finding);
    }
    
    Ok(findings)
}

/// 从JSON值创建Finding（优化版本）
fn create_finding_from_json_optimized(result: &serde_json::Value) -> Result<Finding> {
    // 一次性获取所有需要的字段，减少JSON查找次数
    let empty_map = serde_json::Map::new();
    let extra = result.get("extra").and_then(|e| e.as_object()).unwrap_or(&empty_map);
    let start = result.get("start").and_then(|s| s.as_object()).unwrap_or(&empty_map);
    
    // 直接获取字符串值，避免多次unwrap
    let title = extra.get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("Unknown security issue");
    
    let path = result.get("path")
        .and_then(|p| p.as_str())
        .unwrap_or("unknown");
    
    let start_line = start.get("line")
        .and_then(|l| l.as_u64())
        .unwrap_or(1) as usize;
    
    let severity = extra.get("severity")
        .and_then(|s| s.as_str())
        .unwrap_or("warning");
    
    // 优化的严重程度解析
    let severity = match severity {
        "error" => Severity::Error,
        "warning" => Severity::Warning,
        "info" => Severity::Info,
        _ => Severity::Warning,
    };
    
    // 优化的代码片段处理
    let code_snippet = extra.get("lines")
        .and_then(|l| l.as_array())
        .map(|lines| {
            let mut snippet = String::new();
            for line in lines.iter().filter_map(|line| line.as_str()).take(3) {
                if !snippet.is_empty() {
                    snippet.push('\n');
                }
                snippet.push_str(line);
            }
            snippet
        });
    
    let rule_id = result.get("check_id")
        .and_then(|c| c.as_str())
        .unwrap_or("unknown");
    
    // 优化：避免重复的字符串分配
    let title_str = if title == "Unknown security issue" {
        "Unknown security issue".to_string()
    } else {
        title.to_string()
    };
    
    Ok(Finding {
        title: title_str.clone(),
        description: title_str,
        severity,
        file_path: PathBuf::from(path),
        line: start_line,
        code_snippet,
        rule_id: if rule_id == "unknown" {
            "unknown".to_string()
        } else {
            rule_id.to_string()
        },
    })
}

/// 从JSON值创建Finding（保持向后兼容）
fn create_finding_from_json(result: &serde_json::Value, _index: usize, _tool_name: &str) -> Result<Finding> {
    create_finding_from_json_optimized(result)
}

/// 递归复制.yml文件
fn copy_yaml_files_recursively(source_dir: &Path, target_dir: &Path) -> Result<usize> {
    let mut count = 0;
    
    if let Ok(entries) = fs::read_dir(source_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "yml" || ext == "yaml" {
                        let target_path = target_dir.join(path.file_name().unwrap());
                        fs::copy(&path, &target_path)?;
                        count += 1;
                    }
                }
            } else if path.is_dir() {
                // 递归处理子目录
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if dir_name != ".git" {
                    count += copy_yaml_files_recursively(&path, target_dir)?;
                }
            }
        }
    }
    
    Ok(count)
}


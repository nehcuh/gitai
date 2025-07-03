use std::path::{Path, PathBuf};
use crate::common::{AppError, AppResult};

/// Expands a path starting with `~` to the user's home directory.
///
/// If the input path begins with `~/`, replaces the tilde with the user's home directory.
/// Returns the original path if it does not start with `~/` or if the home directory cannot be determined.
///
/// # Examples
///
/// ```
/// let home_expanded = expand_path("~/Documents");
/// assert!(home_expanded.to_str().unwrap().contains("Documents"));
/// ```
pub fn expand_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if let Some(path_str) = path.to_str() {
        if path_str.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(&path_str[2..]);
            }
        }
    }
    path.to_path_buf()
}

/// Ensures that a directory exists at the specified path, creating it and any missing parent directories if necessary.
///
/// Returns an error if the directory cannot be created.
///
/// # Examples
///
/// ```
/// use crate::common::utils::ensure_dir_exists;
/// use std::path::PathBuf;
///
/// let dir = PathBuf::from("some/test/dir");
/// ensure_dir_exists(&dir).unwrap();
/// assert!(dir.exists() && dir.is_dir());
/// ```
pub fn ensure_dir_exists(path: impl AsRef<Path>) -> AppResult<()> {
    let path = path.as_ref();
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| AppError::io(format!("创建目录失败 {}: {}", path.display(), e)))?;
    }
    Ok(())
}

/// Returns `true` if the specified path exists, is a file, and its metadata can be accessed, indicating the file is readable.
///
/// # Examples
///
/// ```
/// let readable = is_file_readable("Cargo.toml");
/// assert!(readable || !std::path::Path::new("Cargo.toml").exists());
/// ```
pub fn is_file_readable(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    path.exists() && path.is_file() && std::fs::metadata(path).is_ok()
}

/// Reads the contents of a file as a UTF-8 string, returning an error if the file does not exist or is not readable.
///
/// Returns an `AppError` if the file is missing, unreadable, or if reading fails.
///
/// # Examples
///
/// ```
/// let content = read_file_safe("example.txt")?;
/// assert!(content.contains("example"));
/// ```
pub fn read_file_safe(path: impl AsRef<Path>) -> AppResult<String> {
    let path = path.as_ref();
    if !is_file_readable(path) {
        return Err(AppError::io(format!("文件不存在或不可读: {}", path.display())));
    }
    
    std::fs::read_to_string(path)
        .map_err(|e| AppError::io(format!("读取文件失败 {}: {}", path.display(), e)))
}

/// Writes string content to a file, ensuring the parent directory exists.
///
/// If the parent directory does not exist, it is created before writing. Any I/O errors are wrapped in `AppError`.
///
/// # Examples
///
/// ```
/// use crate::common::utils::write_file_safe;
/// let tmp_dir = tempfile::tempdir().unwrap();
/// let file_path = tmp_dir.path().join("example.txt");
/// write_file_safe(&file_path, "Hello, world!").unwrap();
/// assert_eq!(std::fs::read_to_string(&file_path).unwrap(), "Hello, world!");
/// ```
pub fn write_file_safe(path: impl AsRef<Path>, content: &str) -> AppResult<()> {
    let path = path.as_ref();
    
    // 确保父目录存在
    if let Some(parent) = path.parent() {
        ensure_dir_exists(parent)?;
    }
    
    std::fs::write(path, content)
        .map_err(|e| AppError::io(format!("写入文件失败 {}: {}", path.display(), e)))
}

/// Returns the last modification time of a file as a Unix timestamp (seconds since epoch).
///
/// Returns an error if the file metadata or modification time cannot be retrieved, or if the time cannot be converted.
///
/// # Examples
///
/// ```
/// let mtime = get_file_mtime("/etc/hosts")?;
/// assert!(mtime > 0);
/// ```
pub fn get_file_mtime(path: impl AsRef<Path>) -> AppResult<u64> {
    let path = path.as_ref();
    let metadata = std::fs::metadata(path)
        .map_err(|e| AppError::io(format!("获取文件元数据失败 {}: {}", path.display(), e)))?;
    
    metadata
        .modified()
        .map_err(|e| AppError::io(format!("获取文件修改时间失败 {}: {}", path.display(), e)))?
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| AppError::generic(format!("时间转换失败: {}", e)))
}

/// Converts a byte count into a human-readable string with appropriate units (B, KB, MB, GB, TB).
///
/// # Examples
///
/// ```
/// assert_eq!(format_bytes(500), "500 B");
/// assert_eq!(format_bytes(2048), "2.0 KB");
/// assert_eq!(format_bytes(5_242_880), "5.0 MB");
/// ```
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: u64 = 1024;
    
    if bytes < THRESHOLD {
        return format!("{} B", bytes);
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD as f64;
        unit_index += 1;
    }
    
    format!("{:.1} {}", size, UNITS[unit_index])
}

/// Truncates a string to a maximum length, appending an ellipsis if truncated.
///
/// If the input string exceeds `max_len` characters, it is shortened and "..." is appended. If `max_len` is 3 or less, returns only "...". Returns the original string if it fits within `max_len`.
///
/// # Examples
///
/// ```
/// let s = "Hello, world!";
/// assert_eq!(truncate_string(s, 5), "He...");
/// assert_eq!(truncate_string(s, 20), "Hello, world!");
/// assert_eq!(truncate_string(s, 3), "...");
/// ```
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        "...".to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Expands and canonicalizes a file path, resolving `~` to the home directory and symbolic links if possible.
///
/// If canonicalization fails, returns the expanded path without further modification.
///
/// # Examples
///
/// ```
/// let normalized = normalize_path("~/myfolder/../file.txt");
/// // Returns the absolute, canonical path if possible, or the expanded path with `~` resolved.
/// ```
pub fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let path = expand_path(path);
    
    // 尝试规范化路径
    if let Ok(canonical) = path.canonicalize() {
        canonical
    } else {
        path
    }
}

/// Checks whether a path is within a specified base directory to prevent path traversal.
///
/// Returns `true` if the normalized `path` is inside the normalized `base` directory, otherwise `false`.
///
/// # Examples
///
/// ```
/// let base = "/home/user/data";
/// let safe_path = "/home/user/data/file.txt";
/// let unsafe_path = "/home/user/other/file.txt";
/// assert!(is_path_safe(safe_path, base));
/// assert!(!is_path_safe(unsafe_path, base));
/// ```
pub fn is_path_safe(path: impl AsRef<Path>, base: impl AsRef<Path>) -> bool {
    let path = normalize_path(path);
    let base = normalize_path(base);
    
    path.starts_with(base)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("hello world", 8), "hello...");
        assert_eq!(truncate_string("hi", 2), "hi");
    }

    #[test]
    fn test_ensure_dir_exists() {
        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.path().join("test_dir");
        
        assert!(!new_dir.exists());
        ensure_dir_exists(&new_dir).unwrap();
        assert!(new_dir.exists());
    }

    #[test]
    fn test_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        let content = "Hello, World!";
        
        // 写入文件
        write_file_safe(&test_file, content).unwrap();
        assert!(is_file_readable(&test_file));
        
        // 读取文件
        let read_content = read_file_safe(&test_file).unwrap();
        assert_eq!(read_content, content);
    }
}
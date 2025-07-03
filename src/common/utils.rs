use std::path::{Path, PathBuf};
use crate::common::{AppError, AppResult};

/// 展开路径中的波浪号（~）为用户主目录
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

/// 确保目录存在，如果不存在则创建
pub fn ensure_dir_exists(path: impl AsRef<Path>) -> AppResult<()> {
    let path = path.as_ref();
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| AppError::io(format!("创建目录失败 {}: {}", path.display(), e)))?;
    }
    Ok(())
}

/// 检查文件是否存在且可读
pub fn is_file_readable(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    path.exists() && path.is_file() && std::fs::metadata(path).is_ok()
}

/// 安全地读取文件内容
pub fn read_file_safe(path: impl AsRef<Path>) -> AppResult<String> {
    let path = path.as_ref();
    if !is_file_readable(path) {
        return Err(AppError::io(format!("文件不存在或不可读: {}", path.display())));
    }
    
    std::fs::read_to_string(path)
        .map_err(|e| AppError::io(format!("读取文件失败 {}: {}", path.display(), e)))
}

/// 安全地写入文件内容
pub fn write_file_safe(path: impl AsRef<Path>, content: &str) -> AppResult<()> {
    let path = path.as_ref();
    
    // 确保父目录存在
    if let Some(parent) = path.parent() {
        ensure_dir_exists(parent)?;
    }
    
    std::fs::write(path, content)
        .map_err(|e| AppError::io(format!("写入文件失败 {}: {}", path.display(), e)))
}

/// 获取文件的修改时间（Unix 时间戳）
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

/// 格式化字节大小为人类可读的格式
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

/// 截断字符串到指定长度，如果超长则添加省略号
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        "...".to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// 清理和标准化文件路径
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

/// 检查路径是否在指定的父目录下（防止路径遍历攻击）
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
// Git 仓库管理模块

use crate::common::{AppResult, AppError};
use std::path::{Path, PathBuf};

/// Git 仓库信息
#[derive(Debug, Clone)]
pub struct GitRepository {
    pub root_path: PathBuf,
    pub current_branch: Option<String>,
    pub remote_url: Option<String>,
}

impl GitRepository {
    /// 查找并创建仓库实例
    pub fn discover(start_path: impl AsRef<Path>) -> AppResult<Self> {
        let root_path = find_git_root(start_path)?;
        
        Ok(GitRepository {
            root_path,
            current_branch: None,
            remote_url: None,
        })
    }

    /// 获取仓库根目录
    pub fn root(&self) -> &Path {
        &self.root_path
    }

    /// 检查是否是干净的工作目录
    pub async fn is_clean(&self) -> AppResult<bool> {
        // TODO: 实现工作目录状态检查
        Ok(true)
    }
}

/// 查找 Git 仓库根目录
fn find_git_root(start_path: impl AsRef<Path>) -> AppResult<PathBuf> {
    let mut current = start_path.as_ref().to_path_buf();
    
    loop {
        if current.join(".git").exists() {
            return Ok(current);
        }
        
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return Err(AppError::git("未找到 Git 仓库")),
        }
    }
}
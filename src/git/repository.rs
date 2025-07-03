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
    /// Attempts to locate the root of a Git repository starting from the given path and creates a `GitRepository` instance.
    ///
    /// Returns a `GitRepository` with the root path set if a Git repository is found, or an error if not.
    ///
    /// # Examples
    ///
    /// ```
    /// let repo = GitRepository::discover("/some/project/path")?;
    /// assert!(repo.root().ends_with(".git"));
    /// ```
    pub fn discover(start_path: impl AsRef<Path>) -> AppResult<Self> {
        let root_path = find_git_root(start_path)?;
        
        Ok(GitRepository {
            root_path,
            current_branch: None,
            remote_url: None,
        })
    }

    /// Returns the root directory path of the Git repository.
    ///
    /// # Examples
    ///
    /// ```
    /// let repo = GitRepository::discover("/some/path").unwrap();
    /// let root = repo.root();
    /// assert!(root.ends_with(".git") || root.exists());
    /// ```
    pub fn root(&self) -> &Path {
        &self.root_path
    }

    /// Checks if the working directory has no uncommitted changes.
    ///
    /// Currently, this method always returns `Ok(true)` as a placeholder.
    ///
    /// # Returns
    ///
    /// `Ok(true)` if the working directory is considered clean; otherwise, returns an error.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::GitRepository;
    /// # async fn check_clean(repo: &GitRepository) {
    /// let is_clean = repo.is_clean().await.unwrap();
    /// assert!(is_clean);
    /// # }
    /// ```
    pub async fn is_clean(&self) -> AppResult<bool> {
        // TODO: 实现工作目录状态检查
        Ok(true)
    }
}

/// Searches upward from the given path to locate the root directory of a Git repository.
///
/// Starts at `start_path` and traverses parent directories until a directory containing a `.git` subdirectory is found. Returns the path to the Git repository root if found, or an error if no repository is detected.
///
/// # Examples
///
/// ```
/// let repo_root = find_git_root("/path/to/project/subdir")?;
/// assert!(repo_root.ends_with("project"));
/// ```
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
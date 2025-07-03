use crate::common::{AppResult, AppError, CommandOutput};
use std::path::Path;
use std::process::{Command, Stdio};

/// Git 基础操作接口
pub trait GitOperations {
    /// 执行 Git 命令
    async fn execute_git_command(&self, args: &[&str]) -> AppResult<CommandOutput>;
    
    /// 获取 Git 状态
    async fn get_status(&self) -> AppResult<String>;
    
    /// 获取 Git 差异
    async fn get_diff(&self, staged: bool) -> AppResult<String>;
    
    /// 检查是否在 Git 仓库中
    fn is_git_repository(&self) -> bool;
    
    /// 获取当前分支名称
    async fn get_current_branch(&self) -> AppResult<String>;
    
    /// 获取最近的提交
    async fn get_recent_commits(&self, count: usize) -> AppResult<Vec<GitCommit>>;
}

/// Git 操作的默认实现
pub struct GitOps {
    repository_path: std::path::PathBuf,
}

impl GitOps {
    /// 创建新的 Git 操作实例
    pub fn new(repository_path: impl AsRef<Path>) -> Self {
        Self {
            repository_path: repository_path.as_ref().to_path_buf(),
        }
    }

    /// 在当前目录创建 Git 操作实例
    pub fn current_dir() -> AppResult<Self> {
        let current_dir = std::env::current_dir()
            .map_err(|e| AppError::io(format!("获取当前目录失败: {}", e)))?;
        Ok(Self::new(current_dir))
    }

    /// 查找 Git 仓库根目录
    pub fn find_repository_root(start_path: impl AsRef<Path>) -> AppResult<std::path::PathBuf> {
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
}

impl GitOperations for GitOps {
    async fn execute_git_command(&self, args: &[&str]) -> AppResult<CommandOutput> {
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.repository_path)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        tracing::debug!("执行 Git 命令: git {}", args.join(" "));

        let output = cmd.output()
            .map_err(|e| AppError::git(format!("执行 Git 命令失败: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(1);
            let cmd_str = format!("git {}", args.join(" "));
            return Err(AppError::git_command_failed(
                cmd_str,
                Some(exit_code),
                stdout.clone(),
                stderr.clone()
            ));
        }

        Ok(CommandOutput {
            stdout,
            stderr,
            status: output.status,
        })
    }

    async fn get_status(&self) -> AppResult<String> {
        let output = self.execute_git_command(&["status", "--porcelain"]).await?;
        Ok(output.stdout)
    }

    async fn get_diff(&self, staged: bool) -> AppResult<String> {
        let args = if staged {
            vec!["diff", "--staged"]
        } else {
            vec!["diff"]
        };
        
        let output = self.execute_git_command(&args).await?;
        Ok(output.stdout)
    }

    fn is_git_repository(&self) -> bool {
        self.repository_path.join(".git").exists()
    }

    async fn get_current_branch(&self) -> AppResult<String> {
        let output = self.execute_git_command(&["branch", "--show-current"]).await?;
        Ok(output.stdout.trim().to_string())
    }

    async fn get_recent_commits(&self, count: usize) -> AppResult<Vec<GitCommit>> {
        let count_arg = format!("-{}", count);
        let args = vec![
            "log",
            &count_arg,
            "--pretty=format:%H|%an|%ae|%s|%ad",
            "--date=iso"
        ];
        
        let output = self.execute_git_command(&args).await?;
        
        let commits = output.stdout
            .lines()
            .filter_map(|line| GitCommit::from_log_line(line))
            .collect();
            
        Ok(commits)
    }
}

/// Git 提交信息
#[derive(Debug, Clone)]
pub struct GitCommit {
    pub hash: String,
    pub author_name: String,
    pub author_email: String,
    pub message: String,
    pub date: String,
}

impl GitCommit {
    /// 从 git log 输出行解析提交信息
    pub fn from_log_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() == 5 {
            Some(GitCommit {
                hash: parts[0].to_string(),
                author_name: parts[1].to_string(),
                author_email: parts[2].to_string(),
                message: parts[3].to_string(),
                date: parts[4].to_string(),
            })
        } else {
            None
        }
    }

    /// 获取短哈希
    pub fn short_hash(&self) -> String {
        if self.hash.len() >= 7 {
            self.hash[..7].to_string()
        } else {
            self.hash.clone()
        }
    }
}

/// Git 仓库状态信息
#[derive(Debug, Clone)]
pub struct GitStatus {
    pub modified: Vec<String>,
    pub added: Vec<String>,
    pub deleted: Vec<String>,
    pub untracked: Vec<String>,
    pub staged: Vec<String>,
}

impl GitStatus {
    /// 从 git status --porcelain 输出解析状态
    pub fn from_porcelain(output: &str) -> Self {
        let mut status = GitStatus {
            modified: Vec::new(),
            added: Vec::new(),
            deleted: Vec::new(),
            untracked: Vec::new(),
            staged: Vec::new(),
        };

        for line in output.lines() {
            if line.len() < 3 {
                continue;
            }

            let index_status = line.chars().nth(0);
            let worktree_status = line.chars().nth(1);
            let filepath = &line[3..];

            match (index_status, worktree_status) {
                (Some('M'), _) => status.staged.push(filepath.to_string()),
                (Some('A'), _) => status.added.push(filepath.to_string()),
                (Some('D'), _) => status.deleted.push(filepath.to_string()),
                (_, Some('M')) => status.modified.push(filepath.to_string()),
                (Some('?'), Some('?')) => status.untracked.push(filepath.to_string()),
                _ => {}
            }
        }

        status
    }

    /// 检查是否有任何变更
    pub fn has_changes(&self) -> bool {
        !self.modified.is_empty() || 
        !self.added.is_empty() || 
        !self.deleted.is_empty() || 
        !self.staged.is_empty()
    }

    /// 检查是否有暂存的变更
    pub fn has_staged_changes(&self) -> bool {
        !self.staged.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_git_commit_parsing() {
        let line = "abc1234|John Doe|john@example.com|Initial commit|2023-01-01 12:00:00 +0000";
        let commit = GitCommit::from_log_line(line).unwrap();
        
        assert_eq!(commit.hash, "abc1234");
        assert_eq!(commit.author_name, "John Doe");
        assert_eq!(commit.author_email, "john@example.com");
        assert_eq!(commit.message, "Initial commit");
        assert_eq!(commit.short_hash(), "abc1234");
    }

    #[test]
    fn test_git_status_parsing() {
        let output = " M modified.txt\n A added.txt\n D deleted.txt\n?? untracked.txt\nM  staged.txt";
        let status = GitStatus::from_porcelain(output);
        
        assert_eq!(status.modified, vec!["modified.txt"]);
        assert_eq!(status.added, vec!["added.txt"]);
        assert_eq!(status.deleted, vec!["deleted.txt"]);
        assert_eq!(status.untracked, vec!["untracked.txt"]);
        assert_eq!(status.staged, vec!["staged.txt"]);
        assert!(status.has_changes());
        assert!(status.has_staged_changes());
    }

    #[tokio::test]
    async fn test_git_operations_in_non_git_dir() {
        let temp_dir = TempDir::new().unwrap();
        let git_ops = GitOps::new(temp_dir.path());
        
        assert!(!git_ops.is_git_repository());
        
        let result = git_ops.get_status().await;
        assert!(result.is_err());
    }
}
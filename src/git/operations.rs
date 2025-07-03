use crate::common::{AppResult, AppError, CommandOutput};
use async_trait::async_trait;
use std::path::Path;
use std::process::{Command, Stdio};

/// Git 基础操作接口
#[async_trait]
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
    /// Creates a new instance for performing Git operations on the specified repository path.
    ///
    /// # Examples
    ///
    /// ```
    /// let git_ops = GitOps::new("/path/to/repo");
    /// ```
    pub fn new(repository_path: impl AsRef<Path>) -> Self {
        Self {
            repository_path: repository_path.as_ref().to_path_buf(),
        }
    }

    /// Creates a `GitOps` instance for the current working directory.
    ///
    /// Returns an error if the current directory cannot be determined.
    ///
    /// # Examples
    ///
    /// ```
    /// let git_ops = GitOps::current_dir().unwrap();
    /// assert!(git_ops.is_git_repository() || !git_ops.is_git_repository());
    /// ```
    pub fn current_dir() -> AppResult<Self> {
        let current_dir = std::env::current_dir()
            .map_err(|e| AppError::io(format!("获取当前目录失败: {}", e)))?;
        Ok(Self::new(current_dir))
    }

    /// Recursively searches upward from the given path to locate the root of a Git repository.
    ///
    /// Returns the path to the directory containing the `.git` folder, or an error if no Git repository is found.
    ///
    /// # Examples
    ///
    /// ```
    /// let repo_root = find_repository_root("/path/to/some/subdir")?;
    /// assert!(repo_root.join(".git").exists());
    /// ```
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

#[async_trait]
impl GitOperations for GitOps {
    /// Executes a Git command with the specified arguments in the repository directory.
    ///
    /// Runs the given Git command asynchronously, capturing both standard output and error output.
    /// Returns a `CommandOutput` containing the command's output and exit status, or an error if the command fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let output = git_ops.execute_git_command(&["status", "--porcelain"]).await?;
    /// assert!(output.stdout.contains("M"));
    /// ```
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

    /// Retrieves the Git repository status in porcelain format.
    ///
    /// Returns the output of `git status --porcelain`, which provides a machine-readable summary of the repository's current state.
    ///
    /// # Returns
    ///
    /// A string containing the porcelain status output.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_module::{GitOps, GitOperations};
    /// # async fn example() -> anyhow::Result<()> {
    /// let git = GitOps::current_dir()?;
    /// let status = git.get_status().await?;
    /// println!("{}", status);
    /// # Ok(())
    /// # }
    /// ```
    async fn get_status(&self) -> AppResult<String> {
        let output = self.execute_git_command(&["status", "--porcelain"]).await?;
        Ok(output.stdout)
    }

    /// Retrieves the Git diff output for the repository.
    ///
    /// If `staged` is `true`, returns the diff of staged changes (`git diff --staged`); otherwise, returns the diff of unstaged changes (`git diff`).
    ///
    /// # Returns
    /// The diff output as a string.
    ///
    /// # Examples
    ///
    /// ```
    /// let diff = git_ops.get_diff(false).await?;
    /// println!("{}", diff);
    /// ```
    async fn get_diff(&self, staged: bool) -> AppResult<String> {
        let args = if staged {
            vec!["diff", "--staged"]
        } else {
            vec!["diff"]
        };
        
        let output = self.execute_git_command(&args).await?;
        Ok(output.stdout)
    }

    /// Checks if the repository path contains a `.git` directory, indicating a Git repository.
    ///
    /// # Examples
    ///
    /// ```
    /// let git_ops = GitOps::new("/path/to/repo");
    /// assert!(git_ops.is_git_repository());
    /// ```
    fn is_git_repository(&self) -> bool {
        self.repository_path.join(".git").exists()
    }

    /// Retrieves the name of the current Git branch.
    ///
    /// # Returns
    ///
    /// The name of the currently checked-out branch as a `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// let branch = git_ops.get_current_branch().await?;
    /// assert!(!branch.is_empty());
    /// ```
    async fn get_current_branch(&self) -> AppResult<String> {
        let output = self.execute_git_command(&["branch", "--show-current"]).await?;
        Ok(output.stdout.trim().to_string())
    }

    /// Retrieves a list of recent Git commits, limited by the specified count.
    ///
    /// Each commit includes the hash, author name and email, commit message, and date.
    ///
    /// # Examples
    ///
    /// ```
    /// let commits = git_ops.get_recent_commits(5).await?;
    /// assert!(commits.len() <= 5);
    /// ```
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
    /// Parses a single line of `git log` output into a `GitCommit`.
    ///
    /// The input line must contain exactly five fields separated by `|` in the order: hash, author name, author email, commit message, and date.
    /// Returns `Some(GitCommit)` if parsing succeeds, or `None` if the format is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// let line = "abc1234|Alice|alice@example.com|Initial commit|2024-07-01";
    /// let commit = GitCommit::from_log_line(line);
    /// assert!(commit.is_some());
    /// ```
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

    /// Returns the first 7 characters of the commit hash, or the full hash if it is shorter.
    ///
    /// # Examples
    ///
    /// ```
    /// let commit = GitCommit {
    ///     hash: "abcdef1234567890".to_string(),
    ///     author_name: "Alice".to_string(),
    ///     author_email: "alice@example.com".to_string(),
    ///     message: "Initial commit".to_string(),
    ///     date: "2024-06-01".to_string(),
    /// };
    /// assert_eq!(commit.short_hash(), "abcdef1");
    /// ```
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
    /// Parses the output of `git status --porcelain` into a `GitStatus` struct, categorizing files by their change type.
    ///
    /// This method processes each line of the porcelain output and assigns file paths to the appropriate lists for modified, added, deleted, untracked, and staged files.
    ///
    /// # Examples
    ///
    /// ```
    /// let output = " M file1.txt\nA  file2.txt\n?? file3.txt\n";
    /// let status = GitStatus::from_porcelain(output);
    /// assert!(status.modified.contains(&"file1.txt".to_string()));
    /// assert!(status.added.contains(&"file2.txt".to_string()));
    /// assert!(status.untracked.contains(&"file3.txt".to_string()));
    /// ```
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
                // Staged changes (index status)
                (Some('M'), _) => status.staged.push(filepath.to_string()),
                (Some('A'), _) => status.staged.push(filepath.to_string()),
                (Some('D'), _) => status.staged.push(filepath.to_string()),
                // Working tree changes
                (_, Some('M')) => status.modified.push(filepath.to_string()),
                (_, Some('A')) => status.added.push(filepath.to_string()),
                (_, Some('D')) => status.deleted.push(filepath.to_string()),
                // Untracked files
                (Some('?'), Some('?')) => status.untracked.push(filepath.to_string()),
                _ => {}
            }
        }

        status
    }

    /// Returns `true` if there are any modified, added, deleted, or staged files in the repository status.
    ///
    /// # Examples
    ///
    /// ```
    /// let status = GitStatus {
    ///     modified: vec!["file1.txt".into()],
    ///     added: vec![],
    ///     deleted: vec![],
    ///     untracked: vec![],
    ///     staged: vec![],
    /// };
    /// assert!(status.has_changes());
    /// ```
    pub fn has_changes(&self) -> bool {
        !self.modified.is_empty() || 
        !self.added.is_empty() || 
        !self.deleted.is_empty() || 
        !self.staged.is_empty()
    }

    /// Returns `true` if there are any staged changes in the repository.
    ///
    /// # Examples
    ///
    /// ```
    /// let status = GitStatus {
    ///     modified: vec![],
    ///     added: vec![],
    ///     deleted: vec![],
    ///     untracked: vec![],
    ///     staged: vec!["file1.txt".to_string()],
    /// };
    /// assert!(status.has_staged_changes());
    /// ```
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
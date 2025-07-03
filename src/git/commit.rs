// Git commit 操作模块
// TODO: 将从现有代码迁移 commit 相关功能

use crate::common::AppResult;
use crate::git::GitOperations;

/// Git 提交操作
pub struct CommitOperations<T: GitOperations> {
    git_ops: T,
}

impl<T: GitOperations> CommitOperations<T> {
    /// Creates a new `CommitOperations` instance with the specified Git operations handler.
    ///
    /// # Examples
    ///
    /// ```
    /// let git_ops = MyGitOps::default();
    /// let commit_ops = CommitOperations::new(git_ops);
    /// ```
    pub fn new(git_ops: T) -> Self {
        Self { git_ops }
    }

    /// Commits staged changes to the Git repository, optionally staging all modified files first.
    ///
    /// If `auto_stage` is true, all modified files are staged before committing. The commit is performed with the provided message, and the standard output of the commit command is returned.
    ///
    /// # Parameters
    ///
    /// - `message`: The commit message to use.
    /// - `auto_stage`: If true, stages all modified files before committing.
    ///
    /// # Returns
    ///
    /// The standard output from the Git commit command.
    ///
    /// # Examples
    ///
    /// ```
    /// let result = commit_ops.commit("Initial commit", true).await?;
    /// println!("{}", result);
    /// ```
    pub async fn commit(&self, message: &str, auto_stage: bool) -> AppResult<String> {
        if auto_stage {
            // 暂存所有修改的文件
            self.git_ops.execute_git_command(&["add", "-u"]).await?;
        }

        // 执行提交
        let output = self.git_ops.execute_git_command(&["commit", "-m", message]).await?;
        Ok(output.stdout)
    }

    /// Checks if there are any staged changes ready to be committed.
    ///
    /// Returns `Ok(true)` if there are staged changes, or `Ok(false)` if there are none. Errors from the underlying Git operation are propagated.
    ///
    /// # Examples
    ///
    /// ```
    /// let commit_ops = CommitOperations::new(git_ops);
    /// let has_changes = commit_ops.has_staged_changes().await?;
    /// if has_changes {
    ///     println!("There are staged changes to commit.");
    /// }
    /// ```
    pub async fn has_staged_changes(&self) -> AppResult<bool> {
        let diff = self.git_ops.get_diff(true).await?;
        Ok(!diff.trim().is_empty())
    }
}
// Git commit 操作模块
// TODO: 将从现有代码迁移 commit 相关功能

use crate::common::AppResult;
use crate::git::GitOperations;

/// Git 提交操作
pub struct CommitOperations<T: GitOperations> {
    git_ops: T,
}

impl<T: GitOperations> CommitOperations<T> {
    pub fn new(git_ops: T) -> Self {
        Self { git_ops }
    }

    /// 执行提交操作
    pub async fn commit(&self, message: &str, auto_stage: bool) -> AppResult<String> {
        if auto_stage {
            // 暂存所有修改的文件
            self.git_ops.execute_git_command(&["add", "-u"]).await?;
        }

        // 执行提交
        let output = self.git_ops.execute_git_command(&["commit", "-m", message]).await?;
        Ok(output.stdout)
    }

    /// 检查是否有可提交的更改
    pub async fn has_staged_changes(&self) -> AppResult<bool> {
        let diff = self.git_ops.get_diff(true).await?;
        Ok(!diff.trim().is_empty())
    }
}
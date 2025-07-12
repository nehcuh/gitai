//! 工作区状态检测模块
//! 
//! 该模块提供检测 Git 工作区状态的功能，用于"防呆设计"
//! 帮助用户了解当前要分析的代码状态

use crate::errors::AppError;
use crate::handlers::git;
use serde::{Deserialize, Serialize};

/// 工作区状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceStatus {
    /// 是否是 Git 仓库
    pub is_git_repo: bool,
    /// 是否有已暂存的变更
    pub has_staged_changes: bool,
    /// 是否有未暂存的变更
    pub has_unstaged_changes: bool,
    /// 是否有未跟踪的文件
    pub has_untracked_files: bool,
    /// 工作区是否干净（无任何变更）
    pub is_clean: bool,
    /// 状态描述
    pub description: String,
    /// 友好的提示信息
    pub user_friendly_message: String,
}

impl WorkspaceStatus {
    /// 检测指定目录的工作区状态
    pub async fn detect(dir: Option<&str>) -> Result<Self, AppError> {
        // 检查是否是 Git 仓库
        let is_git_repo = git::is_git_repository_in_dir(dir).unwrap_or(false);
        
        if !is_git_repo {
            return Ok(Self {
                is_git_repo: false,
                has_staged_changes: false,
                has_unstaged_changes: false,
                has_untracked_files: false,
                is_clean: true,
                description: "非 Git 仓库".to_string(),
                user_friendly_message: "当前目录不是 Git 仓库，将分析当前工作目录中的所有文件".to_string(),
            });
        }

        // 获取仓库状态
        let status_output = git::get_repository_status_in_dir(dir).await?;
        let is_clean = status_output.trim().is_empty();
        
        // 分析状态详情
        let mut has_staged_changes = false;
        let mut has_unstaged_changes = false;
        let mut has_untracked_files = false;
        
        for line in status_output.lines() {
            if line.len() < 3 {
                continue;
            }
            
            let staged_status = line.chars().nth(0).unwrap_or(' ');
            let unstaged_status = line.chars().nth(1).unwrap_or(' ');
            
            // 检查已暂存的变更
            if staged_status != ' ' && staged_status != '?' {
                has_staged_changes = true;
            }
            
            // 检查未暂存的变更
            if unstaged_status != ' ' && unstaged_status != '?' {
                has_unstaged_changes = true;
            }
            
            // 检查未跟踪的文件
            if staged_status == '?' && unstaged_status == '?' {
                has_untracked_files = true;
            }
        }
        
        // 生成描述和用户友好信息
        let (description, user_friendly_message) = Self::generate_status_messages(
            is_clean,
            has_staged_changes,
            has_unstaged_changes,
            has_untracked_files,
        );
        
        Ok(Self {
            is_git_repo: true,
            has_staged_changes,
            has_unstaged_changes,
            has_untracked_files,
            is_clean,
            description,
            user_friendly_message,
        })
    }
    
    /// 生成状态描述和用户友好信息
    fn generate_status_messages(
        is_clean: bool,
        has_staged: bool,
        has_unstaged: bool,
        has_untracked: bool,
    ) -> (String, String) {
        if is_clean {
            return (
                "工作区干净".to_string(),
                "✅ 所有变更已提交，将基于最新的提交内容进行分析".to_string(),
            );
        }
        
        let mut status_parts = Vec::new();
        let mut analysis_parts = Vec::new();
        
        if has_staged {
            status_parts.push("已暂存变更");
            analysis_parts.push("已暂存的变更");
        }
        
        if has_unstaged {
            status_parts.push("未暂存变更");
            analysis_parts.push("未暂存的变更");
        }
        
        if has_untracked {
            status_parts.push("未跟踪文件");
            analysis_parts.push("新创建的文件");
        }
        
        let description = format!("包含: {}", status_parts.join("、"));
        
        let analysis_scope = if analysis_parts.len() == 1 {
            analysis_parts[0].to_string()
        } else {
            format!("{} 和 {}", 
                analysis_parts[..analysis_parts.len()-1].join("、"),
                analysis_parts.last().unwrap()
            )
        };
        
        let user_friendly_message = format!(
            "⚠️  检测到未提交的代码变更，将分析{}",
            analysis_scope
        );
        
        (description, user_friendly_message)
    }
    
    /// 获取用于输出的状态标签
    pub fn get_status_badge(&self) -> String {
        if !self.is_git_repo {
            return "📁 非Git仓库".to_string();
        }
        
        if self.is_clean {
            return "✅ 已提交".to_string();
        }
        
        let mut badges = Vec::new();
        
        if self.has_staged_changes {
            badges.push("📋 已暂存");
        }
        
        if self.has_unstaged_changes {
            badges.push("📝 未暂存");
        }
        
        if self.has_untracked_files {
            badges.push("❓ 新文件");
        }
        
        format!("⚠️  {}", badges.join(" | "))
    }
    
    /// 检查是否应该显示未提交代码警告
    pub fn should_show_uncommitted_warning(&self) -> bool {
        self.is_git_repo && !self.is_clean
    }
    
    /// 获取分析范围说明
    pub fn get_analysis_scope_description(&self) -> String {
        if !self.is_git_repo {
            return "分析当前工作目录中的所有文件".to_string();
        }
        
        if self.is_clean {
            return "基于最新提交的代码进行分析".to_string();
        }
        
        let mut scopes = Vec::new();
        
        if self.has_staged_changes {
            scopes.push("已暂存的变更");
        }
        
        if self.has_unstaged_changes {
            scopes.push("工作区中的变更");
        }
        
        if self.has_untracked_files {
            scopes.push("新创建的文件");
        }
        
        if scopes.len() == 1 {
            format!("分析{}", scopes[0])
        } else {
            format!("分析{}", scopes.join("和"))
        }
    }
}

/// 格式化输出工作区状态信息
pub fn format_workspace_status_header(status: &WorkspaceStatus) -> String {
    let mut header = String::new();
    
    // 状态标签
    header.push_str(&format!("📊 代码状态: {}\n", status.get_status_badge()));
    
    // 友好提示信息
    header.push_str(&format!("{}\n", status.user_friendly_message));
    
    // 分析范围说明
    header.push_str(&format!("🔍 分析范围: {}\n", status.get_analysis_scope_description()));
    
    header.push_str("\n");
    header
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_status_messages_clean() {
        let (desc, msg) = WorkspaceStatus::generate_status_messages(true, false, false, false);
        assert_eq!(desc, "工作区干净");
        assert!(msg.contains("所有变更已提交"));
    }

    #[test]
    fn test_generate_status_messages_staged_only() {
        let (desc, msg) = WorkspaceStatus::generate_status_messages(false, true, false, false);
        assert_eq!(desc, "包含: 已暂存变更");
        assert!(msg.contains("已暂存的变更"));
    }

    #[test]
    fn test_generate_status_messages_mixed() {
        let (desc, msg) = WorkspaceStatus::generate_status_messages(false, true, true, true);
        assert_eq!(desc, "包含: 已暂存变更、未暂存变更、未跟踪文件");
        assert!(msg.contains("已暂存的变更 和 未暂存的变更 和 新创建的文件"));
    }

    #[test]
    fn test_status_badges() {
        let status = WorkspaceStatus {
            is_git_repo: true,
            has_staged_changes: true,
            has_unstaged_changes: false,
            has_untracked_files: false,
            is_clean: false,
            description: "test".to_string(),
            user_friendly_message: "test".to_string(),
        };
        
        assert_eq!(status.get_status_badge(), "⚠️  📋 已暂存");
    }
}
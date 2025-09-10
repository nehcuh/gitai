// Git 变更前代码状态获取模块
// 用于获取 git diff 中变更前的代码状态并进行 Tree-sitter 分析

use gitai_types::GitAIError;
use crate::tree_sitter::{StructuralSummary, SupportedLanguage, TreeSitterManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

/// 架构影响分析结果（简化版本）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalImpact {
    pub function_changes: Vec<FunctionChange>,
    pub struct_changes: Vec<StructChange>,
    pub interface_changes: Vec<InterfaceChange>,
    pub impact_summary: ImpactSummary,
}

/// 函数变更信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionChange {
    pub name: String,
    pub change_type: ChangeType,
    pub file_path: String,
    pub description: String,
}

/// 结构体变更信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructChange {
    pub name: String,
    pub change_type: ChangeType,
    pub file_path: String,
    pub description: String,
}

/// 接口变更信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceChange {
    pub name: String,
    pub change_type: ChangeType,
    pub file_path: String,
    pub description: String,
}

/// 变更类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Removed,
}

/// 影响摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactSummary {
    pub affected_modules: Vec<String>,
    pub breaking_changes: Vec<String>,
    pub risk_level: String,
}

/// Git 状态分析器
pub struct GitStateAnalyzer {
    tree_sitter_manager: Option<TreeSitterManager>,
}

impl Default for GitStateAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl GitStateAnalyzer {
    /// 创建新的 Git 状态分析器（同步版本）
    pub fn new() -> Self {
        Self {
            tree_sitter_manager: None,
        }
    }

    /// 创建新的 Git 状态分析器（异步版本）
    pub async fn new_async() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            tree_sitter_manager: None,
        })
    }

    /// 懒加载 TreeSitter 管理器
    async fn get_or_init_tree_sitter(
        &mut self,
    ) -> Result<&mut TreeSitterManager, Box<dyn std::error::Error + Send + Sync>> {
        if self.tree_sitter_manager.is_none() {
            self.tree_sitter_manager = Some(TreeSitterManager::new().await?);
        }
        Ok(self
            .tree_sitter_manager
            .as_mut()
            .expect("TreeSitterManager should be initialized"))
    }

    /// 获取指定文件在指定提交中的内容
    pub fn get_file_content_at_commit(
        &self,
        file_path: &str,
        commit_ref: &str,
    ) -> Result<String, GitAIError> {
        let output = Command::new("git")
            .args(["show", &format!("{commit_ref}:{file_path}")])
            .output()
            .map_err(|e| {
                GitAIError::Git(format!(
                    "无法执行 git show 命令: {e}"
                ))
            })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(GitAIError::Git(format!(
                "git show 命令执行失败: {error_msg}"
            )));
        }

        String::from_utf8(output.stdout).map_err(|e| {
            GitAIError::Git(format!(
                "无法解析文件内容为UTF-8: {e}"
            ))
        })
    }

    /// 获取当前工作目录相对于 git 根目录的文件列表
    pub fn get_changed_files(&self) -> Result<Vec<String>, GitAIError> {
        let output = Command::new("git")
            .args(["diff", "--name-only", "HEAD~1..HEAD"])
            .output()
            .map_err(|e| {
                GitAIError::Git(format!(
                    "无法执行 git diff 命令: {e}"
                ))
            })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(GitAIError::Git(format!(
                "git diff 命令执行失败: {error_msg}"
            )));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let files: Vec<String> = output_str
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.to_string())
            .collect();

        Ok(files)
    }

    /// 获取变更前的代码分析结果
    pub async fn analyze_before_state(
        &mut self,
        file_path: &str,
        base_commit: Option<&str>,
    ) -> Result<StructuralSummary, Box<dyn std::error::Error + Send + Sync>> {
        let commit_ref = base_commit.unwrap_or("HEAD~1");

        // 获取变更前的文件内容
        let before_content = self.get_file_content_at_commit(file_path, commit_ref)?;

        // 推断语言类型
        let language = infer_language_from_path(file_path)?;

        // 获取 TreeSitter 管理器并分析
        let manager = self.get_or_init_tree_sitter().await?;
        let summary = manager.analyze_structure(&before_content, language)?;

        Ok(summary)
    }

    /// 获取变更后的代码分析结果（当前工作区）
    pub async fn analyze_after_state(
        &mut self,
        file_path: &str,
    ) -> Result<StructuralSummary, Box<dyn std::error::Error + Send + Sync>> {
        // 读取当前文件内容
        let current_content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("无法读取文件 {file_path}: {e}"))?;

        // 推断语言类型
        let language = infer_language_from_path(file_path)?;

        // 获取 TreeSitter 管理器并分析
        let manager = self.get_or_init_tree_sitter().await?;
        let summary = manager.analyze_structure(&current_content, language)?;

        Ok(summary)
    }

    /// 批量分析变更前后的文件
    pub async fn analyze_all_changed_files(
        &mut self,
        base_commit: Option<&str>,
    ) -> Result<
        HashMap<String, (StructuralSummary, StructuralSummary)>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let changed_files = self.get_changed_files()?;
        let mut results = HashMap::new();

        for file_path in &changed_files {
            // 只分析代码文件
            if !is_code_file(file_path) {
                continue;
            }

            log::debug!("分析文件变更: {file_path}");

            // 尝试分析变更前后的状态
            match (
                self.analyze_before_state(file_path, base_commit).await,
                self.analyze_after_state(file_path).await,
            ) {
                (Ok(before), Ok(after)) => {
                    results.insert(file_path.to_string(), (before, after));
                    log::info!("成功分析文件变更: {file_path}");
                }
                (Err(before_err), Ok(_)) => {
                    log::warn!("无法获取文件 {file_path} 的变更前状态: {before_err}");
                    // 可能是新文件，继续处理
                }
                (Ok(_), Err(after_err)) => {
                    log::warn!("无法获取文件 {file_path} 的变更后状态: {after_err}");
                }
                (Err(before_err), Err(after_err)) => {
                    log::error!(
                        "无法分析文件 {file_path} 的变更: before={before_err}, after={after_err}",
                    );
                }
            }
        }

        Ok(results)
    }

    /// 获取当前 git 仓库信息
    pub fn get_git_info(&self) -> Result<crate::architectural_impact::GitInfo, GitAIError> {
        let current_commit = self.get_current_commit()?;
        let base_commit = self.get_base_commit("HEAD~1")?;
        let branch = self.get_current_branch().ok(); // 分支信息是可选的

        Ok(crate::architectural_impact::GitInfo {
            current_commit,
            base_commit,
            branch,
        })
    }

    /// 获取当前提交 hash
    fn get_current_commit(&self) -> Result<String, GitAIError> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .map_err(|e| {
                GitAIError::Git(format!(
                    "无法执行 git rev-parse: {e}"
                ))
            })?;

        if !output.status.success() {
            return Err(GitAIError::Git(
                "无法获取当前提交hash".to_string()
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// 获取基准提交 hash
    fn get_base_commit(&self, commit_ref: &str) -> Result<String, GitAIError> {
        let output = Command::new("git")
            .args(["rev-parse", commit_ref])
            .output()
            .map_err(|e| {
                GitAIError::Git(format!(
                    "无法执行 git rev-parse: {e}"
                ))
            })?;

        if !output.status.success() {
            return Err(GitAIError::Git(format!(
                "无法获取提交hash: {commit_ref}"
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// 获取当前分支名
    fn get_current_branch(&self) -> Result<String, GitAIError> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .map_err(|e| {
                GitAIError::Git(format!(
                    "无法执行 git rev-parse: {e}"
                ))
            })?;

        if !output.status.success() {
            return Err(GitAIError::Git(
                "无法获取当前分支名".to_string()
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// 分析 git diff 并返回架构影响
    pub async fn analyze_git_diff(
        &self,
        diff: &str,
    ) -> Result<ArchitecturalImpact, Box<dyn std::error::Error + Send + Sync>> {
        let mut function_changes = Vec::new();
        let mut struct_changes = Vec::new();
        let mut interface_changes = Vec::new();
        let mut affected_modules = Vec::new();
        let mut breaking_changes = Vec::new();

        // 解析 diff 内容
        let mut current_file = String::new();
        for line in diff.lines() {
            if line.starts_with("diff --git") {
                // 提取文件路径
                if let Some(path) = line.split_whitespace().last() {
                    current_file = path.trim_start_matches("b/").to_string();

                    // 添加到受影响模块列表
                    if !affected_modules.contains(&current_file) {
                        affected_modules.push(current_file.clone());
                    }
                }
            } else if line.starts_with("+") && !line.starts_with("+++") {
                // 分析添加的行
                let content = &line[1..];

                // 检测函数定义
                if content.contains("fn ")
                    || content.contains("function")
                    || content.contains("def ")
                {
                    if let Some(func_name) = extract_function_name(content) {
                        function_changes.push(FunctionChange {
                            name: func_name.clone(),
                            change_type: ChangeType::Added,
                            file_path: current_file.clone(),
                            description: format!("新增函数: {func_name}"),
                        });
                    }
                }

                // 检测结构体定义
                if content.contains("struct ") || content.contains("class ") {
                    if let Some(struct_name) = extract_struct_name(content) {
                        struct_changes.push(StructChange {
                            name: struct_name.clone(),
                            change_type: ChangeType::Added,
                            file_path: current_file.clone(),
                            description: format!("新增结构体: {struct_name}"),
                        });
                    }
                }

                // 检测接口定义
                if content.contains("trait ") || content.contains("interface ") {
                    if let Some(interface_name) = extract_interface_name(content) {
                        interface_changes.push(InterfaceChange {
                            name: interface_name.clone(),
                            change_type: ChangeType::Added,
                            file_path: current_file.clone(),
                            description: format!("新增接口: {interface_name}"),
                        });
                        // 接口变更通常是破坏性的
                        breaking_changes
                            .push(format!("接口变更: {interface_name} in {current_file}"));
                    }
                }
            } else if line.starts_with("-") && !line.starts_with("---") {
                // 分析删除的行
                let content = &line[1..];

                // 检测函数删除
                if content.contains("fn ")
                    || content.contains("function")
                    || content.contains("def ")
                {
                    if let Some(func_name) = extract_function_name(content) {
                        function_changes.push(FunctionChange {
                            name: func_name.clone(),
                            change_type: ChangeType::Removed,
                            file_path: current_file.clone(),
                            description: format!("删除函数: {func_name}"),
                        });
                        // 函数删除是破坏性变更
                        breaking_changes.push(format!("函数删除: {func_name} in {current_file}"));
                    }
                }
            }
        }

        // 确定风险级别
        let risk_level = if !breaking_changes.is_empty() {
            "High".to_string()
        } else if !function_changes.is_empty() || !struct_changes.is_empty() {
            "Medium".to_string()
        } else {
            "Low".to_string()
        };

        Ok(ArchitecturalImpact {
            function_changes,
            struct_changes,
            interface_changes,
            impact_summary: ImpactSummary {
                affected_modules,
                breaking_changes,
                risk_level,
            },
        })
    }
}

/// 从文件路径推断编程语言
pub fn infer_language_from_path(
    path: &str,
) -> Result<SupportedLanguage, Box<dyn std::error::Error + Send + Sync>> {
    let extension = std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| "无法确定文件类型".to_string())?;

    SupportedLanguage::from_extension(extension)
        .ok_or_else(|| format!("不支持的文件扩展名: {extension}").into())
}

/// 检查是否为代码文件
pub fn is_code_file(path: &str) -> bool {
    infer_language_from_path(path).is_ok()
}

/// 提取函数名称
fn extract_function_name(line: &str) -> Option<String> {
    // Rust
    if let Some(pos) = line.find("fn ") {
        let rest = &line[pos + 3..];
        if let Some(end) = rest.find('(') {
            return Some(rest[..end].trim().to_string());
        }
    }

    // JavaScript/TypeScript
    if let Some(pos) = line.find("function ") {
        let rest = &line[pos + 9..];
        if let Some(end) = rest.find('(') {
            return Some(rest[..end].trim().to_string());
        }
    }

    // Python
    if let Some(pos) = line.find("def ") {
        let rest = &line[pos + 4..];
        if let Some(end) = rest.find('(') {
            return Some(rest[..end].trim().to_string());
        }
    }

    None
}

/// 提取结构体名称
fn extract_struct_name(line: &str) -> Option<String> {
    // Rust
    if let Some(pos) = line.find("struct ") {
        let rest = &line[pos + 7..];
        if let Some(end) = rest.find(|c: char| c == '{' || c == '<' || c.is_whitespace()) {
            return Some(rest[..end].trim().to_string());
        }
    }

    // Java/C++/JavaScript
    if let Some(pos) = line.find("class ") {
        let rest = &line[pos + 6..];
        if let Some(end) =
            rest.find(|c: char| c == '{' || c == '<' || c == ':' || c.is_whitespace())
        {
            return Some(rest[..end].trim().to_string());
        }
    }

    None
}

/// 提取接口名称
fn extract_interface_name(line: &str) -> Option<String> {
    // Rust
    if let Some(pos) = line.find("trait ") {
        let rest = &line[pos + 6..];
        if let Some(end) = rest.find(|c: char| c == '{' || c == '<' || c.is_whitespace()) {
            return Some(rest[..end].trim().to_string());
        }
    }

    // Java/TypeScript
    if let Some(pos) = line.find("interface ") {
        let rest = &line[pos + 10..];
        if let Some(end) = rest.find(|c: char| c == '{' || c == '<' || c.is_whitespace()) {
            return Some(rest[..end].trim().to_string());
        }
    }

    None
}

/// 便利函数：为单个文件执行完整的架构影响分析
pub async fn analyze_file_architectural_impact(
    file_path: &str,
    base_commit: Option<&str>,
) -> Result<
    crate::architectural_impact::ArchitecturalImpactAnalysis,
    Box<dyn std::error::Error + Send + Sync>,
> {
    let mut analyzer = GitStateAnalyzer::new_async().await?;

    let before_summary = analyzer
        .analyze_before_state(file_path, base_commit)
        .await?;
    let after_summary = analyzer.analyze_after_state(file_path).await?;

    let mut analysis = crate::architectural_impact::ast_comparison::compare_structural_summaries(
        &before_summary,
        &after_summary,
    );

    // 添加文件路径信息
    for change in &mut analysis.breaking_changes {
        change.file_path = file_path.to_string();
    }

    // 添加 git 信息
    if let Ok(git_info) = analyzer.get_git_info() {
        analysis.metadata.git_info = Some(git_info);
    }

    analysis.metadata.affected_files = vec![file_path.to_string()];

    Ok(analysis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_inference() {
        assert!(matches!(
            infer_language_from_path("test.rs"),
            Ok(SupportedLanguage::Rust)
        ));
        assert!(matches!(
            infer_language_from_path("test.java"),
            Ok(SupportedLanguage::Java)
        ));
        assert!(matches!(
            infer_language_from_path("test.py"),
            Ok(SupportedLanguage::Python)
        ));
        assert!(infer_language_from_path("test.txt").is_err());
    }

    #[test]
    fn test_is_code_file() {
        assert!(is_code_file("main.rs"));
        assert!(is_code_file("App.java"));
        assert!(is_code_file("script.py"));
        assert!(!is_code_file("README.md"));
        assert!(!is_code_file("config.json"));
    }

    #[tokio::test]
    async fn test_git_state_analyzer_creation() {
        let result = GitStateAnalyzer::new_async().await;
        assert!(result.is_ok());
    }

    // 注意：以下测试需要在 git 仓库中运行
    #[tokio::test]
    #[ignore] // 需要 git 环境，在 CI 中可能失败
    async fn test_get_changed_files() {
        let analyzer = GitStateAnalyzer::new_async().await.unwrap();
        let result = analyzer.get_changed_files();
        // 这个测试可能会因为没有变更而返回空列表，这是正常的
        assert!(result.is_ok());
    }
}

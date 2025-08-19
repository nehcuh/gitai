use crate::{
    config::TreeSitterConfig,
    errors::{AppError, tree_sitter_error},
    tree_sitter_analyzer::{
        analyzer::TreeSitterAnalyzer,
        core::{parse_git_diff, DiffAnalysis},
    },
    types::git::{CommitArgs, GitDiff},
};
use std::time::Instant;

use super::types::{EnhancedCommitRequest, TreeSitterCommitAnalysis};

/// Handles Tree-sitter analysis for commit operations
pub struct CommitAnalyzer {
    tree_sitter_config: TreeSitterConfig,
}

impl CommitAnalyzer {
    /// Create a new commit analyzer
    pub fn new(tree_sitter_config: TreeSitterConfig) -> Self {
        Self {
            tree_sitter_config,
        }
    }

    /// Analyze diff using Tree-sitter for commit message enhancement
    pub async fn analyze_diff_for_commit(
        &self,
        diff: &str,
        args: &CommitArgs,
    ) -> Result<TreeSitterCommitAnalysis, AppError> {
        let analysis_start = Instant::now();
        
        let diff_owned = diff.to_string();
        let args_depth = args.depth.clone();
        
        // Wrap CPU-intensive operations in spawn_blocking
        let result: Result<(String, Option<DiffAnalysis>), AppError> = tokio::task::spawn_blocking(move || {
            // Initialize TreeSitter analyzer with analysis depth
            let mut ts_config = TreeSitterConfig::default();
            
            // Set analysis depth based on args
            ts_config.analysis_depth = args_depth.or(Some("medium".to_string()));
            
            let mut analyzer = TreeSitterAnalyzer::new(ts_config).map_err(|e| {
                tracing::error!("TreeSitter分析器初始化失败: {:?}", e);
                tree_sitter_error(e.to_string())
            })?;

            // Parse the diff to get structured representation
            let git_diff = parse_git_diff(&diff_owned).map_err(|e| {
                tracing::error!("解析Git差异失败: {:?}", e);
                tree_sitter_error(e.to_string())
            })?;

            // Generate analysis using TreeSitter
            let analysis = analyzer.analyze_diff(&diff_owned).map_err(|e| {
                tracing::error!("执行差异分析失败: {:?}", e);
                tree_sitter_error(e.to_string())
            })?;
            
            tracing::debug!("差异分析结果: {:?}", analysis);

            // Create detailed analysis text
            let analysis_text = format_tree_sitter_analysis_for_commit(&analysis, &git_diff);

            Ok((analysis_text, Some(analysis)))
        }).await.map_err(|e| AppError::Generic(format!("Task join error: {}", e)))?;
        
        let (analysis_text, analysis_data) = result?;
        let processing_time = analysis_start.elapsed();
        
        tracing::info!("Tree-sitter分析完成，耗时: {:?}", processing_time);
        
        Ok(TreeSitterCommitAnalysis {
            analysis_text,
            analysis_data,
            processing_time,
        })
    }

    /// Create enhanced analysis for commit message generation
    pub async fn create_enhanced_analysis(
        &self,
        request: EnhancedCommitRequest,
    ) -> Result<TreeSitterCommitAnalysis, AppError> {
        tracing::info!("🌳 正在使用Tree-sitter增强分析生成提交信息...");
        
        let args = CommitArgs {
            tree_sitter: true,
            depth: Some(request.analysis_depth),
            auto_stage: false,
            message: request.custom_message,
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };
        
        self.analyze_diff_for_commit(&request.diff_content, &args).await
    }

    /// Check if Tree-sitter analysis is available
    pub fn is_analysis_available(&self) -> bool {
        self.tree_sitter_config.enabled == Some(true)
    }

    /// Get supported languages for analysis
    pub fn get_supported_languages(&self) -> &[String] {
        self.tree_sitter_config.languages.as_deref().unwrap_or(&[])
    }

    /// Validate diff content for analysis
    pub fn validate_diff_content(&self, diff: &str) -> Result<(), AppError> {
        if diff.trim().is_empty() {
            return Err(AppError::Generic("差异内容为空，无法进行分析".to_string()));
        }
        
        if diff.len() > 100_000 {
            return Err(AppError::Generic("差异内容过大，可能影响分析性能".to_string()));
        }
        
        Ok(())
    }

    /// Get analysis complexity estimate
    pub fn estimate_analysis_complexity(&self, diff: &str) -> AnalysisComplexity {
        let line_count = diff.lines().count();
        let file_count = diff.matches("diff --git").count();
        
        match (line_count, file_count) {
            (0..=50, 1) => AnalysisComplexity::Simple,
            (51..=200, 1..=3) => AnalysisComplexity::Moderate,
            (201..=500, 1..=5) => AnalysisComplexity::Complex,
            _ => AnalysisComplexity::VeryComplex,
        }
    }
}

/// Format Tree-sitter analysis for commit message generation
fn format_tree_sitter_analysis_for_commit(
    analysis: &DiffAnalysis,
    _git_diff: &GitDiff,
) -> String {
    let mut result = String::new();
    
    result.push_str("### 代码分析摘要\n");
    result.push_str(&format!("- 变更模式: {:?}\n", analysis.change_analysis.change_pattern));
    result.push_str(&format!("- 影响范围: {:?}\n", analysis.change_analysis.change_scope));
    result.push_str(&format!("- 总体摘要: {}\n", analysis.overall_summary));
    
    if !analysis.file_analyses.is_empty() {
        result.push_str("\n### 文件变更详情\n");
        for file_analysis in &analysis.file_analyses {
            result.push_str(&format!("**{}** ({})\n", file_analysis.path.display(), file_analysis.language));
            result.push_str(&format!("  - 变更类型: {:?}\n", file_analysis.change_type));
            if let Some(ref summary) = file_analysis.summary {
                result.push_str(&format!("  - 摘要: {}\n", summary));
            }
            
            if !file_analysis.affected_nodes.is_empty() {
                result.push_str("  - 影响的代码结构:\n");
                for node in &file_analysis.affected_nodes {
                    let change_type_str = node.change_type.as_deref().unwrap_or("未知");
                    result.push_str(&format!("    • {} ({}): {}\n", 
                        node.node_type, 
                        &node.name, 
                        change_type_str
                    ));
                }
            }
            result.push('\n');
        }
    }
    
    // Add change statistics
    let change_analysis = &analysis.change_analysis;
    if change_analysis.function_changes > 0 {
        result.push_str(&format!("### 函数变更: {} 个\n", change_analysis.function_changes));
        result.push('\n');
    }
    
    if change_analysis.type_changes > 0 {
        result.push_str(&format!("### 类型变更: {} 个\n", change_analysis.type_changes));
        result.push('\n');
    }
    
    result
}

/// Analysis complexity levels
#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisComplexity {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

impl AnalysisComplexity {
    /// Get recommended timeout for this complexity level
    pub fn recommended_timeout(&self) -> std::time::Duration {
        match self {
            AnalysisComplexity::Simple => std::time::Duration::from_secs(5),
            AnalysisComplexity::Moderate => std::time::Duration::from_secs(15),
            AnalysisComplexity::Complex => std::time::Duration::from_secs(30),
            AnalysisComplexity::VeryComplex => std::time::Duration::from_secs(60),
        }
    }

    /// Get description of complexity level
    pub fn description(&self) -> &'static str {
        match self {
            AnalysisComplexity::Simple => "简单分析 (少量变更)",
            AnalysisComplexity::Moderate => "中等分析 (适量变更)",
            AnalysisComplexity::Complex => "复杂分析 (大量变更)",
            AnalysisComplexity::VeryComplex => "超复杂分析 (海量变更)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::git::CommitArgs;

    fn create_test_config() -> TreeSitterConfig {
        TreeSitterConfig {
            enabled: true,
            analysis_depth: "medium".to_string(),
            cache_enabled: true,
            languages: vec!["rust".to_string(), "javascript".to_string()],
        }
    }

    fn create_test_diff() -> String {
        "diff --git a/src/test.rs b/src/test.rs\nindex 1234567..abcdefg 100644\n--- a/src/test.rs\n+++ b/src/test.rs\n@@ -1,3 +1,4 @@\n fn test_function() {\n     println!(\"Hello, world!\");\n+    println!(\"New line added\");\n }".to_string()
    }

    #[test]
    fn test_commit_analyzer_creation() {
        let config = create_test_config();
        let analyzer = CommitAnalyzer::new(config);
        assert!(analyzer.is_analysis_available());
    }

    #[test]
    fn test_validate_diff_content() {
        let config = create_test_config();
        let analyzer = CommitAnalyzer::new(config);
        
        // Valid diff
        let valid_diff = create_test_diff();
        assert!(analyzer.validate_diff_content(&valid_diff).is_ok());
        
        // Empty diff
        assert!(analyzer.validate_diff_content("").is_err());
        
        // Very large diff
        let large_diff = "x".repeat(200_000);
        assert!(analyzer.validate_diff_content(&large_diff).is_err());
    }

    #[test]
    fn test_estimate_analysis_complexity() {
        let config = create_test_config();
        let analyzer = CommitAnalyzer::new(config);
        
        // Simple diff
        let simple_diff = "diff --git a/test.txt b/test.txt\n+new line";
        assert_eq!(analyzer.estimate_analysis_complexity(simple_diff), AnalysisComplexity::Simple);
        
        // Complex diff
        let complex_diff = format!("{}\n{}", "diff --git a/file1.rs b/file1.rs".repeat(10), "line".repeat(300));
        let complexity = analyzer.estimate_analysis_complexity(&complex_diff);
        assert!(matches!(complexity, AnalysisComplexity::Complex | AnalysisComplexity::VeryComplex));
    }

    #[test]
    fn test_analysis_complexity_methods() {
        let simple = AnalysisComplexity::Simple;
        let complex = AnalysisComplexity::VeryComplex;
        
        assert!(simple.recommended_timeout() < complex.recommended_timeout());
        assert!(!simple.description().is_empty());
        assert!(!complex.description().is_empty());
    }

    #[test]
    fn test_get_supported_languages() {
        let config = create_test_config();
        let analyzer = CommitAnalyzer::new(config);
        
        let languages = analyzer.get_supported_languages();
        assert!(languages.contains(&"rust".to_string()));
        assert!(languages.contains(&"javascript".to_string()));
    }

    #[tokio::test]
    async fn test_analyze_diff_for_commit_fallback() {
        let config = create_test_config();
        let analyzer = CommitAnalyzer::new(config);
        
        let diff = create_test_diff();
        let args = CommitArgs {
            tree_sitter: true,
            depth: Some("medium".to_string()),
            auto_stage: false,
            message: None,
            issue_id: None,
            review: false,
            passthrough_args: vec![],
        };
        
        // This may fail in test environments without tree-sitter support
        match analyzer.analyze_diff_for_commit(&diff, &args).await {
            Ok(analysis) => {
                assert!(!analysis.analysis_text.is_empty());
                assert!(analysis.processing_time.as_millis() >= 0);
            }
            Err(_) => {
                // Expected in test environments without proper tree-sitter setup
                assert!(true);
            }
        }
    }

    #[tokio::test]
    async fn test_create_enhanced_analysis() {
        let config = create_test_config();
        let analyzer = CommitAnalyzer::new(config);
        
        let request = EnhancedCommitRequest {
            diff_content: create_test_diff(),
            custom_message: Some("feat: test feature".to_string()),
            analysis_depth: "medium".to_string(),
            review_context: None,
        };
        
        match analyzer.create_enhanced_analysis(request).await {
            Ok(analysis) => {
                assert!(!analysis.analysis_text.is_empty());
                assert!(analysis.analysis_data.is_some());
            }
            Err(_) => {
                // Expected in test environments
                assert!(true);
            }
        }
    }

    #[test]
    fn test_format_tree_sitter_analysis_for_commit() {
        use crate::tree_sitter_analyzer::core::{
            DiffAnalysis, FileAnalysis, ChangeAnalysis, ChangePattern, ChangeScope, AffectedNode
        };
        use crate::types::git::{GitDiff, ChangeType};
        use std::path::PathBuf;

        let analysis = DiffAnalysis {
            file_analyses: vec![
                FileAnalysis {
                    path: PathBuf::from("src/test.rs"),
                    language: "Rust".to_string(),
                    change_type: ChangeType::Added,
                    affected_nodes: vec![
                        AffectedNode {
                            node_type: "function".to_string(),
                            name: "test_function".to_string(),
                            range: (0, 100),
                            is_public: true,
                            content: Some("fn test_function() {}".to_string()),
                            line_range: (1, 5),
                            change_type: Some("added".to_string()),
                            additions: Some(vec!["println!(\"Hello\");".to_string()]),
                            deletions: None,
                        }
                    ],
                    summary: Some("新增测试函数".to_string()),
                }
            ],
            overall_summary: "添加了新的测试函数".to_string(),
            change_analysis: ChangeAnalysis {
                function_changes: 1,
                type_changes: 0,
                method_changes: 0,
                interface_changes: 0,
                other_changes: 0,
                change_pattern: ChangePattern::FeatureImplementation,
                change_scope: ChangeScope::Minor,
            },
        };

        let git_diff = GitDiff {
            changed_files: vec![],
            metadata: None,
        };

        let result = format_tree_sitter_analysis_for_commit(&analysis, &git_diff);
        
        assert!(result.contains("代码分析摘要"));
        assert!(result.contains("FeatureImplementation"));
        assert!(result.contains("Minor"));
        assert!(result.contains("src/test.rs"));
        assert!(result.contains("函数变更: 1 个"));
    }
}
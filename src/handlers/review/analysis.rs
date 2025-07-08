use crate::{
    config::TreeSitterConfig,
    errors::AppError,
    handlers::analysis::AIAnalysisEngine,
    tree_sitter_analyzer::{
        analyzer::TreeSitterAnalyzer,
        core::{detect_language_from_extension, parse_git_diff},
    },
    types::{
        ai::{AnalysisDepth, AnalysisRequest, OutputFormat},
        git::GitDiff,
    },
};
use std::{collections::HashMap, sync::Arc};
use tracing;

use super::types::{DiffAnalysisResult, TreeSitterAnalysis};

/// Handles diff analysis using TreeSitter and simple text analysis
pub struct DiffAnalyzer {
    tree_sitter_config: TreeSitterConfig,
    ai_engine: Arc<AIAnalysisEngine>,
}

impl DiffAnalyzer {
    /// Create a new diff analyzer
    pub fn new(tree_sitter_config: TreeSitterConfig, ai_engine: Arc<AIAnalysisEngine>) -> Self {
        Self {
            tree_sitter_config,
            ai_engine,
        }
    }

    /// Analyze diff with optional TreeSitter analysis
    pub async fn analyze_diff(
        &self,
        diff_text: &str,
        use_tree_sitter: bool,
    ) -> Result<DiffAnalysisResult, AppError> {
        let git_diff = parse_git_diff(diff_text)?;
        let language_info = self.extract_language_info(&git_diff);

        let (analysis_text, tree_sitter_analysis) = if use_tree_sitter {
            self.analyze_diff_with_tree_sitter(diff_text, &git_diff)
                .await?
        } else {
            (
                self.analyze_diff_simple(diff_text, &git_diff).await?,
                None,
            )
        };

        Ok(DiffAnalysisResult {
            git_diff,
            analysis_text,
            tree_sitter_analysis,
            language_info,
        })
    }

    /// Extract language information from the diff
    pub fn extract_language_info(&self, diff: &GitDiff) -> String {
        let mut languages = Vec::new();
        let mut file_count = 0;

        for file in &diff.changed_files {
            file_count += 1;
            if let Some(extension) = file.path.extension() {
                if let Some(ext_str) = extension.to_str() {
                    if let Some(lang) = detect_language_from_extension(ext_str) {
                        if !languages.contains(&lang) {
                            languages.push(lang);
                        }
                    }
                }
            }
        }

        if languages.is_empty() {
            format!("未知语言类型 ({} 个文件)", file_count)
        } else {
            let lang_str = languages.join(", ");
            format!("{} ({} 个文件)", lang_str, file_count)
        }
    }

    /// Perform TreeSitter-based analysis
    async fn analyze_diff_with_tree_sitter(
        &self,
        diff_text: &str,
        git_diff: &GitDiff,
    ) -> Result<(String, Option<TreeSitterAnalysis>), AppError> {
        tracing::info!("执行 TreeSitter 分析");

        // Initialize TreeSitter analyzer
        let mut analyzer = TreeSitterAnalyzer::new(self.tree_sitter_config.clone())?;

        // Perform analysis request
        let analysis_request = AnalysisRequest {
            work_items: Vec::new(), // No work items for TreeSitter analysis
            git_diff: diff_text.to_string(),
            focus_areas: None,
            analysis_depth: AnalysisDepth::Basic,
            output_format: OutputFormat::Text,
        };

        match self.ai_engine.analyze_with_requirements(analysis_request).await {
            Ok(analysis_result) => {
                // Format the analysis result into a readable string
                let content = format!(
                    "代码质量分析:\n评分: {}/100\n结构评估: {}\n",
                    analysis_result.code_quality.quality_score,
                    analysis_result.code_quality.structure_assessment
                );
                
                let formatted_analysis = self.format_tree_sitter_analysis(&content);

                // Create TreeSitter analysis structure
                let tree_sitter_analysis = TreeSitterAnalysis {
                    structural_changes: content.clone(),
                    complexity_metrics: HashMap::new(), // Could be populated from actual analysis
                    affected_nodes: vec![], // Could be populated from actual analysis
                };

                Ok((formatted_analysis, Some(tree_sitter_analysis)))
            }
            Err(e) => {
                tracing::warn!("TreeSitter 分析失败，回退到简单分析: {:?}", e);
                let simple_analysis = self.analyze_diff_simple(diff_text, git_diff).await?;
                Ok((simple_analysis, None))
            }
        }
    }

    /// Perform simple text-based analysis
    async fn analyze_diff_simple(
        &self,
        diff_text: &str,
        git_diff: &GitDiff,
    ) -> Result<String, AppError> {
        tracing::info!("执行简单差分析");

        let mut analysis = String::new();
        analysis.push_str("=== 代码变更分析 ===\n\n");

        // File count and types
        analysis.push_str(&format!(
            "📁 变更文件数量: {} 个\n",
            git_diff.changed_files.len()
        ));

        // Analyze each file
        for file in &git_diff.changed_files {
            analysis.push_str(&format!("📝 文件: {}\n", file.path.display()));
            analysis.push_str(&format!("   变更类型: {:?}\n", file.change_type));
            analysis.push_str(&format!("   变更块数量: {}\n", file.hunks.len()));

            // Calculate approximate additions/deletions from hunks
            let mut total_additions = 0;
            let mut total_deletions = 0;
            for hunk in &file.hunks {
                total_additions += hunk.new_range.count;
                if hunk.old_range.count > 0 {
                    total_deletions += hunk.old_range.count;
                }
            }
            
            if total_additions > 0 {
                analysis.push_str(&format!("   新增行数: {}\n", total_additions));
            }
            if total_deletions > 0 {
                analysis.push_str(&format!("   删除行数: {}\n", total_deletions));
            }
        }

        // Basic statistics from diff content
        let lines: Vec<&str> = diff_text.lines().collect();
        let added_lines = lines.iter().filter(|line| line.starts_with('+')).count();
        let removed_lines = lines.iter().filter(|line| line.starts_with('-')).count();

        analysis.push_str(&format!("\n📊 变更统计:\n"));
        analysis.push_str(&format!("   新增: {} 行\n", added_lines));
        analysis.push_str(&format!("   删除: {} 行\n", removed_lines));
        analysis.push_str(&format!("   净变更: {} 行\n", added_lines as i32 - removed_lines as i32));

        Ok(analysis)
    }

    /// Format TreeSitter analysis results
    fn format_tree_sitter_analysis(&self, analysis_content: &str) -> String {
        let mut formatted = String::new();
        formatted.push_str("=== TreeSitter 代码结构分析 ===\n\n");
        formatted.push_str(analysis_content);
        formatted.push_str("\n");
        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::git::{ChangedFile, ChangeType};

    fn create_test_diff() -> GitDiff {
        GitDiff {
            changed_files: vec![
                ChangedFile {
                    path: "src/main.rs".to_string(),
                    change_type: ChangeType::Modified,
                    additions: Some(10),
                    deletions: Some(5),
                    file_mode_change: None,
                },
                ChangedFile {
                    path: "src/lib.rs".to_string(),
                    change_type: ChangeType::Added,
                    additions: Some(20),
                    deletions: Some(0),
                    file_mode_change: None,
                },
            ],
            metadata: None,
        }
    }

    #[test]
    fn test_extract_language_info() {
        let tree_sitter_config = TreeSitterConfig::default();
        let ai_engine = Arc::new(AIAnalysisEngine::new(Default::default()));
        let analyzer = DiffAnalyzer::new(tree_sitter_config, ai_engine);

        let diff = create_test_diff();
        let language_info = analyzer.extract_language_info(&diff);

        assert!(language_info.contains("Rust"));
        assert!(language_info.contains("2 个文件"));
    }

    #[tokio::test]
    async fn test_analyze_diff_simple() {
        let tree_sitter_config = TreeSitterConfig::default();
        let ai_engine = Arc::new(AIAnalysisEngine::new(Default::default()));
        let analyzer = DiffAnalyzer::new(tree_sitter_config, ai_engine);

        let diff_text = r#"
diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,5 @@
 fn main() {
+    println!("Hello, world!");
+    println!("New feature!");
     // existing code
 }
"#;

        let git_diff = create_test_diff();
        let result = analyzer.analyze_diff_simple(diff_text, &git_diff).await;

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.contains("代码变更分析"));
        assert!(analysis.contains("变更文件数量: 2"));
    }
}
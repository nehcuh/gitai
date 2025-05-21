use std::{path::PathBuf, time::SystemTime};

use tree_sitter::{Language, Tree};

use crate::core::errors::TreeSitterError;

/// Add missing variants to TreeSitterError if needed
/// Uncomment these if these variants are required
// impl TreeSitterError {
//     pub fn LanguageError(msg: String) -> Self {
//         TreeSitterError::UnsupportedLanguage(msg)
//     }
// }

// Rust Ast Parser
pub fn get_tree_sitter_rust() -> Language {
    tree_sitter_rust::language()
}

// Java Ast Parser
pub fn get_tree_sitter_java() -> Language {
    tree_sitter_java::language()
}

// Python Ast Parser
pub fn get_tree_sitter_python() -> Language {
    tree_sitter_python::language()
}

// Go Ast Parser
pub fn get_tree_sitter_go() -> Language {
    tree_sitter_go::language()
}

// JS Ast Parser
pub fn get_tree_sitter_js() -> Language {
    tree_sitter_javascript::language()
}

// File AST structure
// This structure represents the abstract syntax tree (AST) of a file
// Uses the actual Tree type provided by the tree-sitter
#[derive(Debug, Clone)]
pub struct FileAst {
    /// File path
    pub path: PathBuf,
    /// tree-sitter parse tree
    pub tree: Tree,
    /// Source code
    pub source: String,
    /// Content hash value
    pub content_hash: String,
    /// Last parsed time
    #[allow(dead_code)]
    pub last_parsed: SystemTime,
    /// Language identifier
    pub language_id: String,
}

impl From<std::io::Error> for TreeSitterError {
    fn from(value: std::io::Error) -> Self {
        TreeSitterError::IOError(e)
    }
}

// Defines the type of change in a Git diff
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    #[allow(dead_code)]
    Copied,
    #[allow(dead_code)]
    TypeChanged,
}

// Represents a range of lines in a diff hunk
#[derive(Debug, Clone)]
pub struct LineRange {
    pub start: u32,
    pub count: u32,
}

impl LineRange {
    #[allow(dead_code)]
    pub fn new(start: u32, count: u32) -> Self {
        Self { start, count }
    }
}

// Represents a hunk range in git diff format (@@ -a,b +c,d @@)
#[derive(Debug, Clone)]
pub struct HunkRange {
    pub start: usize,
    pub count: usize,
}

// Represents a single hunk in a Git diff
#[derive(Debug, Clone)]
pub struct DiffHunk {
    #[allow(dead_code)]
    pub old_range: HunkRange,
    pub new_range: HunkRange,
    #[allow(dead_code)]
    pub lines: Vec<String>,
}

// Legacy structure, keeping this for backward compatibility
// but we're migrating to ChangedFile
pub struct FileDiff {
    pub path: PathBuf,
    pub old_path: Option<PathBuf>,
    pub change_type: ChangeType,
    pub hunks: Vec<DiffHunk>,
}

// Conversion functions between FileDiff and ChangedFile
impl From<FileDiff> for ChangedFile {
    fn from(file_diff: FileDiff) -> Self {
        ChangedFile {
            path: file_diff.path,
            change_type: file_diff.change_type,
            hunks: file_diff.hunks,
            file_mode_change: None,
        }
    }
}

impl From<ChangedFile> for FileDiff {
    fn from(changed_file: ChangedFile) -> Self {
        FileDiff {
            path: changed_file.path,
            old_path: None,
            change_type: changed_file.change_type,
            hunks: changed_file.hunks,
        }
    }
}

// Represents a changed file in a Git diff
#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub hunks: Vec<DiffHunk>,
    pub file_mode_change: Option<String>,
}

// Represents the entire Git diff
#[derive(Debug, Clone)]
pub struct GitDiff {
    pub changed_files: Vec<ChangedFile>,
    pub metadata: Option<HashMap<String, String>>,
}

// Represents a node in the AST affected by changes
#[derive(Debug, Clone)]
pub struct AffectedNode {
    pub node_type: String,
    pub name: String,
    pub range: (usize, usize),
    pub is_public: bool,
    #[allow(dead_code)]
    pub content: Option<String>,
    #[allow(dead_code)]
    pub line_range: (usize, usize),
}

impl AffectedNode {
    pub fn new(node_type: String, name: String, range: (usize, usize), is_public: bool) -> Self {
        Self {
            node_type,
            name,
            range,
            is_public,
            content: None,
            line_range: (0, 0),
        }
    }
}

// Analysis of a single file
pub struct FileAnalysis {
    pub path: PathBuf,
    #[allow(dead_code)]
    pub language: String,
    #[allow(dead_code)]
    pub change_type: ChangeType,
    pub affected_nodes: Vec<AffectedNode>,
    pub summary: Option<String>,
}

// Analysis of changes in a diff
#[derive(Debug, Clone, Default)]
pub struct ChangeAnalysis {
    #[allow(dead_code)]
    pub function_changes: usize,
    #[allow(dead_code)]
    pub type_changes: usize,
    #[allow(dead_code)]
    pub method_changes: usize,
    #[allow(dead_code)]
    pub interface_changes: usize,
    #[allow(dead_code)]
    pub other_changes: usize,
    #[allow(dead_code)]
    pub change_pattern: ChangePattern,
    #[allow(dead_code)]
    pub change_scope: ChangeScope,
}

// Complete analysis of a Git diff
#[derive(Debug, Clone)]
pub struct DiffAnalysis {
    pub file_analyses: Vec<FileAnalysis>,
    pub overall_summary: String,
    #[allow(dead_code)]
    pub change_analysis: ChangeAnalysis,
}

// Mapping between diff and AST
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DiffAstMapping {
    pub files: HashMap<String, FileDiffAstMapping>,
}

// Mapping for a single file
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileDiffAstMapping {
    pub file_path: String,
    pub hunks: Vec<HunkAstMapping>,
}

// Mapping for a single hunk
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HunkAstMapping {
    pub hunk: DiffHunk,
    pub nodes: Vec<AffectedNode>,
}

// Types of change patterns
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangePattern {
    #[allow(dead_code)]
    FeatureImplementation,
    #[allow(dead_code)]
    BugFix,
    #[allow(dead_code)]
    Refactoring,
    #[allow(dead_code)]
    ModelChange,
    #[allow(dead_code)]
    BehaviorChange,
    #[allow(dead_code)]
    ConfigurationChange,
    MixedChange,
    #[allow(dead_code)]
    JavaStructuralChange,
    #[allow(dead_code)]
    JavaVisibilityChange,
    #[allow(dead_code)]
    JavaAnnotationChange,
}

impl Default for ChangePattern {
    fn default() -> Self {
        ChangePattern::MixedChange
    }
}

// Scope of changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeScope {
    Minor,
    #[allow(dead_code)]
    Moderate,
    #[allow(dead_code)]
    Major,
}

impl Default for ChangeScope {
    fn default() -> Self {
        ChangeScope::Minor
    }
}

// Parse git diff output into a GitDiff structure
pub fn parse_git_diff(diff_text: &str) -> Result<GitDiff, TreeSitterError> {
    // Delegate to the newer parser implementation
    match crate::tree_sitter_analyzer::parse_utils::parse_git_diff_text(diff_text) {
        Ok(git_diff) => Ok(git_diff),
        Err(e) => Err(TreeSitterError::ParseError(format!(
            "Failed to parse diff: {}",
            e
        ))),
    }
}

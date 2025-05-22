use std::time::SystemTime;

use tree_sitter::{Language, Tree};

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

// 文件AST结构
// 这个结构体代表一个文件的语法分析树(AST)
// 使用tree-sitter提供的实际Tree类型
#[derive(Debug, Clone)]
pub struct FileAst {
    /// 文件路径
    pub path: PathBuf,
    /// tree-sitter解析树
    pub tree: Tree,
    /// 源代码
    pub source: String,
    /// 内容哈希值
    pub content_hash: String,
    /// 最后解析时间
    #[allow(dead_code)]
    pub last_parsed: SystemTime,
    /// 语言标识
    pub language_id: String,
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

// Rust 语言解析器
pub fn get_tree_sitter_rust() -> Language {
    tree_sitter_rust::language()
}

// Java 语言解析器
pub fn get_tree_sitter_java() -> Language {
    tree_sitter_java::language()
}

// Python 语言解析器
pub fn get_tree_sitter_python() -> Language {
    tree_sitter_python::language()
}

// Go 语言解析器
pub fn get_tree_sitter_go() -> Language {
    tree_sitter_go::language()
}

// JS 语言解析器
pub fn get_tree_sitter_js() -> Language {
    tree_sitter_javascript::language()
}

// C 语言解析器
pub fn get_tree_sitter_c() -> Language {
    tree_sitter_c::language()
}

// C++ 语言解析器
pub fn get_tree_sitter_cpp() -> Language {
    tree_sitter_cpp::language()
}

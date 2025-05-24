use std::{collections::HashMap, path::PathBuf, time::SystemTime};

use tree_sitter::{Tree, Language};

// Represents the entire Git diff
#[derive(Debug, Clone)]
pub struct GitDiff {
    pub changed_files: Vec<ChangedFile>,
    pub metadata: Option<HashMap<String, String>>,
}

/// 分析深度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnalysisDepth {
    /// 基础分析
    Basic,
    /// 标准分析
    Normal,
    /// 深度分析
    Deep,
}

// Complete analysis of a Git diff
#[derive(Debug, Clone)]
pub struct DiffAnalysis {
    pub file_analyses: Vec<FileAnalysis>,
    pub overall_summary: String,
    #[allow(dead_code)]
    pub change_analysis: ChangeAnalysis,
}

// Analysis of a single file
#[derive(Debug, Clone)]
pub struct FileAnalysis {
    pub path: PathBuf,
    #[allow(dead_code)]
    pub language: String,
    #[allow(dead_code)]
    pub change_type: ChangeType,
    pub affected_nodes: Vec<AffectedNode>,
    pub summary: Option<String>,
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

// Represents a node in the AST affected by changes
#[derive(Debug, Clone)]
pub struct AffectedNode {
    pub node_type: String,
    pub name: String,
    pub range: (usize, usize),
    pub is_public: bool,
    pub content: Option<String>,
    pub line_range: (usize, usize),
    pub change_type: Option<String>, // 新增：变更类型（added, deleted, modified）
    pub additions: Option<Vec<String>>, // 新增：添加的行
    pub deletions: Option<Vec<String>>, // 新增：删除的行
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
    LanguageSpecificChange(String),
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

#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub hunks: Vec<DiffHunk>,
    pub file_mode_change: Option<String>,
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

pub struct ChangeStats {
    pub node_type_counts: HashMap<String, usize>,
    pub change_type_counts: HashMap<String, usize>,
}

// Tree-sitter 语言配置结构
#[derive(Debug, Clone)]
pub struct LanguageConfig {
    pub name: &'static str,
    pub display_name: &'static str,
    pub extensions: &'static [&'static str],
    pub language_fn: fn() -> Language,
    pub highlights_query: &'static str,
    pub injections_query: &'static str,
    pub locals_query: &'static str,
}

impl LanguageConfig {
    pub fn get_language(&self) -> Language {
        (self.language_fn)()
    }

    pub fn get_full_queries(&self) -> (&'static str, &'static str, &'static str) {
        (self.highlights_query, self.injections_query, self.locals_query)
    }

    pub fn supports_extension(&self, ext: &str) -> bool {
        self.extensions.iter().any(|&e| e.eq_ignore_ascii_case(ext))
    }
}

// 语言配置注册表
pub struct LanguageRegistry {
    configs: HashMap<&'static str, LanguageConfig>,
}

impl LanguageRegistry {
    pub fn new() -> Self {
        // 这里会在后面实现具体的初始化逻辑
        Self {
            configs: HashMap::new(),
        }
    }

    pub fn get_config(&self, language: &str) -> Option<&LanguageConfig> {
        self.configs.get(language)
    }

    pub fn detect_language_by_extension(&self, extension: &str) -> Option<&LanguageConfig> {
        self.configs.values().find(|config| config.supports_extension(extension))
    }

    pub fn get_all_languages(&self) -> Vec<&str> {
        self.configs.keys().copied().collect()
    }

    pub fn is_supported(&self, language: &str) -> bool {
        self.configs.contains_key(language)
    }

    pub fn register_language(&mut self, config: LanguageConfig) {
        self.configs.insert(config.name, config);
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// 节点分析配置
#[derive(Debug, Clone)]
pub struct NodeAnalysisConfig {
    pub language: &'static str,
    pub capture_names: &'static [&'static str],
    pub important_nodes: &'static [&'static str],
    pub visibility_indicators: &'static [&'static str],
    pub scope_indicators: &'static [&'static str],
}

use std::{collections::HashMap, path::PathBuf, time::SystemTime};

use super::{
    analyzer::NodeAnalysisConfig,
    query_provider::{QueryProvider, QueryType},
};
use crate::{
    errors::{AppError, tree_sitter_error},
    types::git::{ChangeType, ChangedFile, DiffHunk, GitDiff, HunkRange},
};
use tree_sitter::{Language, Tree, Query};

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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiffAnalysis {
    pub file_analyses: Vec<FileAnalysis>,
    pub overall_summary: String,
    #[allow(dead_code)]
    pub change_analysis: ChangeAnalysis,
}

// Analysis of a single file
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileAnalysis {
    pub path: PathBuf,
    #[allow(dead_code)]
    pub language: String,
    #[allow(dead_code)]
    pub change_type: ChangeType,
    pub affected_nodes: Vec<AffectedNode>,
    pub summary: Option<String>,
}

// Represents a node in the AST affected by changes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
}

impl LanguageConfig {
    pub fn get_language(&self) -> Language {
        (self.language_fn)()
    }

    pub fn get_highlights_query(&self, provider: &QueryProvider) -> Result<Option<Query>, AppError> {
        provider.get_query_content(self.name, QueryType::Highlights)
            .map(|content| Query::new(&self.get_language(), content))
            .transpose()
            .map_err(|e| tree_sitter_error(format!("Failed to compile highlights query: {}", e)))
    }

    pub fn get_injections_query(&self, provider: &QueryProvider) -> Result<Option<Query>, AppError> {
        provider.get_query_content(self.name, QueryType::Injections)
            .filter(|content| !content.is_empty())
            .map(|content| Query::new(&self.get_language(), content))
            .transpose()
            .map_err(|e| tree_sitter_error(format!("Failed to compile injections query: {}", e)))
    }

    pub fn get_locals_query(&self, provider: &QueryProvider) -> Result<Option<Query>, AppError> {
        provider.get_query_content(self.name, QueryType::Locals)
            .filter(|content| !content.is_empty())
            .map(|content| Query::new(&self.get_language(), content))
            .transpose()
            .map_err(|e| tree_sitter_error(format!("Failed to compile locals query: {}", e)))
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
        self.configs
            .values()
            .find(|config| config.supports_extension(extension))
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


macro_rules! define_ts_langs {
    ($( $fn_name:ident => $module:ident => $doc:expr ),* $(,)?) => {
        $(
            #[doc = $doc]
            pub fn $fn_name() -> Language {
                $module::LANGUAGE.into()
            }
        )*
    };
}

// 一次性定义所有语言的 getter
define_ts_langs! {
    get_tree_sitter_rust  => tree_sitter_rust      => "Rust 语言解析器",
    get_tree_sitter_java  => tree_sitter_java      => "Java 语言解析器",
    get_tree_sitter_python=> tree_sitter_python    => "Python 语言解析器",
    get_tree_sitter_go    => tree_sitter_go        => "Go 语言解析器",
    get_tree_sitter_js    => tree_sitter_javascript=> "JS 语言解析器",
    get_tree_sitter_c     => tree_sitter_c         => "C 语言解析器",
    get_tree_sitter_cpp   => tree_sitter_cpp       => "C++ 语言解析器",
}

// TypeScript 使用 JavaScript 解析器
pub fn get_tree_sitter_typescript() -> Language {
    tree_sitter_javascript::LANGUAGE.into()
}


// 语言检测优化
pub fn detect_language_from_extension(extension: &str) -> Option<&'static str> {
    match extension.to_lowercase().as_str() {
        "rs" => Some("rust"),
        "java" => Some("java"),
        "py" | "pyw" => Some("python"),
        "go" => Some("go"),
        "js" | "mjs" | "jsx" => Some("js"),
        "ts" | "tsx" => Some("js"), // TypeScript也使用JavaScript解析器
        "c" | "h" => Some("c"),
        "cpp" | "cc" | "cxx" | "c++" | "hpp" | "hxx" | "h++" => Some("cpp"),
        _ => None,
    }
}

// 获取当前实现了 tree-sitter 解析器的语言列表
pub fn get_implemented_tree_sitter_languages() -> &'static [&'static str] {
    &["rust", "java", "python", "go", "javascript", "typescript", "c", "cpp"]
}

// 向后兼容的别名
pub fn get_supported_languages() -> &'static [&'static str] {
    get_implemented_tree_sitter_languages()
}

// 检查语言是否被支持（任何形式的支持）
pub fn is_language_supported(language: &str) -> bool {
    get_all_supported_languages().contains(&language)
}

// 标准化语言名称
pub fn normalize_language_name(language: &str) -> String {
    match language.to_lowercase().as_str() {
        "js" => "javascript".to_string(),
        "ts" => "typescript".to_string(), 
        "c++" => "cpp".to_string(),
        "c#" => "csharp".to_string(),
        _ => language.to_lowercase(),
    }
}

// 获取所有支持的语言列表（包括通过 AST-grep 支持的）
pub fn get_all_supported_languages() -> Vec<&'static str> {
    vec![
        // Tree-sitter 已实现的语言
        "rust", "java", "python", "go", "javascript", "typescript", "c", "cpp",
        
        // AST-grep 支持的其他语言
        "ruby", "php", "csharp", "swift", "kotlin", "scala", "dart", "lua",
        "perl", "r", "julia", "fortran", "objc", "haskell", "ocaml", "elixir",
        "erlang", "clojure", "elm", "nim", "zig", "vlang", "pascal", "ada",
        "dlang", "crystal", "vala", "groovy",
        
        // Web 和标记语言
        "html", "css", "scss", "less", "vue", "svelte", "xml", "json",
        "yaml", "toml", "markdown", "latex",
        
        // 配置和脚本语言
        "bash", "zsh", "fish", "powershell", "batch", "sql", "dockerfile",
        "hcl", "protobuf", "thrift", "graphql"
    ]
}

// 检查语言是否有完整的 tree-sitter 实现
pub fn has_tree_sitter_implementation(language: &str) -> bool {
    let normalized = normalize_language_name(language);
    get_implemented_tree_sitter_languages().contains(&normalized.as_str())
}

// 获取语言的标准文件扩展名
pub fn get_extensions_for_language(language: &str) -> &'static [&'static str] {
    match language {
        "rust" => &["rs"],
        "java" => &["java"],
        "python" => &["py", "pyw"],
        "go" => &["go"],
        "js" | "javascript" => &["js", "mjs", "jsx", "ts", "tsx"],
        "c" => &["c", "h"],
        "cpp" | "c++" => &["cpp", "cc", "cxx", "c++", "hpp", "hxx", "h++"],
        _ => &[],
    }
}

// 创建语言注册表的实用函数
pub fn create_language_registry() -> LanguageRegistry {
    let mut registry = LanguageRegistry::new();

    // Rust 配置
    registry.register_language(LanguageConfig {
        name: "rust",
        display_name: "Rust",
        extensions: &["rs"],
        language_fn: get_tree_sitter_rust,
    });

    // Java 配置
    registry.register_language(LanguageConfig {
        name: "java",
        display_name: "Java",
        extensions: &["java"],
        language_fn: get_tree_sitter_java,
    });

    // Python 配置
    registry.register_language(LanguageConfig {
        name: "python",
        display_name: "Python",
        extensions: &["py", "pyw"],
        language_fn: get_tree_sitter_python,
    });

    // Go 配置
    registry.register_language(LanguageConfig {
        name: "go",
        display_name: "Go",
        extensions: &["go"],
        language_fn: get_tree_sitter_go,
    });

    // JavaScript 配置
    registry.register_language(LanguageConfig {
        name: "js",
        display_name: "JavaScript",
        extensions: &["js", "mjs", "jsx"],
        language_fn: get_tree_sitter_js,
    });

    // TypeScript 配置
    registry.register_language(LanguageConfig {
        name: "typescript",
        display_name: "TypeScript",
        extensions: &["ts", "tsx"],
        language_fn: get_tree_sitter_typescript,
    });

    // C 配置
    registry.register_language(LanguageConfig {
        name: "c",
        display_name: "C",
        extensions: &["c", "h"],
        language_fn: get_tree_sitter_c,
    });

    // C++ 配置
    registry.register_language(LanguageConfig {
        name: "cpp",
        display_name: "C++",
        extensions: &["cpp", "cc", "cxx", "c++", "hpp", "hxx", "h++"],
        language_fn: get_tree_sitter_cpp,
    });

    registry
}

// 获取语言特定的节点分析配置
pub fn get_node_analysis_config(language: &str) -> Option<NodeAnalysisConfig> {
    match language {
        "rust" => Some(NodeAnalysisConfig {
            language: "rust",
            capture_names: &[
                "function.name",
                "struct.name",
                "enum.name",
                "trait.name",
                "impl.name",
            ],
            important_nodes: &[
                "function_item",
                "struct_item",
                "enum_item",
                "trait_item",
                "impl_item",
            ],
            visibility_indicators: &["pub", "pub(crate)", "pub(super)", "pub(self)"],
            scope_indicators: &["mod", "fn", "impl", "trait"],
        }),
        "java" => Some(NodeAnalysisConfig {
            language: "java",
            capture_names: &["class.name", "method.name", "field.name", "interface.name"],
            important_nodes: &[
                "class_declaration",
                "method_declaration",
                "field_declaration",
                "interface_declaration",
            ],
            visibility_indicators: &["public", "private", "protected"],
            scope_indicators: &["class", "interface", "method", "package"],
        }),
        "python" => Some(NodeAnalysisConfig {
            language: "python",
            capture_names: &["function.name", "class.name"],
            important_nodes: &["function_definition", "class_definition"],
            visibility_indicators: &["__"],
            scope_indicators: &["class", "def"],
        }),
        "go" => Some(NodeAnalysisConfig {
            language: "go",
            capture_names: &["function.name", "type.name", "method.name"],
            important_nodes: &[
                "function_declaration",
                "type_declaration",
                "method_declaration",
            ],
            visibility_indicators: &[],
            scope_indicators: &["func", "type", "package"],
        }),
        "js" => Some(NodeAnalysisConfig {
            language: "js",
            capture_names: &["function.name", "class.name", "method.name"],
            important_nodes: &[
                "function_declaration",
                "class_declaration",
                "method_definition",
            ],
            visibility_indicators: &["export", "static"],
            scope_indicators: &["function", "class", "method"],
        }),
        "c" => Some(NodeAnalysisConfig {
            language: "c",
            capture_names: &["function.name", "struct.name"],
            important_nodes: &["function_definition", "struct_specifier"],
            visibility_indicators: &["static", "extern"],
            scope_indicators: &["function", "struct"],
        }),
        "cpp" => Some(NodeAnalysisConfig {
            language: "cpp",
            capture_names: &["function.name", "class.name", "struct.name"],
            important_nodes: &["function_definition", "class_specifier", "struct_specifier"],
            visibility_indicators: &["public", "private", "protected", "static"],
            scope_indicators: &["class", "struct", "namespace", "function"],
        }),
        _ => None,
    }
}

// 检查节点是否为公开的
pub fn is_node_public(node: &tree_sitter::Node, file_ast: &FileAst) -> bool {
    match file_ast.language_id.as_str() {
        "rust" => {
            let node_sexp = node.to_sexp();
            if node_sexp.starts_with("(visibility_modifier")
                && node
                    .utf8_text(file_ast.source.as_bytes())
                    .unwrap_or("")
                    .contains("pub")
            {
                return true;
            }

            let mut cursor = node.walk();
            for child_node in node.children(&mut cursor) {
                if child_node.kind() == "visibility_modifier" {
                    return child_node
                        .utf8_text(file_ast.source.as_bytes())
                        .unwrap_or("")
                        .contains("pub");
                }
            }
            false
        }
        "java" => {
            let mut cursor = node.walk();
            for child_node in node.children(&mut cursor) {
                if child_node.kind() == "modifiers" {
                    let modifiers_text = child_node
                        .utf8_text(file_ast.source.as_bytes())
                        .unwrap_or("");
                    return modifiers_text.contains("public");
                }
            }
            false
        }
        _ => false,
    }
}

// 解析 Git diff 文本
pub fn parse_git_diff(diff_text: &str) -> Result<GitDiff, AppError> {
    let mut changed_files = Vec::new();
    let lines: Vec<&str> = diff_text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if line.starts_with("diff --git") {
            // 解析文件头
            if let Some(file_path) = parse_file_path(line) {
                let mut hunks = Vec::new();
                let mut change_type = ChangeType::Modified;

                // 查找文件模式和hunk信息
                i += 1;
                while i < lines.len() && !lines[i].starts_with("diff --git") {
                    let current_line = lines[i];

                    if current_line.starts_with("new file mode") {
                        change_type = ChangeType::Added;
                    } else if current_line.starts_with("deleted file mode") {
                        change_type = ChangeType::Deleted;
                    } else if current_line.starts_with("@@") {
                        // 解析 hunk
                        if let Some(hunk) = parse_hunk(current_line) {
                            hunks.push(hunk);
                        }
                    }
                    i += 1;
                }

                changed_files.push(ChangedFile {
                    path: PathBuf::from(file_path),
                    change_type,
                    hunks,
                    file_mode_change: None,
                });
                continue;
            }
        }
        i += 1;
    }

    Ok(GitDiff {
        changed_files,
        metadata: Some(HashMap::new()),
    })
}

fn parse_file_path(line: &str) -> Option<String> {
    // 解析 "diff --git a/path b/path" 格式
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 4 {
        let b_path = parts[3];
        if b_path.starts_with("b/") {
            return Some(b_path[2..].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_path() {
        assert_eq!(
            parse_file_path("diff --git a/src/main.rs b/src/main.rs"),
            Some("src/main.rs".to_string())
        );
        assert_eq!(
            parse_file_path("diff --git a/foo b/bar"),
            Some("bar".to_string())
        );
        assert_eq!(parse_file_path("diff --git a/foo"), None);
    }

    #[test]
    fn test_parse_hunk() {
        let line = "@@ -1,5 +1,6 @@";
        let hunk = parse_hunk(line).expect("Should parse hunk");
        assert_eq!(hunk.old_range.start, 1);
        assert_eq!(hunk.old_range.count, 5);
        assert_eq!(hunk.new_range.start, 1);
        assert_eq!(hunk.new_range.count, 6);
    }

    #[test]
    fn test_detect_language_from_extension() {
        assert_eq!(detect_language_from_extension("rs"), Some("rust"));
        assert_eq!(detect_language_from_extension("TS"), Some("js"));
        assert_eq!(detect_language_from_extension("unknown"), None);
    }

  
    
    #[test]
    fn test_language_support_helpers() {
        let langs = get_supported_languages();
        assert!(langs.contains(&"rust"));
        assert!(is_language_supported("cpp"));
        assert!(!is_language_supported("haskell"));
        let exts = get_extensions_for_language("js");
        assert!(exts.contains(&"js"));
    }

    #[test]
    fn test_parse_git_diff_simple() {
        let diff = concat!(
            "diff --git a/foo.rs b/foo.rs\n",
            "new file mode 100644\n",
            "@@ -0,0 +1,2 @@\n",
            "+line1\n",
            "+line2\n"
        );
        let gd = parse_git_diff(diff).unwrap();
        assert_eq!(gd.changed_files.len(), 1);
        let cf = &gd.changed_files[0];
        assert_eq!(cf.path.to_str().unwrap(), "foo.rs");
        assert_eq!(cf.change_type, ChangeType::Added);
        assert_eq!(cf.hunks.len(), 1);
        let h = &cf.hunks[0];
        assert_eq!(h.new_range.start, 1);
        assert_eq!(h.new_range.count, 2);
    }
}

fn parse_hunk(line: &str) -> Option<DiffHunk> {
    // 解析 "@@ -old_start,old_count +new_start,new_count @@" 格式
    use regex::Regex;

    let re = Regex::new(r"@@ -(\d+),(\d+) \+(\d+),(\d+) @@").ok()?;
    if let Some(captures) = re.captures(line) {
        let old_start: usize = captures.get(1)?.as_str().parse().ok()?;
        let old_count: usize = captures.get(2)?.as_str().parse().ok()?;
        let new_start: usize = captures.get(3)?.as_str().parse().ok()?;
        let new_count: usize = captures.get(4)?.as_str().parse().ok()?;

        return Some(DiffHunk {
            old_range: HunkRange {
                start: old_start,
                count: old_count,
            },
            new_range: HunkRange {
                start: new_start,
                count: new_count,
            },
            lines: Vec::new(),
        });
    }
    None
}

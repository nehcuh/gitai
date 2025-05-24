use std::{collections::HashMap, path::PathBuf, thread::current, time::SystemTime};

use tree_sitter::Language;
use tree_sitter_highlight::HighlightConfiguration;

use crate::{errors::TreeSitterError, types::analyze::GitDiff};

use crate::types::analyze::{
    ChangeType, ChangedFile, DiffHunk, FileAst, HunkRange, 
    LanguageConfig, LanguageRegistry, NodeAnalysisConfig
};

const RUST_HIGHLIGHTS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/rust/highlights.scm"
));
const RUST_INJECTIONS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/rust/injections.scm"
));
const RUST_LOCALS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/rust/locals.scm"
));

const JAVA_HIGHLIGHTS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/java/highlights.scm"
));
const JAVA_INJECTIONS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/java/injections.scm"
));
const JAVA_LOCALS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/java/locals.scm"
));

const PYTHON_HIGHLIGHTS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/python/highlights.scm"
));
const PYTHON_INJECTIONS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/python/injections.scm"
));
const PYTHON_LOCALS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/python/locals.scm"
));

const GO_HIGHLIGHTS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/go/highlights.scm"
));
const GO_INJECTIONS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/go/injections.scm"
));
const GO_LOCALS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/go/locals.scm"
));

const JAVASCRIPT_HIGHLIGHTS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/javascript/highlights.scm"
));
const JAVASCRIPT_INJECTIONS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/javascript/injections.scm"
));
const JAVASCRIPT_LOCALS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/javascript/locals.scm"
));

const C_HIGHLIGHTS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/c/highlights.scm"
));
const C_INJECTIONS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/c/injections.scm"
));
const C_LOCALS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/c/locals.scm"
));

const CPP_HIGHLIGHTS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/cpp/highlights.scm"
));
const CPP_INJECTIONS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/cpp/injections.scm"
));
const CPP_LOCALS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/queries/cpp/locals.scm"
));

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

// 获取各语言的查询模式
pub fn get_rust_query_pattern() -> &'static str {
    RUST_HIGHLIGHTS
}

pub fn get_java_query_pattern() -> &'static str {
    JAVA_HIGHLIGHTS
}

pub fn get_python_query_pattern() -> &'static str {
    PYTHON_HIGHLIGHTS
}

pub fn get_go_query_pattern() -> &'static str {
    GO_HIGHLIGHTS
}

pub fn get_js_query_pattern() -> &'static str {
    JAVASCRIPT_HIGHLIGHTS
}

pub fn get_c_query_pattern() -> &'static str {
    C_HIGHLIGHTS
}

pub fn get_cpp_query_pattern() -> &'static str {
    CPP_HIGHLIGHTS
}

// 获取所有查询模式的组合版本（用于更复杂的分析）
pub fn get_rust_full_queries() -> (&'static str, &'static str, &'static str) {
    (RUST_HIGHLIGHTS, RUST_INJECTIONS, RUST_LOCALS)
}

pub fn get_java_full_queries() -> (&'static str, &'static str, &'static str) {
    (JAVA_HIGHLIGHTS, JAVA_INJECTIONS, JAVA_LOCALS)
}

pub fn get_python_full_queries() -> (&'static str, &'static str, &'static str) {
    (PYTHON_HIGHLIGHTS, PYTHON_INJECTIONS, PYTHON_LOCALS)
}

pub fn get_go_full_queries() -> (&'static str, &'static str, &'static str) {
    (GO_HIGHLIGHTS, GO_INJECTIONS, GO_LOCALS)
}

pub fn get_js_full_queries() -> (&'static str, &'static str, &'static str) {
    (JAVASCRIPT_HIGHLIGHTS, JAVASCRIPT_INJECTIONS, JAVASCRIPT_LOCALS)
}

pub fn get_c_full_queries() -> (&'static str, &'static str, &'static str) {
    (C_HIGHLIGHTS, C_INJECTIONS, C_LOCALS)
}

pub fn get_cpp_full_queries() -> (&'static str, &'static str, &'static str) {
    (CPP_HIGHLIGHTS, CPP_INJECTIONS, CPP_LOCALS)
}

// 通用查询模式获取函数
pub fn get_query_pattern_for_language(language: &str) -> Option<&'static str> {
    match language {
        "rust" => Some(RUST_HIGHLIGHTS),
        "java" => Some(JAVA_HIGHLIGHTS),
        "python" => Some(PYTHON_HIGHLIGHTS),
        "go" => Some(GO_HIGHLIGHTS),
        "js" | "javascript" => Some(JAVASCRIPT_HIGHLIGHTS),
        "c" => Some(C_HIGHLIGHTS),
        "cpp" | "c++" => Some(CPP_HIGHLIGHTS),
        _ => None,
    }
}

// 获取语言的完整查询集合
pub fn get_full_queries_for_language(language: &str) -> Option<(&'static str, &'static str, &'static str)> {
    match language {
        "rust" => Some((RUST_HIGHLIGHTS, RUST_INJECTIONS, RUST_LOCALS)),
        "java" => Some((JAVA_HIGHLIGHTS, JAVA_INJECTIONS, JAVA_LOCALS)),
        "python" => Some((PYTHON_HIGHLIGHTS, PYTHON_INJECTIONS, PYTHON_LOCALS)),
        "go" => Some((GO_HIGHLIGHTS, GO_INJECTIONS, GO_LOCALS)),
        "js" | "javascript" => Some((JAVASCRIPT_HIGHLIGHTS, JAVASCRIPT_INJECTIONS, JAVASCRIPT_LOCALS)),
        "c" => Some((C_HIGHLIGHTS, C_INJECTIONS, C_LOCALS)),
        "cpp" | "c++" => Some((CPP_HIGHLIGHTS, CPP_INJECTIONS, CPP_LOCALS)),
        _ => None,
    }
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

// 获取支持的语言列表
pub fn get_supported_languages() -> &'static [&'static str] {
    &["rust", "java", "python", "go", "js", "c", "cpp"]
}

// 检查语言是否被支持
pub fn is_language_supported(language: &str) -> bool {
    matches!(language, "rust" | "java" | "python" | "go" | "js" | "javascript" | "c" | "cpp" | "c++")
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
        highlights_query: RUST_HIGHLIGHTS,
        injections_query: RUST_INJECTIONS,
        locals_query: RUST_LOCALS,
    });

    // Java 配置
    registry.register_language(LanguageConfig {
        name: "java",
        display_name: "Java",
        extensions: &["java"],
        language_fn: get_tree_sitter_java,
        highlights_query: JAVA_HIGHLIGHTS,
        injections_query: JAVA_INJECTIONS,
        locals_query: JAVA_LOCALS,
    });

    // Python 配置
    registry.register_language(LanguageConfig {
        name: "python",
        display_name: "Python",
        extensions: &["py", "pyw"],
        language_fn: get_tree_sitter_python,
        highlights_query: PYTHON_HIGHLIGHTS,
        injections_query: PYTHON_INJECTIONS,
        locals_query: PYTHON_LOCALS,
    });

    // Go 配置
    registry.register_language(LanguageConfig {
        name: "go",
        display_name: "Go",
        extensions: &["go"],
        language_fn: get_tree_sitter_go,
        highlights_query: GO_HIGHLIGHTS,
        injections_query: GO_INJECTIONS,
        locals_query: GO_LOCALS,
    });

    // JavaScript 配置
    registry.register_language(LanguageConfig {
        name: "js",
        display_name: "JavaScript",
        extensions: &["js", "mjs", "jsx", "ts", "tsx"],
        language_fn: get_tree_sitter_js,
        highlights_query: JAVASCRIPT_HIGHLIGHTS,
        injections_query: JAVASCRIPT_INJECTIONS,
        locals_query: JAVASCRIPT_LOCALS,
    });

    // C 配置
    registry.register_language(LanguageConfig {
        name: "c",
        display_name: "C",
        extensions: &["c", "h"],
        language_fn: get_tree_sitter_c,
        highlights_query: C_HIGHLIGHTS,
        injections_query: C_INJECTIONS,
        locals_query: C_LOCALS,
    });

    // C++ 配置
    registry.register_language(LanguageConfig {
        name: "cpp",
        display_name: "C++",
        extensions: &["cpp", "cc", "cxx", "c++", "hpp", "hxx", "h++"],
        language_fn: get_tree_sitter_cpp,
        highlights_query: CPP_HIGHLIGHTS,
        injections_query: CPP_INJECTIONS,
        locals_query: CPP_LOCALS,
    });

    registry
}

// 获取语言特定的节点分析配置
pub fn get_node_analysis_config(language: &str) -> Option<NodeAnalysisConfig> {
    match language {
        "rust" => Some(NodeAnalysisConfig {
            language: "rust",
            capture_names: &["function.name", "struct.name", "enum.name", "trait.name", "impl.name"],
            important_nodes: &["function_item", "struct_item", "enum_item", "trait_item", "impl_item"],
            visibility_indicators: &["pub", "pub(crate)", "pub(super)", "pub(self)"],
            scope_indicators: &["mod", "fn", "impl", "trait"],
        }),
        "java" => Some(NodeAnalysisConfig {
            language: "java",
            capture_names: &["class.name", "method.name", "field.name", "interface.name"],
            important_nodes: &["class_declaration", "method_declaration", "field_declaration", "interface_declaration"],
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
            important_nodes: &["function_declaration", "type_declaration", "method_declaration"],
            visibility_indicators: &[],
            scope_indicators: &["func", "type", "package"],
        }),
        "js" => Some(NodeAnalysisConfig {
            language: "js",
            capture_names: &["function.name", "class.name", "method.name"],
            important_nodes: &["function_declaration", "class_declaration", "method_definition"],
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
pub fn parse_git_diff(diff_text: &str) -> Result<GitDiff, TreeSitterError> {
    use crate::types::analyze::{GitDiff, ChangedFile, ChangeType, DiffHunk, HunkRange};
    use std::collections::HashMap;
    
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

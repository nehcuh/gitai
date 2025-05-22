use std::{collections::HashMap, path::PathBuf, time::SystemTime};

use tree_sitter::Language;

// Rust 语言解析器
pub fn get_tree_sitter_rust() -> Language {
    tree_sitter_rust::LANGUAGE.into()
}

// Java 语言解析器
pub fn get_tree_sitter_java() -> Language {
    tree_sitter_java::LANGUAGE.into()
}

// Python 语言解析器
pub fn get_tree_sitter_python() -> Language {
    tree_sitter_python::LANGUAGE.into()
}

// Go 语言解析器
pub fn get_tree_sitter_go() -> Language {
    tree_sitter_go::LANGUAGE.into()
}

// JS 语言解析器
pub fn get_tree_sitter_js() -> Language {
    tree_sitter_javascript::LANGUAGE.into()
}

// C 语言解析器
pub fn get_tree_sitter_c() -> Language {
    tree_sitter_c::LANGUAGE.into()
}

// C++ 语言解析器
pub fn get_tree_sitter_cpp() -> Language {
    tree_sitter_cpp::LANGUAGE.into()
}

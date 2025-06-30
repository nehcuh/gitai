use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Dynamic language support for ast-grep
#[derive(Debug, Clone)]
pub struct LanguageSupport {
    /// Map of language names to their configurations
    languages: HashMap<String, LanguageConfig>,
    /// File extension to language mapping
    extension_map: HashMap<String, String>,
    /// Supported ast-grep languages (detected dynamically)
    supported_languages: Vec<String>,
}

/// Configuration for a specific programming language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// Language name (e.g., "rust", "python")
    pub name: String,
    /// Display name (e.g., "Rust", "Python")
    pub display_name: String,
    /// File extensions for this language
    pub extensions: Vec<String>,
    /// Whether this language is enabled for analysis
    pub enabled: bool,
    /// Language-specific configuration
    pub config: LanguageSpecificConfig,
}

/// Language-specific configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageSpecificConfig {
    /// Comment patterns for this language
    pub comment_patterns: Vec<String>,
    /// Function/method detection patterns
    pub function_patterns: Vec<String>,
    /// Class/struct detection patterns
    pub class_patterns: Vec<String>,
    /// Import/use statement patterns
    pub import_patterns: Vec<String>,
    /// Common complexity indicators
    pub complexity_keywords: Vec<String>,
}

impl LanguageSupport {
    /// Create a new LanguageSupport instance with default configurations
    pub fn new() -> Self {
        let mut support = Self {
            languages: HashMap::new(),
            extension_map: HashMap::new(),
            supported_languages: Vec::new(),
        };

        support.initialize_default_languages();
        support.detect_ast_grep_languages();
        support
    }

    /// Initialize default language configurations
    fn initialize_default_languages(&mut self) {
        // Rust
        self.add_language(LanguageConfig {
            name: "rust".to_string(),
            display_name: "Rust".to_string(),
            extensions: vec!["rs".to_string()],
            enabled: true,
            config: LanguageSpecificConfig {
                comment_patterns: vec!["//".to_string(), "/*".to_string(), "*/".to_string()],
                function_patterns: vec!["fn ".to_string()],
                class_patterns: vec![
                    "struct ".to_string(),
                    "enum ".to_string(),
                    "trait ".to_string(),
                ],
                import_patterns: vec!["use ".to_string(), "mod ".to_string()],
                complexity_keywords: vec![
                    "if".to_string(),
                    "match".to_string(),
                    "for".to_string(),
                    "while".to_string(),
                    "loop".to_string(),
                ],
            },
        });

        // Python
        self.add_language(LanguageConfig {
            name: "python".to_string(),
            display_name: "Python".to_string(),
            extensions: vec!["py".to_string(), "pyw".to_string(), "pyi".to_string()],
            enabled: true,
            config: LanguageSpecificConfig {
                comment_patterns: vec!["#".to_string()],
                function_patterns: vec!["def ".to_string()],
                class_patterns: vec!["class ".to_string()],
                import_patterns: vec!["import ".to_string(), "from ".to_string()],
                complexity_keywords: vec![
                    "if".to_string(),
                    "for".to_string(),
                    "while".to_string(),
                    "try".to_string(),
                    "except".to_string(),
                    "elif".to_string(),
                ],
            },
        });

        // JavaScript
        self.add_language(LanguageConfig {
            name: "javascript".to_string(),
            display_name: "JavaScript".to_string(),
            extensions: vec!["js".to_string(), "jsx".to_string(), "mjs".to_string()],
            enabled: true,
            config: LanguageSpecificConfig {
                comment_patterns: vec!["//".to_string(), "/*".to_string(), "*/".to_string()],
                function_patterns: vec![
                    "function ".to_string(),
                    "() =>".to_string(),
                    ") =>".to_string(),
                ],
                class_patterns: vec!["class ".to_string()],
                import_patterns: vec!["import ".to_string(), "require(".to_string()],
                complexity_keywords: vec![
                    "if".to_string(),
                    "for".to_string(),
                    "while".to_string(),
                    "switch".to_string(),
                    "try".to_string(),
                    "catch".to_string(),
                ],
            },
        });

        // TypeScript
        self.add_language(LanguageConfig {
            name: "typescript".to_string(),
            display_name: "TypeScript".to_string(),
            extensions: vec!["ts".to_string(), "tsx".to_string()],
            enabled: true,
            config: LanguageSpecificConfig {
                comment_patterns: vec!["//".to_string(), "/*".to_string(), "*/".to_string()],
                function_patterns: vec![
                    "function ".to_string(),
                    "() =>".to_string(),
                    ") =>".to_string(),
                ],
                class_patterns: vec![
                    "class ".to_string(),
                    "interface ".to_string(),
                    "type ".to_string(),
                ],
                import_patterns: vec!["import ".to_string(), "require(".to_string()],
                complexity_keywords: vec![
                    "if".to_string(),
                    "for".to_string(),
                    "while".to_string(),
                    "switch".to_string(),
                    "try".to_string(),
                    "catch".to_string(),
                ],
            },
        });

        // Java
        self.add_language(LanguageConfig {
            name: "java".to_string(),
            display_name: "Java".to_string(),
            extensions: vec!["java".to_string()],
            enabled: true,
            config: LanguageSpecificConfig {
                comment_patterns: vec!["//".to_string(), "/*".to_string(), "*/".to_string()],
                function_patterns: vec![
                    "public ".to_string(),
                    "private ".to_string(),
                    "protected ".to_string(),
                ],
                class_patterns: vec![
                    "class ".to_string(),
                    "interface ".to_string(),
                    "enum ".to_string(),
                ],
                import_patterns: vec!["import ".to_string(), "package ".to_string()],
                complexity_keywords: vec![
                    "if".to_string(),
                    "for".to_string(),
                    "while".to_string(),
                    "switch".to_string(),
                    "try".to_string(),
                    "catch".to_string(),
                ],
            },
        });

        // C
        self.add_language(LanguageConfig {
            name: "c".to_string(),
            display_name: "C".to_string(),
            extensions: vec!["c".to_string(), "h".to_string()],
            enabled: true,
            config: LanguageSpecificConfig {
                comment_patterns: vec!["//".to_string(), "/*".to_string(), "*/".to_string()],
                function_patterns: vec!["(".to_string()],
                class_patterns: vec![
                    "struct ".to_string(),
                    "union ".to_string(),
                    "enum ".to_string(),
                ],
                import_patterns: vec!["#include".to_string()],
                complexity_keywords: vec![
                    "if".to_string(),
                    "for".to_string(),
                    "while".to_string(),
                    "switch".to_string(),
                ],
            },
        });

        // C++
        self.add_language(LanguageConfig {
            name: "cpp".to_string(),
            display_name: "C++".to_string(),
            extensions: vec![
                "cpp".to_string(),
                "cxx".to_string(),
                "cc".to_string(),
                "hpp".to_string(),
                "hxx".to_string(),
            ],
            enabled: true,
            config: LanguageSpecificConfig {
                comment_patterns: vec!["//".to_string(), "/*".to_string(), "*/".to_string()],
                function_patterns: vec!["(".to_string()],
                class_patterns: vec![
                    "class ".to_string(),
                    "struct ".to_string(),
                    "namespace ".to_string(),
                ],
                import_patterns: vec!["#include".to_string(), "using ".to_string()],
                complexity_keywords: vec![
                    "if".to_string(),
                    "for".to_string(),
                    "while".to_string(),
                    "switch".to_string(),
                    "try".to_string(),
                    "catch".to_string(),
                ],
            },
        });

        // Go
        self.add_language(LanguageConfig {
            name: "go".to_string(),
            display_name: "Go".to_string(),
            extensions: vec!["go".to_string()],
            enabled: true,
            config: LanguageSpecificConfig {
                comment_patterns: vec!["//".to_string(), "/*".to_string(), "*/".to_string()],
                function_patterns: vec!["func ".to_string()],
                class_patterns: vec![
                    "type ".to_string(),
                    "struct ".to_string(),
                    "interface ".to_string(),
                ],
                import_patterns: vec!["import ".to_string(), "package ".to_string()],
                complexity_keywords: vec![
                    "if".to_string(),
                    "for".to_string(),
                    "switch".to_string(),
                    "select".to_string(),
                ],
            },
        });

        // C#
        self.add_language(LanguageConfig {
            name: "csharp".to_string(),
            display_name: "C#".to_string(),
            extensions: vec!["cs".to_string()],
            enabled: true,
            config: LanguageSpecificConfig {
                comment_patterns: vec!["//".to_string(), "/*".to_string(), "*/".to_string()],
                function_patterns: vec![
                    "public ".to_string(),
                    "private ".to_string(),
                    "protected ".to_string(),
                ],
                class_patterns: vec![
                    "class ".to_string(),
                    "interface ".to_string(),
                    "struct ".to_string(),
                ],
                import_patterns: vec!["using ".to_string(), "namespace ".to_string()],
                complexity_keywords: vec![
                    "if".to_string(),
                    "for".to_string(),
                    "while".to_string(),
                    "switch".to_string(),
                    "try".to_string(),
                    "catch".to_string(),
                ],
            },
        });

        // Additional languages that ast-grep supports
        self.add_basic_language("kotlin", "Kotlin", vec!["kt", "kts"]);
        self.add_basic_language("swift", "Swift", vec!["swift"]);
        self.add_basic_language("ruby", "Ruby", vec!["rb"]);
        self.add_basic_language("php", "PHP", vec!["php"]);
        self.add_basic_language("scala", "Scala", vec!["scala", "sc"]);
        self.add_basic_language("bash", "Bash", vec!["sh", "bash"]);
        self.add_basic_language("html", "HTML", vec!["html", "htm"]);
        self.add_basic_language("css", "CSS", vec!["css"]);
        self.add_basic_language("yaml", "YAML", vec!["yml", "yaml"]);
        self.add_basic_language("toml", "TOML", vec!["toml"]);
        self.add_basic_language("json", "JSON", vec!["json"]);
        self.add_basic_language("xml", "XML", vec!["xml"]);
        self.add_basic_language("lua", "Lua", vec!["lua"]);
        self.add_basic_language("dart", "Dart", vec!["dart"]);
        self.add_basic_language("elixir", "Elixir", vec!["ex", "exs"]);
        self.add_basic_language("erlang", "Erlang", vec!["erl"]);
        self.add_basic_language("haskell", "Haskell", vec!["hs"]);
    }

    /// Add a basic language configuration with minimal setup
    fn add_basic_language(&mut self, name: &str, display_name: &str, extensions: Vec<&str>) {
        let config = LanguageConfig {
            name: name.to_string(),
            display_name: display_name.to_string(),
            extensions: extensions.iter().map(|s| s.to_string()).collect(),
            enabled: true,
            config: LanguageSpecificConfig {
                comment_patterns: vec![],
                function_patterns: vec![],
                class_patterns: vec![],
                import_patterns: vec![],
                complexity_keywords: vec![],
            },
        };
        self.add_language(config);
    }

    /// Add a language configuration
    pub fn add_language(&mut self, config: LanguageConfig) {
        // Update extension mapping
        for ext in &config.extensions {
            self.extension_map.insert(ext.clone(), config.name.clone());
        }

        self.languages.insert(config.name.clone(), config);
    }

    /// Detect languages supported by ast-grep (placeholder for dynamic detection)
    fn detect_ast_grep_languages(&mut self) {
        // For now, we'll use the languages we've configured
        // In a real implementation, this could query ast-grep's capabilities
        self.supported_languages = self.languages.keys().cloned().collect();
        self.supported_languages.sort();
    }

    /// Detect language from file path
    pub fn detect_language_from_path(&self, path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| self.extension_map.get(ext))
            .cloned()
    }

    /// Get language configuration
    pub fn get_language_config(&self, language: &str) -> Option<&LanguageConfig> {
        self.languages.get(language)
    }

    /// Check if a language is supported
    pub fn is_language_supported(&self, language: &str) -> bool {
        self.supported_languages.contains(&language.to_string())
    }

    /// Get all supported languages
    pub fn get_supported_languages(&self) -> &[String] {
        &self.supported_languages
    }

    /// Get all enabled languages
    pub fn get_enabled_languages(&self) -> Vec<String> {
        self.languages
            .values()
            .filter(|config| config.enabled)
            .map(|config| config.name.clone())
            .collect()
    }

    /// Enable or disable a language
    pub fn set_language_enabled(&mut self, language: &str, enabled: bool) -> bool {
        if let Some(config) = self.languages.get_mut(language) {
            config.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// Get languages by file extensions
    pub fn get_languages_for_extensions(&self, extensions: &[String]) -> Vec<String> {
        let mut languages = Vec::new();
        for ext in extensions {
            if let Some(lang) = self.extension_map.get(ext) {
                if !languages.contains(lang) {
                    languages.push(lang.clone());
                }
            }
        }
        languages
    }

    /// Filter files by supported languages
    pub fn filter_supported_files<'a>(&self, files: Vec<&'a Path>) -> Vec<(&'a Path, String)> {
        files
            .into_iter()
            .filter_map(|path| {
                self.detect_language_from_path(path)
                    .map(|lang| (path, lang))
            })
            .filter(|(_, lang)| self.is_language_supported(lang))
            .collect()
    }

    /// Get file pattern for a language
    pub fn get_file_patterns(&self, language: &str) -> Vec<String> {
        if let Some(config) = self.get_language_config(language) {
            config
                .extensions
                .iter()
                .map(|ext| format!("*.{}", ext))
                .collect()
        } else {
            vec![]
        }
    }

    /// Get all file patterns for enabled languages
    pub fn get_all_file_patterns(&self) -> Vec<String> {
        let mut patterns = Vec::new();
        for lang in self.get_enabled_languages() {
            patterns.extend(self.get_file_patterns(&lang));
        }
        patterns.sort();
        patterns.dedup();
        patterns
    }

    /// Update language configuration from external source
    pub fn update_language_config(&mut self, language: &str, config: LanguageConfig) -> bool {
        if self.languages.contains_key(language) {
            self.add_language(config);
            true
        } else {
            false
        }
    }

    /// Get language statistics
    pub fn get_language_stats(&self) -> LanguageStats {
        let total_languages = self.languages.len();
        let enabled_languages = self.get_enabled_languages().len();
        let supported_languages = self.supported_languages.len();

        LanguageStats {
            total_languages,
            enabled_languages,
            supported_languages,
            languages_by_category: self.get_languages_by_category(),
        }
    }

    /// Group languages by category (based on common characteristics)
    fn get_languages_by_category(&self) -> HashMap<String, Vec<String>> {
        let mut categories = HashMap::new();

        // System programming
        categories.insert(
            "System".to_string(),
            vec![
                "rust".to_string(),
                "c".to_string(),
                "cpp".to_string(),
                "go".to_string(),
            ],
        );

        // Web development
        categories.insert(
            "Web".to_string(),
            vec![
                "javascript".to_string(),
                "typescript".to_string(),
                "html".to_string(),
                "css".to_string(),
            ],
        );

        // Object-oriented
        categories.insert(
            "OOP".to_string(),
            vec![
                "java".to_string(),
                "csharp".to_string(),
                "kotlin".to_string(),
                "swift".to_string(),
            ],
        );

        // Scripting
        categories.insert(
            "Scripting".to_string(),
            vec![
                "python".to_string(),
                "ruby".to_string(),
                "php".to_string(),
                "bash".to_string(),
            ],
        );

        // Functional
        categories.insert(
            "Functional".to_string(),
            vec![
                "haskell".to_string(),
                "elixir".to_string(),
                "erlang".to_string(),
                "scala".to_string(),
            ],
        );

        // Markup/Config
        categories.insert(
            "Markup".to_string(),
            vec![
                "yaml".to_string(),
                "toml".to_string(),
                "json".to_string(),
                "xml".to_string(),
            ],
        );

        categories
    }
}

/// Statistics about language support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    pub total_languages: usize,
    pub enabled_languages: usize,
    pub supported_languages: usize,
    pub languages_by_category: HashMap<String, Vec<String>>,
}

impl Default for LanguageSupport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_language_detection() {
        let support = LanguageSupport::new();

        assert_eq!(
            support.detect_language_from_path(&PathBuf::from("test.rs")),
            Some("rust".to_string())
        );
        assert_eq!(
            support.detect_language_from_path(&PathBuf::from("test.py")),
            Some("python".to_string())
        );
        assert_eq!(
            support.detect_language_from_path(&PathBuf::from("test.js")),
            Some("javascript".to_string())
        );
        assert_eq!(
            support.detect_language_from_path(&PathBuf::from("test.unknown")),
            None
        );
    }

    #[test]
    fn test_supported_languages() {
        let support = LanguageSupport::new();

        assert!(support.is_language_supported("rust"));
        assert!(support.is_language_supported("python"));
        assert!(support.is_language_supported("javascript"));
        assert!(!support.get_supported_languages().is_empty());
    }

    #[test]
    fn test_file_patterns() {
        let support = LanguageSupport::new();

        let rust_patterns = support.get_file_patterns("rust");
        assert!(rust_patterns.contains(&"*.rs".to_string()));

        let all_patterns = support.get_all_file_patterns();
        assert!(all_patterns.contains(&"*.rs".to_string()));
        assert!(all_patterns.contains(&"*.py".to_string()));
    }

    #[test]
    fn test_language_stats() {
        let support = LanguageSupport::new();
        let stats = support.get_language_stats();

        assert!(stats.total_languages > 0);
        assert!(stats.enabled_languages > 0);
        assert!(stats.supported_languages > 0);
        assert!(!stats.languages_by_category.is_empty());
    }
}

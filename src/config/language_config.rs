use serde::Deserialize;

/// Language Configuration for internationalization
#[derive(Deserialize, Debug, Clone)]
pub struct LanguageConfig {
    /// Primary language: "cn" (Chinese) or "us" (English)
    #[serde(default = "default_primary_language")]
    pub primary: String,

    /// Secondary language for fallback
    #[serde(default = "default_secondary_language")]
    pub secondary: String,

    /// Whether to auto-detect system language
    #[serde(default = "default_auto_detect")]
    pub auto_detect: bool,
}

impl Default for LanguageConfig {
    fn default() -> Self {
        Self {
            primary: default_primary_language(),
            secondary: default_secondary_language(),
            auto_detect: default_auto_detect(),
        }
    }
}

impl LanguageConfig {
    /// Get the effective language based on configuration and optional override
    pub fn get_effective_language(&self, override_lang: Option<&str>) -> String {
        if let Some(lang) = override_lang {
            return lang.to_string();
        }

        if self.auto_detect {
            // Try to detect system language
            if let Ok(locale) = std::env::var("LANG") {
                if locale.starts_with("zh") {
                    return "cn".to_string();
                } else if locale.starts_with("en") {
                    return "us".to_string();
                }
            }
        }

        self.primary.clone()
    }

    /// Check if a language is supported
    pub fn is_supported_language(lang: &str) -> bool {
        matches!(lang, "cn" | "us" | "default")
    }
}

// Default functions
fn default_primary_language() -> String {
    "cn".to_string()
}

fn default_secondary_language() -> String {
    "us".to_string()
}

fn default_auto_detect() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_language_config() {
        let config = LanguageConfig::default();
        assert_eq!(config.primary, "cn");
        assert_eq!(config.secondary, "us");
        assert!(config.auto_detect);
    }

    #[test]
    fn test_override_language() {
        let config = LanguageConfig::default();
        assert_eq!(config.get_effective_language(Some("us")), "us");
        assert_eq!(config.get_effective_language(Some("cn")), "cn");
    }

    #[test]
    fn test_no_override_returns_primary() {
        let config = LanguageConfig {
            primary: "us".to_string(),
            secondary: "cn".to_string(),
            auto_detect: false,
        };
        assert_eq!(config.get_effective_language(None), "us");
    }

    #[test]
    fn test_supported_languages() {
        assert!(LanguageConfig::is_supported_language("cn"));
        assert!(LanguageConfig::is_supported_language("us"));
        assert!(LanguageConfig::is_supported_language("default"));
        assert!(!LanguageConfig::is_supported_language("fr"));
    }
}
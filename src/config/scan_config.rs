use serde::Deserialize;

/// Configuration for code scanning functionality
#[derive(Deserialize, Debug, Default, Clone)]
pub struct ScanConfig {
    #[serde(default)]
    pub rule_manager: RuleManagerConfig,
    #[serde(default)]
    #[allow(dead_code)]
    pub remote_scan: RemoteScanConfig,
    #[serde(default = "default_scan_results_path")]
    pub results_path: String,
}

/// Rule manager configuration
#[derive(Deserialize, Debug, Default, Clone)]
pub struct RuleManagerConfig {
    #[serde(default = "default_rules_path")]
    pub path: String,
    #[serde(default = "default_auto_update")]
    pub auto_update: bool,
    #[serde(default = "default_rules_ttl")]
    pub ttl_hours: u32,
    #[serde(default = "default_rules_url")]
    pub url: String,
    #[serde(default = "default_cache_path")]
    pub cache_path: String,
}

/// Remote scan configuration
#[derive(Deserialize, Debug, Default, Clone)]
#[allow(dead_code)]
pub struct RemoteScanConfig {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
}

/// Partial configurations for loading from files
#[derive(Deserialize, Debug, Default)]
pub struct PartialScanConfig {
    rule_manager: Option<PartialRuleManagerConfig>,
    remote_scan: Option<PartialRemoteScanConfig>,
    results_path: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct PartialRuleManagerConfig {
    path: Option<String>,
    auto_update: Option<bool>,
    ttl_hours: Option<u32>,
    url: Option<String>,
    cache_path: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct PartialRemoteScanConfig {
    url: Option<String>,
    token: Option<String>,
}

impl ScanConfig {
    /// Create ScanConfig from partial config with defaults
    pub fn from_partial(partial: Option<PartialScanConfig>) -> Self {
        let partial = partial.unwrap_or_default();
        
        let rule_manager = if let Some(partial_rm) = partial.rule_manager {
            RuleManagerConfig {
                path: partial_rm.path.unwrap_or_else(default_rules_path),
                auto_update: partial_rm.auto_update.unwrap_or_else(default_auto_update),
                ttl_hours: partial_rm.ttl_hours.unwrap_or_else(default_rules_ttl),
                url: partial_rm.url.unwrap_or_else(default_rules_url),
                cache_path: partial_rm.cache_path.unwrap_or_else(default_cache_path),
            }
        } else {
            RuleManagerConfig::default()
        };

        let remote_scan = if let Some(partial_remote) = partial.remote_scan {
            RemoteScanConfig {
                url: partial_remote.url,
                token: partial_remote.token,
            }
        } else {
            RemoteScanConfig::default()
        };

        Self {
            rule_manager,
            remote_scan,
            results_path: partial.results_path.unwrap_or_else(default_scan_results_path),
        }
    }
}

// Default functions
fn default_rules_path() -> String {
    "~/.cache/gitai/scan-rules".to_string()
}

fn default_auto_update() -> bool {
    true
}

fn default_rules_ttl() -> u32 {
    24 // 24 hours
}

fn default_scan_results_path() -> String {
    "~/.config/gitai/scan-results".to_string()
}

fn default_rules_url() -> String {
    "https://github.com/coderabbitai/ast-grep-essentials".to_string()
}

fn default_cache_path() -> String {
    "~/.cache/gitai/rule-cache".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_scan_config() {
        let config = ScanConfig::default();
        assert_eq!(config.rule_manager.path, "~/.cache/gitai/scan-rules");
        assert!(config.rule_manager.auto_update);
        assert_eq!(config.rule_manager.ttl_hours, 24);
        assert_eq!(config.results_path, "~/.config/gitai/scan-results");
    }

    #[test]
    fn test_from_partial_config() {
        let partial = PartialScanConfig {
            rule_manager: Some(PartialRuleManagerConfig {
                path: Some("/custom/rules".to_string()),
                auto_update: Some(false),
                ttl_hours: None, // Should use default
            }),
            remote_scan: Some(PartialRemoteScanConfig {
                url: Some("https://api.example.com".to_string()),
                token: Some("test-token".to_string()),
            }),
            results_path: Some("/custom/results".to_string()),
        };

        let config = ScanConfig::from_partial(Some(partial));
        assert_eq!(config.rule_manager.path, "/custom/rules");
        assert!(!config.rule_manager.auto_update);
        assert_eq!(config.rule_manager.ttl_hours, 24); // default
        assert_eq!(config.remote_scan.url, Some("https://api.example.com".to_string()));
        assert_eq!(config.results_path, "/custom/results");
    }

    #[test]
    fn test_from_none_partial() {
        let config = ScanConfig::from_partial(None);
        assert_eq!(config.rule_manager.path, "~/.cache/gitai/scan-rules");
        assert!(config.rule_manager.auto_update);
        assert_eq!(config.rule_manager.ttl_hours, 24);
        assert_eq!(config.results_path, "~/.config/gitai/scan-results");
    }
}
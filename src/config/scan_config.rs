use serde::Deserialize;

/// 代码扫描配置 - 简化版本，直接使用Option字段
#[derive(Deserialize, Debug, Default, Clone)]
pub struct ScanConfig {
    /// 规则管理器配置
    #[serde(default)]
    pub rule_manager: RuleManagerConfig,
    /// 远程扫描配置
    #[serde(default)]
    pub remote_scan: RemoteScanConfig,
    /// 扫描结果路径
    pub results_path: Option<String>,
}

/// 规则管理器配置 - 简化版本
#[derive(Deserialize, Debug, Default, Clone)]
pub struct RuleManagerConfig {
    /// 规则路径
    pub path: Option<String>,
    /// 是否自动更新
    pub auto_update: Option<bool>,
    /// 规则TTL小时数
    pub ttl_hours: Option<u32>,
    /// 规则URL
    pub url: Option<String>,
    /// 缓存路径
    pub cache_path: Option<String>,
}

/// 远程扫描配置
#[derive(Deserialize, Debug, Default, Clone)]
pub struct RemoteScanConfig {
    /// 远程扫描URL
    pub url: Option<String>,
    /// 远程扫描令牌
    pub token: Option<String>,
}

impl ScanConfig {
    /// 解析配置，应用默认值
    pub fn resolve(self) -> Self {
        Self {
            rule_manager: self.rule_manager.resolve(),
            remote_scan: self.remote_scan,
            results_path: Some(self.results_path.unwrap_or_else(default_scan_results_path)),
        }
    }

    /// 获取结果路径
    pub fn get_results_path(&self) -> String {
        self.results_path.as_ref()
            .unwrap_or(&default_scan_results_path())
            .clone()
    }
}

impl RuleManagerConfig {
    /// 解析规则管理器配置
    pub fn resolve(self) -> Self {
        Self {
            path: Some(self.path.unwrap_or_else(default_rules_path)),
            auto_update: Some(self.auto_update.unwrap_or_else(default_auto_update)),
            ttl_hours: Some(self.ttl_hours.unwrap_or_else(default_rules_ttl)),
            url: Some(self.url.unwrap_or_else(default_rules_url)),
            cache_path: Some(self.cache_path.unwrap_or_else(default_cache_path)),
        }
    }

    /// 获取规则路径
    pub fn get_path(&self) -> String {
        self.path.as_ref()
            .unwrap_or(&default_rules_path())
            .clone()
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
        assert_eq!(config.rule_manager.get_path(), "~/.cache/gitai/scan-rules");
        assert!(config.rule_manager.auto_update.unwrap());
        assert_eq!(config.rule_manager.ttl_hours.unwrap(), 24);
        assert_eq!(config.get_results_path(), "~/.config/gitai/scan-results");
    }

    #[test]
    fn test_resolve_config() {
        let config = ScanConfig {
            rule_manager: RuleManagerConfig {
                path: Some("/custom/rules".to_string()),
                auto_update: Some(false),
                ttl_hours: None, // Should use default
                url: None,
                cache_path: None,
            },
            remote_scan: RemoteScanConfig {
                url: Some("https://api.example.com".to_string()),
                token: Some("test-token".to_string()),
            },
            results_path: Some("/custom/results".to_string()),
        };

        let resolved = config.resolve();
        assert_eq!(resolved.rule_manager.path.unwrap(), "/custom/rules");
        assert!(!resolved.rule_manager.auto_update.unwrap());
        assert_eq!(resolved.rule_manager.ttl_hours.unwrap(), 24); // default
        assert_eq!(resolved.remote_scan.url, Some("https://api.example.com".to_string()));
        assert_eq!(resolved.results_path.unwrap(), "/custom/results");
    }

    #[test]
    fn test_rule_manager_getters() {
        let rule_manager = RuleManagerConfig {
            path: Some("/custom/rules".to_string()),
            auto_update: Some(false),
            ttl_hours: None,
            url: None,
            cache_path: None,
        };

        assert_eq!(rule_manager.get_path(), "/custom/rules");
        assert!(!rule_manager.auto_update.unwrap());
        assert_eq!(rule_manager.ttl_hours.unwrap(), 24); // default
        assert_eq!(rule_manager.url.unwrap(), "https://github.com/coderabbitai/ast-grep-essentials"); // default
        assert_eq!(rule_manager.cache_path.unwrap(), "~/.cache/gitai/rule-cache"); // default
    }

    #[test]
    fn test_scan_config_getters() {
        let config = ScanConfig {
            rule_manager: RuleManagerConfig::default(),
            remote_scan: RemoteScanConfig::default(),
            results_path: Some("/custom/results".to_string()),
        };

        assert_eq!(config.get_results_path(), "/custom/results");
    }
}
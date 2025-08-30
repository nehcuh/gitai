// 功能检测模块
// 提供运行时功能检测和报告

use std::collections::HashMap;

/// 功能信息
#[derive(Debug, Clone)]
pub struct FeatureInfo {
    pub name: &'static str,
    pub enabled: bool,
    pub description: &'static str,
    pub category: &'static str,
}

/// 获取所有功能的状态
pub fn get_features() -> Vec<FeatureInfo> {
    vec![
        // 核心功能
        FeatureInfo {
            name: "core",
            enabled: true,
            description: "核心功能（代码评审、智能提交、Git 代理）",
            category: "核心",
        },
        
        // AI 功能
        FeatureInfo {
            name: "ai",
            enabled: cfg!(feature = "ai"),
            description: "AI 功能（智能分析、代码解释）",
            category: "增强",
        },
        
        // 安全功能
        FeatureInfo {
            name: "security",
            enabled: cfg!(feature = "security"),
            description: "安全扫描功能（OpenGrep 集成）",
            category: "安全",
        },
        
        // MCP 功能
        FeatureInfo {
            name: "mcp",
            enabled: cfg!(feature = "mcp"),
            description: "MCP 服务器（Model Context Protocol）",
            category: "集成",
        },
        
        // 度量功能
        FeatureInfo {
            name: "metrics",
            enabled: cfg!(feature = "metrics"),
            description: "代码质量度量和趋势分析",
            category: "分析",
        },
        
        // DevOps 功能
        FeatureInfo {
            name: "devops",
            enabled: cfg!(feature = "devops"),
            description: "DevOps 平台集成",
            category: "集成",
        },
        
        // 更新功能
        FeatureInfo {
            name: "update-notifier",
            enabled: cfg!(feature = "update-notifier"),
            description: "自动更新通知和规则更新",
            category: "工具",
        },
        
        // 语言支持
        FeatureInfo {
            name: "tree-sitter-rust",
            enabled: cfg!(feature = "tree-sitter-rust"),
            description: "Rust 语言支持",
            category: "语言",
        },
        FeatureInfo {
            name: "tree-sitter-python",
            enabled: cfg!(feature = "tree-sitter-python"),
            description: "Python 语言支持",
            category: "语言",
        },
        FeatureInfo {
            name: "tree-sitter-javascript",
            enabled: cfg!(feature = "tree-sitter-javascript"),
            description: "JavaScript 语言支持",
            category: "语言",
        },
        FeatureInfo {
            name: "tree-sitter-typescript",
            enabled: cfg!(feature = "tree-sitter-typescript"),
            description: "TypeScript 语言支持",
            category: "语言",
        },
        FeatureInfo {
            name: "tree-sitter-go",
            enabled: cfg!(feature = "tree-sitter-go"),
            description: "Go 语言支持",
            category: "语言",
        },
        FeatureInfo {
            name: "tree-sitter-java",
            enabled: cfg!(feature = "tree-sitter-java"),
            description: "Java 语言支持",
            category: "语言",
        },
        FeatureInfo {
            name: "tree-sitter-c",
            enabled: cfg!(feature = "tree-sitter-c"),
            description: "C 语言支持",
            category: "语言",
        },
        FeatureInfo {
            name: "tree-sitter-cpp",
            enabled: cfg!(feature = "tree-sitter-cpp"),
            description: "C++ 语言支持",
            category: "语言",
        },
    ]
}

/// 获取功能摘要
pub fn get_feature_summary() -> FeatureSummary {
    let features = get_features();
    let total = features.len();
    let enabled = features.iter().filter(|f| f.enabled).count();
    
    let mut by_category: HashMap<String, Vec<FeatureInfo>> = HashMap::new();
    for feature in features {
        by_category
            .entry(feature.category.to_string())
            .or_insert_with(Vec::new)
            .push(feature);
    }
    
    FeatureSummary {
        total,
        enabled,
        disabled: total - enabled,
        by_category,
    }
}

/// 功能摘要
#[derive(Debug)]
pub struct FeatureSummary {
    pub total: usize,
    pub enabled: usize,
    pub disabled: usize,
    pub by_category: HashMap<String, Vec<FeatureInfo>>,
}

/// 显示功能报告
pub fn display_features(format: &str) {
    let features = get_features();
    let summary = get_feature_summary();
    
    match format {
        "json" => display_json(&features),
        "table" => display_table(&features, &summary),
        _ => display_default(&features, &summary),
    }
}

fn display_default(features: &[FeatureInfo], summary: &FeatureSummary) {
    println!("🎯 GitAI 功能状态");
    println!("═══════════════════════════════════════════");
    println!();
    println!("📊 总览: {}/{} 功能已启用", summary.enabled, summary.total);
    println!();
    
    // 按类别显示
    let categories = ["核心", "增强", "安全", "分析", "集成", "工具", "语言"];
    
    for category in &categories {
        if let Some(cat_features) = summary.by_category.get(*category) {
            let enabled_count = cat_features.iter().filter(|f| f.enabled).count();
            let total_count = cat_features.len();
            
            println!("📦 {} ({}/{})", category, enabled_count, total_count);
            
            for feature in cat_features {
                let status = if feature.enabled { "✅" } else { "❌" };
                let name_display = format!("{:<20}", feature.name);
                println!("  {} {} - {}", status, name_display, feature.description);
            }
            println!();
        }
    }
    
    // 构建类型提示
    println!("💡 构建类型提示:");
    if summary.enabled <= 3 {
        println!("  这是一个最小化构建，仅包含核心功能");
    } else if summary.enabled <= 6 {
        println!("  这是一个标准构建，包含常用功能");
    } else if summary.enabled <= 10 {
        println!("  这是一个增强构建，包含大部分功能");
    } else {
        println!("  这是一个完整构建，包含所有功能");
    }
    
    // 建议
    if !cfg!(feature = "ai") {
        println!();
        println!("💡 提示: AI 功能未启用，某些智能功能将不可用");
    }
    
    let enabled_langs: Vec<_> = features.iter()
        .filter(|f| f.category == "语言" && f.enabled)
        .collect();
    
    if enabled_langs.is_empty() {
        println!();
        println!("⚠️  警告: 没有启用任何语言支持，代码分析功能将受限");
    } else if enabled_langs.len() < 3 {
        println!();
        println!("💡 提示: 仅启用了 {} 种语言支持", enabled_langs.len());
    }
}

fn display_table(features: &[FeatureInfo], summary: &FeatureSummary) {
    println!("┌─────────────────────┬─────────┬──────────┬────────────────────────────────────┐");
    println!("│ 功能名称            │ 状态    │ 类别     │ 描述                               │");
    println!("├─────────────────────┼─────────┼──────────┼────────────────────────────────────┤");
    
    for feature in features {
        let status = if feature.enabled { "启用" } else { "禁用" };
        let status_color = if feature.enabled { "\x1b[32m" } else { "\x1b[31m" };
        let reset = "\x1b[0m";
        
        println!("│ {:<19} │ {}{:<7}{} │ {:<8} │ {:<34} │",
            feature.name,
            status_color, status, reset,
            feature.category,
            truncate_string(&feature.description, 34)
        );
    }
    
    println!("└─────────────────────┴─────────┴──────────┴────────────────────────────────────┘");
    println!();
    println!("总计: {}/{} 功能已启用", summary.enabled, summary.total);
}

fn display_json(features: &[FeatureInfo]) {
    use serde_json::json;
    
    let json_features: Vec<_> = features.iter().map(|f| {
        json!({
            "name": f.name,
            "enabled": f.enabled,
            "description": f.description,
            "category": f.category,
        })
    }).collect();
    
    let output = json!({
        "features": json_features,
        "summary": {
            "total": features.len(),
            "enabled": features.iter().filter(|f| f.enabled).count(),
            "disabled": features.iter().filter(|f| !f.enabled).count(),
        }
    });
    
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn truncate_string(s: &str, max_len: usize) -> String {
    // 使用字符长度避免 UTF-8 字节边界问题
    let char_count = s.chars().count();
    if char_count <= max_len {
        return s.to_string();
    }
    let keep = max_len.saturating_sub(3);
    let mut out = String::with_capacity(max_len);
    for (i, ch) in s.chars().enumerate() {
        if i >= keep { break; }
        out.push(ch);
    }
    out.push_str("...");
    out
}

/// 获取版本信息
pub fn get_version_info() -> String {
    let mut info = format!("GitAI v{}", env!("CARGO_PKG_VERSION"));
    
    let summary = get_feature_summary();
    
    // 添加构建类型标签
    if summary.enabled <= 3 {
        info.push_str(" [minimal]");
    } else if summary.enabled == summary.total {
        info.push_str(" [full]");
    } else if cfg!(feature = "ai") && cfg!(feature = "tree-sitter-rust") {
        info.push_str(" [default]");
    } else {
        info.push_str(" [custom]");
    }
    
    // 添加平台信息
    info.push_str(&format!(" ({})", std::env::consts::OS));
    
    info
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_features() {
        let features = get_features();
        assert!(!features.is_empty());
        
        // 核心功能应该始终启用
        let core = features.iter().find(|f| f.name == "core");
        assert!(core.is_some());
        assert!(core.unwrap().enabled);
    }
    
    #[test]
    fn test_feature_summary() {
        let summary = get_feature_summary();
        assert!(summary.total > 0);
        assert!(summary.enabled > 0); // 至少核心功能是启用的
        assert_eq!(summary.total, summary.enabled + summary.disabled);
    }
    
    #[test]
    fn test_version_info() {
        let version = get_version_info();
        assert!(version.contains("GitAI"));
        assert!(version.contains(env!("CARGO_PKG_VERSION")));
    }
}

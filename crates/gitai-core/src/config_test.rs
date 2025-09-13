#[cfg(test)]
mod tests {
    use gitai_core::config::Config;

    #[test]
    fn test_config_loading() {
        let config = Config::load().expect("Failed to load config");
        println!("AI URL: {}", config.ai.api_url);
        println!("AI Model: {}", config.ai.model);
        println!("MCP Enabled: {:?}", config.mcp.as_ref().map(|m| m.enabled));
        
        // 验证是否读取了全局配置
        let config_path = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".config")
            .join("gitai")
            .join("config.toml");
        
        if config_path.exists() {
            println!("✅ 全局配置文件存在");
            // 检查是否使用了正确的AI URL
            assert!(config.ai.api_url.contains("api.oaipro.com"), "应该使用全局配置中的AI URL");
        } else {
            println!("⚠️ 全局配置文件不存在，使用默认配置");
        }
    }
}
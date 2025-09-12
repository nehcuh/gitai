use gitai_core::config::Config;

fn main() {
    match Config::load() {
        Ok(config) => {
            println!("Config loaded successfully!");
            println!("AI API URL: {}", config.ai.api_url);
            println!("AI Model: {}", config.ai.model);
            println!("AI API Key: {}", config.ai.api_key.as_deref().unwrap_or("<not set>"));
        }
        Err(e) => {
            println!("Failed to load config: {}", e);
        }
    }
}

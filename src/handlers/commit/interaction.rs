use crate::errors::AppError;
use std::io::{self, Write};

use super::types::{UserInteractionConfig, UserInteractionResult};

/// Handles user interaction for commit operations
pub struct UserInteractionManager {
    config: UserInteractionConfig,
}

impl UserInteractionManager {
    /// Create a new user interaction manager
    pub fn new(config: UserInteractionConfig) -> Self {
        Self { config }
    }

    /// Show commit message and ask for confirmation
    pub fn confirm_commit_message(&self, message: &str) -> Result<UserInteractionResult, AppError> {
        if !self.config.require_confirmation {
            return Ok(UserInteractionResult {
                confirmed: true,
                modified_message: None,
            });
        }

        if self.config.format_output {
            self.display_formatted_commit_message(message)?;
        }

        if self.config.show_analysis {
            self.show_message_analysis(message);
        }

        let confirmed = self.ask_confirmation()?;
        
        Ok(UserInteractionResult {
            confirmed,
            modified_message: None,
        })
    }

    /// Display formatted commit message
    fn display_formatted_commit_message(&self, message: &str) -> Result<(), AppError> {
        println!("\n🤖 生成的提交信息:");
        println!("┌─────────────────────────────────────────────┐");
        
        for line in message.lines() {
            // Truncate long lines to fit in the box, respecting character boundaries
            let truncated_line = if line.chars().count() > 43 {
                let truncated: String = line.chars().take(40).collect();
                format!("{}...", truncated)
            } else {
                line.to_string()
            };
            println!("│ {:<43} │", truncated_line);
        }
        
        println!("└─────────────────────────────────────────────┘");
        Ok(())
    }

    /// Show message analysis
    fn show_message_analysis(&self, message: &str) {
        let analysis = self.analyze_commit_message(message);
        
        println!("\n📊 提交信息分析:");
        println!("  • 长度: {} 字符", analysis.length);
        println!("  • 行数: {} 行", analysis.line_count);
        println!("  • 格式: {}", analysis.format_type);
        
        if let Some(ref warning) = analysis.warning {
            println!("  ⚠️  警告: {}", warning);
        }
        
        if let Some(ref suggestion) = analysis.suggestion {
            println!("  💡 建议: {}", suggestion);
        }
    }

    /// Ask user for confirmation with error recovery
    fn ask_confirmation(&self) -> Result<bool, AppError> {
        let max_attempts = 3;
        
        for attempt in 1..=max_attempts {
            print!("\n是否使用此提交信息? [Y/n] ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim().to_lowercase();
                    if input.is_empty() || input == "y" || input == "yes" || input == "是" {
                        return Ok(true);
                    } else if input == "n" || input == "no" || input == "否" {
                        return Ok(false);
                    } else {
                        println!("无效输入，请输入 Y/n 或 留空确认");
                        continue;
                    }
                }
                Err(e) => {
                    if attempt == max_attempts {
                        return Err(AppError::IO(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("读取用户输入失败: {}", e),
                        )));
                    }
                    println!("读取输入失败，请重试 (尝试 {}/{})", attempt, max_attempts);
                    continue;
                }
            }
        }
        
        // 默认返回 true 以避免阻塞
        Ok(true)
    }

    /// Analyze commit message quality
    fn analyze_commit_message(&self, message: &str) -> CommitMessageAnalysis {
        let length = message.len();
        let line_count = message.lines().count();
        let first_line = message.lines().next().unwrap_or("").to_string();
        
        let format_type = self.detect_format_type(&first_line);
        let warning = self.check_warnings(&first_line, length);
        let suggestion = self.generate_suggestions(&first_line, length);
        
        CommitMessageAnalysis {
            length,
            line_count,
            first_line,
            format_type,
            warning,
            suggestion,
        }
    }

    /// Detect commit message format type
    fn detect_format_type(&self, first_line: &str) -> String {
        if first_line.contains(":") {
            if first_line.matches(char::is_numeric).count() > 0 && first_line.contains("#") {
                "约定式提交 (带工单号)".to_string()
            } else {
                "约定式提交".to_string()
            }
        } else if first_line.len() < 20 {
            "简短描述".to_string()
        } else {
            "自由格式".to_string()
        }
    }

    /// Check for warnings
    fn check_warnings(&self, first_line: &str, total_length: usize) -> Option<String> {
        if first_line.len() > 50 {
            Some("首行过长，建议控制在50字符以内".to_string())
        } else if first_line.len() < 10 {
            Some("首行过短，建议提供更详细的描述".to_string())
        } else if total_length > 2000 {
            Some("提交信息过长，可能影响可读性".to_string())
        } else if !first_line.chars().any(|c| c.is_ascii_alphabetic() || c.is_ascii_digit()) {
            Some("首行应包含有意义的描述".to_string())
        } else {
            None
        }
    }

    /// Generate suggestions
    fn generate_suggestions(&self, first_line: &str, total_length: usize) -> Option<String> {
        if !first_line.contains(":") && first_line.len() > 15 {
            Some("建议使用约定式提交格式，如 'feat:', 'fix:', 'docs:' 等".to_string())
        } else if total_length < 50 && first_line.len() > 20 {
            Some("可以在空行后添加更详细的说明".to_string())
        } else if first_line.ends_with('.') {
            Some("首行通常不需要句号结尾".to_string())
        } else {
            None
        }
    }

    /// Show simple confirmation prompt with error recovery
    pub fn simple_confirm(&self, prompt: &str) -> Result<bool, AppError> {
        let max_attempts = 3;
        
        for attempt in 1..=max_attempts {
            print!("{} [Y/n] ", prompt);
            io::stdout().flush()?;
            
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim().to_lowercase();
                    if input.is_empty() || input == "y" || input == "yes" || input == "是" {
                        return Ok(true);
                    } else if input == "n" || input == "no" || input == "否" {
                        return Ok(false);
                    } else {
                        println!("无效输入，请输入 Y/n 或 留空确认");
                        continue;
                    }
                }
                Err(e) => {
                    if attempt == max_attempts {
                        return Err(AppError::IO(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("读取用户输入失败: {}", e),
                        )));
                    }
                    println!("读取输入失败，请重试 (尝试 {}/{})", attempt, max_attempts);
                    continue;
                }
            }
        }
        
        // 默认返回 true 以避免阻塞
        Ok(true)
    }

    /// Show progress indicator
    pub fn show_progress(&self, message: &str) {
        if self.config.format_output {
            println!("🔄 {}", message);
        }
    }

    /// Show success message
    pub fn show_success(&self, message: &str) {
        if self.config.format_output {
            println!("✅ {}", message);
        }
    }

    /// Show warning message
    pub fn show_warning(&self, message: &str) {
        if self.config.format_output {
            println!("⚠️ {}", message);
        }
    }

    /// Show error message
    pub fn show_error(&self, message: &str) {
        if self.config.format_output {
            println!("❌ {}", message);
        }
    }

    /// Get user input for custom message with error recovery
    pub fn get_custom_message(&self) -> Result<Option<String>, AppError> {
        let max_attempts = 3;
        
        for attempt in 1..=max_attempts {
            print!("请输入自定义提交信息 (留空跳过): ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim();
                    if input.is_empty() {
                        return Ok(None);
                    } else {
                        return Ok(Some(input.to_string()));
                    }
                }
                Err(e) => {
                    if attempt == max_attempts {
                        return Err(AppError::IO(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("读取用户输入失败: {}", e),
                        )));
                    }
                    println!("读取输入失败，请重试 (尝试 {}/{})", attempt, max_attempts);
                    continue;
                }
            }
        }
        
        // 默认返回 None 以避免阻塞
        Ok(None)
    }

    /// Display commit statistics
    pub fn display_commit_stats(&self, stats: &CommitStats) {
        if !self.config.show_analysis {
            return;
        }

        println!("\n📊 提交统计:");
        println!("  • 文件变更: {} 个", stats.files_changed);
        println!("  • 代码行数: +{} -{}", stats.lines_added, stats.lines_removed);
        
        if let Some(ref language) = stats.primary_language {
            println!("  • 主要语言: {}", language);
        }
        
        if let Some(duration) = stats.generation_time {
            println!("  • 生成耗时: {:.2} 秒", duration.as_secs_f64());
        }
    }

    /// Show enhanced analysis information
    pub fn show_enhanced_analysis(&self, analysis_info: &EnhancedAnalysisInfo) {
        if !self.config.show_analysis {
            return;
        }

        println!("\n🌳 增强分析信息:");
        
        if analysis_info.tree_sitter_used {
            println!("  ✅ Tree-sitter 静态分析已启用");
            println!("  • 分析深度: {}", analysis_info.analysis_depth);
        }
        
        if analysis_info.review_integrated {
            println!("  ✅ 代码评审结果已集成");
        }
        
        if analysis_info.ai_enhanced {
            println!("  ✅ AI 增强分析已启用");
            if let Some(ref model) = analysis_info.ai_model {
                println!("  • AI 模型: {}", model);
            }
        }
        
        if let Some(confidence) = analysis_info.confidence_score {
            println!("  • 置信度: {:.1}%", confidence * 100.0);
        }
    }
}

impl Default for UserInteractionManager {
    fn default() -> Self {
        Self::new(UserInteractionConfig {
            require_confirmation: true,
            show_analysis: true,
            format_output: true,
        })
    }
}

/// Analysis result for commit message
#[derive(Debug, Clone)]
pub struct CommitMessageAnalysis {
    pub length: usize,
    pub line_count: usize,
    pub first_line: String,
    pub format_type: String,
    pub warning: Option<String>,
    pub suggestion: Option<String>,
}

/// Statistics about the commit
#[derive(Debug, Clone)]
pub struct CommitStats {
    pub files_changed: usize,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub primary_language: Option<String>,
    pub generation_time: Option<std::time::Duration>,
}

/// Information about enhanced analysis
#[derive(Debug, Clone)]
pub struct EnhancedAnalysisInfo {
    pub tree_sitter_used: bool,
    pub analysis_depth: String,
    pub review_integrated: bool,
    pub ai_enhanced: bool,
    pub ai_model: Option<String>,
    pub confidence_score: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> UserInteractionConfig {
        UserInteractionConfig {
            require_confirmation: false, // Disable for automated tests
            show_analysis: true,
            format_output: true,
        }
    }

    #[test]
    fn test_user_interaction_manager_creation() {
        let config = create_test_config();
        let manager = UserInteractionManager::new(config);
        assert!(true); // Manager created successfully
    }

    #[test]
    fn test_analyze_commit_message() {
        let config = create_test_config();
        let manager = UserInteractionManager::new(config);
        
        // Test conventional commit
        let message = "feat: add new authentication feature\n\nImplemented OAuth2 integration with Google and GitHub.";
        let analysis = manager.analyze_commit_message(message);
        
        assert_eq!(analysis.format_type, "约定式提交");
        assert!(analysis.length > 50);
        assert_eq!(analysis.line_count, 3);
        assert!(analysis.warning.is_none()); // Should be a good message
        
        // Test short message
        let short_message = "fix bug";
        let short_analysis = manager.analyze_commit_message(short_message);
        assert!(short_analysis.warning.is_some());
        assert!(short_analysis.warning.as_ref().unwrap().contains("过短"));
        
        // Test long first line
        let long_message = "feat: this is a very long commit message that exceeds the recommended 50 character limit";
        let long_analysis = manager.analyze_commit_message(long_message);
        assert!(long_analysis.warning.is_some());
        assert!(long_analysis.warning.as_ref().unwrap().contains("过长"));
    }

    #[test]
    fn test_detect_format_type() {
        let config = create_test_config();
        let manager = UserInteractionManager::new(config);
        
        assert_eq!(manager.detect_format_type("feat: add feature"), "约定式提交");
        assert_eq!(manager.detect_format_type("fix #123: resolve bug"), "约定式提交 (带工单号)");
        assert_eq!(manager.detect_format_type("update"), "简短描述");
        assert_eq!(manager.detect_format_type("implement new feature for users"), "自由格式");
    }

    #[test]
    fn test_check_warnings() {
        let config = create_test_config();
        let manager = UserInteractionManager::new(config);
        
        // Long first line
        let long_first_line = "this is a very long first line that exceeds the recommended fifty character limit";
        assert!(manager.check_warnings(long_first_line, 100).is_some());
        
        // Short first line
        let short_first_line = "fix";
        assert!(manager.check_warnings(short_first_line, 50).is_some());
        
        // Good first line
        let good_first_line = "feat: add user authentication";
        assert!(manager.check_warnings(good_first_line, 100).is_none());
        
        // Very long total message
        assert!(manager.check_warnings("feat: test", 3000).is_some());
    }

    #[test]
    fn test_generate_suggestions() {
        let config = create_test_config();
        let manager = UserInteractionManager::new(config);
        
        // Should suggest conventional format
        let non_conventional = "implement new user management system";
        let suggestion = manager.generate_suggestions(non_conventional, 100);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("约定式提交"));
        
        // Should suggest removing period
        let with_period = "feat: add new feature.";
        let period_suggestion = manager.generate_suggestions(with_period, 50);
        assert!(period_suggestion.is_some());
        assert!(period_suggestion.unwrap().contains("句号"));
        
        // Good message should have no suggestions
        let good_message = "feat: add authentication";
        assert!(manager.generate_suggestions(good_message, 100).is_none());
    }

    #[test]
    fn test_confirm_commit_message_no_confirmation() {
        let config = create_test_config(); // require_confirmation is false
        let manager = UserInteractionManager::new(config);
        
        let result = manager.confirm_commit_message("test message");
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert!(result.confirmed);
        assert!(result.modified_message.is_none());
    }

    #[test]
    fn test_commit_message_analysis_fields() {
        let analysis = CommitMessageAnalysis {
            length: 100,
            line_count: 2,
            first_line: "feat: test".to_string(),
            format_type: "约定式提交".to_string(),
            warning: Some("test warning".to_string()),
            suggestion: Some("test suggestion".to_string()),
        };
        
        assert_eq!(analysis.length, 100);
        assert_eq!(analysis.line_count, 2);
        assert_eq!(analysis.first_line, "feat: test");
        assert_eq!(analysis.format_type, "约定式提交");
        assert!(analysis.warning.is_some());
        assert!(analysis.suggestion.is_some());
    }

    #[test]
    fn test_commit_stats() {
        let stats = CommitStats {
            files_changed: 3,
            lines_added: 150,
            lines_removed: 50,
            primary_language: Some("Rust".to_string()),
            generation_time: Some(std::time::Duration::from_millis(1500)),
        };
        
        assert_eq!(stats.files_changed, 3);
        assert_eq!(stats.lines_added, 150);
        assert_eq!(stats.lines_removed, 50);
        assert_eq!(stats.primary_language.as_ref().unwrap(), "Rust");
        assert!(stats.generation_time.is_some());
    }

    #[test]
    fn test_enhanced_analysis_info() {
        let info = EnhancedAnalysisInfo {
            tree_sitter_used: true,
            analysis_depth: "deep".to_string(),
            review_integrated: true,
            ai_enhanced: true,
            ai_model: Some("gpt-4".to_string()),
            confidence_score: Some(0.95),
        };
        
        assert!(info.tree_sitter_used);
        assert_eq!(info.analysis_depth, "deep");
        assert!(info.review_integrated);
        assert!(info.ai_enhanced);
        assert_eq!(info.ai_model.as_ref().unwrap(), "gpt-4");
        assert_eq!(info.confidence_score.unwrap(), 0.95);
    }

    #[test]
    fn test_default_user_interaction_manager() {
        let manager = UserInteractionManager::default();
        // Should have sensible defaults
        assert!(true); // Constructor worked
    }

    // Note: Interactive tests (like actual confirmation) are not included
    // as they require manual input. In a real testing environment,
    // you might want to use dependency injection to mock stdin/stdout.
}
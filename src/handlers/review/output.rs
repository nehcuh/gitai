use crate::types::devops::AnalysisWorkItem;
use colored::Colorize;
use std::time::Instant;

use super::types::{AIAnalysisResult, AnalysisType, OutputConfig};

/// Handles output formatting and display for review results
#[derive(Debug, Clone)]
pub struct OutputFormatter {
    pub config: OutputConfig,
}

impl OutputFormatter {
    /// Create a new output formatter
    pub fn new(config: OutputConfig) -> Self {
        Self { config }
    }

    /// Format content for display based on configuration
    pub fn format_for_display(&self, content: &str) -> String {
        match self.config.format.as_str() {
            "json" => self.format_as_json(content),
            "markdown" => self.format_as_markdown(content),
            "plain" => content.to_string(),
            _ => self.format_for_console(content),
        }
    }

    /// Format content specifically for console output with colors
    pub fn format_for_console(&self, content: &str) -> String {
        let mut formatted = String::new();

        // Add header
        formatted.push_str(&"🔍 GitAI 代码审查报告".bright_blue().bold().to_string());
        formatted.push_str("\n");
        formatted.push_str(&"=".repeat(50).cyan().to_string());
        formatted.push_str("\n\n");

        // Process content line by line for better formatting
        for line in content.lines() {
            let formatted_line = self.format_line_for_console(line);
            formatted.push_str(&formatted_line);
            formatted.push('\n');
        }

        formatted
    }

    /// Format enhanced analysis result
    pub fn format_enhanced_result(&self, result: &AIAnalysisResult, work_items: &[AnalysisWorkItem]) -> String {
        let mut formatted = String::new();

        // Add header with analysis type indicator
        match result.analysis_type {
            AnalysisType::Enhanced => {
                formatted.push_str(&"🚀 增强型 AI 代码审查".bright_green().bold().to_string());
            }
            AnalysisType::Standard => {
                formatted.push_str(&"🔍 标准 AI 代码审查".bright_blue().bold().to_string());
            }
            AnalysisType::Fallback => {
                formatted.push_str(&"⚠️ 离线模式审查".bright_yellow().bold().to_string());
            }
        }
        formatted.push_str("\n");
        formatted.push_str(&"=".repeat(60).cyan().to_string());
        formatted.push_str("\n\n");

        // Add fallback warning if applicable
        if result.is_fallback {
            formatted.push_str(&"⚠️ 警告: AI 服务不可用，显示基础分析结果\n\n".yellow().to_string());
        }

        // Add work item context for enhanced analysis
        if !work_items.is_empty() {
            formatted.push_str(&"📋 相关工作项:".bright_cyan().bold().to_string());
            formatted.push_str("\n");
            for item in work_items {
                let item_type = item.item_type_name.as_deref().unwrap_or("未知类型");
                let id = item.id.map(|id| id.to_string()).unwrap_or_else(|| "N/A".to_string());
                let title = item.title.as_deref().unwrap_or("无标题");
                
                formatted.push_str(&format!(
                    "   • {} (ID: {}): {}\n",
                    item_type.bright_white().bold(),
                    id.bright_yellow(),
                    title.white()
                ));
            }
            formatted.push_str("\n");
        }

        // Add main content
        formatted.push_str(&self.format_for_console(&result.content));

        formatted
    }

    /// Format content as JSON
    fn format_as_json(&self, content: &str) -> String {
        let json_obj = serde_json::json!({
            "review_content": content,
            "format": "json",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        serde_json::to_string_pretty(&json_obj).unwrap_or_else(|_| content.to_string())
    }

    /// Format content as Markdown
    fn format_as_markdown(&self, content: &str) -> String {
        let mut markdown = String::new();
        markdown.push_str("# GitAI 代码审查报告\n\n");
        markdown.push_str(&format!("> 生成时间: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        
        // Convert console formatting to markdown
        let processed_content = content
            .replace("=== ", "## ")
            .replace("📁 ", "### 📁 ")
            .replace("📝 ", "- **")
            .replace("📊 ", "### 📊 ")
            .replace("🔍 ", "#### 🔍 ")
            .replace("⚠️ ", "> ⚠️ ")
            .replace("💡 ", "> 💡 ");
        
        markdown.push_str(&processed_content);
        markdown
    }

    /// Format a single line for console output
    fn format_line_for_console(&self, line: &str) -> String {
        // Apply different colors based on line content
        if line.starts_with("===") {
            line.bright_blue().bold().to_string()
        } else if line.starts_with("📁") || line.starts_with("📊") {
            line.bright_green().to_string()
        } else if line.starts_with("📝") {
            line.white().to_string()
        } else if line.starts_with("⚠️") {
            line.yellow().to_string()
        } else if line.starts_with("✅") {
            line.green().to_string()
        } else if line.starts_with("❌") {
            line.red().to_string()
        } else if line.starts_with("💡") {
            line.cyan().to_string()
        } else if line.starts_with("   ") {
            format!("   {}", line.trim().dimmed())
        } else {
            line.to_string()
        }
    }

    /// Output review statistics
    pub fn output_review_stats(&self, start_time: Instant, analysis_type: &AnalysisType) -> String {
        let duration = start_time.elapsed();
        let mut stats = String::new();

        stats.push_str("\n");
        stats.push_str(&"📊 审查统计信息".bright_cyan().bold().to_string());
        stats.push_str("\n");
        stats.push_str(&"-".repeat(30).cyan().to_string());
        stats.push_str("\n");

        stats.push_str(&format!(
            "⏱️  处理时间: {:.2} 秒\n",
            duration.as_secs_f64()
        ));

        let analysis_desc = match analysis_type {
            AnalysisType::Enhanced => "增强型分析 (包含工作项上下文)",
            AnalysisType::Standard => "标准分析",
            AnalysisType::Fallback => "离线模式 (静态分析)",
        };
        stats.push_str(&format!("🔬 分析类型: {}\n", analysis_desc));

        if self.config.verbose {
            stats.push_str(&format!(
                "📅 生成时间: {}\n",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            ));
        }

        stats
    }

    /// Format content for saving to file
    pub fn format_for_saving(&self, content: &str, format: &str) -> String {
        match format {
            "json" => self.format_as_json(content),
            "markdown" | "md" => self.format_as_markdown(content),
            "html" => self.format_as_html(content),
            _ => {
                // Plain text - remove color codes
                self.strip_ansi_codes(content)
            }
        }
    }

    /// Format content as HTML
    fn format_as_html(&self, content: &str) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n");
        html.push_str("<title>GitAI 代码审查报告</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; }\n");
        html.push_str("h1, h2, h3 { color: #333; }\n");
        html.push_str("pre { background: #f6f8fa; padding: 16px; border-radius: 6px; overflow-x: auto; }\n");
        html.push_str("code { background: #f6f8fa; padding: 2px 4px; border-radius: 3px; }\n");
        html.push_str(".warning { color: #e36209; }\n");
        html.push_str(".success { color: #28a745; }\n");
        html.push_str(".info { color: #0366d6; }\n");
        html.push_str("</style>\n</head>\n<body>\n");

        html.push_str("<h1>GitAI 代码审查报告</h1>\n");
        html.push_str(&format!("<p><em>生成时间: {}</em></p>\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        // Convert content to HTML (basic conversion)
        let html_content = content
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('\n', "<br>\n")
            .replace("===", "<h2>")
            .replace("📁", "<span class=\"info\">📁</span>")
            .replace("⚠️", "<span class=\"warning\">⚠️</span>")
            .replace("✅", "<span class=\"success\">✅</span>");

        html.push_str(&html_content);
        html.push_str("\n</body>\n</html>");
        html
    }

    /// Strip ANSI color codes for plain text output
    fn strip_ansi_codes(&self, content: &str) -> String {
        // Simple regex-like approach to remove ANSI escape sequences
        let mut result = String::new();
        let mut chars = content.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // Skip ANSI escape sequence
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    while let Some(escape_ch) = chars.next() {
                        if escape_ch.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
            } else {
                result.push(ch);
            }
        }

        result
    }
}

impl Default for OutputFormatter {
    fn default() -> Self {
        Self::new(OutputConfig {
            format: "console".to_string(),
            show_stats: true,
            verbose: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> OutputConfig {
        OutputConfig {
            format: "console".to_string(),
            show_stats: true,
            verbose: true,
        }
    }

    #[test]
    fn test_format_for_console() {
        let formatter = OutputFormatter::new(create_test_config());
        let content = "=== Test Header ===\n📁 File: test.rs\n⚠️ Warning message";
        
        let result = formatter.format_for_console(content);
        
        assert!(result.contains("GitAI 代码审查报告"));
        assert!(result.contains("Test Header"));
    }

    #[test]
    fn test_format_as_json() {
        let formatter = OutputFormatter::new(create_test_config());
        let content = "Test review content";
        
        let result = formatter.format_as_json(content);
        
        assert!(result.contains("review_content"));
        assert!(result.contains("Test review content"));
        assert!(result.contains("timestamp"));
    }

    #[test]
    fn test_format_as_markdown() {
        let formatter = OutputFormatter::new(create_test_config());
        let content = "=== Test Section ===\n📁 File information";
        
        let result = formatter.format_as_markdown(content);
        
        assert!(result.contains("# GitAI 代码审查报告"));
        assert!(result.contains("## Test Section"));
        assert!(result.contains("### 📁"));
    }

    #[test]
    fn test_strip_ansi_codes() {
        let formatter = OutputFormatter::new(create_test_config());
        let content = "\x1b[31mRed text\x1b[0m normal text";
        
        let result = formatter.strip_ansi_codes(content);
        
        assert_eq!(result, "Red text normal text");
    }
}
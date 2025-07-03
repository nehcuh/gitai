use crate::cli::commands::{GitAIArgs, GitAICommand};
use crate::common::{SupportedLanguage, AppResult, AppError};
use clap::{Parser, error::ErrorKind};
use std::str::FromStr;

/// CLI 解析器
pub struct CLIParser;

impl CLIParser {
    /// 解析命令行参数
    pub fn parse() -> AppResult<ParsedArgs> {
        match GitAIArgs::try_parse() {
            Ok(args) => Self::process_args(args),
            Err(e) => {
                match e.kind() {
                    ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                        // 对于帮助和版本信息，直接显示并返回 Help 操作
                        println!("{}", e);
                        std::process::exit(0);
                    },
                    _ => Err(AppError::cli(format!("参数解析失败: {}", e)))
                }
            }
        }
    }

    /// 从环境参数解析
    pub fn parse_from(args: Vec<String>) -> AppResult<ParsedArgs> {
        let args = GitAIArgs::try_parse_from(args)
            .map_err(|e| AppError::cli(format!("参数解析失败: {}", e)))?;
        
        Self::process_args(args)
    }

    /// 处理解析后的参数
    fn process_args(args: GitAIArgs) -> AppResult<ParsedArgs> {
        // 解析语言参数
        let language = if let Some(lang_str) = &args.lang {
            Some(SupportedLanguage::from_str(lang_str)
                .map_err(|e| AppError::cli(e))?)
        } else {
            None
        };

        // 检查互斥的 AI 标志
        if args.ai && args.noai {
            return Err(AppError::cli("--ai 和 --noai 参数不能同时使用"));
        }

        // 确定 AI 模式
        let ai_mode = if args.noai {
            AIMode::Disabled
        } else if args.ai {
            AIMode::Global
        } else {
            AIMode::Smart
        };

        // 处理命令
        let operation = match args.command {
            Some(cmd) => Operation::GitAI(cmd),
            None => {
                if args.git_args.is_empty() {
                    Operation::Help
                } else {
                    Operation::GitPassthrough(args.git_args)
                }
            }
        };

        Ok(ParsedArgs {
            operation,
            ai_mode,
            language,
        })
    }
}

/// 解析后的参数结构
#[derive(Debug)]
pub struct ParsedArgs {
    pub operation: Operation,
    pub ai_mode: AIMode,
    pub language: Option<SupportedLanguage>,
}

/// 操作类型
#[derive(Debug)]
pub enum Operation {
    /// GitAI 特殊命令
    GitAI(GitAICommand),
    /// Git 命令透传
    GitPassthrough(Vec<String>),
    /// 显示帮助
    Help,
}

/// AI 工作模式
#[derive(Debug, PartialEq, Eq)]
pub enum AIMode {
    /// 禁用 AI
    Disabled,
    /// 全局启用 AI
    Global,
    /// 智能 AI（根据上下文决定）
    Smart,
}

impl AIMode {
    /// 检查是否应该使用 AI
    pub fn should_use_ai(&self, context: &AIContext) -> bool {
        match self {
            AIMode::Disabled => false,
            AIMode::Global => true,
            AIMode::Smart => context.suggests_ai_usage(),
        }
    }
}

/// AI 使用上下文
#[derive(Debug)]
pub struct AIContext {
    pub has_errors: bool,
    pub is_complex_command: bool,
    pub user_requested_explanation: bool,
}

impl AIContext {
    /// 根据上下文判断是否建议使用 AI
    pub fn suggests_ai_usage(&self) -> bool {
        self.has_errors || self.user_requested_explanation
    }
}

impl Default for AIContext {
    fn default() -> Self {
        Self {
            has_errors: false,
            is_complex_command: false,
            user_requested_explanation: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_commit() {
        let args = vec!["gitai".to_string(), "commit".to_string()];
        let result = CLIParser::parse_from(args).unwrap();
        
        match result.operation {
            Operation::GitAI(GitAICommand::Commit(_)) => {},
            _ => panic!("Expected commit command"),
        }
        assert_eq!(result.ai_mode, AIMode::Smart);
    }

    #[test]
    fn test_parse_global_ai() {
        let args = vec!["gitai".to_string(), "--ai".to_string(), "status".to_string()];
        let result = CLIParser::parse_from(args).unwrap();
        
        match result.operation {
            Operation::GitPassthrough(git_args) => {
                assert_eq!(git_args, vec!["status"]);
            },
            _ => panic!("Expected git passthrough"),
        }
        assert_eq!(result.ai_mode, AIMode::Global);
    }

    #[test]
    fn test_parse_language() {
        let args = vec!["gitai".to_string(), "--lang=zh".to_string(), "commit".to_string()];
        let result = CLIParser::parse_from(args).unwrap();
        
        assert_eq!(result.language, Some(SupportedLanguage::Chinese));
    }

    #[test]
    fn test_mutually_exclusive_ai_flags() {
        let args = vec!["gitai".to_string(), "--ai".to_string(), "--noai".to_string()];
        let result = CLIParser::parse_from(args);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_ai_context() {
        let context = AIContext {
            has_errors: true,
            is_complex_command: false,
            user_requested_explanation: false,
        };
        
        assert!(AIMode::Smart.should_use_ai(&context));
        assert!(!AIMode::Disabled.should_use_ai(&context));
        assert!(AIMode::Global.should_use_ai(&context));
    }
}
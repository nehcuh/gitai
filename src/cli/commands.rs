use clap::{Args, Parser, Subcommand};
use std::str::FromStr;
use crate::common::{SupportedLanguage, AppResult, AppError};

/// GitAI 主命令行参数
#[derive(Parser, Debug)]
#[clap(
    name = "gitai",
    version = "0.1.0",
    about = "AI-powered Git tools suite",
    long_about = "GitAI is an AI-driven Git tools suite that integrates artificial intelligence into Git workflows."
)]
pub struct GitAIArgs {
    /// Enable AI functionality globally for all commands
    #[clap(long, global = true)]
    pub ai: bool,

    /// Disable AI functionality globally for all commands  
    #[clap(long, global = true)]
    pub noai: bool,

    /// Specify output language (zh|en|auto)
    #[clap(long, global = true, value_name = "LANG")]
    pub lang: Option<String>,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Option<GitAICommand>,

    /// Git command and arguments to pass through
    #[clap(allow_hyphen_values = true, trailing_var_arg = true)]
    pub git_args: Vec<String>,
}

/// GitAI 支持的子命令
#[derive(Subcommand, Debug, Clone)]
pub enum GitAICommand {
    /// AI-powered commit message generation
    #[clap(alias = "cm")]
    Commit(CommitArgs),
    
    /// AI-driven code review
    #[clap(alias = "rv")]
    Review(ReviewArgs),
    
    /// AST-Grep powered code scanning
    #[clap(alias = "sc")]
    Scan(ScanArgs),
    
    /// Update AST-Grep rules
    #[clap(alias = "ur")]
    UpdateRules(UpdateRulesArgs),
}

/// Commit 命令参数
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct CommitArgs {
    /// Enable AST-Grep analysis for enhanced commit messages
    #[clap(short = 't', long = "ast-grep")]
    pub ast_grep: bool,

    /// Automatically stage all tracked, modified files before commit
    #[clap(short = 'a', long = "all")]
    pub auto_stage: bool,

    /// Commit message
    #[clap(short, long, value_name = "MESSAGE")]
    pub message: Option<String>,

    /// Issue IDs to include in commit message (e.g., "#123,#456")
    #[clap(long = "issue-id", value_name = "ISSUE_IDS")]
    pub issue_id: Option<String>,

    /// Perform code review before commit
    #[clap(short = 'r', long = "review")]
    pub review: bool,

    /// Additional git commit arguments
    #[clap(allow_hyphen_values = true, last = true)]
    pub git_args: Vec<String>,
}

/// Review 命令参数
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct ReviewArgs {
    /// Focus areas for the review
    #[clap(long, value_name = "AREA")]
    pub focus: Option<String>,

    /// Output format (text|markdown|json)
    #[clap(long, value_name = "FORMAT", default_value = "text")]
    pub format: String,

    /// Output file
    #[clap(long, value_name = "FILE")]
    pub output: Option<String>,

    /// Enable AST-Grep analysis (enabled by default)
    #[clap(long = "ast-grep")]
    pub ast_grep: bool,

    /// Disable automatic code scanning
    #[clap(long = "no-scan")]
    pub no_scan: bool,

    /// Force new scan (ignore cache)
    #[clap(long = "force-scan")]
    pub force_scan: bool,

    /// First commit reference
    #[clap(long, value_name = "COMMIT")]
    pub commit1: Option<String>,

    /// Second commit reference
    #[clap(long, value_name = "COMMIT")]
    pub commit2: Option<String>,

    /// Stories associated with the review
    #[clap(long, value_name = "STORIES")]
    pub stories: Option<String>,

    /// Tasks associated with the review
    #[clap(long, value_name = "TASKS")]
    pub tasks: Option<String>,

    /// Defects associated with the review
    #[clap(long, value_name = "DEFECTS")]
    pub defects: Option<String>,

    /// Space ID for DevOps integration
    #[clap(long, value_name = "SPACE_ID")]
    pub space_id: Option<u32>,

    /// Additional git arguments
    #[clap(allow_hyphen_values = true, last = true)]
    pub git_args: Vec<String>,
}

/// Scan 命令参数
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct ScanArgs {
    /// Target directory or file to scan
    #[clap(value_name = "PATH", default_value = ".")]
    pub target: String,

    /// Programming languages to scan (comma-separated)
    #[clap(long, value_name = "LANGUAGES")]
    pub languages: Option<String>,

    /// Specific rules to run (comma-separated rule IDs)
    #[clap(long, value_name = "RULES")]
    pub rules: Option<String>,

    /// Rule severity levels (error,warning,info,hint)
    #[clap(long, value_name = "SEVERITY", default_value = "error,warning,info")]
    pub severity: String,

    /// Output format (text|json|sarif|csv)
    #[clap(long, value_name = "FORMAT", default_value = "text")]
    pub format: String,

    /// Output file path
    #[clap(long, value_name = "FILE")]
    pub output: Option<String>,

    /// Maximum number of issues to report (0 = unlimited)
    #[clap(long, value_name = "COUNT", default_value = "0")]
    pub max_issues: usize,

    /// Include file paths matching pattern
    #[clap(long, value_name = "PATTERN")]
    pub include: Option<String>,

    /// Exclude file paths matching pattern
    #[clap(long, value_name = "PATTERN")]
    pub exclude: Option<String>,

    /// Enable parallel processing
    #[clap(long)]
    pub parallel: bool,

    /// Verbose output
    #[clap(short, long)]
    pub verbose: bool,

    /// Show rule statistics
    #[clap(long)]
    pub stats: bool,

    /// Fail with non-zero exit code if issues found
    #[clap(long)]
    pub fail_on_error: bool,
}

/// UpdateRules 命令参数
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct UpdateRulesArgs {
    /// Source to update rules from (github|local|url)
    #[clap(long, value_name = "SOURCE", default_value = "github")]
    pub source: String,

    /// Specific repository or URL for rules
    #[clap(long, value_name = "REPO")]
    pub repository: Option<String>,

    /// Branch or tag to use
    #[clap(long, value_name = "REF", default_value = "main")]
    pub reference: String,

    /// Target directory for rules
    #[clap(long, value_name = "DIR")]
    pub target_dir: Option<String>,

    /// Force update even if rules are newer
    #[clap(long)]
    pub force: bool,

    /// Backup existing rules before update
    #[clap(long)]
    pub backup: bool,

    /// Verify rules after download
    #[clap(long)]
    pub verify: bool,

    /// List available rule sources
    #[clap(long)]
    pub list_sources: bool,

    /// Verbose output
    #[clap(short, long)]
    pub verbose: bool,
}

impl Default for CommitArgs {
    fn default() -> Self {
        Self {
            ast_grep: false,
            auto_stage: false,
            message: None,
            issue_id: None,
            review: false,
            git_args: Vec::new(),
        }
    }
}

impl Default for ReviewArgs {
    fn default() -> Self {
        Self {
            focus: None,
            format: "text".to_string(),
            output: None,
            ast_grep: true,
            no_scan: false,
            force_scan: false,
            commit1: None,
            commit2: None,
            stories: None,
            tasks: None,
            defects: None,
            space_id: None,
            git_args: Vec::new(),
        }
    }
}

impl Default for ScanArgs {
    fn default() -> Self {
        Self {
            target: ".".to_string(),
            languages: None,
            rules: None,
            severity: "error,warning,info".to_string(),
            format: "text".to_string(),
            output: None,
            max_issues: 0,
            include: None,
            exclude: None,
            parallel: false,
            verbose: false,
            stats: false,
            fail_on_error: false,
        }
    }
}

impl Default for UpdateRulesArgs {
    fn default() -> Self {
        Self {
            source: "github".to_string(),
            repository: None,
            reference: "main".to_string(),
            target_dir: None,
            force: false,
            backup: false,
            verify: false,
            list_sources: false,
            verbose: false,
        }
    }
}
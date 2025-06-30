use clap::{Args, Parser, Subcommand};
use std::{collections::HashMap, path::PathBuf, str::FromStr};

// Wrapper type for comma-separated u32 lists to be used with clap
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommaSeparatedU32List(pub Vec<u32>);

impl FromStr for CommaSeparatedU32List {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            // Handles cases like --stories=
            Ok(CommaSeparatedU32List(Vec::new()))
        } else {
            s.split(',')
                .map(|item_str| {
                    item_str
                        .trim()
                        .parse::<u32>()
                        .map_err(|e| format!("Invalid u32 value '{}': {}", item_str.trim(), e))
                })
                .collect::<Result<Vec<u32>, String>>()
                .map(CommaSeparatedU32List)
        }
    }
}

/// Defines the command-line arguments specific to `gitai` own subcommands.
/// This is typically used after determining that the invocation is not a global AI explanation request.
#[derive(Parser, Debug)]
#[clap(author = "huchen", version = "0.1.0", about="Git with AI support (enabled by default)", long_about=None, name="gitai-subcommand-parser")]
pub struct GitaiArgs {
    /// Enable AI functionality globally for all commands
    #[clap(long, global = true)]
    pub ai: bool,

    /// Diable AI functionality globally for all commands
    #[clap(long, global = true)]
    pub noai: bool,

    /// Specify output language (zh|en|auto)
    #[clap(long, global = true, value_name = "LANG")]
    pub language: Option<String>,

    #[command(subcommand)]
    pub command: GitaiSubCommand,
}

/// Represents the specific subcommands, that `gitai` itself understands.
#[derive(Subcommand, Debug, Clone)]
pub enum GitaiSubCommand {
    /// Handle git command operation, potentially with AI assistance for message generation
    #[clap(alias = "cm")]
    Commit(CommitArgs),
    /// Perform code review with AI assistance.
    #[clap(alias = "rv")]
    Review(ReviewArgs),
    /// Perform comprehensive code scanning with ast-grep
    #[clap(alias = "sc")]
    Scan(ScanArgs),
    /// Update ast-grep rules to the latest version
    #[clap(alias = "ur")]
    UpdateRules(UpdateRulesArgs),
    // Future: Add(AddArgs)
    // Future: Config(ConfigArgs)
}

/// Arguments for the `commit` subcommand
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct CommitArgs {
    /// Enable AstGrep syntax analysis for improved commit messages.
    /// Optional value can specify analysis depth: 'shallow', 'medium' (default), or 'deep'.
    #[clap(short = 't', long = "ast-grep")]
    pub ast_grep: bool,

    /// Automatically stage all tracked, modified files before commit (like git commit -a).
    #[clap(short = 'a', long = "all")]
    pub auto_stage: bool,

    /// Pass a message directly to the commit
    #[clap(short, long, value_name = "MESSAGE")]
    pub message: Option<String>,

    /// Issue IDs to include as prefix in commit message (e.g., "#123,#354")
    #[clap(long = "issue-id", value_name = "ISSUE_IDS")]
    pub issue_id: Option<String>,

    /// Perform code review before commit
    #[clap(short = 'r', long = "review")]
    pub review: bool,

    /// Allow all other flags and arguments to be passed through to the underlying `git commit`
    #[clap(allow_hyphen_values = true, last = true)]
    pub passthrough_args: Vec<String>,
}

/// Arguments for the `review` subcommand
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct ReviewArgs {
    /// Focus areas for the review
    #[clap(long, value_name = "AREA")]
    pub focus: Option<String>,

    /// Translation language (zh|en|auto)
    #[clap(long, value_name = "LANG")]
    pub lang: Option<String>,

    /// Output format
    #[clap(long, value_name = "FORMAT", default_value = "text")]
    pub format: String,

    /// Output file
    #[clap(long, value_name = "FILE")]
    pub output: Option<String>,

    /// Use ast-grep for enhanced code analysis (enabled by default)
    #[clap(long = "ast-grep", alias = "ag")]
    pub ast_grep: bool,

    /// Disable automatic code scanning during review
    #[clap(long = "no-scan")]
    pub no_scan: bool,

    /// Force code scanning even if cached results exist
    #[clap(long = "force-scan")]
    pub force_scan: bool,

    /// Use cached scan results if available
    #[clap(long = "use-cache")]
    pub use_cache: bool,

    /// First commit reference
    #[clap(long, value_name = "COMMIT")]
    pub commit1: Option<String>,

    /// Second commit reference (if comparing two commits)
    #[clap(long, value_name = "COMMIT")]
    pub commit2: Option<String>,

    /// Stories associated with the review
    #[clap(long, value_name = "STORIES", require_equals = true)]
    pub stories: Option<CommaSeparatedU32List>,

    /// Tasks associated with the review
    #[clap(long, value_name = "TASKS", require_equals = true)]
    pub tasks: Option<CommaSeparatedU32List>,

    /// Defects associated with the review
    #[clap(long, value_name = "DEFECTS", require_equals = true)]
    pub defects: Option<CommaSeparatedU32List>,

    /// Space ID for the review
    #[clap(long, value_name = "SPACE_ID")]
    pub space_id: Option<u32>,

    /// Allow all other flags and arguments to be passed through to git.
    #[clap(allow_hyphen_values = true, last = true)]
    pub passthrough_args: Vec<String>,
}

/// Arguments for the `scan` subcommand
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

    /// Rule severity levels to include (error,warning,info,hint)
    #[clap(long, value_name = "SEVERITY", default_value = "error,warning,info")]
    pub severity: String,

    /// Output format (text, json, sarif, csv)
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

    /// Custom rules configuration file
    #[clap(long, value_name = "FILE")]
    pub config: Option<String>,

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

/// Arguments for the `update-rules` subcommand
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct UpdateRulesArgs {
    /// Source to update rules from (github, local, url)
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

// Represents the entire Git diff
#[derive(Debug, Clone)]
pub struct GitDiff {
    pub changed_files: Vec<ChangedFile>,
    pub metadata: Option<HashMap<String, String>>,
}

// Defines the type of change in a Git diff
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    #[allow(dead_code)]
    Copied,
    #[allow(dead_code)]
    TypeChanged,
}

#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub hunks: Vec<DiffHunk>,
    pub file_mode_change: Option<String>,
}

// Represents a hunk range in git diff format (@@ -a,b +c,d @@)
#[derive(Debug, Clone)]
pub struct HunkRange {
    pub start: usize,
    pub count: usize,
}

// Represents a single hunk in a Git diff
#[derive(Debug, Clone)]
pub struct DiffHunk {
    #[allow(dead_code)]
    pub old_range: HunkRange,
    pub new_range: HunkRange,
    #[allow(dead_code)]
    pub lines: Vec<String>,
}

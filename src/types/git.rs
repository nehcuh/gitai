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

    /// Specify output language (cn|us|default)
    #[clap(long, global = true, value_name = "LANG")]
    pub lang: Option<String>,

    #[command(subcommand)]
    pub command: GitaiSubCommand,
}

/// Represents the specific subcommands, that `gitai` itself understands.
#[derive(Subcommand, Debug, Clone)]
pub enum GitaiSubCommand {
    /// Handle git commit operation, potentially with AI assistance for message generation
    #[clap(alias = "cm")]
    Commit(CommitArgs),
    /// Perform code review with AI assistance.
    #[clap(alias = "rv")]
    Review(ReviewArgs),
    /// Perform local code scan with ast-grep.
    Scan(ScanArgs),
    /// Translate various resources using AI
    Translate(TranslateArgs),
    // Future: Add(AddArgs)
    // Future: Config(ConfigArgs)
}

/// Arguments for the `commit` subcommand
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct CommitArgs {
    /// Enable Tree-sitter syntax analysis for improved commit messages.
    /// Optional value can specify analysis depth: 'shallow', 'medium' (default), or 'deep'.
    #[clap(short = 't', long = "tree-sitter")]
    pub tree_sitter: bool,

    /// When `tree-sitter` flag is enabled, this argument is used to control the analysis depth
    #[clap(short = 'l', long = "level", value_name = "TREESITTER_LEVEL")]
    pub depth: Option<String>,

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
    /// Analysis depth level
    #[clap(long, value_name = "LEVEL", default_value = "medium")]
    pub depth: String,

    /// Focus areas for the review
    #[clap(long, value_name = "AREA")]
    pub focus: Option<String>,

    /// Limit analysis to specific programming language
    #[clap(long, value_name = "PROGRAMMING_LANGUAGE")]
    pub language: Option<String>,

    /// Output format
    #[clap(long, value_name = "FORMAT", default_value = "text")]
    pub format: String,

    /// Output file
    #[clap(long, value_name = "FILE")]
    pub output: Option<String>,

    /// Use tree-sitter for enhanced code analysis (enabled by default)
    #[clap(long = "tree-sitter", alias = "ts")]
    pub tree_sitter: bool,

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
    /// Path to scan
    #[clap(value_name = "PATH")]
    pub path: Option<String>,

    /// Perform a full scan instead of an incremental one
    #[clap(long)]
    pub full: bool,

    /// Use remote scanning service
    #[clap(long)]
    pub remote: bool,

    /// Force update of local scan rules
    #[clap(long)]
    pub update_rules: bool,

    /// Output file
    #[clap(long, value_name = "FILE")]
    pub output: Option<String>,

    /// Output format
    #[clap(long, value_name = "FORMAT", default_value = "json")]
    pub format: String,
}

/// Arguments for the `translate` subcommand
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct TranslateArgs {
    /// Target to translate (rules)
    #[clap(value_name = "TARGET")]
    pub target: String,

    /// Force re-translation of existing files
    #[clap(short, long)]
    pub force: bool,

    /// Output directory for translated files
    #[clap(short, long, value_name = "DIR")]
    pub output: Option<PathBuf>,

    /// Target language for translation (cn|us)
    #[clap(short = 'l', long = "to-lang", value_name = "LANG", default_value = "cn")]
    pub to_language: String,
}

// Represents the entire Git diff
#[derive(Debug, Clone)]
pub struct GitDiff {
    pub changed_files: Vec<ChangedFile>,
    pub metadata: Option<HashMap<String, String>>,
}

// Defines the type of change in a Git diff
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

// DevOps work item types
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefectList(pub Vec<u32>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoryList(pub Vec<u32>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskList(pub Vec<u32>);

// Implement FromStr for DevOps work item types
impl FromStr for DefectList {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(DefectList(Vec::new()))
        } else {
            s.split(',')
                .map(|item_str| {
                    item_str
                        .trim()
                        .parse::<u32>()
                        .map_err(|e| format!("Invalid u32 value '{}': {}", item_str.trim(), e))
                })
                .collect::<Result<Vec<u32>, String>>()
                .map(DefectList)
        }
    }
}

impl FromStr for StoryList {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(StoryList(Vec::new()))
        } else {
            s.split(',')
                .map(|item_str| {
                    item_str
                        .trim()
                        .parse::<u32>()
                        .map_err(|e| format!("Invalid u32 value '{}': {}", item_str.trim(), e))
                })
                .collect::<Result<Vec<u32>, String>>()
                .map(StoryList)
        }
    }
}

impl FromStr for TaskList {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(TaskList(Vec::new()))
        } else {
            s.split(',')
                .map(|item_str| {
                    item_str
                        .trim()
                        .parse::<u32>()
                        .map_err(|e| format!("Invalid u32 value '{}': {}", item_str.trim(), e))
                })
                .collect::<Result<Vec<u32>, String>>()
                .map(TaskList)
        }
    }
}

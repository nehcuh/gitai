use clap::{Args, Parser, Subcommand};

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

    /// Specify output language
    // #[clap(long, global = true)]
    // pub language: Option<String>,

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
    // Future: Add(AddArgs)
    // Future: Config(ConfigArgs)
}

/// Arguments for the `commit` subcommand
#[derive(Args, Debug, Clone)]
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

    /// Perform code review before commit
    #[clap(short = 'r', long = "review")]
    pub review: bool,

    /// Allow all other flags and arguments to be passed through to the underlying `git commit`
    #[clap(allow_hyphen_values = true, last = true)]
    pub passthrough_args: Vec<String>,
}

/// Arguments for the `review` subcommand
#[derive(Args, Debug, Clone)]
pub struct ReviewArgs {
    /// Analysis depth level
    #[clap(long, value_name = "LEVEL", default_value = "medium")]
    pub depth: String,

    /// Focus areas for the review
    #[clap(long, value_name = "AREA")]
    pub focus: Option<String>,

    /// Limit analysis to specific language
    #[clap(long, value_name = "LANGUAGE")]
    pub lang: Option<String>,

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

    /// Allow all other flags and arguments to be passed through to git.
    #[clap(allow_hyphen_values = true, last = true)]
    pub passthrough_args: Vec<String>,
}

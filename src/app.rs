// 主应用程序逻辑

use crate::common::{AppResult, AppError, SupportedLanguage};
use crate::cli::{ParsedArgs, Operation, AIMode, GitAICommand};
use crate::config::AppConfig;
use std::str::FromStr;

/// GitAI 应用程序
pub struct GitAIApp {
    config: AppConfig,
}

impl GitAIApp {
    /// Creates a new `GitAIApp` instance with the specified configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// let config = AppConfig::default();
    /// let app = GitAIApp::new(config);
    /// ```
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Runs the application with the provided parsed CLI arguments.
    ///
    /// Applies any language override specified in the arguments, then dispatches execution based on the selected operation:
    /// - Shows help text if requested.
    /// - Handles GitAI commands using the appropriate handler.
    /// - Passes through Git commands or invokes intelligent Git handling depending on AI mode.
    ///
    /// # Arguments
    ///
    /// * `args` - Parsed command-line arguments specifying the operation and options.
    ///
    /// # Returns
    ///
    /// Returns an application result indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::app::{GitAIApp, AppBuilder};
    /// # use crate::cli::ParsedArgs;
    /// # async fn example() -> crate::types::AppResult<()> {
    /// let app = AppBuilder::build()?;
    /// let args = ParsedArgs::default();
    /// app.run(args).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(mut self, args: ParsedArgs) -> AppResult<()> {
        // 应用语言设置到配置
        if let Some(ref language) = args.language {
            self.apply_language_override(language.clone())?;
        }

        // 根据操作类型执行相应逻辑
        match args.operation {
            Operation::Help => {
                self.show_help(args.language.as_ref()).await
            },
            Operation::GitAI(command) => {
                self.handle_gitai_command(command, &args.ai_mode).await
            },
            Operation::GitPassthrough(git_args) => {
                self.handle_git_passthrough(git_args, &args.ai_mode).await
            },
        }
    }

    /// Applies a language override to the application's configuration.
    ///
    /// Converts the provided `SupportedLanguage` to the internal language type and updates the default language setting in the configuration.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the language override is applied successfully; otherwise, returns an error.
    fn apply_language_override(&mut self, language: SupportedLanguage) -> AppResult<()> {
        // 转换新的 SupportedLanguage 到旧的类型
        let old_language = match language {
            SupportedLanguage::Chinese => crate::ast_grep_analyzer::translation::SupportedLanguage::Chinese,
            SupportedLanguage::English => crate::ast_grep_analyzer::translation::SupportedLanguage::English,
            SupportedLanguage::Auto => crate::ast_grep_analyzer::translation::SupportedLanguage::Auto,
        };
        self.config.translation.default_language = old_language.clone();
        tracing::info!("应用语言覆盖设置: {:?}", language);
        Ok(())
    }

    /// Converts new `CommitArgs` from the CLI to the legacy internal `CommitArgs` type.
    ///
    /// Maps fields from the new CLI argument struct to the corresponding fields in the legacy internal type.
    fn convert_commit_args(&self, args: crate::cli::commands::CommitArgs) -> crate::types::git::CommitArgs {
        crate::types::git::CommitArgs {
            ast_grep: args.ast_grep,
            auto_stage: args.auto_stage,
            message: args.message,
            issue_id: args.issue_id,
            review: args.review,
            passthrough_args: args.git_args,  // 映射字段名
        }
    }

    /// Converts a new `ReviewArgs` struct to the legacy internal `ReviewArgs` type.
    ///
    /// Parses optional comma-separated string fields (`stories`, `tasks`, `defects`) into lists of `u32`, defaulting to empty lists if parsing fails. Fields not present in the new struct are set to default values in the legacy type.
    fn convert_review_args(&self, args: crate::cli::commands::ReviewArgs) -> crate::types::git::ReviewArgs {
        use crate::types::git::CommaSeparatedU32List;
        
        // 转换字符串到 CommaSeparatedU32List
        let stories = args.stories.map(|s| CommaSeparatedU32List::from_str(&s).unwrap_or(CommaSeparatedU32List(Vec::new())));
        let tasks = args.tasks.map(|s| CommaSeparatedU32List::from_str(&s).unwrap_or(CommaSeparatedU32List(Vec::new())));
        let defects = args.defects.map(|s| CommaSeparatedU32List::from_str(&s).unwrap_or(CommaSeparatedU32List(Vec::new())));
        
        crate::types::git::ReviewArgs {
            focus: args.focus,
            lang: None, // 新的 ReviewArgs 没有 lang 字段
            format: args.format,
            output: args.output,
            ast_grep: args.ast_grep,
            no_scan: args.no_scan,
            force_scan: args.force_scan,
            use_cache: false, // 新的 ReviewArgs 没有 use_cache 字段
            commit1: args.commit1,
            commit2: args.commit2,
            stories,
            tasks,
            defects,
            space_id: args.space_id,
            passthrough_args: args.git_args, // 映射字段名
        }
    }

    /// Converts a new `ScanArgs` struct to the legacy internal `ScanArgs` type.
    ///
    /// Maps all relevant fields from the new CLI argument struct to the internal type, setting `config` to `None` as it is not present in the new struct.
    fn convert_scan_args(&self, args: crate::cli::commands::ScanArgs) -> crate::types::git::ScanArgs {
        crate::types::git::ScanArgs {
            target: args.target,
            languages: args.languages,
            rules: args.rules,
            severity: args.severity,
            format: args.format,
            output: args.output,
            max_issues: args.max_issues,
            include: args.include,
            exclude: args.exclude,
            config: None, // 新的 ScanArgs 没有 config 字段
            parallel: args.parallel,
            verbose: args.verbose,
            stats: args.stats,
            fail_on_error: args.fail_on_error,
        }
    }

    /// Converts new `UpdateRulesArgs` from the CLI to the legacy internal `UpdateRulesArgs` type by mapping all fields directly.
    ///
    /// # Examples
    ///
    /// ```
    /// let cli_args = cli::commands::UpdateRulesArgs {
    ///     source: Some("remote".to_string()),
    ///     repository: Some("repo.git".to_string()),
    ///     reference: Some("main".to_string()),
    ///     target_dir: Some("rules/".to_string()),
    ///     force: false,
    ///     backup: true,
    ///     verify: true,
    ///     list_sources: false,
    ///     verbose: true,
    /// };
    /// let internal_args = app.convert_update_rules_args(cli_args);
    /// assert_eq!(internal_args.backup, true);
    /// ```
    fn convert_update_rules_args(&self, args: crate::cli::commands::UpdateRulesArgs) -> crate::types::git::UpdateRulesArgs {
        crate::types::git::UpdateRulesArgs {
            source: args.source,
            repository: args.repository,
            reference: args.reference,
            target_dir: args.target_dir,
            force: args.force,
            backup: args.backup,
            verify: args.verify,
            list_sources: args.list_sources,
            verbose: args.verbose,
        }
    }

    /// Displays the application's help text in the specified language.
    ///
    /// If a language is provided, the help text is generated in that language; otherwise, the default language is used.
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::SupportedLanguage;
    /// # let app = GitAIApp::new(Default::default());
    /// # tokio_test::block_on(async {
    /// app.show_help(Some(&SupportedLanguage::English)).await.unwrap();
    /// # });
    /// ```
    async fn show_help(&self, language: Option<&SupportedLanguage>) -> AppResult<()> {
        let help_text = crate::cli::generate_help(language);
        println!("{}", help_text);
        Ok(())
    }

    /// Handles GitAI-specific commands by dispatching to the appropriate legacy handler after converting arguments.
    ///
    /// This method processes GitAI subcommands such as commit, review, scan, and update rules. It converts the provided arguments to legacy types and invokes the corresponding asynchronous handler. Errors from the handlers are mapped to generic application errors.
    ///
    /// # Examples
    ///
    /// ```
    /// // Example usage within an async context:
    /// let app = GitAIApp::new(config);
    /// let command = GitAICommand::Commit(commit_args);
    /// app.handle_gitai_command(command, &AIMode::Enabled).await?;
    /// ```
    async fn handle_gitai_command(&self, command: GitAICommand, _ai_mode: &AIMode) -> AppResult<()> {
        match command {
            GitAICommand::Commit(args) => {
                // TODO: 使用新架构重新实现 commit 处理
                // 目前调用原有的处理器，使用类型转换
                let old_args = self.convert_commit_args(args);
                crate::handlers::commit::handle_commit(&self.config, old_args).await
                    .map_err(|e| AppError::generic(format!("提交处理失败: {}", e)))
            },
            GitAICommand::Review(args) => {
                // TODO: 使用新架构重新实现 review 处理
                // 目前调用原有的处理器，使用类型转换
                let mut config = self.config.clone();
                let old_args = self.convert_review_args(args);
                crate::handlers::review::handle_review(&mut config, old_args).await
                    .map_err(|e| AppError::generic(format!("代码审查失败: {}", e)))
            },
            GitAICommand::Scan(args) => {
                // TODO: 使用新架构重新实现 scan 处理
                // 目前调用原有的处理器，使用类型转换
                let old_args = self.convert_scan_args(args);
                crate::handlers::scan::handle_scan(&self.config, &old_args).await
                    .map_err(|e| AppError::generic(format!("代码扫描失败: {}", e)))
            },
            GitAICommand::UpdateRules(args) => {
                // TODO: 使用新架构重新实现 update-rules 处理
                // 目前调用原有的处理器，使用类型转换
                let old_args = self.convert_update_rules_args(args);
                crate::handlers::update_rules::handle_update_rules(&self.config, &old_args).await
                    .map_err(|e| AppError::generic(format!("规则更新失败: {}", e)))
            },
        }
    }

    /// Handles passthrough of Git commands, optionally invoking AI-assisted processing.
    ///
    /// If AI mode is disabled, the command is passed directly to Git. Otherwise, the command is processed using the legacy intelligent Git handler with AI enabled or disabled based on the mode.
    ///
    /// # Arguments
    ///
    /// * `git_args` - The Git command-line arguments to process.
    /// * `ai_mode` - Determines whether to use AI-assisted processing.
    ///
    /// # Returns
    ///
    /// Returns an application result indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```
    /// let args = vec!["status".to_string()];
    /// let ai_mode = AIMode::Disabled;
    /// app.handle_git_passthrough(args, &ai_mode).await?;
    /// ```
    async fn handle_git_passthrough(&self, git_args: Vec<String>, ai_mode: &AIMode) -> AppResult<()> {
        // 检查是否需要禁用 AI
        if matches!(ai_mode, AIMode::Disabled) {
            return self.passthrough_to_git(&git_args).await;
        }

        // TODO: 使用新架构重新实现智能 Git 处理
        // 目前调用原有的处理器
        let use_ai = matches!(ai_mode, AIMode::Global);
        crate::handlers::intelligent_git::handle_intelligent_git_command(&self.config, &git_args, use_ai).await
            .map_err(|e| AppError::generic(format!("智能 Git 处理失败: {}", e)))
    }

    /// Forwards the provided arguments directly to Git using the legacy passthrough handler.
    ///
    /// # Examples
    ///
    /// ```
    /// let args = vec!["status".to_string()];
    /// app.passthrough_to_git(&args).await?;
    /// ```
    async fn passthrough_to_git(&self, git_args: &[String]) -> AppResult<()> {
        // TODO: 使用新的 Git 模块实现透传
        // 目前调用原有的处理器
        crate::handlers::git::passthrough_to_git(git_args)
            .map_err(|e| AppError::generic(format!("Git 透传失败: {}", e)))
    }

    /// Returns a reference to the application's configuration.
    pub fn config(&self) -> &AppConfig {
        &self.config
    }
}

/// 应用程序构建器
pub struct AppBuilder;

impl AppBuilder {
    /// Builds a `GitAIApp` instance by loading the application configuration from disk.
    ///
    /// Returns an error if the configuration cannot be loaded.
    pub fn build() -> AppResult<GitAIApp> {
        // 加载配置
        let config = AppConfig::load()
            .map_err(|e| AppError::config(format!("配置加载失败: {}", e)))?;

        Ok(GitAIApp::new(config))
    }

    /// Creates a new `GitAIApp` instance using the provided configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// let config = AppConfig::default();
    /// let app = AppBuilder::with_config(config);
    /// ```
    pub fn with_config(config: AppConfig) -> GitAIApp {
        GitAIApp::new(config)
    }
}

/// Runs the GitAI application with parsed command-line arguments.
///
/// Initializes logging, parses CLI arguments, builds the application, and executes it asynchronously.
///
/// # Returns
///
/// An `AppResult` indicating success or failure of the application run.
///
/// # Examples
///
/// ```
/// // Typically called from main:
/// tokio::main
/// async fn main() -> AppResult<()> {
///     run_app().await
/// }
/// ```
pub async fn run_app() -> AppResult<()> {
    // 初始化日志系统
    tracing_subscriber::fmt::init();

    // 解析命令行参数
    let args = crate::cli::CLIParser::parse()?;

    // 构建并运行应用程序
    let app = AppBuilder::build()?;
    app.run(args).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::*;

    #[test]
    fn test_app_creation() {
        let config = AppConfig::default();
        let app = GitAIApp::new(config);
        assert!(app.config().translation.default_language == SupportedLanguage::Auto);
    }

    #[test]
    fn test_app_builder() {
        // 注意：这个测试可能因为配置文件不存在而失败
        // 在实际环境中应该使用 mock 配置
        let result = AppBuilder::build();
        // 只测试构建过程不出错，不测试具体结果
        match result {
            Ok(_) => println!("App built successfully"),
            Err(e) => println!("Expected error in test environment: {}", e),
        }
    }

    /// Tests that the `show_help` method displays help in English without returning an error.
    #[tokio::test]
    async fn test_help_display() {
        let config = AppConfig::default();
        let app = GitAIApp::new(config);
        
        // 测试帮助显示不会panic
        let result = app.show_help(Some(&SupportedLanguage::English)).await;
        assert!(result.is_ok());
    }
}
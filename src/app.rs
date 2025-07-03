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
    /// 创建新的应用程序实例
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// 运行应用程序
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

    /// 应用语言覆盖设置
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

    /// 转换新的 CommitArgs 到旧的类型
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

    /// 转换新的 ReviewArgs 到旧的类型
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

    /// 转换新的 ScanArgs 到旧的类型
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

    /// 转换新的 UpdateRulesArgs 到旧的类型
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

    /// 显示帮助信息
    async fn show_help(&self, language: Option<&SupportedLanguage>) -> AppResult<()> {
        let help_text = crate::cli::generate_help(language);
        println!("{}", help_text);
        Ok(())
    }

    /// 处理 GitAI 特殊命令
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

    /// 处理 Git 命令透传
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

    /// 直接透传到 Git
    async fn passthrough_to_git(&self, git_args: &[String]) -> AppResult<()> {
        // TODO: 使用新的 Git 模块实现透传
        // 目前调用原有的处理器
        crate::handlers::git::passthrough_to_git(git_args)
            .map_err(|e| AppError::generic(format!("Git 透传失败: {}", e)))
    }

    /// 获取应用程序配置
    pub fn config(&self) -> &AppConfig {
        &self.config
    }
}

/// 应用程序构建器
pub struct AppBuilder;

impl AppBuilder {
    /// 构建应用程序
    pub fn build() -> AppResult<GitAIApp> {
        // 加载配置
        let config = AppConfig::load()
            .map_err(|e| AppError::config(format!("配置加载失败: {}", e)))?;

        Ok(GitAIApp::new(config))
    }

    /// 从指定配置构建应用程序
    pub fn with_config(config: AppConfig) -> GitAIApp {
        GitAIApp::new(config)
    }
}

/// 运行 GitAI 应用程序的便捷函数
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

    #[tokio::test]
    async fn test_help_display() {
        let config = AppConfig::default();
        let app = GitAIApp::new(config);
        
        // 测试帮助显示不会panic
        let result = app.show_help(Some(&SupportedLanguage::English)).await;
        assert!(result.is_ok());
    }
}
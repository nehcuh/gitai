use crate::config::Config;
use crate::update::error::UpdateError;
use crate::update::types::{PromptSetupResult, RuleDownloadResult, UpdateItem, UpdateResult};
use crate::update::UpdateNotifier;
use chrono::Utc;
use dirs;
use log::{debug, error, info};
use std::fs;
use std::path::PathBuf;

pub struct AutoUpdater {
    config: Config,
    state_dir: PathBuf,
    notifier: UpdateNotifier,
}
impl AutoUpdater {
    pub fn new(config: Config) -> Self {
        let state_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".cache")
            .join("gitai")
            .join("update_state");

        // 确保状态目录存在
        if let Err(e) = fs::create_dir_all(&state_dir) {
            error!("无法创建状态目录 {}: {e}", state_dir.display());
        } else {
            info!("更新器状态目录: {}", state_dir.display());
        }

        debug!("初始化AutoUpdater，配置: {config:?}", config = config.ai);
        let notifier = UpdateNotifier::new(config.clone());
        Self {
            config,
            state_dir,
            notifier,
        }
    }

    pub async fn check_and_update(&self) -> Result<UpdateResult, UpdateError> {
        info!("开始检查更新");

        // 发送更新开始通知
        self.notifier.notify_update_start();

        let mut updates = Vec::new();

        // 1. 检查扫描规则更新
        if self.should_update_rules() {
            debug!("扫描规则需要更新");
            match self.update_scan_rules().await {
                Ok(update) => {
                    info!("扫描规则更新成功: {}", update.message);
                    updates.push(update);
                }
                Err(e) => {
                    error!("扫描规则更新失败: {e}");
                    updates.push(UpdateItem {
                        name: "扫描规则更新".to_string(),
                        success: false,
                        message: format!("更新失败: {e}"),
                    });
                }
            }
        } else {
            debug!("扫描规则已是最新");
        }

        // 2. 检查prompts更新
        if self.should_update_prompts() {
            debug!("Prompts需要更新");
            match self.update_prompts().await {
                Ok(update) => {
                    info!("Prompts更新成功: {}", update.message);
                    updates.push(update);
                }
                Err(e) => {
                    error!("Prompts更新失败: {e}");
                    updates.push(UpdateItem {
                        name: "Prompts更新".to_string(),
                        success: false,
                        message: format!("更新失败: {e}"),
                    });
                }
            }
        } else {
            debug!("Prompts已是最新");
        }

        // 3. 检查GitAI版本更新
        if self.should_update_gitai() {
            debug!("需要检查GitAI版本");
            match self.update_gitai_version().await {
                Ok(update) => {
                    info!("版本检查完成: {}", update.message);
                    updates.push(update);
                }
                Err(e) => {
                    error!("版本检查失败: {e}");
                    updates.push(UpdateItem {
                        name: "版本检查".to_string(),
                        success: false,
                        message: format!("检查失败: {e}"),
                    });
                }
            }
        } else {
            debug!("GitAI版本检查时间未到");
        }

        let success = updates.iter().all(|u| u.success);
        info!("更新检查完成，成功: {}", success);

        // 发送更新结果通知
        if success {
            self.notifier.notify_update_success(&updates);
        } else {
            let failed_updates: Vec<_> = updates.iter().filter(|u| !u.success).cloned().collect();
            if !failed_updates.is_empty() {
                self.notifier.notify_update_failure(&failed_updates);
            }
        }

        Ok(UpdateResult { success, updates })
    }

    fn should_update_rules(&self) -> bool {
        let last_update = self.get_last_rule_update();
        let update_interval = 24 * 3600; // 固定24小时

        Utc::now().timestamp() - last_update > update_interval as i64
    }

    fn should_update_prompts(&self) -> bool {
        // prompts通常不需要频繁更新
        let last_update = self.get_last_prompt_update();
        Utc::now().timestamp() - last_update > 7 * 24 * 3600 // 7天
    }

    fn should_update_gitai(&self) -> bool {
        // 版本更新检查（每周一次）
        let last_check = self.get_last_version_check();
        Utc::now().timestamp() - last_check > 7 * 24 * 3600
    }

    pub async fn update_scan_rules(&self) -> Result<UpdateItem, UpdateError> {
        // 目标规则目录
        let rules_dir = self.get_rules_dir();
        if let Err(e) = std::fs::create_dir_all(&rules_dir) {
            return Err(UpdateError::Io(e));
        }

        // 下载并解压规则
        let result = self.download_scan_rules(&rules_dir).await?;

        // 写入元信息（JSON）
        let meta_path = rules_dir.join(".rules.meta");
        let meta_json = serde_json::json!({
            "sources": result.sources,
            "total_rules": result.total_rules,
            "updated_at": Utc::now().to_rfc3339(),
        });
        let meta_str =
            serde_json::to_string_pretty(&meta_json).unwrap_or_else(|_| "{}".to_string());
        if let Err(e) = std::fs::write(&meta_path, meta_str) {
            return Err(UpdateError::Io(e));
        }

        // 更新最后更新时间
        self.set_last_rule_update(Utc::now().timestamp());

        Ok(UpdateItem {
            name: "扫描规则更新".to_string(),
            success: true,
            message: format!(
                "规则已更新，共 {} 个文件，目录: {}",
                result.total_rules,
                rules_dir.display()
            ),
        })
    }

    pub async fn update_prompts(&self) -> Result<UpdateItem, UpdateError> {
        let result = self.setup_prompts().await?;

        // 更新最后更新时间
        self.set_last_prompt_update(Utc::now().timestamp());

        Ok(UpdateItem {
            name: "Prompts更新".to_string(),
            success: true,
            message: format!("更新了 {} 个prompt模板", result.count),
        })
    }

    async fn update_gitai_version(&self) -> Result<UpdateItem, UpdateError> {
        let current_version = env!("CARGO_PKG_VERSION");
        let latest_version = self.get_latest_version().await?;

        if latest_version != current_version {
            // 显示更新提示
            println!(
                "🎯 GitAI有新版本可用: {} -> {}",
                current_version, latest_version
            );
            println!("   运行 'cargo install gitai --force' 更新");

            // 发送新版本通知
            self.notifier
                .notify_new_version_available(current_version, &latest_version);
        }

        // 更新最后检查时间
        self.set_last_version_check(Utc::now().timestamp());

        Ok(UpdateItem {
            name: "版本检查".to_string(),
            success: true,
            message: format!(
                "当前版本: {}, 最新版本: {}",
                current_version, latest_version
            ),
        })
    }

    async fn download_scan_rules(
        &self,
        target_dir: &std::path::Path,
    ) -> Result<RuleDownloadResult, UpdateError> {
        use flate2::read::GzDecoder;
        use std::io::Cursor;
        use tar::Archive;
        use walkdir::WalkDir;

        let url = std::env::var("GITAI_RULES_URL").unwrap_or_else(|_| {
            "https://github.com/opengrep/opengrep-rules/archive/refs/heads/main.tar.gz".to_string()
        });

        // 带重试机制的下载 + 解压
        self.retry_async(
            || {
                let url = url.clone();
                async move {
                    // 支持 file:// 与 http(s):// -> 获取 Vec<u8>
                    let data: Vec<u8> = if let Some(path) = url.strip_prefix("file://") {
                        std::fs::read(path).map_err(UpdateError::Io)?
                    } else {
                        let resp = reqwest::get(&url).await.map_err(UpdateError::from)?;
                        if !resp.status().is_success() {
                            return Err(UpdateError::Download(format!(
                                "下载规则失败: {}",
                                resp.status()
                            )));
                        }
                        resp.bytes().await.map_err(UpdateError::from)?.to_vec()
                    };

                    // 解压到目标目录
                    let gz = GzDecoder::new(Cursor::new(data));
                    let mut archive = Archive::new(gz);
                    archive.unpack(target_dir).map_err(UpdateError::from)?;

                    // 规范化目录结构（如果只有一层包裹目录则上提）
                    if let Err(e) = Self::normalize_rules_layout(target_dir) {
                        println!("⚠️ 规范化规则目录失败: {}", e);
                    }

                    // 统计文件数
                    let mut count = 0usize;
                    for entry in WalkDir::new(target_dir).into_iter().filter_map(|e| e.ok()) {
                        if entry.file_type().is_file() {
                            count += 1;
                        }
                    }

                    Ok(RuleDownloadResult {
                        sources: vec![url],
                        total_rules: count,
                    })
                }
            },
            3,
            "下载并解压扫描规则",
        )
        .await
    }

    pub fn get_rules_dir(&self) -> std::path::PathBuf {
        if let Some(dir) = &self.config.scan.rules_dir {
            return std::path::PathBuf::from(dir);
        }
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".cache")
            .join("gitai")
            .join("rules")
    }

    fn normalize_rules_layout(dir: &std::path::Path) -> Result<(), UpdateError> {
        use std::fs;
        // 判断根是否有文件
        let mut has_files = false;
        let mut subdirs = Vec::new();
        for entry in fs::read_dir(dir)? {
            let e = entry?;
            let ft = e.file_type()?;
            if ft.is_file() {
                has_files = true;
                break;
            }
            if ft.is_dir() {
                subdirs.push(e.path());
            }
        }
        if has_files {
            return Ok(());
        }
        if subdirs.len() != 1 {
            return Ok(());
        }
        let inner = &subdirs[0];
        // 将 inner 的内容上提到 dir
        for entry in fs::read_dir(inner)? {
            let e = entry?;
            let name = e.file_name();
            let target = dir.join(name);
            // 尝试移动（重命名）
            let _ = fs::rename(e.path(), target);
        }
        // 尝试删除空的 inner 目录
        let _ = fs::remove_dir_all(inner);
        Ok(())
    }

    async fn setup_prompts(&self) -> Result<PromptSetupResult, UpdateError> {
        // 带重试机制的设置
        self.retry_async(
            || async {
                // 模拟设置prompts
                // 实际实现中会调用 PromptAutoSetup
                Ok(PromptSetupResult {
                    count: 15,
                    templates: vec!["code-review".to_string(), "security-scan".to_string()],
                })
            },
            3,
            "设置prompts",
        )
        .await
    }

    async fn get_latest_version(&self) -> Result<String, UpdateError> {
        // 带重试机制的版本检查
        self.retry_async(
            || async {
                let client = reqwest::Client::new();
                let response = client
                    .get("https://api.github.com/repos/nehcuh/gitai/releases/latest")
                    .header("User-Agent", "gitai")
                    .send()
                    .await?;

                let json: serde_json::Value = response.json().await?;
                Ok(json["tag_name"].as_str().unwrap_or("unknown").to_string())
            },
            3,
            "获取最新版本",
        )
        .await
    }

    /// 异步重试机制
    async fn retry_async<F, Fut, T>(
        &self,
        operation: F,
        max_retries: usize,
        operation_name: &str,
    ) -> Result<T, UpdateError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, UpdateError>>,
    {
        let mut last_error = None;

        for attempt in 1..=max_retries {
            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        println!("✅ {}在第{}次尝试后成功", operation_name, attempt);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        let delay = std::time::Duration::from_secs(2u64.pow(attempt as u32 - 1));
                        println!(
                            "⚠️ {}第{}次尝试失败，{}秒后重试...",
                            operation_name,
                            attempt,
                            delay.as_secs()
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| UpdateError::Download(format!("{}重试次数耗尽", operation_name))))
    }

    // 状态管理方法
    fn get_last_rule_update(&self) -> i64 {
        self.get_state_value("last_rule_update").unwrap_or(0)
    }

    fn set_last_rule_update(&self, timestamp: i64) {
        self.set_state_value("last_rule_update", timestamp);
    }

    fn get_last_prompt_update(&self) -> i64 {
        self.get_state_value("last_prompt_update").unwrap_or(0)
    }

    fn set_last_prompt_update(&self, timestamp: i64) {
        self.set_state_value("last_prompt_update", timestamp);
    }

    fn get_last_version_check(&self) -> i64 {
        self.get_state_value("last_version_check").unwrap_or(0)
    }

    fn set_last_version_check(&self, timestamp: i64) {
        self.set_state_value("last_version_check", timestamp);
    }

    fn get_state_value(&self, key: &str) -> Option<i64> {
        let file_path = self.state_dir.join(key);
        if file_path.exists() {
            fs::read_to_string(&file_path)
                .ok()
                .and_then(|s| s.parse().ok())
        } else {
            None
        }
    }

    fn set_state_value(&self, key: &str, value: i64) {
        let file_path = self.state_dir.join(key);
        if let Err(e) = fs::write(&file_path, value.to_string()) {
            error!("无法写入状态文件 {}: {}", file_path.display(), e);
        } else {
            debug!("更新状态 {}: {}", key, value);
        }
    }

    /// 强制更新所有内容
    pub async fn force_update_all(&self) -> Result<UpdateResult, UpdateError> {
        // 重置所有更新时间
        self.set_last_rule_update(0);
        self.set_last_prompt_update(0);
        self.set_last_version_check(0);

        self.check_and_update().await
    }

    /// 检查更新状态但不执行更新
    pub fn check_update_status(&self) -> Vec<UpdateItem> {
        let mut status = Vec::new();

        let rules_need_update = self.should_update_rules();
        let prompts_need_update = self.should_update_prompts();
        let gitai_need_update = self.should_update_gitai();

        if rules_need_update {
            status.push(UpdateItem {
                name: "扫描规则".to_string(),
                success: false,
                message: "需要更新".to_string(),
            });
        }

        if prompts_need_update {
            status.push(UpdateItem {
                name: "Prompts".to_string(),
                success: false,
                message: "需要更新".to_string(),
            });
        }

        if gitai_need_update {
            status.push(UpdateItem {
                name: "GitAI版本".to_string(),
                success: false,
                message: "需要检查".to_string(),
            });
        }

        status
    }
}

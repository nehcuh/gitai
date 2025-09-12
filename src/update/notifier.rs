use gitai_core::config::Config;
use crate::update::error::UpdateError;
use crate::update::types::UpdateItem;
use std::process::Command;

pub struct UpdateNotifier {
    #[allow(dead_code)]
    config: Config,
}

impl UpdateNotifier {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// 发送桌面通知
    pub fn send_desktop_notification(&self, title: &str, message: &str) -> Result<(), UpdateError> {
        // 尝试使用系统通知
        #[cfg(target_os = "macos")]
        {
            fn escape_osascript(s: &str) -> String {
                s.replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
            }
            let title_e = escape_osascript(title);
            let message_e = escape_osascript(message);
            let script = format!(
                "display notification \"{}\" with title \"{}\"",
                message_e, title_e
            );
            let output = Command::new("osascript").args(["-e", &script]).output()?;

            if output.status.success() {
                return Ok(());
            }
        }

        #[cfg(target_os = "linux")]
        {
            // 作为独立参数传递，避免 shell 插值问题
            let output = Command::new("notify-send")
                .args([title, message])
                .output()?;

            if output.status.success() {
                return Ok(());
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Windows通知实现可以添加
        }

        // 如果系统通知失败，记录到日志
        log::info!("通知: {} - {}", title, message);
        Ok(())
    }

    /// 发送更新成功通知
    pub fn notify_update_success(&self, updates: &[UpdateItem]) {
        let success_count = updates.iter().filter(|u| u.success).count();
        let title = "GitAI 更新完成";
        let message = format!("成功更新了 {} 个组件", success_count);

        if let Err(e) = self.send_desktop_notification(title, &message) {
            log::warn!("发送更新成功通知失败: {}", e);
        }
    }

    /// 发送更新失败通知
    pub fn notify_update_failure(&self, failed_updates: &[UpdateItem]) {
        let title = "GitAI 更新失败";
        let message = format!("{} 个组件更新失败", failed_updates.len());

        if let Err(e) = self.send_desktop_notification(title, &message) {
            log::warn!("发送更新失败通知失败: {}", e);
        }
    }

    /// 发送新版本可用通知
    pub fn notify_new_version_available(&self, current: &str, latest: &str) {
        let title = "GitAI 新版本可用";
        let message = format!("{} -> {}", current, latest);

        if let Err(e) = self.send_desktop_notification(title, &message) {
            log::warn!("发送新版本通知失败: {}", e);
        }
    }

    /// 发送更新开始通知
    pub fn notify_update_start(&self) {
        let title = "GitAI 正在更新";
        let message = "正在检查并更新组件...";

        if let Err(e) = self.send_desktop_notification(title, message) {
            log::warn!("发送更新开始通知失败: {}", e);
        }
    }
}

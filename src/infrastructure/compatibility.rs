//! 向后兼容层
//! 
//! 提供从旧版本API到新版本API的平滑迁移路径
//! 
//! # 迁移指南
//! 
//! ## 服务注册变更
//! 
//! ### 旧版本 (1.x)
//! ```rust
//! container.register_service(|| MyService::new()).await;
//! ```
//! 
//! ### 新版本 (2.x)
//! ```rust
//! container.register_singleton_simple(|| MyService::new()).await;
//! ```
//! 
//! ## 服务解析变更
//! 
//! ### 旧版本 (1.x)
//! ```rust
//! let service = container.get_service::<MyService>().await?;
//! ```
//! 
//! ### 新版本 (2.x)
//! ```rust
//! let service = container.resolve::<MyService>().await?;
//! // service 是 Arc<MyService>，需要时可以使用 (*service).clone()
//! ```

use super::container::{ServiceContainer, ContainerError};
use super::container::ServiceLifetime;
use std::sync::Arc;

/// 兼容性包装器，提供旧版本API
pub struct CompatibilityWrapper {
    container: ServiceContainer,
}

impl CompatibilityWrapper {
    /// 创建新的兼容性包装器
    pub fn new(container: ServiceContainer) -> Self {
        Self { container }
    }
    
    /// 获取内部容器（用于迁移到新API）
    pub fn into_inner(self) -> ServiceContainer {
        self.container
    }
    
    /// 获取内部容器的引用
    pub fn inner(&self) -> &ServiceContainer {
        &self.container
    }
    
    // ===== 旧版本兼容API =====
    
    /// 注册服务（兼容旧API，默认单例）
    pub async fn register_service<T, F>(&self, factory: F) 
    where
        T: Send + Sync + Clone + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.container.register_singleton_simple(move || Ok(factory())).await;
    }
    
    /// 获取服务实例（兼容旧API，返回克隆值）
    pub async fn get_service<T: Send + Sync + Clone + 'static>(
        &self
    ) -> Result<T, ContainerError> {
        let arc_service = self.container.resolve::<T>().await?;
        Ok((*arc_service).clone())
    }
    
    /// 检查容器是否包含服务（兼容旧API）
    pub async fn contains_service<T: 'static>(
        &self
    ) -> bool {
        self.container.is_registered::<T>().await
    }
    
    /// 解析服务（兼容旧API）
    pub async fn resolve_service<T: Send + Sync + Clone + 'static>(
        &self
    ) -> Result<T, ContainerError> {
        self.get_service::<T>().await
    }
    
    /// 获取容器状态（兼容旧API）
    pub async fn get_container_status(
        &self
    ) -> String {
        self.container.get_performance_summary().await
    }
    
    /// 获取缓存命中率（兼容旧API，百分比格式）
    pub async fn get_cache_hit_percentage(
        &self
    ) -> f64 {
        let stats = self.container.get_stats().await;
        stats.cache_hit_rate()
    }
}

/// 迁移助手，帮助从旧API迁移到新API
pub struct MigrationHelper;

impl MigrationHelper {
    /// 创建迁移报告
    pub fn create_migration_report() -> String {
        format!(
            r#"
GitAI 容器 API 迁移报告
========================

当前API版本: {}
建议迁移路径:

1. 服务注册:
   - register_service() → register_singleton_simple()
   - register_transient_service() → register_transient_simple()
   - register_scoped_service() → register_scoped_simple()

2. 服务解析:
   - get_service() → resolve() (注意返回Arc<T>)
   - resolve_service() → resolve() (注意返回Arc<T>)

3. 状态查询:
   - get_container_status() → get_performance_summary()
   - get_cache_hit_percentage() → get_stats().cache_hit_rate()

4. 服务检查:
   - contains_service() → is_registered()

迁移建议:
- 逐步替换，不要一次性全部更改
- 优先使用明确指定生命周期的新API
- 利用Arc<T>的优势，减少不必要的克隆

兼容性保证:
- 所有旧API在2.x版本中仍然可用
- 提供详细的弃用警告和迁移建议
- 兼容性层将在3.0版本中移除
            "#,
            ServiceContainer::check_api_version()
        )
    }
    
    /// 检查代码中的潜在兼容性问题
    pub fn check_compatibility_issues(code: &str) -> Vec<String> {
        let mut issues = Vec::new();
        
        if code.contains("register_service") {
            issues.push("检测到 register_service()，建议使用 register_singleton_simple()".to_string());
        }
        
        if code.contains("get_service") {
            issues.push("检测到 get_service()，建议使用 resolve() 并处理 Arc<T>".to_string());
        }
        
        if code.contains("contains_service") {
            issues.push("检测到 contains_service()，建议使用 is_registered()".to_string());
        }
        
        if code.contains("get_container_status") {
            issues.push("检测到 get_container_status()，建议使用 get_performance_summary()".to_string());
        }
        
        issues
    }
    
    /// 提供自动迁移建议
    pub fn suggest_migration(code: &str) -> String {
        let issues = Self::check_compatibility_issues(code);
        
        if issues.is_empty() {
            return "未检测到兼容性问题，代码已经是最新API版本。".to_string();
        }
        
        let mut suggestions = String::from("迁移建议:\n");
        for (i, issue) in issues.iter().enumerate() {
            suggestions.push_str(&format!("{}. {}\n", i + 1, issue));
        }
        
        suggestions.push_str("\n详细迁移指南:\n");
        suggestions.push_str(Self::create_migration_report().as_str());
        
        suggestions
    }
}

/// 兼容性宏，简化迁移过程
#[macro_export]
macro_rules! compat_register {
    ($container:expr, $factory:expr, singleton) => {
        $container.register_singleton_simple($factory).await
    };
    ($container:expr, $factory:expr, transient) => {
        $container.register_transient_simple($factory).await
    };
    ($container:expr, $factory:expr, scoped) => {
        $container.register_scoped_simple($factory).await
    };
}

/// 兼容性结果类型
pub type CompatibilityResult<T> = Result<T, ContainerError>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_compatibility_wrapper() {
        let container = ServiceContainer::new();
        let compat = CompatibilityWrapper::new(container);
        
        // 测试基本功能
        let status = compat.get_container_status().await;
        assert!(status.contains("Container Performance"));
        
        let hit_rate = compat.get_cache_hit_percentage().await;
        assert_eq!(hit_rate, 0.0); // 新容器应该为0
    }
    
    #[test]
    fn test_migration_helper() {
        let report = MigrationHelper::create_migration_report();
        assert!(report.contains("当前API版本"));
        assert!(report.contains("迁移建议"));
    }
    
    #[test]
    fn test_compatibility_check() {
        let code = r#"
            container.register_service(|| MyService::new());
            let service = container.get_service::<MyService>().await?;
        "#;
        
        let issues = MigrationHelper::check_compatibility_issues(code);
        assert_eq!(issues.len(), 2);
        assert!(issues[0].contains("register_service"));
        assert!(issues[1].contains("get_service"));
    }
    
    #[test]
    fn test_migration_suggestions() {
        let code = "container.register_service(|| MyService::new());";
        let suggestions = MigrationHelper::suggest_migration(code);
        
        assert!(suggestions.contains("迁移建议"));
        assert!(suggestions.contains("register_service"));
        assert!(suggestions.contains("register_singleton_simple"));
    }
}
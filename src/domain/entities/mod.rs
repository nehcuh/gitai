//! 领域实体定义
//! 
//! 包含核心业务对象和数据结构

pub mod git;
pub mod review;
pub mod scan;
pub mod common;


use std::fmt;

/// 聚合根trait
/// 标识这是一个聚合根实体
pub trait AggregateRoot: Send + Sync {
    /// 获取聚合根ID
    fn id(&self) -> String;
    
    /// 验证聚合根的一致性
    fn validate(&self) -> Result<(), String>;
}

/// 实体trait
pub trait Entity: Send + Sync {
    /// 获取实体ID
    fn entity_id(&self) -> String;
}

/// 值对象trait
pub trait ValueObject: Send + Sync + Clone + PartialEq {
    /// 验证值对象的有效性
    fn is_valid(&self) -> bool;
}

/// 领域事件trait
pub trait DomainEvent: Send + Sync {
    /// 获取事件ID
    fn event_id(&self) -> String;
    
    /// 获取事件类型
    fn event_type(&self) -> &str;
    
    /// 获取事件发生时间
    fn occurred_at(&self) -> chrono::DateTime<chrono::Utc>;
    
    /// 获取事件数据
    fn event_data(&self) -> serde_json::Value;
}

/// 领域事件发布者trait
#[async_trait::async_trait]
pub trait DomainEventPublisher: Send + Sync {
    /// 发布领域事件
    async fn publish<E: DomainEvent + 'static>(&self, event: E) -> Result<(), String>;
    
    /// 批量发布领域事件
    async fn publish_batch<E: DomainEvent + 'static>(
        &self, 
        events: Vec<E>
    ) -> Result<(), String>;
}

/// 领域事件订阅者trait
#[async_trait::async_trait]
pub trait DomainEventSubscriber: Send + Sync {
    /// 处理领域事件
    async fn handle<E: DomainEvent + 'static>(&self, event: &E) -> Result<(), String>;
    
    /// 获取订阅的事件类型
    fn subscribed_events(&self) -> Vec<String>;
}

/// 业务规则验证trait
pub trait BusinessRuleValidator: Send + Sync {
    /// 验证业务规则
    fn validate(&self) -> Result<(), BusinessRuleViolation>;
}

/// 业务规则违反
#[derive(Debug, Clone)]
pub struct BusinessRuleViolation {
    pub rule_name: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl BusinessRuleViolation {
    pub fn new(rule_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            rule_name: rule_name.into(),
            message: message.into(),
            details: None,
        }
    }
    
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl fmt::Display for BusinessRuleViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Business rule '{}' violated: {}", self.rule_name, self.message)
    }
}

impl std::error::Error for BusinessRuleViolation {}

/// 规格模式trait
pub trait Specification<T>: Send + Sync {
    /// 检查是否满足规格
    fn is_satisfied_by(&self, candidate: &T) -> bool;
    
    /// 组合规格（AND）
    fn and<S: Specification<T>>(self, other: S) -> AndSpecification<T, Self, S>
    where
        Self: Sized + 'static,
        S: 'static,
    {
        AndSpecification {
            left: self,
            right: other,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// 组合规格（OR）
    fn or<S: Specification<T>>(self, other: S) -> OrSpecification<T, Self, S>
    where
        Self: Sized + 'static,
        S: 'static,
    {
        OrSpecification {
            left: self,
            right: other,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// 否定规格（NOT）
    fn not(self) -> NotSpecification<T, Self>
    where
        Self: Sized + 'static,
    {
        NotSpecification {
            spec: self,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// AND规格组合
pub struct AndSpecification<T, L, R> {
    left: L,
    right: R,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Send + Sync, L: Specification<T>, R: Specification<T>> Specification<T> for AndSpecification<T, L, R> {
    fn is_satisfied_by(&self, candidate: &T) -> bool {
        self.left.is_satisfied_by(candidate) && self.right.is_satisfied_by(candidate)
    }
}

/// OR规格组合
pub struct OrSpecification<T, L, R> {
    left: L,
    right: R,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Send + Sync, L: Specification<T>, R: Specification<T>> Specification<T> for OrSpecification<T, L, R> {
    fn is_satisfied_by(&self, candidate: &T) -> bool {
        self.left.is_satisfied_by(candidate) || self.right.is_satisfied_by(candidate)
    }
}

/// NOT规格组合
pub struct NotSpecification<T, S> {
    spec: S,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Send + Sync, S: Specification<T>> Specification<T> for NotSpecification<T, S> {
    fn is_satisfied_by(&self, candidate: &T) -> bool {
        !self.spec.is_satisfied_by(candidate)
    }
}

/// 审计信息trait
pub trait Auditable: Send + Sync {
    /// 获取创建时间
    fn created_at(&self) -> chrono::DateTime<chrono::Utc>;
    
    /// 获取更新时间
    fn updated_at(&self) -> chrono::DateTime<chrono::Utc>;
    
    /// 获取创建者
    fn created_by(&self) -> Option<&str>;
    
    /// 获取更新者
    fn updated_by(&self) -> Option<&str>;
    
    /// 获取版本号
    fn version(&self) -> u64;
}

/// 软删除trait
pub trait SoftDeletable: Send + Sync {
    /// 是否已删除
    fn is_deleted(&self) -> bool;
    
    /// 删除时间
    fn deleted_at(&self) -> Option<chrono::DateTime<chrono::Utc>>;
    
    /// 删除者
    fn deleted_by(&self) -> Option<&str>;
}

/// 可版本化trait
pub trait Versionable: Send + Sync {
    /// 获取当前版本
    fn current_version(&self) -> &str;
    
    /// 获取支持的版本
    fn supported_versions(&self) -> Vec<&str>;
    
    /// 检查版本兼容性
    fn is_version_compatible(&self, version: &str) -> bool;
}

/// 结果类型别名
pub type DomainResult<T> = Result<T, crate::domain::errors::DomainError>;
pub type ValidationResult<T> = Result<T, BusinessRuleViolation>;
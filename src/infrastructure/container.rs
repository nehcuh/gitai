//! 依赖注入容器实现
//! 
//! 提供类型安全的服务注册和解析功能，支持：
//! - 单例模式
//! - 瞬态模式
//! - 工厂模式
//! - 循环依赖检测

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::fmt;

/// 服务生命周期
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceLifetime {
    /// 单例 - 整个应用生命周期只有一个实例
    Singleton,
    /// 瞬态 - 每次请求都创建新实例
    Transient,
    /// 作用域 - 在特定作用域内共享实例
    Scoped,
}

/// 服务提供者trait
pub trait ServiceProvider: Send + Sync {
    type Service: Send + Sync + 'static;
    
    /// 创建服务实例
    fn create(&self, container: &ServiceContainer) -> Result<Self::Service, ContainerError>;
    
    /// 获取服务类型ID
    fn service_type_id(&self) -> TypeId {
        TypeId::of::<Self::Service>()
    }
}

/// 简单的服务工厂 - 用于不需要容器的服务创建
pub struct SimpleServiceFactory<T, F> 
where 
    T: Send + Sync + 'static,
    F: Fn() -> Result<T, ContainerError> + Send + Sync,
{
    factory: F,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, F> SimpleServiceFactory<T, F>
where 
    T: Send + Sync + 'static,
    F: Fn() -> Result<T, ContainerError> + Send + Sync,
{
    pub fn new(factory: F) -> Self {
        Self {
            factory,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, F> ServiceProvider for SimpleServiceFactory<T, F>
where 
    T: Send + Sync + 'static,
    F: Fn() -> Result<T, ContainerError> + Send + Sync,
{
    type Service = T;

    fn create(&self, _container: &ServiceContainer) -> Result<Self::Service, ContainerError> {
        (self.factory)()
    }
}

/// 为闭包实现ServiceProvider - 简化API的关键（保留用于需要容器的服务）
impl<T: Send + Sync + 'static, F> ServiceProvider for F
where
    F: Fn(&ServiceContainer) -> Result<T, ContainerError> + Send + Sync,
{
    type Service = T;

    fn create(&self, container: &ServiceContainer) -> Result<Self::Service, ContainerError> {
        self(container)
    }
}

/// 类型擦除的服务提供者
pub trait ErasedServiceProvider: Send + Sync {
    fn create_erased(&self, container: &ServiceContainer) -> Result<Box<dyn Any + Send + Sync>, ContainerError>;
    fn service_type_id(&self) -> TypeId;
}

impl<T: ServiceProvider> ErasedServiceProvider for T {
    fn create_erased(&self, container: &ServiceContainer) -> Result<Box<dyn Any + Send + Sync>, ContainerError> {
        let service = self.create(container)?;
        Ok(Box::new(service))
    }
    
    fn service_type_id(&self) -> TypeId {
        <T as ServiceProvider>::service_type_id(self)
    }
}

/// 作用域状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeState {
    /// 作用域已创建但未激活
    Created,
    /// 作用域已激活，可以使用
    Active,
    /// 作用域正在结束
    Ending,
    /// 作用域已结束，不能继续使用
    Ended,
}

/// 作用域信息
#[derive(Debug, Clone)]
pub struct ScopeInfo {
    /// 作用域ID
    pub id: uuid::Uuid,
    /// 作用域名称
    pub name: String,
    /// 作用域状态
    pub state: ScopeState,
    /// 创建时间
    pub created_at: std::time::Instant,
    /// 激活时间
    pub activated_at: Option<std::time::Instant>,
    /// 结束时间
    pub ended_at: Option<std::time::Instant>,
    /// 父作用域ID
    pub parent_id: Option<uuid::Uuid>,
    /// 子作用域数量
    pub child_count: usize,
    /// 作用域元数据
    pub metadata: std::collections::HashMap<String, String>,
}

impl ScopeInfo {
    /// 创建新的作用域信息
    pub fn new(name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name,
            state: ScopeState::Created,
            created_at: std::time::Instant::now(),
            activated_at: None,
            ended_at: None,
            parent_id: None,
            child_count: 0,
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// 获取作用域持续时间
    pub fn duration(&self) -> Option<std::time::Duration> {
        match (self.activated_at, self.ended_at) {
            (Some(start), Some(end)) => Some(end - start),
            (Some(start), None) => Some(start.elapsed()),
            _ => None,
        }
    }
    
    /// 检查作用域是否有效
    pub fn is_valid(&self) -> bool {
        matches!(self.state, ScopeState::Active)
    }
}

/// 依赖注入容器错误
#[derive(Debug)]
pub enum ContainerError {
    /// 服务未注册 - 包含服务类型信息和可用服务建议
    ServiceNotRegistered {
        type_id: TypeId,
        type_name: String,
        available_services: Vec<String>,
        suggestion: Option<String>,
    },
    /// 循环依赖检测 - 包含详细的依赖链信息
    CircularDependency {
        service_chain: Vec<String>,
        cycle_point: String,
        resolution_stack: Vec<String>,
    },
    /// 服务创建失败 - 包含服务信息和底层错误
    ServiceCreationFailed {
        service_type: String,
        service_name: Option<String>,
        reason: String,
        source_error: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    /// 类型转换失败 - 包含详细的类型信息
    TypeCastFailed {
        expected_type: String,
        actual_type: String,
        context: String,
        backtrace_hint: Option<String>,
    },
    /// 作用域错误
    ScopeError {
        scope_id: uuid::Uuid,
        scope_name: String,
        operation: String,
        reason: String,
        current_state: ScopeState,
    },
}

impl fmt::Display for ContainerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerError::ServiceNotRegistered { 
                type_name, 
                available_services, 
                suggestion, 
                .. 
            } => {
                write!(f, "Service '{}' is not registered", type_name)?;
                if !available_services.is_empty() {
                    write!(f, ". Available services: {}", available_services.join(", "))?;
                }
                if let Some(suggestion) = suggestion {
                    write!(f, ". Did you mean: {}?", suggestion)?;
                }
                Ok(())
            }
            ContainerError::CircularDependency { 
                service_chain, 
                cycle_point, 
                resolution_stack, 
            } => {
                write!(f, "Circular dependency detected")?;
                if !service_chain.is_empty() {
                    write!(f, " in chain: {}", service_chain.join(" -> "))?;
                }
                write!(f, ". Cycle at: {}. Resolution stack: {:?}", cycle_point, resolution_stack)?;
                Ok(())
            }
            ContainerError::ServiceCreationFailed { 
                service_type, 
                service_name, 
                reason, 
                source_error, 
            } => {
                write!(f, "Failed to create service '{}'", service_type)?;
                if let Some(name) = service_name {
                    write!(f, " ({})", name)?;
                }
                write!(f, ": {}", reason)?;
                if let Some(source) = source_error {
                    write!(f, ". Caused by: {}", source)?;
                }
                Ok(())
            }
            ContainerError::TypeCastFailed { 
                expected_type, 
                actual_type, 
                context, 
                backtrace_hint, 
            } => {
                write!(f, "Type cast failed: expected '{}', found '{}' in {}", 
                    expected_type, actual_type, context)?;
                if let Some(hint) = backtrace_hint {
                    write!(f, ". Hint: {}", hint)?;
                }
                Ok(())
            }
            ContainerError::ScopeError {
                scope_name,
                operation,
                reason,
                current_state,
                ..
            } => {
                write!(f, "Scope '{}' error during '{}': {} (current state: {:?})", 
                    scope_name, operation, reason, current_state)
            }
        }
    }
}

impl std::error::Error for ContainerError {}

/// 容器统计信息
#[derive(Debug, Clone)]
pub struct ContainerStats {
    /// 总解析次数
    pub total_resolutions: u64,
    /// 单例缓存命中次数
    pub singleton_cache_hits: u64,
    /// 单例缓存未命中次数
    pub singleton_cache_misses: u64,
    /// 瞬态服务创建次数
    pub transient_creations: u64,
    /// 作用域服务创建次数
    pub scoped_creations: u64,
    /// 服务注册数量
    pub registered_services: usize,
    /// 活跃单例数量
    pub active_singletons: usize,
    /// 活跃作用域实例数量
    pub active_scoped_instances: usize,
    /// 循环依赖检测次数
    pub circular_dependency_checks: u64,
    /// 类型转换失败次数
    pub type_cast_failures: u64,
}

impl Default for ContainerStats {
    fn default() -> Self {
        Self {
            total_resolutions: 0,
            singleton_cache_hits: 0,
            singleton_cache_misses: 0,
            transient_creations: 0,
            scoped_creations: 0,
            registered_services: 0,
            active_singletons: 0,
            active_scoped_instances: 0,
            circular_dependency_checks: 0,
            type_cast_failures: 0,
        }
    }
}

impl ContainerStats {
    /// 获取缓存命中率（百分比）
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.singleton_cache_hits + self.singleton_cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.singleton_cache_hits as f64 / total as f64) * 100.0
        }
    }
    
    /// 获取服务创建分布
    pub fn service_creation_distribution(&self) -> (u64, u64, u64) {
        (self.singleton_cache_misses, self.transient_creations, self.scoped_creations)
    }
    
    /// 获取性能指标摘要
    pub fn performance_summary(&self) -> String {
        format!(
            "Container Performance: {} total resolutions, {:.1}% cache hit rate, {} registered services, {} active singletons",
            self.total_resolutions,
            self.cache_hit_rate(),
            self.registered_services,
            self.active_singletons
        )
    }
    
    /// 获取总解析次数（向后兼容）
    pub fn total(&self) -> u64 {
        self.total_resolutions
    }
    
    /// 获取缓存命中率（小数形式，向后兼容）
    pub fn hit_rate(&self) -> f64 {
        let total = self.singleton_cache_hits + self.singleton_cache_misses;
        if total == 0 {
            0.0
        } else {
            self.singleton_cache_hits as f64 / total as f64
        }
    }
}

/// 服务注册信息
#[derive(Clone)]
struct ServiceRegistration {
    provider: Arc<dyn ErasedServiceProvider>,
    lifetime: ServiceLifetime,
}
/// 作用域管理器
#[derive(Clone)]
struct ScopeManager {
    /// 作用域层次结构 (作用域ID -> 作用域信息)
    scopes: Arc<RwLock<HashMap<uuid::Uuid, ScopeInfo>>>,
    /// 当前激活的作用域栈
    active_scope_stack: Arc<RwLock<Vec<uuid::Uuid>>>,
    /// 根作用域ID
    _root_scope_id: Arc<RwLock<Option<uuid::Uuid>>>,
    /// 作用域实例缓存 (作用域ID -> 服务实例)
    scope_instances: Arc<RwLock<HashMap<uuid::Uuid, Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>>>>,
}

impl ScopeManager {
    /// 创建新的作用域管理器
    fn new() -> Self {
        Self {
            scopes: Arc::new(RwLock::new(HashMap::new())),
            active_scope_stack: Arc::new(RwLock::new(Vec::new())),
            _root_scope_id: Arc::new(RwLock::new(None)),
            scope_instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 创建新的作用域
    async fn create_scope(&self, 
        name: String, 
        parent_id: Option<uuid::Uuid>
    ) -> Result<uuid::Uuid, ContainerError> {
        let mut scopes = self.scopes.write().await;
        
        // 验证父作用域
        if let Some(parent) = parent_id {
            if let Some(parent_info) = scopes.get(&parent) {
                if !parent_info.is_valid() {
                    return Err(ContainerError::ScopeError {
                        scope_id: parent,
                        scope_name: parent_info.name.clone(),
                        operation: "create_child_scope".to_string(),
                        reason: "Parent scope is not active".to_string(),
                        current_state: parent_info.state.clone(),
                    });
                }
            } else {
                return Err(ContainerError::ScopeError {
                    scope_id: parent,
                    scope_name: "unknown".to_string(),
                    operation: "create_child_scope".to_string(),
                    reason: "Parent scope not found".to_string(),
                    current_state: ScopeState::Ended,
                });
            }
        }
        
        let mut scope_info = ScopeInfo::new(name);
        scope_info.parent_id = parent_id;
        scope_info.state = ScopeState::Created;
        
        let scope_id = scope_info.id;
        scopes.insert(scope_id, scope_info.clone());
        
        // 更新父作用域的子计数
        if let Some(parent) = parent_id {
            if let Some(parent_info) = scopes.get_mut(&parent) {
                parent_info.child_count += 1;
            }
        }
        
        Ok(scope_id)
    }
    
    /// 激活作用域
    async fn activate_scope(&self, 
        scope_id: uuid::Uuid
    ) -> Result<(), ContainerError> {
        let mut scopes = self.scopes.write().await;
        let mut scope_stack = self.active_scope_stack.write().await;
        
        let scope_info = scopes.get_mut(&scope_id)
            .ok_or_else(|| ContainerError::ScopeError {
                scope_id,
                scope_name: "unknown".to_string(),
                operation: "activate_scope".to_string(),
                reason: "Scope not found".to_string(),
                current_state: ScopeState::Ended,
            })?;
        
        if scope_info.state != ScopeState::Created {
            return Err(ContainerError::ScopeError {
                scope_id,
                scope_name: scope_info.name.clone(),
                operation: "activate_scope".to_string(),
                reason: format!("Cannot activate scope in state {:?}", scope_info.state),
                current_state: scope_info.state.clone(),
            });
        }
        
        scope_info.state = ScopeState::Active;
        scope_info.activated_at = Some(std::time::Instant::now());
        scope_stack.push(scope_id);
        
        // 初始化该作用域的实例缓存
        let mut instances = self.scope_instances.write().await;
        instances.insert(scope_id, Arc::new(RwLock::new(HashMap::new())));
        
        Ok(())
    }
    
    /// 获取当前激活的作用域
    async fn current_scope(&self
    ) -> Option<(uuid::Uuid, ScopeInfo)> {
        let scope_stack = self.active_scope_stack.read().await;
        let scopes = self.scopes.read().await;
        
        scope_stack.last()
            .and_then(|scope_id| {
                scopes.get(scope_id)
                    .map(|info| (*scope_id, info.clone()))
            })
    }
    
    /// 结束作用域
    async fn end_scope(&self, 
        scope_id: uuid::Uuid
    ) -> Result<(), ContainerError> {
        let mut scopes = self.scopes.write().await;
        let mut scope_stack = self.active_scope_stack.write().await;
        let mut instances = self.scope_instances.write().await;
        
        // 先获取并校验作用域状态，同时记录父作用域ID，然后将当前作用域标记为 Ending
        let parent_id_opt = {
            let scope_info = scopes.get_mut(&scope_id)
                .ok_or_else(|| ContainerError::ScopeError {
                    scope_id,
                    scope_name: "unknown".to_string(),
                    operation: "end_scope".to_string(),
                    reason: "Scope not found".to_string(),
                    current_state: ScopeState::Ended,
                })?;
            
            if scope_info.state != ScopeState::Active {
                return Err(ContainerError::ScopeError {
                    scope_id,
                    scope_name: scope_info.name.clone(),
                    operation: "end_scope".to_string(),
                    reason: format!("Cannot end scope in state {:?}", scope_info.state),
                    current_state: scope_info.state.clone(),
                });
            }
            
            // 检查是否有子作用域
            if scope_info.child_count > 0 {
                return Err(ContainerError::ScopeError {
                    scope_id,
                    scope_name: scope_info.name.clone(),
                    operation: "end_scope".to_string(),
                    reason: format!("Cannot end scope with {} active child scopes", scope_info.child_count),
                    current_state: scope_info.state.clone(),
                });
            }
            
            // 记录父作用域ID，并先标记为 Ending
            let parent_id = scope_info.parent_id;
            scope_info.state = ScopeState::Ending;
            parent_id
        };
        
        // 清理作用域实例
        instances.remove(&scope_id);
        
        // 从激活栈中移除
        if let Some(pos) = scope_stack.iter().position(|&id| id == scope_id) {
            scope_stack.remove(pos);
        }

        // 更新父作用域的子计数（当子作用域被结束时）
        if let Some(parent_id) = parent_id_opt {
            if let Some(parent_info) = scopes.get_mut(&parent_id) {
                if parent_info.child_count > 0 {
                    parent_info.child_count -= 1;
                }
            }
        }
        
        // 最后将当前作用域状态置为 Ended，并记录结束时间
        if let Some(scope_info) = scopes.get_mut(&scope_id) {
            scope_info.state = ScopeState::Ended;
            scope_info.ended_at = Some(std::time::Instant::now());
        }
        
        Ok(())
    }
    
    /// 获取作用域实例缓存
    async fn get_scope_instances(&self, 
        scope_id: uuid::Uuid
    ) -> Option<Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>> {
        let instances = self.scope_instances.read().await;
        instances.get(&scope_id).cloned()
    }
}

/// 依赖注入容器
#[derive(Clone)]
pub struct ServiceContainer {
    /// 服务注册表
    registrations: Arc<RwLock<HashMap<TypeId, ServiceRegistration>>>,
    /// 单例实例缓存
    singletons: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
    /// 作用域管理器
    scope_manager: ScopeManager,
    /// 循环依赖检测栈
    resolution_stack: Arc<RwLock<Vec<TypeId>>>,
    /// 容器统计信息
    stats: Arc<RwLock<ContainerStats>>,
}

impl ServiceContainer {
    /// 创建新的容器实例
    pub fn new() -> Self {
        Self {
            registrations: Arc::new(RwLock::new(HashMap::new())),
            singletons: Arc::new(RwLock::new(HashMap::new())),
            scope_manager: ScopeManager::new(),
            resolution_stack: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(ContainerStats::default())),
        }
    }
    
    /// 注册服务
    pub async fn register<P>(&self, provider: P, lifetime: ServiceLifetime)
    where
        P: ServiceProvider + 'static,
    {
        let registration = ServiceRegistration {
            provider: Arc::new(provider),
            lifetime,
        };
        
        let mut registrations = self.registrations.write().await;
        registrations.insert(TypeId::of::<P::Service>(), registration);
        
        // 更新统计信息
        let mut stats = self.stats.write().await;
        stats.registered_services = registrations.len();
    }
    
    /// 注册瞬态服务
    pub async fn register_transient<P>(&self, provider: P)
    where
        P: ServiceProvider + 'static,
    {
        self.register(provider, ServiceLifetime::Transient).await;
    }
    
    /// 注册简单的瞬态服务（不需要容器） - 推荐方式
    pub async fn register_transient_simple<T, F>(&self, factory: F)
    where
        T: Send + Sync + 'static,
        F: Fn() -> Result<T, ContainerError> + Send + Sync + 'static,
    {
        let provider = SimpleServiceFactory::new(factory);
        self.register_transient(provider).await;
    }
    
    /// 注册单例服务
    pub async fn register_singleton<P>(&self, provider: P)
    where
        P: ServiceProvider + 'static,
    {
        self.register(provider, ServiceLifetime::Singleton).await;
    }
    
    /// 注册简单的单例服务（不需要容器） - 推荐方式
    pub async fn register_singleton_simple<T, F>(&self, factory: F)
    where
        T: Send + Sync + 'static,
        F: Fn() -> Result<T, ContainerError> + Send + Sync + 'static,
    {
        let provider = SimpleServiceFactory::new(factory);
        self.register_singleton(provider).await;
    }
    
    /// 注册作用域服务
    pub async fn register_scoped<P>(&self, provider: P)
    where
        P: ServiceProvider + 'static,
    {
        self.register(provider, ServiceLifetime::Scoped).await;
    }
    
    /// 注册简单的作用域服务（不需要容器） - 推荐方式
    pub async fn register_scoped_simple<T, F>(&self, factory: F)
    where
        T: Send + Sync + 'static,
        F: Fn() -> Result<T, ContainerError> + Send + Sync + 'static,
    {
        let provider = SimpleServiceFactory::new(factory);
        self.register_scoped(provider).await;
    }
    
    /// 解析服务
    pub async fn resolve<T: Send + Sync + Clone + 'static>(&self) -> Result<Arc<T>, ContainerError> {
        let type_id = TypeId::of::<T>();
        
        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.total_resolutions += 1;
            stats.circular_dependency_checks += 1;
        }
        
        // 检查循环依赖
        {
            let stack = self.resolution_stack.read().await;
            if stack.contains(&type_id) {
                let resolution_stack: Vec<String> = stack.iter()
                    .map(|id| format!("{:?}", id))
                    .collect();
                
                // 构建服务依赖链
                let service_chain = if !resolution_stack.is_empty() {
                    resolution_stack.clone()
                } else {
                    vec![format!("{:?}", type_id)]
                };
                
                let cycle_point = format!("{:?}", type_id);
                
                return Err(ContainerError::CircularDependency {
                    service_chain,
                    cycle_point,
                    resolution_stack,
                });
            }
        }
        
        // 添加到解析栈
        self.resolution_stack.write().await.push(type_id);
        
        // 执行解析
        let result = self.resolve_internal::<T>().await;
        
        // 从解析栈移除
        self.resolution_stack.write().await.pop();
        
        result
    }
    
    /// 内部解析逻辑
    async fn resolve_internal<T: Send + Sync + Clone + 'static>(&self) -> Result<Arc<T>, ContainerError> {
        let type_id = TypeId::of::<T>();
        
        // 获取服务注册信息
        let registration = {
            let registrations = self.registrations.read().await;
            registrations.get(&type_id)
                .ok_or_else(|| {
                    let type_name = std::any::type_name::<T>();
                    let available_services: Vec<String> = registrations.keys()
                        .map(|id| format!("{:?}", id))
                        .collect();
                    
                    // 简单的模糊匹配建议
                    let suggestion = if type_name.contains("Config") && available_services.iter().any(|s| s.contains("Config")) {
                        Some("Check available Config services".to_string())
                    } else if type_name.contains("Service") && available_services.iter().any(|s| s.contains("Service")) {
                        Some("Check available Service implementations".to_string())
                    } else {
                        None
                    };
                    
                    ContainerError::ServiceNotRegistered {
                        type_id,
                        type_name: type_name.to_string(),
                        available_services,
                        suggestion,
                    }
                })?
                .clone()
        };
        
        // 更新服务创建统计
        {
            let mut stats = self.stats.write().await;
            match registration.lifetime {
                ServiceLifetime::Singleton => {
                    // 将在 resolve_singleton 中更新
                }
                ServiceLifetime::Transient => {
                    stats.transient_creations += 1;
                }
                ServiceLifetime::Scoped => {
                    stats.scoped_creations += 1;
                }
            }
        }
        
        match registration.lifetime {
            ServiceLifetime::Singleton => {
                self.resolve_singleton::<T>(registration).await
            }
            ServiceLifetime::Transient => {
                self.resolve_transient::<T>(registration).await
            }
            ServiceLifetime::Scoped => {
                self.resolve_scoped::<T>(registration).await
            }
        }
    }
    
    /// 解析单例服务
    async fn resolve_singleton<T: Send + Sync + Clone + 'static>(
        &self,
        registration: ServiceRegistration,
    ) -> Result<Arc<T>, ContainerError> {
        let type_id = TypeId::of::<T>();
        
        // 检查缓存
        {
            let singletons = self.singletons.read().await;
            if let Some(cached) = singletons.get(&type_id) {
                // 缓存命中
                {
                    let mut stats = self.stats.write().await;
                    stats.singleton_cache_hits += 1;
                }
                // 将 Arc<dyn Any> 转换回 Arc<T>
                return match cached.clone().downcast::<T>() {
                    Ok(downcasted) => Ok(downcasted),
                    Err(_) => {
                        let expected_type = std::any::type_name::<T>();
                        let actual_type = "Arc<dyn Any + Send + Sync>";
                        let context = format!("singleton cache lookup for type {:?}", type_id);
                        let backtrace_hint = Some("This usually indicates a type mismatch in service registration or a bug in the container".to_string());
                        
                        Err(ContainerError::TypeCastFailed {
                            expected_type: expected_type.to_string(),
                            actual_type: actual_type.to_string(),
                            context,
                            backtrace_hint,
                        })
                    }
                };
            }
        }
        
        // 缓存未命中，需要创建新实例
        {
            let mut stats = self.stats.write().await;
            stats.singleton_cache_misses += 1;
        }
        
        // 创建新实例并直接转换为 Arc<T>
        let instance = registration.provider.create_erased(self)?;
        let service = instance.downcast::<T>()
            .map_err(|_| {
                let expected_type = std::any::type_name::<T>();
                let actual_type = "Box<dyn Any + Send + Sync>";
                let context = format!("singleton service creation");
                let backtrace_hint = Some("Check that the service provider returns the correct type".to_string());
                
                ContainerError::TypeCastFailed {
                    expected_type: expected_type.to_string(),
                    actual_type: actual_type.to_string(),
                    context,
                    backtrace_hint,
                }
            })?;
        let service_arc: Arc<T> = Arc::from(service);
        
        // 缓存实例
        {
            let mut singletons = self.singletons.write().await;
            let cached_instance: Arc<dyn Any + Send + Sync> = service_arc.clone();
            singletons.insert(type_id, cached_instance);
            
            // 更新活跃单例数量
            let mut stats = self.stats.write().await;
            stats.active_singletons = singletons.len();
        }
        
        Ok(service_arc)
    }
    
    /// 解析瞬态服务
    async fn resolve_transient<T: Send + Sync + 'static>(
        &self,
        registration: ServiceRegistration,
    ) -> Result<Arc<T>, ContainerError> {
        let instance = registration.provider.create_erased(self)?;
        let service = instance.downcast::<T>()
            .map_err(|_| {
                let expected_type = std::any::type_name::<T>();
                let actual_type = "Box<dyn Any + Send + Sync>";
                let context = format!("transient service creation");
                let backtrace_hint = Some("Check that the service provider returns the correct type".to_string());
                
                ContainerError::TypeCastFailed {
                    expected_type: expected_type.to_string(),
                    actual_type: actual_type.to_string(),
                    context,
                    backtrace_hint,
                }
            })?;
        
        Ok(Arc::from(service))
    }
    
    /// 解析作用域服务
    async fn resolve_scoped<T: Send + Sync + Clone + 'static>(
        &self,
        registration: ServiceRegistration,
    ) -> Result<Arc<T>, ContainerError> {
        let type_id = TypeId::of::<T>();
        
        // 获取当前激活的作用域
        let current_scope = self.scope_manager.current_scope().await
            .ok_or_else(|| ContainerError::ScopeError {
                scope_id: uuid::Uuid::nil(),
                scope_name: "no_active_scope".to_string(),
                operation: "resolve_scoped_service".to_string(),
                reason: "No active scope available for scoped service resolution".to_string(),
                current_state: ScopeState::Ended,
            })?;
        
        let (scope_id, scope_info) = current_scope;
        
        // 验证作用域状态
        if !scope_info.is_valid() {
            return Err(ContainerError::ScopeError {
                scope_id,
                scope_name: scope_info.name,
                operation: "resolve_scoped_service".to_string(),
                reason: format!("Cannot resolve service in scope with state {:?}", scope_info.state),
                current_state: scope_info.state,
            });
        }
        
        // 获取作用域实例缓存
        let instances = self.scope_manager.get_scope_instances(scope_id).await
            .ok_or_else(|| ContainerError::ScopeError {
                scope_id,
                scope_name: scope_info.name.clone(),
                operation: "resolve_scoped_service".to_string(),
                reason: "Scope instances not initialized".to_string(),
                current_state: scope_info.state,
            })?;
        
        // 检查当前作用域缓存
        {
            let scoped = instances.read().await;
            if let Some(cached) = scoped.get(&type_id) {
                // 将 Arc<dyn Any> 转换回 Arc<T>
                return match cached.clone().downcast::<T>() {
                    Ok(downcasted) => Ok(downcasted),
                    Err(_) => {
                        let expected_type = std::any::type_name::<T>();
                        let actual_type = "Arc<dyn Any + Send + Sync>";
                        let context = format!("scoped service cache lookup for type {:?} in scope '{}'", type_id, scope_info.name);
                        let backtrace_hint = Some("Check that the service was registered with the correct type".to_string());
                        
                        Err(ContainerError::TypeCastFailed {
                            expected_type: expected_type.to_string(),
                            actual_type: actual_type.to_string(),
                            context,
                            backtrace_hint,
                        })
                    }
                };
            }
        }
        
        // 创建新实例并直接转换为 Arc<T>
        let instance = registration.provider.create_erased(self)?;
        let service = instance.downcast::<T>()
            .map_err(|_| {
                let expected_type = std::any::type_name::<T>();
                let actual_type = "Box<dyn Any + Send + Sync>";
                let context = format!("scoped service creation in scope '{}'", scope_info.name);
                let backtrace_hint = Some("Check that the service provider returns the correct type".to_string());
                
                ContainerError::TypeCastFailed {
                    expected_type: expected_type.to_string(),
                    actual_type: actual_type.to_string(),
                    context,
                    backtrace_hint,
                }
            })?;
        let service_arc: Arc<T> = Arc::from(service);
        
        // 缓存实例到作用域
        {
            let mut scoped = instances.write().await;
            let cached_instance: Arc<dyn Any + Send + Sync> = service_arc.clone();
            scoped.insert(type_id, cached_instance);
            
            // 更新活跃作用域实例数量
            let mut stats = self.stats.write().await;
            stats.active_scoped_instances = scoped.len();
        }
        
        Ok(service_arc)
    }
    
    /// 开始新的作用域（向后兼容）
    pub async fn begin_scope(&self) {
        // 创建名为"default"的根作用域
        let _ = self.begin_scope_named("default".to_string()).await;
    }
    
    /// 结束当前作用域（向后兼容）
    pub async fn end_scope(&self) {
        // 结束当前激活的作用域
        if let Some((scope_id, _)) = self.scope_manager.current_scope().await {
            let _ = self.scope_manager.end_scope(scope_id).await;
        }
    }
    
    /// 创建新的作用域
    pub async fn create_scope(&self, name: String) -> Result<uuid::Uuid, ContainerError> {
        // 获取当前作用域作为父作用域
        let parent_id = self.scope_manager.current_scope().await.map(|(id, _)| id);
        self.scope_manager.create_scope(name, parent_id).await
    }
    
    /// 创建并激活新的作用域
    pub async fn begin_scope_named(&self, name: String) -> Result<uuid::Uuid, ContainerError> {
        let scope_id = self.create_scope(name).await?;
        self.scope_manager.activate_scope(scope_id).await?;
        Ok(scope_id)
    }
    
    /// 获取当前作用域信息
    pub async fn current_scope_info(&self) -> Option<ScopeInfo> {
        self.scope_manager.current_scope().await.map(|(_, info)| info)
    }
    
    /// 获取所有作用域信息
    pub async fn all_scopes(&self) -> Vec<ScopeInfo> {
        let scopes = self.scope_manager.scopes.read().await;
        scopes.values().cloned().collect()
    }
    
    /// 结束指定作用域
    pub async fn end_scope_named(&self, name: &str) -> Result<(), ContainerError> {
        let scopes = self.scope_manager.scopes.read().await;
        
        // 找到指定名称的作用域
        let scope_id = scopes.values()
            .find(|info| info.name == name && info.is_valid())
            .map(|info| info.id)
            .ok_or_else(|| ContainerError::ScopeError {
                scope_id: uuid::Uuid::nil(),
                scope_name: name.to_string(),
                operation: "end_scope_named".to_string(),
                reason: "Active scope not found".to_string(),
                current_state: ScopeState::Ended,
            })?;
        
        drop(scopes);
        self.scope_manager.end_scope(scope_id).await
    }
    
    /// 检查服务是否已注册
    pub async fn is_registered<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        let registrations = self.registrations.read().await;
        registrations.contains_key(&type_id)
    }
    
    /// 获取已注册的服务类型列表
    pub async fn get_registered_types(&self) -> Vec<TypeId> {
        let registrations = self.registrations.read().await;
        registrations.keys().cloned().collect()
    }
    
    /// 获取容器统计信息
    pub async fn get_stats(&self) -> ContainerStats {
        let stats = self.stats.read().await;
        stats.clone()
    }
    
    /// 获取缓存命中率（向后兼容）
    pub async fn get_cache_hit_rate(&self) -> f64 {
        let stats = self.stats.read().await;
        stats.hit_rate()
    }
    
    /// 获取容器性能摘要
    pub async fn get_performance_summary(&self) -> String {
        let stats = self.stats.read().await;
        stats.performance_summary()
    }
    
    /// 重置统计信息
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = ContainerStats::default();
    }
    
    // ===== 向后兼容API =====
    
    /// 向后兼容：注册服务（默认单例）
    #[deprecated(since = "1.1.0", note = "请使用 register_singleton 或 register_transient 明确指定生命周期")]
    pub async fn register_service<T, F>(&self, factory: F) 
    where
        T: Send + Sync + Clone + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let provider = SimpleServiceFactory::new(move || Ok(factory()));
        self.register_singleton(provider).await;
    }
    
    /// 向后兼容：获取服务实例（非Arc版本）
    #[deprecated(since = "1.1.0", note = "请使用 resolve 方法返回的 Arc 版本")]
    pub async fn get_service<T: Send + Sync + Clone + 'static>(&self
    ) -> Result<T, ContainerError> {
        let arc_service = self.resolve::<T>().await?;
        Ok((*arc_service).clone())
    }
    
    /// 向后兼容：检查容器是否包含服务
    #[deprecated(since = "1.1.0", note = "请使用 is_registered 方法")]
    pub async fn contains_service<T: 'static>(&self
    ) -> bool {
        self.is_registered::<T>().await
    }
    
    /// 向后兼容：获取所有注册的服务类型名称
    #[deprecated(since = "1.1.0", note = "请使用 get_registered_types 方法")]
    pub async fn get_service_types(&self
    ) -> Vec<String> {
        let types = self.get_registered_types().await;
        types.iter()
            .map(|type_id| format!("{:?}", type_id))
            .collect()
    }
    
    /// 向后兼容：获取容器状态摘要
    #[deprecated(since = "1.1.0", note = "请使用 get_stats 方法获取详细信息")]
    pub async fn get_container_status(&self
    ) -> String {
        self.get_performance_summary().await
    }
    
    /// 向后兼容：创建瞬态服务（旧API）
    #[deprecated(since = "1.1.0", note = "请使用 register_transient_simple 方法")]
    pub async fn register_transient_service<T, F>(&self, factory: F
    ) where
        T: Send + Sync + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.register_transient_simple(move || Ok(factory())).await;
    }
    
    /// 向后兼容：创建单例服务（旧API）
    #[deprecated(since = "1.1.0", note = "请使用 register_singleton_simple 方法")]
    pub async fn register_singleton_service<T, F>(&self, factory: F
    ) where
        T: Send + Sync + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.register_singleton_simple(move || Ok(factory())).await;
    }
    
    /// 向后兼容：创建作用域服务（旧API）
    #[deprecated(since = "1.1.0", note = "请使用 register_scoped_simple 方法")]
    pub async fn register_scoped_service<T, F>(&self, factory: F
    ) where
        T: Send + Sync + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.register_scoped_simple(move || Ok(factory())).await;
    }
    
    /// 向后兼容：解析服务（旧API）
    #[deprecated(since = "1.1.0", note = "请使用 resolve 方法")]
    pub async fn resolve_service<T: Send + Sync + Clone + 'static>(
        &self
    ) -> Result<T, ContainerError> {
        let arc_result = self.resolve::<T>().await?;
        Ok((*arc_result).clone())
    }
    
    /// 向后兼容：获取缓存命中率（百分比格式）
    #[deprecated(since = "1.1.0", note = "请使用 get_stats().cache_hit_rate() 方法")]
    pub async fn get_cache_hit_percentage(&self
    ) -> f64 {
        let stats = self.get_stats().await;
        stats.cache_hit_rate()
    }
}

impl Default for ServiceContainer {
    fn default() -> Self {
        Self::new()
    }
}

/// 便捷的服务注册宏
#[macro_export]
macro_rules! register_service {
    ($container:expr, $provider:expr, singleton) => {
        $container.register_singleton($provider).await;
    };
    ($container:expr, $provider:expr, transient) => {
        $container.register_transient($provider).await;
    };
    ($container:expr, $provider:expr, scoped) => {
        $container.register_scoped($provider).await;
    };
}

/// 服务解析便捷宏
#[macro_export]
macro_rules! resolve_service {
    ($container:expr, $type:ty) => {
        $container.resolve::<$type>().await
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    
    // 测试用的服务trait
    #[async_trait]
    trait TestService: Send + Sync {
        async fn get_value(&self) -> String;
    }
    
    #[derive(Clone)]
    struct TestServiceImpl {
        value: String,
    }
    
    #[async_trait]
    impl TestService for TestServiceImpl {
        async fn get_value(&self) -> String {
            self.value.clone()
        }
    }
    
    struct TestServiceProvider {
        value: String,
    }
    
    #[async_trait]
    impl ServiceProvider for TestServiceProvider {
        type Service = TestServiceImpl;
        
        fn create(&self, _container: &ServiceContainer) -> Result<Self::Service, ContainerError> {
            Ok(TestServiceImpl {
                value: self.value.clone(),
            })
        }
    }
    
    #[tokio::test]
    async fn test_transient_service() {
        let container = ServiceContainer::new();
        let provider = TestServiceProvider {
            value: "test".to_string(),
        };
        
        container.register_transient(provider).await;
        
        let service1 = container.resolve::<TestServiceImpl>().await.unwrap();
        let service2 = container.resolve::<TestServiceImpl>().await.unwrap();
        
        // 瞬态服务应该返回不同实例
        assert_ne!(Arc::as_ptr(&service1), Arc::as_ptr(&service2));
    }
    
    #[tokio::test]
    async fn test_singleton_service() {
        let container = ServiceContainer::new();
        let provider = TestServiceProvider {
            value: "singleton".to_string(),
        };
        
        container.register_singleton(provider).await;
        
        let service1 = container.resolve::<TestServiceImpl>().await.unwrap();
        let service2 = container.resolve::<TestServiceImpl>().await.unwrap();
        
        // 单例服务应该返回相同实例
        assert_eq!(Arc::as_ptr(&service1), Arc::as_ptr(&service2));
    }
    
    #[tokio::test]
    async fn test_service_not_registered() {
        let container = ServiceContainer::new();
        
        let result = container.resolve::<TestServiceImpl>().await;
        
        assert!(matches!(result, Err(ContainerError::ServiceNotRegistered { .. })));
    }
    
    #[tokio::test]
    async fn test_container_stats() {
        let container = ServiceContainer::new();
        
        // 初始统计应该为空
        let initial_stats = container.get_stats().await;
        assert_eq!(initial_stats.total_resolutions, 0);
        assert_eq!(initial_stats.registered_services, 0);
        assert_eq!(initial_stats.active_singletons, 0);
        assert_eq!(initial_stats.singleton_cache_hits, 0);
        assert_eq!(initial_stats.singleton_cache_misses, 0);
        
        // 注册服务
        let provider = TestServiceProvider {
            value: "stats_test".to_string(),
        };
        container.register_singleton(provider).await;
        
        // 检查注册后的统计
        let after_register_stats = container.get_stats().await;
        assert_eq!(after_register_stats.registered_services, 1);
        
        // 解析服务（首次，应该缓存未命中）
        let service1 = container.resolve::<TestServiceImpl>().await.unwrap();
        let after_first_resolve = container.get_stats().await;
        assert_eq!(after_first_resolve.total_resolutions, 1);
        assert_eq!(after_first_resolve.singleton_cache_misses, 1);
        assert_eq!(after_first_resolve.singleton_cache_hits, 0);
        assert_eq!(after_first_resolve.active_singletons, 1);
        
        // 再次解析服务（应该缓存命中）
        let service2 = container.resolve::<TestServiceImpl>().await.unwrap();
        let after_second_resolve = container.get_stats().await;
        assert_eq!(after_second_resolve.total_resolutions, 2);
        assert_eq!(after_second_resolve.singleton_cache_misses, 1); // 未增加
        assert_eq!(after_second_resolve.singleton_cache_hits, 1);   // 增加1
        assert_eq!(after_second_resolve.active_singletons, 1);      // 仍然是1个单例
        
        // 验证缓存命中率
        assert_eq!(after_second_resolve.cache_hit_rate(), 50.0);
        
        // 验证服务是同一个实例
        assert_eq!(Arc::as_ptr(&service1), Arc::as_ptr(&service2));
        
        // 测试性能摘要
        let summary = container.get_performance_summary().await;
        assert!(summary.contains("2 total resolutions"));
        assert!(summary.contains("50.0% cache hit rate"));
        assert!(summary.contains("1 registered services"));
        assert!(summary.contains("1 active singletons"));
    }
    
    #[tokio::test]
    async fn test_container_stats_reset() {
        let container = ServiceContainer::new();
        
        // 注册并解析服务
        let provider = TestServiceProvider {
            value: "reset_test".to_string(),
        };
        container.register_singleton(provider).await;
        container.resolve::<TestServiceImpl>().await.unwrap();
        
        // 验证有统计信息
        let stats_before = container.get_stats().await;
        assert_eq!(stats_before.total_resolutions, 1);
        assert_eq!(stats_before.registered_services, 1);
        
        // 重置统计
        container.reset_stats().await;
        
        // 验证统计被重置
        let stats_after = container.get_stats().await;
        assert_eq!(stats_after.total_resolutions, 0);
        assert_eq!(stats_after.registered_services, 0); // 注册服务数量也会被重置
        assert_eq!(stats_after.active_singletons, 0);
    }
    
    #[tokio::test]
    async fn test_enhanced_scope_functionality() {
        let container = ServiceContainer::new();
        
        // 注册一个作用域服务
        let provider = TestServiceProvider {
            value: "scoped_test".to_string(),
        };
        container.register_scoped(provider).await;
        
        // 创建并激活作用域
        let scope_id = container.begin_scope_named("test_scope".to_string()).await.unwrap();
        
        // 验证当前作用域信息
        let current_scope = container.current_scope_info().await;
        assert!(current_scope.is_some());
        let scope_info = current_scope.unwrap();
        assert_eq!(scope_info.name, "test_scope");
        assert!(scope_info.is_valid());
        
        // 在作用域中解析服务
        let service1 = container.resolve::<TestServiceImpl>().await.unwrap();
        let service2 = container.resolve::<TestServiceImpl>().await.unwrap();
        
        // 同一个作用域中应该返回相同实例
        assert_eq!(Arc::as_ptr(&service1), Arc::as_ptr(&service2));
        
        // 结束当前作用域
        container.scope_manager.end_scope(scope_id).await.unwrap();
        
        // 验证作用域已结束
        let current_scope_after = container.current_scope_info().await;
        assert!(current_scope_after.is_none());
        
        // 尝试在没有作用域的情况下解析作用域服务应该失败
        let result = container.resolve::<TestServiceImpl>().await;
        assert!(result.is_err());
        match result {
            Err(ContainerError::ScopeError { .. }) => {},
            _ => panic!("Expected ScopeError when resolving scoped service without active scope"),
        }
    }
    
    #[tokio::test]
    async fn test_nested_scopes() {
        let container = ServiceContainer::new();
        
        // 注册一个作用域服务
        let provider = TestServiceProvider {
            value: "nested_test".to_string(),
        };
        container.register_scoped(provider).await;
        
        // 创建父作用域
        let parent_scope_id = container.begin_scope_named("parent_scope".to_string()).await.unwrap();
        
        // 在父作用域中解析服务
        let parent_service = container.resolve::<TestServiceImpl>().await.unwrap();
        
        // 创建子作用域
        let child_scope_id = container.begin_scope_named("child_scope".to_string()).await.unwrap();
        
        // 验证当前作用域是子作用域
        let current_scope = container.current_scope_info().await.unwrap();
        assert_eq!(current_scope.name, "child_scope");
        assert_eq!(current_scope.parent_id, Some(parent_scope_id));
        
        // 在子作用域中解析服务（应该与父作用域不同）
        let child_service = container.resolve::<TestServiceImpl>().await.unwrap();
        
        // 子作用域和父作用域应该有不同的实例
        assert_ne!(Arc::as_ptr(&parent_service), Arc::as_ptr(&child_service));
        
        // 结束子作用域
        container.scope_manager.end_scope(child_scope_id).await.unwrap();
        
        // 验证当前作用域回到父作用域
        let current_scope_after = container.current_scope_info().await.unwrap();
        assert_eq!(current_scope_after.name, "parent_scope");
        
        // 在父作用域中再次解析服务（应该与之前相同）
        let parent_service_again = container.resolve::<TestServiceImpl>().await.unwrap();
        assert_eq!(Arc::as_ptr(&parent_service), Arc::as_ptr(&parent_service_again));
        
        // 结束父作用域
        container.scope_manager.end_scope(parent_scope_id).await.unwrap();
        
        // 验证没有活跃作用域
        assert!(container.current_scope_info().await.is_none());
    }
    
    #[tokio::test]
    async fn test_scope_error_handling() {
        let container = ServiceContainer::new();
        
        // 测试结束不存在的作用域
        let result = container.scope_manager.end_scope(uuid::Uuid::new_v4()).await;
        assert!(result.is_err());
        match result {
            Err(ContainerError::ScopeError { reason, .. }) => {
                assert!(reason.contains("Scope not found"));
            }
            _ => panic!("Expected ScopeError for non-existent scope"),
        }
        
        // 测试在已结束的作用域中创建子作用域
        let scope_id = container.begin_scope_named("temp_scope".to_string()).await.unwrap();
        container.scope_manager.end_scope(scope_id).await.unwrap();
        
        let result = container.scope_manager.create_scope("child".to_string(), Some(scope_id)).await;
        assert!(result.is_err());
        match result {
            Err(ContainerError::ScopeError { reason, .. }) => {
                assert!(reason.contains("Parent scope is not active"));
            }
            _ => panic!("Expected ScopeError for inactive parent scope"),
        }
        
        // 测试在有子作用域的情况下结束父作用域
        let parent_id = container.begin_scope_named("parent".to_string()).await.unwrap();
        let child_id = container.begin_scope_named("child".to_string()).await.unwrap();
        
        let result = container.scope_manager.end_scope(parent_id).await;
        assert!(result.is_err());
        match result {
            Err(ContainerError::ScopeError { reason, .. }) => {
                assert!(reason.contains("Cannot end scope with"));
                assert!(reason.contains("active child scopes"));
            }
            _ => panic!("Expected ScopeError for scope with active children"),
        }
        
        // 清理：结束子作用域，然后结束父作用域
        let _ = container.scope_manager.end_scope(child_id).await;
        let _ = container.scope_manager.end_scope(parent_id).await;
    }
}
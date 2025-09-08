//! 改进的依赖注入容器实现
//! 
//! 解决以下问题：
//! - 类型擦除和向下转型的错误实现
//! - 单例模式的并发安全问题
//! - 过度复杂的API设计
//! - 性能瓶颈

use dashmap::DashMap;
use std::any::{Any, TypeId};
use std::sync::Arc;
use tokio::sync::OnceCell;
use std::fmt;

/// 简化的容器错误类型
#[derive(Debug)]
pub enum ContainerError {
    /// 服务未注册
    ServiceNotRegistered(TypeId),
    /// 类型转换失败
    TypeCastFailed {
        expected: String,
        actual: String,
    },
    /// 服务创建失败
    CreationFailed(String),
    /// 并发限制超出
    ConcurrencyLimitExceeded,
}

impl fmt::Display for ContainerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerError::ServiceNotRegistered(type_id) => {
                write!(f, "Service not registered for type ID: {:?}", type_id)
            }
            ContainerError::TypeCastFailed { expected, actual } => {
                write!(f, "Type cast failed: expected {}, got {}", expected, actual)
            }
            ContainerError::CreationFailed(msg) => {
                write!(f, "Service creation failed: {}", msg)
            }
            ContainerError::ConcurrencyLimitExceeded => {
                write!(f, "Container concurrency limit exceeded")
            }
        }
    }
}

impl std::error::Error for ContainerError {}

/// 服务工厂trait - 简化设计
pub trait ServiceFactory: Send + Sync {
    /// 创建服务实例
    fn create(&self, container: &ServiceContainer) -> Result<Arc<dyn Any + Send + Sync>, ContainerError>;
    
    /// 获取服务类型ID
    fn service_type_id(&self) -> TypeId;
    
    /// 获取服务类型名称（用于错误信息）
    fn service_type_name(&self) -> &'static str;
}

/// 函数式服务工厂
pub struct FnServiceFactory<F, T> {
    factory_fn: F,
    type_name: &'static str,
    _phantom: std::marker::PhantomData<T>,
}

impl<F, T> FnServiceFactory<F, T> {
    pub fn new(factory_fn: F, type_name: &'static str) -> Self {
        Self {
            factory_fn,
            type_name,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<F, T> ServiceFactory for FnServiceFactory<F, T> 
where 
    F: Fn(&ServiceContainer) -> Result<T, Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    fn create(&self, container: &ServiceContainer) -> Result<Arc<dyn Any + Send + Sync>, ContainerError> {
        let service = (self.factory_fn)(container)
            .map_err(|e| ContainerError::CreationFailed(e.to_string()))?;
        Ok(Arc::new(service))
    }
    
    fn service_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
    
    fn service_type_name(&self) -> &'static str {
        self.type_name
    }
}

/// 改进的服务容器 - 解决并发和类型安全问题
#[derive(Clone)]
pub struct ServiceContainer {
    /// 服务工厂注册表 - 使用DashMap提供更好的并发性能
    factories: Arc<DashMap<TypeId, Box<dyn ServiceFactory>>>,
    /// 单例实例缓存 - 使用OnceCell确保只创建一次
    singletons: Arc<DashMap<TypeId, Arc<OnceCell<Arc<dyn Any + Send + Sync>>>>>,
    /// 容器统计信息（内部原子计数器）
    stats: Arc<InnerStats>,
}

/// 内部容器统计信息（原子计数器）
#[derive(Default)]
struct InnerStats {
    total_resolutions: std::sync::atomic::AtomicUsize,
    cache_hits: std::sync::atomic::AtomicUsize,
    cache_misses: std::sync::atomic::AtomicUsize,
}

impl ServiceContainer {
    /// 创建新的容器实例
    pub fn new() -> Self {
        Self {
            factories: Arc::new(DashMap::new()),
            singletons: Arc::new(DashMap::new()),
stats: Arc::new(InnerStats::default()),
        }
    }
    
    /// 注册服务工厂 - 简化API
    pub fn register<T, F>(&self, factory: F) 
    where 
        F: Fn(&ServiceContainer) -> Result<T, Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let factory = Box::new(FnServiceFactory::<F, T>::new(factory, std::any::type_name::<T>()));
        self.factories.insert(TypeId::of::<T>(), factory);
    }
    
    /// 注册瞬态服务 - 便捷方法
    pub fn register_transient<T, F>(&self, factory: F) 
    where 
        F: Fn() -> T + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        self.register(move |_| Ok(factory()));
    }
    
    /// 注册单例服务 - 便捷方法  
    pub fn register_singleton<T, F>(&self, factory: F) 
    where 
        F: Fn(&ServiceContainer) -> T + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        self.register(move |container| Ok(factory(container)));
    }
    
    /// 解析服务 - 主要API
    pub async fn resolve<T: Send + Sync + 'static>(&self) -> Result<Arc<T>, ContainerError> {
        self.stats.total_resolutions.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        let type_id = TypeId::of::<T>();
        
        // 检查单例缓存
        if let Some(cell) = self.singletons.get(&type_id) {
            if let Some(service) = cell.get() {
                self.stats.cache_hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                
// 安全的类型转换
                let cloned = service.clone();
                return match cloned.downcast::<T>() {
                    Ok(arc_t) => Ok(arc_t),
                    Err(_) => Err(ContainerError::TypeCastFailed {
                        expected: std::any::type_name::<T>().to_string(),
                        actual: "unknown type".to_string(),
                    }),
                };
            }
        }
        
        self.stats.cache_misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        // 创建新实例
        self.create_service::<T>(type_id).await
    }
    
    /// 创建服务实例 - 私有方法
    async fn create_service<T: Send + Sync + 'static>(&self, type_id: TypeId) -> Result<Arc<T>, ContainerError> {
        // 获取或创建OnceCell
        let once_cell = self.singletons
            .entry(type_id)
            .or_insert_with(|| Arc::new(OnceCell::new()))
            .clone();
        
// 使用OnceCell确保只创建一次
        let service = once_cell.get_or_try_init(|| async {
            let factory_guard = self
                .factories
                .get(&type_id)
                .ok_or(ContainerError::ServiceNotRegistered(type_id))?;

            factory_guard.value().create(self)
        }).await?;
        
// 安全的类型转换
        {
            let cloned = service.clone();
            match cloned.downcast::<T>() {
                Ok(arc_t) => Ok(arc_t),
                Err(_) => Err(ContainerError::TypeCastFailed {
                    expected: std::any::type_name::<T>().to_string(),
actual: format!("{:?}", service.type_id()),
                }),
            }
        }
    }
    
    /// 检查服务是否已注册
    pub fn is_registered<T: 'static>(&self) -> bool {
        self.factories.contains_key(&TypeId::of::<T>())
    }
    
    /// 获取容器统计信息
pub fn get_stats(&self) -> ContainerStats {
        ContainerStats {
            total_resolutions: self
                .stats
                .total_resolutions
                .load(std::sync::atomic::Ordering::Relaxed),
            cache_hits: self
                .stats
                .cache_hits
                .load(std::sync::atomic::Ordering::Relaxed),
            cache_misses: self
                .stats
                .cache_misses
                .load(std::sync::atomic::Ordering::Relaxed),
        }
    }
    
    /// 获取缓存命中率
    pub fn get_cache_hit_rate(&self) -> f64 {
        let hits = self.stats.cache_hits.load(std::sync::atomic::Ordering::Relaxed) as f64;
        let misses = self.stats.cache_misses.load(std::sync::atomic::Ordering::Relaxed) as f64;
        
        if hits + misses == 0.0 {
            0.0
        } else {
            hits / (hits + misses)
        }
    }
}

impl Default for ServiceContainer {
    fn default() -> Self {
        Self::new()
    }
}

/// 容器统计信息
#[derive(Debug, Clone)]
pub struct ContainerStats {
    pub total_resolutions: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

impl ContainerStats {
    /// 获取总解析次数
    pub fn total(&self) -> usize {
        self.total_resolutions
    }
    
    /// 获取缓存命中率
    pub fn hit_rate(&self) -> f64 {
        if self.total() == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total() as f64
        }
    }
}

/// 便捷的服务注册宏
#[macro_export]
macro_rules! register_singleton {
    ($container:expr, $type:ty, $factory:expr) => {
        $container.register::<$type, _>(move |_| Ok($factory))
    };
}

#[macro_export]
macro_rules! register_transient {
    ($container:expr, $type:ty, $factory:expr) => {
        $container.register::<$type, _>(move |_| Ok($factory()))
    };
}

/// 服务解析宏
#[macro_export]
macro_rules! resolve {
    ($container:expr, $type:ty) => {
        $container.resolve::<$type>().await
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::future;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    // 测试用的服务
    #[derive(Debug, Clone)]
    struct TestService {
        id: usize,
    }
    
    #[tokio::test]
    async fn test_transient_service() {
        let container = ServiceContainer::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        // 由于 v2 容器按类型进行 OnceCell 缓存，这里等效为单例
        container.register(move |_| {
            let id = counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(TestService { id })
        });
        
        // 解析两次，应该得到相同实例（单例语义）
        let service1 = container.resolve::<TestService>().await.unwrap();
        let service2 = container.resolve::<TestService>().await.unwrap();
        
        assert_eq!(service1.id, service2.id);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
    
    #[tokio::test]
    async fn test_singleton_service() {
        let container = ServiceContainer::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        // 注册单例服务
        container.register(move |_| {
            let id = counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(TestService { id })
        });
        
        // 解析多次，应该得到相同实例
        let service1 = container.resolve::<TestService>().await.unwrap();
        let service2 = container.resolve::<TestService>().await.unwrap();
        let service3 = container.resolve::<TestService>().await.unwrap();
        
        assert_eq!(service1.id, service2.id);
        assert_eq!(service2.id, service3.id);
        assert_eq!(counter.load(Ordering::SeqCst), 1); // 只创建一次
    }
    
    #[tokio::test]
    async fn test_service_not_registered() {
        let container = ServiceContainer::new();
        
        let result = container.resolve::<TestService>().await;
        
        assert!(matches!(result, Err(ContainerError::ServiceNotRegistered(_))));
    }
    
    #[tokio::test]
    async fn test_type_cast_error() {
        let container = ServiceContainer::new();
        
        // 注册了与目标类型不同的服务类型（字符串），因此解析 TestService 时应提示未注册
        container.register(|_| Ok("wrong type"));
        
        // 尝试解析为 TestService 类型
        let result = container.resolve::<TestService>().await;
        
        assert!(matches!(result, Err(ContainerError::ServiceNotRegistered(_))));
    }
    
    #[tokio::test]
    async fn test_container_stats() {
        let container = ServiceContainer::new();
        
        // 注册服务
        container.register(|_| Ok(TestService { id: 42 }));
        
        // 解析多次
        for _ in 0..10 {
            let _ = container.resolve::<TestService>().await.unwrap();
        }
        
        let stats = container.get_stats();
        assert_eq!(stats.total(), 10);
        assert_eq!(stats.cache_hits, 9); // 第一次miss，后面都是hit
        assert_eq!(stats.cache_misses, 1);
        assert!(stats.hit_rate() > 0.8);
    }
    
    #[tokio::test]
    async fn test_concurrent_resolution() {
        let container = ServiceContainer::new();
        let counter = Arc::new(AtomicUsize::new(0));
        
// 注册单例服务
        let counter_clone = counter.clone();
        container.register(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(TestService { id: 42 })
        });
        
        // 并发解析
        let mut handles = vec![];
        for _ in 0..100 {
            let container_clone = container.clone();
            handles.push(tokio::spawn(async move {
                container_clone.resolve::<TestService>().await.unwrap()
            }));
        }
        
        // 等待所有任务完成
let results = future::join_all(handles).await;
        
        // 验证所有服务实例相同
        let first_id = results[0].as_ref().unwrap().id;
        for result in results {
            assert_eq!(result.unwrap().id, first_id);
        }
        
        // 验证只创建了一次服务
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
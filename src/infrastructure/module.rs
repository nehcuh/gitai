//! 模块抽象层
//! 
//! 定义模块的初始化、配置和依赖关系管理

use super::container::{ServiceContainer, ContainerError};
use crate::domain::interfaces::*;
use std::sync::Arc;
use async_trait::async_trait;

/// 模块状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleStatus {
    /// 未初始化
    NotInitialized,
    /// 初始化中
    Initializing,
    /// 已初始化
    Initialized,
    /// 初始化失败
    Failed,
    /// 已卸载
    Unloaded,
}

/// 模块接口
#[async_trait]
pub trait Module: Send + Sync {
    /// 模块名称
    fn name(&self) -> &str;
    
    /// 模块版本
    fn version(&self) -> &str;
    
    /// 依赖的模块
    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
    }
    
    /// 初始化模块
    async fn initialize(&self, container: &ServiceContainer) -> Result<(), ModuleError>;
    
    /// 卸载模块
    async fn unload(&self, container: &ServiceContainer) -> Result<(), ModuleError>;
    
    /// 检查模块是否可用
    fn is_enabled(&self) -> bool {
        true
    }
}

/// 模块错误
#[derive(Debug)]
pub enum ModuleError {
    /// 依赖模块未找到
    DependencyNotFound(String),
    /// 依赖模块未初始化
    DependencyNotInitialized(String),
    /// 模块初始化失败
    InitializationFailed(String),
    /// 模块卸载失败
    UnloadFailed(String),
    /// 循环依赖
    CircularDependency(Vec<String>),
    /// 容器错误
    ContainerError(ContainerError),
}

impl std::fmt::Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleError::DependencyNotFound(name) => {
                write!(f, "Dependency module not found: {}", name)
            }
            ModuleError::DependencyNotInitialized(name) => {
                write!(f, "Dependency module not initialized: {}", name)
            }
            ModuleError::InitializationFailed(msg) => {
                write!(f, "Module initialization failed: {}", msg)
            }
            ModuleError::UnloadFailed(msg) => {
                write!(f, "Module unload failed: {}", msg)
            }
            ModuleError::CircularDependency(modules) => {
                write!(f, "Circular dependency detected: {:?}", modules)
            }
            ModuleError::ContainerError(err) => {
                write!(f, "Container error: {}", err)
            }
        }
    }
}

impl std::error::Error for ModuleError {}

impl From<ContainerError> for ModuleError {
    fn from(err: ContainerError) -> Self {
        ModuleError::ContainerError(err)
    }
}

/// 模块管理器
pub struct ModuleManager {
    modules: Arc<tokio::sync::RwLock<Vec<Arc<dyn Module>>>>,
    status: Arc<tokio::sync::RwLock<std::collections::HashMap<String, ModuleStatus>>,
    container: Arc<ServiceContainer>,
}

impl ModuleManager {
    /// 创建新的模块管理器
    pub fn new(container: Arc<ServiceContainer>) -> Self {
        Self {
            modules: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            status: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            container,
        }
    }
    
    /// 注册模块
    pub async fn register_module(&self, module: Arc<dyn Module>) -> Result<(), ModuleError> {
        let mut modules = self.modules.write().await;
        let mut status = self.status.write().await;
        
        let module_name = module.name().to_string();
        
        // 检查是否已注册
        if modules.iter().any(|m| m.name() == module_name) {
            return Ok(());
        }
        
        modules.push(module);
        status.insert(module_name, ModuleStatus::NotInitialized);
        
        Ok(())
    }
    
    /// 初始化所有模块
    pub async fn initialize_all(&self) -> Result<(), ModuleError> {
        let modules = self.modules.read().await;
        let mut initialized_modules = std::collections::HashSet::new();
        
        // 检查循环依赖
        for module in modules.iter() {
            self.check_circular_dependency(module, &mut Vec::new())?;
        }
        
        // 按依赖顺序初始化
        while initialized_modules.len() < modules.len() {
            let mut progress = false;
            
            for module in modules.iter() {
                let module_name = module.name();
                
                if initialized_modules.contains(module_name) {
                    continue;
                }
                
                // 检查依赖是否都已初始化
                let dependencies = module.dependencies();
                let all_deps_initialized = dependencies.iter()
                    .all(|dep| initialized_modules.contains(dep));
                
                if all_deps_initialized {
                    self.initialize_module(module).await?;
                    initialized_modules.insert(module_name.to_string());
                    progress = true;
                }
            }
            
            if !progress {
                return Err(ModuleError::InitializationFailed(
                    "Unable to resolve module initialization order".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// 初始化特定模块
    async fn initialize_module(&self, module: &Arc<dyn Module>) -> Result<(), ModuleError> {
        let module_name = module.name().to_string();
        
        // 检查模块是否启用
        if !module.is_enabled() {
            return Ok(());
        }
        
        // 更新状态
        {
            let mut status = self.status.write().await;
            status.insert(module_name.clone(), ModuleStatus::Initializing);
        }
        
        // 执行初始化
        let result = module.initialize(&self.container).await;
        
        // 更新状态
        {
            let mut status = self.status.write().await;
            match &result {
                Ok(_) => {
                    status.insert(module_name.clone(), ModuleStatus::Initialized);
                }
                Err(_) => {
                    status.insert(module_name.clone(), ModuleStatus::Failed);
                }
            }
        }
        
        result
    }
    
    /// 卸载所有模块
    pub async fn unload_all(&self) -> Result<(), ModuleError> {
        let modules = self.modules.read().await;
        
        // 按依赖关系的逆序卸载
        for module in modules.iter().rev() {
            self.unload_module(module).await?;
        }
        
        Ok(())
    }
    
    /// 卸载特定模块
    async fn unload_module(&self, module: &Arc<dyn Module>) -> Result<(), ModuleError> {
        let module_name = module.name().to_string();
        
        // 更新状态
        {
            let mut status = self.status.write().await;
            status.insert(module_name.clone(), ModuleStatus::Unloaded);
        }
        
        module.unload(&self.container).await
    }
    
    /// 检查循环依赖
    fn check_circular_dependency(
        &self,
        module: &Arc<dyn Module>,
        visited: &mut Vec<String>,
    ) -> Result<(), ModuleError> {
        let module_name = module.name().to_string();
        
        if visited.contains(&module_name) {
            visited.push(module_name);
            return Err(ModuleError::CircularDependency(visited.clone()));
        }
        
        visited.push(module_name.clone());
        
        for dep in module.dependencies() {
            // 这里需要获取依赖模块，简化实现
            // 实际应该检查依赖模块的依赖关系
        }
        
        visited.pop();
        Ok(())
    }
    
    /// 获取模块状态
    pub async fn get_module_status(&self, module_name: &str) -> Option<ModuleStatus> {
        let status = self.status.read().await;
        status.get(module_name).copied()
    }
    
    /// 获取所有模块状态
    pub async fn get_all_module_statuses(&self) -> std::collections::HashMap<String, ModuleStatus> {
        let status = self.status.read().await;
        status.clone()
    }
    
    /// 获取已初始化的模块
    pub async fn get_initialized_modules(&self) -> Vec<String> {
        let status = self.status.read().await;
        status.iter()
            .filter(|(_, s)| **s == ModuleStatus::Initialized)
            .map(|(name, _)| name.clone())
            .collect()
    }
}

/// 核心模块
pub struct CoreModule {
    config: Arc<gitai_core::config::Config>,
}

impl CoreModule {
    pub fn new(config: Arc<gitai_core::config::Config>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Module for CoreModule {
    fn name(&self) -> &str {
        "core"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    async fn initialize(&self, container: &ServiceContainer) -> Result<(), ModuleError> {
        // 注册核心服务
        let provider_factory = ProviderFactory::new(self.config.clone());
        provider_factory.register_default_providers(container).await?;
        
        Ok(())
    }
    
    async fn unload(&self, _container: &ServiceContainer) -> Result<(), ModuleError> {
        // 清理核心资源
        Ok(())
    }
}

/// 评审模块
pub struct ReviewModule {
    enabled: bool,
}

impl ReviewModule {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

#[async_trait]
impl Module for ReviewModule {
    fn name(&self) -> &str {
        "review"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn dependencies(&self) -> Vec<&str> {
        vec!["core"]
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    async fn initialize(&self, container: &ServiceContainer) -> Result<(), ModuleError> {
        // 注册评审相关服务
        // TODO: 实现评审服务的注册逻辑
        Ok(())
    }
    
    async fn unload(&self, _container: &ServiceContainer) -> Result<(), ModuleError> {
        // 清理评审模块资源
        Ok(())
    }
}

/// 扫描模块
pub struct ScanModule {
    enabled: bool,
}

impl ScanModule {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

#[async_trait]
impl Module for ScanModule {
    fn name(&self) -> &str {
        "scan"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn dependencies(&self) -> Vec<&str> {
        vec!["core"]
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    async fn initialize(&self, container: &ServiceContainer) -> Result<(), ModuleError> {
        // 注册扫描相关服务
        // TODO: 实现扫描服务的注册逻辑
        Ok(())
    }
    
    async fn unload(&self, _container: &ServiceContainer) -> Result<(), ModuleError> {
        // 清理扫描模块资源
        Ok(())
    }
}

use super::provider::ProviderFactory;
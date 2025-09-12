/// MCP 服务注册表整合模块
///
/// 将 MCP 服务注册表整合到 GitAiMcpManager 中
use gitai_core::config::Config;
use crate::mcp::registry::{ServiceEvent, ServiceEventListener, ServiceMetadata, ServiceRegistry};
use crate::mcp::{GitAiMcpService, McpResult, Tool};
use log::{debug, error, info, warn};
use std::sync::Arc;

/// 整合服务注册表到 MCP 服务管理器中
pub struct ManagedServiceRegistry {
    /// 配置
    config: Config,
    /// 服务注册表
    registry: ServiceRegistry,
}

/// 服务事件处理器
struct ServiceEventHandler {
    #[allow(dead_code)]
    name: String,
}

#[async_trait::async_trait]
impl ServiceEventListener for ServiceEventHandler {
    async fn on_service_event(&self, event: ServiceEvent) {
        match event {
            ServiceEvent::Registered {
                service_id,
                metadata,
            } => {
                info!(
                    "🎉 服务已注册: {} (ID: {}, 版本: {})",
                    metadata.name, service_id, metadata.version
                );
            }
            ServiceEvent::Unregistered { service_id, reason } => {
                info!("📤 服务已注销: {} (原因: {})", service_id, reason);
            }
            ServiceEvent::StatusChanged {
                service_id,
                old_status,
                new_status,
            } => {
                info!(
                    "🔄 服务状态变更: {} {:?} -> {:?}",
                    service_id, old_status, new_status
                );
            }
            ServiceEvent::HealthCheckCompleted {
                service_id,
                healthy,
                response_time,
            } => {
                if healthy {
                    debug!(
                        "💚 服务健康检查通过: {} (响应时间: {:?})",
                        service_id, response_time
                    );
                } else {
                    warn!(
                        "💔 服务健康检查失败: {} (响应时间: {:?})",
                        service_id, response_time
                    );
                }
            }
        }
    }
}

impl ManagedServiceRegistry {
    /// 创建新的服务注册表管理器
    pub async fn new(config: Config) -> McpResult<Self> {
        let registry = ServiceRegistry::new();

        // 添加事件监听器
        let event_handler = Arc::new(ServiceEventHandler {
            name: "GitAI-MCP".to_string(),
        });
        registry.add_event_listener(event_handler).await;

        let mut manager = Self {
            config: config.clone(),
            registry,
        };

        // 根据配置初始化服务
        if let Some(mcp_config) = &config.mcp {
            if mcp_config.enabled {
                info!("📋 启用 MCP 服务: {:?}", mcp_config.services.enabled);
                for service_name in &mcp_config.services.enabled {
                    if let Err(e) = manager.register_service_by_name(service_name).await {
                        error!("❌ 服务 '{}' 注册失败: {}", service_name, e);
                    }
                }

                let services = manager.list_services().await;
                info!(
                    "🎯 MCP 服务管理器初始化完成，共注册 {} 个服务",
                    services.len()
                );
            } else {
                info!("ℹ️ MCP 服务已禁用");
            }
        } else {
            info!("ℹ️ 未找到 MCP 配置");
        }

        Ok(manager)
    }

    /// 根据服务名称注册服务
    async fn register_service_by_name(&mut self, service_name: &str) -> McpResult<()> {
        debug!("🔧 正在初始化服务: {}", service_name);

        let config = self.config.clone();
        let service_config = serde_json::json!({
            "service_name": service_name,
            "enabled": true
        });

        match service_name {
            "review" => {
                let service = crate::mcp::services::ReviewService::new(config).map_err(|e| {
                    crate::mcp::configuration_error(format!(
                        "Failed to create review service: {}",
                        e
                    ))
                })?;
                self.registry
                    .register_service(Arc::new(service), service_config)
                    .await?;
                info!("✅ 服务 'review' 注册成功");
            }
            "commit" => {
                let service = crate::mcp::services::CommitService::new(config).map_err(|e| {
                    crate::mcp::configuration_error(format!(
                        "Failed to create commit service: {}",
                        e
                    ))
                })?;
                self.registry
                    .register_service(Arc::new(service), service_config)
                    .await?;
                info!("✅ 服务 'commit' 注册成功");
            }
            "scan" => {
                let service = crate::mcp::services::ScanService::new(config).map_err(|e| {
                    crate::mcp::configuration_error(format!("Failed to create scan service: {}", e))
                })?;
                self.registry
                    .register_service(Arc::new(service), service_config)
                    .await?;
                info!("✅ 服务 'scan' 注册成功");
            }
            "analysis" => {
                let service = crate::mcp::services::AnalysisService::new(config).map_err(|e| {
                    crate::mcp::configuration_error(format!(
                        "Failed to create analysis service: {}",
                        e
                    ))
                })?;
                self.registry
                    .register_service(Arc::new(service), service_config)
                    .await?;
                info!("✅ 服务 'analysis' 注册成功");
            }
            "dependency" => {
                let service =
                    crate::mcp::services::DependencyService::new(config).map_err(|e| {
                        crate::mcp::configuration_error(format!(
                            "Failed to create dependency service: {}",
                            e
                        ))
                    })?;
                self.registry
                    .register_service(Arc::new(service), service_config)
                    .await?;
                info!("✅ 服务 'dependency' 注册成功");
            }
            "deviation" => {
                let service = crate::mcp::services::DeviationService::new(config).map_err(|e| {
                    crate::mcp::configuration_error(format!(
                        "Failed to create deviation service: {}",
                        e
                    ))
                })?;
                self.registry
                    .register_service(Arc::new(service), service_config)
                    .await?;
                info!("✅ 服务 'deviation' 注册成功");
            }
            _ => {
                warn!("⚠️ 未知的服务名称: {}", service_name);
                return Err(crate::mcp::configuration_error(format!(
                    "Unknown service: {}",
                    service_name
                )));
            }
        }

        Ok(())
    }

    /// 获取所有工具
    pub async fn get_all_tools(&self) -> Vec<Tool> {
        let mut tools = Vec::new();
        let services = self.registry.list_services().await;

        for metadata in services {
            if let Some(service) = self.registry.get_service(&metadata.id).await {
                tools.extend(service.tools());
            }
        }

        tools
    }

    /// 处理工具调用
    pub async fn handle_tool_call(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> McpResult<serde_json::Value> {
        debug!("🔧 处理工具调用: {}", tool_name);
        debug!(
            "📋 工具参数: {}",
            serde_json::to_string_pretty(&arguments).unwrap_or_default()
        );

        // 查找处理该工具的服务
        if let Some(service) = self.registry.find_service_by_tool(tool_name).await {
            debug!("🎯 找到处理服务: {}", service.name());
            let result = service.handle_tool_call(tool_name, arguments).await;

            match &result {
                Ok(_) => {
                    info!("✅ 工具调用成功: {}", tool_name);
                }
                Err(e) => {
                    warn!("❌ 工具调用失败: {} (错误: {})", tool_name, e);
                }
            }

            return result;
        }

        error!("❌ 未找到处理工具的服务: {}", tool_name);
        Err(crate::mcp::invalid_parameters_error(format!(
            "Unknown tool: {}",
            tool_name
        )))
    }

    /// 动态注册服务
    pub async fn register_service(
        &self,
        service: Arc<dyn GitAiMcpService + Send + Sync>,
        config: serde_json::Value,
    ) -> McpResult<()> {
        self.registry.register_service(service, config).await
    }

    /// 动态注销服务
    pub async fn unregister_service(&self, service_id: &str, reason: String) -> McpResult<()> {
        self.registry.unregister_service(service_id, reason).await
    }

    /// 获取所有服务列表
    pub async fn list_services(&self) -> Vec<ServiceMetadata> {
        self.registry.list_services().await
    }

    /// 获取健康的服务列表
    pub async fn get_healthy_services(&self) -> Vec<ServiceMetadata> {
        self.registry.get_healthy_services().await
    }
}

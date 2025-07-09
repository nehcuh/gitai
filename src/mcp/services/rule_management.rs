use rmcp::{
    handler::server::ServerHandler,
    model::{ServerInfo, Tool, Resource, Implementation, ServerCapabilities, ProtocolVersion},
};
use crate::mcp::rmcp_compat::{
    ServiceError, CompatServerHandler, ServerHandlerAdapter,
};
use crate::mcp::{McpService, RmcpResult};

/// 规则管理服务（占位符实现）
pub struct RuleManagementService {
    name: String,
    version: String,
    description: String,
    running: bool,
}

impl RuleManagementService {
    pub fn new() -> Self {
        Self {
            name: "gitai-rule-management-service".to_string(),
            version: "1.0.0".to_string(),
            description: "GitAI 规则管理服务".to_string(),
            running: false,
        }
    }
}

impl McpService for RuleManagementService {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn start_sync(&mut self) -> RmcpResult<()> {
        self.running = true;
        Ok(())
    }

    fn stop_sync(&mut self) -> RmcpResult<()> {
        self.running = false;
        Ok(())
    }

    fn health_check_sync(&self) -> RmcpResult<bool> {
        Ok(self.running)
    }

    fn get_handler_info(&self) -> String {
        format!("RuleManagementHandler for service: {}", self.name)
    }
}

impl Clone for RuleManagementService {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            running: self.running,
        }
    }
}

pub struct RuleManagementHandler {
    service: RuleManagementService,
}

impl RuleManagementHandler {
    pub fn new(service: RuleManagementService) -> Self {
        Self { service }
    }
}

impl CompatServerHandler for RuleManagementHandler {
    fn get_server_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::default(),
            server_info: Implementation {
                name: self.service.name.clone(),
                version: self.service.version.clone(),
            },
            instructions: None,
        }
    }

    fn list_tools(&self) -> Vec<Tool> {
        vec![]
    }

    fn list_resources(&self) -> Vec<Resource> {
        vec![]
    }

    fn call_tool(&self, _name: &str, _args: serde_json::Value) -> Result<serde_json::Value, ServiceError> {
        Err(ServiceError::MethodNotFound("call_tool".to_string()))
    }

    fn read_resource(&self, _uri: &str) -> Result<String, ServiceError> {
        Err(ServiceError::MethodNotFound("read_resource".to_string()))
    }
}
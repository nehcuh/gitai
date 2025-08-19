use rmcp::{
    model::{ServerInfo, Tool, Resource, Implementation, ServerCapabilities, ProtocolVersion},
};
use crate::mcp::rmcp_compat::{
    ServiceError, CompatServerHandler,
};
use crate::mcp::{McpService, RmcpResult};

/// AI 分析服务（占位符实现）
pub struct AiAnalysisService {
    name: String,
    version: String,
    description: String,
    running: bool,
}

impl AiAnalysisService {
    pub fn new() -> Self {
        Self {
            name: "gitai-ai-analysis-service".to_string(),
            version: "1.0.0".to_string(),
            description: "GitAI AI 分析服务".to_string(),
            running: false,
        }
    }
}

impl McpService for AiAnalysisService {
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
        format!("AiAnalysisHandler for service: {}", self.name)
    }
}

impl Clone for AiAnalysisService {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            running: self.running,
        }
    }
}

pub struct AiAnalysisHandler {
    service: AiAnalysisService,
}

impl AiAnalysisHandler {
    pub fn new(service: AiAnalysisService) -> Self {
        Self { service }
    }
}

impl CompatServerHandler for AiAnalysisHandler {
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
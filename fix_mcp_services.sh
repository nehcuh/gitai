#!/bin/bash

# 批量修复 MCP 服务文件以使用兼容性适配层

files=("rule_management.rs" "scanner.rs")

for file in "${files[@]}"; do
    echo "Fixing $file..."
    
    # 1. 更新 imports
    sed -i '' 's/service::ServiceError,/};/' "src/mcp/services/$file"
    sed -i '' '/model::{ServerInfo, Tool, Resource},/a\
use crate::mcp::rmcp_compat::\
    ServiceError, CompatServerHandler, ServerHandlerAdapter,\
};' "src/mcp/services/$file"
    
    # 2. 更新 ServerHandler 实现
    sed -i '' 's/impl ServerHandler for \(.*\)Handler {/impl CompatServerHandler for \1Handler {\
    fn get_server_info(\&self) -> ServerInfo {\
        ServerInfo {\
            name: self.service.name.clone(),\
            version: self.service.version.clone(),\
        }\
    }/' "src/mcp/services/$file"
    
    # 3. 移除注释行
    sed -i '' '/Note: info method is not part of ServerHandler trait/d' "src/mcp/services/$file"
    
    # 4. 更新 create_handler 方法
    sed -i '' 's/Box::new(\(.*\)Handler::new(self.clone()))/Box::new(ServerHandlerAdapter::new(\1Handler::new(self.clone())))/' "src/mcp/services/$file"
done

echo "All files fixed!"
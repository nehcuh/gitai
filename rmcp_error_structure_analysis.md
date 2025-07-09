# rmcp 0.2.1 库中 Error 结构分析

## 概述

这个文档详细分析了 rmcp 0.2.1 库中 `ServiceError` 和 `Error` (也称为 `McpError`) 的实际结构和关系。

## 1. ServiceError 的实际结构

在 rmcp 0.2.1 中，`ServiceError` 位于 `rmcp::service::ServiceError`，它的实际定义如下：

```rust
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ServiceError {
    #[error("Mcp error: {0}")]
    McpError(McpError),
    #[error("Transport send error: {0}")]
    TransportSend(Box<dyn std::error::Error + Send + Sync>),
    #[error("Transport closed")]
    TransportClosed,
    #[error("Unexpected response type")]
    UnexpectedResponse,
    #[error("task cancelled for reason {}", reason.as_deref().unwrap_or("<unknown>"))]
    Cancelled { reason: Option<String> },
    #[error("request timeout after {}", chrono::Duration::from_std(*timeout).unwrap_or_default())]
    Timeout { timeout: Duration },
}
```

**重要发现：**
- `ServiceError` **不直接包含** `InvalidParams`、`InternalError` 等 MCP 协议错误变体
- 这些协议错误通过 `ServiceError::McpError(error)` 变体来包装
- `ServiceError` 主要处理传输层和服务层的错误

## 2. McpError (Error) 的结构

`McpError` 是 `ErrorData` 的类型别名：

```rust
// 在 rmcp::error 模块中
pub type Error = ErrorData;

// ErrorData 的定义
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ErrorData {
    /// 错误类型代码 (使用标准 JSON-RPC 错误码)
    pub code: ErrorCode,
    /// 简短的错误描述
    pub message: Cow<'static, str>,
    /// 可选的额外错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}
```

## 3. ErrorCode 的标准常量

`ErrorCode` 是一个 `i32` 的包装器，提供了标准的 JSON-RPC 错误码常量：

```rust
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct ErrorCode(pub i32);

impl ErrorCode {
    pub const RESOURCE_NOT_FOUND: Self = Self(-32002);
    pub const INVALID_REQUEST: Self = Self(-32600);
    pub const METHOD_NOT_FOUND: Self = Self(-32601);
    pub const INVALID_PARAMS: Self = Self(-32602);
    pub const INTERNAL_ERROR: Self = Self(-32603);
    pub const PARSE_ERROR: Self = Self(-32700);
}
```

## 4. 错误类型的层次关系

```
ServiceError
├── McpError(ErrorData)  ← MCP 协议层错误
│   ├── InvalidParams (code: -32602)
│   ├── InternalError (code: -32603)
│   ├── MethodNotFound (code: -32601)
│   ├── ParseError (code: -32700)
│   ├── InvalidRequest (code: -32600)
│   └── ResourceNotFound (code: -32002)
├── TransportSend(...)   ← 传输发送错误
├── TransportClosed      ← 传输连接关闭
├── UnexpectedResponse   ← 意外响应类型
├── Cancelled {...}      ← 请求被取消
└── Timeout {...}        ← 请求超时
```

## 5. 如何创建 MCP 协议错误

要创建传统的 MCP 协议错误（如 `InvalidParams`），需要：

1. 创建 `ErrorData` 并设置相应的 `ErrorCode`
2. 将其包装在 `ServiceError::McpError()` 中

示例：

```rust
use rmcp::{
    service::ServiceError,
    error::Error as McpError,
    model::{ErrorCode, ErrorData},
};

// 创建 InvalidParams 错误
let invalid_params = ErrorData {
    code: ErrorCode::INVALID_PARAMS,
    message: "参数无效".into(),
    data: None,
};
let service_error = ServiceError::McpError(invalid_params);

// 创建 InternalError 错误
let internal_error = ErrorData {
    code: ErrorCode::INTERNAL_ERROR,
    message: "内部错误".into(),
    data: None,
};
let service_error = ServiceError::McpError(internal_error);

// 创建 MethodNotFound 错误
let method_not_found = ErrorData {
    code: ErrorCode::METHOD_NOT_FOUND,
    message: "方法未找到".into(),
    data: Some(serde_json::json!({"method": "unknown_method"})),
};
let service_error = ServiceError::McpError(method_not_found);
```

## 6. 项目中的兼容层

项目中的 `rmcp_compat.rs` 提供了一个兼容层，简化了错误处理：

```rust
// 兼容层的简化 ServiceError
pub enum ServiceError {
    InvalidParams(String),
    InternalError(String),
    ParseError(String),
    MethodNotFound(String),
    Custom(String),
}

// 提供了与 rmcp::service::ServiceError 的转换
impl From<ServiceError> for RmcpServiceError {
    fn from(error: ServiceError) -> Self {
        match error {
            ServiceError::InvalidParams(_) => RmcpServiceError::McpError(ErrorData {
                code: ErrorCode::INVALID_PARAMS,
                message: "参数无效".into(),
                data: None,
            }),
            // ... 其他转换
        }
    }
}
```

## 7. 总结

1. **ServiceError 结构**：rmcp 0.2.1 中的 `ServiceError` 包含 `McpError` 变体来处理 MCP 协议错误
2. **McpError 是 ErrorData**：`McpError` 是 `ErrorData` 的类型别名，包含 `code`、`message` 和 `data` 字段
3. **ErrorCode 常量**：提供了标准的 JSON-RPC 错误码，如 `INVALID_PARAMS` (-32602)、`INTERNAL_ERROR` (-32603) 等
4. **错误创建**：要创建 `InvalidParams` 等错误，需要创建 `ErrorData` 并设置相应的 `ErrorCode`，然后包装在 `ServiceError::McpError()` 中
5. **兼容层**：项目提供了兼容层来简化错误处理和类型转换

这个结构设计遵循了 JSON-RPC 2.0 规范，提供了标准化的错误报告机制。
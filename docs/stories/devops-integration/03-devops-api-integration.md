# 用户故事 03: DevOps API 集成

## 故事概述
**作为一名开发者**
**我希望 gitai 能够自动从 DevOps 平台获取工作项的详细信息**
**这样我就能够获得准确的需求描述，用于代码评审时的需求对比分析**

## 详细描述

### 用户角色
- 开发工程师
- 技术负责人
- QA 工程师

### 功能需求
实现与 DevOps 平台的 REST API 集成：

1. 根据工作项 ID 调用 DevOps 平台 API
2. 获取工作项的详细信息
3. 解析响应数据并提取关键字段
4. 处理 API 调用的各种异常情况
5. 支持并发请求以提高效率
6. 实现请求重试和超时机制

### API 规格

#### 请求格式
```http
GET /external/collaboration/api/project/{space_id}/issues/{issue_id}
Host: codingcorp.devops.xxx.com.cn
Accept: application/json
Authorization: token {token}
Content-Type: application/json
```

#### 响应格式
所有 DevOps API 响应都包含 `code`，`msg` 和 `data` 三个顶层字段。其中 `data` 字段的内部结构会根据工作项的类型（如用户故事、缺陷、任务）而有所不同。

##### 通用 `data` 结构建议
由于 `data` 内部字段差异较大，建议在代码中将 `data` 字段首先解析为通用的 JSON 对象（例如，在 Rust 中使用 `serde_json::Value`）。然后，根据具体的业务需求，从中提取所需的字段。

对于不同类型的工作项，我们通常关注以下核心信息：
- **项目/产品名称**: 通常在 `data.program.display_name` 中。
- **工作项类型**: 在 `data.issueTypeDetail.name` 中，例如 "用户故事", "缺陷", "任务"。
- **工作项标题**: 在 `data.name` 中。
- **工作项描述**: 在 `data.description` 中。

以下是针对不同工作项类型 `data` 字段的简化示例，重点展示上述核心信息：

##### 用户故事 (User Story) `data` 示例 (部分字段)
```json
{
  "code": 0,
  "msg": null,
  "data": {
    // ... 其他用户故事特有字段 ...
    "name": "封装 requests 函数到用户自定义函数", // 工作项标题
    "description": "# 描述\n- 作为 普通用户...",     // 工作项描述
    "issueTypeDetail": {
      "name": "用户故事" // 工作项类型
      // ...
    },
    "program": {
      "display_name": "金科中心代码扫描引擎项目预研" // 项目/产品名称
      // ...
    }
    // ... 其他用户故事特有字段 ...
  }
}
```

##### 缺陷 (Defect) `data` 示例 (部分字段)
```json
{
  "code": 0,
  "msg": null,
  "data": {
    // ... 其他缺陷特有字段 ...
    "name": "交易运营部-公募T0账单-资金信息汇总未统计理财持仓市值。", // 工作项标题
    "description": "![picture](/api/project/726365/files/3439204/imagePreview)\n ", // 工作项描述
    "issueTypeDetail": {
      "name": "缺陷" // 工作项类型
      // ...
    },
    "program": {
      "display_name": "T7.6券结(含券结ETF)融资行权业务回归及单客户上线" // 项目/产品名称
      // ...
    }
    // ... 其他缺陷特有字段 ...
  }
}
```

##### 任务 (Task) `data` 示例 (部分字段)
```json
{
  "code": 0,
  "msg": null,
  "data": {
    // ... 其他任务特有字段 ...
    "name": "交易网关9502超时优化", // 工作项标题
    "description": "2024年5月，柜台反馈...", // 工作项描述
    "issueTypeDetail": {
      "name": "任务" // 工作项类型
      // ...
    },
    "program": {
      // "display_name" 可能为 null 或不存在于所有任务类型中，需注意处理
      "display_name": null // 项目/产品名称 (示例中为null)
      // ...
    }
    // ... 其他任务特有字段 ...
  }
}
```

### 使用场景

#### 场景 1: 单个工作项获取
```bash
gitai review --space-id=726226 --stories=99
# 系统调用: GET /project/726226/issues/99
```

#### 场景 2: 多个工作项批量获取
```bash
gitai review --space-id=726226 --stories=99,100,101
# 系统并发调用:
# GET /project/726226/issues/99
# GET /project/726226/issues/100
# GET /project/726226/issues/101
```

#### 场景 3: 混合工作项类型获取
```bash
gitai review --space-id=726226 --stories=99 --tasks=200 --defects=301
# 系统调用多个不同类型的工作项
```

## 验收标准

### API 调用功能
- [ ] 成功调用 DevOps 平台 REST API
- [ ] 正确构造 API 请求 URL
- [ ] 正确设置请求头（Authorization, Accept, Content-Type）
- [ ] 支持 HTTPS 加密传输
- [ ] 正确处理 HTTP 状态码

### 数据解析功能
- [ ] 正确解析 JSON 响应数据
- [ ] 提取 `issueTypeDetail.name` 字段
- [ ] 提取 `description` 字段
- [ ] 提取工作项基本信息（id, name, type, status）
- [ ] 处理缺失字段的情况
- [ ] 处理 JSON 格式错误

### 并发处理
- [ ] 支持多个工作项的并发请求
- [ ] 限制并发数量（最多10个）
- [ ] 正确汇总所有请求结果
- [ ] 处理部分请求失败的情况
- [ ] 保持结果顺序与输入顺序一致

### 错误处理
- [ ] 网络连接错误处理
- [ ] 认证失败（401）错误处理
- [ ] 工作项不存在（404）错误处理
- [ ] API 限流（429）错误处理
- [ ] 服务器错误（5xx）处理
- [ ] 请求超时错误处理

### 重试机制
- [ ] 实现指数退避重试策略
- [ ] 可配置重试次数（默认3次）
- [ ] 可配置请求超时时间（默认10秒）
- [ ] 对临时错误进行重试
- [ ] 对永久错误不进行重试

## 技术实现要求

### 数据结构定义
```rust
#[derive(Debug, Deserialize)]
pub struct DevOpsResponse {
    pub code: i32,
    pub msg: Option<String>,
    // The 'data' field is flexible and will be parsed further based on context
    // using serde_json::Value allows handling different structures for stories, defects, tasks etc.
    pub data: Option<serde_json::Value>,
}

// Represents common information to be extracted from the 'data' field (serde_json::Value) 
// of any work item type (story, defect, task).
// Fields are Option<> to gracefully handle missing data.
#[derive(Debug, Deserialize, Default)]
pub struct WorkItemCommonInfo {
    pub id: Option<u32>, // Unique ID of the work item
    pub code: Option<u32>, // Work item's own code/identifier (e.g., 99 for story, 833118 for defect)
    
    // "卡片标题" (Work item title)
    pub name: Option<String>, 
    
    // "卡片内容" (Work item description)
    pub description: Option<String>, 
    
    // Top-level type of the work item, e.g., "REQUIREMENT", "DEFECT", "MISSION"
    #[serde(rename = "type")] 
    pub item_type: Option<String>, 
    
    // Contains "卡片类型" (e.g., "用户故事", "缺陷") via its 'name' field
    #[serde(rename = "issueTypeDetail")]
    pub issue_type_detail: Option<IssueTypeDetail>, 
    
    // Contains context for the work item, like "program.display_name"
    pub program: Option<ProgramInfo>, 
    
    #[serde(rename = "issueStatusName")]
    pub status_name: Option<String>, // e.g., "未开始", "已预审"
    pub priority: Option<u32>,
    
    // Consider adding other common fields like 'creator', 'assignee', 'createdAt', 'updatedAt'
    // if they are consistently needed and available across different work item types.
    // Example:
    // pub creator: Option<UserInfo>,
    // pub assignee: Option<UserInfo>,
    // #[serde(rename = "createdAt")]
    // pub created_at: Option<u64>, // Assuming timestamp
}

// Detailed information about the issue type (e.g., "用户故事", "缺陷", "任务")
#[derive(Debug, Deserialize, Default)]
pub struct IssueTypeDetail {
    pub id: Option<u32>,
    // "卡片类型" (e.g., "用户故事", "缺陷", "任务")
    pub name: Option<String>, 
    #[serde(rename = "iconType")]
    pub icon_type: Option<String>, // e.g., "story", "defect", "mission"
    // This field often mirrors WorkItemCommonInfo.item_type (e.g., "REQUIREMENT", "DEFECT")
    #[serde(rename = "issueType")] 
    pub type_category: Option<String>, 
}

// Information about the program/project the work item belongs to
#[derive(Debug, Deserialize, Default)]
pub struct ProgramInfo {
    pub id: Option<u32>,
    // Context for the work item, e.g., "金科中心代码扫描引擎项目预研"
    #[serde(alias = "display_name")] 
    pub display_name: Option<String>, 
    // Internal name of the program/project
    pub name: Option<String>, 
    // pub html_url: Option<String>, // Example of another potentially useful field
}

// Example UserInfo struct if creator/assignee details are needed
// #[derive(Debug, Deserialize, Default)]
// pub struct UserInfo {
//     pub id: Option<u32>,
//     pub name: Option<String>,
//     #[serde(rename = "globalKey")]
//     pub global_key: Option<String>,
//     pub avatar: Option<String>,
// }

// Note on usage:
// 1. Deserialize the entire JSON response into `DevOpsResponse`.
// 2. If `response.data` is `Some(value)`, this `value` is a `serde_json::Value`.
// 3. You can then attempt to deserialize this `value` into `WorkItemCommonInfo`:
//    `let specific_item_info: Result<WorkItemCommonInfo, _> = serde_json::from_value(value.clone());`
//    Or, access fields directly: `value["name"].as_str()`, `value["program"]["display_name"].as_str()`.
//    This approach allows extracting the common fields defined in `WorkItemCommonInfo` while
//    still retaining the flexibility to access any other specific fields from the `serde_json::Value`
//    that might be unique to a particular work item type.
```

### API 客户端实现
```rust
pub struct DevOpsClient {
    base_url: String,
    token: String,
    client: reqwest::Client,
    retry_count: u32,
    timeout: Duration,
}

impl DevOpsClient {
    pub async fn get_work_item(&self, space_id: u32, item_id: u32) -> Result<WorkItem, ApiError> {
        // 实现单个工作项获取
    }
    
    pub async fn get_work_items(&self, space_id: u32, item_ids: &[u32]) -> Result<Vec<WorkItem>, ApiError> {
        // 实现并发批量获取
    }
    
    async fn make_request_with_retry(&self, url: &str) -> Result<DevOpsResponse, ApiError> {
        // 实现重试逻辑
    }
}
```

### 错误类型定义
```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("网络请求失败: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("认证失败: 无效的 token")]
    AuthenticationError,
    
    #[error("工作项 {item_id} 不存在")]
    WorkItemNotFound { item_id: u32 },
    
    #[error("API 限流，请稍后重试")]
    RateLimitExceeded,
    
    #[error("服务器错误: {status_code}")]
    ServerError { status_code: u16 },
    
    #[error("响应数据解析失败: {0}")]
    ParseError(#[from] serde_json::Error),
    
    #[error("请求超时")]
    TimeoutError,
}
```

## 性能要求

### 响应时间
- [ ] 单个工作项请求：< 2秒
- [ ] 10个工作项并发请求：< 5秒
- [ ] 网络超时时间：10秒
- [ ] 重试间隔：1秒、2秒、4秒（指数退避）

### 资源使用
- [ ] 内存使用合理，及时释放响应数据
- [ ] 连接池管理，复用 HTTP 连接
- [ ] 限制并发连接数，避免过载
- [ ] 支持请求取消机制

## 安全要求

### 数据安全
- [ ] 使用 HTTPS 加密传输
- [ ] 不在日志中记录完整的 token
- [ ] 不在错误信息中暴露敏感配置
- [ ] 验证 SSL 证书有效性

### 请求安全
- [ ] 设置合理的 User-Agent
- [ ] 防止 SSRF 攻击
- [ ] 验证响应内容长度
- [ ] 限制重定向次数

## 优先级
**高优先级** - 这是获取 DevOps 数据的核心功能，直接影响后续的 AI 分析。

## 估算工作量
- API 客户端基础实现：2天
- 错误处理和重试机制：1天
- 并发处理优化：1天
- 数据解析和验证：1天
- 单元测试和集成测试：2天
- 文档编写：0.5天

## 依赖关系
- 依赖：用户故事 02 (配置管理集成)
- 被依赖：用户故事 04 (AI 分析集成)

## 测试用例

### 正常场景测试
1. 测试单个工作项成功获取
2. 测试多个工作项并发获取
3. 测试不同类型工作项获取
4. 测试大型响应数据处理

### 异常场景测试
1. 测试网络连接失败
2. 测试认证失败处理
3. 测试工作项不存在处理
4. 测试 API 限流处理
5. 测试服务器错误处理
6. 测试响应格式错误处理

### 性能测试
1. 测试并发请求性能
2. 测试重试机制性能
3. 测试超时处理
4. 测试内存使用情况

### 安全测试
1. 测试 HTTPS 连接
2. 测试敏感信息保护
3. 测试证书验证
4. 测试恶意响应处理

## 完成定义 (Definition of Done)
- [ ] 代码实现完成并通过代码评审
- [ ] 单元测试覆盖率达到 90% 以上
- [ ] 集成测试通过
- [ ] 性能测试满足要求
- [ ] 安全测试通过
- [ ] API 文档更新完成
- [ ] 功能演示通过产品验收
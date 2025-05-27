# GitAI Review DevOps 集成功能产品需求文档

## 1. 产品概述

### 1.1 功能简介
扩展 `gitai review` 命令，支持与 DevOps 平台集成，通过工作项 ID（用户故事、任务、缺陷）自动获取需求描述，结合代码变更进行智能评审，分析代码实现与需求描述的契合度。

### 1.2 核心价值
- **需求追溯性**：确保代码变更与对应的工作项需求一致
- **质量保障**：通过 AI 分析代码实现与需求描述的偏离度
- **效率提升**：自动化获取需求信息，减少手动查找成本
- **团队协作**：增强开发团队对需求理解的一致性

## 2. 功能需求

### 2.1 核心功能
支持 `gitai review` 命令新增以下参数：
- `--stories=[story_id_1,story_id_2,...]`：用户故事 ID 列表
- `--tasks=[task_id_1,task_id_2,...]`：任务 ID 列表  
- `--defects=[defect_id_1,defect_id_2,...]`：缺陷 ID 列表
- `--space-id=space_id`：必需参数，指定 DevOps 空间 ID

### 2.2 配置集成
- 读取用户配置文件中的 `account` section
- 获取 DevOps 平台的认证 token
- 支持多种 DevOps 平台配置

### 2.3 API 集成
- 调用 DevOps 平台 REST API 获取工作项详情
- 解析响应数据，提取关键字段：
  - `issueTypeDetail.name`：工作项类型名称
  - `description`：工作项描述内容

### 2.4 智能评审
- 结合 Git diff 内容和 DevOps 工作项描述
- 使用 AI 分析代码实现与需求描述的一致性
- 生成评审报告，包括：
  - 代码质量评审
  - 需求实现完整性分析
  - 代码与需求描述的偏离度评估

## 3. 技术规格

### 3.1 命令行接口
```bash
# 基本用法
gitai review --space-id=726226 --stories=99,100,101

# 混合工作项类型
gitai review --space-id=726226 --stories=99 --tasks=200 --defects=301

# 结合现有参数
gitai review --space-id=726226 --stories=99 --depth=deep --format=json
```

### 3.2 配置文件格式
```toml
[account]
# DevOps 平台配置
devops_platform = "coding"  # 支持 coding, jira, azure-devops 等
base_url = "https://codingcorp.devops.cmschina.com.cn"
token = "c4e9e27573a4437d6ecad5119b2ebe026f5fdbc8"
```

### 3.3 API 请求格式
```http
GET /external/collaboration/api/project/{space_id}/issues/{issue_id}
Accept: application/json
Authorization: token {token}
Content-Type: application/json
```

### 3.4 响应数据处理
从 API 响应中提取：
```json
{
  "data": {
    "issueTypeDetail": {
      "name": "用户故事"
    },
    "description": "# 描述\n- 作为 普通用户\n- 我可以 通过添加指定 url，headers，method，payload\n- 这样我就可以 直接获取指定 url 的 json 格式返回值"
  }
}
```

## 4. 用户体验设计

### 4.1 参数验证
- 当使用 `--stories`、`--tasks` 或 `--defects` 时，必须提供 `--space-id`
- 工作项 ID 支持逗号分隔的列表格式
- 提供清晰的错误提示信息

### 4.2 输出格式
- 标准文本格式：人性化的评审报告
- JSON 格式：便于工具集成
- Markdown 格式：便于文档化

### 4.3 进度反馈
- 显示 API 请求进度
- 显示工作项数据获取状态
- 显示 AI 分析进度

## 5. 错误处理

### 5.1 配置错误
- 缺少 `account` 配置项
- 无效的认证 token
- 错误的 DevOps 平台 URL

### 5.2 API 错误
- 网络连接失败
- 认证失败（401）
- 工作项不存在（404）
- API 限流（429）

### 5.3 数据错误
- 工作项数据格式异常
- 必需字段缺失
- 编码问题

## 6. 性能要求

### 6.1 响应时间
- 单个工作项 API 请求：< 2 秒
- 多个工作项并发请求：< 5 秒
- AI 分析处理：< 10 秒

### 6.2 并发处理
- 支持最多 10 个工作项并发请求
- 实现请求重试机制
- 支持请求超时控制

## 7. 安全要求

### 7.1 认证安全
- Token 存储在用户配置文件中
- 支持环境变量覆盖配置
- 避免在日志中暴露敏感信息

### 7.2 数据安全
- HTTPS 加密传输
- 不在缓存中存储敏感数据
- 遵循最小权限原则

## 8. 扩展性设计

### 8.1 多平台支持
- 抽象 DevOps 平台接口
- 支持插件化平台适配器
- 统一的工作项数据模型

### 8.2 自定义字段
- 支持配置自定义提取字段
- 支持字段映射配置
- 支持数据转换规则

## 9. 验收标准

### 9.1 功能验收
- [ ] 正确解析新增的命令行参数
- [ ] 成功集成 DevOps 平台 API
- [ ] 正确提取工作项关键信息
- [ ] 生成包含偏离度分析的评审报告

### 9.2 质量验收
- [ ] 单元测试覆盖率 > 90%
- [ ] 集成测试覆盖主要场景
- [ ] 错误处理完整且用户友好
- [ ] 性能满足要求指标

### 9.3 用户体验验收
- [ ] 命令行界面直观易用
- [ ] 错误信息清晰准确
- [ ] 输出格式规范一致
- [ ] 文档完整准确

## 10. 实施计划

### 10.1 第一阶段（基础功能）
- 扩展命令行参数解析
- 实现配置文件读取
- 完成 DevOps API 集成

### 10.2 第二阶段（核心功能）
- 实现工作项数据处理
- 集成 AI 偏离度分析
- 完善错误处理机制

### 10.3 第三阶段（优化完善）
- 性能优化和并发处理
- 多平台支持扩展
- 用户体验优化
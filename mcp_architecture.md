# GitAI MCP 服务化架构设计

## 概述

本文档描述了 GitAI 项目的 MCP（Model Context Protocol）服务化架构设计，旨在将核心功能模块转换为独立的 MCP 服务，提高系统的可扩展性和可维护性。

## 目标架构

### 核心 MCP 服务列表

1. **Tree-sitter 分析服务** (`gitai-treesitter-service`)
2. **AI 分析引擎服务** (`gitai-ai-analysis-service`)
3. **DevOps 集成服务** (`gitai-devops-service`)
4. **规则管理服务** (`gitai-rule-management-service`)
5. **安全扫描服务** (`gitai-scanner-service`)

### 服务架构图

```
┌─────────────────────────────────────────────────────────────┐
│                     GitAI Client                            │
│                 (MCP Client Host)                           │
├─────────────────────────────────────────────────────────────┤
│                    MCP Registry                             │
│              (Service Discovery Layer)                      │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │Tree-sitter  │  │AI Analysis  │  │DevOps Int.  │         │
│  │Service      │  │Service      │  │Service      │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐                          │
│  │Rule Mgmt    │  │Scanner      │                          │
│  │Service      │  │Service      │                          │
│  └─────────────┘  └─────────────┘                          │
└─────────────────────────────────────────────────────────────┘
```

## 服务详细设计

### 1. Tree-sitter 分析服务

**职责**：提供代码静态分析和 AST 解析功能

**MCP 接口**：
```json
{
  "name": "gitai-treesitter-service",
  "version": "1.0.0",
  "tools": [
    {
      "name": "analyze_code",
      "description": "分析代码结构和语法",
      "inputSchema": {
        "type": "object",
        "properties": {
          "content": {"type": "string"},
          "language": {"type": "string"},
          "focus_areas": {"type": "array", "items": {"type": "string"}}
        }
      }
    },
    {
      "name": "parse_diff",
      "description": "解析和分析 Git diff",
      "inputSchema": {
        "type": "object",
        "properties": {
          "diff": {"type": "string"},
          "context_lines": {"type": "number", "default": 3}
        }
      }
    },
    {
      "name": "detect_language",
      "description": "检测代码语言",
      "inputSchema": {
        "type": "object",
        "properties": {
          "file_path": {"type": "string"},
          "content": {"type": "string"}
        }
      }
    }
  ],
  "resources": [
    {
      "uri": "queries/{language}/highlights.scm",
      "description": "语言高亮查询规则"
    },
    {
      "uri": "queries/{language}/structure.scm",
      "description": "语言结构查询规则"
    }
  ]
}
```

### 2. AI 分析引擎服务

**职责**：提供 AI 驱动的代码分析和提交信息生成

**MCP 接口**：
```json
{
  "name": "gitai-ai-analysis-service",
  "version": "1.0.0",
  "tools": [
    {
      "name": "analyze_with_requirements",
      "description": "基于需求分析代码变更",
      "inputSchema": {
        "type": "object",
        "properties": {
          "work_items": {"type": "array"},
          "git_diff": {"type": "string"},
          "analysis_depth": {"type": "string", "enum": ["basic", "detailed", "comprehensive"]}
        }
      }
    },
    {
      "name": "generate_commit_message",
      "description": "生成智能提交信息",
      "inputSchema": {
        "type": "object",
        "properties": {
          "diff": {"type": "string"},
          "context": {"type": "string"},
          "style": {"type": "string", "enum": ["conventional", "descriptive", "technical"]}
        }
      }
    },
    {
      "name": "create_review_prompt",
      "description": "创建代码审查提示",
      "inputSchema": {
        "type": "object",
        "properties": {
          "diff": {"type": "string"},
          "focus_areas": {"type": "array", "items": {"type": "string"}}
        }
      }
    }
  ],
  "resources": [
    {
      "uri": "templates/analysis/{type}.json",
      "description": "分析模板"
    },
    {
      "uri": "templates/prompts/{category}.md",
      "description": "提示模板"
    }
  ]
}
```

### 3. DevOps 集成服务

**职责**：提供工作项管理和 DevOps 平台集成

**MCP 接口**：
```json
{
  "name": "gitai-devops-service",
  "version": "1.0.0",
  "tools": [
    {
      "name": "get_work_item",
      "description": "获取单个工作项",
      "inputSchema": {
        "type": "object",
        "properties": {
          "space_id": {"type": "number"},
          "item_id": {"type": "number"}
        }
      }
    },
    {
      "name": "get_work_items_batch",
      "description": "批量获取工作项",
      "inputSchema": {
        "type": "object",
        "properties": {
          "space_id": {"type": "number"},
          "item_ids": {"type": "array", "items": {"type": "number"}}
        }
      }
    },
    {
      "name": "validate_work_item",
      "description": "验证工作项信息",
      "inputSchema": {
        "type": "object",
        "properties": {
          "item": {"type": "object"}
        }
      }
    }
  ],
  "resources": [
    {
      "uri": "work_items/{space_id}/{item_id}",
      "description": "工作项详情"
    },
    {
      "uri": "projects/{space_id}/metadata",
      "description": "项目元数据"
    }
  ]
}
```

### 4. 规则管理服务

**职责**：管理安全扫描规则和翻译

**MCP 接口**：
```json
{
  "name": "gitai-rule-management-service",
  "version": "1.0.0",
  "tools": [
    {
      "name": "update_rules",
      "description": "更新扫描规则",
      "inputSchema": {
        "type": "object",
        "properties": {
          "force_update": {"type": "boolean", "default": false}
        }
      }
    },
    {
      "name": "translate_rules",
      "description": "翻译规则到目标语言",
      "inputSchema": {
        "type": "object",
        "properties": {
          "target_language": {"type": "string"},
          "rule_categories": {"type": "array", "items": {"type": "string"}}
        }
      }
    },
    {
      "name": "get_rule_version_info",
      "description": "获取规则版本信息",
      "inputSchema": {
        "type": "object",
        "properties": {}
      }
    }
  ],
  "resources": [
    {
      "uri": "rules/{language}/{category}/",
      "description": "规则文件"
    },
    {
      "uri": "translated_rules/{language}/",
      "description": "翻译后的规则"
    }
  ]
}
```

### 5. 安全扫描服务

**职责**：执行代码安全扫描

**MCP 接口**：
```json
{
  "name": "gitai-scanner-service",
  "version": "1.0.0",
  "tools": [
    {
      "name": "scan_files",
      "description": "扫描文件列表",
      "inputSchema": {
        "type": "object",
        "properties": {
          "files": {"type": "array", "items": {"type": "string"}},
          "rules": {"type": "array", "items": {"type": "string"}},
          "severity_filter": {"type": "string", "enum": ["low", "medium", "high", "critical"]}
        }
      }
    },
    {
      "name": "scan_git_diff",
      "description": "扫描 Git diff 中的变更",
      "inputSchema": {
        "type": "object",
        "properties": {
          "diff": {"type": "string"},
          "incremental": {"type": "boolean", "default": true}
        }
      }
    },
    {
      "name": "get_scan_history",
      "description": "获取扫描历史",
      "inputSchema": {
        "type": "object",
        "properties": {
          "repository": {"type": "string"},
          "limit": {"type": "number", "default": 10}
        }
      }
    }
  ],
  "resources": [
    {
      "uri": "scan_results/{repository}/{timestamp}",
      "description": "扫描结果"
    },
    {
      "uri": "scan_templates/",
      "description": "扫描模板"
    }
  ]
}
```

## 服务间通信模式

### 数据流设计

1. **代码分析流程**：
   ```
   Code Input → Tree-sitter Service → AI Analysis Service → Formatted Results
   ```

2. **代码审查流程**：
   ```
   Git Diff → DevOps Service (work items) → AI Analysis Service → Review Output
   ```

3. **安全扫描流程**：
   ```
   Code Files → Rule Management Service → Scanner Service → Scan Results
   ```

### 服务依赖关系

- **AI 分析服务** 依赖 **Tree-sitter 服务** 和 **DevOps 服务**
- **扫描服务** 依赖 **规则管理服务**
- **所有服务** 通过 **MCP Registry** 进行服务发现

## 实施计划

### 阶段一：基础服务 (2-3 周)
1. 实现 Tree-sitter 分析服务
2. 实现 AI 分析引擎服务
3. 建立基础 MCP 通信框架

### 阶段二：集成服务 (2-3 周)
1. 实现 DevOps 集成服务
2. 实现规则管理服务
3. 完善服务间通信机制

### 阶段三：扩展服务 (1-2 周)
1. 实现安全扫描服务
2. 完善监控和日志系统
3. 性能优化和测试

## 技术考虑

### 性能优化
- **缓存策略**：Tree-sitter 分析结果缓存
- **并行处理**：多服务并行调用
- **连接池**：复用 MCP 连接

### 可靠性保证
- **错误处理**：统一的错误码和消息格式
- **重试机制**：自动重试失败的服务调用
- **降级策略**：服务不可用时的备选方案

### 安全性
- **认证授权**：基于 token 的服务认证
- **数据加密**：敏感数据传输加密
- **审计日志**：完整的调用链路记录

## 监控与维护

### 监控指标
- 服务可用性和响应时间
- 调用频率和错误率
- 资源使用情况

### 日志管理
- 结构化日志记录
- 分布式链路追踪
- 错误告警机制

---

此架构设计将使 GitAI 具备更好的可扩展性、可维护性和可测试性，同时保持高性能和稳定性。
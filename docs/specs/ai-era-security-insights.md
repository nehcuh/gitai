# GitAI AI时代安全洞察功能需求文档

## 1. 背景与问题

### 1.1 现有问题
- Tree-sitter功能过度复杂但价值有限：统计函数数量、类数量对代码评审没有实质帮助
- DevOps集成过于简单：偏离度分析只是从AI响应中提取数字，缺乏实际洞察
- 功能定位错误：从技术展示角度设计，而非解决AI时代的真实问题

### 1.2 AI时代的新挑战
- **架构漂移**：AI生成的代码可能不符合项目架构模式
- **需求偏离**：AI可能实现与需求不符的功能，或引入无关功能
- **安全边界**：AI可能引入意想不到的安全风险（如eval()、命令注入）
- **上下文丢失**：AI不了解项目的隐式约束和最佳实践

## 2. 目标用户

### 2.1 主要用户
- **开发者**：使用AI生成代码，需要快速识别潜在问题
- **团队负责人**：确保AI生成的代码符合项目标准
- **DevOps工程师**：在CI/CD流程中集成代码质量检查

### 2.2 使用场景
- 代码提交前的安全检查
- AI生成代码的质量验证
- Issue完成度的需求验证
- 项目架构一致性维护

## 3. 功能需求

### 3.1 核心功能

#### 3.1.1 架构一致性检查
**描述**：检测代码是否符合项目架构模式

**功能要求**：
- 分析代码的职责分离是否合理
- 检测模块间的耦合度
- 识别违反设计模式的代码
- 提供架构改进建议

**输出示例**：
```
🏗️ 架构一致性：UserService职责过于分散
💡 建议：将邮件发送功能提取到独立的EmailService中
🔥 严重程度：High
```

#### 3.1.2 需求符合度验证
**描述**：确保代码实现完全对应Issue需求

**功能要求**：
- 解析Issue描述的需求点
- 分析代码实现是否覆盖所有需求
- 检测是否引入了需求外的功能
- 计算需求覆盖率

**输出示例**：
```
📋 需求偏离：引入了社交登录功能（未在需求中定义）
💡 建议：移除社交登录功能，专注于核心需求
🔥 严重程度：Medium
📊 需求覆盖率：75%
```

#### 3.1.3 安全边界保护
**描述**：识别潜在的安全风险模式

**功能要求**：
- 检测危险函数调用（eval、exec等）
- 识别潜在的注入攻击风险
- 检查输入验证和输出编码
- 语言特定的安全规则

**输出示例**：
```
🛡️ 安全风险：第25行使用eval()函数
💡 建议：使用更安全的替代方案，如JSON.parse()
🔥 严重程度：Critical
```

#### 3.1.4 模式合规性检查
**描述**：验证代码遵循项目最佳实践

**功能要求**：
- 检查代码长度和复杂度
- 识别重复代码模式
- 检测TODO/FIXME标记
- 验证命名规范和注释质量

**输出示例**：
```
🔧 模式合规：函数过长（120行）
💡 建议：将长函数拆分为多个子函数
🔥 严重程度：Low
```

### 3.2 集成功能

#### 3.2.1 与现有Review命令集成
**描述**：在现有的review命令中增加安全洞察功能

**功能要求**：
- 新增 `--security-insights` 标志
- 与现有的 `--tree-sitter` 和 `--security-scan` 兼容
- 支持多种输出格式（text、json）
- 提供严重程度过滤选项

#### 3.2.2 与DevOps集成
**描述**：增强现有的DevOps集成，专注需求一致性

**功能要求**：
- 深度解析Issue需求和上下文
- 智能分析代码变更与需求的匹配度
- 提供具体的偏离度报告
- 支持多种DevOps平台（GitHub、Coding等）

**API集成规范**：
- Coding.net: 使用标准REST API，token认证
- GitHub: 使用标准GitHub API
- 统一的Issue数据结构映射
- 错误处理和重试机制

### 3.3 技术架构

#### 3.3.1 核心组件
- **SecurityInsights**：安全洞察分析器
- **SecurityReviewer**：安全评审执行器
- **InsightCategory**：洞察类别枚举
- **Severity**：严重程度枚举

#### 3.3.2 数据流
```
代码变更 → 语言检测 → 安全洞察分析 → 结果聚合 → 输出报告
Issue上下文 → 需求解析 → 需求符合度验证 ↗
```

#### 3.3.3 AI集成
- 使用现有的AI服务接口
- 专门的Tree-sitter代码洞察提示词设计
- 智能的结果解析和验证
- 失败时的优雅降级

#### 3.3.4 提示词工程设计
针对Tree-sitter代码洞察的专门提示词架构：

**配置化提示词系统**：
- 所有提示词模板通过配置文件管理，而非硬编码
- 支持用户自定义提示词模板
- 提供默认提示词模板作为起点
- 运行时动态加载提示词配置

**默认提示词模板示例**（存储在配置文件中）：

```yaml
# ~/.config/gitai/prompts/security-insights.yaml
architectural_analysis:
  role: "资深软件架构师"
  template: |
    你是一位{{role}}，专门分析代码架构问题。
    
    基于以下Tree-sitter解析的代码结构信息，分析代码的架构一致性：
    
    代码结构信息：
    - 语言: {{language}}
    - 函数数量: {{function_count}}
    - 类数量: {{class_count}}
    - 函数详情: {{function_details}}
    - 类详情: {{class_details}}
    - 依赖关系: {{dependencies}}
    
    完整代码：
    ```{{language}}
    {{code}}
    ```
    
    请从以下维度分析：
    1. 职责分离：每个类/模块是否职责单一
    2. 耦合度：模块间的依赖关系是否合理
    3. 内聚性：相关功能是否聚合在一起
    4. 设计模式：是否遵循了合适的设计模式
    5. 架构原则：是否符合SOLID、DRY等原则
    
    返回JSON格式的分析结果：
    {
      "issues": [
        {
          "category": "architectural_consistency",
          "severity": "critical|high|medium|low|info",
          "title": "问题描述",
          "description": "详细描述",
          "suggestion": "具体改进建议",
          "code_references": ["相关代码片段"]
        }
      ],
      "architecture_score": 0.0-1.0,
      "summary": "总体评估"
    }
  variables:
    - "language"
    - "function_count"
    - "class_count"
    - "function_details"
    - "class_details"
    - "dependencies"
    - "code"
  output_format: "json"

requirement_validation:
  role: "需求分析师"
  template: |
    你是一位{{role}}，专门验证代码实现是否符合需求。
    # ... (完整的模板内容)
  variables:
    - "issue_description"
    - "acceptance_criteria"
    - "language"
    - "code"
    - "implemented_functions"
    - "class_structure"
    - "key_features"
  output_format: "json"

security_analysis:
  role: "安全专家"
  template: |
    你是一位{{role}}，专门识别代码中的安全风险。
    # ... (完整的模板内容)
  variables:
    - "language"
    - "context"
    - "code"
    - "dangerous_calls"
    - "input_handling"
    - "output_generation"
    - "permission_operations"
  output_format: "json"

quality_analysis:
  role: "代码质量专家"
  template: |
    你是一位{{role}}，专门分析代码的最佳实践遵循情况。
    # ... (完整的模板内容)
  variables:
    - "language"
    - "project_type"
    - "function_count"
    - "avg_function_length"
    - "max_function_length"
    - "complexity_metrics"
    - "code"
  output_format: "json"
```

**提示词配置管理**：

**配置文件结构**：
```yaml
# ~/.config/gitai/config.toml
[prompts]
enabled = true
directory = "~/.config/gitai/prompts"
auto_reload = true
fallback_to_defaults = true

[prompts.templates]
architectural_analysis = "security-insights.yaml#architectural_analysis"
requirement_validation = "security-insights.yaml#requirement_validation"
security_analysis = "security-insights.yaml#security_analysis"
quality_analysis = "security-insights.yaml#quality_analysis"
```

**运行时加载机制**：
1. 启动时读取配置文件中的提示词路径
2. 解析YAML格式的提示词模板
3. 验证模板变量完整性
4. 缓存提示词模板以提高性能
5. 支持热重载（文件变更时自动重新加载）

**用户自定义支持**：
- 用户可以修改默认提示词模板
- 支持创建自定义提示词模板
- 提供模板验证和测试工具
- 保持向后兼容性

**安全边界保护提示词**：
```
你是一位安全专家，专门识别代码中的安全风险。

代码语言：{{language}}
代码上下文：{{context}}

完整代码：
```{{language}}
{{code}}
```

Tree-sitter解析信息：
- 危险函数调用：{{dangerous_calls}}
- 输入处理：{{input_handling}}
- 输出生成：{{output_generation}}
- 权限操作：{{permission_operations}}

请重点关注：
1. 注入攻击：SQL注入、命令注入、代码注入
2. XSS漏洞：跨站脚本攻击风险
3. 输入验证：用户输入是否充分验证
4. 权限控制：是否有越权操作
5. 数据泄露：敏感信息是否可能泄露

返回JSON格式：
{
  "security_findings": [
    {
      "category": "injection|xss|validation|authorization|data_leak",
      "severity": "critical|high|medium|low",
      "title": "安全问题标题",
      "description": "问题描述",
      "code_snippet": "问题代码",
      "line_numbers": [行号],
      "fix_suggestion": "修复建议",
      "risk_level": "影响程度"
    }
  ],
  "security_score": 0.0-1.0,
  "critical_issues_count": 0
}
```

**模式合规性检查提示词**：
```
你是一位代码质量专家，专门分析代码的最佳实践遵循情况。

代码语言：{{language}}
项目类型：{{project_type}}

代码结构信息：
- 函数数量：{{function_count}}
- 平均函数长度：{{avg_function_length}}
- 最大函数长度：{{max_function_length}}
- 复杂度指标：{{complexity_metrics}}

完整代码：
```{{language}}
{{code}}
```

请分析以下模式合规性：
1. 代码结构：函数长度、复杂度、嵌套层次
2. 命名规范：变量、函数、类命名是否合理
3. 注释质量：注释是否充分和准确
4. 错误处理：是否有完善的错误处理
5. 可维护性：代码是否易于理解和维护

返回JSON格式：
{
  "quality_issues": [
    {
      "category": "structure|naming|comments|error_handling|maintainability",
      "severity": "medium|low|info",
      "title": "质量问题",
      "description": "问题描述",
      "code_example": "示例代码",
      "best_practice": "最佳实践建议",
      "improvement_suggestion": "改进建议"
    }
  ],
  "quality_score": 0.0-1.0,
  "maintainability_index": "A|B|C|D"
}
```

**提示词工程原则**：
1. **结构化输出**：强制JSON格式，便于程序解析
2. **上下文丰富**：提供Tree-sitter解析的结构化信息
3. **任务专注**：每个提示词专注一个分析维度
4. **可量化结果**：提供评分和统计指标
5. **具体建议**：提供可操作的改进建议
6. **代码引用**：引用具体的代码片段，便于定位

#### 3.3.4 DevOps API处理
- 统一的Issue数据模型，支持多平台
- 自动数据脱敏处理，保护敏感信息
- API失败时的缓存和重试机制
- 错误信息的友好处理

**数据脱敏要求**：
- 移除或替换真实的token值
- 隐藏内部URL和路径信息
- 过滤用户个人信息
- 脱敏处理项目敏感数据

## 4. 非功能需求

### 4.1 性能要求
- 分析速度：单文件分析 < 5秒
- 内存使用：< 100MB
- 支持大文件（< 10MB）
- 缓存机制避免重复分析

### 4.2 可用性要求
- 清晰的错误信息和建议
- 多语言支持（优先级：Rust、Java、JavaScript、Python）
- 可配置的规则和严重程度
- 详细的文档和示例

### 4.3 可靠性要求
- 失败时不影响现有功能
- 网络问题时优雅降级
- 有效的错误处理和日志
- 完整的测试覆盖

## 5. 迁移策略

### 5.1 破坏性变更
- 移除现有的StructuralSummary输出
- 重新设计DeviationAnalysis结构
- 简化Tree-sitter查询系统
- 更新命令行接口

### 5.2 向后兼容
- 保持基本的review命令功能
- 提供配置选项保留旧行为
- 渐进式的迁移指南
- 充分的deprecated警告

## 6. 验收标准

### 6.1 功能验收
- [ ] 能够检测架构问题并给出建议
- [ ] 能够分析需求符合度并计算覆盖率
- [ ] 能够识别常见的安全风险模式
- [ ] 能够检查代码模式合规性
- [ ] 与现有review命令无缝集成

### 6.2 性能验收
- [ ] 单文件分析时间 < 5秒
- [ ] 内存使用 < 100MB
- [ ] 支持缓存和增量分析
- [ ] 并发分析能力

### 6.3 质量验收
- [ ] 完整的单元测试覆盖
- [ ] 集成测试覆盖主要场景
- [ ] 性能基准测试
- [ ] 用户文档和示例
- [ ] DevOps API集成测试（使用模拟数据）
- [ ] 数据脱敏功能测试
- [ ] 错误处理和降级测试

## 7. 风险评估

### 7.1 技术风险
- AI分析结果的质量和一致性
- 多语言支持的复杂度
- 性能和内存使用
- 与现有系统的集成
- 外部API依赖（Coding.net、GitHub等）的稳定性

### 7.2 数据安全风险
- DevOps平台token的安全管理
- 敏感项目信息的处理
- API响应数据的脱敏处理
- 错误信息中的敏感信息过滤

### 7.3 用户风险
- 用户习惯的改变
- 输出格式的变化
- 配置和使用的复杂度
- 文档和学习成本

### 7.4 缓解措施
- 渐进式发布和反馈收集
- 详细的迁移指南
- 充分的测试和验证
- 持续的性能优化
- API失败时的优雅降级
- 敏感信息的自动脱敏处理

## 8. 时间规划

### Phase 1：核心框架（1周）
- 创建SecurityInsights基础架构
- 实现架构一致性检查
- 基本的集成测试

### Phase 2：功能完善（1周）
- 实现需求符合度验证
- 添加安全边界保护
- 完善模式合规性检查

### Phase 3：集成优化（1周）
- 与现有review命令集成
- 性能优化和缓存
- 文档和示例完善

### Phase 4：测试发布（1周）
- 完整的测试覆盖
- 性能基准测试
- 用户文档和迁移指南
- 发布和反馈收集

## 9. 成功指标

### 9.1 技术指标
- 分析准确率 > 80%
- 假阳性率 < 20%
- 分析时间 < 5秒/文件
- 用户满意度 > 4.0/5.0

### 9.2 业务指标
- 减少AI生成代码的架构问题
- 提高需求实现的符合度
- 降低安全风险引入率
- 提升代码评审效率

---

**状态**：草稿
**版本**：1.0
**创建日期**：2025-08-25
**最后更新**：2025-08-25
**负责人**：GitAI Team
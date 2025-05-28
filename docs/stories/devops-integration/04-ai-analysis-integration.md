# 用户故事 04: AI 分析集成

## 故事概述
**作为一名开发者**
**我希望 AI 能够结合 DevOps 工作项描述和代码变更进行智能分析**
**这样我就能够获得代码实现与需求描述一致性的专业评估，确保代码变更符合业务需求**

## 详细描述

### 用户角色
- 开发工程师
- 技术负责人
- QA 工程师
- 产品经理

### 功能需求
集成 AI 分析引擎，提供代码与需求的智能对比分析：

1. 结合 Git diff 内容和 DevOps 工作项描述进行分析
2. 生成代码质量评审报告
3. 分析代码实现与需求描述的偏离度
4. 提供改进建议和风险评估
5. 支持多种输出格式（文本、JSON、Markdown）
6. 处理多个工作项的综合分析
7. 提供可配置的分析深度和关注点

### 分析维度

#### 需求一致性分析
- 功能实现完整性评估
- 业务逻辑正确性验证
- 用户故事验收标准对照
- 需求遗漏或超范围实现检测

#### 代码质量分析
- 代码结构和设计模式评估
- 性能和安全性分析
- 可维护性和可测试性评估
- 编码规范和最佳实践检查

#### 偏离度量化
- 实现与需求的匹配度评分（0-100）
- 关键偏离点详细说明
- 风险等级评估（低/中/高）
- 修复建议和优先级排序

### 使用场景

#### 场景 1: 单个用户故事分析
```bash
gitai review --space-id=726226 --stories=99
# AI 输出:
# ========== 代码评审报告 ==========
# 工作项: [用户故事] 封装 requests 函数到用户自定义函数
# 需求匹配度: 85/100
# 主要发现:
# ✅ 已实现核心功能：HTTP 请求封装
# ⚠️  缺少错误处理机制
# ❌ 未实现用户自定义头部配置
```

#### 场景 2: 多工作项综合分析
```bash
gitai review --space-id=726226 --stories=99,100 --tasks=201
# AI 生成包含所有工作项的综合评审报告
```

#### 场景 3: 特定关注点分析
```bash
gitai review --space-id=726226 --stories=99 --focus="安全性,性能"
# AI 重点关注安全性和性能相关的实现
```

#### 场景 4: 详细分析输出
```bash
gitai review --space-id=726226 --stories=99 --depth=deep --format=json --output=analysis.json
# 生成详细的 JSON 格式分析报告
```

## 验收标准

### 核心分析功能
- [ ] 成功结合 Git diff 和 DevOps 工作项数据
- [ ] 生成需求一致性分析报告
- [ ] 提供量化的偏离度评分
- [ ] 识别关键的实现缺陷和风险点
- [ ] 生成可执行的改进建议

### AI 提示词工程
- [ ] 设计专门的需求对比分析提示词
- [ ] 支持不同工作项类型的分析策略
- [ ] 处理多语言代码分析
- [ ] 适应不同项目规模和复杂度
- [ ] 提供上下文感知的分析

### 输出格式支持
- [ ] 标准文本格式：人性化的评审报告
- [ ] JSON 格式：结构化数据，便于工具集成
- [ ] Markdown 格式：便于文档化和分享
- [ ] 支持自定义输出模板
- [ ] 支持输出到文件或标准输出

### 多工作项处理
- [ ] 正确处理多个工作项的数据聚合
- [ ] 生成综合性的分析报告
- [ ] 识别工作项之间的关联性
- [ ] 处理工作项优先级和依赖关系
- [ ] 支持工作项类型混合分析

### 错误处理和鲁棒性
- [ ] 处理不完整的工作项描述
- [ ] 处理空的或无效的 Git diff
- [ ] 处理 AI API 调用失败
- [ ] 提供降级分析策略
- [ ] 生成部分分析结果

## 技术实现要求

### 数据结构定义
```rust
/// Represents a work item (e.g., User Story, Defect, Task) prepared for AI analysis.
/// This struct is populated by extracting specific fields from the DevOps API response
/// as defined in User Story 03, where the response structure is:
/// ```
/// {
///   "code": 0,
///   "msg": null,
///   "data": { ... } // serde_json::Value containing work item details
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkItem {
    /// DevOps work item unique identifier
    /// Source: DevOps API response `data.id`
    /// Example: 1255140 (用户故事), 1455437 (缺陷), 1323273 (任务)
    pub id: Option<u32>,
    
    /// DevOps work item code/number used for reference
    /// Source: DevOps API response `data.code`
    /// Example: 99 (用户故事), 833118 (缺陷), 655911 (任务)
    pub code: Option<u32>,
    
    /// Project/Product context name for AI to understand business domain
    /// Source: DevOps API response `data.program.display_name`
    /// Example: "金科中心代码扫描引擎项目预研" (用户故事)
    ///          "T7.6券结(含券结ETF)融资行权业务回归及单客户上线" (缺陷)
    ///          null or missing (任务 - handle gracefully)
    pub project_name: Option<String>,
    
    /// Human-readable work item type for AI analysis strategy differentiation
    /// Source: DevOps API response `data.issueTypeDetail.name`
    /// Expected values: "用户故事", "缺陷", "任务"
    /// This field helps AI apply different analysis approaches for different work item types
    pub item_type_name: Option<String>,
    
    /// Work item title/summary - the main subject of the work item
    /// Source: DevOps API response `data.name`
    /// Example: "封装 requests 函数到用户自定义函数" (用户故事)
    ///          "交易运营部-公募T0账单-资金信息汇总未统计理财持仓市值。" (缺陷)
    ///          "交易网关9502超时优化" (任务)
    pub title: Option<String>,
    
    /// Detailed work item description - contains requirements, acceptance criteria, etc.
    /// Source: DevOps API response `data.description`
    /// This is the primary content AI will use to understand requirements and compare against code
    /// May contain markdown formatting, images, and structured content
    pub description: Option<String>,
    
    /// Optional: Raw DevOps work item type for fine-grained AI analysis strategy
    /// Source: DevOps API response `data.type`
    /// Values: "REQUIREMENT", "DEFECT", "MISSION"
    /// Uncomment if AI needs to differentiate analysis based on DevOps internal categorization
    // pub devops_raw_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisRequest {
    /// Collection of work items to be analyzed together.
    /// Each WorkItem is constructed by processing the DevOps API response as detailed in User Story 03.
    /// 
    /// Construction process:
    /// 1. Call DevOps API to get response: `{"code": 0, "msg": null, "data": {...}}`
    /// 2. Parse `data` field as `serde_json::Value` (since structure varies by work item type)
    /// 3. Extract fields to populate WorkItem struct:
    ///    - `WorkItem.id` ← `data["id"].as_u64()`
    ///    - `WorkItem.code` ← `data["code"].as_u64()`
    ///    - `WorkItem.project_name` ← `data["program"]["display_name"].as_str()`
    ///    - `WorkItem.item_type_name` ← `data["issueTypeDetail"]["name"].as_str()`
    ///    - `WorkItem.title` ← `data["name"].as_str()`
    ///    - `WorkItem.description` ← `data["description"].as_str()`
    /// 
    /// Note: Handle missing fields gracefully using Option types, as some fields
    /// may be null or absent in certain work item types (e.g., program.display_name in tasks).
    pub work_items: Vec<WorkItem>,
    pub git_diff: String,
    pub focus_areas: Option<Vec<String>>,
    pub analysis_depth: AnalysisDepth, // Assuming AnalysisDepth enum is defined elsewhere
    pub output_format: OutputFormat,   // Assuming OutputFormat enum is defined elsewhere
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub overall_score: u8,           // 0-100
    pub requirement_consistency: RequirementAnalysis,
    pub code_quality: CodeQualityAnalysis,
    pub deviations: Vec<Deviation>,
    pub recommendations: Vec<Recommendation>,
    pub risk_assessment: RiskAssessment,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequirementAnalysis {
    pub completion_score: u8,        // 0-100
    pub accuracy_score: u8,          // 0-100
    pub missing_features: Vec<String>,
    pub extra_implementations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Deviation {
    pub severity: DeviationSeverity,
    pub category: String,
    pub description: String,
    pub file_location: Option<String>,
    pub suggestion: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DeviationSeverity {
    Low,
    Medium,
    High,
    Critical,
}
```

### AI 分析引擎实现
```rust
pub struct AIAnalysisEngine {
    config: Arc<AppConfig>,
    ai_client: Arc<dyn AIClient>,
}

impl AIAnalysisEngine {
    pub async fn analyze_with_requirements(
        &self,
        request: AnalysisRequest,
    ) -> Result<AnalysisResult, AnalysisError> {
        // 构造增强的分析提示词
        let enhanced_prompt = self.build_requirement_analysis_prompt(&request)?;
        
        // 调用 AI 进行分析
        let ai_response = self.ai_client
            .execute_analysis_request(&enhanced_prompt)
            .await?;
        
        // 解析并结构化 AI 响应
        self.parse_analysis_response(&ai_response)
    }
    
    fn build_requirement_analysis_prompt(&self, request: &AnalysisRequest) -> Result<String, AnalysisError> {
        // 构造包含工作项描述和代码差异的提示词
    }
    
    fn parse_analysis_response(&self, response: &str) -> Result<AnalysisResult, AnalysisError> {
        // 解析 AI 响应，提取结构化分析结果
    }
}
```

### 提示词模板设计
```rust
const REQUIREMENT_ANALYSIS_TEMPLATE: &str = r#"
你是一位资深的代码评审专家和需求分析师。请分析以下代码变更与业务需求的一致性。

## 工作项信息
{work_items_description}

## 代码变更
```diff
{git_diff}
```

## 分析要求
请从以下维度进行详细分析：

1. **需求实现完整性**：
   - 代码是否完整实现了所有需求功能
   - 是否存在需求遗漏或功能缺失
   - 实现是否超出了需求范围

2. **业务逻辑正确性**：
   - 代码逻辑是否符合业务规则
   - 边界条件和异常情况处理是否恰当
   - 用户体验是否符合预期

3. **技术实现质量**：
   - 代码结构和设计是否合理
   - 性能和安全性考虑是否充分
   - 可维护性和扩展性评估

4. **偏离度分析**：
   - 量化实现与需求的匹配程度（0-100分）
   - 识别主要的偏离点和风险
   - 提供具体的改进建议

请以结构化的方式输出分析结果，包括评分、发现的问题、建议等。
"#;
```

### 输出格式化器
```rust
pub struct OutputFormatter;

impl OutputFormatter {
    pub fn format_analysis_result(
        result: &AnalysisResult,
        format: OutputFormat,
        template: Option<&str>,
    ) -> Result<String, FormattingError> {
        match format {
            OutputFormat::Text => Self::format_as_text(result),
            OutputFormat::Json => Self::format_as_json(result),
            OutputFormat::Markdown => Self::format_as_markdown(result),
            OutputFormat::Custom(template) => Self::format_with_template(result, template),
        }
    }
    
    fn format_as_text(result: &AnalysisResult) -> Result<String, FormattingError> {
        // 生成人性化的文本报告
    }
    
    fn format_as_markdown(result: &AnalysisResult) -> Result<String, FormattingError> {
        // 生成 Markdown 格式报告
    }
}
```

## 性能要求

### 分析效率
- [ ] 单个工作项分析：< 15秒
- [ ] 多个工作项分析：< 30秒
- [ ] 大型代码变更分析：< 60秒
- [ ] 支持分析结果缓存

### 资源使用
- [ ] 内存使用优化，处理大型 diff
- [ ] 支持流式处理大型分析结果
- [ ] AI API 调用频率控制
- [ ] 并发分析任务管理

## 质量要求

### 分析准确性
- [ ] 需求匹配度评估准确率 > 85%
- [ ] 关键问题识别率 > 90%
- [ ] 误报率 < 10%
- [ ] 建议实用性评分 > 80%

### 用户体验
- [ ] 分析报告易读易懂
- [ ] 提供可执行的建议
- [ ] 支持不同技术水平的用户
- [ ] 提供分析过程的透明度

## 优先级
**高优先级** - 这是整个 DevOps 集成功能的核心价值体现。

## 估算工作量
- AI 提示词设计和优化：2天
- 分析引擎核心实现：3天
- 多格式输出支持：1天
- 错误处理和鲁棒性：1天
- 性能优化：1天
- 单元测试和集成测试：2天
- 文档和示例：1天

## 依赖关系
- 依赖：用户故事 03 (DevOps API 集成)
- 被依赖：用户故事 05 (输出格式化优化)

## 测试用例

### 功能测试
1. 测试单工作项分析功能
2. 测试多工作项综合分析
3. 测试不同工作项类型的分析
4. 测试特定关注点分析
5. 测试不同分析深度的效果

### 准确性测试
1. 使用已知的正确实现进行验证
2. 使用已知的偏离实现进行验证
3. 测试边界条件和异常情况
4. 对比人工评审结果验证准确性

### 性能测试
1. 测试大型代码变更的分析性能
2. 测试多工作项并发分析性能
3. 测试 AI API 调用优化效果
4. 测试内存使用情况

### 鲁棒性测试
1. 测试不完整工作项描述的处理
2. 测试无效 Git diff 的处理
3. 测试 AI API 失败的降级处理
4. 测试各种异常情况的错误处理

## 完成定义 (Definition of Done)
- [ ] 代码实现完成并通过代码评审
- [ ] 单元测试覆盖率达到 90% 以上
- [ ] 集成测试通过
- [ ] 准确性测试达到质量要求
- [ ] 性能测试满足要求
- [ ] 用户体验测试通过
- [ ] AI 提示词优化完成
- [ ] 输出格式文档更新完成
- [ ] 功能演示通过产品验收
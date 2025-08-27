# GitAI Tree-sitter 代码洞察提示词工程设计

## 设计原则

### 1. 提示词工程核心理念
基于Claude和Google的提示词工程最佳实践，我们的设计遵循以下原则：

**Claude最佳实践应用**：
- **角色扮演**：为每个分析维度设定专业角色（架构师、安全专家等）
- **结构化输出**：强制JSON格式，便于程序解析
- **上下文丰富**：提供Tree-sitter解析的结构化信息
- **任务分解**：每个提示词专注一个分析维度

**Google提示词工程原则应用**：
- **具体明确**：明确的任务定义和期望输出
- **示例驱动**：提供具体的分析维度和标准
- **迭代优化**：基于结果反馈不断优化提示词
- **安全考虑**：避免提示词注入和敏感信息泄露

### 2. Tree-sitter数据整合策略

**数据提取层次**：
```
原始代码 → Tree-sitter解析 → 结构化数据 → 提示词变量 → AI分析
```

**关键数据映射**：
- **架构分析**：函数数量、类数量、依赖关系、继承关系
- **需求验证**：实现的功能点、类结构、函数签名
- **安全分析**：危险函数调用、输入输出处理、权限操作
- **质量分析**：代码长度、复杂度、命名规范、注释情况

## 提示词架构设计

### 1. 配置化模板系统架构

**核心设计原则**：
- 所有提示词模板通过外部配置文件管理
- 运行时动态加载，避免硬编码
- 支持用户自定义和扩展
- 提供默认模板作为起点

**配置驱动的架构**：
```rust
pub struct PromptConfig {
    pub enabled: bool,
    pub directory: PathBuf,
    pub auto_reload: bool,
    pub fallback_to_defaults: bool,
    pub templates: HashMap<String, TemplateReference>,
}

pub struct TemplateReference {
    pub file: String,
    pub template_name: String,
}

pub struct PromptTemplate {
    pub name: String,
    pub role: String,
    pub template: String,
    pub variables: Vec<String>,
    pub output_format: String,
    pub language: Option<SupportedLanguage>,
    pub category: InsightCategory,
}

pub struct PromptEngine {
    config: PromptConfig,
    templates: HashMap<String, PromptTemplate>,
    renderer: TemplateRenderer,
    config_watcher: Option<ConfigWatcher>,
}
```

### 2. 配置加载和管理

**配置文件结构**：
```yaml
# ~/.config/gitai/prompts/security-insights.yaml
version: "1.0"
description: "AI时代安全洞察提示词模板"

templates:
  architectural_analysis:
    role: "资深软件架构师"
    description: "分析代码架构一致性和设计模式"
    template: |
      你是一位{{role}}，专门分析代码架构问题。
      # ... (完整模板内容)
    variables:
      - "language"
      - "function_count"
      - "class_count"
      - "function_details"
      - "class_details"
      - "dependencies"
      - "code"
    output_format: "json"
    supported_languages: ["rust", "java", "javascript", "python", "go", "c", "cpp"]
    
  requirement_validation:
    role: "需求分析师"
    description: "验证代码实现是否符合Issue需求"
    template: |
      你是一位{{role}}，专门验证代码实现是否符合需求。
      # ... (完整模板内容)
    variables:
      - "issue_description"
      - "acceptance_criteria"
      - "language"
      - "code"
      - "implemented_functions"
      - "class_structure"
      - "key_features"
    output_format: "json"
    supported_languages: ["rust", "java", "javascript", "python", "go", "c", "cpp"]
```

**运行时配置加载**：
```rust
impl PromptEngine {
    pub async fn new(config: PromptConfig) -> Result<Self> {
        let mut engine = Self {
            config: config.clone(),
            templates: HashMap::new(),
            renderer: TemplateRenderer::new(),
            config_watcher: None,
        };
        
        // 加载配置的提示词模板
        engine.load_templates().await?;
        
        // 启动配置文件监听器（如果启用）
        if config.auto_reload {
            engine.start_config_watcher().await?;
        }
        
        Ok(engine)
    }
    
    async fn load_templates(&mut self) -> Result<()> {
        if !self.config.enabled {
            log::info!("提示词系统已禁用");
            return Ok(());
        }
        
        // 加载所有配置的模板文件
        for (name, template_ref) in &self.config.templates {
            match self.load_template_from_file(&template_ref.file, &template_ref.template_name).await {
                Ok(template) => {
                    self.templates.insert(name.clone(), template);
                    log::debug!("成功加载提示词模板: {}", name);
                }
                Err(e) => {
                    log::warn!("加载提示词模板失败 {}: {}", name, e);
                    if self.config.fallback_to_defaults {
                        self.templates.insert(name.clone(), Self::get_default_template(name));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn load_template_from_file(&self, file_path: &str, template_name: &str) -> Result<PromptTemplate> {
        let full_path = self.config.directory.join(file_path);
        let content = tokio::fs::read_to_string(&full_path).await?;
        
        let yaml_config: YamlPromptConfig = serde_yaml::from_str(&content)?;
        let template_def = yaml_config.templates.get(template_name)
            .ok_or_else(|| format!("模板 {} 未找到", template_name))?;
        
        Ok(PromptTemplate {
            name: template_name.to_string(),
            role: template_def.role.clone(),
            template: template_def.template.clone(),
            variables: template_def.variables.clone(),
            output_format: template_def.output_format.clone(),
            language: None,
            category: self.map_category(template_name),
        })
    }
    
    fn get_default_template(name: &str) -> PromptTemplate {
        // 返回硬编码的默认模板作为fallback
        match name {
            "architectural_analysis" => Self::default_architectural_template(),
            "requirement_validation" => Self::default_requirement_template(),
            "security_analysis" => Self::default_security_template(),
            "quality_analysis" => Self::default_quality_template(),
            _ => PromptTemplate {
                name: name.to_string(),
                role: "通用分析师".to_string(),
                template: "请分析以下代码：\n{{code}}".to_string(),
                variables: vec!["code".to_string()],
                output_format: "json".to_string(),
                language: None,
                category: InsightCategory::PatternCompliance,
            }
        }
    }
}
```

### 3. 变量替换和渲染系统

**变量定义规范**：
- `{{language}}` - 编程语言
- `{{code}}` - 完整代码内容
- `{{function_count}}` - 函数数量
- `{{class_count}}` - 类数量
- `{{function_details}}` - 函数详细信息
- `{{class_details}}` - 类详细信息
- `{{dependencies}}` - 依赖关系信息
- `{{issue_description}}` - Issue描述
- `{{acceptance_criteria}}` - 验收标准

**智能渲染逻辑**：
```rust
impl PromptEngine {
    pub fn render(&self, template_name: &str, context: &HashMap<String, String>) -> Result<String> {
        let template = self.templates.get(template_name)
            .ok_or_else(|| format!("模板 {} 未找到", template_name))?;
        
        // 验证必需变量
        self.validate_variables(template, context)?;
        
        // 执行变量替换
        let mut rendered = template.template.clone();
        
        for (key, value) in context {
            let var_name = format!("{{{{{}}}}}", key);
            rendered = rendered.replace(&var_name, value);
        }
        
        // 移除未替换的变量（使用默认值）
        rendered = self.handle_missing_variables(&rendered);
        
        Ok(rendered)
    }
    
    fn validate_variables(&self, template: &PromptTemplate, context: &HashMap<String, String>) -> Result<()> {
        for var in &template.variables {
            if !context.contains_key(var) {
                return Err(format!("缺少必需变量: {}", var));
            }
        }
        Ok(())
    }
    
    fn handle_missing_variables(&self, content: &str) -> String {
        let re = regex::Regex::new(r"\{\{([^}]+)\}\}").unwrap();
        re.replace_all(content, |caps: &regex::Captures| {
            let var_name = &caps[1];
            match var_name {
                "language" => "未知语言",
                "function_count" => "0",
                "class_count" => "0",
                _ => "N/A"
            }
        }).to_string()
    }
}
```

### 3. 结果解析机制

**JSON Schema验证**：
```rust
#[derive(Deserialize, Validate)]
pub struct AnalysisResult {
    pub issues: Vec<SecurityInsight>,
    pub score: f32,
    pub summary: String,
}

#[derive(Deserialize, Validate)]
pub struct SecurityInsight {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    
    #[validate(length(min = 1))]
    pub description: String,
    
    #[validate(custom = "validate_severity")]
    pub severity: String,
    
    #[validate(length(min = 1))]
    pub suggestion: String,
}
```

## 具体提示词设计

### 1. 架构一致性分析提示词

**角色定位**：资深软件架构师
**分析重点**：SOLID原则、设计模式、架构合理性

**优化策略**：
- 提供具体的架构原则定义
- 包含正反示例说明
- 强调可量化的评估标准

**提示词增强**：
```
你是一位拥有15年经验的资深软件架构师，专精于SOLID原则和设计模式。

基于以下Tree-sitter解析的代码结构信息，进行架构一致性分析：

**架构原则参考**：
- 单一职责原则(SRP)：一个类应该只有一个改变的理由
- 开放封闭原则(OCP)：对扩展开放，对修改封闭
- 里氏替换原则(LSP)：子类必须能够替换其基类
- 接口隔离原则(ISP)：不应该强迫客户端依赖它们不使用的方法
- 依赖倒置原则(DIP)：高层模块不应该依赖低层模块

**代码结构数据**：
- 语言: {{language}}
- 函数数量: {{function_count}}
- 类数量: {{class_count}}
- 平均函数长度: {{avg_function_length}}
- 最大函数长度: {{max_function_length}}
- 继承关系: {{inheritance_relations}}
- 依赖关系: {{dependency_relations}}

**函数详情**：
{{function_details}}

**类详情**：
{{class_details}}

**完整代码**：
```{{language}}
{{code}}
```

**分析要求**：
1. 识别违反SOLID原则的地方
2. 评估类的职责是否单一
3. 检查模块间的耦合度
4. 识别可以应用的设计模式
5. 提供具体的重构建议

**输出格式**（必须严格按照JSON格式）：
{
  "architectural_issues": [
    {
      "principle": "SRP|OCP|LSP|ISP|DIP",
      "severity": "critical|high|medium|low",
      "title": "问题标题",
      "description": "详细问题描述",
      "code_references": ["相关代码片段"],
      "impact": "对系统的影响",
      "refactoring_suggestion": "具体的重构建议",
      "best_practice": "应该遵循的最佳实践"
    }
  ],
  "architecture_metrics": {
    "cohesion_score": 0.0-1.0,
    "coupling_score": 0.0-1.0,
    "abstraction_score": 0.0-1.0,
    "overall_score": 0.0-1.0
  },
  "summary": "总体架构评估"
}
```
```

### 2. 需求符合度验证提示词

**角色定位**：资深需求分析师和QA专家
**分析重点**：需求覆盖率、功能偏离、实现质量

**优化策略**：
- 明确需求分解和映射
- 提供具体的验证标准
- 强调风险识别和影响评估

### 3. 安全边界保护提示词

**角色定位**：安全专家和渗透测试工程师
**分析重点**：常见安全漏洞、输入验证、权限控制

**优化策略**：
- 基于OWASP Top 10安全风险
- 提供具体的攻击场景
- 强调实际风险和修复建议

### 4. 模式合规性检查提示词

**角色定位**：代码质量专家和资深开发者
**分析重点**：代码规范、最佳实践、可维护性

**优化策略**：
- 基于Clean Code原则
- 提供具体的质量指标
- 强调可操作性的改进建议

## 错误处理和降级策略

### 1. 提示词渲染错误
```rust
pub enum PromptError {
    TemplateNotFound(String),
    VariableMissing(String),
    RenderError(String),
}

impl PromptEngine {
    pub fn render_with_fallback(&self, template_name: &str, context: &HashMap<String, String>) -> Result<String> {
        match self.render(template_name, context) {
            Ok(prompt) => Ok(prompt),
            Err(PromptError::VariableMissing(var)) => {
                // 使用默认值或移除变量
                let mut fallback_context = context.clone();
                fallback_context.insert(var, "N/A".to_string());
                self.render(template_name, &fallback_context)
            }
            Err(e) => Err(e),
        }
    }
}
```

### 2. AI响应解析错误
```rust
pub fn parse_ai_response(response: &str) -> Result<AnalysisResult> {
    // 尝试解析JSON
    if let Ok(result) = serde_json::from_str::<AnalysisResult>(response) {
        return Ok(result);
    }
    
    // 尝试提取JSON部分
    let json_start = response.find('{');
    let json_end = response.rfind('}');
    
    if let (Some(start), Some(end)) = (json_start, json_end) {
        let json_str = &response[start..=end];
        if let Ok(result) = serde_json::from_str::<AnalysisResult>(json_str) {
            return Ok(result);
        }
    }
    
    // 降级为文本分析
    Ok(AnalysisResult::fallback(response))
}
```

### 3. 性能优化策略

**缓存机制**：
- 提示词模板缓存
- AI响应结果缓存
- 解析结果缓存

**并发处理**：
- 多文件并行分析
- 异步AI调用
- 批量结果处理

## 测试策略

### 1. 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_rendering() {
        let engine = PromptEngine::new();
        let mut context = HashMap::new();
        context.insert("language".to_string(), "Rust".to_string());
        context.insert("code".to_string(), "fn main() {}".to_string());
        
        let result = engine.render("architectural_analysis", &context);
        assert!(result.is_ok());
        let prompt = result.unwrap();
        assert!(prompt.contains("Rust"));
        assert!(prompt.contains("fn main() {}"));
    }
    
    #[test]
    fn test_json_parsing() {
        let mock_response = r#"
        {
            "architectural_issues": [],
            "architecture_metrics": {
                "cohesion_score": 0.8,
                "coupling_score": 0.7,
                "abstraction_score": 0.9,
                "overall_score": 0.8
            },
            "summary": "Good architecture"
        }
        "#;
        
        let result = parse_ai_response(mock_response);
        assert!(result.is_ok());
    }
}
```

### 2. 集成测试
- 真实AI服务调用测试
- 多种编程语言测试
- 错误场景覆盖测试

### 3. 性能测试
- 提示词渲染性能
- AI响应解析性能
- 内存使用测试

## 持续优化

### 1. 提示词优化循环
1. 收集分析结果质量反馈
2. 识别常见问题和错误
3. 调整提示词内容和结构
4. 验证改进效果

### 2. 数据驱动优化
- 统计分析准确率
- 收集用户反馈
- 监控AI服务质量
- 优化变量选择策略

### 3. 多模型适配
- 支持多种AI服务提供商
- 模型特定的提示词优化
- 成本和性能平衡

---

**创建日期**: 2025-08-25
**版本**: 1.0
**状态**: 设计阶段
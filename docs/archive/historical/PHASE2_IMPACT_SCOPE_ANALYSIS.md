# Phase 2: 智能影响范围分析

## 目标
基于依赖图分析代码变更的传播影响，让 AI 理解变更的连锁反应和影响范围。

## 核心功能
1. **依赖图构建** - 分析代码间的调用关系和依赖关系
2. **影响传播分析** - 计算变更如何通过依赖链传播
3. **影响半径计算** - 量化变更的影响范围
4. **级联效应识别** - 发现潜在的连锁反应

## 实现计划

### 2.1 依赖图数据结构 (Day 1)

#### 任务 2.1.1: 创建依赖图模块
```rust
// src/architectural_impact/dependency_graph.rs
pub struct DependencyGraph {
    nodes: HashMap<String, Node>,
    edges: Vec<Edge>,
}

pub struct Node {
    id: String,
    node_type: NodeType,
    metadata: NodeMetadata,
}

pub enum NodeType {
    Function(FunctionNode),
    Class(ClassNode),
    Module(ModuleNode),
    File(FileNode),
}

pub struct Edge {
    from: String,
    to: String,
    edge_type: EdgeType,
    weight: f32,
}

pub enum EdgeType {
    Calls,           // 函数调用
    Imports,         // 导入依赖
    Inherits,        // 继承关系
    Implements,      // 实现关系
    Uses,            // 使用关系
    References,      // 引用关系
}
```

#### 任务 2.1.2: 实现图构建算法
- 从 StructuralSummary 构建依赖图
- 解析函数调用关系
- 解析模块导入关系
- 解析类继承关系

### 2.2 影响传播算法 (Day 2)

#### 任务 2.2.1: 实现影响传播计算
```rust
// src/architectural_impact/impact_propagation.rs
pub struct ImpactPropagation {
    graph: DependencyGraph,
    impact_scores: HashMap<String, f32>,
}

impl ImpactPropagation {
    /// 计算从变更节点开始的影响传播
    pub fn calculate_impact(
        &mut self,
        changed_nodes: Vec<String>,
        max_depth: usize,
    ) -> ImpactScope {
        // 使用 BFS/DFS 遍历依赖图
        // 计算每个节点的影响分数
        // 考虑传播衰减因子
    }
    
    /// 计算影响半径
    pub fn calculate_radius(&self) -> f32 {
        // 基于影响节点数量和权重计算
    }
}
```

#### 任务 2.2.2: 实现传播规则引擎
```rust
pub struct PropagationRules {
    rules: Vec<Rule>,
}

pub struct Rule {
    condition: RuleCondition,
    impact_factor: f32,
    propagation_type: PropagationType,
}

pub enum PropagationType {
    Direct,      // 直接影响
    Transitive,  // 传递影响
    Conditional, // 条件影响
    None,        // 不传播
}
```

### 2.3 影响范围可视化 (Day 3)

#### 任务 2.3.1: 实现影响范围报告
```rust
// src/architectural_impact/impact_scope.rs
pub struct ImpactScope {
    /// 直接影响的组件
    pub direct_impacts: Vec<ImpactedComponent>,
    /// 间接影响的组件
    pub indirect_impacts: Vec<ImpactedComponent>,
    /// 影响半径（0-1）
    pub impact_radius: f32,
    /// 影响深度（传播层数）
    pub impact_depth: usize,
    /// 关键路径
    pub critical_paths: Vec<ImpactPath>,
}

pub struct ImpactedComponent {
    pub component_id: String,
    pub component_type: ComponentType,
    pub impact_score: f32,
    pub impact_reason: String,
    pub distance_from_change: usize,
}
```

#### 任务 2.3.2: 生成 AI 友好的影响报告
```markdown
## 影响范围分析

### 📊 影响统计
- 影响半径: 0.75 (高)
- 影响深度: 3 层
- 直接影响: 5 个组件
- 间接影响: 12 个组件

### 🎯 直接影响组件
1. `UserService::authenticate()` - 调用了被修改的函数
2. `AuthController::login()` - 依赖变更的接口

### 🌊 影响传播路径
```
parse_config() [变更]
  └─> ConfigManager::load() [直接影响]
      └─> ApplicationContext::init() [间接影响]
          └─> MainApplication::start() [间接影响]
```

### ⚠️ 高风险影响
- 认证模块可能受影响，建议重点测试
- 数据库连接池配置可能需要调整
```

### 2.4 级联效应检测 (Day 4)

#### 任务 2.4.1: 实现级联效应检测器
```rust
// src/architectural_impact/cascade_detector.rs
pub struct CascadeDetector {
    graph: DependencyGraph,
    thresholds: CascadeThresholds,
}

pub struct CascadeEffect {
    pub trigger: String,
    pub affected_chain: Vec<String>,
    pub probability: f32,
    pub severity: Severity,
    pub description: String,
}

impl CascadeDetector {
    /// 检测潜在的级联效应
    pub fn detect_cascades(
        &self,
        changes: &[BreakingChange],
    ) -> Vec<CascadeEffect> {
        // 分析强依赖链
        // 识别单点故障
        // 计算级联概率
    }
    
    /// 识别系统中的关键节点
    pub fn identify_critical_nodes(&self) -> Vec<CriticalNode> {
        // 计算节点中心性
        // 识别高扇出节点
        // 识别瓶颈节点
    }
}
```

### 2.5 集成到现有系统 (Day 5)

#### 任务 2.5.1: 修改 OperationContext
```rust
// 在 OperationContext 中添加
pub struct OperationContext {
    // ... 现有字段
    pub dependency_graph: Option<DependencyGraph>,
    pub impact_scope: Option<ImpactScope>,
    pub cascade_effects: Vec<CascadeEffect>,
}
```

#### 任务 2.5.2: 更新 review.rs
```rust
// 在架构影响分析后添加影响范围分析
if let Some(impact) = architectural_impact {
    let graph = build_dependency_graph(&structural_summary)?;
    let scope = analyze_impact_scope(&graph, &impact)?;
    let cascades = detect_cascade_effects(&graph, &impact)?;
    
    context = context
        .with_dependency_graph(graph)
        .with_impact_scope(scope)
        .with_cascade_effects(cascades);
}
```

## 技术挑战与解决方案

### 挑战 1: 准确构建依赖图
**问题**: 从 AST 中准确提取所有依赖关系
**解决方案**: 
- 使用 Tree-sitter 的查询能力
- 增量构建依赖图
- 处理动态依赖和运行时依赖

### 挑战 2: 影响传播的准确性
**问题**: 如何准确计算影响传播
**解决方案**:
- 使用加权图算法
- 引入衰减因子
- 考虑不同类型依赖的传播特性

### 挑战 3: 性能优化
**问题**: 大型项目的依赖图可能很大
**解决方案**:
- 使用稀疏图表示
- 实现增量更新
- 缓存计算结果

## 成功标准

### 功能标准
- [ ] 能够构建准确的项目依赖图
- [ ] 能够计算变更的影响范围
- [ ] 能够识别潜在的级联效应
- [ ] 生成清晰的影响范围报告

### 性能标准
- [ ] 中型项目（~10K LOC）分析时间 < 5秒
- [ ] 大型项目（~100K LOC）分析时间 < 30秒
- [ ] 内存使用合理（< 500MB）

### 质量标准
- [ ] 单元测试覆盖率 > 80%
- [ ] 影响分析准确率 > 85%
- [ ] 无误报的关键路径识别

## 实现优先级

### 🚀 立即开始
1. 依赖图数据结构设计
2. 基础图构建算法

### 📅 本周完成
1. 影响传播算法
2. 基本的影响范围报告
3. 与现有系统集成

### 🎯 后续优化
1. 高级级联效应检测
2. 可视化改进
3. 性能优化

## 预期效果示例

### 输入：函数签名变更
```rust
// 变更前
fn process_data(input: String) -> Result<Data>

// 变更后  
fn process_data(input: String, options: ProcessOptions) -> Result<Data>
```

### 输出：影响范围分析
```
影响范围分析：
- 直接影响：3个调用方需要更新
  - DataController::handle_request()
  - BatchProcessor::run()
  - TestHelper::setup_data()
  
- 间接影响：7个组件可能受影响
  - API层：2个端点可能需要调整
  - 服务层：3个服务依赖此函数
  - 测试：2个集成测试需要更新

- 级联风险：中等
  - BatchProcessor 是关键路径组件
  - 可能影响数据处理管道
  
建议：
1. 优先更新 BatchProcessor
2. 添加向后兼容的重载函数
3. 分阶段迁移调用方
```

## 下一步行动

### 今天的任务
1. 创建 `dependency_graph.rs` 模块
2. 实现基础的图数据结构
3. 设计依赖关系提取算法

### 明天的任务
1. 实现影响传播算法
2. 创建影响范围计算逻辑
3. 开始集成测试

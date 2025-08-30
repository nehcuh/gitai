// Project Insights - 项目级架构洞察
//
// 基于 Tree-sitter 提供的 AST，生成项目级别的架构洞察
// 而不是简单的函数/类统计

use crate::tree_sitter::StructuralSummary;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// 项目洞察 - 提供架构级别的理解
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInsights {
    /// 架构层面的洞察
    pub architecture: ArchitecturalInsights,
    /// 变更影响分析
    pub impact_analysis: ImpactAnalysis,
    /// API 表面分析
    pub api_surface: ApiSurface,
    /// 代码质量热点
    pub quality_hotspots: QualityHotspots,
}

/// 架构洞察
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalInsights {
    /// 模块依赖图
    pub module_dependencies: DependencyGraph,
    /// 架构模式违规
    pub pattern_violations: Vec<PatternViolation>,
    /// 架构层次
    pub architectural_layers: Vec<ArchitecturalLayer>,
    /// 耦合度分析
    pub coupling_analysis: CouplingAnalysis,
}

/// 依赖关系图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// 节点（模块/类）
    pub nodes: Vec<DependencyNode>,
    /// 边（依赖关系）
    pub edges: Vec<DependencyEdge>,
    /// 循环依赖
    pub circular_dependencies: Vec<CircularDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Module,
    Class,
    Interface,
    Function,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub dependency_type: DependencyType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Import,
    Inheritance,
    Composition,
    MethodCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependency {
    pub cycle: Vec<String>,
    pub severity: Severity,
}

/// 架构模式违规
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternViolation {
    pub pattern_type: ArchitecturalPattern,
    pub violation_description: String,
    pub location: String,
    pub severity: Severity,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchitecturalPattern {
    SingleResponsibility,
    OpenClosed,
    LiskovSubstitution,
    InterfaceSegregation,
    DependencyInversion,
    LayerViolation,
    CircularDependency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// 架构层次
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalLayer {
    pub name: String,
    pub components: Vec<String>,
    pub allowed_dependencies: Vec<String>,
    pub violations: Vec<LayerViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerViolation {
    pub from_layer: String,
    pub to_layer: String,
    pub violating_component: String,
}

/// 耦合度分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingAnalysis {
    pub average_coupling: f64,
    pub highly_coupled_components: Vec<CoupledComponent>,
    pub loosely_coupled_components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoupledComponent {
    pub component: String,
    pub coupling_score: f64,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
}

/// 影响分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    /// 破坏性变更
    pub breaking_changes: Vec<BreakingChange>,
    /// 受影响的模块
    pub affected_modules: Vec<AffectedModule>,
    /// 影响范围评分
    pub impact_score: ImpactScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    pub change_type: BreakingChangeType,
    pub component: String,
    pub description: String,
    pub affected_consumers: Vec<String>,
    pub migration_suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BreakingChangeType {
    ApiSignatureChange,
    ApiRemoval,
    InterfaceChange,
    DataStructureChange,
    BehaviorChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedModule {
    pub module_path: String,
    pub impact_level: ImpactLevel,
    pub required_changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    Direct,
    Indirect,
    Potential,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactScore {
    pub overall_score: f64,
    pub breaking_changes_count: usize,
    pub affected_modules_count: usize,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
}

/// API 表面分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSurface {
    pub public_apis: Vec<PublicApi>,
    pub deprecated_apis: Vec<DeprecatedApi>,
    pub api_stability: ApiStability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicApi {
    pub name: String,
    pub api_type: ApiType,
    pub signature: String,
    pub documentation: Option<String>,
    pub usage_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiType {
    Function,
    Method,
    Class,
    Interface,
    Module,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecatedApi {
    pub api: PublicApi,
    pub deprecation_reason: String,
    pub replacement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiStability {
    pub stable_apis: usize,
    pub unstable_apis: usize,
    pub deprecated_apis: usize,
    pub stability_score: f64,
}

/// 代码质量热点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityHotspots {
    pub complexity_hotspots: Vec<ComplexityHotspot>,
    pub duplication_areas: Vec<DuplicationArea>,
    pub maintenance_burden: MaintenanceBurden,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityHotspot {
    pub component: String,
    pub complexity_score: u32,
    pub reasons: Vec<String>,
    pub refactoring_suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationArea {
    pub locations: Vec<String>,
    pub duplicated_lines: usize,
    pub similarity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceBurden {
    pub high_maintenance_components: Vec<String>,
    pub technical_debt_score: f64,
    pub estimated_refactoring_hours: f64,
}

/// ProjectInsights 生成器
pub struct InsightsGenerator;

impl InsightsGenerator {
    /// 从 Tree-sitter 的结构化摘要生成项目洞察
    pub fn generate(
        summary: &StructuralSummary,
        previous_summary: Option<&StructuralSummary>,
    ) -> ProjectInsights {
        ProjectInsights {
            architecture: Self::analyze_architecture(summary),
            impact_analysis: Self::analyze_impact(summary, previous_summary),
            api_surface: Self::analyze_api_surface(summary),
            quality_hotspots: Self::identify_quality_hotspots(summary),
        }
    }

    /// 分析架构
    fn analyze_architecture(summary: &StructuralSummary) -> ArchitecturalInsights {
        let dependencies = Self::build_dependency_graph(summary);
        let violations = Self::detect_pattern_violations(summary);
        let layers = Self::identify_architectural_layers(summary);
        let coupling = Self::analyze_coupling(summary);

        ArchitecturalInsights {
            module_dependencies: dependencies,
            pattern_violations: violations,
            architectural_layers: layers,
            coupling_analysis: coupling,
        }
    }

    /// 构建依赖关系图
    fn build_dependency_graph(summary: &StructuralSummary) -> DependencyGraph {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // 从类信息构建节点
        for class in &summary.classes {
            nodes.push(DependencyNode {
                id: class.name.clone(),
                name: class.name.clone(),
                node_type: NodeType::Class,
                file_path: format!("line:{}", class.line_start),
            });

            // 如果有继承关系，创建边
            if let Some(ref extends) = class.extends {
                edges.push(DependencyEdge {
                    from: class.name.clone(),
                    to: extends.clone(),
                    dependency_type: DependencyType::Inheritance,
                });
            }
        }

        // 检测循环依赖
        let circular = Self::detect_circular_dependencies(&edges);

        DependencyGraph {
            nodes,
            edges,
            circular_dependencies: circular,
        }
    }

    /// 检测循环依赖
    fn detect_circular_dependencies(edges: &[DependencyEdge]) -> Vec<CircularDependency> {
        let mut circular_deps = Vec::new();

        // 构建邻接表
        let mut graph: HashMap<String, HashSet<String>> = HashMap::new();
        for edge in edges {
            graph
                .entry(edge.from.clone())
                .or_insert_with(HashSet::new)
                .insert(edge.to.clone());
        }

        // 使用 DFS 检测循环
        for node in graph.keys() {
            let mut visited = HashSet::new();
            let mut stack = Vec::new();
            if Self::dfs_detect_cycle(node, &graph, &mut visited, &mut stack) {
                circular_deps.push(CircularDependency {
                    cycle: stack,
                    severity: Severity::High,
                });
            }
        }

        circular_deps
    }

    fn dfs_detect_cycle(
        node: &str,
        graph: &HashMap<String, HashSet<String>>,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
    ) -> bool {
        if stack.contains(&node.to_string()) {
            // 找到循环
            return true;
        }

        if visited.contains(node) {
            return false;
        }

        visited.insert(node.to_string());
        stack.push(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if Self::dfs_detect_cycle(neighbor, graph, visited, stack) {
                    return true;
                }
            }
        }

        stack.pop();
        false
    }

    /// 检测架构模式违规
    fn detect_pattern_violations(summary: &StructuralSummary) -> Vec<PatternViolation> {
        let mut violations = Vec::new();

        // 检测单一职责违规
        for class in &summary.classes {
            let method_count = class.methods.len();
            if method_count > 20 {
                violations.push(PatternViolation {
                    pattern_type: ArchitecturalPattern::SingleResponsibility,
                    violation_description: format!(
                        "类 {} 包含 {} 个方法，可能违反单一职责原则",
                        class.name, method_count
                    ),
                    location: format!("{}:{}", class.name, class.line_start),
                    severity: Severity::Medium,
                    suggestion: "考虑将该类拆分为多个更小、更专注的类".to_string(),
                });
            }
        }

        // 检测过长的函数
        for func in &summary.functions {
            let line_count = func.line_end - func.line_start;
            if line_count > 50 {
                violations.push(PatternViolation {
                    pattern_type: ArchitecturalPattern::SingleResponsibility,
                    violation_description: format!(
                        "函数 {} 包含 {} 行，过于复杂",
                        func.name, line_count
                    ),
                    location: format!("{}:{}", func.name, func.line_start),
                    severity: if line_count > 100 {
                        Severity::High
                    } else {
                        Severity::Medium
                    },
                    suggestion: "考虑将该函数拆分为多个更小的函数".to_string(),
                });
            }
        }

        violations
    }

    /// 识别架构层次
    fn identify_architectural_layers(_summary: &StructuralSummary) -> Vec<ArchitecturalLayer> {
        // 基于命名约定识别层次
        let mut layers = Vec::new();

        // 这里是简化的实现，实际可以基于文件路径、包结构等识别
        layers.push(ArchitecturalLayer {
            name: "Domain".to_string(),
            components: vec![],
            allowed_dependencies: vec!["Infrastructure".to_string()],
            violations: vec![],
        });

        layers
    }

    /// 分析耦合度
    fn analyze_coupling(_summary: &StructuralSummary) -> CouplingAnalysis {
        let avg_coupling = 0.3; // 简化计算

        CouplingAnalysis {
            average_coupling: avg_coupling,
            highly_coupled_components: vec![],
            loosely_coupled_components: vec![],
        }
    }

    /// 分析影响
    fn analyze_impact(
        summary: &StructuralSummary,
        previous_summary: Option<&StructuralSummary>,
    ) -> ImpactAnalysis {
        let breaking_changes = if let Some(prev) = previous_summary {
            Self::detect_breaking_changes(summary, prev)
        } else {
            vec![]
        };

        let affected_modules = Self::identify_affected_modules(&breaking_changes);
        let impact_score = Self::calculate_impact_score(&breaking_changes, &affected_modules);

        ImpactAnalysis {
            breaking_changes,
            affected_modules,
            impact_score,
        }
    }

    /// 检测破坏性变更
    fn detect_breaking_changes(
        current: &StructuralSummary,
        previous: &StructuralSummary,
    ) -> Vec<BreakingChange> {
        let mut breaking_changes = Vec::new();

        // 检测函数签名变化
        let prev_funcs: HashMap<_, _> = previous
            .functions
            .iter()
            .map(|f| (f.name.clone(), f))
            .collect();

        for func in &current.functions {
            if let Some(prev_func) = prev_funcs.get(&func.name) {
                if func.parameters != prev_func.parameters {
                    breaking_changes.push(BreakingChange {
                        change_type: BreakingChangeType::ApiSignatureChange,
                        component: func.name.clone(),
                        description: format!(
                            "函数 {} 的参数从 {:?} 改为 {:?}",
                            func.name, prev_func.parameters, func.parameters
                        ),
                        affected_consumers: vec![],
                        migration_suggestion: "更新所有调用点以匹配新的参数签名".to_string(),
                    });
                }
            }
        }

        // 检测删除的 API
        for prev_func in &previous.functions {
            if !current.functions.iter().any(|f| f.name == prev_func.name) {
                breaking_changes.push(BreakingChange {
                    change_type: BreakingChangeType::ApiRemoval,
                    component: prev_func.name.clone(),
                    description: format!("函数 {} 被删除", prev_func.name),
                    affected_consumers: vec![],
                    migration_suggestion: "寻找替代 API 或恢复该函数".to_string(),
                });
            }
        }

        breaking_changes
    }

    /// 识别受影响的模块
    fn identify_affected_modules(breaking_changes: &[BreakingChange]) -> Vec<AffectedModule> {
        breaking_changes
            .iter()
            .map(|change| AffectedModule {
                module_path: change.component.clone(),
                impact_level: ImpactLevel::Direct,
                required_changes: vec![change.migration_suggestion.clone()],
            })
            .collect()
    }

    /// 计算影响评分
    fn calculate_impact_score(
        breaking_changes: &[BreakingChange],
        affected_modules: &[AffectedModule],
    ) -> ImpactScore {
        let score = breaking_changes.len() as f64 * 10.0 + affected_modules.len() as f64 * 5.0;

        ImpactScore {
            overall_score: score,
            breaking_changes_count: breaking_changes.len(),
            affected_modules_count: affected_modules.len(),
            risk_level: match score {
                s if s > 50.0 => RiskLevel::Critical,
                s if s > 30.0 => RiskLevel::High,
                s if s > 10.0 => RiskLevel::Medium,
                _ => RiskLevel::Low,
            },
        }
    }

    /// 分析 API 表面
    fn analyze_api_surface(summary: &StructuralSummary) -> ApiSurface {
        let public_apis: Vec<PublicApi> = summary
            .functions
            .iter()
            .filter(|f| f.visibility.as_ref().map_or(false, |v| v == "public"))
            .map(|f| PublicApi {
                name: f.name.clone(),
                api_type: ApiType::Function,
                signature: format!("{}({:?})", f.name, f.parameters),
                documentation: None,
                usage_count: 0,
            })
            .collect();

        let stability = ApiStability {
            stable_apis: public_apis.len(),
            unstable_apis: 0,
            deprecated_apis: 0,
            stability_score: 1.0,
        };

        ApiSurface {
            public_apis,
            deprecated_apis: vec![],
            api_stability: stability,
        }
    }

    /// 识别质量热点
    fn identify_quality_hotspots(summary: &StructuralSummary) -> QualityHotspots {
        let complexity_hotspots: Vec<ComplexityHotspot> = summary
            .functions
            .iter()
            .filter(|f| f.parameters.len() > 5 || (f.line_end - f.line_start) > 50)
            .map(|f| ComplexityHotspot {
                component: f.name.clone(),
                complexity_score: ((f.line_end - f.line_start) + f.parameters.len() * 5) as u32,
                reasons: vec![
                    if f.parameters.len() > 5 {
                        Some(format!("参数过多: {}", f.parameters.len()))
                    } else {
                        None
                    },
                    if (f.line_end - f.line_start) > 50 {
                        Some(format!("函数过长: {} 行", f.line_end - f.line_start))
                    } else {
                        None
                    },
                ]
                .into_iter()
                .flatten()
                .collect(),
                refactoring_suggestions: vec![
                    "考虑使用参数对象模式".to_string(),
                    "将函数拆分为更小的功能单元".to_string(),
                ],
            })
            .collect();

        let maintenance_burden = MaintenanceBurden {
            high_maintenance_components: complexity_hotspots
                .iter()
                .map(|h| h.component.clone())
                .collect(),
            technical_debt_score: complexity_hotspots.len() as f64 * 10.0,
            estimated_refactoring_hours: complexity_hotspots.len() as f64 * 2.5,
        };

        QualityHotspots {
            complexity_hotspots,
            duplication_areas: vec![],
            maintenance_burden,
        }
    }
}

/// 将洞察转换为 AI 可理解的上下文
impl ProjectInsights {
    pub fn to_ai_context(&self) -> String {
        let mut context = String::new();

        context.push_str("## 项目架构洞察\n\n");

        // 架构问题
        if !self.architecture.pattern_violations.is_empty() {
            context.push_str("### 架构模式违规\n");
            for violation in &self.architecture.pattern_violations {
                context.push_str(&format!(
                    "- **{}**: {} (位置: {})\n  建议: {}\n",
                    format!("{:?}", violation.pattern_type),
                    violation.violation_description,
                    violation.location,
                    violation.suggestion
                ));
            }
            context.push_str("\n");
        }

        // 循环依赖
        if !self
            .architecture
            .module_dependencies
            .circular_dependencies
            .is_empty()
        {
            context.push_str("### 循环依赖\n");
            for circular in &self.architecture.module_dependencies.circular_dependencies {
                context.push_str(&format!(
                    "- 循环: {} (严重性: {:?})\n",
                    circular.cycle.join(" -> "),
                    circular.severity
                ));
            }
            context.push_str("\n");
        }

        // 破坏性变更
        if !self.impact_analysis.breaking_changes.is_empty() {
            context.push_str("### 破坏性变更\n");
            for change in &self.impact_analysis.breaking_changes {
                context.push_str(&format!(
                    "- **{:?}**: {}\n  影响: {:?}\n  建议: {}\n",
                    change.change_type,
                    change.description,
                    change.affected_consumers,
                    change.migration_suggestion
                ));
            }
            context.push_str("\n");
        }

        // 质量热点
        if !self.quality_hotspots.complexity_hotspots.is_empty() {
            context.push_str("### 复杂度热点\n");
            for hotspot in &self.quality_hotspots.complexity_hotspots {
                context.push_str(&format!(
                    "- **{}** (复杂度: {})\n  原因: {}\n  建议: {}\n",
                    hotspot.component,
                    hotspot.complexity_score,
                    hotspot.reasons.join(", "),
                    hotspot.refactoring_suggestions.join("; ")
                ));
            }
            context.push_str("\n");
        }

        // 影响评估
        context.push_str(&format!(
            "### 影响评估\n- 风险级别: {:?}\n- 破坏性变更数: {}\n- 受影响模块数: {}\n- 整体评分: {:.1}\n",
            self.impact_analysis.impact_score.risk_level,
            self.impact_analysis.impact_score.breaking_changes_count,
            self.impact_analysis.impact_score.affected_modules_count,
            self.impact_analysis.impact_score.overall_score
        ));

        context
    }
}

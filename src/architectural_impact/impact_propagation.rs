// 影响传播算法模块 - 计算代码变更的影响传播范围
use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};
use crate::architectural_impact::dependency_graph::{DependencyGraph, NodeType, EdgeType};

/// 影响传播分析器
#[derive(Debug, Clone)]
pub struct ImpactPropagation {
    /// 依赖图
    graph: DependencyGraph,
    /// 节点影响分数缓存
    impact_scores: HashMap<String, f32>,
    /// 传播规则引擎
    rules: PropagationRules,
}

/// 影响传播的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactScope {
    /// 变更的源节点ID
    pub source_nodes: Vec<String>,
    /// 直接影响的组件
    pub direct_impacts: Vec<ImpactedComponent>,
    /// 间接影响的组件
    pub indirect_impacts: Vec<ImpactedComponent>,
    /// 影响半径（0-1之间的值）
    pub impact_radius: f32,
    /// 影响深度（传播的最大层数）
    pub impact_depth: usize,
    /// 关键传播路径
    pub critical_paths: Vec<ImpactPath>,
    /// 影响统计信息
    pub statistics: ImpactStatistics,
}

/// 受影响的组件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactedComponent {
    /// 组件ID
    pub component_id: String,
    /// 组件类型
    pub component_type: ComponentType,
    /// 影响分数（0-1）
    pub impact_score: f32,
    /// 影响原因
    pub impact_reason: String,
    /// 距离变更源的层数
    pub distance_from_change: usize,
    /// 传播类型
    pub propagation_type: PropagationType,
}

/// 影响传播路径
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactPath {
    /// 路径上的节点序列
    pub nodes: Vec<String>,
    /// 路径的总权重
    pub weight: f32,
    /// 路径描述
    pub description: String,
    /// 是否为关键路径
    pub is_critical: bool,
}

/// 影响统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactStatistics {
    /// 总影响节点数
    pub total_impacted_nodes: usize,
    /// 直接影响节点数
    pub direct_impact_count: usize,
    /// 间接影响节点数
    pub indirect_impact_count: usize,
    /// 平均影响分数
    pub average_impact_score: f32,
    /// 最大传播深度
    pub max_propagation_depth: usize,
    /// 高影响节点数（影响分数 > 0.7）
    pub high_impact_count: usize,
}

/// 组件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    Function,
    Class,
    Module,
    File,
    Unknown,
}

/// 传播类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropagationType {
    /// 直接传播（一步可达）
    Direct,
    /// 传递传播（通过中介节点）
    Transitive,
    /// 条件传播（依赖特定条件）
    Conditional,
    /// 无传播
    None,
}

/// 传播规则引擎
#[derive(Debug, Clone)]
pub struct PropagationRules {
    /// 规则列表
    rules: Vec<Rule>,
}

/// 单条传播规则
#[derive(Debug, Clone)]
pub struct Rule {
    /// 规则条件
    condition: RuleCondition,
    /// 影响因子（0-1）
    impact_factor: f32,
    /// 传播类型
    propagation_type: PropagationType,
    /// 规则描述
    description: String,
}

/// 规则条件
#[derive(Debug, Clone)]
pub enum RuleCondition {
    /// 基于边类型的条件
    EdgeType(EdgeType),
    /// 基于节点类型的条件
    NodeType(NodeType),
    /// 基于距离的条件
    Distance(usize),
    /// 复合条件（AND）
    And(Box<RuleCondition>, Box<RuleCondition>),
    /// 复合条件（OR）
    Or(Box<RuleCondition>, Box<RuleCondition>),
    /// 总是匹配
    Always,
}

impl ImpactPropagation {
    /// 创建新的影响传播分析器
    pub fn new(graph: DependencyGraph) -> Self {
        let rules = PropagationRules::default();
        Self {
            graph,
            impact_scores: HashMap::new(),
            rules,
        }
    }

    /// 计算从变更节点开始的影响传播
    pub fn calculate_impact(
        &mut self,
        changed_nodes: Vec<String>,
        max_depth: usize,
    ) -> ImpactScope {
        // 清理之前的缓存
        self.impact_scores.clear();
        
        // 使用BFS计算影响传播
        let impact_map = self.bfs_propagation(&changed_nodes, max_depth);
        
        // 构建影响范围结果
        self.build_impact_scope(changed_nodes, impact_map, max_depth)
    }

    /// 使用BFS算法计算影响传播
    fn bfs_propagation(
        &mut self,
        source_nodes: &[String],
        max_depth: usize,
    ) -> HashMap<String, (f32, usize, PropagationType)> {
        let mut impact_map = HashMap::new();
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        // 初始化源节点
        for node_id in source_nodes {
            if self.graph.nodes.contains_key(node_id) {
                queue.push_back((node_id.clone(), 1.0, 0, PropagationType::Direct));
                impact_map.insert(node_id.clone(), (1.0, 0, PropagationType::Direct));
                visited.insert(node_id.clone());
            }
        }

        while let Some((current_node, current_score, depth, _)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            // 获取当前节点的所有依赖者
            let dependents = self.graph.get_dependents(&current_node);
            
            for dependent_id in dependents {
                if visited.contains(dependent_id) {
                    continue;
                }

                // 计算传播到此节点的影响分数
                let (propagated_score, propagation_type) = self.calculate_propagated_impact(
                    &current_node,
                    dependent_id,
                    current_score,
                    depth + 1,
                );

                if propagated_score > 0.05 {  // 只保留有意义的影响
                    impact_map.insert(
                        dependent_id.clone(),
                        (propagated_score, depth + 1, propagation_type.clone())
                    );
                    queue.push_back((dependent_id.clone(), propagated_score, depth + 1, propagation_type));
                    visited.insert(dependent_id.clone());
                }
            }
        }

        impact_map
    }

    /// 计算传播到目标节点的影响分数
    fn calculate_propagated_impact(
        &self,
        source_id: &str,
        target_id: &str,
        source_score: f32,
        depth: usize,
    ) -> (f32, PropagationType) {
        // 基础衰减因子
        let distance_decay = 0.8_f32.powi(depth as i32);
        
        // 根据边类型和节点类型调整影响因子
        let edge_factor = self.get_edge_impact_factor(source_id, target_id);
        let node_factor = self.get_node_impact_factor(target_id);
        
        let final_score = source_score * distance_decay * edge_factor * node_factor;
        
        let propagation_type = if depth == 1 {
            PropagationType::Direct
        } else if final_score > 0.5 {
            PropagationType::Transitive
        } else {
            PropagationType::Conditional
        };

        (final_score, propagation_type)
    }

    /// 获取边的影响因子
    fn get_edge_impact_factor(&self, source_id: &str, target_id: &str) -> f32 {
        // 查找对应的边（支持反向匹配，以适配反向传播的邻接遍历）
        let edge_opt = self.graph.edges.iter()
            .find(|e| e.from == source_id && e.to == target_id)
            .or_else(|| self.graph.edges.iter().find(|e| e.from == target_id && e.to == source_id));

        match edge_opt {
            Some(e) => match e.edge_type {
                EdgeType::Calls => 0.9,        // 函数调用影响很大
                EdgeType::Inherits => 0.95,    // 继承关系影响极大
                EdgeType::Implements => 0.9,   // 实现关系影响很大
                EdgeType::Uses => 0.7,         // 使用关系中等影响
                EdgeType::References => 0.6,   // 引用关系较小影响
                EdgeType::Imports => 0.5,      // 导入关系较小影响
                EdgeType::Contains => 0.8,     // 包含关系影响较大
                EdgeType::DependsOn => 0.8,    // 依赖关系影响较大
                EdgeType::Exports => 0.3,      // 导出关系影响较小
            },
            None => 0.4, // 默认影响因子
        }
    }

    /// 获取节点的影响因子
    fn get_node_impact_factor(&self, node_id: &str) -> f32 {
        if let Some(node) = self.graph.nodes.get(node_id) {
            // 基于节点重要性分数
            let base_factor = node.importance_score;
            
            // 基于节点类型调整
            let type_factor = match node.node_type {
                NodeType::Function(_) => 1.0,
                NodeType::Class(_) => 1.2,      // 类变更影响更大
                NodeType::Module(_) => 1.1,     // 模块变更影响较大
                NodeType::File(_) => 0.8,       // 文件变更影响相对较小
            };

            base_factor * type_factor
        } else {
            0.5 // 默认因子
        }
    }

    /// 构建最终的影响范围结果
    fn build_impact_scope(
        &self,
        source_nodes: Vec<String>,
        impact_map: HashMap<String, (f32, usize, PropagationType)>,
        max_depth: usize,
    ) -> ImpactScope {
        let mut direct_impacts = Vec::new();
        let mut indirect_impacts = Vec::new();
        let mut max_distance = 0;
        let mut total_score = 0.0;
        let mut high_impact_count = 0;

        for (node_id, (score, distance, prop_type)) in &impact_map {
            if source_nodes.contains(node_id) {
                continue; // 跳过源节点
            }

            let component = ImpactedComponent {
                component_id: node_id.clone(),
                component_type: self.get_component_type(node_id),
                impact_score: *score,
                impact_reason: self.generate_impact_reason(node_id, &prop_type),
                distance_from_change: *distance,
                propagation_type: prop_type.clone(),
            };

            total_score += score;
            if *score > 0.7 {
                high_impact_count += 1;
            }
            max_distance = max_distance.max(*distance);

            if *distance == 1 {
                direct_impacts.push(component);
            } else {
                indirect_impacts.push(component);
            }
        }

        // 按影响分数排序
        direct_impacts.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap());
        indirect_impacts.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap());

        let total_nodes = direct_impacts.len() + indirect_impacts.len();
        let average_score = if total_nodes > 0 {
            total_score / (direct_impacts.len() as f32 + indirect_impacts.len() as f32)
        } else {
            0.0
        };

        let statistics = ImpactStatistics {
            total_impacted_nodes: total_nodes,
            direct_impact_count: direct_impacts.len(),
            indirect_impact_count: indirect_impacts.len(),
            average_impact_score: average_score,
            max_propagation_depth: max_distance,
            high_impact_count,
        };

        // 计算影响半径（基于受影响节点数与总节点数的比例）
        let impact_radius = if self.graph.nodes.is_empty() {
            0.0
        } else {
            total_nodes as f32 / self.graph.nodes.len() as f32
        };

        // 找到关键路径
        let critical_paths = self.find_critical_paths(&source_nodes, &impact_map, max_depth);

        ImpactScope {
            source_nodes,
            direct_impacts,
            indirect_impacts,
            impact_radius,
            impact_depth: max_distance,
            critical_paths,
            statistics,
        }
    }

    /// 获取组件类型
    fn get_component_type(&self, node_id: &str) -> ComponentType {
        if let Some(node) = self.graph.nodes.get(node_id) {
            match node.node_type {
                NodeType::Function(_) => ComponentType::Function,
                NodeType::Class(_) => ComponentType::Class,
                NodeType::Module(_) => ComponentType::Module,
                NodeType::File(_) => ComponentType::File,
            }
        } else {
            ComponentType::Unknown
        }
    }

    /// 生成影响原因描述
    fn generate_impact_reason(&self, _node_id: &str, prop_type: &PropagationType) -> String {
        match prop_type {
            PropagationType::Direct => "直接调用或依赖关系".to_string(),
            PropagationType::Transitive => "通过传递依赖受到影响".to_string(),
            PropagationType::Conditional => "在特定条件下可能受到影响".to_string(),
            PropagationType::None => "无直接影响".to_string(),
        }
    }

    /// 查找关键影响路径
    fn find_critical_paths(
        &self,
        source_nodes: &[String],
        impact_map: &HashMap<String, (f32, usize, PropagationType)>,
        max_depth: usize,
    ) -> Vec<ImpactPath> {
        let mut paths = Vec::new();
        
        // 选择受影响分数最高的若干个节点作为终点（排除源节点）
        let mut candidates: Vec<(String, f32)> = impact_map
            .iter()
            .filter(|(id, _)| !source_nodes.contains(*id))
            .map(|(id, (score, _, _))| (id.clone(), *score))
            .collect();
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let targets: Vec<String> = candidates.into_iter().take(5).map(|(id, _)| id).collect();

        // 对每个目标，尝试找到从源节点到它的最短路径
        for source in source_nodes {
            for target in &targets {
                if let Some(path) = self.find_shortest_path(source, target, max_depth) {
                    let weight = self.calculate_path_weight(&path);
                    let description = format!("从 {} 到 {} 的关键影响路径", source, target);
                    
                    paths.push(ImpactPath {
                        nodes: path,
                        weight,
                        description,
                        is_critical: weight > 0.8,
                    });
                }
            }
        }

        // 按权重排序，只保留前5条关键路径
        paths.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
        paths.truncate(5);
        
        paths
    }

    /// 使用BFS查找两个节点间的最短路径
    fn find_shortest_path(&self, start: &str, end: &str, max_depth: usize) -> Option<Vec<String>> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent: HashMap<String, String> = HashMap::new();

        queue.push_back((start.to_string(), 0));
        visited.insert(start.to_string());

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            if current == end {
                // 重构路径
                let mut path = Vec::new();
                let mut current_node = end.to_string();
                
                while let Some(prev) = parent.get(&current_node) {
                    path.push(current_node.clone());
                    current_node = prev.clone();
                }
                path.push(start.to_string());
                path.reverse();
                return Some(path);
            }

            // 探索邻居节点
            for neighbor in self.graph.get_dependents(&current) {
                if !visited.contains(neighbor) {
                    visited.insert(neighbor.clone());
                    parent.insert(neighbor.clone(), current.clone());
                    queue.push_back((neighbor.clone(), depth + 1));
                }
            }
        }

        None
    }

    /// 计算路径权重
    fn calculate_path_weight(&self, path: &[String]) -> f32 {
        if path.len() < 2 {
            return 0.0;
        }

        let mut total_weight = 1.0;
        
        for window in path.windows(2) {
            let from = &window[0];
            let to = &window[1];
            let edge_weight = self.get_edge_impact_factor(from, to);
            total_weight *= edge_weight;
        }

        // 考虑路径长度的衰减
        let length_factor = 0.9_f32.powi((path.len() - 1) as i32);
        total_weight * length_factor
    }

    /// 计算影响半径
    pub fn calculate_radius(&self, impact_scope: &ImpactScope) -> f32 {
        if self.graph.nodes.is_empty() {
            return 0.0;
        }

        let total_impacted = impact_scope.statistics.total_impacted_nodes as f32;
        let total_nodes = self.graph.nodes.len() as f32;
        
        // 基础半径：受影响节点数占比
        let base_radius = total_impacted / total_nodes;
        
        // 根据平均影响分数调整
        let score_factor = impact_scope.statistics.average_impact_score;
        
        // 根据传播深度调整
        let depth_factor = (impact_scope.impact_depth as f32) / 10.0;
        
        (base_radius + score_factor * 0.3 + depth_factor * 0.2).min(1.0)
    }
}

impl PropagationRules {
    /// 创建默认的传播规则
    pub fn default() -> Self {
        let rules = vec![
            // 函数调用规则
            Rule {
                condition: RuleCondition::EdgeType(EdgeType::Calls),
                impact_factor: 0.9,
                propagation_type: PropagationType::Direct,
                description: "函数调用关系的直接影响".to_string(),
            },
            // 继承关系规则
            Rule {
                condition: RuleCondition::EdgeType(EdgeType::Inherits),
                impact_factor: 0.95,
                propagation_type: PropagationType::Direct,
                description: "继承关系的强影响".to_string(),
            },
            // 距离衰减规则
            Rule {
                condition: RuleCondition::Distance(3),
                impact_factor: 0.3,
                propagation_type: PropagationType::Conditional,
                description: "远距离传播的条件影响".to_string(),
            },
        ];

        Self { rules }
    }

    /// 根据条件评估影响因子
    pub fn evaluate_impact_factor(
        &self,
        _source_id: &str,
        _target_id: &str,
        _edge_type: EdgeType,
        _distance: usize,
    ) -> f32 {
        // 简化实现，返回默认值
        // 实际实现中应该根据规则进行匹配和计算
        0.7
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::architectural_impact::dependency_graph::{DependencyGraph, Node, Edge, NodeType, FunctionNode, NodeMetadata, Visibility};

    fn create_test_graph() -> DependencyGraph {
        let mut graph = DependencyGraph::new();
        
        // 创建测试节点
        for i in 1..=5 {
            let id = format!("func{}", i);
            let node = Node {
                id: id.clone(),
                node_type: NodeType::Function(FunctionNode {
                    name: format!("function{}", i),
                    visibility: Some(Visibility::Public),
                    parameters: vec![],
                    return_type: None,
                    is_async: false,
                }),
                metadata: NodeMetadata {
                    file_path: "test.rs".to_string(),
                    start_line: i,
                    end_line: i + 5,
                    complexity: 1,
                    created_at: 0,
                },
                importance_score: 0.8,
            };
            graph.add_node(node);
        }

        // 创建依赖关系: func1 -> func2 -> func3 -> func4
        //                    \-> func5
        graph.add_edge(Edge {
            from: "func2".to_string(),
            to: "func1".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1.0,
            metadata: None,
        });

        graph.add_edge(Edge {
            from: "func3".to_string(),
            to: "func2".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1.0,
            metadata: None,
        });

        graph.add_edge(Edge {
            from: "func4".to_string(),
            to: "func3".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1.0,
            metadata: None,
        });

        graph.add_edge(Edge {
            from: "func5".to_string(),
            to: "func1".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1.0,
            metadata: None,
        });

        graph.rebuild_adjacency_lists();
        graph
    }

    #[test]
    fn test_impact_propagation_calculation() {
        let graph = create_test_graph();
        let mut propagation = ImpactPropagation::new(graph);

        let impact_scope = propagation.calculate_impact(vec!["func1".to_string()], 3);

        assert!(!impact_scope.source_nodes.is_empty());
        assert!(impact_scope.statistics.total_impacted_nodes > 0);
        assert!(impact_scope.impact_radius > 0.0);
    }

    #[test]
    fn test_direct_and_indirect_impacts() {
        let graph = create_test_graph();
        let mut propagation = ImpactPropagation::new(graph);

        let impact_scope = propagation.calculate_impact(vec!["func1".to_string()], 3);

        // func2 和 func5 应该是直接影响
        assert!(!impact_scope.direct_impacts.is_empty());
        
        // func3 和 func4 应该是间接影响
        assert!(!impact_scope.indirect_impacts.is_empty());
    }

    #[test]
    fn test_critical_path_finding() {
        let graph = create_test_graph();
        let mut propagation = ImpactPropagation::new(graph);

        let impact_scope = propagation.calculate_impact(vec!["func1".to_string()], 3);

        // 应该能找到关键路径
        assert!(!impact_scope.critical_paths.is_empty());
        
        // 检查路径结构
        for path in &impact_scope.critical_paths {
            assert!(path.nodes.len() >= 2);
            assert!(path.weight > 0.0);
        }
    }

    #[test]
    fn test_impact_statistics() {
        let graph = create_test_graph();
        let mut propagation = ImpactPropagation::new(graph);

        let impact_scope = propagation.calculate_impact(vec!["func1".to_string()], 3);
        let stats = &impact_scope.statistics;

        assert!(stats.total_impacted_nodes > 0);
        assert_eq!(
            stats.total_impacted_nodes,
            stats.direct_impact_count + stats.indirect_impact_count
        );
        assert!(stats.average_impact_score > 0.0);
        assert!(stats.max_propagation_depth > 0);
    }
}

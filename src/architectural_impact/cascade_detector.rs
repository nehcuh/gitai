// 级联效应检测模块
use std::collections::HashSet;
use serde::{Serialize, Deserialize};

use crate::architectural_impact::dependency_graph::{DependencyGraph, EdgeType, NodeType, Node};
use crate::architectural_impact::{BreakingChange};

/// 级联效应严重性
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// 级联效应阈值配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeThresholds {
    /// 最小概率阈值（低于该值的链路不计入）
    pub min_probability: f32,
    /// 最小链路长度
    pub min_chain_len: usize,
    /// 判定关键节点的中心性阈值
    pub critical_centrality: f32,
    /// 搜索的最大深度
    pub max_depth: usize,
    /// 返回的最大结果数量
    pub max_results: usize,
}

impl Default for CascadeThresholds {
    fn default() -> Self {
        Self {
            min_probability: 0.3,
            min_chain_len: 2,
            critical_centrality: 0.15,
            max_depth: 4,
            max_results: 10,
        }
    }
}

/// 关键节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalNode {
    pub id: String,
    pub centrality: f32,
    pub fan_in: usize,
    pub fan_out: usize,
}

/// 检测到的级联效应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeEffect {
    /// 触发节点（变更源）
    pub trigger: String,
    /// 受影响的链路（节点序列）
    pub affected_chain: Vec<String>,
    /// 概率（0-1）
    pub probability: f32,
    /// 严重性
    pub severity: Severity,
    /// 描述
    pub description: String,
}

/// 级联效应检测器
#[derive(Debug, Clone)]
pub struct CascadeDetector {
    graph: DependencyGraph,
    thresholds: CascadeThresholds,
}

impl CascadeDetector {
    pub fn new(graph: DependencyGraph) -> Self {
        Self { graph, thresholds: CascadeThresholds::default() }
    }

    pub fn with_thresholds(mut self, thresholds: CascadeThresholds) -> Self {
        self.thresholds = thresholds;
        self
    }

    /// 检测潜在的级联效应
    pub fn detect_cascades(&self, changes: &[BreakingChange]) -> Vec<CascadeEffect> {
        let mut effects: Vec<CascadeEffect> = Vec::new();
        let mut seen: HashSet<Vec<String>> = HashSet::new();

        for change in changes {
            let trigger_nodes = self.find_nodes_for_component(&change.component);
            for trigger in trigger_nodes {
                let chains = self.enumerate_chains(&trigger, self.thresholds.max_depth);
                for chain in chains {
                    if chain.len() < self.thresholds.min_chain_len { continue; }
                    let prob = self.estimate_chain_probability(&chain);
                    if prob < self.thresholds.min_probability { continue; }

                    // 去重（相同链路不重复添加）
                    if !seen.insert(chain.clone()) { continue; }

                    let severity = self.estimate_severity(&chain, prob);
                    let description = format!(
                        "由 {} 触发，经 {} 层传播的{}级别级联效应",
                        &trigger,
                        chain.len().saturating_sub(1),
                        match severity { Severity::Critical=>"紧急", Severity::High=>"高", Severity::Medium=>"中等", Severity::Low=>"低" }
                    );

                    effects.push(CascadeEffect {
                        trigger: trigger.clone(),
                        affected_chain: chain,
                        probability: prob,
                        severity,
                        description,
                    });
                }
            }
        }

        // 结果排序与截断
        effects.sort_by(|a, b| b.probability.partial_cmp(&a.probability).unwrap());
        effects.truncate(self.thresholds.max_results);
        effects
    }

    /// 识别系统中的关键节点
    pub fn identify_critical_nodes(&self) -> Vec<CriticalNode> {
        let mut critical = Vec::new();
        for node_id in self.graph.nodes.keys() {
            let centrality = self.graph.calculate_centrality(node_id);
            if centrality >= self.thresholds.critical_centrality {
                let fan_out = self.graph.get_dependencies(node_id).len();
                let fan_in = self.graph.get_dependents(node_id).len();
                critical.push(CriticalNode {
                    id: node_id.clone(), centrality, fan_in, fan_out
                });
            }
        }
        critical.sort_by(|a, b| b.centrality.partial_cmp(&a.centrality).unwrap());
        critical
    }

    /// 查找匹配组件的节点（基于名称/ID/文件路径的启发式匹配）
    fn find_nodes_for_component(&self, component: &str) -> Vec<String> {
        let mut matches = Vec::new();
        for (id, node) in &self.graph.nodes {
            // 直接匹配ID或后缀（如 func:path::name）
            let id_match = id == component || id.ends_with(component);

            // 基于节点内部名称匹配
            let name_match = match &node.node_type {
                NodeType::Function(f) => f.name == component,
                NodeType::Class(c) => c.name == component,
                NodeType::Module(m) => m.name == component || m.path.ends_with(component),
                NodeType::File(f) => f.path.ends_with(component),
            };

            if id_match || name_match {
                matches.push(id.clone());
            }
        }
        if matches.is_empty() {
            // 宽松匹配：包含子串
            for (id, node) in &self.graph.nodes {
                if id.contains(component) || self.node_contains(node, component) {
                    matches.push(id.clone());
                }
            }
        }
        matches
    }

    fn node_contains(&self, node: &Node, q: &str) -> bool {
        match &node.node_type {
            NodeType::Function(f) => f.name.contains(q),
            NodeType::Class(c) => c.name.contains(q),
            NodeType::Module(m) => m.name.contains(q) || m.path.contains(q),
            NodeType::File(f) => f.path.contains(q),
        }
    }

    /// 枚举从起点出发的传播链（最多到达 max_depth）
    fn enumerate_chains(&self, start: &str, max_depth: usize) -> Vec<Vec<String>> {
        let mut results = Vec::new();
        let mut stack: Vec<(String, Vec<String>)> = Vec::new();
        stack.push((start.to_string(), vec![start.to_string()]));

        while let Some((current, path)) = stack.pop() {
            let depth = path.len() - 1;
            if depth >= max_depth { continue; }

            for dep in self.graph.get_dependents(&current) {
                if path.contains(dep) { continue; } // 避免环
                let mut new_path = path.clone();
                new_path.push(dep.clone());
                results.push(new_path.clone());
                stack.push((dep.clone(), new_path));
            }
        }
        results
    }

    /// 估算链路概率
    fn estimate_chain_probability(&self, chain: &[String]) -> f32 {
        if chain.len() < 2 { return 0.0; }
        let mut prob = 1.0_f32;
        for w in chain.windows(2) {
            let from = &w[0];
            let to = &w[1];
            let edge_factor = self.edge_impact_factor(from, to);
            // 距离衰减（越远越低）
            prob *= edge_factor * 0.85;
            // 节点重要性影响
            if let Some(node) = self.graph.nodes.get(to) {
                prob *= (0.5 + node.importance_score * 0.5).min(1.0);
            }
        }
        prob.min(1.0)
    }

    /// 根据链路与节点中心性估算严重性
    fn estimate_severity(&self, chain: &[String], probability: f32) -> Severity {
        let length = chain.len();
        let mut max_cent = 0.0_f32;
        for node_id in chain {
            let c = self.graph.calculate_centrality(node_id);
            if c > max_cent { max_cent = c; }
        }
        if probability > 0.8 || length >= 5 || max_cent > (self.thresholds.critical_centrality + 0.1) {
            Severity::High
        } else if probability > 0.6 || length >= 4 || max_cent > self.thresholds.critical_centrality {
            Severity::Medium
        } else if probability > self.thresholds.min_probability {
            Severity::Low
        } else {
            Severity::Low
        }
    }

    /// 辅助：根据边类型返回影响因子
    fn edge_impact_factor(&self, from: &str, to: &str) -> f32 {
        if let Some(edge) = self.graph.edges.iter().find(|e| e.from == from && e.to == to) {
            match edge.edge_type {
                EdgeType::Calls => 0.9,
                EdgeType::Inherits => 0.95,
                EdgeType::Implements => 0.9,
                EdgeType::Uses => 0.7,
                EdgeType::References => 0.6,
                EdgeType::Imports => 0.5,
                EdgeType::Contains => 0.8,
                EdgeType::DependsOn => 0.8,
                EdgeType::Exports => 0.3,
            }
        } else {
            0.5
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::architectural_impact::dependency_graph::{DependencyGraph, Node, NodeType, FunctionNode, NodeMetadata, Edge, EdgeType, Visibility};
    use crate::architectural_impact::{BreakingChange, BreakingChangeType, ImpactLevel};

    fn setup_graph() -> DependencyGraph {
        let mut g = DependencyGraph::new();
        for id in ["A","B","C","D","E"] {
            g.add_node(Node {
                id: id.to_string(),
                node_type: NodeType::Function(FunctionNode {
                    name: id.to_string(), visibility: Some(Visibility::Public), parameters: vec![], return_type: None, is_async: false
                }),
                metadata: NodeMetadata { file_path: format!("{}.rs", id), start_line: 1, end_line: 3, complexity: 1, created_at: 0 },
                importance_score: if id == "A" { 0.9 } else { 0.7 },
            });
        }
        // A 被 B 和 E 调用；B -> C -> D
        g.add_edge(Edge { from: "B".into(), to: "A".into(), edge_type: EdgeType::Calls, weight: 1.0, metadata: None });
        g.add_edge(Edge { from: "C".into(), to: "B".into(), edge_type: EdgeType::Calls, weight: 1.0, metadata: None });
        g.add_edge(Edge { from: "D".into(), to: "C".into(), edge_type: EdgeType::Calls, weight: 1.0, metadata: None });
        g.add_edge(Edge { from: "E".into(), to: "A".into(), edge_type: EdgeType::Calls, weight: 1.0, metadata: None });
        g.rebuild_adjacency_lists();
        g
    }

    #[test]
    fn test_detect_cascades_basic() {
        let graph = setup_graph();
        let detector = CascadeDetector::new(graph);
        let changes = vec![BreakingChange {
            change_type: BreakingChangeType::FunctionSignatureChanged,
            component: "A".to_string(),
            description: "A signature changed".to_string(),
            impact_level: ImpactLevel::Module,
            suggestions: vec![],
            before: None,
            after: None,
            file_path: "a.rs".to_string(),
        }];

        let effects = detector.detect_cascades(&changes);
        assert!(!effects.is_empty());
        // 至少应包含 A -> B 和 A -> E 的反向传播链（以B/E为终点）
        let chains: Vec<Vec<String>> = effects.into_iter().map(|e| e.affected_chain).collect();
        assert!(chains.iter().any(|c| c.starts_with(&vec!["A".to_string(), "B".to_string()]) || c.ends_with(&vec!["B".to_string()])));
    }

    #[test]
    fn test_identify_critical_nodes() {
        let graph = setup_graph();
        let detector = CascadeDetector::new(graph);
        let critical = detector.identify_critical_nodes();
        assert!(!critical.is_empty());
        assert!(critical[0].centrality >= detector.thresholds.critical_centrality);
    }
}


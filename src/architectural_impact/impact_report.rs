// 影响范围报告生成器：将 ImpactScope 转为 AI 友好的 Markdown 报告
use crate::architectural_impact::ImpactScope;
use crate::architectural_impact::dependency_graph::DependencyGraph;

pub fn generate_markdown_report(scope: &ImpactScope, graph: Option<&DependencyGraph>) -> String {
    let mut s = String::new();
    s.push_str("## 影响范围分析\n\n");

    // 影响统计
    s.push_str("### 📊 影响统计\n");
    s.push_str(&format!("- 影响半径: {:.2}\n", scope.impact_radius));
    s.push_str(&format!("- 影响深度: {} 层\n", scope.impact_depth));
    s.push_str(&format!("- 直接影响: {} 个组件\n", scope.direct_impacts.len()));
    s.push_str(&format!("- 间接影响: {} 个组件\n\n", scope.indirect_impacts.len()));

    // 直接影响组件（Top 10）
    if !scope.direct_impacts.is_empty() {
        s.push_str("### 🎯 直接影响组件 (Top 10)\n");
        for (i, c) in scope.direct_impacts.iter().take(10).enumerate() {
            let name = display_node(c.component_id.as_str(), graph);
            s.push_str(&format!(
                "{}. `{}` - 分数 {:.2} ({:?})\n",
                i + 1,
                name,
                c.impact_score,
                c.component_type
            ));
        }
        s.push_str("\n");
    }

    // 间接影响组件（Top 10）
    if !scope.indirect_impacts.is_empty() {
        s.push_str("### 🌊 间接影响组件 (Top 10)\n");
        for (i, c) in scope.indirect_impacts.iter().take(10).enumerate() {
            let name = display_node(c.component_id.as_str(), graph);
            s.push_str(&format!(
                "{}. `{}` - 分数 {:.2}，距离 {} 层 ({:?})\n",
                i + 1,
                name,
                c.impact_score,
                c.distance_from_change,
                c.component_type
            ));
        }
        s.push_str("\n");
    }

    // 关键传播路径
    if !scope.critical_paths.is_empty() {
        s.push_str("### 🔗 关键传播路径\n");
        for (i, p) in scope.critical_paths.iter().enumerate() {
            let line = p.nodes.iter()
                .map(|n| display_node(n, graph))
                .collect::<Vec<_>>()
                .join(" -> ");
            s.push_str(&format!("{}. {} (权重 {:.2})\n", i + 1, line, p.weight));
        }
        s.push_str("\n");
    }

    // 建议（基于统计）
    s.push_str("### ✅ 建议\n");
    if scope.statistics.high_impact_count > 0 {
        s.push_str("- 优先验证高影响分数组件的正确性\n");
    }
    if scope.impact_depth >= 3 {
        s.push_str("- 存在较深的传播链，建议添加集成测试覆盖\n");
    }
    if scope.impact_radius > 0.5 {
        s.push_str("- 影响范围较大，建议分阶段发布并启用灰度\n");
    }

    s
}

fn display_node(node_id: &str, graph: Option<&DependencyGraph>) -> String {
    if let Some(g) = graph {
        if let Some(node) = g.nodes.get(node_id) {
            match &node.node_type {
                crate::architectural_impact::dependency_graph::NodeType::Function(f) => format!("{}()", f.name),
                crate::architectural_impact::dependency_graph::NodeType::Class(c) => c.name.clone(),
                crate::architectural_impact::dependency_graph::NodeType::Module(m) => m.name.clone(),
                crate::architectural_impact::dependency_graph::NodeType::File(f) => f.path.clone(),
            }
        } else { node_id.to_string() }
    } else {
        node_id.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::architectural_impact::impact_propagation::{ImpactedComponent, PropagationType, ComponentType, ImpactPath, ImpactStatistics};

    #[test]
    fn test_generate_markdown_report_basic() {
        let scope = ImpactScope {
            source_nodes: vec!["A".to_string()],
            direct_impacts: vec![ImpactedComponent {
                component_id: "B".to_string(),
                component_type: ComponentType::Function,
                impact_score: 0.9,
                impact_reason: "直接调用或依赖关系".to_string(),
                distance_from_change: 1,
                propagation_type: PropagationType::Direct,
            }],
            indirect_impacts: vec![],
            impact_radius: 0.3,
            impact_depth: 1,
            critical_paths: vec![ImpactPath { nodes: vec!["A".to_string(), "B".to_string()], weight: 0.8, description: "test".to_string(), is_critical: true }],
            statistics: ImpactStatistics { total_impacted_nodes: 2, direct_impact_count: 1, indirect_impact_count: 0, average_impact_score: 0.9, max_propagation_depth: 1, high_impact_count: 1 },
        };
        let md = generate_markdown_report(&scope, None);
        assert!(md.contains("影响范围分析"));
        assert!(md.contains("直接影响组件"));
        assert!(md.contains("关键传播路径"));
    }
}


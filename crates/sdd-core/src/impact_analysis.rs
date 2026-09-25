use crate::change::{Change, RiskLevel};
use crate::traceability::{EdgeType, TraceabilityGraph};
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactResult {
    pub change_id: Uuid,
    pub direct_impact: Vec<ImpactNode>,
    pub indirect_impact: Vec<ImpactNode>,
    pub risk_assessment: RiskAssessment,
    pub propagation_path: Vec<PropagationStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactNode {
    pub artifact_id: Uuid,
    pub artifact_type: String,
    pub impact_level: ImpactLevel,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImpactLevel {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk: RiskLevel,
    pub breaking_changes: bool,
    pub affected_modules: HashSet<String>,
    pub affected_tests: HashSet<String>,
    pub requires_manual_review: bool,
    pub estimated_effort: EffortEstimate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EffortEstimate {
    Trivial,
    Small,
    Medium,
    Large,
    ExtraLarge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationStep {
    pub from: Uuid,
    pub to: Uuid,
    pub edge_type: EdgeType,
    pub step_number: usize,
}

pub struct ImpactAnalyzer<'a> {
    graph: &'a TraceabilityGraph,
}

impl<'a> ImpactAnalyzer<'a> {
    pub fn new(graph: &'a TraceabilityGraph) -> Self {
        Self { graph }
    }

    pub fn analyze(&self, change: &Change) -> ImpactResult {
        let mut direct_impact = Vec::new();
        let mut indirect_impact = Vec::new();
        let mut propagation_path = Vec::new();
        let mut affected_modules = HashSet::new();
        let mut affected_tests = HashSet::new();

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        for artifact_id_str in &change.affected_artifacts {
            if let Ok(artifact_id) = Uuid::parse_str(artifact_id_str) {
                if let Some(node) = self.graph.get_node(artifact_id) {
                    direct_impact.push(ImpactNode {
                        artifact_id,
                        artifact_type: format!("{:?}", node.node_type),
                        impact_level: ImpactLevel::Critical,
                        reason: "Directly affected by change".to_string(),
                    });
                    queue.push_back((artifact_id, 1));
                    visited.insert(artifact_id);
                }
            }
        }

        let mut step_number = 1;
        while let Some((current_id, depth)) = queue.pop_front() {
            let neighbors = self.graph.get_downstream(current_id);

            for neighbor_id in neighbors {
                if visited.contains(&neighbor_id) {
                    continue;
                }

                visited.insert(neighbor_id);
                step_number += 1;

                if let Some(edge) = self
                    .graph
                    .edges
                    .iter()
                    .find(|e| e.source == current_id && e.target == neighbor_id)
                {
                    propagation_path.push(PropagationStep {
                        from: current_id,
                        to: neighbor_id,
                        edge_type: edge.edge_type.clone(),
                        step_number,
                    });
                }

                if let Some(node) = self.graph.get_node(neighbor_id) {
                    let impact_level = match depth {
                        1 => ImpactLevel::High,
                        2 => ImpactLevel::Medium,
                        _ => ImpactLevel::Low,
                    };

                    let impact_node = ImpactNode {
                        artifact_id: neighbor_id,
                        artifact_type: format!("{:?}", node.node_type),
                        impact_level: impact_level.clone(),
                        reason: format!("Indirectly affected via propagation (depth: {})", depth),
                    };

                    if impact_level == ImpactLevel::High {
                        direct_impact.push(impact_node);
                    } else {
                        indirect_impact.push(impact_node);
                    }

                    if depth < 3 {
                        queue.push_back((neighbor_id, depth + 1));
                    }

                    match format!("{:?}", node.node_type).as_str() {
                        "Task" => {
                            affected_modules.insert(node.artifact_id.clone());
                        }
                        "Test" => {
                            affected_tests.insert(node.artifact_id.clone());
                        }
                        _ => {}
                    }
                }
            }
        }

        let breaking_changes = !change.impact.breaking_changes.is_empty();
        let overall_risk =
            self.calculate_overall_risk(&direct_impact, &indirect_impact, breaking_changes);

        let estimated_effort = self.estimate_effort(&direct_impact, &indirect_impact);
        let requires_manual_review = overall_risk == RiskLevel::High
            || overall_risk == RiskLevel::Critical
            || breaking_changes;

        let risk_assessment = RiskAssessment {
            overall_risk,
            breaking_changes,
            affected_modules,
            affected_tests,
            requires_manual_review,
            estimated_effort,
        };

        ImpactResult {
            change_id: change.id,
            direct_impact,
            indirect_impact,
            risk_assessment,
            propagation_path,
        }
    }

    fn calculate_overall_risk(
        &self,
        direct: &[ImpactNode],
        indirect: &[ImpactNode],
        breaking: bool,
    ) -> RiskLevel {
        if breaking {
            return RiskLevel::Critical;
        }

        let critical_count = direct
            .iter()
            .filter(|n| n.impact_level == ImpactLevel::Critical)
            .count();
        let high_count = direct
            .iter()
            .filter(|n| n.impact_level == ImpactLevel::High)
            .count();

        if critical_count > 0 || high_count > 3 {
            return RiskLevel::High;
        }

        if high_count > 0 || indirect.len() > 5 {
            return RiskLevel::Medium;
        }

        RiskLevel::Low
    }

    fn estimate_effort(&self, direct: &[ImpactNode], indirect: &[ImpactNode]) -> EffortEstimate {
        let total_affected = direct.len() + indirect.len();

        match total_affected {
            0..=1 => EffortEstimate::Trivial,
            2..=3 => EffortEstimate::Small,
            4..=7 => EffortEstimate::Medium,
            8..=15 => EffortEstimate::Large,
            _ => EffortEstimate::ExtraLarge,
        }
    }

    pub fn get_affected_artifacts(&self, impact: &ImpactResult) -> Vec<Uuid> {
        let mut artifacts = Vec::new();

        for node in &impact.direct_impact {
            artifacts.push(node.artifact_id);
        }

        for node in &impact.indirect_impact {
            artifacts.push(node.artifact_id);
        }

        artifacts
    }

    pub fn get_propagation_chain(&self, impact: &ImpactResult) -> Vec<String> {
        impact
            .propagation_path
            .iter()
            .map(|step| format!("{} --[{:?}]--> {}", step.from, step.edge_type, step.to))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change::{Change, ChangeOrigin, ChangeType};
    use crate::traceability::{GraphEdge, GraphNode, NodeType};

    fn create_test_graph() -> TraceabilityGraph {
        let mut graph = TraceabilityGraph::new();

        let req_node = GraphNode::new(Uuid::new_v4(), NodeType::Requirement, "REQ-001");
        let task_node = GraphNode::new(Uuid::new_v4(), NodeType::Task, "TASK-001");
        let test_node = GraphNode::new(Uuid::new_v4(), NodeType::Test, "TEST-001");

        graph.add_node(req_node.clone());
        graph.add_node(task_node.clone());
        graph.add_node(test_node.clone());

        let edge1 = GraphEdge::new(req_node.id, task_node.id, EdgeType::Implements);
        let edge2 = GraphEdge::new(task_node.id, test_node.id, EdgeType::Tests);

        graph.add_edge(edge1).unwrap();
        graph.add_edge(edge2).unwrap();

        graph
    }

    #[test]
    fn test_impact_analyzer_creation() {
        let graph = create_test_graph();
        let analyzer = ImpactAnalyzer::new(&graph);
        let _ = analyzer;
    }

    #[test]
    fn test_analyze_simple_change() {
        let graph = create_test_graph();
        let analyzer = ImpactAnalyzer::new(&graph);

        let nodes: Vec<_> = graph.nodes.values().collect();
        let req_node = nodes
            .iter()
            .find(|n| n.node_type == NodeType::Requirement)
            .unwrap();

        let change = Change::new(
            "CHG-001",
            "Update requirement",
            "Changed requirement",
            ChangeType::Requirement,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        )
        .add_affected_artifact(req_node.id.to_string());

        let impact = analyzer.analyze(&change);
        assert!(!impact.direct_impact.is_empty());
    }

    #[test]
    fn test_risk_assessment() {
        let graph = create_test_graph();
        let analyzer = ImpactAnalyzer::new(&graph);

        let nodes: Vec<_> = graph.nodes.values().collect();
        let req_node = nodes
            .iter()
            .find(|n| n.node_type == NodeType::Requirement)
            .unwrap();

        let change = Change::new(
            "CHG-001",
            "Update requirement",
            "Changed requirement",
            ChangeType::Requirement,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        )
        .add_affected_artifact(req_node.id.to_string());

        let impact = analyzer.analyze(&change);
        // Risk can be Low, Medium, or High depending on the graph structure
        assert!(
            impact.risk_assessment.overall_risk == RiskLevel::Low
                || impact.risk_assessment.overall_risk == RiskLevel::Medium
                || impact.risk_assessment.overall_risk == RiskLevel::High
        );
    }

    #[test]
    fn test_effort_estimation() {
        let graph = create_test_graph();
        let analyzer = ImpactAnalyzer::new(&graph);

        let nodes: Vec<_> = graph.nodes.values().collect();
        let req_node = nodes
            .iter()
            .find(|n| n.node_type == NodeType::Requirement)
            .unwrap();

        let change = Change::new(
            "CHG-001",
            "Update requirement",
            "Changed requirement",
            ChangeType::Requirement,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        )
        .add_affected_artifact(req_node.id.to_string());

        let impact = analyzer.analyze(&change);
        assert!(
            impact.risk_assessment.estimated_effort == EffortEstimate::Trivial
                || impact.risk_assessment.estimated_effort == EffortEstimate::Small
        );
    }
}

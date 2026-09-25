use crate::change::Change;
use crate::impact_analysis::ImpactResult;
use crate::traceability::TraceabilityGraph;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationPlan {
    pub id: Uuid,
    pub change_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub steps: Vec<PropagationStep>,
    pub status: PropagationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationStep {
    pub step_id: Uuid,
    pub artifact_id: Uuid,
    pub artifact_type: String,
    pub action: PropagationAction,
    pub priority: PropagationPriority,
    pub status: StepStatus,
    pub dependencies: Vec<Uuid>,
    pub estimated_duration: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PropagationAction {
    Update,
    Regenerate,
    Review,
    Notify,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PropagationPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PropagationStatus {
    Planning,
    Ready,
    Executing,
    Completed,
    Failed,
    Cancelled,
}

pub struct ChangePropagator<'a> {
    graph: &'a TraceabilityGraph,
}

impl<'a> ChangePropagator<'a> {
    pub fn new(graph: &'a TraceabilityGraph) -> Self {
        Self { graph }
    }

    pub fn create_propagation_plan(
        &self,
        change: &Change,
        impact: &ImpactResult,
    ) -> PropagationPlan {
        let mut steps = Vec::new();

        for impact_node in &impact.direct_impact {
            let action = self.determine_action(&impact_node.artifact_type);
            let priority = self.determine_priority(&impact_node.impact_level);

            steps.push(PropagationStep {
                step_id: Uuid::new_v4(),
                artifact_id: impact_node.artifact_id,
                artifact_type: impact_node.artifact_type.clone(),
                action,
                priority,
                status: StepStatus::Pending,
                dependencies: Vec::new(),
                estimated_duration: Some("5m".to_string()),
            });
        }

        for impact_node in &impact.indirect_impact {
            let action = self.determine_action(&impact_node.artifact_type);
            let priority = self.determine_priority(&impact_node.impact_level);

            let mut dependencies = Vec::new();
            for step in &steps {
                if self.has_dependency(&step.artifact_id, &impact_node.artifact_id) {
                    dependencies.push(step.step_id);
                }
            }

            steps.push(PropagationStep {
                step_id: Uuid::new_v4(),
                artifact_id: impact_node.artifact_id,
                artifact_type: impact_node.artifact_type.clone(),
                action,
                priority,
                status: StepStatus::Pending,
                dependencies,
                estimated_duration: Some("10m".to_string()),
            });
        }

        steps.sort_by(|a, b| {
            let priority_order = |p: &PropagationPriority| match p {
                PropagationPriority::Critical => 0,
                PropagationPriority::High => 1,
                PropagationPriority::Medium => 2,
                PropagationPriority::Low => 3,
            };
            priority_order(&a.priority).cmp(&priority_order(&b.priority))
        });

        PropagationPlan {
            id: Uuid::new_v4(),
            change_id: change.id,
            created_at: Utc::now(),
            steps,
            status: PropagationStatus::Ready,
        }
    }

    fn determine_action(&self, artifact_type: &str) -> PropagationAction {
        match artifact_type {
            "Requirement" => PropagationAction::Review,
            "Task" => PropagationAction::Update,
            "Test" => PropagationAction::Regenerate,
            "Documentation" => PropagationAction::Update,
            "Architecture" => PropagationAction::Review,
            _ => PropagationAction::Notify,
        }
    }

    fn determine_priority(
        &self,
        impact_level: &crate::impact_analysis::ImpactLevel,
    ) -> PropagationPriority {
        match impact_level {
            crate::impact_analysis::ImpactLevel::Critical => PropagationPriority::Critical,
            crate::impact_analysis::ImpactLevel::High => PropagationPriority::High,
            crate::impact_analysis::ImpactLevel::Medium => PropagationPriority::Medium,
            crate::impact_analysis::ImpactLevel::Low => PropagationPriority::Low,
            crate::impact_analysis::ImpactLevel::Informational => PropagationPriority::Low,
        }
    }

    fn has_dependency(&self, from: &Uuid, to: &Uuid) -> bool {
        self.graph
            .edges
            .iter()
            .any(|e| e.source == *from && e.target == *to)
    }

    pub fn get_execution_order<'b>(&self, plan: &'b PropagationPlan) -> Vec<&'b PropagationStep> {
        let mut ordered = Vec::new();
        let mut visited = HashSet::new();

        for step in &plan.steps {
            self.topological_sort(step, &plan.steps, &mut visited, &mut ordered);
        }

        ordered
    }

    fn topological_sort<'b>(
        &self,
        step: &'b PropagationStep,
        all_steps: &'b [PropagationStep],
        visited: &mut HashSet<Uuid>,
        ordered: &mut Vec<&'b PropagationStep>,
    ) {
        if visited.contains(&step.step_id) {
            return;
        }

        for dep_id in &step.dependencies {
            if let Some(dep_step) = all_steps.iter().find(|s| s.step_id == *dep_id) {
                self.topological_sort(dep_step, all_steps, visited, ordered);
            }
        }

        visited.insert(step.step_id);
        ordered.push(step);
    }

    pub fn mark_step_completed(&self, plan: &mut PropagationPlan, step_id: Uuid) -> bool {
        if let Some(step) = plan.steps.iter_mut().find(|s| s.step_id == step_id) {
            step.status = StepStatus::Completed;

            if plan
                .steps
                .iter()
                .all(|s| s.status == StepStatus::Completed || s.status == StepStatus::Skipped)
            {
                plan.status = PropagationStatus::Completed;
            } else {
                plan.status = PropagationStatus::Executing;
            }

            true
        } else {
            false
        }
    }

    pub fn mark_step_failed(&self, plan: &mut PropagationPlan, step_id: Uuid) -> bool {
        if let Some(step) = plan.steps.iter_mut().find(|s| s.step_id == step_id) {
            step.status = StepStatus::Failed;
            plan.status = PropagationStatus::Failed;
            true
        } else {
            false
        }
    }

    pub fn get_pending_steps<'b>(&self, plan: &'b PropagationPlan) -> Vec<&'b PropagationStep> {
        plan.steps
            .iter()
            .filter(|s| s.status == StepStatus::Pending)
            .collect()
    }

    pub fn get_completed_steps<'b>(&self, plan: &'b PropagationPlan) -> Vec<&'b PropagationStep> {
        plan.steps
            .iter()
            .filter(|s| s.status == StepStatus::Completed)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change::{Change, ChangeOrigin, ChangeType};
    use crate::impact_analysis::{EffortEstimate, ImpactLevel, ImpactNode, RiskAssessment};
    use crate::traceability::{EdgeType, GraphEdge, GraphNode, NodeType};
    use std::collections::HashSet;

    fn create_test_graph() -> TraceabilityGraph {
        let mut graph = TraceabilityGraph::new();

        let req_node = GraphNode::new(Uuid::new_v4(), NodeType::Requirement, "REQ-001");
        let task_node = GraphNode::new(Uuid::new_v4(), NodeType::Task, "TASK-001");

        graph.add_node(req_node.clone());
        graph.add_node(task_node.clone());

        let edge = GraphEdge::new(req_node.id, task_node.id, EdgeType::Implements);
        graph.add_edge(edge).unwrap();

        graph
    }

    fn create_test_impact() -> ImpactResult {
        ImpactResult {
            change_id: Uuid::new_v4(),
            direct_impact: vec![ImpactNode {
                artifact_id: Uuid::new_v4(),
                artifact_type: "Requirement".to_string(),
                impact_level: ImpactLevel::High,
                reason: "Direct".to_string(),
            }],
            indirect_impact: vec![ImpactNode {
                artifact_id: Uuid::new_v4(),
                artifact_type: "Task".to_string(),
                impact_level: ImpactLevel::Medium,
                reason: "Indirect".to_string(),
            }],
            risk_assessment: RiskAssessment {
                overall_risk: crate::change::RiskLevel::Medium,
                breaking_changes: false,
                affected_modules: HashSet::new(),
                affected_tests: HashSet::new(),
                requires_manual_review: false,
                estimated_effort: EffortEstimate::Small,
            },
            propagation_path: Vec::new(),
        }
    }

    #[test]
    fn test_propagator_creation() {
        let graph = create_test_graph();
        let propagator = ChangePropagator::new(&graph);
        let _ = propagator;
    }

    #[test]
    fn test_create_propagation_plan() {
        let graph = create_test_graph();
        let propagator = ChangePropagator::new(&graph);

        let change = Change::new(
            "CHG-001",
            "Test",
            "Test",
            ChangeType::Requirement,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        );

        let impact = create_test_impact();
        let plan = propagator.create_propagation_plan(&change, &impact);

        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.status, PropagationStatus::Ready);
    }

    #[test]
    fn test_mark_step_completed() {
        let graph = create_test_graph();
        let propagator = ChangePropagator::new(&graph);

        let change = Change::new(
            "CHG-001",
            "Test",
            "Test",
            ChangeType::Requirement,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        );

        let impact = create_test_impact();
        let mut plan = propagator.create_propagation_plan(&change, &impact);

        let step_id = plan.steps[0].step_id;
        assert!(propagator.mark_step_completed(&mut plan, step_id));
        assert_eq!(plan.steps[0].status, StepStatus::Completed);
    }

    #[test]
    fn test_get_pending_steps() {
        let graph = create_test_graph();
        let propagator = ChangePropagator::new(&graph);

        let change = Change::new(
            "CHG-001",
            "Test",
            "Test",
            ChangeType::Requirement,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        );

        let impact = create_test_impact();
        let plan = propagator.create_propagation_plan(&change, &impact);

        let pending = propagator.get_pending_steps(&plan);
        assert_eq!(pending.len(), 2);
    }
}

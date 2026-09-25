use crate::architecture::Architecture;
use crate::context_bundle::{
    ContextError, ContextFragment, ContextPriority, ContextSource, TokenBudget,
};
use crate::requirement::Requirement;
use crate::task::Task;
use crate::traceability::{NodeType, TraceabilityGraph};
use std::collections::HashSet;
use uuid::Uuid;

pub struct ExplicitRelationResolver<'a> {
    graph: &'a TraceabilityGraph,
}

impl<'a> ExplicitRelationResolver<'a> {
    pub fn new(graph: &'a TraceabilityGraph) -> Self {
        Self { graph }
    }

    pub fn resolve_task_context(
        &self,
        task: &Task,
        budget: &mut TokenBudget,
    ) -> Result<Vec<ContextFragment>, ContextError> {
        let mut fragments = Vec::new();
        let mut visited = HashSet::new();

        let task_fragment = ContextFragment::new(
            self.render_task(task),
            ContextSource::Task,
            task.task_id.clone(),
            ContextPriority::P0,
            "Primary task definition".to_string(),
        );
        if task_fragment.estimated_tokens <= budget.remaining() {
            budget.used += task_fragment.estimated_tokens;
            fragments.push(task_fragment);
        }

        let neighbors = self.graph.get_neighbors(task.id);
        for node_id in neighbors {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id);

            if let Some(node) = self.graph.get_node(node_id) {
                if let Some(fragment) = self.node_to_fragment(node, &task.id) {
                    if fragment.estimated_tokens <= budget.remaining() {
                        budget.used += fragment.estimated_tokens;
                        fragments.push(fragment);
                    }
                }
            }
        }

        Ok(fragments)
    }

    pub fn resolve_requirements(
        &self,
        task: &Task,
        requirements: &[Requirement],
    ) -> Vec<ContextFragment> {
        let mut fragments = Vec::new();

        let related_node_ids: HashSet<Uuid> = self.graph.get_neighbors(task.id);

        for req in requirements {
            if related_node_ids.iter().any(|id| {
                self.graph
                    .get_node(*id)
                    .map(|n| n.artifact_id == req.req_id)
                    .unwrap_or(false)
            }) {
                let fragment = ContextFragment::new(
                    self.render_requirement(req),
                    ContextSource::Requirement,
                    req.req_id.clone(),
                    ContextPriority::P1,
                    format!("Related requirement for task {}", task.task_id),
                );
                fragments.push(fragment);
            }
        }

        fragments
    }

    pub fn resolve_dependencies(&self, task: &Task, all_tasks: &[Task]) -> Vec<ContextFragment> {
        let mut fragments = Vec::new();

        for dep_id in &task.dependencies {
            if let Some(dep_task) = all_tasks.iter().find(|t| t.task_id == *dep_id) {
                let fragment = ContextFragment::new(
                    self.render_task_summary(dep_task),
                    ContextSource::Dependency,
                    dep_task.task_id.clone(),
                    ContextPriority::P1,
                    format!("Dependency: {}", dep_task.task_id),
                );
                fragments.push(fragment);
            }
        }

        fragments
    }

    pub fn resolve_architecture(
        &self,
        task: &Task,
        architecture: &Architecture,
    ) -> Option<ContextFragment> {
        let neighbors = self.graph.get_neighbors(task.id);

        let is_related = neighbors.iter().any(|id| {
            self.graph
                .get_node(*id)
                .map(|n| {
                    n.node_type == NodeType::Architecture && n.artifact_id == architecture.name
                })
                .unwrap_or(false)
        });

        if is_related {
            Some(ContextFragment::new(
                self.render_architecture(architecture),
                ContextSource::Architecture,
                architecture.name.clone(),
                ContextPriority::P1,
                "Related architecture".to_string(),
            ))
        } else {
            None
        }
    }

    fn node_to_fragment(
        &self,
        node: &crate::traceability::GraphNode,
        task_id: &Uuid,
    ) -> Option<ContextFragment> {
        let (content, source, priority) = match node.node_type {
            NodeType::Task => (
                format!("Task: {}", node.artifact_id),
                ContextSource::Task,
                ContextPriority::P1,
            ),
            NodeType::Requirement => (
                format!("Requirement: {}", node.artifact_id),
                ContextSource::Requirement,
                ContextPriority::P1,
            ),
            NodeType::Decision => (
                format!("Decision: {}", node.artifact_id),
                ContextSource::Decision,
                ContextPriority::P2,
            ),
            NodeType::Artifact => (
                format!("Artifact: {}", node.artifact_id),
                ContextSource::Documentation,
                ContextPriority::P2,
            ),
            NodeType::Adr => (
                format!("ADR: {}", node.artifact_id),
                ContextSource::Decision,
                ContextPriority::P1,
            ),
            _ => return None,
        };

        Some(ContextFragment::new(
            content,
            source,
            node.artifact_id.clone(),
            priority,
            format!("Related to task {}", task_id),
        ))
    }

    fn render_task(&self, task: &Task) -> String {
        let mut output = String::new();
        output.push_str(&format!("## Task: {}\n", task.task_id));
        output.push_str(&format!("**Title:** {}\n\n", task.title));
        output.push_str(&format!("**Description:**\n{}\n\n", task.description));
        output.push_str(&format!("**Status:** {:?}\n", task.status));

        if !task.dependencies.is_empty() {
            output.push_str(&format!(
                "**Dependencies:** {}\n",
                task.dependencies.join(", ")
            ));
        }

        if let Some(agent) = &task.assigned_agent {
            output.push_str(&format!("**Assigned Agent:** {}\n", agent));
        }

        output
    }

    fn render_task_summary(&self, task: &Task) -> String {
        format!(
            "Task {} ({}): {}\nStatus: {:?}\n",
            task.task_id,
            task.title,
            task.description.lines().next().unwrap_or(""),
            task.status
        )
    }

    fn render_requirement(&self, req: &Requirement) -> String {
        let mut output = String::new();
        output.push_str(&format!("## Requirement: {}\n", req.req_id));
        output.push_str(&format!("**Title:** {}\n\n", req.title));
        output.push_str(&format!("**Description:**\n{}\n\n", req.description));
        output.push_str(&format!("**Priority:** {:?}\n", req.priority));
        output.push_str(&format!("**Status:** {:?}\n", req.status));

        if !req.acceptance_criteria.is_empty() {
            output.push_str("\n**Acceptance Criteria:**\n");
            for criterion in &req.acceptance_criteria {
                output.push_str(&format!("- {}\n", criterion.description));
            }
        }

        output
    }

    fn render_architecture(&self, arch: &Architecture) -> String {
        let mut output = String::new();
        output.push_str(&format!("## Architecture: {}\n", arch.name));
        output.push_str(&format!("**Style:** {:?}\n\n", arch.style));

        if !arch.components.is_empty() {
            output.push_str("**Components:**\n");
            for comp in &arch.components {
                output.push_str(&format!("- {}\n", comp.name));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Task;
    use crate::traceability::{GraphNode, NodeType, TraceabilityGraph};

    #[test]
    fn test_explicit_resolver_creation() {
        let graph = TraceabilityGraph::new();
        let resolver = ExplicitRelationResolver::new(&graph);
        assert_eq!(resolver.graph.node_count(), 0);
    }

    #[test]
    fn test_resolve_task_context() {
        let mut graph = TraceabilityGraph::new();
        let task = Task::new("TASK-001", "Implement feature", "Add new feature");

        let node = GraphNode::new(task.id, NodeType::Task, &task.task_id);
        graph.add_node(node);

        let resolver = ExplicitRelationResolver::new(&graph);
        let mut budget = TokenBudget::new(4000);
        let fragments = resolver.resolve_task_context(&task, &mut budget).unwrap();

        assert!(!fragments.is_empty());
        assert_eq!(fragments[0].source, ContextSource::Task);
    }

    #[test]
    fn test_resolve_dependencies() {
        let graph = TraceabilityGraph::new();
        let mut task = Task::new("TASK-002", "Test feature", "Write tests");
        task.dependencies.push("TASK-001".to_string());

        let dep_task = Task::new("TASK-001", "Implement feature", "Add new feature");
        let all_tasks = vec![dep_task, task.clone()];

        let resolver = ExplicitRelationResolver::new(&graph);
        let fragments = resolver.resolve_dependencies(&task, &all_tasks);

        assert_eq!(fragments.len(), 1);
        assert_eq!(fragments[0].source, ContextSource::Dependency);
    }
}

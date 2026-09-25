use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceabilityGraph {
    pub nodes: HashMap<Uuid, GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: Uuid,
    pub node_type: NodeType,
    pub artifact_id: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    Brief,
    Requirement,
    Question,
    Decision,
    Assumption,
    Risk,
    Architecture,
    Adr,
    Task,
    Artifact,
    Test,
    Review,
    Change,
    Commit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: Uuid,
    pub source: Uuid,
    pub target: Uuid,
    pub edge_type: EdgeType,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeType {
    Implements,
    Verifies,
    DependsOn,
    Supersedes,
    RelatedTo,
    Triggers,
    Affects,
    Documents,
    Tests,
    Reviews,
    Resolves,
    Parent,
    Child,
}

impl TraceabilityGraph {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_node(&mut self, node: GraphNode) {
        self.nodes.insert(node.id, node);
        self.updated_at = Utc::now();
    }

    pub fn add_edge(&mut self, edge: GraphEdge) -> Result<(), GraphError> {
        if !self.nodes.contains_key(&edge.source) {
            return Err(GraphError::NodeNotFound(edge.source));
        }
        if !self.nodes.contains_key(&edge.target) {
            return Err(GraphError::NodeNotFound(edge.target));
        }
        self.edges.push(edge);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn remove_node(&mut self, id: Uuid) -> bool {
        let removed = self.nodes.remove(&id).is_some();
        if removed {
            self.edges.retain(|e| e.source != id && e.target != id);
            self.updated_at = Utc::now();
        }
        removed
    }

    pub fn get_node(&self, id: Uuid) -> Option<&GraphNode> {
        self.nodes.get(&id)
    }

    pub fn get_edges_from(&self, id: Uuid) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.source == id).collect()
    }

    pub fn get_edges_to(&self, id: Uuid) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.target == id).collect()
    }

    pub fn get_neighbors(&self, id: Uuid) -> HashSet<Uuid> {
        let mut neighbors = HashSet::new();
        for edge in &self.edges {
            if edge.source == id {
                neighbors.insert(edge.target);
            }
            if edge.target == id {
                neighbors.insert(edge.source);
            }
        }
        neighbors
    }

    pub fn get_upstream(&self, id: Uuid) -> HashSet<Uuid> {
        self.edges
            .iter()
            .filter(|e| e.target == id)
            .map(|e| e.source)
            .collect()
    }

    pub fn get_downstream(&self, id: Uuid) -> HashSet<Uuid> {
        self.edges
            .iter()
            .filter(|e| e.source == id)
            .map(|e| e.target)
            .collect()
    }

    pub fn find_path(&self, from: Uuid, to: Uuid) -> Option<Vec<Uuid>> {
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        if self.dfs_path(from, to, &mut visited, &mut path) {
            Some(path)
        } else {
            None
        }
    }

    fn dfs_path(
        &self,
        current: Uuid,
        target: Uuid,
        visited: &mut HashSet<Uuid>,
        path: &mut Vec<Uuid>,
    ) -> bool {
        if visited.contains(&current) {
            return false;
        }
        visited.insert(current);
        path.push(current);

        if current == target {
            return true;
        }

        for neighbor in self.get_downstream(current) {
            if self.dfs_path(neighbor, target, visited, path) {
                return true;
            }
        }

        path.pop();
        false
    }

    pub fn get_traceability_chain(&self, from: Uuid) -> Vec<(Uuid, EdgeType, Uuid)> {
        let mut chain = Vec::new();
        let mut visited = HashSet::new();
        self.collect_chain(from, &mut chain, &mut visited);
        chain
    }

    fn collect_chain(
        &self,
        current: Uuid,
        chain: &mut Vec<(Uuid, EdgeType, Uuid)>,
        visited: &mut HashSet<Uuid>,
    ) {
        if visited.contains(&current) {
            return;
        }
        visited.insert(current);

        for edge in self.get_edges_from(current) {
            chain.push((edge.source, edge.edge_type.clone(), edge.target));
            self.collect_chain(edge.target, chain, visited);
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn get_nodes_by_type(&self, node_type: NodeType) -> Vec<&GraphNode> {
        self.nodes
            .values()
            .filter(|n| n.node_type == node_type)
            .collect()
    }

    pub fn get_edges_by_type(&self, edge_type: EdgeType) -> Vec<&GraphEdge> {
        self.edges
            .iter()
            .filter(|e| e.edge_type == edge_type)
            .collect()
    }
}

impl Default for TraceabilityGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphNode {
    pub fn new(id: Uuid, node_type: NodeType, artifact_id: impl Into<String>) -> Self {
        Self {
            id,
            node_type,
            artifact_id: artifact_id.into(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl GraphEdge {
    pub fn new(source: Uuid, target: Uuid, edge_type: EdgeType) -> Self {
        Self {
            id: Uuid::new_v4(),
            source,
            target,
            edge_type,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("Node not found: {0}")]
    NodeNotFound(Uuid),
    #[error("Edge already exists")]
    EdgeAlreadyExists,
    #[error("Cycle detected")]
    CycleDetected,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_graph() -> TraceabilityGraph {
        let mut graph = TraceabilityGraph::new();

        let req_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();
        let test_id = Uuid::new_v4();

        graph.add_node(GraphNode::new(req_id, NodeType::Requirement, "REQ-001"));
        graph.add_node(GraphNode::new(task_id, NodeType::Task, "TASK-001"));
        graph.add_node(GraphNode::new(test_id, NodeType::Test, "TEST-001"));

        graph
            .add_edge(GraphEdge::new(task_id, req_id, EdgeType::Implements))
            .unwrap();
        graph
            .add_edge(GraphEdge::new(test_id, task_id, EdgeType::Verifies))
            .unwrap();

        graph
    }

    #[test]
    fn test_graph_creation() {
        let graph = TraceabilityGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_add_node() {
        let mut graph = TraceabilityGraph::new();
        let node = GraphNode::new(Uuid::new_v4(), NodeType::Requirement, "REQ-001");
        graph.add_node(node);
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn test_add_edge() {
        let mut graph = TraceabilityGraph::new();
        let source = Uuid::new_v4();
        let target = Uuid::new_v4();

        graph.add_node(GraphNode::new(source, NodeType::Task, "TASK-001"));
        graph.add_node(GraphNode::new(target, NodeType::Requirement, "REQ-001"));
        graph
            .add_edge(GraphEdge::new(source, target, EdgeType::Implements))
            .unwrap();

        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_edge_to_nonexistent_node() {
        let mut graph = TraceabilityGraph::new();
        let source = Uuid::new_v4();
        let target = Uuid::new_v4();

        graph.add_node(GraphNode::new(source, NodeType::Task, "TASK-001"));
        let result = graph.add_edge(GraphEdge::new(source, target, EdgeType::Implements));

        assert!(result.is_err());
    }

    #[test]
    fn test_get_neighbors() {
        let graph = create_test_graph();
        let task_id = graph.get_nodes_by_type(NodeType::Task)[0].id;
        let neighbors = graph.get_neighbors(task_id);
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn test_get_upstream_downstream() {
        let graph = create_test_graph();
        let task_id = graph.get_nodes_by_type(NodeType::Task)[0].id;

        let upstream = graph.get_upstream(task_id);
        assert_eq!(upstream.len(), 1);

        let downstream = graph.get_downstream(task_id);
        assert_eq!(downstream.len(), 1);
    }

    #[test]
    fn test_find_path() {
        let graph = create_test_graph();
        let test_id = graph.get_nodes_by_type(NodeType::Test)[0].id;
        let req_id = graph.get_nodes_by_type(NodeType::Requirement)[0].id;

        let path = graph.find_path(test_id, req_id);
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 3);
    }

    #[test]
    fn test_remove_node() {
        let mut graph = create_test_graph();
        let initial_edges = graph.edge_count();

        let task_id = graph.get_nodes_by_type(NodeType::Task)[0].id;
        graph.remove_node(task_id);

        assert_eq!(graph.node_count(), 2);
        assert!(graph.edge_count() < initial_edges);
    }

    #[test]
    fn test_get_nodes_by_type() {
        let graph = create_test_graph();
        let requirements = graph.get_nodes_by_type(NodeType::Requirement);
        assert_eq!(requirements.len(), 1);
    }
}

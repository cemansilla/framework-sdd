use sdd_core::*;
use uuid::Uuid;

#[test]
fn test_change_propagation_simple() {
    let mut graph = TraceabilityGraph::new();

    let req_node = GraphNode::new(Uuid::new_v4(), NodeType::Requirement, "REQ-001");
    let task_node = GraphNode::new(Uuid::new_v4(), NodeType::Task, "TASK-001");

    graph.add_node(req_node.clone());
    graph.add_node(task_node.clone());

    let edge = GraphEdge::new(req_node.id, task_node.id, EdgeType::Implements);
    graph.add_edge(edge).unwrap();

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

    let analyzer = ImpactAnalyzer::new(&graph);
    let impact = analyzer.analyze(&change);

    assert!(!impact.direct_impact.is_empty());
}

#[test]
fn test_change_propagation_chain() {
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

    let analyzer = ImpactAnalyzer::new(&graph);
    let impact = analyzer.analyze(&change);

    assert!(!impact.direct_impact.is_empty());
    assert!(!impact.propagation_path.is_empty());
}

#[test]
fn test_change_propagation_with_propagator() {
    let mut graph = TraceabilityGraph::new();

    let req_node = GraphNode::new(Uuid::new_v4(), NodeType::Requirement, "REQ-001");
    let task_node = GraphNode::new(Uuid::new_v4(), NodeType::Task, "TASK-001");

    graph.add_node(req_node.clone());
    graph.add_node(task_node.clone());

    let edge = GraphEdge::new(req_node.id, task_node.id, EdgeType::Implements);
    graph.add_edge(edge).unwrap();

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

    let analyzer = ImpactAnalyzer::new(&graph);
    let impact = analyzer.analyze(&change);

    let propagator = ChangePropagator::new(&graph);
    let plan = propagator.create_propagation_plan(&change, &impact);

    assert!(!plan.steps.is_empty());
    assert_eq!(plan.status, PropagationStatus::Ready);
}

#[test]
fn test_change_propagation_multiple_artifacts() {
    let mut graph = TraceabilityGraph::new();

    let req_node1 = GraphNode::new(Uuid::new_v4(), NodeType::Requirement, "REQ-001");
    let req_node2 = GraphNode::new(Uuid::new_v4(), NodeType::Requirement, "REQ-002");
    let task_node = GraphNode::new(Uuid::new_v4(), NodeType::Task, "TASK-001");

    graph.add_node(req_node1.clone());
    graph.add_node(req_node2.clone());
    graph.add_node(task_node.clone());

    let edge1 = GraphEdge::new(req_node1.id, task_node.id, EdgeType::Implements);
    let edge2 = GraphEdge::new(req_node2.id, task_node.id, EdgeType::Implements);

    graph.add_edge(edge1).unwrap();
    graph.add_edge(edge2).unwrap();

    let change = Change::new(
        "CHG-001",
        "Update task",
        "Changed task",
        ChangeType::Implementation,
        ChangeOrigin::User {
            reason: "Test".to_string(),
        },
    )
    .add_affected_artifact(task_node.id.to_string());

    let analyzer = ImpactAnalyzer::new(&graph);
    let impact = analyzer.analyze(&change);

    assert!(!impact.direct_impact.is_empty());
}

#[test]
fn test_change_propagation_risk_assessment() {
    let mut graph = TraceabilityGraph::new();

    let req_node = GraphNode::new(Uuid::new_v4(), NodeType::Requirement, "REQ-001");
    let task_node = GraphNode::new(Uuid::new_v4(), NodeType::Task, "TASK-001");

    graph.add_node(req_node.clone());
    graph.add_node(task_node.clone());

    let edge = GraphEdge::new(req_node.id, task_node.id, EdgeType::Implements);
    graph.add_edge(edge).unwrap();

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

    let analyzer = ImpactAnalyzer::new(&graph);
    let impact = analyzer.analyze(&change);

    assert!(
        impact.risk_assessment.overall_risk == RiskLevel::Low
            || impact.risk_assessment.overall_risk == RiskLevel::Medium
            || impact.risk_assessment.overall_risk == RiskLevel::High
    );
}

#[test]
fn test_change_propagation_execution_order() {
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

    let analyzer = ImpactAnalyzer::new(&graph);
    let impact = analyzer.analyze(&change);

    let propagator = ChangePropagator::new(&graph);
    let plan = propagator.create_propagation_plan(&change, &impact);

    let order = propagator.get_execution_order(&plan);
    assert!(!order.is_empty());
}

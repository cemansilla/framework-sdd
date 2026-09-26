use sdd_core::*;
use uuid::Uuid;

#[test]
fn test_project_creation() {
    let project = Project::new("test-project", "1.0.0");
    assert_eq!(project.name, "test-project");
    assert_eq!(project.version, "1.0.0");
}

#[test]
fn test_task_lifecycle() {
    let mut task = Task::new("TASK-001", "Test task", "Description");
    assert_eq!(task.status, TaskStatus::Draft);

    task.status = TaskStatus::Ready;
    assert_eq!(task.status, TaskStatus::Ready);

    task.status = TaskStatus::InProgress;
    assert_eq!(task.status, TaskStatus::InProgress);

    task.status = TaskStatus::Done;
    assert_eq!(task.status, TaskStatus::Done);
}

#[test]
fn test_requirement_creation() {
    let req = Requirement::new(
        "REQ-001",
        "Test requirement",
        "Description",
        RequirementType::Functional,
    );
    assert_eq!(req.req_id, "REQ-001");
    assert_eq!(req.req_type, RequirementType::Functional);
}

#[test]
fn test_architecture_creation() {
    let arch = Architecture::new(
        "arch-001",
        "Test architecture",
        ArchitectureStyle::Hexagonal,
    );
    assert_eq!(arch.name, "arch-001");
    assert_eq!(arch.description, "Test architecture");
    assert_eq!(arch.style, ArchitectureStyle::Hexagonal);
}

#[test]
fn test_change_creation() {
    let change = Change::new(
        "CHG-001",
        "Test change",
        "Description",
        ChangeType::Requirement,
        ChangeOrigin::User {
            reason: "Test".to_string(),
        },
    );
    assert_eq!(change.change_id, "CHG-001");
    assert_eq!(change.change_type, ChangeType::Requirement);
}

#[test]
fn test_agent_creation() {
    let agent = Agent::new(
        "test-agent",
        "Test Agent",
        AgentType::Custom("test".to_string()),
    );
    assert_eq!(agent.name, "test-agent");
}

#[test]
fn test_skill_creation() {
    let skill = Skill::new("test-skill", "Test Skill", "1.0.0", "Test description");
    assert_eq!(skill.skill_id, "test-skill");
    assert_eq!(skill.version, "1.0.0");
}

#[test]
fn test_traceability_graph() {
    let mut graph = TraceabilityGraph::new();

    let node1 = GraphNode::new(Uuid::new_v4(), NodeType::Requirement, "REQ-001");
    let node2 = GraphNode::new(Uuid::new_v4(), NodeType::Task, "TASK-001");

    graph.add_node(node1.clone());
    graph.add_node(node2.clone());

    let edge = GraphEdge::new(node1.id, node2.id, EdgeType::Implements);
    graph.add_edge(edge).unwrap();

    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.edge_count(), 1);
}

#[test]
fn test_context_bundle() {
    let task_id = Uuid::new_v4();
    let budget = TokenBudget::new(4000);
    let mut bundle = ContextBundle::new(task_id, budget);

    let fragment = ContextFragment::new(
        "Test content".to_string(),
        ContextSource::Task,
        "test-id".to_string(),
        ContextPriority::P0,
        "Test reason".to_string(),
    );

    bundle.add_fragment(fragment).unwrap();
    assert_eq!(bundle.fragment_count(), 1);
}

#[test]
fn test_artifact_creation() {
    let artifact = Artifact::new(
        ArtifactCategory::Requirements,
        "requirement",
        "Test artifact",
        "test.md",
    );
    assert_eq!(artifact.category, ArtifactCategory::Requirements);
    assert_eq!(artifact.artifact_type, "requirement");
}

#[test]
fn test_verification_report() {
    let report = VerificationReport::new("TASK-001");
    assert_eq!(report.task_id, "TASK-001");
    assert_eq!(report.overall_status, VerificationStatus::Pending);
}

#[test]
fn test_execution_metrics() {
    let task_id = Uuid::new_v4();
    let agent_id = Uuid::new_v4();
    let execution_id = Uuid::new_v4();

    let metrics = TaskExecutionMetrics::new(task_id, agent_id, execution_id);
    assert_eq!(metrics.task_id, task_id);
    assert_eq!(metrics.agent_id, agent_id);
    assert_eq!(metrics.status, ExecutionMetricsStatus::Running);
}

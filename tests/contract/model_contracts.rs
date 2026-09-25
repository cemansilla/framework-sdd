use sdd_core::*;
use uuid::Uuid;

#[test]
fn test_project_contract() {
    let project = Project::new("test", "1.0.0");

    assert!(!project.id.to_string().is_empty());
    assert!(!project.name.is_empty());
    assert!(!project.version.is_empty());
    assert!(project.created_at.timestamp() > 0);
}

#[test]
fn test_task_contract() {
    let task = Task::new("TASK-001", "Test", "Description");

    assert!(!task.id.to_string().is_empty());
    assert!(!task.task_id.is_empty());
    assert!(!task.title.is_empty());
    assert!(!task.description.is_empty());
    assert!(task.created_at.timestamp() > 0);
}

#[test]
fn test_requirement_contract() {
    let req = Requirement::new(
        "REQ-001",
        "Test",
        "Description",
        RequirementType::Functional,
    );

    assert!(!req.id.to_string().is_empty());
    assert!(!req.req_id.is_empty());
    assert!(!req.title.is_empty());
    assert!(req.created_at.timestamp() > 0);
}

#[test]
fn test_change_contract() {
    let change = Change::new(
        "CHG-001",
        "Test",
        "Description",
        ChangeType::Requirement,
        ChangeOrigin::User {
            reason: "Test".to_string(),
        },
    );

    assert!(!change.id.to_string().is_empty());
    assert!(!change.change_id.is_empty());
    assert!(!change.title.is_empty());
    assert!(change.created_at.timestamp() > 0);
}

#[test]
fn test_context_bundle_contract() {
    let task_id = Uuid::new_v4();
    let budget = TokenBudget::new(4000);
    let bundle = ContextBundle::new(task_id, budget);

    assert!(!bundle.id.to_string().is_empty());
    assert_eq!(bundle.task_id, task_id);
    assert!(bundle.budget.total > 0);
    assert!(bundle.created_at.timestamp() > 0);
}

#[test]
fn test_agent_contract() {
    let agent = Agent::new(
        "test-agent",
        "Test Agent",
        AgentType::Custom("test".to_string()),
    );

    assert!(!agent.id.to_string().is_empty());
    assert!(!agent.name.is_empty());
    assert!(!agent.description.is_empty());
    assert!(agent.created_at.timestamp() > 0);
}

#[test]
fn test_skill_contract() {
    let skill = Skill::new("test-skill", "Test Skill", "1.0.0", "Test description");

    assert!(!skill.id.to_string().is_empty());
    assert!(!skill.skill_id.is_empty());
    assert!(!skill.name.is_empty());
    assert!(!skill.version.is_empty());
    assert!(skill.created_at.timestamp() > 0);
}

#[test]
fn test_artifact_contract() {
    let artifact = Artifact::new(
        ArtifactCategory::Requirements,
        "requirement",
        "Test",
        "test.md",
    );

    assert!(!artifact.id.to_string().is_empty());
    assert!(!artifact.artifact_type.is_empty());
    assert!(!artifact.title.is_empty());
    assert!(!artifact.path.is_empty());
    assert!(artifact.created_at.timestamp() > 0);
}

#[test]
fn test_traceability_graph_contract() {
    let graph = TraceabilityGraph::new();

    assert!(graph.created_at.timestamp() > 0);
    // node_count and edge_count are usize, so they're always >= 0
}

#[test]
fn test_execution_metrics_contract() {
    let task_id = Uuid::new_v4();
    let agent_id = Uuid::new_v4();
    let execution_id = Uuid::new_v4();

    let metrics = TaskExecutionMetrics::new(task_id, agent_id, execution_id);

    assert!(!metrics.id.to_string().is_empty());
    assert_eq!(metrics.task_id, task_id);
    assert_eq!(metrics.agent_id, agent_id);
    assert!(metrics.started_at.timestamp() > 0);
}

#[test]
fn test_token_metrics_contract() {
    let agent_id = Uuid::new_v4();
    let metrics = TokenMetrics::new(agent_id, "gpt-4", 1000, 500, TokenOperation::TaskExecution);

    assert!(!metrics.id.to_string().is_empty());
    assert_eq!(metrics.agent_id, agent_id);
    assert!(!metrics.model.is_empty());
    assert!(metrics.total_tokens > 0);
    assert!(metrics.timestamp.timestamp() > 0);
}

#[test]
fn test_validation_metrics_contract() {
    let task_id = Uuid::new_v4();
    let metrics = ValidationMetrics::new(task_id, ValidationType::CodeQuality, 100);

    assert!(!metrics.id.to_string().is_empty());
    assert_eq!(metrics.task_id, task_id);
    assert!(metrics.duration_ms > 0);
    assert!(metrics.timestamp.timestamp() > 0);
}

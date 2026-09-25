use sdd_core::*;
use sdd_storage::{FilesystemAdapter, StoragePort};
use tempfile::TempDir;
use uuid::Uuid;

#[tokio::test]
async fn test_project_storage_integration() {
    let temp_dir = TempDir::new().unwrap();
    let adapter = FilesystemAdapter::new(temp_dir.path());

    let manifest = adapter.initialize("test-project").await.unwrap();
    assert_eq!(manifest.project_id, "test-project");

    let loaded = adapter.load_manifest().await.unwrap();
    assert_eq!(loaded.project_id, "test-project");
}

#[tokio::test]
async fn test_task_repository_integration() {
    let temp_dir = TempDir::new().unwrap();
    let adapter = FilesystemAdapter::new(temp_dir.path());
    adapter.initialize("test-project").await.unwrap();

    let task = Task::new("TASK-001", "Test task", "Description");
    let task_json = serde_json::to_string(&task).unwrap();

    adapter
        .write("tasks/TASK-001.json", task_json.as_bytes())
        .await
        .unwrap();

    let loaded = adapter.read("tasks/TASK-001.json").await.unwrap();
    let loaded_task: Task = serde_json::from_slice(&loaded).unwrap();

    assert_eq!(loaded_task.task_id, "TASK-001");
}

#[tokio::test]
async fn test_context_bundle_with_resolver() {
    let mut graph = TraceabilityGraph::new();

    let req_node = GraphNode::new(Uuid::new_v4(), NodeType::Requirement, "REQ-001");
    let task_node = GraphNode::new(Uuid::new_v4(), NodeType::Task, "TASK-001");

    graph.add_node(req_node.clone());
    graph.add_node(task_node.clone());

    let edge = GraphEdge::new(req_node.id, task_node.id, EdgeType::Implements);
    graph.add_edge(edge).unwrap();

    let task = Task::new("TASK-001", "Test task", "Description");
    let resolver = ExplicitRelationResolver::new(&graph);

    let mut budget = TokenBudget::new(4000);
    let fragments = resolver.resolve_task_context(&task, &mut budget).unwrap();

    assert!(!fragments.is_empty());
}

#[tokio::test]
async fn test_change_propagation_integration() {
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
        "Test change",
        "Description",
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

#[tokio::test]
async fn test_orchestrator_integration() {
    let agent_registry = AgentRegistry::with_built_in_agents();
    let skill_registry = SkillRegistry::with_built_in_skills();
    let graph = TraceabilityGraph::new();
    let config = OrchestratorConfig::default();

    let mut orchestrator = Orchestrator::new(agent_registry, skill_registry, graph, config);

    let mut task = Task::new("TASK-001", "Test task", "Description");
    task.status = TaskStatus::Ready;

    let request = ExecutionRequest {
        task_id: task.id,
        agent_id: None,
        force: false,
        dry_run: true,
        skip_validation: false,
    };

    let result = orchestrator.execute_task(&mut task, request);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_context_cache_integration() {
    let temp_dir = TempDir::new().unwrap();
    let mut cache = ContextCache::new(temp_dir.path());

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

    cache.put(bundle, vec![]);

    let retrieved = cache.get(&task_id);
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_agent_resolution_integration() {
    let agent_registry = AgentRegistry::with_built_in_agents();
    let skill_registry = SkillRegistry::with_built_in_skills();

    let task = Task::new("TASK-001", "Implement feature", "Write code");

    let resolver = TaskResolver::new(&agent_registry, &skill_registry);
    let request = ResolutionRequest {
        task: task.clone(),
        required_capabilities: vec![],
        preferred_skills: vec![],
        language: Some("rust".to_string()),
        phase: None,
    };

    let result = resolver.resolve(&request);
    assert!(result.selected_agent.is_some());
}

#[tokio::test]
async fn test_validation_integration() {
    let validator = OutputContractValidator::new();
    let task = Task::new("TASK-001", "Test task", "Description");

    let contract = OutputContract::for_code().with_required_tests(false);
    let output = "pub fn test() -> bool { true }";

    let validation = validator.validate(&task, &contract, output);
    assert!(validation.passed);
}

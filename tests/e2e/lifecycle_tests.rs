use sdd_core::*;
use sdd_storage::*;
use tempfile::TempDir;
use uuid::Uuid;

#[tokio::test]
async fn test_full_lifecycle_e2e() {
    let temp_dir = TempDir::new().unwrap();
    let adapter = FilesystemAdapter::new(temp_dir.path());

    // 1. Initialize project
    let manifest = adapter.initialize("e2e-test-project").await.unwrap();
    assert_eq!(manifest.project_id, "e2e-test-project");

    // 2. Create project
    let project = Project::new("E2E Test Project", "1.0.0");
    let project_json = serde_json::to_string(&project).unwrap();
    adapter
        .write("project.json", project_json.as_bytes())
        .await
        .unwrap();

    // 3. Create requirement
    let requirement = Requirement::new(
        "REQ-001",
        "Test requirement",
        "This is a test requirement for E2E testing",
        RequirementType::Functional,
    );
    let req_json = serde_json::to_string(&requirement).unwrap();
    adapter
        .write("requirements/REQ-001.json", req_json.as_bytes())
        .await
        .unwrap();

    // 4. Create architecture
    let architecture = Architecture::new(
        "arch-001",
        "E2E Test Architecture",
        ArchitectureStyle::Hexagonal,
    );
    let arch_json = serde_json::to_string(&architecture).unwrap();
    adapter
        .write("architecture/arch-001.json", arch_json.as_bytes())
        .await
        .unwrap();

    // 5. Create task
    let task = Task::new("TASK-001", "Implement feature", "Implement test feature");
    let task_json = serde_json::to_string(&task).unwrap();
    adapter
        .write("tasks/TASK-001.json", task_json.as_bytes())
        .await
        .unwrap();

    // 6. Create traceability graph
    let mut graph = TraceabilityGraph::new();
    let req_node = GraphNode::new(Uuid::new_v4(), NodeType::Requirement, "REQ-001");
    let task_node = GraphNode::new(Uuid::new_v4(), NodeType::Task, "TASK-001");

    graph.add_node(req_node.clone());
    graph.add_node(task_node.clone());

    let edge = GraphEdge::new(req_node.id, task_node.id, EdgeType::Implements);
    graph.add_edge(edge).unwrap();

    let graph_json = serde_json::to_string(&graph).unwrap();
    adapter
        .write("traceability.json", graph_json.as_bytes())
        .await
        .unwrap();

    // 7. Verify all artifacts exist
    assert!(adapter.exists("project.json").await);
    assert!(adapter.exists("requirements/REQ-001.json").await);
    assert!(adapter.exists("architecture/arch-001.json").await);
    assert!(adapter.exists("tasks/TASK-001.json").await);
    assert!(adapter.exists("traceability.json").await);

    // 8. Load and verify data
    let loaded_project_json = adapter.read("project.json").await.unwrap();
    let loaded_project: Project = serde_json::from_slice(&loaded_project_json).unwrap();
    assert_eq!(loaded_project.name, "E2E Test Project");

    let loaded_req_json = adapter.read("requirements/REQ-001.json").await.unwrap();
    let loaded_req: Requirement = serde_json::from_slice(&loaded_req_json).unwrap();
    assert_eq!(loaded_req.req_id, "REQ-001");

    let loaded_task_json = adapter.read("tasks/TASK-001.json").await.unwrap();
    let loaded_task: Task = serde_json::from_slice(&loaded_task_json).unwrap();
    assert_eq!(loaded_task.task_id, "TASK-001");
}

#[tokio::test]
async fn test_lifecycle_with_orchestrator() {
    let agent_registry = AgentRegistry::with_built_in_agents();
    let skill_registry = SkillRegistry::with_built_in_skills();
    let graph = TraceabilityGraph::new();
    let config = OrchestratorConfig::default();

    let mut orchestrator = Orchestrator::new(agent_registry, skill_registry, graph, config);

    // Create task
    let mut task = Task::new("TASK-001", "Test task", "Description");
    task.status = TaskStatus::Ready;

    // Execute task
    let request = ExecutionRequest {
        task_id: task.id,
        agent_id: None,
        force: false,
        dry_run: true,
        skip_validation: false,
    };

    let result = orchestrator.execute_task(&mut task, request);
    assert!(result.is_ok());

    let execution_result = result.unwrap();
    assert_eq!(execution_result.status, ExecutionStatus::Success);
}

#[tokio::test]
async fn test_lifecycle_with_change_management() {
    let mut graph = TraceabilityGraph::new();

    // Create nodes
    let req_node = GraphNode::new(Uuid::new_v4(), NodeType::Requirement, "REQ-001");
    let task_node = GraphNode::new(Uuid::new_v4(), NodeType::Task, "TASK-001");

    graph.add_node(req_node.clone());
    graph.add_node(task_node.clone());

    let edge = GraphEdge::new(req_node.id, task_node.id, EdgeType::Implements);
    graph.add_edge(edge).unwrap();

    // Create change
    let change = Change::new(
        "CHG-001",
        "Update requirement",
        "Changed requirement description",
        ChangeType::Requirement,
        ChangeOrigin::User {
            reason: "New business need".to_string(),
        },
    )
    .add_affected_artifact(req_node.id.to_string());

    // Analyze impact
    let analyzer = ImpactAnalyzer::new(&graph);
    let impact = analyzer.analyze(&change);

    assert!(!impact.direct_impact.is_empty());

    // Detect changes
    let detector = ChangeDetector::new();
    let old_artifacts = vec![];
    let new_artifact = Artifact::new(
        ArtifactCategory::Requirements,
        "requirement",
        "Updated requirement",
        "REQ-001.md",
    );
    let new_artifacts = vec![new_artifact];

    let detection = detector.detect_changes(&old_artifacts, &new_artifacts);
    assert_eq!(detection.changes.len(), 1);
}

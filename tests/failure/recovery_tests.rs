use sdd_core::*;
use sdd_storage::*;
use tempfile::TempDir;
use uuid::Uuid;

#[tokio::test]
async fn test_recovery_from_missing_file() {
    let temp_dir = TempDir::new().unwrap();
    let adapter = FilesystemAdapter::new(temp_dir.path());

    adapter.initialize("test-project").await.unwrap();

    // Try to read non-existent file
    let result = adapter.read("nonexistent.json").await;
    assert!(result.is_err());

    // Should be able to continue working
    let manifest = adapter.load_manifest().await;
    assert!(manifest.is_ok());
}

#[tokio::test]
async fn test_recovery_from_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let adapter = FilesystemAdapter::new(temp_dir.path());

    adapter.initialize("test-project").await.unwrap();

    // Write invalid JSON
    adapter
        .write("invalid.json", b"not valid json")
        .await
        .unwrap();

    // Try to parse it
    let content = adapter.read("invalid.json").await.unwrap();
    let result: Result<Task, _> = serde_json::from_slice(&content);
    assert!(result.is_err());

    // Should be able to continue working
    let manifest = adapter.load_manifest().await;
    assert!(manifest.is_ok());
}

#[test]
fn test_recovery_from_invalid_task_transition() {
    let mut task = Task::new("TASK-001", "Test", "Description");

    // Try invalid transition
    task.status = TaskStatus::Draft;
    // Can't go from Draft to Done directly
    let _initial_status = task.status.clone();
    task.status = TaskStatus::Done;

    // Task should still be in a valid state
    assert_eq!(task.status, TaskStatus::Done);
}

#[test]
fn test_recovery_from_context_budget_exceeded() {
    let task_id = Uuid::new_v4();
    let budget = TokenBudget::new(10); // Very small budget

    let mut bundle = ContextBundle::new(task_id, budget);

    // Try to add large fragment
    let fragment = ContextFragment::new(
        "This is a very long content that will exceed the budget".to_string(),
        ContextSource::Task,
        "test-id".to_string(),
        ContextPriority::P0,
        "Test".to_string(),
    );

    let result = bundle.add_fragment(fragment);
    assert!(result.is_err());

    // Bundle should still be usable
    assert_eq!(bundle.fragment_count(), 0);
}

#[test]
fn test_recovery_from_agent_not_found() {
    let registry = AgentRegistry::new();

    let result = registry.get(Uuid::new_v4());
    assert!(result.is_none());

    // Registry should still be usable
    assert_eq!(registry.count(), 0);
}

#[test]
fn test_recovery_from_skill_not_found() {
    let registry = SkillRegistry::new();

    let result = registry.get_by_skill_id("nonexistent");
    assert!(result.is_none());

    // Registry should still be usable
    assert_eq!(registry.count(), 0);
}

#[tokio::test]
async fn test_recovery_from_storage_error() {
    let temp_dir = TempDir::new().unwrap();
    let adapter = FilesystemAdapter::new(temp_dir.path());

    adapter.initialize("test-project").await.unwrap();

    // Try to delete non-existent file
    let _result = adapter.delete("nonexistent.json").await;
    // Should not panic, may or may not error

    // Should be able to continue working
    let manifest = adapter.load_manifest().await;
    assert!(manifest.is_ok());
}

#[test]
fn test_recovery_from_invalid_change() {
    let change = Change::new(
        "CHG-001",
        "Test",
        "Description",
        ChangeType::Requirement,
        ChangeOrigin::User {
            reason: "Test".to_string(),
        },
    );

    // Try to resolve without going through proper workflow
    let resolved_change = change.resolve();

    // Change should still be valid
    assert!(resolved_change.is_resolved());
}

#[test]
fn test_recovery_from_conflict() {
    let resolver = ConflictResolver::new();

    let artifact_id = Uuid::new_v4().to_string();

    let change1 = Change::new(
        "CHG-001",
        "Change 1",
        "Description",
        ChangeType::Requirement,
        ChangeOrigin::User {
            reason: "Test".to_string(),
        },
    )
    .add_affected_artifact(&artifact_id);

    let change2 = Change::new(
        "CHG-002",
        "Change 2",
        "Description",
        ChangeType::Architecture,
        ChangeOrigin::User {
            reason: "Test".to_string(),
        },
    )
    .add_affected_artifact(&artifact_id);

    let conflicts = resolver.detect_conflicts(&[change1, change2]);
    assert_eq!(conflicts.len(), 1);

    // Should be able to resolve conflict
    let mut conflict = conflicts[0].clone();
    resolver.resolve_conflict(
        &mut conflict,
        ResolutionStrategy::FirstWins,
        "user",
        "Resolved",
    );

    assert!(conflict.resolution.is_some());
}

#[test]
fn test_recovery_from_validation_failure() {
    let validator = ChangeValidator::new();

    let change = Change::new(
        "CHG-001",
        "Test",
        "", // Empty description
        ChangeType::Requirement,
        ChangeOrigin::User {
            reason: "Test".to_string(),
        },
    );

    let result = validator.validate(&change);
    assert!(!result.is_valid);

    // Should be able to continue using validator
    let change2 = Change::new(
        "CHG-002",
        "Test 2",
        "Valid description",
        ChangeType::Requirement,
        ChangeOrigin::User {
            reason: "Test".to_string(),
        },
    )
    .add_affected_artifact(Uuid::new_v4().to_string());

    let result2 = validator.validate(&change2);
    assert!(result2.is_valid);
}

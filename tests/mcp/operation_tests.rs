use sdd_mcp::*;
use serde_json::json;

#[test]
fn test_mcp_operations_list() {
    let operations = McpOperations::list_operations();

    assert!(!operations.is_empty());
    assert!(operations.iter().any(|op| op.name == "get_project_status"));
    assert!(operations.iter().any(|op| op.name == "get_task"));
    assert!(operations.iter().any(|op| op.name == "get_task_context"));
    assert!(operations.iter().any(|op| op.name == "get_traceability"));
    assert!(operations.iter().any(|op| op.name == "submit_task_output"));
    assert!(operations.iter().any(|op| op.name == "submit_validation"));
    assert!(operations.iter().any(|op| op.name == "register_change"));
    assert!(operations.iter().any(|op| op.name == "get_impact"));
}

#[test]
fn test_mcp_operation_schemas() {
    let operations = McpOperations::list_operations();

    for operation in operations {
        assert!(!operation.name.is_empty());
        assert!(!operation.description.is_empty());
        assert!(operation.input_schema.is_object());
    }
}

#[test]
fn test_mcp_get_project_status_schema() {
    let operations = McpOperations::list_operations();
    let op = operations
        .iter()
        .find(|o| o.name == "get_project_status")
        .unwrap();

    assert!(op.input_schema["type"] == "object");
}

#[test]
fn test_mcp_get_task_schema() {
    let operations = McpOperations::list_operations();
    let op = operations.iter().find(|o| o.name == "get_task").unwrap();

    assert!(op.input_schema["type"] == "object");
    assert!(op.input_schema["properties"]["task_id"].is_object());
    assert!(op.input_schema["required"]
        .as_array()
        .unwrap()
        .contains(&json!("task_id")));
}

#[test]
fn test_mcp_get_task_context_schema() {
    let operations = McpOperations::list_operations();
    let op = operations
        .iter()
        .find(|o| o.name == "get_task_context")
        .unwrap();

    assert!(op.input_schema["type"] == "object");
    assert!(op.input_schema["properties"]["task_id"].is_object());
}

#[test]
fn test_mcp_submit_task_output_schema() {
    let operations = McpOperations::list_operations();
    let op = operations
        .iter()
        .find(|o| o.name == "submit_task_output")
        .unwrap();

    assert!(op.input_schema["type"] == "object");
    assert!(op.input_schema["properties"]["task_id"].is_object());
    assert!(op.input_schema["properties"]["output"].is_object());
}

#[test]
fn test_mcp_register_change_schema() {
    let operations = McpOperations::list_operations();
    let op = operations
        .iter()
        .find(|o| o.name == "register_change")
        .unwrap();

    assert!(op.input_schema["type"] == "object");
    assert!(op.input_schema["properties"]["change_type"].is_object());
    assert!(op.input_schema["properties"]["description"].is_object());
}

#[test]
fn test_mcp_get_impact_schema() {
    let operations = McpOperations::list_operations();
    let op = operations.iter().find(|o| o.name == "get_impact").unwrap();

    assert!(op.input_schema["type"] == "object");
    assert!(op.input_schema["properties"]["change_id"].is_object());
}

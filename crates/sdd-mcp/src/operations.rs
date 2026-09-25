use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpOperations;

impl McpOperations {
    pub fn list_operations() -> Vec<McpOperation> {
        vec![
            McpOperation {
                name: "get_project_status".to_string(),
                description: "Get the current status of the SDD project".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            McpOperation {
                name: "get_task".to_string(),
                description: "Get details of a specific task".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "task_id": {
                            "type": "string",
                            "description": "The ID of the task to retrieve"
                        }
                    },
                    "required": ["task_id"]
                }),
            },
            McpOperation {
                name: "get_task_context".to_string(),
                description: "Get the context bundle for a specific task".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "task_id": {
                            "type": "string",
                            "description": "The ID of the task"
                        }
                    },
                    "required": ["task_id"]
                }),
            },
            McpOperation {
                name: "get_traceability".to_string(),
                description: "Get traceability information for an artifact".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "artifact_id": {
                            "type": "string",
                            "description": "The ID of the artifact"
                        }
                    },
                    "required": ["artifact_id"]
                }),
            },
            McpOperation {
                name: "submit_task_output".to_string(),
                description: "Submit output for a task".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "task_id": {
                            "type": "string",
                            "description": "The ID of the task"
                        },
                        "output": {
                            "type": "string",
                            "description": "The output to submit"
                        }
                    },
                    "required": ["task_id", "output"]
                }),
            },
            McpOperation {
                name: "submit_validation".to_string(),
                description: "Submit validation results for a task".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "task_id": {
                            "type": "string",
                            "description": "The ID of the task"
                        },
                        "validation_result": {
                            "type": "string",
                            "description": "The validation result"
                        }
                    },
                    "required": ["task_id", "validation_result"]
                }),
            },
            McpOperation {
                name: "register_change".to_string(),
                description: "Register a change in the project".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "change_type": {
                            "type": "string",
                            "description": "The type of change"
                        },
                        "description": {
                            "type": "string",
                            "description": "Description of the change"
                        }
                    },
                    "required": ["change_type", "description"]
                }),
            },
            McpOperation {
                name: "get_impact".to_string(),
                description: "Get impact analysis for a change".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "change_id": {
                            "type": "string",
                            "description": "The ID of the change"
                        }
                    },
                    "required": ["change_id"]
                }),
            },
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpOperation {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

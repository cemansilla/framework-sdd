use anyhow::Result;
use serde_json::Value;
use std::io::{self, BufRead, Write};
use uuid::Uuid;

use sdd_core::{
    Change, ChangeOrigin, ChangeRepository, ChangeType, ProjectRepository, Task, TaskRepository,
    TraceabilityGraph, TraceabilityRepository,
};

#[derive(Debug)]
pub struct McpServer<P, T, Tr, C>
where
    P: ProjectRepository,
    T: TaskRepository,
    Tr: TraceabilityRepository,
    C: ChangeRepository,
{
    project_repo: P,
    task_repo: T,
    traceability_repo: Tr,
    change_repo: C,
}

impl<P, T, Tr, C> McpServer<P, T, Tr, C>
where
    P: ProjectRepository,
    T: TaskRepository,
    Tr: TraceabilityRepository,
    C: ChangeRepository,
{
    pub fn new(project_repo: P, task_repo: T, traceability_repo: Tr, change_repo: C) -> Self {
        Self {
            project_repo,
            task_repo,
            traceability_repo,
            change_repo,
        }
    }

    pub async fn run(&self) -> Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let request: Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(e) => {
                    let error_response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "error": {
                            "code": -32700,
                            "message": format!("Parse error: {}", e)
                        },
                        "id": null
                    });
                    writeln!(stdout, "{}", serde_json::to_string(&error_response)?)?;
                    stdout.flush()?;
                    continue;
                }
            };

            let response = self.handle_request(request).await;
            let response_str = serde_json::to_string(&response)?;

            writeln!(stdout, "{}", response_str)?;
            stdout.flush()?;
        }

        Ok(())
    }

    async fn handle_request(&self, request: Value) -> Value {
        let method = request["method"].as_str().unwrap_or("");
        let params = &request["params"];
        let id = &request["id"];

        let result = match method {
            "get_project_status" => self.get_project_status(params).await,
            "get_task" => self.get_task(params).await,
            "get_task_context" => self.get_task_context(params).await,
            "get_traceability" => self.get_traceability(params).await,
            "submit_task_output" => self.submit_task_output(params).await,
            "submit_validation" => self.submit_validation(params).await,
            "register_change" => self.register_change(params).await,
            "get_impact" => self.get_impact(params).await,
            _ => Err(anyhow::anyhow!("Method not found: {}", method)),
        };

        match result {
            Ok(value) => serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": value
            }),
            Err(e) => serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32000,
                    "message": e.to_string()
                }
            }),
        }
    }

    async fn get_project_status(&self, params: &Value) -> Result<Value> {
        let project_id = params["project_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("project_id is required"))?;

        let project_uuid = Uuid::parse_str(project_id)?;
        let project = self.project_repo.get(project_uuid).await?;

        Ok(serde_json::json!({
            "project_id": project.id,
            "name": project.name,
            "description": project.description,
            "version": project.version,
            "created_at": project.created_at.to_rfc3339(),
            "updated_at": project.updated_at.to_rfc3339()
        }))
    }

    async fn get_task(&self, params: &Value) -> Result<Value> {
        let task_id = params["task_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("task_id is required"))?;

        let task = self.task_repo.get_by_id(task_id).await?;

        Ok(serde_json::json!({
            "task_id": task.id,
            "title": task.title,
            "description": task.description,
            "status": format!("{:?}", task.status),
            "assigned_agent": task.assigned_agent,
            "dependencies": task.dependencies,
            "created_at": task.created_at.to_rfc3339(),
            "updated_at": task.updated_at.to_rfc3339()
        }))
    }

    async fn get_task_context(&self, params: &Value) -> Result<Value> {
        let task_id = params["task_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("task_id is required"))?;

        let task = self.task_repo.get_by_id(task_id).await?;
        let graph = self.traceability_repo.get().await?;

        let context = self.build_context_for_task(&task, &graph).await?;

        Ok(serde_json::json!({
            "task_id": task_id,
            "context": context,
            "generated_at": chrono::Utc::now().to_rfc3339()
        }))
    }

    async fn build_context_for_task(
        &self,
        task: &Task,
        graph: &TraceabilityGraph,
    ) -> Result<Value> {
        let neighbors = graph.get_neighbors(task.id);
        let related_artifacts: Vec<String> = neighbors
            .iter()
            .filter_map(|id| graph.get_node(*id))
            .map(|node| node.artifact_id.clone())
            .collect();

        Ok(serde_json::json!({
            "task": {
                "id": task.id,
                "title": task.title,
                "description": task.description,
                "status": format!("{:?}", task.status)
            },
            "related_artifacts": related_artifacts,
            "dependencies": task.dependencies
        }))
    }

    async fn get_traceability(&self, params: &Value) -> Result<Value> {
        let artifact_id = params["artifact_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("artifact_id is required"))?;

        let graph = self.traceability_repo.get().await?;
        let artifact_uuid = Uuid::parse_str(artifact_id)?;
        let neighbors = graph.get_neighbors(artifact_uuid);

        let related: Vec<String> = neighbors
            .iter()
            .filter_map(|id| graph.get_node(*id))
            .map(|node| node.artifact_id.clone())
            .collect();

        Ok(serde_json::json!({
            "artifact_id": artifact_id,
            "related_artifacts": related
        }))
    }

    async fn submit_task_output(&self, params: &Value) -> Result<Value> {
        let task_id = params["task_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("task_id is required"))?;
        let output = params["output"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("output is required"))?;

        let mut task = self.task_repo.get_by_id(task_id).await?;
        task.outputs.artifacts_created.push(output.to_string());
        self.task_repo.save(&task).await?;

        Ok(serde_json::json!({
            "status": "success",
            "task_id": task_id,
            "saved_at": chrono::Utc::now().to_rfc3339()
        }))
    }

    async fn submit_validation(&self, params: &Value) -> Result<Value> {
        let task_id = params["task_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("task_id is required"))?;
        let validation_result = params["validation_result"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("validation_result is required"))?;

        let mut task = self.task_repo.get_by_id(task_id).await?;
        task.outputs.metadata.insert(
            "validation_result".to_string(),
            validation_result.to_string(),
        );
        self.task_repo.save(&task).await?;

        Ok(serde_json::json!({
            "status": "success",
            "task_id": task_id,
            "validated_at": chrono::Utc::now().to_rfc3339()
        }))
    }

    async fn register_change(&self, params: &Value) -> Result<Value> {
        let change_type_str = params["change_type"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("change_type is required"))?;
        let description = params["description"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("description is required"))?;

        let change_type = match change_type_str {
            "requirement" => ChangeType::Requirement,
            "architecture" => ChangeType::Architecture,
            "design" => ChangeType::Design,
            "implementation" => ChangeType::Implementation,
            "bugfix" => ChangeType::Bugfix,
            "refactor" => ChangeType::Refactor,
            "documentation" => ChangeType::Documentation,
            "configuration" => ChangeType::Configuration,
            _ => return Err(anyhow::anyhow!("Invalid change type: {}", change_type_str)),
        };

        let change_id = format!("CHANGE-{}", Uuid::new_v4());
        let change = Change::new(
            change_id.clone(),
            description,
            description,
            change_type,
            ChangeOrigin::User {
                reason: description.to_string(),
            },
        );

        self.change_repo.save(&change).await?;

        Ok(serde_json::json!({
            "status": "success",
            "change_id": change.id,
            "registered_at": chrono::Utc::now().to_rfc3339()
        }))
    }

    async fn get_impact(&self, params: &Value) -> Result<Value> {
        let change_id = params["change_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("change_id is required"))?;

        let change_uuid = Uuid::parse_str(change_id)?;
        let change = self.change_repo.get(change_uuid).await?;
        let graph = self.traceability_repo.get().await?;

        let impact = self.analyze_impact(&change, &graph).await?;

        Ok(serde_json::json!({
            "change_id": change_id,
            "impact": impact
        }))
    }

    async fn analyze_impact(&self, change: &Change, graph: &TraceabilityGraph) -> Result<Value> {
        let neighbors = graph.get_neighbors(change.id);
        let affected_artifacts: Vec<String> = neighbors
            .iter()
            .filter_map(|id| graph.get_node(*id))
            .map(|node| node.artifact_id.clone())
            .collect();

        Ok(serde_json::json!({
            "change_type": format!("{:?}", change.change_type),
            "affected_artifacts": affected_artifacts,
            "estimated_effort": "medium"
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use sdd_core::repository::RepositoryError;
    use sdd_core::{Change, Project, Task, TaskOutputs, TaskScope, TaskStatus, TraceabilityGraph};
    use std::collections::HashMap;

    // Mock repositories for testing
    struct MockProjectRepo;
    struct MockTaskRepo;
    struct MockTraceabilityRepo;
    struct MockChangeRepo;

    #[async_trait]
    impl ProjectRepository for MockProjectRepo {
        async fn get(&self, _id: Uuid) -> Result<Project, RepositoryError> {
            Ok(Project::new("test-project", "1.0.0"))
        }
        async fn save(&self, _project: &Project) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn delete(&self, _id: Uuid) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn list(&self) -> Result<Vec<Project>, RepositoryError> {
            Ok(vec![])
        }
    }

    #[async_trait]
    impl TaskRepository for MockTaskRepo {
        async fn get(&self, _id: Uuid) -> Result<Task, RepositoryError> {
            Ok(Task::new("TASK-001", "Test Task", "Test description"))
        }
        async fn get_by_id(&self, _task_id: &str) -> Result<Task, RepositoryError> {
            Ok(Task::new("TASK-001", "Test Task", "Test description"))
        }
        async fn save(&self, _task: &Task) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn delete(&self, _id: Uuid) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn list_all(&self) -> Result<Vec<Task>, RepositoryError> {
            Ok(vec![])
        }
        async fn list_by_status(&self, _status: &str) -> Result<Vec<Task>, RepositoryError> {
            Ok(vec![])
        }
    }

    #[async_trait]
    impl TraceabilityRepository for MockTraceabilityRepo {
        async fn get(&self) -> Result<TraceabilityGraph, RepositoryError> {
            Ok(TraceabilityGraph::new())
        }
        async fn save(&self, _graph: &TraceabilityGraph) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    #[async_trait]
    impl ChangeRepository for MockChangeRepo {
        async fn get(&self, _id: Uuid) -> Result<Change, RepositoryError> {
            Err(RepositoryError::NotFound {
                entity: "Change".to_string(),
                id: "test".to_string(),
            })
        }
        async fn get_by_id(&self, _change_id: &str) -> Result<Change, RepositoryError> {
            Err(RepositoryError::NotFound {
                entity: "Change".to_string(),
                id: "test".to_string(),
            })
        }
        async fn save(&self, _change: &Change) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn list_all(&self) -> Result<Vec<Change>, RepositoryError> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let server = McpServer::new(
            MockProjectRepo,
            MockTaskRepo,
            MockTraceabilityRepo,
            MockChangeRepo,
        );
        // Server created successfully
        assert!(true);
    }
}

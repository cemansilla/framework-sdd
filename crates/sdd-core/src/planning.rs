use crate::task::{Effort, Task, TaskStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPlan {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub tasks: Vec<Task>,
    pub dependencies: HashMap<Uuid, Vec<Uuid>>,
    pub status: PlanStatus,
    pub estimated_effort: Option<Effort>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlanStatus {
    Draft,
    Ready,
    InProgress,
    Completed,
    Archived,
}

#[derive(Debug, thiserror::Error)]
pub enum PlanningError {
    #[error("circular dependency detected: {0:?}")]
    CircularDependency(Vec<Uuid>),
    #[error("task not found: {0}")]
    TaskNotFound(Uuid),
    #[error("dependency cycle in task graph")]
    DependencyCycle,
}

impl ImplementationPlan {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            description: description.into(),
            tasks: Vec::new(),
            dependencies: HashMap::new(),
            status: PlanStatus::Draft,
            estimated_effort: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_task(&mut self, task: Task) -> Uuid {
        let task_id = task.id;
        self.tasks.push(task);
        self.updated_at = Utc::now();
        task_id
    }

    pub fn add_dependency(&mut self, task_id: Uuid, depends_on: Uuid) -> Result<(), PlanningError> {
        if !self.tasks.iter().any(|t| t.id == task_id) {
            return Err(PlanningError::TaskNotFound(task_id));
        }
        if !self.tasks.iter().any(|t| t.id == depends_on) {
            return Err(PlanningError::TaskNotFound(depends_on));
        }

        self.dependencies
            .entry(task_id)
            .or_default()
            .push(depends_on);

        self.validate_no_cycles()?;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn get_task(&self, task_id: Uuid) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == task_id)
    }

    pub fn get_task_mut(&mut self, task_id: Uuid) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == task_id)
    }

    pub fn get_dependencies(&self, task_id: Uuid) -> Vec<Uuid> {
        self.dependencies.get(&task_id).cloned().unwrap_or_default()
    }

    pub fn get_dependents(&self, task_id: Uuid) -> Vec<Uuid> {
        self.dependencies
            .iter()
            .filter(|(_, deps)| deps.contains(&task_id))
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn ready_tasks(&self) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|task| {
                if task.status != TaskStatus::Draft && task.status != TaskStatus::Ready {
                    return false;
                }

                let deps = self.get_dependencies(task.id);
                deps.iter().all(|dep_id| {
                    self.get_task(*dep_id)
                        .map(|t| t.status == TaskStatus::Done)
                        .unwrap_or(false)
                })
            })
            .collect()
    }

    pub fn blocked_tasks(&self) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|task| {
                let deps = self.get_dependencies(task.id);
                !deps.is_empty()
                    && deps.iter().any(|dep_id| {
                        self.get_task(*dep_id)
                            .map(|t| t.status != TaskStatus::Done)
                            .unwrap_or(true)
                    })
            })
            .collect()
    }

    pub fn topological_sort(&self) -> Result<Vec<Uuid>, PlanningError> {
        let mut visited = HashSet::new();
        let mut temp_mark = HashSet::new();
        let mut result = Vec::new();

        for task in &self.tasks {
            if !visited.contains(&task.id) {
                self.visit(task.id, &mut visited, &mut temp_mark, &mut result)?;
            }
        }

        Ok(result)
    }

    fn visit(
        &self,
        task_id: Uuid,
        visited: &mut HashSet<Uuid>,
        temp_mark: &mut HashSet<Uuid>,
        result: &mut Vec<Uuid>,
    ) -> Result<(), PlanningError> {
        if temp_mark.contains(&task_id) {
            return Err(PlanningError::DependencyCycle);
        }
        if visited.contains(&task_id) {
            return Ok(());
        }

        temp_mark.insert(task_id);

        for dep in self.get_dependencies(task_id) {
            self.visit(dep, visited, temp_mark, result)?;
        }

        temp_mark.remove(&task_id);
        visited.insert(task_id);
        result.push(task_id);

        Ok(())
    }

    fn validate_no_cycles(&self) -> Result<(), PlanningError> {
        self.topological_sort()?;
        Ok(())
    }

    pub fn completion_percentage(&self) -> f32 {
        if self.tasks.is_empty() {
            return 0.0;
        }
        let completed = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Done)
            .count();
        (completed as f32 / self.tasks.len() as f32) * 100.0
    }

    pub fn mark_ready(&mut self) {
        self.status = PlanStatus::Ready;
        self.updated_at = Utc::now();
    }

    pub fn start(&mut self) {
        self.status = PlanStatus::InProgress;
        self.updated_at = Utc::now();
    }

    pub fn complete(&mut self) {
        self.status = PlanStatus::Completed;
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDecomposition {
    pub parent_task_id: Uuid,
    pub subtasks: Vec<Task>,
    pub decomposition_strategy: DecompositionStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecompositionStrategy {
    ByModule,
    ByFunctionality,
    ByLayer,
    Custom(String),
}

impl TaskDecomposition {
    pub fn new(parent_task_id: Uuid, strategy: DecompositionStrategy) -> Self {
        Self {
            parent_task_id,
            subtasks: Vec::new(),
            decomposition_strategy: strategy,
        }
    }

    pub fn add_subtask(&mut self, task: Task) {
        self.subtasks.push(task);
    }

    pub fn subtask_count(&self) -> usize {
        self.subtasks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_implementation_plan_creation() {
        let plan = ImplementationPlan::new("Sprint 1", "First sprint tasks");
        assert_eq!(plan.status, PlanStatus::Draft);
        assert!(plan.tasks.is_empty());
    }

    #[test]
    fn test_add_task() {
        let mut plan = ImplementationPlan::new("Sprint 1", "Tasks");
        let task = Task::new("TASK-001", "Implement feature", "Description");
        let task_id = plan.add_task(task);

        assert_eq!(plan.tasks.len(), 1);
        assert!(plan.get_task(task_id).is_some());
    }

    #[test]
    fn test_add_dependency() {
        let mut plan = ImplementationPlan::new("Sprint 1", "Tasks");
        let task1 = Task::new("TASK-001", "Task 1", "Description 1");
        let task2 = Task::new("TASK-002", "Task 2", "Description 2");

        let id1 = plan.add_task(task1);
        let id2 = plan.add_task(task2);

        plan.add_dependency(id2, id1).unwrap();

        assert_eq!(plan.get_dependencies(id2), vec![id1]);
        assert_eq!(plan.get_dependents(id1), vec![id2]);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut plan = ImplementationPlan::new("Sprint 1", "Tasks");
        let task1 = Task::new("TASK-001", "Task 1", "Description 1");
        let task2 = Task::new("TASK-002", "Task 2", "Description 2");

        let id1 = plan.add_task(task1);
        let id2 = plan.add_task(task2);

        plan.add_dependency(id2, id1).unwrap();
        let result = plan.add_dependency(id1, id2);

        assert!(result.is_err());
    }

    #[test]
    fn test_topological_sort() {
        let mut plan = ImplementationPlan::new("Sprint 1", "Tasks");
        let task1 = Task::new("TASK-001", "Task 1", "Description 1");
        let task2 = Task::new("TASK-002", "Task 2", "Description 2");
        let task3 = Task::new("TASK-003", "Task 3", "Description 3");

        let id1 = plan.add_task(task1);
        let id2 = plan.add_task(task2);
        let id3 = plan.add_task(task3);

        plan.add_dependency(id2, id1).unwrap();
        plan.add_dependency(id3, id2).unwrap();

        let sorted = plan.topological_sort().unwrap();
        assert_eq!(sorted, vec![id1, id2, id3]);
    }

    #[test]
    fn test_ready_tasks() {
        let mut plan = ImplementationPlan::new("Sprint 1", "Tasks");
        let mut task1 = Task::new("TASK-001", "Task 1", "Description 1");
        task1.status = TaskStatus::Ready;
        let mut task2 = Task::new("TASK-002", "Task 2", "Description 2");
        task2.status = TaskStatus::Ready;

        let id1 = plan.add_task(task1);
        let id2 = plan.add_task(task2);

        plan.add_dependency(id2, id1).unwrap();

        let ready = plan.ready_tasks();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].task_id, "TASK-001");
    }

    #[test]
    fn test_completion_percentage() {
        let mut plan = ImplementationPlan::new("Sprint 1", "Tasks");
        let mut task1 = Task::new("TASK-001", "Task 1", "Description 1");
        task1.status = TaskStatus::Done;
        let mut task2 = Task::new("TASK-002", "Task 2", "Description 2");
        task2.status = TaskStatus::InProgress;

        plan.add_task(task1);
        plan.add_task(task2);

        assert_eq!(plan.completion_percentage(), 50.0);
    }

    #[test]
    fn test_task_decomposition() {
        let parent_id = Uuid::new_v4();
        let mut decomposition = TaskDecomposition::new(parent_id, DecompositionStrategy::ByModule);

        let subtask1 = Task::new("TASK-001-1", "Subtask 1", "Description 1");
        let subtask2 = Task::new("TASK-001-2", "Subtask 2", "Description 2");

        decomposition.add_subtask(subtask1);
        decomposition.add_subtask(subtask2);

        assert_eq!(decomposition.subtask_count(), 2);
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Draft,
    Ready,
    InProgress,
    Blocked,
    Review,
    Qa,
    Done,
    Failed,
    Rejected,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub task_id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub scope: TaskScope,
    pub acceptance_criteria: Vec<String>,
    pub dependencies: Vec<String>,
    pub assigned_agent: Option<String>,
    pub required_skills: Vec<String>,
    pub outputs: TaskOutputs,
    pub testing_strategy: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskScope {
    pub files_to_modify: Vec<String>,
    pub modules_affected: Vec<String>,
    pub estimated_effort: Option<Effort>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Effort {
    Trivial,
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskOutputs {
    pub artifacts_created: Vec<String>,
    pub artifacts_modified: Vec<String>,
    pub tests_created: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl Task {
    pub fn new(
        task_id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            task_id: task_id.into(),
            title: title.into(),
            description: description.into(),
            status: TaskStatus::Draft,
            scope: TaskScope {
                files_to_modify: Vec::new(),
                modules_affected: Vec::new(),
                estimated_effort: None,
            },
            acceptance_criteria: Vec::new(),
            dependencies: Vec::new(),
            assigned_agent: None,
            required_skills: Vec::new(),
            outputs: TaskOutputs::default(),
            testing_strategy: None,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
        }
    }

    pub fn with_scope(mut self, scope: TaskScope) -> Self {
        self.scope = scope;
        self.updated_at = Utc::now();
        self
    }

    pub fn add_acceptance_criterion(mut self, criterion: impl Into<String>) -> Self {
        self.acceptance_criteria.push(criterion.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn add_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependencies.push(dependency.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn assign_agent(mut self, agent: impl Into<String>) -> Self {
        self.assigned_agent = Some(agent.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn add_required_skill(mut self, skill: impl Into<String>) -> Self {
        self.required_skills.push(skill.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn with_testing_strategy(mut self, strategy: impl Into<String>) -> Self {
        self.testing_strategy = Some(strategy.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn transition_to(&mut self, status: TaskStatus) -> Result<(), TaskTransitionError> {
        if !self.is_valid_transition(&status) {
            return Err(TaskTransitionError::InvalidTransition {
                from: self.status.clone(),
                to: status,
            });
        }

        let now = Utc::now();
        self.status = status.clone();
        self.updated_at = now;

        match status {
            TaskStatus::InProgress => self.started_at = Some(now),
            TaskStatus::Done | TaskStatus::Failed | TaskStatus::Cancelled => {
                self.completed_at = Some(now)
            }
            _ => {}
        }

        Ok(())
    }

    fn is_valid_transition(&self, target: &TaskStatus) -> bool {
        matches!(
            (&self.status, target),
            (TaskStatus::Draft, TaskStatus::Ready)
                | (TaskStatus::Draft, TaskStatus::Cancelled)
                | (TaskStatus::Ready, TaskStatus::InProgress)
                | (TaskStatus::Ready, TaskStatus::Blocked)
                | (TaskStatus::Ready, TaskStatus::Cancelled)
                | (TaskStatus::InProgress, TaskStatus::Review)
                | (TaskStatus::InProgress, TaskStatus::Blocked)
                | (TaskStatus::InProgress, TaskStatus::Failed)
                | (TaskStatus::InProgress, TaskStatus::Cancelled)
                | (TaskStatus::Blocked, TaskStatus::Ready)
                | (TaskStatus::Blocked, TaskStatus::Cancelled)
                | (TaskStatus::Review, TaskStatus::Qa)
                | (TaskStatus::Review, TaskStatus::InProgress)
                | (TaskStatus::Review, TaskStatus::Rejected)
                | (TaskStatus::Qa, TaskStatus::Done)
                | (TaskStatus::Qa, TaskStatus::InProgress)
                | (TaskStatus::Qa, TaskStatus::Failed)
                | (TaskStatus::Rejected, TaskStatus::InProgress)
        )
    }

    pub fn is_ready(&self) -> bool {
        !self.title.is_empty()
            && !self.description.is_empty()
            && !self.acceptance_criteria.is_empty()
            && self.dependencies.is_empty()
            && self.assigned_agent.is_some()
            && self.testing_strategy.is_some()
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.status, TaskStatus::Done)
    }

    pub fn is_blocked(&self) -> bool {
        matches!(self.status, TaskStatus::Blocked)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TaskTransitionError {
    #[error("Invalid transition from {from:?} to {to:?}")]
    InvalidTransition { from: TaskStatus, to: TaskStatus },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("TASK-001", "Implement feature", "Implement the new feature");
        assert_eq!(task.status, TaskStatus::Draft);
        assert_eq!(task.task_id, "TASK-001");
    }

    #[test]
    fn test_task_with_criteria() {
        let task = Task::new("TASK-001", "Implement feature", "Implement the new feature")
            .add_acceptance_criterion("Feature works")
            .add_acceptance_criterion("Tests pass");
        assert_eq!(task.acceptance_criteria.len(), 2);
    }

    #[test]
    fn test_task_transition() {
        let mut task = Task::new("TASK-001", "Implement feature", "Implement the new feature");
        assert!(task.transition_to(TaskStatus::Ready).is_ok());
        assert_eq!(task.status, TaskStatus::Ready);
    }

    #[test]
    fn test_task_invalid_transition() {
        let mut task = Task::new("TASK-001", "Implement feature", "Implement the new feature");
        assert!(task.transition_to(TaskStatus::Done).is_err());
    }

    #[test]
    fn test_task_is_ready() {
        let task = Task::new("TASK-001", "Implement feature", "Implement the new feature")
            .add_acceptance_criterion("Feature works")
            .assign_agent("coder")
            .with_testing_strategy("Unit tests");
        assert!(task.is_ready());
    }

    #[test]
    fn test_task_not_ready_without_agent() {
        let task = Task::new("TASK-001", "Implement feature", "Implement the new feature")
            .add_acceptance_criterion("Feature works")
            .with_testing_strategy("Unit tests");
        assert!(!task.is_ready());
    }

    #[test]
    fn test_task_start_tracking() {
        let mut task = Task::new("TASK-001", "Implement feature", "Implement the new feature");
        task.transition_to(TaskStatus::Ready).unwrap();
        task.transition_to(TaskStatus::InProgress).unwrap();
        assert!(task.started_at.is_some());
    }
}

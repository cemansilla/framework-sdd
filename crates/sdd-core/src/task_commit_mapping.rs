use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCommitMapping {
    pub id: Uuid,
    pub task_id: String,
    pub commits: Vec<CommitReference>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitReference {
    pub hash: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub branch: String,
    pub author: String,
}

#[derive(Debug, Clone)]
pub struct TaskCommitMapper {
    mappings: HashMap<String, TaskCommitMapping>,
}

impl TaskCommitMapper {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    pub fn add_commit(
        &mut self,
        task_id: &str,
        commit: CommitReference,
    ) -> Result<(), MappingError> {
        let mapping = self.mappings.entry(task_id.to_string()).or_insert_with(|| {
            let now = Utc::now();
            TaskCommitMapping {
                id: Uuid::new_v4(),
                task_id: task_id.to_string(),
                commits: Vec::new(),
                created_at: now,
                updated_at: now,
            }
        });

        if mapping.commits.iter().any(|c| c.hash == commit.hash) {
            return Err(MappingError::DuplicateCommit);
        }

        mapping.commits.push(commit);
        mapping.updated_at = Utc::now();

        Ok(())
    }

    pub fn get_mapping(&self, task_id: &str) -> Option<&TaskCommitMapping> {
        self.mappings.get(task_id)
    }

    pub fn get_commits_for_task(&self, task_id: &str) -> Vec<&CommitReference> {
        self.mappings
            .get(task_id)
            .map(|m| m.commits.iter().collect())
            .unwrap_or_default()
    }

    pub fn get_tasks_for_commit(&self, commit_hash: &str) -> Vec<&str> {
        self.mappings
            .iter()
            .filter(|(_, mapping)| mapping.commits.iter().any(|c| c.hash == commit_hash))
            .map(|(task_id, _)| task_id.as_str())
            .collect()
    }

    pub fn remove_commit(&mut self, task_id: &str, commit_hash: &str) -> bool {
        if let Some(mapping) = self.mappings.get_mut(task_id) {
            let initial_len = mapping.commits.len();
            mapping.commits.retain(|c| c.hash != commit_hash);
            mapping.updated_at = Utc::now();
            mapping.commits.len() < initial_len
        } else {
            false
        }
    }

    pub fn get_all_mappings(&self) -> Vec<&TaskCommitMapping> {
        self.mappings.values().collect()
    }

    pub fn get_mapping_count(&self) -> usize {
        self.mappings.len()
    }

    pub fn get_total_commit_count(&self) -> usize {
        self.mappings.values().map(|m| m.commits.len()).sum()
    }
}

impl Default for TaskCommitMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl CommitReference {
    pub fn new(
        hash: impl Into<String>,
        message: impl Into<String>,
        branch: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        Self {
            hash: hash.into(),
            message: message.into(),
            timestamp: Utc::now(),
            branch: branch.into(),
            author: author.into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MappingError {
    #[error("commit already mapped to this task")]
    DuplicateCommit,
    #[error("task not found: {0}")]
    TaskNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapper_creation() {
        let mapper = TaskCommitMapper::new();
        assert_eq!(mapper.get_mapping_count(), 0);
    }

    #[test]
    fn test_add_commit() {
        let mut mapper = TaskCommitMapper::new();
        let commit = CommitReference::new(
            "abc123",
            "feat: add feature",
            "feature/TASK-FW-001",
            "John Doe",
        );

        let result = mapper.add_commit("TASK-FW-001", commit);
        assert!(result.is_ok());
        assert_eq!(mapper.get_mapping_count(), 1);
    }

    #[test]
    fn test_add_duplicate_commit() {
        let mut mapper = TaskCommitMapper::new();
        let commit1 = CommitReference::new(
            "abc123",
            "feat: add feature",
            "feature/TASK-FW-001",
            "John Doe",
        );
        let commit2 = CommitReference::new(
            "abc123",
            "feat: add feature",
            "feature/TASK-FW-001",
            "John Doe",
        );

        mapper.add_commit("TASK-FW-001", commit1).unwrap();
        let result = mapper.add_commit("TASK-FW-001", commit2);
        assert!(matches!(result, Err(MappingError::DuplicateCommit)));
    }

    #[test]
    fn test_get_commits_for_task() {
        let mut mapper = TaskCommitMapper::new();
        let commit1 = CommitReference::new(
            "abc123",
            "feat: add feature",
            "feature/TASK-FW-001",
            "John Doe",
        );
        let commit2 =
            CommitReference::new("def456", "fix: fix bug", "feature/TASK-FW-001", "John Doe");

        mapper.add_commit("TASK-FW-001", commit1).unwrap();
        mapper.add_commit("TASK-FW-001", commit2).unwrap();

        let commits = mapper.get_commits_for_task("TASK-FW-001");
        assert_eq!(commits.len(), 2);
    }

    #[test]
    fn test_get_tasks_for_commit() {
        let mut mapper = TaskCommitMapper::new();
        let commit = CommitReference::new(
            "abc123",
            "feat: add feature",
            "feature/TASK-FW-001",
            "John Doe",
        );

        mapper.add_commit("TASK-FW-001", commit).unwrap();

        let tasks = mapper.get_tasks_for_commit("abc123");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0], "TASK-FW-001");
    }

    #[test]
    fn test_remove_commit() {
        let mut mapper = TaskCommitMapper::new();
        let commit = CommitReference::new(
            "abc123",
            "feat: add feature",
            "feature/TASK-FW-001",
            "John Doe",
        );

        mapper.add_commit("TASK-FW-001", commit).unwrap();
        assert_eq!(mapper.get_total_commit_count(), 1);

        let removed = mapper.remove_commit("TASK-FW-001", "abc123");
        assert!(removed);
        assert_eq!(mapper.get_total_commit_count(), 0);
    }

    #[test]
    fn test_get_all_mappings() {
        let mut mapper = TaskCommitMapper::new();
        let commit1 = CommitReference::new(
            "abc123",
            "feat: add feature",
            "feature/TASK-FW-001",
            "John Doe",
        );
        let commit2 =
            CommitReference::new("def456", "fix: fix bug", "feature/TASK-FW-002", "Jane Doe");

        mapper.add_commit("TASK-FW-001", commit1).unwrap();
        mapper.add_commit("TASK-FW-002", commit2).unwrap();

        let mappings = mapper.get_all_mappings();
        assert_eq!(mappings.len(), 2);
    }
}

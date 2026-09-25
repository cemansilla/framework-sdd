use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitMetadata {
    pub id: Uuid,
    pub hash: String,
    pub message: CommitMessage,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub branch: String,
    pub task_ids: Vec<String>,
    pub affected_files: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitMessage {
    pub commit_type: CommitType,
    pub scope: Option<String>,
    pub task_id: Option<String>,
    pub description: String,
    pub body: Option<String>,
    pub footer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommitType {
    Feat,
    Fix,
    Docs,
    Style,
    Refactor,
    Test,
    Chore,
    Perf,
    Build,
    Ci,
}

impl CommitMessage {
    pub fn new(
        commit_type: CommitType,
        description: impl Into<String>,
    ) -> Self {
        Self {
            commit_type,
            scope: None,
            task_id: None,
            description: description.into(),
            body: None,
            footer: None,
        }
    }

    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    pub fn with_task_id(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = Some(task_id.into());
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn with_footer(mut self, footer: impl Into<String>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    pub fn format(&self) -> String {
        let mut message = String::new();

        let type_str = match self.commit_type {
            CommitType::Feat => "feat",
            CommitType::Fix => "fix",
            CommitType::Docs => "docs",
            CommitType::Style => "style",
            CommitType::Refactor => "refactor",
            CommitType::Test => "test",
            CommitType::Chore => "chore",
            CommitType::Perf => "perf",
            CommitType::Build => "build",
            CommitType::Ci => "ci",
        };

        message.push_str(type_str);

        if let Some(scope) = &self.scope {
            message.push_str(&format!("({})", scope));
        }

        message.push_str(": ");

        if let Some(task_id) = &self.task_id {
            message.push_str(&format!("[{}] ", task_id));
        }

        message.push_str(&self.description);

        if let Some(body) = &self.body {
            message.push_str("\n\n");
            message.push_str(body);
        }

        if let Some(footer) = &self.footer {
            message.push_str("\n\n");
            message.push_str(footer);
        }

        message
    }

    pub fn parse(message: &str) -> Result<Self, CommitError> {
        let lines: Vec<&str> = message.split('\n').collect();
        if lines.is_empty() {
            return Err(CommitError::InvalidFormat);
        }

        let first_line = lines[0];
        let parts: Vec<&str> = first_line.splitn(2, ": ").collect();
        if parts.len() != 2 {
            return Err(CommitError::InvalidFormat);
        }

        let type_and_scope = parts[0];
        let description_part = parts[1];

        let (commit_type, scope) = if type_and_scope.contains('(') {
            let type_parts: Vec<&str> = type_and_scope.splitn(2, '(').collect();
            let scope_part = type_parts[1].trim_end_matches(')');
            (
                Self::parse_commit_type(type_parts[0])?,
                Some(scope_part.to_string()),
            )
        } else {
            (Self::parse_commit_type(type_and_scope)?, None)
        };

        let (task_id, description) = if description_part.starts_with('[') {
            let bracket_end = description_part.find(']').ok_or(CommitError::InvalidFormat)?;
            let task_id = &description_part[1..bracket_end];
            let desc = description_part[bracket_end + 1..].trim();
            (Some(task_id.to_string()), desc.to_string())
        } else {
            (None, description_part.to_string())
        };

        let body = if lines.len() > 2 {
            Some(lines[2..].join("\n").trim().to_string())
        } else {
            None
        };

        Ok(Self {
            commit_type,
            scope,
            task_id,
            description,
            body,
            footer: None,
        })
    }

    fn parse_commit_type(type_str: &str) -> Result<CommitType, CommitError> {
        match type_str {
            "feat" => Ok(CommitType::Feat),
            "fix" => Ok(CommitType::Fix),
            "docs" => Ok(CommitType::Docs),
            "style" => Ok(CommitType::Style),
            "refactor" => Ok(CommitType::Refactor),
            "test" => Ok(CommitType::Test),
            "chore" => Ok(CommitType::Chore),
            "perf" => Ok(CommitType::Perf),
            "build" => Ok(CommitType::Build),
            "ci" => Ok(CommitType::Ci),
            _ => Err(CommitError::InvalidType),
        }
    }
}

impl CommitMetadata {
    pub fn new(
        hash: impl Into<String>,
        message: CommitMessage,
        author: impl Into<String>,
        branch: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            hash: hash.into(),
            message,
            author: author.into(),
            timestamp: Utc::now(),
            branch: branch.into(),
            task_ids: Vec::new(),
            affected_files: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_task_id(mut self, task_id: impl Into<String>) -> Self {
        self.task_ids.push(task_id.into());
        self
    }

    pub fn with_affected_file(mut self, file: impl Into<String>) -> Self {
        self.affected_files.push(file.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CommitError {
    #[error("invalid commit message format")]
    InvalidFormat,
    #[error("invalid commit type")]
    InvalidType,
    #[error("missing description")]
    MissingDescription,
}

pub struct CommitMetadataExtractor;

impl CommitMetadataExtractor {
    pub fn new() -> Self {
        Self
    }

    pub fn extract_task_ids(&self, message: &CommitMessage) -> Vec<String> {
        let mut task_ids = Vec::new();

        if let Some(task_id) = &message.task_id {
            task_ids.push(task_id.clone());
        }

        if let Some(body) = &message.body {
            for line in body.lines() {
                if line.starts_with("TASK-") {
                    task_ids.push(line.to_string());
                }
            }
        }

        task_ids
    }

    pub fn extract_affected_modules(&self, files: &[String]) -> Vec<String> {
        let mut modules = std::collections::HashSet::new();

        for file in files {
            if let Some(module) = self.extract_module_from_path(file) {
                modules.insert(module);
            }
        }

        modules.into_iter().collect()
    }

    fn extract_module_from_path(&self, path: &str) -> Option<String> {
        if path.starts_with("crates/") {
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                return Some(parts[1].to_string());
            }
        }
        None
    }
}

impl Default for CommitMetadataExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_message_creation() {
        let message = CommitMessage::new(CommitType::Feat, "add new feature");
        assert_eq!(message.commit_type, CommitType::Feat);
        assert_eq!(message.description, "add new feature");
    }

    #[test]
    fn test_commit_message_with_scope() {
        let message = CommitMessage::new(CommitType::Fix, "fix bug")
            .with_scope("core");
        assert_eq!(message.scope, Some("core".to_string()));
    }

    #[test]
    fn test_commit_message_with_task_id() {
        let message = CommitMessage::new(CommitType::Feat, "add feature")
            .with_task_id("TASK-FW-001");
        assert_eq!(message.task_id, Some("TASK-FW-001".to_string()));
    }

    #[test]
    fn test_commit_message_format() {
        let message = CommitMessage::new(CommitType::Feat, "add feature")
            .with_scope("core")
            .with_task_id("TASK-FW-001");
        
        let formatted = message.format();
        assert_eq!(formatted, "feat(core): [TASK-FW-001] add feature");
    }

    #[test]
    fn test_commit_message_parse() {
        let message_str = "feat(core): [TASK-FW-001] add new feature";
        let message = CommitMessage::parse(message_str).unwrap();
        
        assert_eq!(message.commit_type, CommitType::Feat);
        assert_eq!(message.scope, Some("core".to_string()));
        assert_eq!(message.task_id, Some("TASK-FW-001".to_string()));
        assert_eq!(message.description, "add new feature");
    }

    #[test]
    fn test_commit_message_parse_without_scope() {
        let message_str = "fix: [TASK-FW-002] fix bug";
        let message = CommitMessage::parse(message_str).unwrap();
        
        assert_eq!(message.commit_type, CommitType::Fix);
        assert_eq!(message.scope, None);
        assert_eq!(message.task_id, Some("TASK-FW-002".to_string()));
    }

    #[test]
    fn test_commit_metadata_creation() {
        let message = CommitMessage::new(CommitType::Feat, "add feature");
        let metadata = CommitMetadata::new(
            "abc123",
            message,
            "John Doe",
            "feature/TASK-FW-001_add-feature",
        );
        
        assert_eq!(metadata.hash, "abc123");
        assert_eq!(metadata.author, "John Doe");
    }

    #[test]
    fn test_commit_metadata_with_task_id() {
        let message = CommitMessage::new(CommitType::Feat, "add feature");
        let metadata = CommitMetadata::new(
            "abc123",
            message,
            "John Doe",
            "feature/TASK-FW-001_add-feature",
        )
        .with_task_id("TASK-FW-001");
        
        assert_eq!(metadata.task_ids.len(), 1);
        assert_eq!(metadata.task_ids[0], "TASK-FW-001");
    }

    #[test]
    fn test_commit_metadata_extractor_task_ids() {
        let extractor = CommitMetadataExtractor::new();
        let message = CommitMessage::new(CommitType::Feat, "add feature")
            .with_task_id("TASK-FW-001");
        
        let task_ids = extractor.extract_task_ids(&message);
        assert_eq!(task_ids.len(), 1);
        assert_eq!(task_ids[0], "TASK-FW-001");
    }

    #[test]
    fn test_commit_metadata_extractor_modules() {
        let extractor = CommitMetadataExtractor::new();
        let files = vec![
            "crates/sdd-core/src/lib.rs".to_string(),
            "crates/sdd-storage/src/lib.rs".to_string(),
            "crates/sdd-core/src/project.rs".to_string(),
        ];
        
        let modules = extractor.extract_affected_modules(&files);
        assert_eq!(modules.len(), 2);
        assert!(modules.contains(&"sdd-core".to_string()));
        assert!(modules.contains(&"sdd-storage".to_string()));
    }
}

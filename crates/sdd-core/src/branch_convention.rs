use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub id: Uuid,
    pub name: String,
    pub branch_type: BranchType,
    pub task_id: Option<String>,
    pub base_branch: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BranchType {
    Feature,
    Fix,
    Chore,
    Docs,
    Refactor,
    Test,
    Ci,
    Release,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchConvention {
    pub patterns: Vec<BranchPattern>,
    pub default_base: String,
    pub require_task_id: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchPattern {
    pub branch_type: BranchType,
    pub pattern: String,
    pub description: String,
}

impl BranchConvention {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                BranchPattern {
                    branch_type: BranchType::Feature,
                    pattern: "feature/TASK-FW-{id}_{description}".to_string(),
                    description: "New features".to_string(),
                },
                BranchPattern {
                    branch_type: BranchType::Fix,
                    pattern: "fix/TASK-FW-{id}_{description}".to_string(),
                    description: "Bug fixes".to_string(),
                },
                BranchPattern {
                    branch_type: BranchType::Chore,
                    pattern: "chore/{description}".to_string(),
                    description: "Maintenance tasks".to_string(),
                },
                BranchPattern {
                    branch_type: BranchType::Docs,
                    pattern: "docs/TASK-FW-{id}_{description}".to_string(),
                    description: "Documentation".to_string(),
                },
                BranchPattern {
                    branch_type: BranchType::Refactor,
                    pattern: "refactor/TASK-FW-{id}_{description}".to_string(),
                    description: "Code refactoring".to_string(),
                },
                BranchPattern {
                    branch_type: BranchType::Test,
                    pattern: "test/TASK-FW-{id}_{description}".to_string(),
                    description: "Test additions".to_string(),
                },
                BranchPattern {
                    branch_type: BranchType::Ci,
                    pattern: "ci/{description}".to_string(),
                    description: "CI/CD changes".to_string(),
                },
                BranchPattern {
                    branch_type: BranchType::Release,
                    pattern: "release/v{version}".to_string(),
                    description: "Release preparation".to_string(),
                },
            ],
            default_base: "develop".to_string(),
            require_task_id: true,
        }
    }

    pub fn validate_branch_name(&self, name: &str) -> Result<BranchInfo, BranchError> {
        for pattern in &self.patterns {
            if self.matches_pattern(name, &pattern.pattern) {
                let task_id = self.extract_task_id(name);
                
                if self.require_task_id
                    && task_id.is_none()
                    && pattern.branch_type != BranchType::Chore
                    && pattern.branch_type != BranchType::Ci
                {
                    return Err(BranchError::MissingTaskId);
                }

                return Ok(BranchInfo {
                    id: Uuid::new_v4(),
                    name: name.to_string(),
                    branch_type: pattern.branch_type.clone(),
                    task_id,
                    base_branch: self.default_base.clone(),
                    created_at: Utc::now(),
                    created_by: "system".to_string(),
                    metadata: HashMap::new(),
                });
            }
        }

        Err(BranchError::InvalidFormat)
    }

    fn matches_pattern(&self, name: &str, pattern: &str) -> bool {
        let regex_pattern = pattern
            .replace("{id}", "[A-Z0-9-]+")
            .replace("{description}", "[a-z0-9-]+")
            .replace("{version}", "[0-9.]+");

        let parts: Vec<&str> = regex_pattern.split('/').collect();
        let name_parts: Vec<&str> = name.split('/').collect();

        if parts.len() != name_parts.len() {
            return false;
        }

        for (part, name_part) in parts.iter().zip(name_parts.iter()) {
            if part.contains('[') {
                continue;
            }
            if part != name_part {
                return false;
            }
        }

        true
    }

    fn extract_task_id(&self, name: &str) -> Option<String> {
        let parts: Vec<&str> = name.split('/').collect();
        if parts.len() >= 2 {
            let second_part = parts[1];
            let task_parts: Vec<&str> = second_part.split('_').collect();
            if !task_parts.is_empty() && task_parts[0].starts_with("TASK-") {
                return Some(task_parts[0].to_string());
            }
        }
        None
    }

    pub fn generate_branch_name(
        &self,
        branch_type: BranchType,
        task_id: Option<&str>,
        description: &str,
    ) -> String {
        let desc = description.to_lowercase().replace(' ', "-");
        
        match branch_type {
            BranchType::Feature => {
                if let Some(id) = task_id {
                    format!("feature/{}_{}", id, desc)
                } else {
                    format!("feature/{}", desc)
                }
            }
            BranchType::Fix => {
                if let Some(id) = task_id {
                    format!("fix/{}_{}", id, desc)
                } else {
                    format!("fix/{}", desc)
                }
            }
            BranchType::Chore => format!("chore/{}", desc),
            BranchType::Docs => {
                if let Some(id) = task_id {
                    format!("docs/{}_{}", id, desc)
                } else {
                    format!("docs/{}", desc)
                }
            }
            BranchType::Refactor => {
                if let Some(id) = task_id {
                    format!("refactor/{}_{}", id, desc)
                } else {
                    format!("refactor/{}", desc)
                }
            }
            BranchType::Test => {
                if let Some(id) = task_id {
                    format!("test/{}_{}", id, desc)
                } else {
                    format!("test/{}", desc)
                }
            }
            BranchType::Ci => format!("ci/{}", desc),
            BranchType::Release => format!("release/{}", desc),
        }
    }
}

impl Default for BranchConvention {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BranchError {
    #[error("invalid branch name format")]
    InvalidFormat,
    #[error("missing task ID in branch name")]
    MissingTaskId,
    #[error("branch already exists")]
    AlreadyExists,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_convention_creation() {
        let convention = BranchConvention::new();
        assert_eq!(convention.patterns.len(), 8);
        assert_eq!(convention.default_base, "develop");
    }

    #[test]
    fn test_validate_feature_branch() {
        let convention = BranchConvention::new();
        let result = convention.validate_branch_name("feature/TASK-FW-001_add-feature");
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.branch_type, BranchType::Feature);
        assert_eq!(info.task_id, Some("TASK-FW-001".to_string()));
    }

    #[test]
    fn test_validate_fix_branch() {
        let convention = BranchConvention::new();
        let result = convention.validate_branch_name("fix/TASK-FW-002_fix-bug");
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.branch_type, BranchType::Fix);
    }

    #[test]
    fn test_validate_chore_branch() {
        let convention = BranchConvention::new();
        let result = convention.validate_branch_name("chore/update-deps");
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.branch_type, BranchType::Chore);
        assert_eq!(info.task_id, None);
    }

    #[test]
    fn test_generate_branch_name() {
        let convention = BranchConvention::new();
        let name = convention.generate_branch_name(
            BranchType::Feature,
            Some("TASK-FW-001"),
            "add new feature",
        );
        assert_eq!(name, "feature/TASK-FW-001_add-new-feature");
    }

    #[test]
    fn test_generate_chore_branch_name() {
        let convention = BranchConvention::new();
        let name = convention.generate_branch_name(
            BranchType::Chore,
            None,
            "update dependencies",
        );
        assert_eq!(name, "chore/update-dependencies");
    }

    #[test]
    fn test_invalid_branch_format() {
        let convention = BranchConvention::new();
        let result = convention.validate_branch_name("invalid-branch");
        assert!(result.is_err());
    }
}

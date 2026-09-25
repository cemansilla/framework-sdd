use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub config: ProjectConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub msrv: Option<String>,
    pub edition: Option<String>,
    pub platforms: Vec<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
}

impl Project {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            version: version.into(),
            created_at: now,
            updated_at: now,
            config: ProjectConfig::default(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn with_config(mut self, config: ProjectConfig) -> Self {
        self.config = config;
        self.updated_at = Utc::now();
        self
    }
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            msrv: Some("1.80".to_string()),
            edition: Some("2021".to_string()),
            platforms: vec![
                "linux".to_string(),
                "macos".to_string(),
                "windows".to_string(),
            ],
            repository: None,
            license: Some("MIT".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let project = Project::new("test-project", "0.1.0");
        assert_eq!(project.name, "test-project");
        assert_eq!(project.version, "0.1.0");
        assert!(project.description.is_none());
    }

    #[test]
    fn test_project_with_description() {
        let project = Project::new("test-project", "0.1.0").with_description("A test project");
        assert_eq!(project.description, Some("A test project".to_string()));
    }

    #[test]
    fn test_project_config_default() {
        let config = ProjectConfig::default();
        assert_eq!(config.msrv, Some("1.80".to_string()));
        assert_eq!(config.edition, Some("2021".to_string()));
    }
}

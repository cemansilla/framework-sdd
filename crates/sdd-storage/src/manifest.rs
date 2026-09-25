use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub format_version: u32,
    pub project_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_migration: Option<u32>,
    pub metadata: HashMap<String, String>,
}

impl Manifest {
    pub fn new(project_id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            version: "0.1.0".to_string(),
            format_version: CURRENT_FORMAT_VERSION,
            project_id: project_id.into(),
            created_at: now,
            updated_at: now,
            last_migration: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn needs_migration(&self) -> bool {
        self.format_version < CURRENT_FORMAT_VERSION
    }

    pub fn apply_migration(&mut self, to_version: u32) {
        self.format_version = to_version;
        self.last_migration = Some(to_version);
        self.updated_at = Utc::now();
    }
}

pub const CURRENT_FORMAT_VERSION: u32 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = Manifest::new("test-project");
        assert_eq!(manifest.project_id, "test-project");
        assert_eq!(manifest.format_version, CURRENT_FORMAT_VERSION);
    }

    #[test]
    fn test_manifest_needs_migration() {
        let mut manifest = Manifest::new("test-project");
        assert!(!manifest.needs_migration());

        manifest.format_version = 0;
        assert!(manifest.needs_migration());
    }

    #[test]
    fn test_apply_migration() {
        let mut manifest = Manifest::new("test-project");
        manifest.format_version = 0;
        manifest.apply_migration(1);
        assert_eq!(manifest.format_version, 1);
        assert_eq!(manifest.last_migration, Some(1));
    }
}

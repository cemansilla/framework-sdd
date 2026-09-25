use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactCategory {
    Business,
    Discovery,
    Requirements,
    Domain,
    Architecture,
    Planning,
    Verification,
    Change,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: Uuid,
    pub category: ArtifactCategory,
    pub artifact_type: String,
    pub title: String,
    pub path: String,
    pub version: u32,
    pub hash: Option<String>,
    pub origin: ArtifactOrigin,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    pub relations: Vec<ArtifactRelation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactOrigin {
    Manual,
    Generated { source: String },
    Imported { source: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRelation {
    pub target_id: Uuid,
    pub relation_type: RelationType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationType {
    DependsOn,
    Implements,
    Verifies,
    Supersedes,
    RelatedTo,
    Parent,
    Child,
}

impl Artifact {
    pub fn new(
        category: ArtifactCategory,
        artifact_type: impl Into<String>,
        title: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            category,
            artifact_type: artifact_type.into(),
            title: title.into(),
            path: path.into(),
            version: 1,
            hash: None,
            origin: ArtifactOrigin::Manual,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
            relations: Vec::new(),
        }
    }

    pub fn with_hash(mut self, hash: impl Into<String>) -> Self {
        self.hash = Some(hash.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn with_origin(mut self, origin: ArtifactOrigin) -> Self {
        self.origin = origin;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn add_relation(mut self, target_id: Uuid, relation_type: RelationType) -> Self {
        self.relations.push(ArtifactRelation {
            target_id,
            relation_type,
        });
        self.updated_at = Utc::now();
        self
    }

    pub fn bump_version(&mut self) {
        self.version += 1;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_creation() {
        let artifact = Artifact::new(
            ArtifactCategory::Requirements,
            "functional-requirement",
            "User Login",
            ".docs/requirements/login.md",
        );
        assert_eq!(artifact.category, ArtifactCategory::Requirements);
        assert_eq!(artifact.title, "User Login");
        assert_eq!(artifact.version, 1);
    }

    #[test]
    fn test_artifact_with_hash() {
        let artifact = Artifact::new(
            ArtifactCategory::Domain,
            "entity",
            "User",
            ".docs/domain/user.md",
        )
        .with_hash("abc123");
        assert_eq!(artifact.hash, Some("abc123".to_string()));
    }

    #[test]
    fn test_artifact_relations() {
        let target_id = Uuid::new_v4();
        let artifact = Artifact::new(
            ArtifactCategory::Verification,
            "test",
            "Login Test",
            "tests/login.rs",
        )
        .add_relation(target_id, RelationType::Verifies);
        assert_eq!(artifact.relations.len(), 1);
        assert_eq!(artifact.relations[0].target_id, target_id);
    }

    #[test]
    fn test_bump_version() {
        let mut artifact = Artifact::new(
            ArtifactCategory::Architecture,
            "adr",
            "ADR-001",
            "docs/adr/001.md",
        );
        assert_eq!(artifact.version, 1);
        artifact.bump_version();
        assert_eq!(artifact.version, 2);
    }
}

use crate::change::{Change, ChangeStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub id: Uuid,
    pub change_ids: Vec<Uuid>,
    pub conflict_type: ConflictType,
    pub description: String,
    pub affected_artifacts: Vec<Uuid>,
    pub resolution: Option<ConflictResolution>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictType {
    ResourceConflict,
    SemanticConflict,
    DependencyConflict,
    OrderConflict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub strategy: ResolutionStrategy,
    pub resolved_by: String,
    pub notes: String,
    pub resolved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResolutionStrategy {
    FirstWins,
    LastWins,
    ManualMerge,
    Abort,
    Defer,
}

pub struct ConflictResolver;

impl ConflictResolver {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_conflicts(&self, changes: &[Change]) -> Vec<Conflict> {
        let mut conflicts = Vec::new();
        let mut artifact_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

        for change in changes {
            if change.status == ChangeStatus::Resolved || change.status == ChangeStatus::Rejected {
                continue;
            }

            for artifact_id_str in &change.affected_artifacts {
                if let Ok(artifact_id) = Uuid::parse_str(artifact_id_str) {
                    artifact_map.entry(artifact_id).or_default().push(change.id);
                }
            }
        }

        for (artifact_id, change_ids) in artifact_map {
            if change_ids.len() > 1 {
                conflicts.push(Conflict {
                    id: Uuid::new_v4(),
                    change_ids: change_ids.clone(),
                    conflict_type: ConflictType::ResourceConflict,
                    description: format!("Multiple changes affect artifact {}", artifact_id),
                    affected_artifacts: vec![artifact_id],
                    resolution: None,
                    created_at: Utc::now(),
                    resolved_at: None,
                });
            }
        }

        conflicts
    }

    pub fn resolve_conflict(
        &self,
        conflict: &mut Conflict,
        strategy: ResolutionStrategy,
        resolved_by: &str,
        notes: &str,
    ) {
        conflict.resolution = Some(ConflictResolution {
            strategy,
            resolved_by: resolved_by.to_string(),
            notes: notes.to_string(),
            resolved_at: Utc::now(),
        });
        conflict.resolved_at = Some(Utc::now());
    }

    pub fn get_unresolved_conflicts<'a>(&self, conflicts: &'a [Conflict]) -> Vec<&'a Conflict> {
        conflicts
            .iter()
            .filter(|c| c.resolution.is_none())
            .collect()
    }

    pub fn get_conflicts_for_change<'a>(
        &self,
        conflicts: &'a [Conflict],
        change_id: Uuid,
    ) -> Vec<&'a Conflict> {
        conflicts
            .iter()
            .filter(|c| c.change_ids.contains(&change_id))
            .collect()
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change::{ChangeOrigin, ChangeType};

    #[test]
    fn test_conflict_resolver_creation() {
        let resolver = ConflictResolver::new();
        let _ = resolver;
    }

    #[test]
    fn test_detect_no_conflicts() {
        let resolver = ConflictResolver::new();
        let change1 = Change::new(
            "CHG-001",
            "Test 1",
            "Test",
            ChangeType::Requirement,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        );
        let change2 = Change::new(
            "CHG-002",
            "Test 2",
            "Test",
            ChangeType::Architecture,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        );

        let conflicts = resolver.detect_conflicts(&[change1, change2]);
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_detect_conflict() {
        let resolver = ConflictResolver::new();
        let artifact_id = Uuid::new_v4().to_string();

        let change1 = Change::new(
            "CHG-001",
            "Test 1",
            "Test",
            ChangeType::Requirement,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        )
        .add_affected_artifact(&artifact_id);

        let change2 = Change::new(
            "CHG-002",
            "Test 2",
            "Test",
            ChangeType::Architecture,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        )
        .add_affected_artifact(&artifact_id);

        let conflicts = resolver.detect_conflicts(&[change1, change2]);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::ResourceConflict);
    }

    #[test]
    fn test_resolve_conflict() {
        let resolver = ConflictResolver::new();
        let mut conflict = Conflict {
            id: Uuid::new_v4(),
            change_ids: vec![Uuid::new_v4()],
            conflict_type: ConflictType::ResourceConflict,
            description: "Test".to_string(),
            affected_artifacts: vec![],
            resolution: None,
            created_at: Utc::now(),
            resolved_at: None,
        };

        resolver.resolve_conflict(
            &mut conflict,
            ResolutionStrategy::FirstWins,
            "user",
            "Resolved",
        );

        assert!(conflict.resolution.is_some());
        assert!(conflict.resolved_at.is_some());
    }
}

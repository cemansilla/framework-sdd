use crate::artifact::Artifact;
use crate::change::{Change, ChangeOrigin, ChangeType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeDetection {
    pub id: Uuid,
    pub detected_at: DateTime<Utc>,
    pub changes: Vec<DetectedChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedChange {
    pub artifact_id: Uuid,
    pub artifact_type: String,
    pub change_type: DetectedChangeType,
    pub old_hash: Option<String>,
    pub new_hash: Option<String>,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DetectedChangeType {
    Created,
    Modified,
    Deleted,
    Moved,
    Renamed,
}

pub struct ChangeDetector;

impl ChangeDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_changes(
        &self,
        old_artifacts: &[Artifact],
        new_artifacts: &[Artifact],
    ) -> ChangeDetection {
        let mut changes = Vec::new();

        let old_map: HashMap<Uuid, &Artifact> = old_artifacts.iter().map(|a| (a.id, a)).collect();
        let new_map: HashMap<Uuid, &Artifact> = new_artifacts.iter().map(|a| (a.id, a)).collect();

        for (id, new_artifact) in &new_map {
            if let Some(old_artifact) = old_map.get(id) {
                if old_artifact.hash != new_artifact.hash {
                    changes.push(DetectedChange {
                        artifact_id: *id,
                        artifact_type: format!("{:?}", new_artifact.category),
                        change_type: DetectedChangeType::Modified,
                        old_hash: old_artifact.hash.clone(),
                        new_hash: new_artifact.hash.clone(),
                        details: HashMap::new(),
                    });
                }
            } else {
                changes.push(DetectedChange {
                    artifact_id: *id,
                    artifact_type: format!("{:?}", new_artifact.category),
                    change_type: DetectedChangeType::Created,
                    old_hash: None,
                    new_hash: new_artifact.hash.clone(),
                    details: HashMap::new(),
                });
            }
        }

        for (id, old_artifact) in &old_map {
            if !new_map.contains_key(id) {
                changes.push(DetectedChange {
                    artifact_id: *id,
                    artifact_type: format!("{:?}", old_artifact.category),
                    change_type: DetectedChangeType::Deleted,
                    old_hash: old_artifact.hash.clone(),
                    new_hash: None,
                    details: HashMap::new(),
                });
            }
        }

        ChangeDetection {
            id: Uuid::new_v4(),
            detected_at: Utc::now(),
            changes,
        }
    }

    pub fn create_change_from_detection(
        &self,
        detection: &DetectedChange,
        origin: ChangeOrigin,
    ) -> Change {
        let change_type = match detection.artifact_type.as_str() {
            "Requirement" => ChangeType::Requirement,
            "Architecture" => ChangeType::Architecture,
            "Design" => ChangeType::Design,
            "Implementation" => ChangeType::Implementation,
            "Test" => ChangeType::Bugfix,
            "Documentation" => ChangeType::Documentation,
            _ => ChangeType::Implementation,
        };

        let title = format!(
            "{:?} artifact: {}",
            detection.change_type, detection.artifact_type
        );

        let description = format!(
            "Detected {:?} change in {} artifact {}",
            detection.change_type, detection.artifact_type, detection.artifact_id
        );

        Change::new(
            format!("CHG-{}", Uuid::new_v4()),
            title,
            description,
            change_type,
            origin,
        )
        .add_affected_artifact(detection.artifact_id.to_string())
    }
}

impl Default for ChangeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::{Artifact, ArtifactCategory};

    #[test]
    fn test_change_detector_creation() {
        let detector = ChangeDetector::new();
        let _ = detector;
    }

    #[test]
    fn test_detect_created_artifact() {
        let detector = ChangeDetector::new();
        let old_artifacts = vec![];
        let new_artifact = Artifact::new(ArtifactCategory::Business, "doc", "test", "test.md")
            .with_hash("hash123");
        let new_artifacts = vec![new_artifact];

        let detection = detector.detect_changes(&old_artifacts, &new_artifacts);
        assert_eq!(detection.changes.len(), 1);
        assert_eq!(
            detection.changes[0].change_type,
            DetectedChangeType::Created
        );
    }

    #[test]
    fn test_detect_modified_artifact() {
        let detector = ChangeDetector::new();
        let old_artifact = Artifact::new(ArtifactCategory::Business, "doc", "test", "test.md")
            .with_hash("hash123");
        let mut new_artifact = old_artifact.clone();
        new_artifact.hash = Some("hash456".to_string());

        let detection = detector.detect_changes(&[old_artifact], &[new_artifact]);
        assert_eq!(detection.changes.len(), 1);
        assert_eq!(
            detection.changes[0].change_type,
            DetectedChangeType::Modified
        );
    }

    #[test]
    fn test_detect_deleted_artifact() {
        let detector = ChangeDetector::new();
        let old_artifact = Artifact::new(ArtifactCategory::Business, "doc", "test", "test.md")
            .with_hash("hash123");
        let new_artifacts = vec![];

        let detection = detector.detect_changes(&[old_artifact], &new_artifacts);
        assert_eq!(detection.changes.len(), 1);
        assert_eq!(
            detection.changes[0].change_type,
            DetectedChangeType::Deleted
        );
    }

    #[test]
    fn test_create_change_from_detection() {
        let detector = ChangeDetector::new();
        let detected = DetectedChange {
            artifact_id: Uuid::new_v4(),
            artifact_type: "Requirement".to_string(),
            change_type: DetectedChangeType::Modified,
            old_hash: Some("old".to_string()),
            new_hash: Some("new".to_string()),
            details: HashMap::new(),
        };

        let origin = ChangeOrigin::User {
            reason: "Test".to_string(),
        };
        let change = detector.create_change_from_detection(&detected, origin);
        assert_eq!(change.change_type, ChangeType::Requirement);
        assert_eq!(change.affected_artifacts.len(), 1);
    }
}

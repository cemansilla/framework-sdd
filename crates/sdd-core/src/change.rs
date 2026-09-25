use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub id: Uuid,
    pub change_id: String,
    pub title: String,
    pub description: String,
    pub change_type: ChangeType,
    pub origin: ChangeOrigin,
    pub status: ChangeStatus,
    pub impact: ImpactAnalysis,
    pub affected_artifacts: Vec<String>,
    pub related_commits: Vec<String>,
    pub related_tasks: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeType {
    Requirement,
    Architecture,
    Design,
    Implementation,
    Bugfix,
    Refactor,
    Documentation,
    Configuration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeOrigin {
    User { reason: String },
    Review { finding_id: String },
    Test { test_id: String },
    Automated { source: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeStatus {
    Detected,
    Analyzed,
    Approved,
    InProgress,
    Resolved,
    Rejected,
    Deferred,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub affected_modules: Vec<String>,
    pub affected_tests: Vec<String>,
    pub breaking_changes: Vec<String>,
    pub risk_level: RiskLevel,
    pub estimated_effort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

impl Change {
    pub fn new(
        change_id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        change_type: ChangeType,
        origin: ChangeOrigin,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            change_id: change_id.into(),
            title: title.into(),
            description: description.into(),
            change_type,
            origin,
            status: ChangeStatus::Detected,
            impact: ImpactAnalysis::default(),
            affected_artifacts: Vec::new(),
            related_commits: Vec::new(),
            related_tasks: Vec::new(),
            created_at: now,
            updated_at: now,
            resolved_at: None,
        }
    }

    pub fn with_impact(mut self, impact: ImpactAnalysis) -> Self {
        self.impact = impact;
        self.status = ChangeStatus::Analyzed;
        self.updated_at = Utc::now();
        self
    }

    pub fn add_affected_artifact(mut self, artifact_id: impl Into<String>) -> Self {
        self.affected_artifacts.push(artifact_id.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn add_related_commit(mut self, commit_hash: impl Into<String>) -> Self {
        self.related_commits.push(commit_hash.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn add_related_task(mut self, task_id: impl Into<String>) -> Self {
        self.related_tasks.push(task_id.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn approve(mut self) -> Self {
        self.status = ChangeStatus::Approved;
        self.updated_at = Utc::now();
        self
    }

    pub fn start_progress(mut self) -> Self {
        self.status = ChangeStatus::InProgress;
        self.updated_at = Utc::now();
        self
    }

    pub fn resolve(mut self) -> Self {
        self.status = ChangeStatus::Resolved;
        self.resolved_at = Some(Utc::now());
        self.updated_at = Utc::now();
        self
    }

    pub fn reject(mut self) -> Self {
        self.status = ChangeStatus::Rejected;
        self.updated_at = Utc::now();
        self
    }

    pub fn defer(mut self) -> Self {
        self.status = ChangeStatus::Deferred;
        self.updated_at = Utc::now();
        self
    }

    pub fn is_resolved(&self) -> bool {
        self.status == ChangeStatus::Resolved
    }

    pub fn is_breaking(&self) -> bool {
        !self.impact.breaking_changes.is_empty()
    }
}

impl ImpactAnalysis {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_affected_module(mut self, module: impl Into<String>) -> Self {
        self.affected_modules.push(module.into());
        self
    }

    pub fn add_affected_test(mut self, test: impl Into<String>) -> Self {
        self.affected_tests.push(test.into());
        self
    }

    pub fn add_breaking_change(mut self, change: impl Into<String>) -> Self {
        self.breaking_changes.push(change.into());
        self
    }

    pub fn with_risk_level(mut self, level: RiskLevel) -> Self {
        self.risk_level = level;
        self
    }

    pub fn with_effort(mut self, effort: impl Into<String>) -> Self {
        self.estimated_effort = Some(effort.into());
        self
    }
}

impl Default for ImpactAnalysis {
    fn default() -> Self {
        Self {
            affected_modules: Vec::new(),
            affected_tests: Vec::new(),
            breaking_changes: Vec::new(),
            risk_level: RiskLevel::Low,
            estimated_effort: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    pub id: Uuid,
    pub change_id: String,
    pub version: String,
    pub date: DateTime<Utc>,
    pub entries: Vec<ChangelogItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogItem {
    pub change_type: ChangeType,
    pub description: String,
    pub related_issues: Vec<String>,
}

impl ChangelogEntry {
    pub fn new(change_id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            change_id: change_id.into(),
            version: version.into(),
            date: Utc::now(),
            entries: Vec::new(),
        }
    }

    pub fn add_entry(mut self, change_type: ChangeType, description: impl Into<String>) -> Self {
        self.entries.push(ChangelogItem {
            change_type,
            description: description.into(),
            related_issues: Vec::new(),
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_creation() {
        let change = Change::new(
            "CHG-001",
            "Update API",
            "Update the API to v2",
            ChangeType::Architecture,
            ChangeOrigin::User {
                reason: "New requirements".to_string(),
            },
        );
        assert_eq!(change.status, ChangeStatus::Detected);
    }

    #[test]
    fn test_change_with_impact() {
        let impact = ImpactAnalysis::new()
            .add_affected_module("sdd-core")
            .with_risk_level(RiskLevel::Medium);
        let change = Change::new(
            "CHG-001",
            "Update API",
            "Update the API to v2",
            ChangeType::Architecture,
            ChangeOrigin::User {
                reason: "New requirements".to_string(),
            },
        )
        .with_impact(impact);
        assert_eq!(change.status, ChangeStatus::Analyzed);
        assert_eq!(change.impact.risk_level, RiskLevel::Medium);
    }

    #[test]
    fn test_change_resolve() {
        let change = Change::new(
            "CHG-001",
            "Update API",
            "Update the API to v2",
            ChangeType::Architecture,
            ChangeOrigin::User {
                reason: "New requirements".to_string(),
            },
        )
        .approve()
        .start_progress()
        .resolve();
        assert!(change.is_resolved());
        assert!(change.resolved_at.is_some());
    }

    #[test]
    fn test_breaking_change() {
        let impact = ImpactAnalysis::new().add_breaking_change("Changed API signature");
        let change = Change::new(
            "CHG-001",
            "Update API",
            "Update the API to v2",
            ChangeType::Architecture,
            ChangeOrigin::User {
                reason: "New requirements".to_string(),
            },
        )
        .with_impact(impact);
        assert!(change.is_breaking());
    }

    #[test]
    fn test_changelog_entry() {
        let entry = ChangelogEntry::new("CHG-001", "0.2.0")
            .add_entry(ChangeType::Requirement, "Added new feature");
        assert_eq!(entry.entries.len(), 1);
    }
}

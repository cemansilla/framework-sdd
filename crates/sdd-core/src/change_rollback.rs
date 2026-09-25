use crate::change::{Change, ChangeStatus};
use crate::change_history::ChangeHistory;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    pub id: Uuid,
    pub change_id: Uuid,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub steps: Vec<RollbackStep>,
    pub status: RollbackStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStep {
    pub step_id: Uuid,
    pub artifact_id: Uuid,
    pub action: RollbackAction,
    pub target_version: Option<String>,
    pub status: RollbackStepStatus,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RollbackAction {
    RestoreVersion(String),
    DeleteCreated,
    RevertModification,
    NotifyStakeholders,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RollbackStepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RollbackStatus {
    Planning,
    Ready,
    Executing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub rollback_id: Uuid,
    pub change_id: Uuid,
    pub success: bool,
    pub completed_steps: usize,
    pub failed_steps: usize,
    pub errors: Vec<String>,
    pub completed_at: DateTime<Utc>,
}

pub struct ChangeRollback;

impl ChangeRollback {
    pub fn new() -> Self {
        Self
    }

    pub fn create_rollback_plan(
        &self,
        change: &Change,
        reason: &str,
        artifact_versions: &HashMap<Uuid, String>,
    ) -> RollbackPlan {
        let mut steps = Vec::new();

        for artifact_id_str in &change.affected_artifacts {
            if let Ok(artifact_id) = Uuid::parse_str(artifact_id_str) {
                let action = if let Some(version) = artifact_versions.get(&artifact_id) {
                    RollbackAction::RestoreVersion(version.clone())
                } else {
                    RollbackAction::DeleteCreated
                };

                steps.push(RollbackStep {
                    step_id: Uuid::new_v4(),
                    artifact_id,
                    action,
                    target_version: artifact_versions.get(&artifact_id).cloned(),
                    status: RollbackStepStatus::Pending,
                    completed_at: None,
                });
            }
        }

        steps.push(RollbackStep {
            step_id: Uuid::new_v4(),
            artifact_id: change.id,
            action: RollbackAction::NotifyStakeholders,
            target_version: None,
            status: RollbackStepStatus::Pending,
            completed_at: None,
        });

        RollbackPlan {
            id: Uuid::new_v4(),
            change_id: change.id,
            reason: reason.to_string(),
            created_at: Utc::now(),
            steps,
            status: RollbackStatus::Ready,
        }
    }

    pub fn execute_step(&self, plan: &mut RollbackPlan, step_id: Uuid) -> bool {
        if let Some(step) = plan.steps.iter_mut().find(|s| s.step_id == step_id) {
            step.status = RollbackStepStatus::Completed;
            step.completed_at = Some(Utc::now());

            if plan
                .steps
                .iter()
                .all(|s| s.status == RollbackStepStatus::Completed)
            {
                plan.status = RollbackStatus::Completed;
            } else {
                plan.status = RollbackStatus::Executing;
            }

            true
        } else {
            false
        }
    }

    pub fn fail_step(&self, plan: &mut RollbackPlan, step_id: Uuid) -> bool {
        if let Some(step) = plan.steps.iter_mut().find(|s| s.step_id == step_id) {
            step.status = RollbackStepStatus::Failed;
            plan.status = RollbackStatus::Failed;
            true
        } else {
            false
        }
    }

    pub fn create_result(&self, plan: &RollbackPlan) -> RollbackResult {
        let completed_steps = plan
            .steps
            .iter()
            .filter(|s| s.status == RollbackStepStatus::Completed)
            .count();

        let failed_steps = plan
            .steps
            .iter()
            .filter(|s| s.status == RollbackStepStatus::Failed)
            .count();

        let success = plan.status == RollbackStatus::Completed;

        RollbackResult {
            rollback_id: plan.id,
            change_id: plan.change_id,
            success,
            completed_steps,
            failed_steps,
            errors: Vec::new(),
            completed_at: Utc::now(),
        }
    }

    pub fn record_rollback_in_history(
        &self,
        change: &Change,
        history: &mut ChangeHistory,
        actor: &str,
        reason: &str,
    ) {
        history.record_rollback(change, actor, reason);
    }

    pub fn can_rollback(&self, change: &Change) -> bool {
        matches!(
            change.status,
            ChangeStatus::Approved | ChangeStatus::InProgress | ChangeStatus::Resolved
        )
    }

    pub fn get_pending_steps<'a>(&self, plan: &'a RollbackPlan) -> Vec<&'a RollbackStep> {
        plan.steps
            .iter()
            .filter(|s| s.status == RollbackStepStatus::Pending)
            .collect()
    }
}

impl Default for ChangeRollback {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change::{ChangeOrigin, ChangeType};

    #[test]
    fn test_rollback_creation() {
        let rollback = ChangeRollback::new();
        let _ = rollback;
    }

    #[test]
    fn test_create_rollback_plan() {
        let rollback = ChangeRollback::new();
        let change = Change::new(
            "CHG-001",
            "Test",
            "Test",
            ChangeType::Requirement,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        )
        .add_affected_artifact(Uuid::new_v4().to_string());

        let mut versions = HashMap::new();
        versions.insert(Uuid::new_v4(), "v1.0".to_string());

        let plan = rollback.create_rollback_plan(&change, "Test reason", &versions);
        assert_eq!(plan.status, RollbackStatus::Ready);
        assert!(!plan.steps.is_empty());
    }

    #[test]
    fn test_execute_step() {
        let rollback = ChangeRollback::new();
        let change = Change::new(
            "CHG-001",
            "Test",
            "Test",
            ChangeType::Requirement,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        )
        .add_affected_artifact(Uuid::new_v4().to_string());

        let versions = HashMap::new();
        let mut plan = rollback.create_rollback_plan(&change, "Test", &versions);

        let step_id = plan.steps[0].step_id;
        assert!(rollback.execute_step(&mut plan, step_id));
        assert_eq!(plan.steps[0].status, RollbackStepStatus::Completed);
    }

    #[test]
    fn test_can_rollback() {
        let rollback = ChangeRollback::new();
        let change = Change::new(
            "CHG-001",
            "Test",
            "Test",
            ChangeType::Requirement,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        )
        .approve();

        assert!(rollback.can_rollback(&change));
    }
}

use crate::change::{Change, ChangeStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub change_id: Uuid,
    pub validated_at: DateTime<Utc>,
    pub is_valid: bool,
    pub checks: Vec<ValidationCheck>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

pub struct ChangeValidator;

impl ChangeValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, change: &Change) -> ValidationResult {
        let mut checks = Vec::new();
        let errors = Vec::new();
        let mut warnings = Vec::new();

        checks.push(self.check_has_description(change));
        checks.push(self.check_has_affected_artifacts(change));
        checks.push(self.check_status_transition(change));

        if change.impact.breaking_changes.is_empty() {
            warnings.push("No breaking changes documented".to_string());
        }

        let is_valid = errors.is_empty() && checks.iter().all(|c| c.passed);

        ValidationResult {
            change_id: change.id,
            validated_at: Utc::now(),
            is_valid,
            checks,
            errors,
            warnings,
        }
    }

    fn check_has_description(&self, change: &Change) -> ValidationCheck {
        let passed = !change.description.trim().is_empty();
        ValidationCheck {
            name: "has_description".to_string(),
            passed,
            message: if passed {
                "Change has description".to_string()
            } else {
                "Change must have a description".to_string()
            },
        }
    }

    fn check_has_affected_artifacts(&self, change: &Change) -> ValidationCheck {
        let passed = !change.affected_artifacts.is_empty();
        ValidationCheck {
            name: "has_affected_artifacts".to_string(),
            passed,
            message: if passed {
                "Change has affected artifacts".to_string()
            } else {
                "Change must have at least one affected artifact".to_string()
            },
        }
    }

    fn check_status_transition(&self, change: &Change) -> ValidationCheck {
        let valid_transitions = match change.status {
            ChangeStatus::Detected => true,
            ChangeStatus::Analyzed => true,
            ChangeStatus::Approved => true,
            ChangeStatus::InProgress => true,
            ChangeStatus::Resolved => true,
            ChangeStatus::Rejected => true,
            ChangeStatus::Deferred => true,
        };

        ValidationCheck {
            name: "valid_status".to_string(),
            passed: valid_transitions,
            message: "Change has valid status".to_string(),
        }
    }

    pub fn validate_for_approval(&self, change: &Change) -> ValidationResult {
        let mut result = self.validate(change);

        if change.status != ChangeStatus::Analyzed {
            result
                .errors
                .push("Change must be analyzed before approval".to_string());
            result.is_valid = false;
        }

        result
    }

    pub fn validate_for_resolution(&self, change: &Change) -> ValidationResult {
        let mut result = self.validate(change);

        if change.status != ChangeStatus::InProgress {
            result
                .errors
                .push("Change must be in progress before resolution".to_string());
            result.is_valid = false;
        }

        result
    }
}

impl Default for ChangeValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change::{ChangeOrigin, ChangeType};

    #[test]
    fn test_validator_creation() {
        let validator = ChangeValidator::new();
        let _ = validator;
    }

    #[test]
    fn test_validate_valid_change() {
        let validator = ChangeValidator::new();
        let change = Change::new(
            "CHG-001",
            "Test",
            "Test description",
            ChangeType::Requirement,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        )
        .add_affected_artifact(Uuid::new_v4().to_string());

        let result = validator.validate(&change);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_missing_description() {
        let validator = ChangeValidator::new();
        let change = Change::new(
            "CHG-001",
            "Test",
            "",
            ChangeType::Requirement,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        )
        .add_affected_artifact(Uuid::new_v4().to_string());

        let result = validator.validate(&change);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_for_approval() {
        let validator = ChangeValidator::new();
        let change = Change::new(
            "CHG-001",
            "Test",
            "Test description",
            ChangeType::Requirement,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        )
        .add_affected_artifact(Uuid::new_v4().to_string())
        .with_impact(crate::change::ImpactAnalysis::new());

        let result = validator.validate_for_approval(&change);
        assert!(result.is_valid);
    }
}

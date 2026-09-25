use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RequirementType {
    Functional,
    NonFunctional,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RequirementStatus {
    Draft,
    Proposed,
    Accepted,
    Implemented,
    Verified,
    Deprecated,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: Uuid,
    pub req_id: String,
    pub title: String,
    pub description: String,
    pub req_type: RequirementType,
    pub status: RequirementStatus,
    pub priority: Priority,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub description: String,
    pub verified: bool,
}

impl Requirement {
    pub fn new(
        req_id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        req_type: RequirementType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            req_id: req_id.into(),
            title: title.into(),
            description: description.into(),
            req_type,
            status: RequirementStatus::Draft,
            priority: Priority::Medium,
            acceptance_criteria: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self.updated_at = Utc::now();
        self
    }

    pub fn add_acceptance_criterion(
        mut self,
        id: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.acceptance_criteria.push(AcceptanceCriterion {
            id: id.into(),
            description: description.into(),
            verified: false,
        });
        self.updated_at = Utc::now();
        self
    }

    pub fn transition_to(&mut self, status: RequirementStatus) -> Result<(), TransitionError> {
        if !self.is_valid_transition(&status) {
            return Err(TransitionError::InvalidTransition {
                from: self.status.clone(),
                to: status,
            });
        }
        self.status = status;
        self.updated_at = Utc::now();
        Ok(())
    }

    fn is_valid_transition(&self, target: &RequirementStatus) -> bool {
        matches!(
            (&self.status, target),
            (RequirementStatus::Draft, RequirementStatus::Proposed)
                | (RequirementStatus::Draft, RequirementStatus::Rejected)
                | (RequirementStatus::Proposed, RequirementStatus::Accepted)
                | (RequirementStatus::Proposed, RequirementStatus::Rejected)
                | (RequirementStatus::Accepted, RequirementStatus::Implemented)
                | (RequirementStatus::Accepted, RequirementStatus::Deprecated)
                | (RequirementStatus::Implemented, RequirementStatus::Verified)
                | (RequirementStatus::Verified, RequirementStatus::Deprecated)
        )
    }

    pub fn is_ready(&self) -> bool {
        !self.acceptance_criteria.is_empty() && matches!(self.status, RequirementStatus::Accepted)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TransitionError {
    #[error("Invalid transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: RequirementStatus,
        to: RequirementStatus,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requirement_creation() {
        let req = Requirement::new(
            "REQ-001",
            "User Login",
            "Users must be able to log in",
            RequirementType::Functional,
        );
        assert_eq!(req.req_id, "REQ-001");
        assert_eq!(req.status, RequirementStatus::Draft);
        assert_eq!(req.req_type, RequirementType::Functional);
    }

    #[test]
    fn test_add_acceptance_criteria() {
        let req = Requirement::new(
            "REQ-001",
            "User Login",
            "Users must be able to log in",
            RequirementType::Functional,
        )
        .add_acceptance_criterion("AC-1", "User enters credentials")
        .add_acceptance_criterion("AC-2", "System validates credentials");
        assert_eq!(req.acceptance_criteria.len(), 2);
    }

    #[test]
    fn test_valid_transition() {
        let mut req = Requirement::new(
            "REQ-001",
            "User Login",
            "Users must be able to log in",
            RequirementType::Functional,
        );
        assert!(req.transition_to(RequirementStatus::Proposed).is_ok());
        assert_eq!(req.status, RequirementStatus::Proposed);
    }

    #[test]
    fn test_invalid_transition() {
        let mut req = Requirement::new(
            "REQ-001",
            "User Login",
            "Users must be able to log in",
            RequirementType::Functional,
        );
        assert!(req.transition_to(RequirementStatus::Implemented).is_err());
    }

    #[test]
    fn test_is_ready() {
        let req = Requirement::new(
            "REQ-001",
            "User Login",
            "Users must be able to log in",
            RequirementType::Functional,
        )
        .add_acceptance_criterion("AC-1", "User enters credentials");
        assert!(!req.is_ready());
    }
}

use crate::lifecycle::LifecycleState;
use crate::project::Project;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub id: Uuid,
    pub project: Project,
    pub current_state: LifecycleState,
    pub state_history: Vec<StateTransition>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from: LifecycleState,
    pub to: LifecycleState,
    pub timestamp: DateTime<Utc>,
    pub reason: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum LifecycleError {
    #[error("invalid transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: LifecycleState,
        to: LifecycleState,
    },
    #[error("project not in required state: expected {expected:?}, got {actual:?}")]
    WrongState {
        expected: LifecycleState,
        actual: LifecycleState,
    },
    #[error("validation error: {0}")]
    ValidationError(String),
}

impl ProjectContext {
    pub fn new(project: Project) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            project,
            current_state: LifecycleState::Brainstorming,
            state_history: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn transition_to(
        &mut self,
        target: LifecycleState,
        reason: Option<String>,
    ) -> Result<(), LifecycleError> {
        if !self.current_state.can_transition_to(&target) {
            return Err(LifecycleError::InvalidTransition {
                from: self.current_state.clone(),
                to: target,
            });
        }

        let transition = StateTransition {
            from: self.current_state.clone(),
            to: target.clone(),
            timestamp: Utc::now(),
            reason,
        };

        self.state_history.push(transition);
        self.current_state = target;
        self.updated_at = Utc::now();

        Ok(())
    }

    pub fn is_in_state(&self, state: &LifecycleState) -> bool {
        &self.current_state == state
    }

    pub fn can_transition_to(&self, target: &LifecycleState) -> bool {
        self.current_state.can_transition_to(target)
    }

    pub fn time_in_current_state(&self) -> chrono::Duration {
        if let Some(last_transition) = self.state_history.last() {
            Utc::now() - last_transition.timestamp
        } else {
            Utc::now() - self.created_at
        }
    }

    pub fn transition_count(&self) -> usize {
        self.state_history.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_project() -> Project {
        Project::new("test-project", "0.1.0")
    }

    #[test]
    fn test_project_context_creation() {
        let project = create_test_project();
        let context = ProjectContext::new(project);
        assert_eq!(context.current_state, LifecycleState::Brainstorming);
        assert!(context.state_history.is_empty());
    }

    #[test]
    fn test_valid_transition() {
        let project = create_test_project();
        let mut context = ProjectContext::new(project);

        let result = context.transition_to(LifecycleState::Discovery, None);
        assert!(result.is_ok());
        assert_eq!(context.current_state, LifecycleState::Discovery);
        assert_eq!(context.state_history.len(), 1);
    }

    #[test]
    fn test_invalid_transition() {
        let project = create_test_project();
        let mut context = ProjectContext::new(project);

        let result = context.transition_to(LifecycleState::Implementation, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_transition_with_reason() {
        let project = create_test_project();
        let mut context = ProjectContext::new(project);

        context
            .transition_to(
                LifecycleState::Discovery,
                Some("Initial exploration complete".to_string()),
            )
            .unwrap();

        assert_eq!(
            context.state_history[0].reason,
            Some("Initial exploration complete".to_string())
        );
    }

    #[test]
    fn test_full_lifecycle() {
        let project = create_test_project();
        let mut context = ProjectContext::new(project);

        context
            .transition_to(LifecycleState::Discovery, None)
            .unwrap();
        context
            .transition_to(LifecycleState::Requirements, None)
            .unwrap();
        context.transition_to(LifecycleState::Domain, None).unwrap();
        context
            .transition_to(LifecycleState::Architecture, None)
            .unwrap();
        context.transition_to(LifecycleState::Design, None).unwrap();
        context
            .transition_to(LifecycleState::Planning, None)
            .unwrap();
        context
            .transition_to(LifecycleState::Implementation, None)
            .unwrap();
        context.transition_to(LifecycleState::Review, None).unwrap();
        context
            .transition_to(LifecycleState::Testing, None)
            .unwrap();
        context.transition_to(LifecycleState::Done, None).unwrap();

        assert_eq!(context.current_state, LifecycleState::Done);
        assert_eq!(context.transition_count(), 10);
    }

    #[test]
    fn test_change_cycle() {
        let project = create_test_project();
        let mut context = ProjectContext::new(project);

        context
            .transition_to(LifecycleState::Done, None)
            .unwrap_err();

        for state in [
            LifecycleState::Discovery,
            LifecycleState::Requirements,
            LifecycleState::Domain,
            LifecycleState::Architecture,
            LifecycleState::Design,
            LifecycleState::Planning,
            LifecycleState::Implementation,
            LifecycleState::Review,
            LifecycleState::Testing,
            LifecycleState::Done,
        ] {
            context.transition_to(state, None).unwrap();
        }

        context.transition_to(LifecycleState::Change, None).unwrap();
        assert_eq!(context.current_state, LifecycleState::Change);

        context
            .transition_to(LifecycleState::Requirements, None)
            .unwrap();
        assert_eq!(context.current_state, LifecycleState::Requirements);
    }

    #[test]
    fn test_is_in_state() {
        let project = create_test_project();
        let context = ProjectContext::new(project);
        assert!(context.is_in_state(&LifecycleState::Brainstorming));
        assert!(!context.is_in_state(&LifecycleState::Discovery));
    }

    #[test]
    fn test_can_transition_to() {
        let project = create_test_project();
        let context = ProjectContext::new(project);
        assert!(context.can_transition_to(&LifecycleState::Discovery));
        assert!(!context.can_transition_to(&LifecycleState::Implementation));
    }
}

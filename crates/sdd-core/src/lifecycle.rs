use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LifecycleState {
    Brainstorming,
    Discovery,
    Requirements,
    Domain,
    Architecture,
    Design,
    Planning,
    Implementation,
    Review,
    Testing,
    Done,
    Change,
}

impl LifecycleState {
    pub fn can_transition_to(&self, target: &LifecycleState) -> bool {
        matches!(
            (self, target),
            (LifecycleState::Brainstorming, LifecycleState::Discovery)
                | (LifecycleState::Discovery, LifecycleState::Requirements)
                | (LifecycleState::Requirements, LifecycleState::Domain)
                | (LifecycleState::Domain, LifecycleState::Architecture)
                | (LifecycleState::Architecture, LifecycleState::Design)
                | (LifecycleState::Design, LifecycleState::Planning)
                | (LifecycleState::Planning, LifecycleState::Implementation)
                | (LifecycleState::Implementation, LifecycleState::Review)
                | (LifecycleState::Review, LifecycleState::Testing)
                | (LifecycleState::Testing, LifecycleState::Done)
                | (LifecycleState::Done, LifecycleState::Change)
                | (LifecycleState::Change, LifecycleState::Requirements)
                | (LifecycleState::Change, LifecycleState::Architecture)
                | (LifecycleState::Change, LifecycleState::Implementation)
        )
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, LifecycleState::Done)
    }

    pub fn is_active(&self) -> bool {
        !matches!(self, LifecycleState::Done | LifecycleState::Change)
    }
}

impl std::fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifecycleState::Brainstorming => write!(f, "Brainstorming"),
            LifecycleState::Discovery => write!(f, "Discovery"),
            LifecycleState::Requirements => write!(f, "Requirements"),
            LifecycleState::Domain => write!(f, "Domain"),
            LifecycleState::Architecture => write!(f, "Architecture"),
            LifecycleState::Design => write!(f, "Design"),
            LifecycleState::Planning => write!(f, "Planning"),
            LifecycleState::Implementation => write!(f, "Implementation"),
            LifecycleState::Review => write!(f, "Review"),
            LifecycleState::Testing => write!(f, "Testing"),
            LifecycleState::Done => write!(f, "Done"),
            LifecycleState::Change => write!(f, "Change"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        assert!(LifecycleState::Brainstorming.can_transition_to(&LifecycleState::Discovery));
        assert!(LifecycleState::Discovery.can_transition_to(&LifecycleState::Requirements));
        assert!(LifecycleState::Implementation.can_transition_to(&LifecycleState::Review));
    }

    #[test]
    fn test_invalid_transitions() {
        assert!(!LifecycleState::Brainstorming.can_transition_to(&LifecycleState::Implementation));
        assert!(!LifecycleState::Done.can_transition_to(&LifecycleState::Brainstorming));
    }

    #[test]
    fn test_change_transitions() {
        assert!(LifecycleState::Done.can_transition_to(&LifecycleState::Change));
        assert!(LifecycleState::Change.can_transition_to(&LifecycleState::Requirements));
    }

    #[test]
    fn test_is_terminal() {
        assert!(LifecycleState::Done.is_terminal());
        assert!(!LifecycleState::Implementation.is_terminal());
    }

    #[test]
    fn test_display() {
        assert_eq!(
            format!("{}", LifecycleState::Brainstorming),
            "Brainstorming"
        );
        assert_eq!(
            format!("{}", LifecycleState::Implementation),
            "Implementation"
        );
    }
}

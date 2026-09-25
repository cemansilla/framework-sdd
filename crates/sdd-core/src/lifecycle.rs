use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

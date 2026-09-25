use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceabilityGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>,
}

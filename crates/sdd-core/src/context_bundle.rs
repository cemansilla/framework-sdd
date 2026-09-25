use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextPriority {
    P0,
    P1,
    P2,
    P3,
}

impl ContextPriority {
    pub fn as_u8(&self) -> u8 {
        match self {
            ContextPriority::P0 => 0,
            ContextPriority::P1 => 1,
            ContextPriority::P2 => 2,
            ContextPriority::P3 => 3,
        }
    }
}

impl PartialOrd for ContextPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ContextPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_u8().cmp(&other.as_u8())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextSource {
    Task,
    Requirement,
    Decision,
    Architecture,
    Dependency,
    Code,
    Test,
    Documentation,
    SemanticRetrieval,
    ExplicitRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFragment {
    pub id: Uuid,
    pub content: String,
    pub source: ContextSource,
    pub source_id: String,
    pub priority: ContextPriority,
    pub reason: String,
    pub hash: String,
    pub estimated_tokens: usize,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

impl ContextFragment {
    pub fn new(
        content: String,
        source: ContextSource,
        source_id: String,
        priority: ContextPriority,
        reason: String,
    ) -> Self {
        let hash = Self::compute_hash(&content);
        let estimated_tokens = Self::estimate_tokens(&content);

        Self {
            id: Uuid::new_v4(),
            content,
            source,
            source_id,
            priority,
            reason,
            hash,
            estimated_tokens,
            metadata: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn estimate_tokens(content: &str) -> usize {
        let words = content.split_whitespace().count();
        (words as f64 * 1.3) as usize
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscardedElement {
    pub source_id: String,
    pub source: ContextSource,
    pub reason: String,
    pub estimated_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    pub total: usize,
    pub used: usize,
    pub reserved_p0: usize,
    pub reserved_p1: usize,
    pub reserved_p2: usize,
    pub reserved_p3: usize,
}

impl TokenBudget {
    pub fn new(total: usize) -> Self {
        Self {
            total,
            used: 0,
            reserved_p0: 0,
            reserved_p1: 0,
            reserved_p2: 0,
            reserved_p3: 0,
        }
    }

    pub fn remaining(&self) -> usize {
        self.total.saturating_sub(self.used)
    }

    pub fn utilization_percent(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.used as f64 / self.total as f64) * 100.0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBundle {
    pub id: Uuid,
    pub task_id: Uuid,
    pub fragments: Vec<ContextFragment>,
    pub discarded: Vec<DiscardedElement>,
    pub budget: TokenBudget,
    pub hash: String,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl ContextBundle {
    pub fn new(task_id: Uuid, budget: TokenBudget) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_id,
            fragments: Vec::new(),
            discarded: Vec::new(),
            budget,
            hash: String::new(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_fragment(&mut self, fragment: ContextFragment) -> Result<(), ContextError> {
        if fragment.estimated_tokens > self.budget.remaining() {
            self.discarded.push(DiscardedElement {
                source_id: fragment.source_id.clone(),
                source: fragment.source,
                reason: "exceeds token budget".to_string(),
                estimated_tokens: fragment.estimated_tokens,
            });
            return Err(ContextError::BudgetExceeded);
        }

        self.budget.used += fragment.estimated_tokens;
        match fragment.priority {
            ContextPriority::P0 => self.budget.reserved_p0 += fragment.estimated_tokens,
            ContextPriority::P1 => self.budget.reserved_p1 += fragment.estimated_tokens,
            ContextPriority::P2 => self.budget.reserved_p2 += fragment.estimated_tokens,
            ContextPriority::P3 => self.budget.reserved_p3 += fragment.estimated_tokens,
        }

        self.fragments.push(fragment);
        self.recompute_hash();
        Ok(())
    }

    pub fn discard(
        &mut self,
        source_id: String,
        source: ContextSource,
        reason: String,
        tokens: usize,
    ) {
        self.discarded.push(DiscardedElement {
            source_id,
            source,
            reason,
            estimated_tokens: tokens,
        });
    }

    pub fn get_by_priority(&self, priority: ContextPriority) -> Vec<&ContextFragment> {
        self.fragments
            .iter()
            .filter(|f| f.priority == priority)
            .collect()
    }

    pub fn get_by_source(&self, source: ContextSource) -> Vec<&ContextFragment> {
        self.fragments
            .iter()
            .filter(|f| f.source == source)
            .collect()
    }

    pub fn total_tokens(&self) -> usize {
        self.fragments.iter().map(|f| f.estimated_tokens).sum()
    }

    pub fn fragment_count(&self) -> usize {
        self.fragments.len()
    }

    pub fn discarded_count(&self) -> usize {
        self.discarded.len()
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        output.push_str("# Context Bundle\n\n");
        output.push_str(&format!("Task ID: {}\n", self.task_id));
        output.push_str(&format!("Bundle ID: {}\n", self.id));
        output.push_str(&format!("Hash: {}\n", self.hash));
        output.push_str(&format!(
            "Tokens: {} / {} ({:.1}%)\n\n",
            self.budget.used,
            self.budget.total,
            self.budget.utilization_percent()
        ));

        for priority in [
            ContextPriority::P0,
            ContextPriority::P1,
            ContextPriority::P2,
            ContextPriority::P3,
        ] {
            let fragments = self.get_by_priority(priority);
            if fragments.is_empty() {
                continue;
            }

            output.push_str(&format!("## {:?}\n\n", priority));
            for fragment in fragments {
                output.push_str(&format!(
                    "### {} ({:?})\n",
                    fragment.source_id, fragment.source
                ));
                output.push_str(&format!("*Reason: {}*\n\n", fragment.reason));
                output.push_str(&fragment.content);
                output.push_str("\n\n---\n\n");
            }
        }

        if !self.discarded.is_empty() {
            output.push_str("## Discarded Elements\n\n");
            for item in &self.discarded {
                output.push_str(&format!(
                    "- {} ({:?}): {} (~{} tokens)\n",
                    item.source_id, item.source, item.reason, item.estimated_tokens
                ));
            }
        }

        output
    }

    fn recompute_hash(&mut self) {
        let mut hasher = Sha256::new();
        for fragment in &self.fragments {
            hasher.update(fragment.hash.as_bytes());
        }
        self.hash = format!("{:x}", hasher.finalize());
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("token budget exceeded")]
    BudgetExceeded,
    #[error("task not found: {0}")]
    TaskNotFound(Uuid),
    #[error("artifact not found: {0}")]
    ArtifactNotFound(String),
    #[error("retrieval failed: {0}")]
    RetrievalFailed(String),
    #[error("cache error: {0}")]
    CacheError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_priority_ordering() {
        assert!(ContextPriority::P0 < ContextPriority::P1);
        assert!(ContextPriority::P1 < ContextPriority::P2);
        assert!(ContextPriority::P2 < ContextPriority::P3);
    }

    #[test]
    fn test_context_fragment_creation() {
        let fragment = ContextFragment::new(
            "Hello world".to_string(),
            ContextSource::Task,
            "task-001".to_string(),
            ContextPriority::P0,
            "Primary task context".to_string(),
        );

        assert_eq!(fragment.source, ContextSource::Task);
        assert_eq!(fragment.priority, ContextPriority::P0);
        assert!(!fragment.hash.is_empty());
        assert!(fragment.estimated_tokens > 0);
    }

    #[test]
    fn test_context_fragment_with_metadata() {
        let fragment = ContextFragment::new(
            "Content".to_string(),
            ContextSource::Code,
            "file.rs".to_string(),
            ContextPriority::P1,
            "Related code".to_string(),
        )
        .with_metadata("language", "rust");

        assert_eq!(fragment.metadata.get("language"), Some(&"rust".to_string()));
    }

    #[test]
    fn test_token_budget() {
        let mut budget = TokenBudget::new(1000);
        assert_eq!(budget.remaining(), 1000);
        assert_eq!(budget.utilization_percent(), 0.0);

        budget.used = 500;
        assert_eq!(budget.remaining(), 500);
        assert_eq!(budget.utilization_percent(), 50.0);
    }

    #[test]
    fn test_context_bundle_creation() {
        let task_id = Uuid::new_v4();
        let budget = TokenBudget::new(4000);
        let bundle = ContextBundle::new(task_id, budget);

        assert_eq!(bundle.task_id, task_id);
        assert_eq!(bundle.fragment_count(), 0);
        assert_eq!(bundle.discarded_count(), 0);
    }

    #[test]
    fn test_context_bundle_add_fragment() {
        let task_id = Uuid::new_v4();
        let budget = TokenBudget::new(4000);
        let mut bundle = ContextBundle::new(task_id, budget);

        let fragment = ContextFragment::new(
            "Task description".to_string(),
            ContextSource::Task,
            "task-001".to_string(),
            ContextPriority::P0,
            "Primary task".to_string(),
        );

        let result = bundle.add_fragment(fragment);
        assert!(result.is_ok());
        assert_eq!(bundle.fragment_count(), 1);
        assert!(!bundle.hash.is_empty());
    }

    #[test]
    fn test_context_bundle_budget_exceeded() {
        let task_id = Uuid::new_v4();
        let budget = TokenBudget::new(10);
        let mut bundle = ContextBundle::new(task_id, budget);

        let fragment = ContextFragment::new(
            "This is a very long content that will exceed the budget".to_string(),
            ContextSource::Task,
            "task-001".to_string(),
            ContextPriority::P0,
            "Primary task".to_string(),
        );

        let result = bundle.add_fragment(fragment);
        assert!(matches!(result, Err(ContextError::BudgetExceeded)));
        assert_eq!(bundle.fragment_count(), 0);
        assert_eq!(bundle.discarded_count(), 1);
    }

    #[test]
    fn test_context_bundle_get_by_priority() {
        let task_id = Uuid::new_v4();
        let budget = TokenBudget::new(10000);
        let mut bundle = ContextBundle::new(task_id, budget);

        let p0_fragment = ContextFragment::new(
            "P0 content".to_string(),
            ContextSource::Task,
            "task-001".to_string(),
            ContextPriority::P0,
            "Primary".to_string(),
        );

        let p1_fragment = ContextFragment::new(
            "P1 content".to_string(),
            ContextSource::Requirement,
            "req-001".to_string(),
            ContextPriority::P1,
            "Related requirement".to_string(),
        );

        bundle.add_fragment(p0_fragment).unwrap();
        bundle.add_fragment(p1_fragment).unwrap();

        let p0_items = bundle.get_by_priority(ContextPriority::P0);
        assert_eq!(p0_items.len(), 1);

        let p1_items = bundle.get_by_priority(ContextPriority::P1);
        assert_eq!(p1_items.len(), 1);
    }

    #[test]
    fn test_context_bundle_get_by_source() {
        let task_id = Uuid::new_v4();
        let budget = TokenBudget::new(10000);
        let mut bundle = ContextBundle::new(task_id, budget);

        let task_fragment = ContextFragment::new(
            "Task content".to_string(),
            ContextSource::Task,
            "task-001".to_string(),
            ContextPriority::P0,
            "Primary".to_string(),
        );

        let code_fragment = ContextFragment::new(
            "Code content".to_string(),
            ContextSource::Code,
            "file.rs".to_string(),
            ContextPriority::P1,
            "Related code".to_string(),
        );

        bundle.add_fragment(task_fragment).unwrap();
        bundle.add_fragment(code_fragment).unwrap();

        let code_items = bundle.get_by_source(ContextSource::Code);
        assert_eq!(code_items.len(), 1);
    }

    #[test]
    fn test_context_bundle_render() {
        let task_id = Uuid::new_v4();
        let budget = TokenBudget::new(4000);
        let mut bundle = ContextBundle::new(task_id, budget);

        let fragment = ContextFragment::new(
            "Task description".to_string(),
            ContextSource::Task,
            "task-001".to_string(),
            ContextPriority::P0,
            "Primary task".to_string(),
        );

        bundle.add_fragment(fragment).unwrap();

        let rendered = bundle.render();
        assert!(rendered.contains("Context Bundle"));
        assert!(rendered.contains("P0"));
        assert!(rendered.contains("task-001"));
    }

    #[test]
    fn test_token_estimation() {
        let tokens = ContextFragment::estimate_tokens("Hello world this is a test");
        assert!(tokens > 0);
    }

    #[test]
    fn test_hash_computation() {
        let hash1 = ContextFragment::compute_hash("content1");
        let hash2 = ContextFragment::compute_hash("content2");
        let hash3 = ContextFragment::compute_hash("content1");

        assert_ne!(hash1, hash2);
        assert_eq!(hash1, hash3);
    }
}

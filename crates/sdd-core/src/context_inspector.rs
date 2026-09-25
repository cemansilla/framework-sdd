use crate::context_bundle::{ContextBundle, ContextFragment, ContextPriority};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInspection {
    pub bundle_id: String,
    pub task_id: String,
    pub total_tokens: usize,
    pub budget_total: usize,
    pub budget_used: usize,
    pub budget_remaining: usize,
    pub utilization_percent: f64,
    pub fragment_count: usize,
    pub fragments_by_priority: HashMap<String, usize>,
    pub fragments_by_source: HashMap<String, usize>,
    pub discarded_count: usize,
    pub discarded_tokens: usize,
    pub preview: String,
}

pub struct ContextInspector;

impl ContextInspector {
    pub fn new() -> Self {
        Self
    }

    pub fn inspect(&self, bundle: &ContextBundle) -> ContextInspection {
        let mut fragments_by_priority: HashMap<String, usize> = HashMap::new();
        let mut fragments_by_source: HashMap<String, usize> = HashMap::new();

        for fragment in &bundle.fragments {
            let priority_key = format!("{:?}", fragment.priority);
            *fragments_by_priority.entry(priority_key).or_insert(0) += 1;

            let source_key = format!("{:?}", fragment.source);
            *fragments_by_source.entry(source_key).or_insert(0) += 1;
        }

        let discarded_tokens: usize = bundle.discarded.iter().map(|d| d.estimated_tokens).sum();

        ContextInspection {
            bundle_id: bundle.id.to_string(),
            task_id: bundle.task_id.to_string(),
            total_tokens: bundle.total_tokens(),
            budget_total: bundle.budget.total,
            budget_used: bundle.budget.used,
            budget_remaining: bundle.budget.remaining(),
            utilization_percent: bundle.budget.utilization_percent(),
            fragment_count: bundle.fragment_count(),
            fragments_by_priority,
            fragments_by_source,
            discarded_count: bundle.discarded_count(),
            discarded_tokens,
            preview: self.generate_preview(bundle),
        }
    }

    pub fn inspect_fragment(&self, fragment: &ContextFragment) -> FragmentInspection {
        FragmentInspection {
            id: fragment.id.to_string(),
            source_id: fragment.source_id.clone(),
            source: format!("{:?}", fragment.source),
            priority: format!("{:?}", fragment.priority),
            reason: fragment.reason.clone(),
            estimated_tokens: fragment.estimated_tokens,
            hash: fragment.hash.clone(),
            content_length: fragment.content.len(),
            metadata_keys: fragment.metadata.keys().cloned().collect(),
        }
    }

    pub fn generate_preview(&self, bundle: &ContextBundle) -> String {
        let mut preview = String::new();

        preview.push_str(&format!("Bundle ID: {}\n", bundle.id));
        preview.push_str(&format!("Task ID: {}\n", bundle.task_id));
        preview.push_str(&format!(
            "Tokens: {}/{} ({:.1}%)\n\n",
            bundle.budget.used,
            bundle.budget.total,
            bundle.budget.utilization_percent()
        ));

        preview.push_str(&format!("Fragments: {}\n", bundle.fragment_count()));
        preview.push_str(&format!("Discarded: {}\n\n", bundle.discarded_count()));

        for priority in [
            ContextPriority::P0,
            ContextPriority::P1,
            ContextPriority::P2,
            ContextPriority::P3,
        ] {
            let count = bundle.get_by_priority(priority).len();
            if count > 0 {
                preview.push_str(&format!("{:?}: {} fragments\n", priority, count));
            }
        }

        preview
    }

    pub fn render_full(&self, bundle: &ContextBundle) -> String {
        bundle.render()
    }
}

impl Default for ContextInspector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentInspection {
    pub id: String,
    pub source_id: String,
    pub source: String,
    pub priority: String,
    pub reason: String,
    pub estimated_tokens: usize,
    pub hash: String,
    pub content_length: usize,
    pub metadata_keys: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_bundle::{ContextSource, TokenBudget};
    use uuid::Uuid;

    #[test]
    fn test_context_inspector_creation() {
        let inspector = ContextInspector::new();
        let _ = inspector;
    }

    #[test]
    fn test_inspect_empty_bundle() {
        let inspector = ContextInspector::new();
        let task_id = Uuid::new_v4();
        let bundle = ContextBundle::new(task_id, TokenBudget::new(4000));

        let inspection = inspector.inspect(&bundle);
        assert_eq!(inspection.fragment_count, 0);
        assert_eq!(inspection.total_tokens, 0);
    }

    #[test]
    fn test_inspect_bundle_with_fragments() {
        let inspector = ContextInspector::new();
        let task_id = Uuid::new_v4();
        let mut bundle = ContextBundle::new(task_id, TokenBudget::new(4000));

        let fragment = ContextFragment::new(
            "Test content".to_string(),
            ContextSource::Task,
            "task-001".to_string(),
            ContextPriority::P0,
            "Test".to_string(),
        );
        bundle.add_fragment(fragment).unwrap();

        let inspection = inspector.inspect(&bundle);
        assert_eq!(inspection.fragment_count, 1);
        assert!(inspection.total_tokens > 0);
    }

    #[test]
    fn test_inspect_fragment() {
        let inspector = ContextInspector::new();
        let fragment = ContextFragment::new(
            "Test content".to_string(),
            ContextSource::Code,
            "file.rs".to_string(),
            ContextPriority::P1,
            "Related code".to_string(),
        );

        let inspection = inspector.inspect_fragment(&fragment);
        assert_eq!(inspection.source, "Code");
        assert_eq!(inspection.priority, "P1");
    }

    #[test]
    fn test_generate_preview() {
        let inspector = ContextInspector::new();
        let task_id = Uuid::new_v4();
        let bundle = ContextBundle::new(task_id, TokenBudget::new(4000));

        let preview = inspector.generate_preview(&bundle);
        assert!(preview.contains("Bundle ID"));
        assert!(preview.contains("Task ID"));
    }
}

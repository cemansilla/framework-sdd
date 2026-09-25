use crate::context_bundle::{ContextBundle, ContextFragment, ContextPriority, TokenBudget};
use crate::semantic_ranker::{RankedFragment, SemanticRanker};

pub struct ContextPrioritizer {
    ranker: SemanticRanker,
}

impl ContextPrioritizer {
    pub fn new() -> Self {
        Self {
            ranker: SemanticRanker::new(),
        }
    }

    pub fn prioritize(
        &self,
        fragments: Vec<ContextFragment>,
        budget: &mut TokenBudget,
        query: Option<&str>,
    ) -> Vec<ContextFragment> {
        let ranked = self.ranker.rank(fragments, query);
        let mut result = Vec::new();

        for priority in [
            ContextPriority::P0,
            ContextPriority::P1,
            ContextPriority::P2,
            ContextPriority::P3,
        ] {
            let priority_fragments: Vec<&RankedFragment> = ranked
                .iter()
                .filter(|rf| rf.fragment.priority == priority)
                .collect();

            for rf in priority_fragments {
                if rf.fragment.estimated_tokens <= budget.remaining() {
                    budget.used += rf.fragment.estimated_tokens;
                    result.push(rf.fragment.clone());
                }
            }
        }

        result
    }

    pub fn truncate_to_budget(
        &self,
        bundle: &mut ContextBundle,
        max_tokens: usize,
    ) -> Vec<ContextFragment> {
        let mut truncated = Vec::new();

        while bundle.total_tokens() > max_tokens && !bundle.fragments.is_empty() {
            let lowest_priority_idx = bundle
                .fragments
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.priority.cmp(&b.priority))
                .map(|(i, _)| i);

            if let Some(idx) = lowest_priority_idx {
                let removed = bundle.fragments.remove(idx);
                truncated.push(removed);
            }
        }

        truncated
    }
}

impl Default for ContextPrioritizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_bundle::ContextSource;
    use uuid::Uuid;

    fn create_fragment(priority: ContextPriority, tokens: usize) -> ContextFragment {
        let words = (tokens as f64 / 1.3) as usize;
        let content = "word ".repeat(words.max(1));
        ContextFragment::new(
            content,
            ContextSource::Task,
            format!("test-{}", Uuid::new_v4()),
            priority,
            "Test".to_string(),
        )
    }

    #[test]
    fn test_prioritize_respects_budget() {
        let prioritizer = ContextPrioritizer::new();
        let mut budget = TokenBudget::new(100);

        let fragments = vec![
            create_fragment(ContextPriority::P0, 30),
            create_fragment(ContextPriority::P1, 40),
            create_fragment(ContextPriority::P2, 50),
        ];

        let result = prioritizer.prioritize(fragments, &mut budget, None);
        assert!(budget.used <= 100);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_prioritize_p0_first() {
        let prioritizer = ContextPrioritizer::new();
        let mut budget = TokenBudget::new(50);

        let fragments = vec![
            create_fragment(ContextPriority::P2, 30),
            create_fragment(ContextPriority::P0, 30),
        ];

        let result = prioritizer.prioritize(fragments, &mut budget, None);
        assert_eq!(result[0].priority, ContextPriority::P0);
    }

    #[test]
    fn test_truncate_to_budget() {
        let prioritizer = ContextPrioritizer::new();
        let task_id = Uuid::new_v4();
        let mut bundle = ContextBundle::new(task_id, TokenBudget::new(10000));

        bundle
            .add_fragment(create_fragment(ContextPriority::P0, 30))
            .unwrap();
        bundle
            .add_fragment(create_fragment(ContextPriority::P1, 40))
            .unwrap();
        bundle
            .add_fragment(create_fragment(ContextPriority::P2, 50))
            .unwrap();

        let truncated = prioritizer.truncate_to_budget(&mut bundle, 70);
        assert!(bundle.total_tokens() <= 70);
        assert!(!truncated.is_empty());
    }
}

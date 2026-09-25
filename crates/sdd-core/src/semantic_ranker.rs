use crate::context_bundle::{ContextFragment, ContextPriority, ContextSource};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedFragment {
    pub fragment: ContextFragment,
    pub explicit_score: f64,
    pub semantic_score: f64,
    pub final_score: f64,
}

#[derive(Debug, Clone)]
pub struct SemanticRanker {
    explicit_weight: f64,
    semantic_weight: f64,
    recency_weight: f64,
}

impl SemanticRanker {
    pub fn new() -> Self {
        Self {
            explicit_weight: 0.6,
            semantic_weight: 0.3,
            recency_weight: 0.1,
        }
    }

    pub fn with_weights(mut self, explicit: f64, semantic: f64, recency: f64) -> Self {
        self.explicit_weight = explicit;
        self.semantic_weight = semantic;
        self.recency_weight = recency;
        self
    }

    pub fn rank(
        &self,
        fragments: Vec<ContextFragment>,
        query: Option<&str>,
    ) -> Vec<RankedFragment> {
        let mut ranked: Vec<RankedFragment> = fragments
            .into_iter()
            .map(|fragment| {
                let explicit_score = self.calculate_explicit_score(&fragment);
                let semantic_score = self.calculate_semantic_score(&fragment, query);
                let final_score = self.combine_scores(explicit_score, semantic_score);

                RankedFragment {
                    fragment,
                    explicit_score,
                    semantic_score,
                    final_score,
                }
            })
            .collect();

        ranked.sort_by(|a, b| b.final_score.partial_cmp(&a.final_score).unwrap());
        ranked
    }

    fn calculate_explicit_score(&self, fragment: &ContextFragment) -> f64 {
        let priority_score = match fragment.priority {
            ContextPriority::P0 => 1.0,
            ContextPriority::P1 => 0.7,
            ContextPriority::P2 => 0.4,
            ContextPriority::P3 => 0.2,
        };

        let source_score = match fragment.source {
            ContextSource::Task => 1.0,
            ContextSource::Requirement => 0.9,
            ContextSource::Architecture => 0.85,
            ContextSource::Decision => 0.8,
            ContextSource::Dependency => 0.75,
            ContextSource::Code => 0.7,
            ContextSource::Test => 0.6,
            ContextSource::Documentation => 0.5,
            ContextSource::SemanticRetrieval => 0.4,
            ContextSource::ExplicitRequest => 0.95,
        };

        (priority_score + source_score) / 2.0
    }

    fn calculate_semantic_score(&self, fragment: &ContextFragment, query: Option<&str>) -> f64 {
        let Some(query) = query else {
            return 0.5;
        };

        let query_lower = query.to_lowercase();
        let content_lower = fragment.content.to_lowercase();

        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let mut matches = 0;

        for word in &query_words {
            if content_lower.contains(word) {
                matches += 1;
            }
        }

        if query_words.is_empty() {
            0.5
        } else {
            matches as f64 / query_words.len() as f64
        }
    }

    fn combine_scores(&self, explicit: f64, semantic: f64) -> f64 {
        explicit * self.explicit_weight + semantic * self.semantic_weight
    }
}

impl Default for SemanticRanker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_fragment(priority: ContextPriority, source: ContextSource) -> ContextFragment {
        ContextFragment::new(
            "Test content with some keywords".to_string(),
            source,
            "test-id".to_string(),
            priority,
            "Test reason".to_string(),
        )
    }

    #[test]
    fn test_semantic_ranker_creation() {
        let ranker = SemanticRanker::new();
        assert!((ranker.explicit_weight - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_rank_fragments() {
        let ranker = SemanticRanker::new();

        let fragments = vec![
            create_test_fragment(ContextPriority::P0, ContextSource::Task),
            create_test_fragment(ContextPriority::P2, ContextSource::Documentation),
            create_test_fragment(ContextPriority::P1, ContextSource::Requirement),
        ];

        let ranked = ranker.rank(fragments, Some("test"));
        assert_eq!(ranked.len(), 3);
        assert!(ranked[0].final_score >= ranked[1].final_score);
    }

    #[test]
    fn test_explicit_score_priority() {
        let ranker = SemanticRanker::new();

        let p0 = create_test_fragment(ContextPriority::P0, ContextSource::Task);
        let p2 = create_test_fragment(ContextPriority::P2, ContextSource::Task);

        let score_p0 = ranker.calculate_explicit_score(&p0);
        let score_p2 = ranker.calculate_explicit_score(&p2);

        assert!(score_p0 > score_p2);
    }

    #[test]
    fn test_semantic_score_with_query() {
        let ranker = SemanticRanker::new();

        let fragment = ContextFragment::new(
            "Rust programming language is fast".to_string(),
            ContextSource::Code,
            "file.rs".to_string(),
            ContextPriority::P1,
            "Code".to_string(),
        );

        let score = ranker.calculate_semantic_score(&fragment, Some("rust programming"));
        assert!(score > 0.0);
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalMetrics {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub task_id: Uuid,
    pub retrieval_type: RetrievalType,
    pub query: String,
    pub results_count: usize,
    pub relevant_results: usize,
    pub duration_ms: u64,
    pub precision: Option<f64>,
    pub recall: Option<f64>,
    pub f1_score: Option<f64>,
    pub tokens_retrieved: usize,
    pub tokens_used: usize,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RetrievalType {
    Explicit,
    Lexical,
    Semantic,
    Hybrid,
    Graph,
    Ast,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FindingSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationFinding {
    pub id: Uuid,
    pub severity: FindingSeverity,
    pub category: String,
    pub message: String,
    pub location: Option<String>,
    pub recommendation: Option<String>,
    pub resolved: bool,
}

impl RetrievalMetrics {
    pub fn new(
        task_id: Uuid,
        retrieval_type: RetrievalType,
        query: impl Into<String>,
        results_count: usize,
        duration_ms: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            task_id,
            retrieval_type,
            query: query.into(),
            results_count,
            relevant_results: 0,
            duration_ms,
            precision: None,
            recall: None,
            f1_score: None,
            tokens_retrieved: 0,
            tokens_used: 0,
            metadata: HashMap::new(),
        }
    }

    pub fn with_relevant_results(mut self, relevant: usize) -> Self {
        self.relevant_results = relevant;
        self.calculate_scores();
        self
    }

    pub fn with_tokens(mut self, retrieved: usize, used: usize) -> Self {
        self.tokens_retrieved = retrieved;
        self.tokens_used = used;
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    fn calculate_scores(&mut self) {
        if self.results_count > 0 {
            self.precision = Some(self.relevant_results as f64 / self.results_count as f64);
        }

        // Recall requiere conocer el total de documentos relevantes
        // Por ahora lo dejamos como None, se puede calcular externamente
    }

    pub fn calculate_f1(&mut self, recall: f64) {
        if let Some(precision) = self.precision {
            if precision + recall > 0.0 {
                self.f1_score = Some(2.0 * (precision * recall) / (precision + recall));
            }
        }
        self.recall = Some(recall);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalMetricsSummary {
    pub total_retrievals: usize,
    pub total_duration_ms: u64,
    pub average_duration_ms: f64,
    pub total_results: usize,
    pub average_results_per_query: f64,
    pub average_precision: Option<f64>,
    pub average_recall: Option<f64>,
    pub average_f1_score: Option<f64>,
    pub total_tokens_retrieved: usize,
    pub total_tokens_used: usize,
    pub retrieval_type_counts: HashMap<String, usize>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RetrievalMetricsCollector {
    metrics: Vec<RetrievalMetrics>,
}

impl RetrievalMetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
        }
    }

    pub fn record(&mut self, metrics: RetrievalMetrics) {
        self.metrics.push(metrics);
    }

    pub fn get_all_metrics(&self) -> &[RetrievalMetrics] {
        &self.metrics
    }

    pub fn get_metrics_by_task(&self, task_id: Uuid) -> Vec<&RetrievalMetrics> {
        self.metrics
            .iter()
            .filter(|m| m.task_id == task_id)
            .collect()
    }

    pub fn get_metrics_by_type(&self, retrieval_type: &RetrievalType) -> Vec<&RetrievalMetrics> {
        self.metrics
            .iter()
            .filter(|m| &m.retrieval_type == retrieval_type)
            .collect()
    }

    pub fn get_summary(&self) -> RetrievalMetricsSummary {
        self.get_summary_in_period(None, None)
    }

    pub fn get_summary_in_period(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> RetrievalMetricsSummary {
        let filtered_metrics: Vec<&RetrievalMetrics> = self
            .metrics
            .iter()
            .filter(|m| {
                let after_start = start.map(|s| m.timestamp >= s).unwrap_or(true);
                let before_end = end.map(|e| m.timestamp <= e).unwrap_or(true);
                after_start && before_end
            })
            .collect();

        let total_retrievals = filtered_metrics.len();
        let total_duration_ms = filtered_metrics.iter().map(|m| m.duration_ms).sum();
        let average_duration_ms = if total_retrievals > 0 {
            total_duration_ms as f64 / total_retrievals as f64
        } else {
            0.0
        };

        let total_results = filtered_metrics.iter().map(|m| m.results_count).sum();
        let average_results_per_query = if total_retrievals > 0 {
            total_results as f64 / total_retrievals as f64
        } else {
            0.0
        };

        let precisions: Vec<f64> = filtered_metrics
            .iter()
            .filter_map(|m| m.precision)
            .collect();
        let average_precision = if !precisions.is_empty() {
            Some(precisions.iter().sum::<f64>() / precisions.len() as f64)
        } else {
            None
        };

        let recalls: Vec<f64> = filtered_metrics.iter().filter_map(|m| m.recall).collect();
        let average_recall = if !recalls.is_empty() {
            Some(recalls.iter().sum::<f64>() / recalls.len() as f64)
        } else {
            None
        };

        let f1_scores: Vec<f64> = filtered_metrics.iter().filter_map(|m| m.f1_score).collect();
        let average_f1_score = if !f1_scores.is_empty() {
            Some(f1_scores.iter().sum::<f64>() / f1_scores.len() as f64)
        } else {
            None
        };

        let total_tokens_retrieved = filtered_metrics.iter().map(|m| m.tokens_retrieved).sum();
        let total_tokens_used = filtered_metrics.iter().map(|m| m.tokens_used).sum();

        let mut retrieval_type_counts = HashMap::new();
        for metrics in &filtered_metrics {
            let type_name = match &metrics.retrieval_type {
                RetrievalType::Explicit => "explicit",
                RetrievalType::Lexical => "lexical",
                RetrievalType::Semantic => "semantic",
                RetrievalType::Hybrid => "hybrid",
                RetrievalType::Graph => "graph",
                RetrievalType::Ast => "ast",
            };
            *retrieval_type_counts
                .entry(type_name.to_string())
                .or_insert(0) += 1;
        }

        let period_start = start.unwrap_or_else(|| {
            filtered_metrics
                .iter()
                .map(|m| m.timestamp)
                .min()
                .unwrap_or_else(Utc::now)
        });

        let period_end = end.unwrap_or_else(|| {
            filtered_metrics
                .iter()
                .map(|m| m.timestamp)
                .max()
                .unwrap_or_else(Utc::now)
        });

        RetrievalMetricsSummary {
            total_retrievals,
            total_duration_ms,
            average_duration_ms,
            total_results,
            average_results_per_query,
            average_precision,
            average_recall,
            average_f1_score,
            total_tokens_retrieved,
            total_tokens_used,
            retrieval_type_counts,
            period_start,
            period_end,
        }
    }

    pub fn total_retrievals(&self) -> usize {
        self.metrics.len()
    }

    pub fn average_duration_ms(&self) -> f64 {
        if self.metrics.is_empty() {
            return 0.0;
        }
        let total: u64 = self.metrics.iter().map(|m| m.duration_ms).sum();
        total as f64 / self.metrics.len() as f64
    }

    pub fn clear(&mut self) {
        self.metrics.clear();
    }
}

impl Default for RetrievalMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retrieval_metrics_creation() {
        let task_id = Uuid::new_v4();
        let metrics =
            RetrievalMetrics::new(task_id, RetrievalType::Semantic, "test query", 10, 150);

        assert_eq!(metrics.task_id, task_id);
        assert_eq!(metrics.retrieval_type, RetrievalType::Semantic);
        assert_eq!(metrics.query, "test query");
        assert_eq!(metrics.results_count, 10);
        assert_eq!(metrics.duration_ms, 150);
    }

    #[test]
    fn test_retrieval_metrics_with_relevant_results() {
        let task_id = Uuid::new_v4();
        let metrics = RetrievalMetrics::new(task_id, RetrievalType::Lexical, "test query", 10, 100)
            .with_relevant_results(8);

        assert_eq!(metrics.relevant_results, 8);
        assert_eq!(metrics.precision, Some(0.8));
    }

    #[test]
    fn test_retrieval_metrics_with_tokens() {
        let task_id = Uuid::new_v4();
        let metrics = RetrievalMetrics::new(task_id, RetrievalType::Hybrid, "test query", 10, 100)
            .with_tokens(5000, 2000);

        assert_eq!(metrics.tokens_retrieved, 5000);
        assert_eq!(metrics.tokens_used, 2000);
    }

    #[test]
    fn test_retrieval_metrics_calculate_f1() {
        let task_id = Uuid::new_v4();
        let mut metrics =
            RetrievalMetrics::new(task_id, RetrievalType::Semantic, "test query", 10, 100)
                .with_relevant_results(8);

        metrics.calculate_f1(0.9);

        assert_eq!(metrics.recall, Some(0.9));
        assert!(metrics.f1_score.is_some());
    }

    #[test]
    fn test_retrieval_metrics_collector() {
        let mut collector = RetrievalMetricsCollector::new();
        let task_id = Uuid::new_v4();

        let metrics = RetrievalMetrics::new(task_id, RetrievalType::Explicit, "test query", 5, 50);

        collector.record(metrics);
        assert_eq!(collector.total_retrievals(), 1);
    }

    #[test]
    fn test_retrieval_metrics_collector_queries() {
        let mut collector = RetrievalMetricsCollector::new();
        let task_id = Uuid::new_v4();

        let metrics1 = RetrievalMetrics::new(task_id, RetrievalType::Lexical, "query 1", 10, 100);

        let metrics2 = RetrievalMetrics::new(task_id, RetrievalType::Semantic, "query 2", 8, 150);

        collector.record(metrics1);
        collector.record(metrics2);

        assert_eq!(collector.get_metrics_by_task(task_id).len(), 2);
        assert_eq!(
            collector.get_metrics_by_type(&RetrievalType::Lexical).len(),
            1
        );
    }

    #[test]
    fn test_retrieval_metrics_summary() {
        let mut collector = RetrievalMetricsCollector::new();
        let task_id = Uuid::new_v4();

        for i in 0..3 {
            let metrics = RetrievalMetrics::new(
                task_id,
                RetrievalType::Semantic,
                format!("query {}", i),
                10,
                100,
            )
            .with_relevant_results(8);
            collector.record(metrics);
        }

        let summary = collector.get_summary();
        assert_eq!(summary.total_retrievals, 3);
        assert_eq!(summary.total_results, 30);
        assert_eq!(summary.average_results_per_query, 10.0);
        assert!(summary.average_precision.is_some());
    }

    #[test]
    fn test_retrieval_metrics_average_duration() {
        let mut collector = RetrievalMetricsCollector::new();
        let task_id = Uuid::new_v4();

        let metrics1 = RetrievalMetrics::new(task_id, RetrievalType::Explicit, "q1", 5, 100);
        let metrics2 = RetrievalMetrics::new(task_id, RetrievalType::Explicit, "q2", 5, 200);

        collector.record(metrics1);
        collector.record(metrics2);

        assert_eq!(collector.average_duration_ms(), 150.0);
    }
}

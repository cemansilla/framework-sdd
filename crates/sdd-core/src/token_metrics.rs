use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetrics {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub agent_id: Uuid,
    pub task_id: Option<Uuid>,
    pub model: String,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub total_tokens: usize,
    pub cost_usd: Option<f64>,
    pub operation: TokenOperation,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TokenOperation {
    ContextGeneration,
    TaskExecution,
    Retrieval,
    Validation,
    Other(String),
}

impl TokenMetrics {
    pub fn new(
        agent_id: Uuid,
        model: impl Into<String>,
        input_tokens: usize,
        output_tokens: usize,
        operation: TokenOperation,
    ) -> Self {
        let total_tokens = input_tokens + output_tokens;
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            agent_id,
            task_id: None,
            model: model.into(),
            input_tokens,
            output_tokens,
            total_tokens,
            cost_usd: None,
            operation,
            metadata: HashMap::new(),
        }
    }

    pub fn with_task_id(mut self, task_id: Uuid) -> Self {
        self.task_id = Some(task_id);
        self
    }

    pub fn with_cost(mut self, cost_usd: f64) -> Self {
        self.cost_usd = Some(cost_usd);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetricsSummary {
    pub total_input_tokens: usize,
    pub total_output_tokens: usize,
    pub total_tokens: usize,
    pub total_cost_usd: f64,
    pub operation_counts: HashMap<String, usize>,
    pub model_usage: HashMap<String, usize>,
    pub agent_usage: HashMap<Uuid, usize>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TokenMetricsCollector {
    metrics: Vec<TokenMetrics>,
    pricing: HashMap<String, TokenPricing>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPricing {
    pub model: String,
    pub input_cost_per_1k: f64,
    pub output_cost_per_1k: f64,
}

impl TokenPricing {
    pub fn new(model: impl Into<String>, input_cost_per_1k: f64, output_cost_per_1k: f64) -> Self {
        Self {
            model: model.into(),
            input_cost_per_1k,
            output_cost_per_1k,
        }
    }

    pub fn calculate_cost(&self, input_tokens: usize, output_tokens: usize) -> f64 {
        let input_cost = (input_tokens as f64 / 1000.0) * self.input_cost_per_1k;
        let output_cost = (output_tokens as f64 / 1000.0) * self.output_cost_per_1k;
        input_cost + output_cost
    }
}

impl TokenMetricsCollector {
    pub fn new() -> Self {
        let mut pricing = HashMap::new();

        // Precios comunes de modelos (ejemplo)
        pricing.insert("gpt-4".to_string(), TokenPricing::new("gpt-4", 0.03, 0.06));
        pricing.insert(
            "gpt-3.5-turbo".to_string(),
            TokenPricing::new("gpt-3.5-turbo", 0.0015, 0.002),
        );
        pricing.insert(
            "claude-3-opus".to_string(),
            TokenPricing::new("claude-3-opus", 0.015, 0.075),
        );
        pricing.insert(
            "claude-3-sonnet".to_string(),
            TokenPricing::new("claude-3-sonnet", 0.003, 0.015),
        );

        Self {
            metrics: Vec::new(),
            pricing,
        }
    }

    pub fn record(&mut self, mut metrics: TokenMetrics) {
        // Calcular costo si no está definido y tenemos pricing
        if metrics.cost_usd.is_none() {
            if let Some(pricing) = self.pricing.get(&metrics.model) {
                let cost = pricing.calculate_cost(metrics.input_tokens, metrics.output_tokens);
                metrics.cost_usd = Some(cost);
            }
        }

        self.metrics.push(metrics);
    }

    pub fn add_pricing(&mut self, pricing: TokenPricing) {
        self.pricing.insert(pricing.model.clone(), pricing);
    }

    pub fn get_all_metrics(&self) -> &[TokenMetrics] {
        &self.metrics
    }

    pub fn get_metrics_by_agent(&self, agent_id: Uuid) -> Vec<&TokenMetrics> {
        self.metrics
            .iter()
            .filter(|m| m.agent_id == agent_id)
            .collect()
    }

    pub fn get_metrics_by_task(&self, task_id: Uuid) -> Vec<&TokenMetrics> {
        self.metrics
            .iter()
            .filter(|m| m.task_id == Some(task_id))
            .collect()
    }

    pub fn get_metrics_by_operation(&self, operation: &TokenOperation) -> Vec<&TokenMetrics> {
        self.metrics
            .iter()
            .filter(|m| &m.operation == operation)
            .collect()
    }

    pub fn get_metrics_by_model(&self, model: &str) -> Vec<&TokenMetrics> {
        self.metrics.iter().filter(|m| m.model == model).collect()
    }

    pub fn get_summary(&self) -> TokenMetricsSummary {
        self.get_summary_in_period(None, None)
    }

    pub fn get_summary_in_period(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> TokenMetricsSummary {
        let filtered_metrics: Vec<&TokenMetrics> = self
            .metrics
            .iter()
            .filter(|m| {
                let after_start = start.map(|s| m.timestamp >= s).unwrap_or(true);
                let before_end = end.map(|e| m.timestamp <= e).unwrap_or(true);
                after_start && before_end
            })
            .collect();

        let total_input_tokens = filtered_metrics.iter().map(|m| m.input_tokens).sum();
        let total_output_tokens = filtered_metrics.iter().map(|m| m.output_tokens).sum();
        let total_tokens = total_input_tokens + total_output_tokens;
        let total_cost_usd = filtered_metrics.iter().filter_map(|m| m.cost_usd).sum();

        let mut operation_counts = HashMap::new();
        let mut model_usage = HashMap::new();
        let mut agent_usage = HashMap::new();

        for metrics in &filtered_metrics {
            let op_name = match &metrics.operation {
                TokenOperation::ContextGeneration => "context_generation",
                TokenOperation::TaskExecution => "task_execution",
                TokenOperation::Retrieval => "retrieval",
                TokenOperation::Validation => "validation",
                TokenOperation::Other(name) => name.as_str(),
            };
            *operation_counts.entry(op_name.to_string()).or_insert(0) += 1;
            *model_usage.entry(metrics.model.clone()).or_insert(0) += 1;
            *agent_usage.entry(metrics.agent_id).or_insert(0) += 1;
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

        TokenMetricsSummary {
            total_input_tokens,
            total_output_tokens,
            total_tokens,
            total_cost_usd,
            operation_counts,
            model_usage,
            agent_usage,
            period_start,
            period_end,
        }
    }

    pub fn total_tokens(&self) -> usize {
        self.metrics.iter().map(|m| m.total_tokens).sum()
    }

    pub fn total_cost(&self) -> f64 {
        self.metrics.iter().filter_map(|m| m.cost_usd).sum()
    }

    pub fn clear(&mut self) {
        self.metrics.clear();
    }
}

impl Default for TokenMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_metrics_creation() {
        let agent_id = Uuid::new_v4();
        let metrics =
            TokenMetrics::new(agent_id, "gpt-4", 1000, 500, TokenOperation::TaskExecution);

        assert_eq!(metrics.agent_id, agent_id);
        assert_eq!(metrics.model, "gpt-4");
        assert_eq!(metrics.input_tokens, 1000);
        assert_eq!(metrics.output_tokens, 500);
        assert_eq!(metrics.total_tokens, 1500);
    }

    #[test]
    fn test_token_pricing_calculation() {
        let pricing = TokenPricing::new("gpt-4", 0.03, 0.06);
        let cost = pricing.calculate_cost(1000, 500);

        // 1000 input tokens * 0.03/1000 + 500 output tokens * 0.06/1000
        assert!((cost - 0.06).abs() < 0.001);
    }

    #[test]
    fn test_token_metrics_collector() {
        let mut collector = TokenMetricsCollector::new();
        let agent_id = Uuid::new_v4();

        let metrics =
            TokenMetrics::new(agent_id, "gpt-4", 1000, 500, TokenOperation::TaskExecution);

        collector.record(metrics);
        assert_eq!(collector.get_all_metrics().len(), 1);
        assert_eq!(collector.total_tokens(), 1500);
    }

    #[test]
    fn test_token_metrics_collector_queries() {
        let mut collector = TokenMetricsCollector::new();
        let agent_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();

        let metrics1 =
            TokenMetrics::new(agent_id, "gpt-4", 1000, 500, TokenOperation::TaskExecution)
                .with_task_id(task_id);

        let metrics2 = TokenMetrics::new(
            agent_id,
            "gpt-3.5-turbo",
            500,
            250,
            TokenOperation::ContextGeneration,
        );

        collector.record(metrics1);
        collector.record(metrics2);

        assert_eq!(collector.get_metrics_by_agent(agent_id).len(), 2);
        assert_eq!(collector.get_metrics_by_task(task_id).len(), 1);
        assert_eq!(
            collector
                .get_metrics_by_operation(&TokenOperation::TaskExecution)
                .len(),
            1
        );
    }

    #[test]
    fn test_token_metrics_summary() {
        let mut collector = TokenMetricsCollector::new();
        let agent_id = Uuid::new_v4();

        for _ in 0..3 {
            let metrics =
                TokenMetrics::new(agent_id, "gpt-4", 1000, 500, TokenOperation::TaskExecution);
            collector.record(metrics);
        }

        let summary = collector.get_summary();
        assert_eq!(summary.total_input_tokens, 3000);
        assert_eq!(summary.total_output_tokens, 1500);
        assert_eq!(summary.total_tokens, 4500);
        assert!(summary.total_cost_usd > 0.0);
    }

    #[test]
    fn test_token_metrics_with_cost() {
        let agent_id = Uuid::new_v4();
        let metrics =
            TokenMetrics::new(agent_id, "gpt-4", 1000, 500, TokenOperation::TaskExecution)
                .with_cost(0.05);

        assert_eq!(metrics.cost_usd, Some(0.05));
    }

    #[test]
    fn test_token_metrics_with_metadata() {
        let agent_id = Uuid::new_v4();
        let metrics =
            TokenMetrics::new(agent_id, "gpt-4", 1000, 500, TokenOperation::TaskExecution)
                .with_metadata("project".to_string(), "sdd-framework".to_string());

        assert_eq!(
            metrics.metadata.get("project"),
            Some(&"sdd-framework".to_string())
        );
    }
}

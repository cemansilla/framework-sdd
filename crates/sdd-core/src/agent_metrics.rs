use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub agent_name: String,
    pub timestamp: DateTime<Utc>,
    pub tasks_completed: usize,
    pub tasks_failed: usize,
    pub total_tokens_used: usize,
    pub average_task_duration_ms: Option<u64>,
    pub success_rate: f64,
    pub capabilities_used: Vec<String>,
    pub errors: Vec<AgentError>,
    pub performance_score: Option<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentError {
    pub timestamp: DateTime<Utc>,
    pub error_type: String,
    pub message: String,
    pub task_id: Option<Uuid>,
    pub resolved: bool,
}

impl AgentMetrics {
    pub fn new(agent_id: Uuid, agent_name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_id,
            agent_name: agent_name.into(),
            timestamp: Utc::now(),
            tasks_completed: 0,
            tasks_failed: 0,
            total_tokens_used: 0,
            average_task_duration_ms: None,
            success_rate: 0.0,
            capabilities_used: Vec::new(),
            errors: Vec::new(),
            performance_score: None,
            metadata: HashMap::new(),
        }
    }

    pub fn record_task_completion(&mut self, duration_ms: u64, tokens_used: usize) {
        self.tasks_completed += 1;
        self.total_tokens_used += tokens_used;
        self.update_average_duration(duration_ms);
        self.update_success_rate();
    }

    pub fn record_task_failure(&mut self, tokens_used: usize) {
        self.tasks_failed += 1;
        self.total_tokens_used += tokens_used;
        self.update_success_rate();
    }

    pub fn record_error(&mut self, error_type: String, message: String, task_id: Option<Uuid>) {
        self.errors.push(AgentError {
            timestamp: Utc::now(),
            error_type,
            message,
            task_id,
            resolved: false,
        });
    }

    pub fn resolve_error(&mut self, error_index: usize) -> bool {
        if let Some(error) = self.errors.get_mut(error_index) {
            error.resolved = true;
            true
        } else {
            false
        }
    }

    pub fn add_capability_used(&mut self, capability: String) {
        if !self.capabilities_used.contains(&capability) {
            self.capabilities_used.push(capability);
        }
    }

    pub fn calculate_performance_score(&mut self) {
        let total_tasks = self.tasks_completed + self.tasks_failed;
        if total_tasks == 0 {
            self.performance_score = Some(0.0);
            return;
        }

        let success_weight = 0.6;
        let efficiency_weight = 0.3;
        let error_weight = 0.1;

        let success_score = self.success_rate * success_weight;

        let avg_tokens_per_task = if total_tasks > 0 {
            self.total_tokens_used as f64 / total_tasks as f64
        } else {
            0.0
        };
        let efficiency_score = if avg_tokens_per_task > 0.0 {
            (10000.0 / avg_tokens_per_task).min(1.0) * efficiency_weight
        } else {
            0.0
        };

        let error_score = if !self.errors.is_empty() {
            let unresolved = self.errors.iter().filter(|e| !e.resolved).count();
            (1.0 - (unresolved as f64 / self.errors.len() as f64)) * error_weight
        } else {
            error_weight
        };

        self.performance_score = Some(success_score + efficiency_score + error_score);
    }

    fn update_average_duration(&mut self, new_duration_ms: u64) {
        let total_tasks = self.tasks_completed;
        if let Some(current_avg) = self.average_task_duration_ms {
            let new_avg = ((current_avg as f64 * (total_tasks - 1) as f64)
                + new_duration_ms as f64)
                / total_tasks as f64;
            self.average_task_duration_ms = Some(new_avg as u64);
        } else {
            self.average_task_duration_ms = Some(new_duration_ms);
        }
    }

    fn update_success_rate(&mut self) {
        let total_tasks = self.tasks_completed + self.tasks_failed;
        if total_tasks > 0 {
            self.success_rate = self.tasks_completed as f64 / total_tasks as f64;
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetricsSummary {
    pub total_agents: usize,
    pub total_tasks_completed: usize,
    pub total_tasks_failed: usize,
    pub overall_success_rate: f64,
    pub total_tokens_used: usize,
    pub average_performance_score: Option<f64>,
    pub most_used_capabilities: Vec<(String, usize)>,
    pub agent_details: Vec<AgentMetrics>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AgentMetricsCollector {
    metrics: HashMap<Uuid, AgentMetrics>,
}

impl AgentMetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    pub fn register_agent(&mut self, agent_id: Uuid, agent_name: impl Into<String>) {
        self.metrics
            .entry(agent_id)
            .or_insert_with(|| AgentMetrics::new(agent_id, agent_name));
    }

    pub fn get_metrics(&self, agent_id: Uuid) -> Option<&AgentMetrics> {
        self.metrics.get(&agent_id)
    }

    pub fn get_metrics_mut(&mut self, agent_id: Uuid) -> Option<&mut AgentMetrics> {
        self.metrics.get_mut(&agent_id)
    }

    pub fn get_all_metrics(&self) -> Vec<&AgentMetrics> {
        self.metrics.values().collect()
    }

    pub fn record_task_completion(
        &mut self,
        agent_id: Uuid,
        duration_ms: u64,
        tokens_used: usize,
    ) -> bool {
        if let Some(metrics) = self.metrics.get_mut(&agent_id) {
            metrics.record_task_completion(duration_ms, tokens_used);
            true
        } else {
            false
        }
    }

    pub fn record_task_failure(&mut self, agent_id: Uuid, tokens_used: usize) -> bool {
        if let Some(metrics) = self.metrics.get_mut(&agent_id) {
            metrics.record_task_failure(tokens_used);
            true
        } else {
            false
        }
    }

    pub fn record_error(
        &mut self,
        agent_id: Uuid,
        error_type: String,
        message: String,
        task_id: Option<Uuid>,
    ) -> bool {
        if let Some(metrics) = self.metrics.get_mut(&agent_id) {
            metrics.record_error(error_type, message, task_id);
            true
        } else {
            false
        }
    }

    pub fn get_summary(&self) -> AgentMetricsSummary {
        self.get_summary_in_period(None, None)
    }

    pub fn get_summary_in_period(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> AgentMetricsSummary {
        let all_metrics = self.get_all_metrics();

        let total_agents = all_metrics.len();
        let total_tasks_completed = all_metrics.iter().map(|m| m.tasks_completed).sum();
        let total_tasks_failed = all_metrics.iter().map(|m| m.tasks_failed).sum();
        let total_tokens_used = all_metrics.iter().map(|m| m.total_tokens_used).sum();

        let overall_success_rate = if total_tasks_completed + total_tasks_failed > 0 {
            total_tasks_completed as f64 / (total_tasks_completed + total_tasks_failed) as f64
        } else {
            0.0
        };

        let performance_scores: Vec<f64> = all_metrics
            .iter()
            .filter_map(|m| m.performance_score)
            .collect();
        let average_performance_score = if !performance_scores.is_empty() {
            Some(performance_scores.iter().sum::<f64>() / performance_scores.len() as f64)
        } else {
            None
        };

        let mut capability_counts: HashMap<String, usize> = HashMap::new();
        for metrics in &all_metrics {
            for capability in &metrics.capabilities_used {
                *capability_counts.entry(capability.clone()).or_insert(0) += 1;
            }
        }

        let mut most_used_capabilities: Vec<(String, usize)> =
            capability_counts.into_iter().collect();
        most_used_capabilities.sort_by_key(|a| std::cmp::Reverse(a.1));
        most_used_capabilities.truncate(10);

        let period_start = start.unwrap_or_else(Utc::now);
        let period_end = end.unwrap_or_else(Utc::now);

        AgentMetricsSummary {
            total_agents,
            total_tasks_completed,
            total_tasks_failed,
            overall_success_rate,
            total_tokens_used,
            average_performance_score,
            most_used_capabilities,
            agent_details: all_metrics.into_iter().cloned().collect(),
            period_start,
            period_end,
        }
    }

    pub fn total_agents(&self) -> usize {
        self.metrics.len()
    }

    pub fn total_tasks_completed(&self) -> usize {
        self.metrics.values().map(|m| m.tasks_completed).sum()
    }

    pub fn total_tasks_failed(&self) -> usize {
        self.metrics.values().map(|m| m.tasks_failed).sum()
    }

    pub fn overall_success_rate(&self) -> f64 {
        let total = self.total_tasks_completed() + self.total_tasks_failed();
        if total == 0 {
            return 0.0;
        }
        self.total_tasks_completed() as f64 / total as f64
    }

    pub fn remove_agent(&mut self, agent_id: Uuid) -> Option<AgentMetrics> {
        self.metrics.remove(&agent_id)
    }

    pub fn clear(&mut self) {
        self.metrics.clear();
    }
}

impl Default for AgentMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_metrics_creation() {
        let agent_id = Uuid::new_v4();
        let metrics = AgentMetrics::new(agent_id, "test-agent");

        assert_eq!(metrics.agent_id, agent_id);
        assert_eq!(metrics.agent_name, "test-agent");
        assert_eq!(metrics.tasks_completed, 0);
        assert_eq!(metrics.tasks_failed, 0);
        assert_eq!(metrics.success_rate, 0.0);
    }

    #[test]
    fn test_agent_metrics_task_completion() {
        let agent_id = Uuid::new_v4();
        let mut metrics = AgentMetrics::new(agent_id, "test-agent");

        metrics.record_task_completion(1000, 500);
        assert_eq!(metrics.tasks_completed, 1);
        assert_eq!(metrics.total_tokens_used, 500);
        assert_eq!(metrics.success_rate, 1.0);
    }

    #[test]
    fn test_agent_metrics_task_failure() {
        let agent_id = Uuid::new_v4();
        let mut metrics = AgentMetrics::new(agent_id, "test-agent");

        metrics.record_task_completion(1000, 500);
        metrics.record_task_failure(300);

        assert_eq!(metrics.tasks_completed, 1);
        assert_eq!(metrics.tasks_failed, 1);
        assert_eq!(metrics.success_rate, 0.5);
    }

    #[test]
    fn test_agent_metrics_error_tracking() {
        let agent_id = Uuid::new_v4();
        let mut metrics = AgentMetrics::new(agent_id, "test-agent");

        metrics.record_error(
            "timeout".to_string(),
            "Task timed out".to_string(),
            Some(Uuid::new_v4()),
        );

        assert_eq!(metrics.errors.len(), 1);
        assert!(!metrics.errors[0].resolved);

        metrics.resolve_error(0);
        assert!(metrics.errors[0].resolved);
    }

    #[test]
    fn test_agent_metrics_performance_score() {
        let agent_id = Uuid::new_v4();
        let mut metrics = AgentMetrics::new(agent_id, "test-agent");

        metrics.record_task_completion(1000, 500);
        metrics.calculate_performance_score();

        assert!(metrics.performance_score.is_some());
        assert!(metrics.performance_score.unwrap() > 0.0);
    }

    #[test]
    fn test_agent_metrics_collector() {
        let mut collector = AgentMetricsCollector::new();
        let agent_id = Uuid::new_v4();

        collector.register_agent(agent_id, "test-agent");
        assert_eq!(collector.total_agents(), 1);

        collector.record_task_completion(agent_id, 1000, 500);
        let metrics = collector.get_metrics(agent_id).unwrap();
        assert_eq!(metrics.tasks_completed, 1);
    }

    #[test]
    fn test_agent_metrics_collector_summary() {
        let mut collector = AgentMetricsCollector::new();
        let agent_id = Uuid::new_v4();

        collector.register_agent(agent_id, "test-agent");
        collector.record_task_completion(agent_id, 1000, 500);
        collector.record_task_completion(agent_id, 1500, 600);

        let summary = collector.get_summary();
        assert_eq!(summary.total_agents, 1);
        assert_eq!(summary.total_tasks_completed, 2);
        assert_eq!(summary.total_tokens_used, 1100);
        assert_eq!(summary.overall_success_rate, 1.0);
    }

    #[test]
    fn test_agent_metrics_capabilities() {
        let agent_id = Uuid::new_v4();
        let mut metrics = AgentMetrics::new(agent_id, "test-agent");

        metrics.add_capability_used("read_code".to_string());
        metrics.add_capability_used("write_code".to_string());
        metrics.add_capability_used("read_code".to_string()); // Duplicate

        assert_eq!(metrics.capabilities_used.len(), 2);
    }

    #[test]
    fn test_agent_metrics_collector_queries() {
        let mut collector = AgentMetricsCollector::new();
        let agent_id1 = Uuid::new_v4();
        let agent_id2 = Uuid::new_v4();

        collector.register_agent(agent_id1, "agent-1");
        collector.register_agent(agent_id2, "agent-2");

        collector.record_task_completion(agent_id1, 1000, 500);
        collector.record_task_failure(agent_id2, 300);

        assert_eq!(collector.total_tasks_completed(), 1);
        assert_eq!(collector.total_tasks_failed(), 1);
        assert_eq!(collector.overall_success_rate(), 0.5);
    }
}

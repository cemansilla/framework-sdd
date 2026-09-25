use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub id: Uuid,
    pub task_id: Uuid,
    pub agent_id: Uuid,
    pub execution_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub status: ExecutionMetricsStatus,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub total_tokens: usize,
    pub context_size: usize,
    pub artifacts_created: Vec<String>,
    pub artifacts_modified: Vec<String>,
    pub tests_created: Vec<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionMetricsStatus {
    Running,
    Success,
    Failed,
    Cancelled,
    Timeout,
}

impl ExecutionMetrics {
    pub fn new(task_id: Uuid, agent_id: Uuid, execution_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_id,
            agent_id,
            execution_id,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
            status: ExecutionMetricsStatus::Running,
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            context_size: 0,
            artifacts_created: Vec::new(),
            artifacts_modified: Vec::new(),
            tests_created: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn complete(&mut self, status: ExecutionMetricsStatus) {
        self.completed_at = Some(Utc::now());
        self.duration_ms = Some(
            self.completed_at
                .unwrap()
                .signed_duration_since(self.started_at)
                .num_milliseconds() as u64,
        );
        self.status = status;
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn add_artifact_created(&mut self, artifact: String) {
        self.artifacts_created.push(artifact);
    }

    pub fn add_artifact_modified(&mut self, artifact: String) {
        self.artifacts_modified.push(artifact);
    }

    pub fn add_test_created(&mut self, test: String) {
        self.tests_created.push(test);
    }

    pub fn set_token_usage(&mut self, input: usize, output: usize) {
        self.input_tokens = input;
        self.output_tokens = output;
        self.total_tokens = input + output;
    }

    pub fn set_context_size(&mut self, size: usize) {
        self.context_size = size;
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionMetricsCollector {
    metrics: HashMap<Uuid, ExecutionMetrics>,
}

impl ExecutionMetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    pub fn start_execution(&mut self, task_id: Uuid, agent_id: Uuid, execution_id: Uuid) -> Uuid {
        let metrics = ExecutionMetrics::new(task_id, agent_id, execution_id);
        let id = metrics.id;
        self.metrics.insert(id, metrics);
        id
    }

    pub fn complete_execution(&mut self, id: Uuid, status: ExecutionMetricsStatus) -> bool {
        if let Some(metrics) = self.metrics.get_mut(&id) {
            metrics.complete(status);
            true
        } else {
            false
        }
    }

    pub fn get_metrics(&self, id: Uuid) -> Option<&ExecutionMetrics> {
        self.metrics.get(&id)
    }

    pub fn get_all_metrics(&self) -> Vec<&ExecutionMetrics> {
        self.metrics.values().collect()
    }

    pub fn get_metrics_by_task(&self, task_id: Uuid) -> Vec<&ExecutionMetrics> {
        self.metrics
            .values()
            .filter(|m| m.task_id == task_id)
            .collect()
    }

    pub fn get_metrics_by_agent(&self, agent_id: Uuid) -> Vec<&ExecutionMetrics> {
        self.metrics
            .values()
            .filter(|m| m.agent_id == agent_id)
            .collect()
    }

    pub fn get_running_metrics(&self) -> Vec<&ExecutionMetrics> {
        self.metrics
            .values()
            .filter(|m| m.status == ExecutionMetricsStatus::Running)
            .collect()
    }

    pub fn get_successful_metrics(&self) -> Vec<&ExecutionMetrics> {
        self.metrics
            .values()
            .filter(|m| m.status == ExecutionMetricsStatus::Success)
            .collect()
    }

    pub fn get_failed_metrics(&self) -> Vec<&ExecutionMetrics> {
        self.metrics
            .values()
            .filter(|m| m.status == ExecutionMetricsStatus::Failed)
            .collect()
    }

    pub fn total_executions(&self) -> usize {
        self.metrics.len()
    }

    pub fn total_tokens_consumed(&self) -> usize {
        self.metrics.values().map(|m| m.total_tokens).sum()
    }

    pub fn average_duration_ms(&self) -> Option<f64> {
        let durations: Vec<u64> = self
            .metrics
            .values()
            .filter_map(|m| m.duration_ms)
            .collect();

        if durations.is_empty() {
            None
        } else {
            Some(durations.iter().sum::<u64>() as f64 / durations.len() as f64)
        }
    }

    pub fn success_rate(&self) -> f64 {
        let completed = self
            .metrics
            .values()
            .filter(|m| m.status != ExecutionMetricsStatus::Running)
            .count();

        if completed == 0 {
            return 0.0;
        }

        let successful = self
            .metrics
            .values()
            .filter(|m| m.status == ExecutionMetricsStatus::Success)
            .count();

        successful as f64 / completed as f64
    }

    pub fn remove_metrics(&mut self, id: Uuid) -> Option<ExecutionMetrics> {
        self.metrics.remove(&id)
    }

    pub fn clear(&mut self) {
        self.metrics.clear();
    }
}

impl Default for ExecutionMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_metrics_creation() {
        let task_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        let execution_id = Uuid::new_v4();

        let metrics = ExecutionMetrics::new(task_id, agent_id, execution_id);
        assert_eq!(metrics.task_id, task_id);
        assert_eq!(metrics.agent_id, agent_id);
        assert_eq!(metrics.status, ExecutionMetricsStatus::Running);
    }

    #[test]
    fn test_execution_metrics_complete() {
        let task_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        let execution_id = Uuid::new_v4();

        let mut metrics = ExecutionMetrics::new(task_id, agent_id, execution_id);
        metrics.complete(ExecutionMetricsStatus::Success);

        assert_eq!(metrics.status, ExecutionMetricsStatus::Success);
        assert!(metrics.completed_at.is_some());
        assert!(metrics.duration_ms.is_some());
    }

    #[test]
    fn test_execution_metrics_collector() {
        let mut collector = ExecutionMetricsCollector::new();
        let task_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        let execution_id = Uuid::new_v4();

        let id = collector.start_execution(task_id, agent_id, execution_id);
        assert_eq!(collector.total_executions(), 1);

        collector.complete_execution(id, ExecutionMetricsStatus::Success);
        let metrics = collector.get_metrics(id).unwrap();
        assert_eq!(metrics.status, ExecutionMetricsStatus::Success);
    }

    #[test]
    fn test_execution_metrics_collector_queries() {
        let mut collector = ExecutionMetricsCollector::new();
        let task_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();

        let id1 = collector.start_execution(task_id, agent_id, Uuid::new_v4());
        let id2 = collector.start_execution(task_id, agent_id, Uuid::new_v4());

        collector.complete_execution(id1, ExecutionMetricsStatus::Success);
        collector.complete_execution(id2, ExecutionMetricsStatus::Failed);

        assert_eq!(collector.get_metrics_by_task(task_id).len(), 2);
        assert_eq!(collector.get_successful_metrics().len(), 1);
        assert_eq!(collector.get_failed_metrics().len(), 1);
    }

    #[test]
    fn test_execution_metrics_token_usage() {
        let task_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        let execution_id = Uuid::new_v4();

        let mut metrics = ExecutionMetrics::new(task_id, agent_id, execution_id);
        metrics.set_token_usage(1000, 500);

        assert_eq!(metrics.input_tokens, 1000);
        assert_eq!(metrics.output_tokens, 500);
        assert_eq!(metrics.total_tokens, 1500);
    }

    #[test]
    fn test_execution_metrics_collector_stats() {
        let mut collector = ExecutionMetricsCollector::new();
        let task_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();

        let id1 = collector.start_execution(task_id, agent_id, Uuid::new_v4());
        let id2 = collector.start_execution(task_id, agent_id, Uuid::new_v4());

        collector.complete_execution(id1, ExecutionMetricsStatus::Success);
        collector.complete_execution(id2, ExecutionMetricsStatus::Success);

        assert_eq!(collector.success_rate(), 1.0);
    }
}

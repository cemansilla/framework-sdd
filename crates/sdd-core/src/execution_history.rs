use crate::agent::AgentAction;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub id: Uuid,
    pub task_id: Uuid,
    pub agent_id: Uuid,
    pub action: AgentAction,
    pub status: ExecutionRecordStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub total_tokens: usize,
    pub output: Option<String>,
    pub error: Option<String>,
    pub retry_count: u32,
    pub context_hash: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionRecordStatus {
    Pending,
    Running,
    Success,
    Failed,
    Cancelled,
    Timeout,
}

impl ExecutionRecord {
    pub fn new(task_id: Uuid, agent_id: Uuid, action: AgentAction, context_hash: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_id,
            agent_id,
            action,
            status: ExecutionRecordStatus::Pending,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            output: None,
            error: None,
            retry_count: 0,
            context_hash,
            metadata: HashMap::new(),
        }
    }

    pub fn mark_running(&mut self) {
        self.status = ExecutionRecordStatus::Running;
    }

    pub fn mark_success(&mut self, output: String, input_tokens: usize, output_tokens: usize) {
        self.status = ExecutionRecordStatus::Success;
        self.output = Some(output);
        self.input_tokens = input_tokens;
        self.output_tokens = output_tokens;
        self.total_tokens = input_tokens + output_tokens;
        self.completed_at = Some(Utc::now());
        self.duration_ms = Some(
            self.completed_at
                .unwrap()
                .signed_duration_since(self.started_at)
                .num_milliseconds() as u64,
        );
    }

    pub fn mark_failed(&mut self, error: String) {
        self.status = ExecutionRecordStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(Utc::now());
        self.duration_ms = Some(
            self.completed_at
                .unwrap()
                .signed_duration_since(self.started_at)
                .num_milliseconds() as u64,
        );
    }

    pub fn mark_cancelled(&mut self) {
        self.status = ExecutionRecordStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }

    pub fn mark_timeout(&mut self) {
        self.status = ExecutionRecordStatus::Timeout;
        self.completed_at = Some(Utc::now());
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            ExecutionRecordStatus::Success
                | ExecutionRecordStatus::Failed
                | ExecutionRecordStatus::Cancelled
                | ExecutionRecordStatus::Timeout
        )
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionHistory {
    records: Vec<ExecutionRecord>,
    max_records: usize,
}

impl ExecutionHistory {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            max_records: 10000,
        }
    }

    pub fn with_max_records(mut self, max: usize) -> Self {
        self.max_records = max;
        self
    }

    pub fn add_record(&mut self, record: ExecutionRecord) {
        if self.records.len() >= self.max_records {
            self.records.remove(0);
        }
        self.records.push(record);
    }

    pub fn get(&self, id: Uuid) -> Option<&ExecutionRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut ExecutionRecord> {
        self.records.iter_mut().find(|r| r.id == id)
    }

    pub fn get_by_task(&self, task_id: Uuid) -> Vec<&ExecutionRecord> {
        self.records
            .iter()
            .filter(|r| r.task_id == task_id)
            .collect()
    }

    pub fn get_by_agent(&self, agent_id: Uuid) -> Vec<&ExecutionRecord> {
        self.records
            .iter()
            .filter(|r| r.agent_id == agent_id)
            .collect()
    }

    pub fn get_by_status(&self, status: ExecutionRecordStatus) -> Vec<&ExecutionRecord> {
        self.records.iter().filter(|r| r.status == status).collect()
    }

    pub fn get_running(&self) -> Vec<&ExecutionRecord> {
        self.get_by_status(ExecutionRecordStatus::Running)
    }

    pub fn get_failed(&self) -> Vec<&ExecutionRecord> {
        self.get_by_status(ExecutionRecordStatus::Failed)
    }

    pub fn get_recent(&self, count: usize) -> Vec<&ExecutionRecord> {
        self.records.iter().rev().take(count).collect()
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }

    pub fn total_tokens_consumed(&self) -> usize {
        self.records.iter().map(|r| r.total_tokens).sum()
    }

    pub fn total_duration_ms(&self) -> u64 {
        self.records.iter().filter_map(|r| r.duration_ms).sum()
    }

    pub fn success_rate(&self) -> f64 {
        let terminal: Vec<_> = self.records.iter().filter(|r| r.is_terminal()).collect();
        if terminal.is_empty() {
            return 0.0;
        }
        let successes = terminal
            .iter()
            .filter(|r| r.status == ExecutionRecordStatus::Success)
            .count();
        successes as f64 / terminal.len() as f64
    }

    pub fn failure_rate(&self) -> f64 {
        1.0 - self.success_rate()
    }

    pub fn average_duration_ms(&self) -> Option<f64> {
        let durations: Vec<u64> = self.records.iter().filter_map(|r| r.duration_ms).collect();
        if durations.is_empty() {
            None
        } else {
            Some(durations.iter().sum::<u64>() as f64 / durations.len() as f64)
        }
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn records(&self) -> &[ExecutionRecord] {
        &self.records
    }
}

impl Default for ExecutionHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub cancelled_executions: usize,
    pub total_tokens: usize,
    pub average_tokens_per_execution: f64,
    pub average_duration_ms: f64,
    pub success_rate: f64,
    pub failure_rate: f64,
}

impl ExecutionMetrics {
    pub fn from_history(history: &ExecutionHistory) -> Self {
        let total = history.total_records();
        let successful = history.get_by_status(ExecutionRecordStatus::Success).len();
        let failed = history.get_by_status(ExecutionRecordStatus::Failed).len();
        let cancelled = history
            .get_by_status(ExecutionRecordStatus::Cancelled)
            .len();
        let total_tokens = history.total_tokens_consumed();

        Self {
            total_executions: total,
            successful_executions: successful,
            failed_executions: failed,
            cancelled_executions: cancelled,
            total_tokens,
            average_tokens_per_execution: if total > 0 {
                total_tokens as f64 / total as f64
            } else {
                0.0
            },
            average_duration_ms: history.average_duration_ms().unwrap_or(0.0),
            success_rate: history.success_rate(),
            failure_rate: history.failure_rate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentAction;

    fn create_test_record() -> ExecutionRecord {
        ExecutionRecord::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            AgentAction::DeleteFile,
            "hash123".to_string(),
        )
    }

    #[test]
    fn test_execution_record_creation() {
        let record = create_test_record();
        assert_eq!(record.status, ExecutionRecordStatus::Pending);
        assert_eq!(record.retry_count, 0);
    }

    #[test]
    fn test_execution_record_lifecycle() {
        let mut record = create_test_record();

        record.mark_running();
        assert_eq!(record.status, ExecutionRecordStatus::Running);

        record.mark_success("output".to_string(), 100, 50);
        assert_eq!(record.status, ExecutionRecordStatus::Success);
        assert_eq!(record.total_tokens, 150);
        assert!(record.duration_ms.is_some());
    }

    #[test]
    fn test_execution_record_failure() {
        let mut record = create_test_record();
        record.mark_running();
        record.mark_failed("error occurred".to_string());

        assert_eq!(record.status, ExecutionRecordStatus::Failed);
        assert!(record.error.is_some());
        assert!(record.is_terminal());
    }

    #[test]
    fn test_execution_record_retry() {
        let mut record = create_test_record();
        record.increment_retry();
        record.increment_retry();
        assert_eq!(record.retry_count, 2);
    }

    #[test]
    fn test_execution_history_creation() {
        let history = ExecutionHistory::new();
        assert_eq!(history.total_records(), 0);
    }

    #[test]
    fn test_execution_history_add_and_get() {
        let mut history = ExecutionHistory::new();
        let record = create_test_record();
        let id = record.id;

        history.add_record(record);
        assert_eq!(history.total_records(), 1);

        let retrieved = history.get(id);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_execution_history_get_by_task() {
        let mut history = ExecutionHistory::new();
        let task_id = Uuid::new_v4();

        let mut record1 = create_test_record();
        record1.task_id = task_id;
        let mut record2 = create_test_record();
        record2.task_id = task_id;
        let record3 = create_test_record();

        history.add_record(record1);
        history.add_record(record2);
        history.add_record(record3);

        let task_records = history.get_by_task(task_id);
        assert_eq!(task_records.len(), 2);
    }

    #[test]
    fn test_execution_history_metrics() {
        let mut history = ExecutionHistory::new();

        let mut record1 = create_test_record();
        record1.mark_running();
        record1.mark_success("output".to_string(), 100, 50);

        let mut record2 = create_test_record();
        record2.mark_running();
        record2.mark_failed("error".to_string());

        history.add_record(record1);
        history.add_record(record2);

        assert_eq!(history.success_rate(), 0.5);
        assert_eq!(history.failure_rate(), 0.5);
        assert_eq!(history.total_tokens_consumed(), 150);
    }

    #[test]
    fn test_execution_metrics_from_history() {
        let mut history = ExecutionHistory::new();

        let mut record = create_test_record();
        record.mark_running();
        record.mark_success("output".to_string(), 100, 50);
        history.add_record(record);

        let metrics = ExecutionMetrics::from_history(&history);
        assert_eq!(metrics.total_executions, 1);
        assert_eq!(metrics.successful_executions, 1);
        assert_eq!(metrics.success_rate, 1.0);
    }

    #[test]
    fn test_execution_history_max_records() {
        let mut history = ExecutionHistory::new().with_max_records(3);

        for _ in 0..5 {
            history.add_record(create_test_record());
        }

        assert_eq!(history.total_records(), 3);
    }

    #[test]
    fn test_execution_history_get_recent() {
        let mut history = ExecutionHistory::new();

        for _ in 0..10 {
            history.add_record(create_test_record());
        }

        let recent = history.get_recent(3);
        assert_eq!(recent.len(), 3);
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetrics {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub task_id: Uuid,
    pub validation_type: ValidationType,
    pub status: ValidationStatus,
    pub checks_performed: usize,
    pub checks_passed: usize,
    pub checks_failed: usize,
    pub duration_ms: u64,
    pub findings: Vec<ValidationFinding>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationType {
    CodeQuality,
    Security,
    Performance,
    Compliance,
    Functional,
    Integration,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationStatus {
    Passed,
    Failed,
    Warning,
    Skipped,
    Error,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FindingSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl ValidationMetrics {
    pub fn new(task_id: Uuid, validation_type: ValidationType, duration_ms: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            task_id,
            validation_type,
            status: ValidationStatus::Passed,
            checks_performed: 0,
            checks_passed: 0,
            checks_failed: 0,
            duration_ms,
            findings: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_check(&mut self, passed: bool) {
        self.checks_performed += 1;
        if passed {
            self.checks_passed += 1;
        } else {
            self.checks_failed += 1;
        }
    }

    pub fn add_finding(
        &mut self,
        severity: FindingSeverity,
        category: impl Into<String>,
        message: impl Into<String>,
        location: Option<String>,
        recommendation: Option<String>,
    ) {
        self.findings.push(ValidationFinding {
            id: Uuid::new_v4(),
            severity,
            category: category.into(),
            message: message.into(),
            location,
            recommendation,
            resolved: false,
        });
    }

    pub fn resolve_finding(&mut self, finding_id: Uuid) -> bool {
        if let Some(finding) = self.findings.iter_mut().find(|f| f.id == finding_id) {
            finding.resolved = true;
            true
        } else {
            false
        }
    }

    pub fn calculate_status(&mut self) {
        let has_critical = self
            .findings
            .iter()
            .any(|f| f.severity == FindingSeverity::Critical && !f.resolved);
        let has_high = self
            .findings
            .iter()
            .any(|f| f.severity == FindingSeverity::High && !f.resolved);

        if has_critical || self.checks_failed > 0 {
            self.status = ValidationStatus::Failed;
        } else if has_high {
            self.status = ValidationStatus::Warning;
        } else {
            self.status = ValidationStatus::Passed;
        }
    }

    pub fn pass_rate(&self) -> f64 {
        if self.checks_performed == 0 {
            return 0.0;
        }
        self.checks_passed as f64 / self.checks_performed as f64
    }

    pub fn unresolved_findings_count(&self) -> usize {
        self.findings.iter().filter(|f| !f.resolved).count()
    }

    pub fn critical_findings_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == FindingSeverity::Critical && !f.resolved)
            .count()
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetricsSummary {
    pub total_validations: usize,
    pub passed_validations: usize,
    pub failed_validations: usize,
    pub warning_validations: usize,
    pub overall_pass_rate: f64,
    pub total_checks_performed: usize,
    pub total_checks_passed: usize,
    pub total_checks_failed: usize,
    pub total_findings: usize,
    pub critical_findings: usize,
    pub high_findings: usize,
    pub validation_type_counts: HashMap<String, usize>,
    pub average_duration_ms: f64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ValidationMetricsCollector {
    metrics: Vec<ValidationMetrics>,
}

impl ValidationMetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
        }
    }

    pub fn record(&mut self, metrics: ValidationMetrics) {
        self.metrics.push(metrics);
    }

    pub fn get_all_metrics(&self) -> &[ValidationMetrics] {
        &self.metrics
    }

    pub fn get_metrics_by_task(&self, task_id: Uuid) -> Vec<&ValidationMetrics> {
        self.metrics
            .iter()
            .filter(|m| m.task_id == task_id)
            .collect()
    }

    pub fn get_metrics_by_type(&self, validation_type: &ValidationType) -> Vec<&ValidationMetrics> {
        self.metrics
            .iter()
            .filter(|m| &m.validation_type == validation_type)
            .collect()
    }

    pub fn get_metrics_by_status(&self, status: &ValidationStatus) -> Vec<&ValidationMetrics> {
        self.metrics
            .iter()
            .filter(|m| &m.status == status)
            .collect()
    }

    pub fn get_summary(&self) -> ValidationMetricsSummary {
        self.get_summary_in_period(None, None)
    }

    pub fn get_summary_in_period(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> ValidationMetricsSummary {
        let filtered_metrics: Vec<&ValidationMetrics> = self
            .metrics
            .iter()
            .filter(|m| {
                let after_start = start.map(|s| m.timestamp >= s).unwrap_or(true);
                let before_end = end.map(|e| m.timestamp <= e).unwrap_or(true);
                after_start && before_end
            })
            .collect();

        let total_validations = filtered_metrics.len();
        let passed_validations = filtered_metrics
            .iter()
            .filter(|m| m.status == ValidationStatus::Passed)
            .count();
        let failed_validations = filtered_metrics
            .iter()
            .filter(|m| m.status == ValidationStatus::Failed)
            .count();
        let warning_validations = filtered_metrics
            .iter()
            .filter(|m| m.status == ValidationStatus::Warning)
            .count();

        let overall_pass_rate = if total_validations > 0 {
            passed_validations as f64 / total_validations as f64
        } else {
            0.0
        };

        let total_checks_performed = filtered_metrics.iter().map(|m| m.checks_performed).sum();
        let total_checks_passed = filtered_metrics.iter().map(|m| m.checks_passed).sum();
        let total_checks_failed = filtered_metrics.iter().map(|m| m.checks_failed).sum();

        let total_findings = filtered_metrics.iter().map(|m| m.findings.len()).sum();
        let critical_findings = filtered_metrics
            .iter()
            .flat_map(|m| &m.findings)
            .filter(|f| f.severity == FindingSeverity::Critical)
            .count();
        let high_findings = filtered_metrics
            .iter()
            .flat_map(|m| &m.findings)
            .filter(|f| f.severity == FindingSeverity::High)
            .count();

        let mut validation_type_counts = HashMap::new();
        for metrics in &filtered_metrics {
            let type_name = match &metrics.validation_type {
                ValidationType::CodeQuality => "code_quality",
                ValidationType::Security => "security",
                ValidationType::Performance => "performance",
                ValidationType::Compliance => "compliance",
                ValidationType::Functional => "functional",
                ValidationType::Integration => "integration",
                ValidationType::Custom(name) => name.as_str(),
            };
            *validation_type_counts
                .entry(type_name.to_string())
                .or_insert(0) += 1;
        }

        let total_duration_ms: u64 = filtered_metrics.iter().map(|m| m.duration_ms).sum();
        let average_duration_ms = if total_validations > 0 {
            total_duration_ms as f64 / total_validations as f64
        } else {
            0.0
        };

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

        ValidationMetricsSummary {
            total_validations,
            passed_validations,
            failed_validations,
            warning_validations,
            overall_pass_rate,
            total_checks_performed,
            total_checks_passed,
            total_checks_failed,
            total_findings,
            critical_findings,
            high_findings,
            validation_type_counts,
            average_duration_ms,
            period_start,
            period_end,
        }
    }

    pub fn total_validations(&self) -> usize {
        self.metrics.len()
    }

    pub fn total_findings(&self) -> usize {
        self.metrics.iter().map(|m| m.findings.len()).sum()
    }

    pub fn clear(&mut self) {
        self.metrics.clear();
    }
}

impl Default for ValidationMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_metrics_creation() {
        let task_id = Uuid::new_v4();
        let metrics = ValidationMetrics::new(task_id, ValidationType::CodeQuality, 100);

        assert_eq!(metrics.task_id, task_id);
        assert_eq!(metrics.validation_type, ValidationType::CodeQuality);
        assert_eq!(metrics.status, ValidationStatus::Passed);
        assert_eq!(metrics.checks_performed, 0);
    }

    #[test]
    fn test_validation_metrics_add_check() {
        let task_id = Uuid::new_v4();
        let mut metrics = ValidationMetrics::new(task_id, ValidationType::Security, 100);

        metrics.add_check(true);
        metrics.add_check(true);
        metrics.add_check(false);

        assert_eq!(metrics.checks_performed, 3);
        assert_eq!(metrics.checks_passed, 2);
        assert_eq!(metrics.checks_failed, 1);
    }

    #[test]
    fn test_validation_metrics_add_finding() {
        let task_id = Uuid::new_v4();
        let mut metrics = ValidationMetrics::new(task_id, ValidationType::Security, 100);

        metrics.add_finding(
            FindingSeverity::High,
            "security",
            "SQL injection vulnerability",
            Some("src/db.rs:42".to_string()),
            Some("Use parameterized queries".to_string()),
        );

        assert_eq!(metrics.findings.len(), 1);
        assert_eq!(metrics.findings[0].severity, FindingSeverity::High);
    }

    #[test]
    fn test_validation_metrics_calculate_status() {
        let task_id = Uuid::new_v4();
        let mut metrics = ValidationMetrics::new(task_id, ValidationType::CodeQuality, 100);

        metrics.add_finding(
            FindingSeverity::Critical,
            "quality",
            "Critical issue",
            None,
            None,
        );
        metrics.calculate_status();

        assert_eq!(metrics.status, ValidationStatus::Failed);
    }

    #[test]
    fn test_validation_metrics_pass_rate() {
        let task_id = Uuid::new_v4();
        let mut metrics = ValidationMetrics::new(task_id, ValidationType::Functional, 100);

        metrics.add_check(true);
        metrics.add_check(true);
        metrics.add_check(false);

        assert_eq!(metrics.pass_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_validation_metrics_collector() {
        let mut collector = ValidationMetricsCollector::new();
        let task_id = Uuid::new_v4();

        let metrics = ValidationMetrics::new(task_id, ValidationType::CodeQuality, 100);
        collector.record(metrics);

        assert_eq!(collector.total_validations(), 1);
    }

    #[test]
    fn test_validation_metrics_collector_queries() {
        let mut collector = ValidationMetricsCollector::new();
        let task_id = Uuid::new_v4();

        let metrics1 = ValidationMetrics::new(task_id, ValidationType::CodeQuality, 100);
        let metrics2 = ValidationMetrics::new(task_id, ValidationType::Security, 150);

        collector.record(metrics1);
        collector.record(metrics2);

        assert_eq!(collector.get_metrics_by_task(task_id).len(), 2);
        assert_eq!(
            collector
                .get_metrics_by_type(&ValidationType::CodeQuality)
                .len(),
            1
        );
    }

    #[test]
    fn test_validation_metrics_summary() {
        let mut collector = ValidationMetricsCollector::new();
        let task_id = Uuid::new_v4();

        for _ in 0..3 {
            let mut metrics = ValidationMetrics::new(task_id, ValidationType::CodeQuality, 100);
            metrics.add_check(true);
            metrics.add_check(true);
            collector.record(metrics);
        }

        let summary = collector.get_summary();
        assert_eq!(summary.total_validations, 3);
        assert_eq!(summary.passed_validations, 3);
        assert_eq!(summary.total_checks_performed, 6);
        assert_eq!(summary.total_checks_passed, 6);
        assert_eq!(summary.overall_pass_rate, 1.0);
    }

    #[test]
    fn test_validation_metrics_resolve_finding() {
        let task_id = Uuid::new_v4();
        let mut metrics = ValidationMetrics::new(task_id, ValidationType::Security, 100);

        metrics.add_finding(FindingSeverity::Medium, "security", "Issue", None, None);

        let finding_id = metrics.findings[0].id;
        assert!(!metrics.findings[0].resolved);

        metrics.resolve_finding(finding_id);
        assert!(metrics.findings[0].resolved);
    }

    #[test]
    fn test_validation_metrics_counts() {
        let task_id = Uuid::new_v4();
        let mut metrics = ValidationMetrics::new(task_id, ValidationType::CodeQuality, 100);

        metrics.add_finding(FindingSeverity::Critical, "cat1", "msg1", None, None);
        metrics.add_finding(FindingSeverity::High, "cat2", "msg2", None, None);
        metrics.add_finding(FindingSeverity::Medium, "cat3", "msg3", None, None);

        assert_eq!(metrics.unresolved_findings_count(), 3);
        assert_eq!(metrics.critical_findings_count(), 1);
    }
}

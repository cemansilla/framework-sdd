use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub tests: Vec<TestCase>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub id: String,
    pub name: String,
    pub description: String,
    pub test_type: TestType,
    pub status: TestStatus,
    pub result: Option<TestResult>,
    pub verifies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestType {
    Unit,
    Integration,
    Contract,
    E2e,
    Performance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestStatus {
    Pending,
    Passed,
    Failed,
    Skipped,
    Ignored,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub executed_at: DateTime<Utc>,
    pub duration_ms: u64,
    pub output: Option<String>,
    pub error: Option<String>,
    pub evidence: Vec<String>,
}

impl TestSuite {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            tests: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_test(mut self, test: TestCase) -> Self {
        self.tests.push(test);
        self.updated_at = Utc::now();
        self
    }

    pub fn passed_count(&self) -> usize {
        self.tests
            .iter()
            .filter(|t| t.status == TestStatus::Passed)
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.tests
            .iter()
            .filter(|t| t.status == TestStatus::Failed)
            .count()
    }

    pub fn total_count(&self) -> usize {
        self.tests.len()
    }
}

impl TestCase {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        test_type: TestType,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            test_type,
            status: TestStatus::Pending,
            result: None,
            verifies: Vec::new(),
        }
    }

    pub fn verifies(mut self, artifact_id: impl Into<String>) -> Self {
        self.verifies.push(artifact_id.into());
        self
    }

    pub fn pass(mut self, duration_ms: u64) -> Self {
        self.status = TestStatus::Passed;
        self.result = Some(TestResult {
            executed_at: Utc::now(),
            duration_ms,
            output: None,
            error: None,
            evidence: Vec::new(),
        });
        self
    }

    pub fn fail(mut self, duration_ms: u64, error: impl Into<String>) -> Self {
        self.status = TestStatus::Failed;
        self.result = Some(TestResult {
            executed_at: Utc::now(),
            duration_ms,
            output: None,
            error: Some(error.into()),
            evidence: Vec::new(),
        });
        self
    }

    pub fn skip(mut self) -> Self {
        self.status = TestStatus::Skipped;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: Uuid,
    pub review_id: String,
    pub reviewer: String,
    pub verdict: ReviewVerdict,
    pub findings: Vec<Finding>,
    pub summary: String,
    pub reviewed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReviewVerdict {
    Approved,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub severity: FindingSeverity,
    pub category: FindingCategory,
    pub description: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FindingSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FindingCategory {
    Security,
    Performance,
    Quality,
    Adherence,
    Testing,
    Documentation,
}

impl Review {
    pub fn new(
        review_id: impl Into<String>,
        reviewer: impl Into<String>,
        verdict: ReviewVerdict,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            review_id: review_id.into(),
            reviewer: reviewer.into(),
            verdict,
            findings: Vec::new(),
            summary: summary.into(),
            reviewed_at: Utc::now(),
        }
    }

    pub fn add_finding(mut self, finding: Finding) -> Self {
        self.findings.push(finding);
        self
    }

    pub fn critical_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == FindingSeverity::Critical)
            .count()
    }

    pub fn is_approved(&self) -> bool {
        self.verdict == ReviewVerdict::Approved
    }
}

impl Finding {
    pub fn new(
        id: impl Into<String>,
        severity: FindingSeverity,
        category: FindingCategory,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            severity,
            category,
            description: description.into(),
            file: None,
            line: None,
            suggestion: None,
        }
    }

    pub fn with_location(mut self, file: impl Into<String>, line: u32) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub id: Uuid,
    pub task_id: String,
    pub test_results: HashMap<String, TestSuite>,
    pub reviews: Vec<Review>,
    pub overall_status: VerificationStatus,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerificationStatus {
    Pending,
    Passed,
    Failed,
    Partial,
}

impl VerificationReport {
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_id: task_id.into(),
            test_results: HashMap::new(),
            reviews: Vec::new(),
            overall_status: VerificationStatus::Pending,
            generated_at: Utc::now(),
        }
    }

    pub fn add_test_suite(mut self, name: impl Into<String>, suite: TestSuite) -> Self {
        self.test_results.insert(name.into(), suite);
        self.updated();
        self
    }

    pub fn add_review(mut self, review: Review) -> Self {
        self.reviews.push(review);
        self.updated();
        self
    }

    fn updated(&mut self) {
        self.generated_at = Utc::now();
        self.recalculate_status();
    }

    fn recalculate_status(&mut self) {
        let all_tests_passed = self.test_results.values().all(|s| s.failed_count() == 0);
        let all_reviews_approved = self.reviews.iter().all(|r| r.is_approved());

        if self.test_results.is_empty() && self.reviews.is_empty() {
            self.overall_status = VerificationStatus::Pending;
        } else if all_tests_passed && all_reviews_approved {
            self.overall_status = VerificationStatus::Passed;
        } else if !all_tests_passed
            || self
                .reviews
                .iter()
                .any(|r| r.verdict == ReviewVerdict::Rejected)
        {
            self.overall_status = VerificationStatus::Failed;
        } else {
            self.overall_status = VerificationStatus::Partial;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_suite_creation() {
        let suite = TestSuite::new("Unit Tests", "Unit test suite");
        assert_eq!(suite.total_count(), 0);
    }

    #[test]
    fn test_test_case_pass() {
        let test = TestCase::new("T-001", "test_login", "Test login", TestType::Unit).pass(150);
        assert_eq!(test.status, TestStatus::Passed);
        assert!(test.result.is_some());
    }

    #[test]
    fn test_test_case_fail() {
        let test = TestCase::new("T-001", "test_login", "Test login", TestType::Unit)
            .fail(100, "assertion failed");
        assert_eq!(test.status, TestStatus::Failed);
    }

    #[test]
    fn test_review_approved() {
        let review = Review::new("REV-001", "reviewer", ReviewVerdict::Approved, "Looks good");
        assert!(review.is_approved());
    }

    #[test]
    fn test_verification_report() {
        let mut report = VerificationReport::new("TASK-001");
        let suite = TestSuite::new("Unit", "Unit tests")
            .add_test(TestCase::new("T-001", "test_1", "Test 1", TestType::Unit).pass(100));
        report.test_results.insert("unit".to_string(), suite);
        report.recalculate_status();
        assert_eq!(report.overall_status, VerificationStatus::Passed);
    }
}

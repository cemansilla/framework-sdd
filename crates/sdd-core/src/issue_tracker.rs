use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: IssueStatus,
    pub priority: IssuePriority,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IssueStatus {
    Open,
    InProgress,
    Review,
    Closed,
    WontFix,
    Duplicate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IssuePriority {
    Critical,
    High,
    Medium,
    Low,
}

#[async_trait]
pub trait IssueTrackerAdapter: Send + Sync {
    async fn get_issue(&self, issue_id: &str) -> Result<Issue, IssueTrackerError>;
    async fn list_issues(&self, status: Option<IssueStatus>) -> Result<Vec<Issue>, IssueTrackerError>;
    async fn create_issue(&self, title: &str, description: &str) -> Result<Issue, IssueTrackerError>;
    async fn update_issue(&self, issue_id: &str, updates: IssueUpdate) -> Result<Issue, IssueTrackerError>;
    async fn close_issue(&self, issue_id: &str, reason: Option<&str>) -> Result<(), IssueTrackerError>;
    async fn add_comment(&self, issue_id: &str, comment: &str) -> Result<(), IssueTrackerError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<IssueStatus>,
    pub priority: Option<IssuePriority>,
    pub assignee: Option<String>,
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, thiserror::Error)]
pub enum IssueTrackerError {
    #[error("issue not found: {0}")]
    NotFound(String),
    #[error("connection error: {0}")]
    ConnectionError(String),
    #[error("authentication error: {0}")]
    AuthError(String),
    #[error("rate limit exceeded")]
    RateLimitExceeded,
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

pub struct GitHubAdapter {
    #[allow(dead_code)]
    base_url: String,
    #[allow(dead_code)]
    token: String,
    #[allow(dead_code)]
    owner: String,
    #[allow(dead_code)]
    repo: String,
}

impl GitHubAdapter {
    pub fn new(
        owner: impl Into<String>,
        repo: impl Into<String>,
        token: impl Into<String>,
    ) -> Self {
        Self {
            base_url: "https://api.github.com".to_string(),
            token: token.into(),
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}

#[async_trait]
impl IssueTrackerAdapter for GitHubAdapter {
    async fn get_issue(&self, _issue_id: &str) -> Result<Issue, IssueTrackerError> {
        Err(IssueTrackerError::ConnectionError(
            "GitHub adapter not fully implemented".to_string(),
        ))
    }

    async fn list_issues(&self, _status: Option<IssueStatus>) -> Result<Vec<Issue>, IssueTrackerError> {
        Err(IssueTrackerError::ConnectionError(
            "GitHub adapter not fully implemented".to_string(),
        ))
    }

    async fn create_issue(&self, _title: &str, _description: &str) -> Result<Issue, IssueTrackerError> {
        Err(IssueTrackerError::ConnectionError(
            "GitHub adapter not fully implemented".to_string(),
        ))
    }

    async fn update_issue(&self, _issue_id: &str, _updates: IssueUpdate) -> Result<Issue, IssueTrackerError> {
        Err(IssueTrackerError::ConnectionError(
            "GitHub adapter not fully implemented".to_string(),
        ))
    }

    async fn close_issue(&self, _issue_id: &str, _reason: Option<&str>) -> Result<(), IssueTrackerError> {
        Err(IssueTrackerError::ConnectionError(
            "GitHub adapter not fully implemented".to_string(),
        ))
    }

    async fn add_comment(&self, _issue_id: &str, _comment: &str) -> Result<(), IssueTrackerError> {
        Err(IssueTrackerError::ConnectionError(
            "GitHub adapter not fully implemented".to_string(),
        ))
    }
}

pub struct JiraAdapter {
    #[allow(dead_code)]
    base_url: String,
    #[allow(dead_code)]
    username: String,
    #[allow(dead_code)]
    api_token: String,
    #[allow(dead_code)]
    project_key: String,
}

impl JiraAdapter {
    pub fn new(
        base_url: impl Into<String>,
        username: impl Into<String>,
        api_token: impl Into<String>,
        project_key: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            username: username.into(),
            api_token: api_token.into(),
            project_key: project_key.into(),
        }
    }
}

#[async_trait]
impl IssueTrackerAdapter for JiraAdapter {
    async fn get_issue(&self, _issue_id: &str) -> Result<Issue, IssueTrackerError> {
        Err(IssueTrackerError::ConnectionError(
            "Jira adapter not fully implemented".to_string(),
        ))
    }

    async fn list_issues(&self, _status: Option<IssueStatus>) -> Result<Vec<Issue>, IssueTrackerError> {
        Err(IssueTrackerError::ConnectionError(
            "Jira adapter not fully implemented".to_string(),
        ))
    }

    async fn create_issue(&self, _title: &str, _description: &str) -> Result<Issue, IssueTrackerError> {
        Err(IssueTrackerError::ConnectionError(
            "Jira adapter not fully implemented".to_string(),
        ))
    }

    async fn update_issue(&self, _issue_id: &str, _updates: IssueUpdate) -> Result<Issue, IssueTrackerError> {
        Err(IssueTrackerError::ConnectionError(
            "Jira adapter not fully implemented".to_string(),
        ))
    }

    async fn close_issue(&self, _issue_id: &str, _reason: Option<&str>) -> Result<(), IssueTrackerError> {
        Err(IssueTrackerError::ConnectionError(
            "Jira adapter not fully implemented".to_string(),
        ))
    }

    async fn add_comment(&self, _issue_id: &str, _comment: &str) -> Result<(), IssueTrackerError> {
        Err(IssueTrackerError::ConnectionError(
            "Jira adapter not fully implemented".to_string(),
        ))
    }
}

pub struct MockIssueTracker {
    issues: HashMap<String, Issue>,
}

impl MockIssueTracker {
    pub fn new() -> Self {
        Self {
            issues: HashMap::new(),
        }
    }

    pub fn add_mock_issue(&mut self, issue: Issue) {
        self.issues.insert(issue.id.clone(), issue);
    }
}

impl Default for MockIssueTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IssueTrackerAdapter for MockIssueTracker {
    async fn get_issue(&self, issue_id: &str) -> Result<Issue, IssueTrackerError> {
        self.issues
            .get(issue_id)
            .cloned()
            .ok_or_else(|| IssueTrackerError::NotFound(issue_id.to_string()))
    }

    async fn list_issues(&self, status: Option<IssueStatus>) -> Result<Vec<Issue>, IssueTrackerError> {
        let issues: Vec<Issue> = self
            .issues
            .values()
            .filter(|issue| {
                status
                    .as_ref()
                    .map(|s| &issue.status == s)
                    .unwrap_or(true)
            })
            .cloned()
            .collect();
        Ok(issues)
    }

    async fn create_issue(&self, title: &str, description: &str) -> Result<Issue, IssueTrackerError> {
        let issue = Issue {
            id: format!("MOCK-{}", self.issues.len() + 1),
            title: title.to_string(),
            description: description.to_string(),
            status: IssueStatus::Open,
            priority: IssuePriority::Medium,
            assignee: None,
            labels: Vec::new(),
            url: format!("https://mock-tracker.example.com/issues/{}", self.issues.len() + 1),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };
        Ok(issue)
    }

    async fn update_issue(&self, issue_id: &str, updates: IssueUpdate) -> Result<Issue, IssueTrackerError> {
        let mut issue = self
            .issues
            .get(issue_id)
            .cloned()
            .ok_or_else(|| IssueTrackerError::NotFound(issue_id.to_string()))?;

        if let Some(title) = updates.title {
            issue.title = title;
        }
        if let Some(description) = updates.description {
            issue.description = description;
        }
        if let Some(status) = updates.status {
            issue.status = status;
        }
        if let Some(priority) = updates.priority {
            issue.priority = priority;
        }
        if let Some(assignee) = updates.assignee {
            issue.assignee = Some(assignee);
        }
        if let Some(labels) = updates.labels {
            issue.labels = labels;
        }

        issue.updated_at = Utc::now();
        Ok(issue)
    }

    async fn close_issue(&self, issue_id: &str, _reason: Option<&str>) -> Result<(), IssueTrackerError> {
        if !self.issues.contains_key(issue_id) {
            return Err(IssueTrackerError::NotFound(issue_id.to_string()));
        }
        Ok(())
    }

    async fn add_comment(&self, issue_id: &str, _comment: &str) -> Result<(), IssueTrackerError> {
        if !self.issues.contains_key(issue_id) {
            return Err(IssueTrackerError::NotFound(issue_id.to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_tracker_creation() {
        let tracker = MockIssueTracker::new();
        let issues = tracker.list_issues(None).await.unwrap();
        assert_eq!(issues.len(), 0);
    }

    #[tokio::test]
    async fn test_mock_tracker_add_issue() {
        let mut tracker = MockIssueTracker::new();
        let issue = Issue {
            id: "TEST-001".to_string(),
            title: "Test Issue".to_string(),
            description: "Test Description".to_string(),
            status: IssueStatus::Open,
            priority: IssuePriority::Medium,
            assignee: None,
            labels: Vec::new(),
            url: "https://example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        tracker.add_mock_issue(issue);
        let issues = tracker.list_issues(None).await.unwrap();
        assert_eq!(issues.len(), 1);
    }

    #[tokio::test]
    async fn test_mock_tracker_get_issue() {
        let mut tracker = MockIssueTracker::new();
        let issue = Issue {
            id: "TEST-001".to_string(),
            title: "Test Issue".to_string(),
            description: "Test Description".to_string(),
            status: IssueStatus::Open,
            priority: IssuePriority::Medium,
            assignee: None,
            labels: Vec::new(),
            url: "https://example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        tracker.add_mock_issue(issue);
        let retrieved = tracker.get_issue("TEST-001").await.unwrap();
        assert_eq!(retrieved.title, "Test Issue");
    }

    #[tokio::test]
    async fn test_mock_tracker_get_nonexistent_issue() {
        let tracker = MockIssueTracker::new();
        let result = tracker.get_issue("NONEXISTENT").await;
        assert!(matches!(result, Err(IssueTrackerError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_mock_tracker_list_by_status() {
        let mut tracker = MockIssueTracker::new();
        
        let issue1 = Issue {
            id: "TEST-001".to_string(),
            title: "Open Issue".to_string(),
            description: "Description".to_string(),
            status: IssueStatus::Open,
            priority: IssuePriority::Medium,
            assignee: None,
            labels: Vec::new(),
            url: "https://example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        let issue2 = Issue {
            id: "TEST-002".to_string(),
            title: "Closed Issue".to_string(),
            description: "Description".to_string(),
            status: IssueStatus::Closed,
            priority: IssuePriority::Medium,
            assignee: None,
            labels: Vec::new(),
            url: "https://example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        tracker.add_mock_issue(issue1);
        tracker.add_mock_issue(issue2);

        let open_issues = tracker.list_issues(Some(IssueStatus::Open)).await.unwrap();
        assert_eq!(open_issues.len(), 1);
        assert_eq!(open_issues[0].status, IssueStatus::Open);
    }

    #[tokio::test]
    async fn test_github_adapter_creation() {
        let adapter = GitHubAdapter::new("owner", "repo", "token");
        assert_eq!(adapter.owner, "owner");
        assert_eq!(adapter.repo, "repo");
    }

    #[tokio::test]
    async fn test_jira_adapter_creation() {
        let adapter = JiraAdapter::new(
            "https://jira.example.com",
            "user",
            "token",
            "PROJ",
        );
        assert_eq!(adapter.project_key, "PROJ");
    }
}

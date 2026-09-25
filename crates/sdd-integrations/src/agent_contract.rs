use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContract {
    pub id: Uuid,
    pub agent_id: String,
    pub version: String,
    pub protocol: AgentProtocol,
    pub capabilities: Vec<AgentCapability>,
    pub request_format: RequestFormat,
    pub response_format: ResponseFormat,
    pub authentication: AuthenticationMethod,
    pub rate_limits: RateLimits,
    pub timeout_seconds: u64,
    pub retry_policy: RetryPolicy,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentProtocol {
    Http { endpoint: String },
    Mcp { server_url: String },
    WebSocket { url: String },
    Grpc { endpoint: String },
    Stdio,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentCapability {
    ExecuteTask,
    GetContext,
    SubmitResult,
    RequestApproval,
    QueryStatus,
    ListTasks,
    GetTraceability,
    SubmitValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequestFormat {
    Json,
    Xml,
    Protobuf,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResponseFormat {
    Json,
    Xml,
    Protobuf,
    Stream,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthenticationMethod {
    None,
    ApiKey { header_name: String },
    BearerToken,
    OAuth2 { flow: OAuth2Flow },
    MutualTls,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OAuth2Flow {
    ClientCredentials,
    AuthorizationCode,
    DeviceCode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RateLimits {
    pub requests_per_minute: Option<u32>,
    pub requests_per_hour: Option<u32>,
    pub requests_per_day: Option<u32>,
    pub concurrent_requests: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub retryable_errors: Vec<String>,
}

impl AgentContract {
    pub fn new(
        agent_id: impl Into<String>,
        version: impl Into<String>,
        protocol: AgentProtocol,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            agent_id: agent_id.into(),
            version: version.into(),
            protocol,
            capabilities: Vec::new(),
            request_format: RequestFormat::Json,
            response_format: ResponseFormat::Json,
            authentication: AuthenticationMethod::None,
            rate_limits: RateLimits::default(),
            timeout_seconds: 30,
            retry_policy: RetryPolicy::default(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_capabilities(mut self, capabilities: Vec<AgentCapability>) -> Self {
        self.capabilities = capabilities;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_request_format(mut self, format: RequestFormat) -> Self {
        self.request_format = format;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_response_format(mut self, format: ResponseFormat) -> Self {
        self.response_format = format;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_authentication(mut self, auth: AuthenticationMethod) -> Self {
        self.authentication = auth;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_rate_limits(mut self, limits: RateLimits) -> Self {
        self.rate_limits = limits;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn supports_capability(&self, capability: &AgentCapability) -> bool {
        self.capabilities.contains(capability)
    }
}

impl Default for RateLimits {
    fn default() -> Self {
        Self {
            requests_per_minute: Some(60),
            requests_per_hour: Some(1000),
            requests_per_day: Some(10000),
            concurrent_requests: Some(10),
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            retryable_errors: vec![
                "timeout".to_string(),
                "connection_error".to_string(),
                "rate_limited".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    pub task_id: Uuid,
    pub agent_id: String,
    pub context_bundle_id: Option<Uuid>,
    pub parameters: HashMap<String, String>,
    pub requested_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task_id: Uuid,
    pub agent_id: String,
    pub status: TaskExecutionStatus,
    pub output: Option<String>,
    pub artifacts: Vec<String>,
    pub validation_results: Option<ValidationResults>,
    pub error: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskExecutionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    AwaitingApproval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResults {
    pub passed: bool,
    pub checks: Vec<ValidationCheck>,
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub details: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_contract_creation() {
        let contract = AgentContract::new(
            "opencode-agent",
            "1.0.0",
            AgentProtocol::Mcp {
                server_url: "http://localhost:8080".to_string(),
            },
        );

        assert_eq!(contract.agent_id, "opencode-agent");
        assert_eq!(contract.version, "1.0.0");
    }

    #[test]
    fn test_agent_contract_builder() {
        let contract = AgentContract::new(
            "test-agent",
            "1.0.0",
            AgentProtocol::Http {
                endpoint: "http://localhost:3000".to_string(),
            },
        )
        .with_capabilities(vec![
            AgentCapability::ExecuteTask,
            AgentCapability::GetContext,
        ])
        .with_authentication(AuthenticationMethod::ApiKey {
            header_name: "X-API-Key".to_string(),
        })
        .with_timeout(60);

        assert_eq!(contract.capabilities.len(), 2);
        assert_eq!(contract.timeout_seconds, 60);
    }

    #[test]
    fn test_capability_support() {
        let contract = AgentContract::new("test", "1.0.0", AgentProtocol::Stdio)
            .with_capabilities(vec![AgentCapability::ExecuteTask]);

        assert!(contract.supports_capability(&AgentCapability::ExecuteTask));
        assert!(!contract.supports_capability(&AgentCapability::GetContext));
    }
}

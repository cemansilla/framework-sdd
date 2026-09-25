pub mod agent_contract;
pub mod model_profile;
pub mod profile_versioning;
pub mod profiles;
pub mod validation;

pub use agent_contract::{
    AgentCapability, AgentContract, AgentProtocol, AuthenticationMethod, OAuth2Flow, RateLimits,
    RequestFormat, ResponseFormat, RetryPolicy, TaskExecutionStatus, TaskRequest, TaskResponse,
    ValidationCheck, ValidationResults,
};
pub use model_profile::{
    ContextFormat, InvocationMechanism, McpCapabilities, ModelCapability, ModelProfile,
    ModelProvider,
};
pub use profile_versioning::{
    ChangeType, ProfileVersion, ProfileVersionManager, VersionChange, VersionError,
};
pub use profiles::{create_opencode_profile, create_warp_profile, get_builtin_profiles};
pub use validation::{
    ErrorSeverity, IntegrationValidator, ValidationError, ValidationResult, ValidationWarning,
};

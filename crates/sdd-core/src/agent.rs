use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub agent_type: AgentType,
    pub capabilities: HashSet<Capability>,
    pub permissions: AgentPermissions,
    pub tools: HashSet<String>,
    pub allowed_task_types: HashSet<TaskType>,
    pub compatible_skills: HashSet<String>,
    pub output_contract: OutputContract,
    pub context_limits: ContextLimits,
    pub approval_required: HashSet<AgentAction>,
    pub status: AgentStatus,
    pub config: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentType {
    Planner,
    Implementer,
    Reviewer,
    Qa,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Capability {
    ReadCode,
    WriteCode,
    RunTests,
    RunBuild,
    ReadDocs,
    WriteDocs,
    AnalyzeAst,
    SearchCode,
    ExecuteCommands,
    CreateBranch,
    CreateCommit,
    CreatePr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPermissions {
    pub read_paths: Vec<String>,
    pub write_paths: Vec<String>,
    pub denied_paths: Vec<String>,
    pub max_file_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskType {
    Implementation,
    Testing,
    Review,
    Documentation,
    Refactoring,
    BugFix,
    Feature,
    Architecture,
    Planning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputContract {
    pub expected_artifacts: Vec<String>,
    pub required_tests: bool,
    pub quality_gates: Vec<QualityGate>,
    pub format: OutputFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QualityGate {
    FmtCheck,
    ClippyCheck,
    TestPass,
    BuildSuccess,
    CoverageThreshold(f32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputFormat {
    Code,
    Documentation,
    TestSuite,
    Review,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLimits {
    pub max_tokens: u32,
    pub max_files: u32,
    pub max_context_depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentAction {
    DeleteFile,
    ModifyConfig,
    MergeBranch,
    DeployArtifact,
    ExternalApiCall,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    Active,
    Inactive,
    Busy { task_id: Uuid },
    Error { message: String },
}

impl Agent {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        agent_type: AgentType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            agent_type,
            capabilities: HashSet::new(),
            permissions: AgentPermissions::default(),
            tools: HashSet::new(),
            allowed_task_types: HashSet::new(),
            compatible_skills: HashSet::new(),
            output_contract: OutputContract::default(),
            context_limits: ContextLimits::default(),
            approval_required: HashSet::new(),
            status: AgentStatus::Active,
            config: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.capabilities.insert(capability);
        self.updated_at = Utc::now();
        self
    }

    pub fn with_tool(mut self, tool: impl Into<String>) -> Self {
        self.tools.insert(tool.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn with_allowed_task_type(mut self, task_type: TaskType) -> Self {
        self.allowed_task_types.insert(task_type);
        self.updated_at = Utc::now();
        self
    }

    pub fn with_compatible_skill(mut self, skill_id: impl Into<String>) -> Self {
        self.compatible_skills.insert(skill_id.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn with_output_contract(mut self, contract: OutputContract) -> Self {
        self.output_contract = contract;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_context_limits(mut self, limits: ContextLimits) -> Self {
        self.context_limits = limits;
        self.updated_at = Utc::now();
        self
    }

    pub fn requires_approval(mut self, action: AgentAction) -> Self {
        self.approval_required.insert(action);
        self.updated_at = Utc::now();
        self
    }

    pub fn can_handle_task_type(&self, task_type: &TaskType) -> bool {
        self.allowed_task_types.is_empty() || self.allowed_task_types.contains(task_type)
    }

    pub fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }

    pub fn can_read_path(&self, path: &str) -> bool {
        if self
            .permissions
            .denied_paths
            .iter()
            .any(|p| path.starts_with(p))
        {
            return false;
        }
        self.permissions.read_paths.is_empty()
            || self
                .permissions
                .read_paths
                .iter()
                .any(|p| path.starts_with(p))
    }

    pub fn can_write_path(&self, path: &str) -> bool {
        if self
            .permissions
            .denied_paths
            .iter()
            .any(|p| path.starts_with(p))
        {
            return false;
        }
        self.permissions.write_paths.is_empty()
            || self
                .permissions
                .write_paths
                .iter()
                .any(|p| path.starts_with(p))
    }

    pub fn is_available(&self) -> bool {
        matches!(self.status, AgentStatus::Active)
    }

    pub fn set_busy(&mut self, task_id: Uuid) {
        self.status = AgentStatus::Busy { task_id };
        self.updated_at = Utc::now();
    }

    pub fn set_available(&mut self) {
        self.status = AgentStatus::Active;
        self.updated_at = Utc::now();
    }
}

impl Default for AgentPermissions {
    fn default() -> Self {
        Self {
            read_paths: vec!["src/".to_string(), "tests/".to_string()],
            write_paths: vec!["src/".to_string(), "tests/".to_string()],
            denied_paths: vec![".git/".to_string(), "target/".to_string()],
            max_file_size: Some(10 * 1024 * 1024),
        }
    }
}

impl Default for OutputContract {
    fn default() -> Self {
        Self {
            expected_artifacts: Vec::new(),
            required_tests: false,
            quality_gates: vec![QualityGate::FmtCheck, QualityGate::ClippyCheck],
            format: OutputFormat::Code,
        }
    }
}

impl Default for ContextLimits {
    fn default() -> Self {
        Self {
            max_tokens: 100_000,
            max_files: 50,
            max_context_depth: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new("planner", "Plans tasks", AgentType::Planner);
        assert_eq!(agent.name, "planner");
        assert_eq!(agent.agent_type, AgentType::Planner);
        assert!(agent.is_available());
    }

    #[test]
    fn test_agent_with_capabilities() {
        let agent = Agent::new("coder", "Writes code", AgentType::Implementer)
            .with_capability(Capability::ReadCode)
            .with_capability(Capability::WriteCode);

        assert!(agent.has_capability(&Capability::ReadCode));
        assert!(agent.has_capability(&Capability::WriteCode));
        assert!(!agent.has_capability(&Capability::RunTests));
    }

    #[test]
    fn test_agent_task_type_handling() {
        let agent = Agent::new("coder", "Writes code", AgentType::Implementer)
            .with_allowed_task_type(TaskType::Implementation)
            .with_allowed_task_type(TaskType::BugFix);

        assert!(agent.can_handle_task_type(&TaskType::Implementation));
        assert!(agent.can_handle_task_type(&TaskType::BugFix));
        assert!(!agent.can_handle_task_type(&TaskType::Review));
    }

    #[test]
    fn test_agent_path_permissions() {
        let agent = Agent::new("coder", "Writes code", AgentType::Implementer);

        assert!(agent.can_read_path("src/main.rs"));
        assert!(agent.can_write_path("src/lib.rs"));
        assert!(!agent.can_read_path(".git/config"));
        assert!(!agent.can_write_path("target/debug"));
    }

    #[test]
    fn test_agent_availability() {
        let mut agent = Agent::new("planner", "Plans tasks", AgentType::Planner);
        assert!(agent.is_available());

        let task_id = Uuid::new_v4();
        agent.set_busy(task_id);
        assert!(!agent.is_available());

        agent.set_available();
        assert!(agent.is_available());
    }

    #[test]
    fn test_approval_required() {
        let agent = Agent::new("coder", "Writes code", AgentType::Implementer)
            .requires_approval(AgentAction::DeleteFile)
            .requires_approval(AgentAction::MergeBranch);

        assert!(agent.approval_required.contains(&AgentAction::DeleteFile));
        assert!(agent.approval_required.contains(&AgentAction::MergeBranch));
        assert!(!agent
            .approval_required
            .contains(&AgentAction::DeployArtifact));
    }
}

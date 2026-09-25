use crate::agent::{Agent, AgentAction, AgentPermissions, Capability};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheck {
    pub agent_id: Uuid,
    pub action: PermissionAction,
    pub granted: bool,
    pub reason: String,
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionAction {
    ReadFile(String),
    WriteFile(String),
    DeleteFile(String),
    ExecuteCommand(String),
    CreateBranch(String),
    CreateCommit,
    CreatePr,
    ModifyConfig(String),
    DeployArtifact(String),
    ExternalApiCall(String),
    Custom(String),
}

pub struct PermissionEngine;

impl PermissionEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn check_permission(&self, agent: &Agent, action: &PermissionAction) -> PermissionCheck {
        let (granted, reason) = self.evaluate(agent, action);

        PermissionCheck {
            agent_id: agent.id,
            action: action.clone(),
            granted,
            reason,
            checked_at: chrono::Utc::now(),
        }
    }

    pub fn check_multiple(
        &self,
        agent: &Agent,
        actions: &[PermissionAction],
    ) -> Vec<PermissionCheck> {
        actions
            .iter()
            .map(|action| self.check_permission(agent, action))
            .collect()
    }

    pub fn requires_approval(&self, agent: &Agent, action: &PermissionAction) -> bool {
        let agent_action = self.to_agent_action(action);
        if let Some(agent_action) = agent_action {
            agent.approval_required.contains(&agent_action)
        } else {
            false
        }
    }

    fn evaluate(&self, agent: &Agent, action: &PermissionAction) -> (bool, String) {
        match action {
            PermissionAction::ReadFile(path) => {
                if self.is_denied(&agent.permissions.denied_paths, path) {
                    return (false, format!("Path '{}' is in denied list", path));
                }
                if self.is_allowed(&agent.permissions.read_paths, path) {
                    return (true, format!("Read access to '{}' granted", path));
                }
                if agent.capabilities.contains(&Capability::ReadCode) {
                    return (true, "Read access via ReadCode capability".to_string());
                }
                (false, format!("No read permission for '{}'", path))
            }

            PermissionAction::WriteFile(path) => {
                if self.is_denied(&agent.permissions.denied_paths, path) {
                    return (false, format!("Path '{}' is in denied list", path));
                }
                if self.is_allowed(&agent.permissions.write_paths, path) {
                    return (true, format!("Write access to '{}' granted", path));
                }
                if agent.capabilities.contains(&Capability::WriteCode) {
                    return (true, "Write access via WriteCode capability".to_string());
                }
                (false, format!("No write permission for '{}'", path))
            }

            PermissionAction::DeleteFile(path) => {
                if self.is_denied(&agent.permissions.denied_paths, path) {
                    return (false, format!("Path '{}' is in denied list", path));
                }
                if agent.approval_required.contains(&AgentAction::DeleteFile) {
                    return (false, "Delete requires approval".to_string());
                }
                if agent.capabilities.contains(&Capability::WriteCode) {
                    return (true, "Delete access via WriteCode capability".to_string());
                }
                (false, "No delete permission".to_string())
            }

            PermissionAction::ExecuteCommand(cmd) => {
                if agent.capabilities.contains(&Capability::ExecuteCommands) {
                    return (
                        true,
                        format!("Execute '{}' via ExecuteCommands capability", cmd),
                    );
                }
                (false, format!("No permission to execute '{}'", cmd))
            }

            PermissionAction::CreateBranch(_) => {
                if agent.capabilities.contains(&Capability::CreateBranch) {
                    return (
                        true,
                        "Branch creation via CreateBranch capability".to_string(),
                    );
                }
                (false, "No permission to create branches".to_string())
            }

            PermissionAction::CreateCommit => {
                if agent.capabilities.contains(&Capability::CreateCommit) {
                    return (
                        true,
                        "Commit creation via CreateCommit capability".to_string(),
                    );
                }
                (false, "No permission to create commits".to_string())
            }

            PermissionAction::CreatePr => {
                if agent.capabilities.contains(&Capability::CreatePr) {
                    return (true, "PR creation via CreatePr capability".to_string());
                }
                (false, "No permission to create PRs".to_string())
            }

            PermissionAction::ModifyConfig(path) => {
                if agent.approval_required.contains(&AgentAction::ModifyConfig) {
                    return (false, "Config modification requires approval".to_string());
                }
                if self.is_allowed(&agent.permissions.write_paths, path) {
                    return (true, format!("Config modification at '{}' allowed", path));
                }
                (
                    false,
                    format!("No permission to modify config at '{}'", path),
                )
            }

            PermissionAction::DeployArtifact(_) => {
                if agent
                    .approval_required
                    .contains(&AgentAction::DeployArtifact)
                {
                    return (false, "Deployment requires approval".to_string());
                }
                (false, "No deployment permission".to_string())
            }

            PermissionAction::ExternalApiCall(_) => {
                if agent
                    .approval_required
                    .contains(&AgentAction::ExternalApiCall)
                {
                    return (false, "External API call requires approval".to_string());
                }
                (false, "No external API permission".to_string())
            }

            PermissionAction::Custom(name) => {
                (false, format!("Custom action '{}' not permitted", name))
            }
        }
    }

    fn is_denied(&self, denied_paths: &[String], path: &str) -> bool {
        denied_paths.iter().any(|p| self.path_matches(path, p))
    }

    fn is_allowed(&self, allowed_paths: &[String], path: &str) -> bool {
        if allowed_paths.is_empty() {
            return true;
        }
        allowed_paths.iter().any(|p| self.path_matches(path, p))
    }

    fn path_matches(&self, path: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return path.starts_with(parts[0]) && path.ends_with(parts[1]);
            }
        }
        path.starts_with(pattern) || path.contains(pattern)
    }

    fn to_agent_action(&self, action: &PermissionAction) -> Option<AgentAction> {
        match action {
            PermissionAction::DeleteFile(_) => Some(AgentAction::DeleteFile),
            PermissionAction::ModifyConfig(_) => Some(AgentAction::ModifyConfig),
            PermissionAction::DeployArtifact(_) => Some(AgentAction::DeployArtifact),
            PermissionAction::ExternalApiCall(_) => Some(AgentAction::ExternalApiCall),
            _ => None,
        }
    }
}

impl Default for PermissionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentPermissions {
    pub fn new() -> Self {
        Self {
            read_paths: Vec::new(),
            write_paths: Vec::new(),
            denied_paths: Vec::new(),
            max_file_size: None,
        }
    }

    pub fn with_read_paths(mut self, paths: Vec<String>) -> Self {
        self.read_paths = paths;
        self
    }

    pub fn with_write_paths(mut self, paths: Vec<String>) -> Self {
        self.write_paths = paths;
        self
    }

    pub fn with_denied_paths(mut self, paths: Vec<String>) -> Self {
        self.denied_paths = paths;
        self
    }

    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = Some(size);
        self
    }

    pub fn deny_path(&mut self, path: &str) {
        self.denied_paths.push(path.to_string());
    }

    pub fn allow_read(&mut self, path: &str) {
        self.read_paths.push(path.to_string());
    }

    pub fn allow_write(&mut self, path: &str) {
        self.write_paths.push(path.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{Agent, AgentType};

    fn create_test_agent() -> Agent {
        let mut agent = Agent::new(
            "test-agent",
            "Test Agent",
            AgentType::Custom("test".to_string()),
        );
        agent.capabilities.insert(Capability::ReadCode);
        agent.capabilities.insert(Capability::WriteCode);
        agent.capabilities.insert(Capability::ExecuteCommands);
        agent.permissions = AgentPermissions::new()
            .with_read_paths(vec!["src/**".to_string()])
            .with_write_paths(vec!["src/**".to_string()])
            .with_denied_paths(vec![".env".to_string(), "secrets/**".to_string()]);
        agent
    }

    #[test]
    fn test_permission_engine_creation() {
        let engine = PermissionEngine::new();
        let _ = engine;
    }

    #[test]
    fn test_read_permission_granted() {
        let engine = PermissionEngine::new();
        let agent = create_test_agent();

        let check = engine.check_permission(
            &agent,
            &PermissionAction::ReadFile("src/main.rs".to_string()),
        );
        assert!(check.granted);
    }

    #[test]
    fn test_read_permission_denied_path() {
        let engine = PermissionEngine::new();
        let agent = create_test_agent();

        let check =
            engine.check_permission(&agent, &PermissionAction::ReadFile(".env".to_string()));
        assert!(!check.granted);
    }

    #[test]
    fn test_write_permission_granted() {
        let engine = PermissionEngine::new();
        let agent = create_test_agent();

        let check = engine.check_permission(
            &agent,
            &PermissionAction::WriteFile("src/lib.rs".to_string()),
        );
        assert!(check.granted);
    }

    #[test]
    fn test_execute_permission() {
        let engine = PermissionEngine::new();
        let agent = create_test_agent();

        let check = engine.check_permission(
            &agent,
            &PermissionAction::ExecuteCommand("cargo test".to_string()),
        );
        assert!(check.granted);
    }

    #[test]
    fn test_requires_approval() {
        let engine = PermissionEngine::new();
        let mut agent = create_test_agent();
        agent.approval_required.insert(AgentAction::DeleteFile);

        assert!(
            engine.requires_approval(&agent, &PermissionAction::DeleteFile("test.rs".to_string()))
        );
        assert!(
            !engine.requires_approval(&agent, &PermissionAction::ReadFile("test.rs".to_string()))
        );
    }

    #[test]
    fn test_check_multiple_permissions() {
        let engine = PermissionEngine::new();
        let agent = create_test_agent();

        let actions = vec![
            PermissionAction::ReadFile("src/main.rs".to_string()),
            PermissionAction::WriteFile("src/lib.rs".to_string()),
            PermissionAction::ReadFile(".env".to_string()),
        ];

        let checks = engine.check_multiple(&agent, &actions);
        assert_eq!(checks.len(), 3);
        assert!(checks[0].granted);
        assert!(checks[1].granted);
        assert!(!checks[2].granted);
    }

    #[test]
    fn test_agent_permissions_builder() {
        let perms = AgentPermissions::new()
            .with_read_paths(vec!["src/**".to_string()])
            .with_write_paths(vec!["src/**".to_string()])
            .with_denied_paths(vec![".env".to_string()])
            .with_max_file_size(1_000_000);

        assert_eq!(perms.read_paths.len(), 1);
        assert_eq!(perms.write_paths.len(), 1);
        assert_eq!(perms.denied_paths.len(), 1);
        assert_eq!(perms.max_file_size, Some(1_000_000));
    }
}

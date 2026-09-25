use crate::agent::{Agent, AgentType, Capability, TaskType};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AgentRegistry {
    agents: HashMap<Uuid, Agent>,
    agents_by_name: HashMap<String, Uuid>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            agents_by_name: HashMap::new(),
        }
    }

    pub fn with_built_in_agents() -> Self {
        let mut registry = Self::new();
        registry.register(Self::create_planner());
        registry.register(Self::create_implementer());
        registry.register(Self::create_reviewer());
        registry.register(Self::create_qa());
        registry
    }

    pub fn register(&mut self, agent: Agent) -> Uuid {
        let id = agent.id;
        let name = agent.name.clone();
        self.agents.insert(id, agent);
        self.agents_by_name.insert(name, id);
        id
    }

    pub fn get(&self, id: Uuid) -> Option<&Agent> {
        self.agents.get(&id)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Agent> {
        self.agents_by_name
            .get(name)
            .and_then(|id| self.agents.get(id))
    }

    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut Agent> {
        self.agents.get_mut(&id)
    }

    pub fn unregister(&mut self, id: Uuid) -> Option<Agent> {
        if let Some(agent) = self.agents.remove(&id) {
            self.agents_by_name.remove(&agent.name);
            Some(agent)
        } else {
            None
        }
    }

    pub fn list(&self) -> Vec<&Agent> {
        self.agents.values().collect()
    }

    pub fn list_by_type(&self, agent_type: &AgentType) -> Vec<&Agent> {
        self.agents
            .values()
            .filter(|a| &a.agent_type == agent_type)
            .collect()
    }

    pub fn list_available(&self) -> Vec<&Agent> {
        self.agents.values().filter(|a| a.is_available()).collect()
    }

    pub fn find_for_task_type(&self, task_type: &TaskType) -> Vec<&Agent> {
        self.agents
            .values()
            .filter(|a| a.is_available() && a.can_handle_task_type(task_type))
            .collect()
    }

    pub fn find_for_capabilities(&self, required: &[Capability]) -> Vec<&Agent> {
        self.agents
            .values()
            .filter(|a| a.is_available() && required.iter().all(|cap| a.has_capability(cap)))
            .collect()
    }

    pub fn count(&self) -> usize {
        self.agents.len()
    }

    fn create_planner() -> Agent {
        Agent::new("planner", "Plans and decomposes tasks", AgentType::Planner)
            .with_capability(Capability::ReadCode)
            .with_capability(Capability::ReadDocs)
            .with_capability(Capability::SearchCode)
            .with_allowed_task_type(TaskType::Planning)
            .with_allowed_task_type(TaskType::Architecture)
    }

    fn create_implementer() -> Agent {
        Agent::new(
            "implementer",
            "Implements code changes",
            AgentType::Implementer,
        )
        .with_capability(Capability::ReadCode)
        .with_capability(Capability::WriteCode)
        .with_capability(Capability::RunTests)
        .with_capability(Capability::RunBuild)
        .with_capability(Capability::SearchCode)
        .with_capability(Capability::ExecuteCommands)
        .with_allowed_task_type(TaskType::Implementation)
        .with_allowed_task_type(TaskType::BugFix)
        .with_allowed_task_type(TaskType::Feature)
        .with_allowed_task_type(TaskType::Refactoring)
    }

    fn create_reviewer() -> Agent {
        Agent::new(
            "reviewer",
            "Reviews code for quality and security",
            AgentType::Reviewer,
        )
        .with_capability(Capability::ReadCode)
        .with_capability(Capability::ReadDocs)
        .with_capability(Capability::RunTests)
        .with_capability(Capability::RunBuild)
        .with_capability(Capability::AnalyzeAst)
        .with_capability(Capability::SearchCode)
        .with_allowed_task_type(TaskType::Review)
    }

    fn create_qa() -> Agent {
        Agent::new("qa", "Generates and runs tests", AgentType::Qa)
            .with_capability(Capability::ReadCode)
            .with_capability(Capability::WriteCode)
            .with_capability(Capability::RunTests)
            .with_capability(Capability::SearchCode)
            .with_allowed_task_type(TaskType::Testing)
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = AgentRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_with_built_in_agents() {
        let registry = AgentRegistry::with_built_in_agents();
        assert_eq!(registry.count(), 4);
        assert!(registry.get_by_name("planner").is_some());
        assert!(registry.get_by_name("implementer").is_some());
        assert!(registry.get_by_name("reviewer").is_some());
        assert!(registry.get_by_name("qa").is_some());
    }

    #[test]
    fn test_register_custom_agent() {
        let mut registry = AgentRegistry::new();
        let agent = Agent::new(
            "custom",
            "Custom agent",
            AgentType::Custom("custom".to_string()),
        );
        let id = registry.register(agent);

        assert_eq!(registry.count(), 1);
        assert!(registry.get(id).is_some());
        assert!(registry.get_by_name("custom").is_some());
    }

    #[test]
    fn test_unregister_agent() {
        let mut registry = AgentRegistry::with_built_in_agents();
        let agent = registry.get_by_name("planner").unwrap();
        let id = agent.id;

        registry.unregister(id);
        assert_eq!(registry.count(), 3);
        assert!(registry.get_by_name("planner").is_none());
    }

    #[test]
    fn test_find_for_task_type() {
        let registry = AgentRegistry::with_built_in_agents();

        let implementers = registry.find_for_task_type(&TaskType::Implementation);
        assert!(!implementers.is_empty());
        assert!(implementers.iter().any(|a| a.name == "implementer"));

        let testers = registry.find_for_task_type(&TaskType::Testing);
        assert!(!testers.is_empty());
        assert!(testers.iter().any(|a| a.name == "qa"));
    }

    #[test]
    fn test_find_for_capabilities() {
        let registry = AgentRegistry::with_built_in_agents();

        let agents = registry.find_for_capabilities(&[Capability::WriteCode]);
        assert!(agents.iter().any(|a| a.name == "implementer"));
    }

    #[test]
    fn test_list_available() {
        let mut registry = AgentRegistry::with_built_in_agents();
        let initial = registry.list_available().len();

        let planner = registry.get_by_name("planner").unwrap();
        let id = planner.id;
        registry.get_mut(id).unwrap().set_busy(Uuid::new_v4());

        assert_eq!(registry.list_available().len(), initial - 1);
    }
}

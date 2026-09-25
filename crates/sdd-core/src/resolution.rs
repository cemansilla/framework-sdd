use crate::agent::{Agent, Capability, TaskType};
use crate::agent_registry::AgentRegistry;
use crate::skill_registry::SkillRegistry;
use crate::task::Task;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ResolutionResult {
    pub task_id: Uuid,
    pub selected_agent: Option<Uuid>,
    pub selected_skills: Vec<Uuid>,
    pub reasoning: ResolutionReasoning,
}

#[derive(Debug, Clone, Default)]
pub struct ResolutionReasoning {
    pub agent_candidates: Vec<Uuid>,
    pub skill_candidates: Vec<Uuid>,
    pub excluded_agents: Vec<(Uuid, String)>,
    pub excluded_skills: Vec<(Uuid, String)>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResolutionRequest {
    pub task: Task,
    pub required_capabilities: Vec<Capability>,
    pub preferred_skills: Vec<String>,
    pub language: Option<String>,
    pub phase: Option<String>,
}

#[derive(Debug)]
pub struct AgentResolver<'a> {
    registry: &'a AgentRegistry,
}

impl<'a> AgentResolver<'a> {
    pub fn new(registry: &'a AgentRegistry) -> Self {
        Self { registry }
    }

    pub fn resolve(&self, request: &ResolutionRequest) -> ResolutionResult {
        let mut reasoning = ResolutionReasoning::default();

        let task_type = self.infer_task_type(&request.task);
        let candidates = self.registry.find_for_task_type(&task_type);

        reasoning.agent_candidates = candidates.iter().map(|a| a.id).collect();

        let mut filtered: Vec<&Agent> = candidates
            .into_iter()
            .filter(|a| self.meets_capabilities(a, &request.required_capabilities))
            .collect();

        let excluded_by_caps: Vec<_> = self
            .registry
            .list()
            .into_iter()
            .filter(|a| !self.meets_capabilities(a, &request.required_capabilities))
            .map(|a| (a.id, "missing required capabilities".to_string()))
            .collect();
        reasoning.excluded_agents.extend(excluded_by_caps);

        filtered.retain(|a| {
            if a.is_available() {
                true
            } else {
                reasoning
                    .excluded_agents
                    .push((a.id, "agent is busy".to_string()));
                false
            }
        });

        let selected = if filtered.is_empty() {
            reasoning
                .warnings
                .push("No suitable agent found for task".to_string());
            None
        } else {
            filtered.sort_by(|a, b| {
                let score_a = self.calculate_agent_score(a, request);
                let score_b = self.calculate_agent_score(b, request);
                score_b.cmp(&score_a)
            });
            Some(filtered[0].id)
        };

        ResolutionResult {
            task_id: request.task.id,
            selected_agent: selected,
            selected_skills: Vec::new(),
            reasoning,
        }
    }

    fn infer_task_type(&self, task: &Task) -> TaskType {
        let title_lower = task.title.to_lowercase();
        let desc_lower = task.description.to_lowercase();
        let combined = format!("{} {}", title_lower, desc_lower);

        if combined.contains("test") {
            TaskType::Testing
        } else if combined.contains("review") || combined.contains("audit") {
            TaskType::Review
        } else if combined.contains("fix") || combined.contains("bug") {
            TaskType::BugFix
        } else if combined.contains("refactor") {
            TaskType::Refactoring
        } else if combined.contains("document") || combined.contains("doc") {
            TaskType::Documentation
        } else if combined.contains("plan") || combined.contains("design") {
            TaskType::Planning
        } else if combined.contains("architect") {
            TaskType::Architecture
        } else {
            TaskType::Implementation
        }
    }

    fn meets_capabilities(&self, agent: &Agent, required: &[Capability]) -> bool {
        required.iter().all(|cap| agent.has_capability(cap))
    }

    fn calculate_agent_score(&self, agent: &Agent, request: &ResolutionRequest) -> u32 {
        let mut score = 0;

        let matching_skills = agent
            .compatible_skills
            .iter()
            .filter(|s| request.preferred_skills.contains(s))
            .count();
        score += matching_skills as u32 * 10;

        score += agent.capabilities.len() as u32;

        score
    }
}

#[derive(Debug)]
pub struct SkillResolver<'a> {
    registry: &'a SkillRegistry,
}

impl<'a> SkillResolver<'a> {
    pub fn new(registry: &'a SkillRegistry) -> Self {
        Self { registry }
    }

    pub fn resolve(&self, request: &ResolutionRequest) -> Vec<Uuid> {
        let mut selected = Vec::new();
        let mut seen = HashSet::new();

        let task_type = self.infer_task_type_string(&request.task);

        let task_skills = self.registry.find_for_task_type(&task_type);
        for skill in task_skills {
            if seen.insert(skill.id) {
                selected.push(skill.id);
            }
        }

        if let Some(lang) = &request.language {
            let lang_skills = self.registry.find_for_language(lang);
            for skill in lang_skills {
                if seen.insert(skill.id) {
                    selected.push(skill.id);
                }
            }
        }

        if let Some(phase) = &request.phase {
            let phase_skills = self.registry.find_for_phase(phase);
            for skill in phase_skills {
                if seen.insert(skill.id) {
                    selected.push(skill.id);
                }
            }
        }

        if !request.preferred_skills.is_empty() {
            let tag_skills = self.registry.find_by_tags(
                &request
                    .preferred_skills
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>(),
            );
            for skill in tag_skills {
                if seen.insert(skill.id) {
                    selected.push(skill.id);
                }
            }
        }

        selected
    }

    fn infer_task_type_string(&self, task: &Task) -> String {
        let title_lower = task.title.to_lowercase();
        let desc_lower = task.description.to_lowercase();
        let combined = format!("{} {}", title_lower, desc_lower);

        if combined.contains("test") {
            "testing".to_string()
        } else if combined.contains("review") || combined.contains("audit") {
            "review".to_string()
        } else if combined.contains("architect") || combined.contains("design") {
            "architecture".to_string()
        } else if combined.contains("plan") {
            "planning".to_string()
        } else {
            "implementation".to_string()
        }
    }
}

pub struct TaskResolver<'a> {
    agent_registry: &'a AgentRegistry,
    skill_registry: &'a SkillRegistry,
}

impl<'a> TaskResolver<'a> {
    pub fn new(agent_registry: &'a AgentRegistry, skill_registry: &'a SkillRegistry) -> Self {
        Self {
            agent_registry,
            skill_registry,
        }
    }

    pub fn resolve(&self, request: &ResolutionRequest) -> ResolutionResult {
        let agent_resolver = AgentResolver::new(self.agent_registry);
        let skill_resolver = SkillResolver::new(self.skill_registry);

        let mut result = agent_resolver.resolve(request);
        result.selected_skills = skill_resolver.resolve(request);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Task;

    fn create_test_registries() -> (AgentRegistry, SkillRegistry) {
        (
            AgentRegistry::with_built_in_agents(),
            SkillRegistry::with_built_in_skills(),
        )
    }

    #[test]
    fn test_agent_resolver_implementation_task() {
        let (agent_registry, _) = create_test_registries();
        let resolver = AgentResolver::new(&agent_registry);

        let task = Task::new("TASK-001", "Implement user login", "Add login feature");
        let request = ResolutionRequest {
            task,
            required_capabilities: vec![],
            preferred_skills: vec![],
            language: Some("rust".to_string()),
            phase: None,
        };

        let result = resolver.resolve(&request);
        assert!(result.selected_agent.is_some());

        let agent = agent_registry.get(result.selected_agent.unwrap()).unwrap();
        assert_eq!(agent.name, "implementer");
    }

    #[test]
    fn test_agent_resolver_testing_task() {
        let (agent_registry, _) = create_test_registries();
        let resolver = AgentResolver::new(&agent_registry);

        let task = Task::new("TASK-002", "Write unit tests", "Add tests for module");
        let request = ResolutionRequest {
            task,
            required_capabilities: vec![],
            preferred_skills: vec![],
            language: Some("rust".to_string()),
            phase: None,
        };

        let result = resolver.resolve(&request);
        assert!(result.selected_agent.is_some());

        let agent = agent_registry.get(result.selected_agent.unwrap()).unwrap();
        assert_eq!(agent.name, "qa");
    }

    #[test]
    fn test_agent_resolver_with_capabilities() {
        let (agent_registry, _) = create_test_registries();
        let resolver = AgentResolver::new(&agent_registry);

        let task = Task::new("TASK-003", "Implement feature", "Write code");
        let request = ResolutionRequest {
            task,
            required_capabilities: vec![Capability::WriteCode],
            preferred_skills: vec![],
            language: None,
            phase: None,
        };

        let result = resolver.resolve(&request);
        assert!(result.selected_agent.is_some());
    }

    #[test]
    fn test_skill_resolver_testing() {
        let (_, skill_registry) = create_test_registries();
        let resolver = SkillResolver::new(&skill_registry);

        let task = Task::new("TASK-001", "Write tests", "Add unit tests");
        let request = ResolutionRequest {
            task,
            required_capabilities: vec![],
            preferred_skills: vec![],
            language: Some("rust".to_string()),
            phase: Some("testing".to_string()),
        };

        let skills = resolver.resolve(&request);
        assert!(!skills.is_empty());
    }

    #[test]
    fn test_skill_resolver_with_language() {
        let (_, skill_registry) = create_test_registries();
        let resolver = SkillResolver::new(&skill_registry);

        let task = Task::new("TASK-001", "Implement feature", "Write code");
        let request = ResolutionRequest {
            task,
            required_capabilities: vec![],
            preferred_skills: vec![],
            language: Some("rust".to_string()),
            phase: None,
        };

        let skills = resolver.resolve(&request);
        assert!(!skills.is_empty());
    }

    #[test]
    fn test_task_resolver_full() {
        let (agent_registry, skill_registry) = create_test_registries();
        let resolver = TaskResolver::new(&agent_registry, &skill_registry);

        let task = Task::new("TASK-001", "Implement feature", "Write Rust code");
        let request = ResolutionRequest {
            task,
            required_capabilities: vec![Capability::WriteCode],
            preferred_skills: vec!["quality".to_string()],
            language: Some("rust".to_string()),
            phase: Some("implementation".to_string()),
        };

        let result = resolver.resolve(&request);
        assert!(result.selected_agent.is_some());
        assert!(!result.selected_skills.is_empty());
    }
}

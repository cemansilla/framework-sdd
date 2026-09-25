use crate::skill::{
    InputType, OutputType, Skill, SkillApplicability, SkillExample, SkillInput, SkillOutput,
    SkillType,
};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SkillRegistry {
    skills: HashMap<Uuid, Skill>,
    skills_by_id: HashMap<String, Uuid>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            skills_by_id: HashMap::new(),
        }
    }

    pub fn with_built_in_skills() -> Self {
        let mut registry = Self::new();
        registry.register(Self::create_rust_testing_skill());
        registry.register(Self::create_rust_quality_skill());
        registry.register(Self::create_conventional_commits_skill());
        registry.register(Self::create_hexagonal_architecture_skill());
        registry.register(Self::create_context_engineering_skill());
        registry
    }

    pub fn register(&mut self, skill: Skill) -> Uuid {
        let id = skill.id;
        let skill_id = skill.skill_id.clone();
        self.skills.insert(id, skill);
        self.skills_by_id.insert(skill_id, id);
        id
    }

    pub fn get(&self, id: Uuid) -> Option<&Skill> {
        self.skills.get(&id)
    }

    pub fn get_by_skill_id(&self, skill_id: &str) -> Option<&Skill> {
        self.skills_by_id
            .get(skill_id)
            .and_then(|id| self.skills.get(id))
    }

    pub fn unregister(&mut self, id: Uuid) -> Option<Skill> {
        if let Some(skill) = self.skills.remove(&id) {
            self.skills_by_id.remove(&skill.skill_id);
            Some(skill)
        } else {
            None
        }
    }

    pub fn list(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }

    pub fn list_active(&self) -> Vec<&Skill> {
        self.skills.values().filter(|s| s.is_active()).collect()
    }

    pub fn list_by_type(&self, skill_type: &SkillType) -> Vec<&Skill> {
        self.skills
            .values()
            .filter(|s| &s.skill_type == skill_type)
            .collect()
    }

    pub fn find_for_task_type(&self, task_type: &str) -> Vec<&Skill> {
        self.skills
            .values()
            .filter(|s| s.is_active() && s.is_applicable_to_task_type(task_type))
            .collect()
    }

    pub fn find_for_language(&self, language: &str) -> Vec<&Skill> {
        self.skills
            .values()
            .filter(|s| s.is_active() && s.is_applicable_to_language(language))
            .collect()
    }

    pub fn find_for_phase(&self, phase: &str) -> Vec<&Skill> {
        self.skills
            .values()
            .filter(|s| s.is_active() && s.is_applicable_to_phase(phase))
            .collect()
    }

    pub fn find_by_tags(&self, tags: &[&str]) -> Vec<&Skill> {
        self.skills
            .values()
            .filter(|s| s.is_active() && tags.iter().any(|tag| s.has_tag(tag)))
            .collect()
    }

    pub fn resolve_dependencies(
        &self,
        skill_id: &str,
    ) -> Result<Vec<String>, SkillResolutionError> {
        let skill = self
            .get_by_skill_id(skill_id)
            .ok_or_else(|| SkillResolutionError::NotFound(skill_id.to_string()))?;

        let mut resolved = Vec::new();
        let mut visited = HashSet::new();
        self.collect_dependencies(skill, &mut resolved, &mut visited)?;
        Ok(resolved)
    }

    fn collect_dependencies(
        &self,
        skill: &Skill,
        resolved: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) -> Result<(), SkillResolutionError> {
        for dep_id in &skill.dependencies {
            if visited.contains(dep_id) {
                continue;
            }

            let dep_skill = self.get_by_skill_id(dep_id).ok_or_else(|| {
                SkillResolutionError::DependencyNotFound {
                    skill: skill.skill_id.clone(),
                    dependency: dep_id.clone(),
                }
            })?;

            visited.insert(dep_id.clone());
            self.collect_dependencies(dep_skill, resolved, visited)?;
            resolved.push(dep_id.clone());
        }
        Ok(())
    }

    pub fn count(&self) -> usize {
        self.skills.len()
    }

    fn create_rust_testing_skill() -> Skill {
        let mut applicability = SkillApplicability::default();
        applicability.task_types.insert("testing".to_string());
        applicability.languages.insert("rust".to_string());
        applicability.phases.insert("testing".to_string());
        applicability.tags.insert("testing".to_string());
        applicability.tags.insert("quality".to_string());

        Skill::new(
            "rust-testing",
            "Rust Testing Best Practices",
            "1.0.0",
            "Guidelines for writing effective Rust tests including unit, integration, and property-based testing.",
        )
        .with_applicability(applicability)
        .with_instructions(RUST_TESTING_INSTRUCTIONS)
        .add_input(SkillInput::new("module_path", "Path to module being tested", InputType::FilePath))
        .add_expected_output(SkillOutput::new("test_file", "Generated test file", OutputType::Code))
        .add_example(
            SkillExample::new("Unit test example", "Generate unit tests for a module")
                .with_input("module_path", "src/core/project.rs")
                .with_expected_output("tests/project_test.rs"),
        )
    }

    fn create_rust_quality_skill() -> Skill {
        let mut applicability = SkillApplicability::default();
        applicability.task_types.insert("review".to_string());
        applicability
            .task_types
            .insert("implementation".to_string());
        applicability.languages.insert("rust".to_string());
        applicability.tags.insert("quality".to_string());
        applicability.tags.insert("linting".to_string());

        Skill::new(
            "rust-quality",
            "Rust Quality Gates",
            "1.0.0",
            "Quality gates for Rust code including fmt, clippy, and best practices.",
        )
        .with_applicability(applicability)
        .with_instructions(RUST_QUALITY_INSTRUCTIONS)
        .add_required_tool("cargo")
        .add_expected_output(SkillOutput::new(
            "quality_report",
            "Quality assessment",
            OutputType::Report,
        ))
    }

    fn create_conventional_commits_skill() -> Skill {
        let mut applicability = SkillApplicability::default();
        applicability.tags.insert("commits".to_string());
        applicability.tags.insert("conventions".to_string());

        Skill::new(
            "conventional-commits",
            "Conventional Commits",
            "1.0.0",
            "Guidelines for writing conventional commit messages.",
        )
        .with_applicability(applicability)
        .with_instructions(CONVENTIONAL_COMMITS_INSTRUCTIONS)
        .add_expected_output(SkillOutput::new(
            "commit_message",
            "Formatted commit message",
            OutputType::Documentation,
        ))
    }

    fn create_hexagonal_architecture_skill() -> Skill {
        let mut applicability = SkillApplicability::default();
        applicability.task_types.insert("architecture".to_string());
        applicability
            .task_types
            .insert("implementation".to_string());
        applicability.tags.insert("architecture".to_string());
        applicability.tags.insert("design".to_string());

        Skill::new(
            "hexagonal-architecture",
            "Hexagonal Architecture Patterns",
            "1.0.0",
            "Guidelines for implementing hexagonal architecture (ports and adapters).",
        )
        .with_applicability(applicability)
        .with_instructions(HEXAGONAL_ARCH_INSTRUCTIONS)
        .add_expected_output(SkillOutput::new(
            "architecture_doc",
            "Architecture documentation",
            OutputType::Documentation,
        ))
    }

    fn create_context_engineering_skill() -> Skill {
        let mut applicability = SkillApplicability::default();
        applicability.phases.insert("planning".to_string());
        applicability.phases.insert("implementation".to_string());
        applicability.tags.insert("context".to_string());
        applicability.tags.insert("engineering".to_string());

        Skill::new(
            "context-engineering",
            "Context Engineering",
            "1.0.0",
            "Best practices for building effective context bundles for AI agents.",
        )
        .with_applicability(applicability)
        .with_instructions(CONTEXT_ENGINEERING_INSTRUCTIONS)
        .add_expected_output(SkillOutput::new(
            "context_bundle",
            "Optimized context bundle",
            OutputType::Analysis,
        ))
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SkillResolutionError {
    #[error("skill not found: {0}")]
    NotFound(String),
    #[error("dependency not found: {dependency} required by {skill}")]
    DependencyNotFound { skill: String, dependency: String },
    #[error("circular dependency detected: {0}")]
    CircularDependency(String),
}

const RUST_TESTING_INSTRUCTIONS: &str = r#"# Rust Testing Best Practices

## Unit Tests
- Place unit tests in the same file as the code being tested
- Use `#[cfg(test)]` module attribute
- Test public API, not implementation details
- Use descriptive test names: `test_<what>_<condition>_<expected>`

## Integration Tests
- Place in `tests/` directory
- Test public API from external perspective
- Use fixtures and helpers for common setup

## Property-Based Testing
- Use `proptest` or `quickcheck` for property-based tests
- Define invariants that must hold for all inputs

## Test Organization
- Group related tests in modules
- Use `#[test]` attribute for each test
- Use `#[should_panic]` for expected panics
- Use `#[ignore]` for slow tests

## Assertions
- Use `assert!`, `assert_eq!`, `assert_ne!`
- Provide meaningful error messages
- Use custom assertion helpers for complex checks
"#;

const RUST_QUALITY_INSTRUCTIONS: &str = r#"# Rust Quality Gates

## Required Checks
1. `cargo fmt --check` - Code formatting
2. `cargo clippy -- -D warnings` - Linting
3. `cargo test` - All tests pass
4. `cargo build` - Successful compilation

## Best Practices
- No `unwrap()` in production code
- Use `?` operator for error propagation
- Document public APIs with `///` comments
- Keep functions small and focused
- Use meaningful variable names

## Common Clippy Lints
- `clippy::pedantic` - Additional pedantic lints
- `clippy::nursery` - Experimental lints
- `clippy::cargo` - Cargo-specific lints
"#;

const CONVENTIONAL_COMMITS_INSTRUCTIONS: &str = r#"# Conventional Commits

## Format
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

## Types
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting (no code change)
- `refactor`: Code change (no feature/fix)
- `test`: Adding/modifying tests
- `chore`: Maintenance

## Rules
- Use imperative mood in description
- Don't capitalize first letter
- No period at the end
- Scope is optional but recommended
"#;

const HEXAGONAL_ARCH_INSTRUCTIONS: &str = r#"# Hexagonal Architecture

## Core Principles
1. Business logic at the center
2. Ports define interfaces
3. Adapters implement ports
4. Dependencies point inward

## Structure
```
src/
├── domain/       # Core business logic
│   ├── model/    # Entities and value objects
│   ├── port/     # Input/output ports
│   └── service/  # Domain services
├── application/  # Use cases
└── adapter/      # External implementations
    ├── inbound/  # Driving adapters (API, CLI)
    └── outbound/ # Driven adapters (DB, API clients)
```

## Rules
- Domain has NO external dependencies
- Ports are traits in domain
- Adapters implement ports
- Application orchestrates use cases
"#;

const CONTEXT_ENGINEERING_INSTRUCTIONS: &str = r#"# Context Engineering

## Principles
1. Minimal sufficient context
2. Prioritize by relevance
3. Track token budget
4. Include traceability

## Context Bundle Structure
- P0: Task, acceptance criteria, constraints
- P1: Related requirements, decisions, dependencies
- P2: Documentation, previous results
- P3: Additional requested context

## Best Practices
- Start with task definition
- Add only necessary context
- Remove redundant information
- Track what was included/excluded
- Respect token limits
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = SkillRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_with_built_in_skills() {
        let registry = SkillRegistry::with_built_in_skills();
        assert_eq!(registry.count(), 5);
        assert!(registry.get_by_skill_id("rust-testing").is_some());
        assert!(registry.get_by_skill_id("rust-quality").is_some());
        assert!(registry.get_by_skill_id("conventional-commits").is_some());
    }

    #[test]
    fn test_register_custom_skill() {
        let mut registry = SkillRegistry::new();
        let skill = Skill::new("custom-skill", "Custom Skill", "1.0.0", "A custom skill")
            .with_skill_type(SkillType::Project);
        let id = registry.register(skill);

        assert_eq!(registry.count(), 1);
        assert!(registry.get(id).is_some());
        assert!(registry.get_by_skill_id("custom-skill").is_some());
    }

    #[test]
    fn test_find_for_task_type() {
        let registry = SkillRegistry::with_built_in_skills();

        let testing_skills = registry.find_for_task_type("testing");
        assert!(!testing_skills.is_empty());
        assert!(testing_skills.iter().any(|s| s.skill_id == "rust-testing"));
    }

    #[test]
    fn test_find_for_language() {
        let registry = SkillRegistry::with_built_in_skills();

        let rust_skills = registry.find_for_language("rust");
        assert!(!rust_skills.is_empty());
    }

    #[test]
    fn test_find_by_tags() {
        let registry = SkillRegistry::with_built_in_skills();

        let quality_skills = registry.find_by_tags(&["quality"]);
        assert!(!quality_skills.is_empty());
    }

    #[test]
    fn test_resolve_dependencies() {
        let mut registry = SkillRegistry::new();

        let base_skill = Skill::new("base", "Base Skill", "1.0.0", "Base skill");
        registry.register(base_skill);

        let dependent_skill =
            Skill::new("dependent", "Dependent Skill", "1.0.0", "Depends on base")
                .add_dependency("base");
        registry.register(dependent_skill);

        let deps = registry.resolve_dependencies("dependent").unwrap();
        assert_eq!(deps, vec!["base".to_string()]);
    }

    #[test]
    fn test_resolve_dependencies_not_found() {
        let registry = SkillRegistry::new();
        let result = registry.resolve_dependencies("nonexistent");
        assert!(result.is_err());
    }
}

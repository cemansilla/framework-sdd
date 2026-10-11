use std::path::PathBuf;

pub fn project_templates() -> Vec<(PathBuf, &'static str)> {
    vec![
        (PathBuf::from("AGENTS.md"), AGENTS_TEMPLATE),
        (PathBuf::from(".gitignore"), GITIGNORE_TEMPLATE),
        (
            PathBuf::from(".opencode/agents/orchestrator.md"),
            ORCHESTRATOR_TEMPLATE,
        ),
        (
            PathBuf::from(".opencode/agents/architect.md"),
            ARCHITECT_TEMPLATE,
        ),
        (
            PathBuf::from(".opencode/agents/planner.md"),
            PLANNER_TEMPLATE,
        ),
        (PathBuf::from(".opencode/agents/coder.md"), CODER_TEMPLATE),
        (PathBuf::from(".opencode/agents/tester.md"), TESTER_TEMPLATE),
        (
            PathBuf::from(".opencode/agents/reviewer.md"),
            REVIEWER_TEMPLATE,
        ),
        (
            PathBuf::from(".opencode/skills/branching-sdd/SKILL.md"),
            BRANCHING_SDD_TEMPLATE,
        ),
        (
            PathBuf::from(".opencode/skills/cargo-workspace/SKILL.md"),
            CARGO_WORKSPACE_TEMPLATE,
        ),
        (
            PathBuf::from(".opencode/skills/conventional-commits/SKILL.md"),
            CONVENTIONAL_COMMITS_TEMPLATE,
        ),
        (
            PathBuf::from(".opencode/skills/rust-quality/SKILL.md"),
            RUST_QUALITY_TEMPLATE,
        ),
        (
            PathBuf::from(".opencode/skills/sdd-task-workflow/SKILL.md"),
            SDD_TASK_WORKFLOW_TEMPLATE,
        ),
        (
            PathBuf::from(".opencode/commands/plan.md"),
            PLAN_COMMAND_TEMPLATE,
        ),
        (
            PathBuf::from(".opencode/commands/tasks.md"),
            TASKS_COMMAND_TEMPLATE,
        ),
        (
            PathBuf::from(".opencode/commands/implement.md"),
            IMPLEMENT_COMMAND_TEMPLATE,
        ),
        (
            PathBuf::from(".opencode/commands/test.md"),
            TEST_COMMAND_TEMPLATE,
        ),
        (
            PathBuf::from(".opencode/commands/review.md"),
            REVIEW_COMMAND_TEMPLATE,
        ),
        (
            PathBuf::from(".opencode/commands/orchestrate.md"),
            ORCHESTRATE_COMMAND_TEMPLATE,
        ),
    ]
}

const AGENTS_TEMPLATE: &str = include_str!("../templates/AGENTS.md");
const GITIGNORE_TEMPLATE: &str = include_str!("../templates/.gitignore");
const ORCHESTRATOR_TEMPLATE: &str = include_str!("../templates/.opencode/agents/orchestrator.md");
const ARCHITECT_TEMPLATE: &str = include_str!("../templates/.opencode/agents/architect.md");
const PLANNER_TEMPLATE: &str = include_str!("../templates/.opencode/agents/planner.md");
const CODER_TEMPLATE: &str = include_str!("../templates/.opencode/agents/coder.md");
const TESTER_TEMPLATE: &str = include_str!("../templates/.opencode/agents/tester.md");
const REVIEWER_TEMPLATE: &str = include_str!("../templates/.opencode/agents/reviewer.md");
const BRANCHING_SDD_TEMPLATE: &str =
    include_str!("../templates/.opencode/skills/branching-sdd/SKILL.md");
const CARGO_WORKSPACE_TEMPLATE: &str =
    include_str!("../templates/.opencode/skills/cargo-workspace/SKILL.md");
const CONVENTIONAL_COMMITS_TEMPLATE: &str =
    include_str!("../templates/.opencode/skills/conventional-commits/SKILL.md");
const RUST_QUALITY_TEMPLATE: &str =
    include_str!("../templates/.opencode/skills/rust-quality/SKILL.md");
const SDD_TASK_WORKFLOW_TEMPLATE: &str =
    include_str!("../templates/.opencode/skills/sdd-task-workflow/SKILL.md");
const PLAN_COMMAND_TEMPLATE: &str = include_str!("../templates/.opencode/commands/plan.md");
const TASKS_COMMAND_TEMPLATE: &str = include_str!("../templates/.opencode/commands/tasks.md");
const IMPLEMENT_COMMAND_TEMPLATE: &str =
    include_str!("../templates/.opencode/commands/implement.md");
const TEST_COMMAND_TEMPLATE: &str = include_str!("../templates/.opencode/commands/test.md");
const REVIEW_COMMAND_TEMPLATE: &str = include_str!("../templates/.opencode/commands/review.md");
const ORCHESTRATE_COMMAND_TEMPLATE: &str =
    include_str!("../templates/.opencode/commands/orchestrate.md");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_templates_not_empty() {
        let templates = project_templates();
        assert!(!templates.is_empty());
    }

    #[test]
    fn test_project_templates_contains_agents_md() {
        let templates = project_templates();
        assert!(templates
            .iter()
            .any(|(path, _)| path == std::path::Path::new("AGENTS.md")));
    }

    #[test]
    fn test_project_templates_contains_gitignore() {
        let templates = project_templates();
        assert!(templates
            .iter()
            .any(|(path, _)| path == std::path::Path::new(".gitignore")));
    }

    #[test]
    fn test_project_templates_contains_opencode_agents() {
        let templates = project_templates();
        assert!(templates
            .iter()
            .any(|(path, _)| path.to_string_lossy().contains(".opencode/agents")));
    }

    #[test]
    fn test_project_templates_contains_opencode_skills() {
        let templates = project_templates();
        assert!(templates
            .iter()
            .any(|(path, _)| path.to_string_lossy().contains(".opencode/skills")));
    }

    #[test]
    fn test_project_templates_contains_opencode_commands() {
        let templates = project_templates();
        assert!(templates
            .iter()
            .any(|(path, _)| path.to_string_lossy().contains(".opencode/commands")));
    }

    #[test]
    fn test_agents_template_has_content() {
        assert!(AGENTS_TEMPLATE.contains("SDD AI Harness"));
    }

    #[test]
    fn test_gitignore_template_has_content() {
        assert!(GITIGNORE_TEMPLATE.contains("/target"));
    }
}

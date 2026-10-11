use std::path::PathBuf;

const TEMPLATE_FILES: &[(&str, &str)] = &[
    ("AGENTS.md", "AGENTS.md"),
    (".gitignore", ".gitignore"),
    (".opencode/agents/orchestrator.md", "orchestrator.md"),
    (".opencode/agents/architect.md", "architect.md"),
    (".opencode/agents/planner.md", "planner.md"),
    (".opencode/agents/coder.md", "coder.md"),
    (".opencode/agents/tester.md", "tester.md"),
    (".opencode/agents/reviewer.md", "reviewer.md"),
    (
        ".opencode/skills/branching-sdd/SKILL.md",
        "branching-sdd/SKILL.md",
    ),
    (
        ".opencode/skills/cargo-workspace/SKILL.md",
        "cargo-workspace/SKILL.md",
    ),
    (
        ".opencode/skills/conventional-commits/SKILL.md",
        "conventional-commits/SKILL.md",
    ),
    (
        ".opencode/skills/rust-quality/SKILL.md",
        "rust-quality/SKILL.md",
    ),
    (
        ".opencode/skills/sdd-task-workflow/SKILL.md",
        "sdd-task-workflow/SKILL.md",
    ),
    (".opencode/commands/plan.md", "plan.md"),
    (".opencode/commands/tasks.md", "tasks.md"),
    (".opencode/commands/implement.md", "implement.md"),
    (".opencode/commands/test.md", "test.md"),
    (".opencode/commands/review.md", "review.md"),
    (".opencode/commands/orchestrate.md", "orchestrate.md"),
];

pub fn project_templates() -> Vec<(PathBuf, &'static str)> {
    TEMPLATE_FILES
        .iter()
        .map(|(relative, include_path)| {
            let content = template_content(include_path);
            (PathBuf::from(*relative), content)
        })
        .collect()
}

fn template_content(path: &str) -> &'static str {
    match path {
        "AGENTS.md" => include_str!("../templates/AGENTS.md"),
        ".gitignore" => include_str!("../templates/.gitignore"),
        "orchestrator.md" => include_str!("../templates/.opencode/agents/orchestrator.md"),
        "architect.md" => include_str!("../templates/.opencode/agents/architect.md"),
        "planner.md" => include_str!("../templates/.opencode/agents/planner.md"),
        "coder.md" => include_str!("../templates/.opencode/agents/coder.md"),
        "tester.md" => include_str!("../templates/.opencode/agents/tester.md"),
        "reviewer.md" => include_str!("../templates/.opencode/agents/reviewer.md"),
        "branching-sdd/SKILL.md" => {
            include_str!("../templates/.opencode/skills/branching-sdd/SKILL.md")
        }
        "cargo-workspace/SKILL.md" => {
            include_str!("../templates/.opencode/skills/cargo-workspace/SKILL.md")
        }
        "conventional-commits/SKILL.md" => {
            include_str!("../templates/.opencode/skills/conventional-commits/SKILL.md")
        }
        "rust-quality/SKILL.md" => {
            include_str!("../templates/.opencode/skills/rust-quality/SKILL.md")
        }
        "sdd-task-workflow/SKILL.md" => {
            include_str!("../templates/.opencode/skills/sdd-task-workflow/SKILL.md")
        }
        "plan.md" => include_str!("../templates/.opencode/commands/plan.md"),
        "tasks.md" => include_str!("../templates/.opencode/commands/tasks.md"),
        "implement.md" => include_str!("../templates/.opencode/commands/implement.md"),
        "test.md" => include_str!("../templates/.opencode/commands/test.md"),
        "review.md" => include_str!("../templates/.opencode/commands/review.md"),
        "orchestrate.md" => include_str!("../templates/.opencode/commands/orchestrate.md"),
        _ => panic!("unknown template: {path}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_templates_not_empty() {
        assert!(!project_templates().is_empty());
    }

    #[test]
    fn test_project_templates_contains_agents_md() {
        assert!(project_templates()
            .iter()
            .any(|(path, _)| path == std::path::Path::new("AGENTS.md")));
    }

    #[test]
    fn test_project_templates_contains_gitignore() {
        assert!(project_templates()
            .iter()
            .any(|(path, _)| path == std::path::Path::new(".gitignore")));
    }

    #[test]
    fn test_project_templates_contains_opencode_agents() {
        assert!(project_templates()
            .iter()
            .any(|(path, _)| path.to_string_lossy().contains(".opencode/agents")));
    }

    #[test]
    fn test_project_templates_contains_opencode_skills() {
        assert!(project_templates()
            .iter()
            .any(|(path, _)| path.to_string_lossy().contains(".opencode/skills")));
    }

    #[test]
    fn test_project_templates_contains_opencode_commands() {
        assert!(project_templates()
            .iter()
            .any(|(path, _)| path.to_string_lossy().contains(".opencode/commands")));
    }

    #[test]
    fn test_agents_template_has_content() {
        assert!(template_content("AGENTS.md").contains("SDD AI Harness"));
    }

    #[test]
    fn test_gitignore_template_has_content() {
        assert!(template_content(".gitignore").contains("/target"));
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: Uuid,
    pub skill_id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub applicability: SkillApplicability,
    pub inputs: Vec<SkillInput>,
    pub instructions: String,
    pub restrictions: Vec<String>,
    pub required_tools: HashSet<String>,
    pub expected_outputs: Vec<SkillOutput>,
    pub dependencies: Vec<String>,
    pub examples: Vec<SkillExample>,
    pub compatibility: SkillCompatibility,
    pub skill_type: SkillType,
    pub status: SkillStatus,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillApplicability {
    pub task_types: HashSet<String>,
    pub languages: HashSet<String>,
    pub frameworks: HashSet<String>,
    pub phases: HashSet<String>,
    pub tags: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInput {
    pub name: String,
    pub description: String,
    pub input_type: InputType,
    pub required: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InputType {
    String,
    Number,
    Boolean,
    FilePath,
    Code,
    Json,
    List,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillOutput {
    pub name: String,
    pub description: String,
    pub output_type: OutputType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputType {
    File,
    Code,
    Documentation,
    TestResult,
    Analysis,
    Report,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExample {
    pub title: String,
    pub description: String,
    pub input: HashMap<String, String>,
    pub expected_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCompatibility {
    pub min_framework_version: Option<String>,
    pub max_framework_version: Option<String>,
    pub supported_os: HashSet<String>,
    pub required_capabilities: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkillType {
    BuiltIn,
    Project,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkillStatus {
    Active,
    Deprecated,
    Experimental,
}

impl Skill {
    pub fn new(
        skill_id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            skill_id: skill_id.into(),
            name: name.into(),
            version: version.into(),
            description: description.into(),
            applicability: SkillApplicability::default(),
            inputs: Vec::new(),
            instructions: String::new(),
            restrictions: Vec::new(),
            required_tools: HashSet::new(),
            expected_outputs: Vec::new(),
            dependencies: Vec::new(),
            examples: Vec::new(),
            compatibility: SkillCompatibility::default(),
            skill_type: SkillType::BuiltIn,
            status: SkillStatus::Active,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_applicability(mut self, applicability: SkillApplicability) -> Self {
        self.applicability = applicability;
        self.updated_at = Utc::now();
        self
    }

    pub fn add_input(mut self, input: SkillInput) -> Self {
        self.inputs.push(input);
        self.updated_at = Utc::now();
        self
    }

    pub fn with_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = instructions.into();
        self.updated_at = Utc::now();
        self
    }

    pub fn add_restriction(mut self, restriction: impl Into<String>) -> Self {
        self.restrictions.push(restriction.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn add_required_tool(mut self, tool: impl Into<String>) -> Self {
        self.required_tools.insert(tool.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn add_expected_output(mut self, output: SkillOutput) -> Self {
        self.expected_outputs.push(output);
        self.updated_at = Utc::now();
        self
    }

    pub fn add_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependencies.push(dependency.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn add_example(mut self, example: SkillExample) -> Self {
        self.examples.push(example);
        self.updated_at = Utc::now();
        self
    }

    pub fn with_skill_type(mut self, skill_type: SkillType) -> Self {
        self.skill_type = skill_type;
        self.updated_at = Utc::now();
        self
    }

    pub fn is_applicable_to_task_type(&self, task_type: &str) -> bool {
        self.applicability.task_types.is_empty()
            || self.applicability.task_types.contains(task_type)
    }

    pub fn is_applicable_to_language(&self, language: &str) -> bool {
        self.applicability.languages.is_empty() || self.applicability.languages.contains(language)
    }

    pub fn is_applicable_to_phase(&self, phase: &str) -> bool {
        self.applicability.phases.is_empty() || self.applicability.phases.contains(phase)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.applicability.tags.contains(tag)
    }

    pub fn is_active(&self) -> bool {
        self.status == SkillStatus::Active
    }

    pub fn deprecate(&mut self) {
        self.status = SkillStatus::Deprecated;
        self.updated_at = Utc::now();
    }
}

impl Default for SkillCompatibility {
    fn default() -> Self {
        Self {
            min_framework_version: None,
            max_framework_version: None,
            supported_os: HashSet::from([
                "linux".to_string(),
                "macos".to_string(),
                "windows".to_string(),
            ]),
            required_capabilities: HashSet::new(),
        }
    }
}

impl SkillInput {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_type: InputType,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_type,
            required: true,
            default_value: None,
        }
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn with_default(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self.required = false;
        self
    }
}

impl SkillOutput {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        output_type: OutputType,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            output_type,
        }
    }
}

impl SkillExample {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            input: HashMap::new(),
            expected_output: String::new(),
        }
    }

    pub fn with_input(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.input.insert(key.into(), value.into());
        self
    }

    pub fn with_expected_output(mut self, output: impl Into<String>) -> Self {
        self.expected_output = output.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_creation() {
        let skill = Skill::new(
            "rust-testing",
            "Rust Testing",
            "1.0.0",
            "Best practices for Rust testing",
        );
        assert_eq!(skill.skill_id, "rust-testing");
        assert_eq!(skill.status, SkillStatus::Active);
        assert!(skill.is_active());
    }

    #[test]
    fn test_skill_applicability() {
        let mut applicability = SkillApplicability::default();
        applicability.task_types.insert("testing".to_string());
        applicability.languages.insert("rust".to_string());

        let skill = Skill::new("rust-testing", "Rust Testing", "1.0.0", "Testing skill")
            .with_applicability(applicability);

        assert!(skill.is_applicable_to_task_type("testing"));
        assert!(!skill.is_applicable_to_task_type("implementation"));
        assert!(skill.is_applicable_to_language("rust"));
    }

    #[test]
    fn test_skill_with_inputs() {
        let skill = Skill::new("test-skill", "Test Skill", "1.0.0", "A test skill")
            .add_input(SkillInput::new(
                "file_path",
                "Path to file",
                InputType::FilePath,
            ))
            .add_input(SkillInput::new("verbose", "Enable verbose", InputType::Boolean).optional());

        assert_eq!(skill.inputs.len(), 2);
        assert!(skill.inputs[0].required);
        assert!(!skill.inputs[1].required);
    }

    #[test]
    fn test_skill_with_examples() {
        let skill = Skill::new("test-skill", "Test Skill", "1.0.0", "A test skill").add_example(
            SkillExample::new("Basic usage", "Simple example")
                .with_input("file", "src/main.rs")
                .with_expected_output("Test results"),
        );

        assert_eq!(skill.examples.len(), 1);
        assert_eq!(
            skill.examples[0].input.get("file"),
            Some(&"src/main.rs".to_string())
        );
    }

    #[test]
    fn test_skill_deprecation() {
        let mut skill = Skill::new("old-skill", "Old Skill", "1.0.0", "Deprecated skill");
        assert!(skill.is_active());

        skill.deprecate();
        assert!(!skill.is_active());
        assert_eq!(skill.status, SkillStatus::Deprecated);
    }

    #[test]
    fn test_skill_tags() {
        let mut applicability = SkillApplicability::default();
        applicability.tags.insert("testing".to_string());
        applicability.tags.insert("quality".to_string());

        let skill = Skill::new("test-skill", "Test Skill", "1.0.0", "A test skill")
            .with_applicability(applicability);

        assert!(skill.has_tag("testing"));
        assert!(skill.has_tag("quality"));
        assert!(!skill.has_tag("documentation"));
    }
}

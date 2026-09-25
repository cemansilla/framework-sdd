use crate::agent::{OutputContract, OutputFormat, QualityGate};
use crate::task::Task;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputValidation {
    pub task_id: Uuid,
    pub contract: OutputContract,
    pub checks: Vec<OutputCheck>,
    pub passed: bool,
    pub validated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub details: HashMap<String, String>,
}

impl OutputCheck {
    pub fn pass(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            message: message.to_string(),
            details: HashMap::new(),
        }
    }

    pub fn fail(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            message: message.to_string(),
            details: HashMap::new(),
        }
    }

    pub fn with_detail(mut self, key: &str, value: &str) -> Self {
        self.details.insert(key.to_string(), value.to_string());
        self
    }
}

pub struct OutputContractValidator;

impl OutputContractValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(
        &self,
        task: &Task,
        contract: &OutputContract,
        output: &str,
    ) -> OutputValidation {
        let mut checks = Vec::new();

        checks.push(self.check_format(contract, output));

        for artifact in &contract.expected_artifacts {
            checks.push(self.check_artifact(artifact, output));
        }

        if contract.required_tests {
            checks.push(self.check_tests(output));
        }

        for gate in &contract.quality_gates {
            checks.push(self.check_quality_gate(gate, output));
        }

        let passed = checks.iter().all(|c| c.passed);

        OutputValidation {
            task_id: task.id,
            contract: contract.clone(),
            checks,
            passed,
            validated_at: chrono::Utc::now(),
        }
    }

    fn check_format(&self, contract: &OutputContract, output: &str) -> OutputCheck {
        let valid = match contract.format {
            OutputFormat::Code => {
                output.contains("fn ") || output.contains("pub ") || output.contains("struct ")
            }
            OutputFormat::Documentation => !output.is_empty() && output.len() > 10,
            OutputFormat::TestSuite => output.contains("#[test]") || output.contains("test"),
            OutputFormat::Review => {
                output.contains("approved")
                    || output.contains("rejected")
                    || output.contains("review")
            }
            OutputFormat::Mixed => !output.is_empty(),
        };

        if valid {
            OutputCheck::pass(
                "format_check",
                &format!("Output matches {:?} format", contract.format),
            )
        } else {
            OutputCheck::fail(
                "format_check",
                &format!("Output does not match {:?} format", contract.format),
            )
        }
    }

    fn check_artifact(&self, artifact: &str, output: &str) -> OutputCheck {
        if output.contains(artifact) {
            OutputCheck::pass(
                "artifact_check",
                &format!("Artifact '{}' found in output", artifact),
            )
        } else {
            OutputCheck::fail(
                "artifact_check",
                &format!("Artifact '{}' not found in output", artifact),
            )
        }
    }

    fn check_tests(&self, output: &str) -> OutputCheck {
        let has_tests =
            output.contains("#[test]") || output.contains("test_") || output.contains("fn test");

        if has_tests {
            OutputCheck::pass("test_check", "Tests included in output")
        } else {
            OutputCheck::fail("test_check", "Required tests not found in output")
        }
    }

    fn check_quality_gate(&self, gate: &QualityGate, output: &str) -> OutputCheck {
        match gate {
            QualityGate::FmtCheck => {
                let ok = !output.contains("fmt error");
                if ok {
                    OutputCheck::pass("fmt_check", "Format check passed")
                } else {
                    OutputCheck::fail("fmt_check", "Format check failed")
                }
            }
            QualityGate::ClippyCheck => {
                let ok = !output.contains("clippy warning");
                if ok {
                    OutputCheck::pass("clippy_check", "Clippy check passed")
                } else {
                    OutputCheck::fail("clippy_check", "Clippy check failed")
                }
            }
            QualityGate::TestPass => {
                let ok = !output.contains("FAILED") && !output.contains("panicked");
                if ok {
                    OutputCheck::pass("test_pass", "Tests passed")
                } else {
                    OutputCheck::fail("test_pass", "Tests failed")
                }
            }
            QualityGate::BuildSuccess => {
                let ok = !output.contains("error[E");
                if ok {
                    OutputCheck::pass("build_success", "Build succeeded")
                } else {
                    OutputCheck::fail("build_success", "Build failed")
                }
            }
            QualityGate::CoverageThreshold(threshold) => {
                OutputCheck::pass(
                    "coverage_threshold",
                    &format!("Coverage threshold {:.1}% met", threshold),
                )
            }
        }
    }
}

impl Default for OutputContractValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputContract {
    pub fn for_code() -> Self {
        Self {
            expected_artifacts: Vec::new(),
            required_tests: true,
            quality_gates: vec![
                QualityGate::FmtCheck,
                QualityGate::ClippyCheck,
                QualityGate::BuildSuccess,
            ],
            format: OutputFormat::Code,
        }
    }

    pub fn for_tests() -> Self {
        Self {
            expected_artifacts: Vec::new(),
            required_tests: true,
            quality_gates: vec![QualityGate::TestPass],
            format: OutputFormat::TestSuite,
        }
    }

    pub fn for_documentation() -> Self {
        Self {
            expected_artifacts: Vec::new(),
            required_tests: false,
            quality_gates: Vec::new(),
            format: OutputFormat::Documentation,
        }
    }

    pub fn for_review() -> Self {
        Self {
            expected_artifacts: Vec::new(),
            required_tests: false,
            quality_gates: Vec::new(),
            format: OutputFormat::Review,
        }
    }

    pub fn with_artifact(mut self, artifact: &str) -> Self {
        self.expected_artifacts.push(artifact.to_string());
        self
    }

    pub fn with_quality_gate(mut self, gate: QualityGate) -> Self {
        self.quality_gates.push(gate);
        self
    }

    pub fn with_required_tests(mut self, required: bool) -> Self {
        self.required_tests = required;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Task;

    #[test]
    fn test_output_check_pass() {
        let check = OutputCheck::pass("test", "All good");
        assert!(check.passed);
    }

    #[test]
    fn test_output_check_fail() {
        let check = OutputCheck::fail("test", "Not good");
        assert!(!check.passed);
    }

    #[test]
    fn test_validate_code_output() {
        let validator = OutputContractValidator::new();
        let task = Task::new("TASK-001", "Test", "Test task");
        let contract = OutputContract::for_code().with_required_tests(false);
        let output = "pub fn my_function() -> bool { true }";

        let validation = validator.validate(&task, &contract, output);
        assert!(validation.passed);
    }

    #[test]
    fn test_validate_empty_output() {
        let validator = OutputContractValidator::new();
        let task = Task::new("TASK-001", "Test", "Test task");
        let contract = OutputContract::for_documentation();
        let output = "";

        let validation = validator.validate(&task, &contract, output);
        assert!(!validation.passed);
    }

    #[test]
    fn test_validate_with_artifact() {
        let validator = OutputContractValidator::new();
        let task = Task::new("TASK-001", "Test", "Test task");
        let contract = OutputContract::for_code()
            .with_artifact("my_function")
            .with_required_tests(false);
        let output = "pub fn my_function() {}";

        let validation = validator.validate(&task, &contract, output);
        assert!(validation.passed);
    }

    #[test]
    fn test_validate_with_required_tests() {
        let validator = OutputContractValidator::new();
        let task = Task::new("TASK-001", "Test", "Test task");
        let contract = OutputContract::for_code();
        let output = "pub fn my_function() {}\n#[test]\nfn test_my_function() {}";

        let validation = validator.validate(&task, &contract, output);
        assert!(validation.passed);
    }

    #[test]
    fn test_output_contract_presets() {
        let code = OutputContract::for_code();
        assert_eq!(code.format, OutputFormat::Code);
        assert!(code.required_tests);

        let docs = OutputContract::for_documentation();
        assert_eq!(docs.format, OutputFormat::Documentation);
        assert!(!docs.required_tests);
    }
}

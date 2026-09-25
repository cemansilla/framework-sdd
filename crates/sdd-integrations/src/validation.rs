use crate::agent_contract::AgentContract;
use crate::model_profile::{ContextFormat, ModelCapability, ModelProfile};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub profile_id: String,
    pub validated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorSeverity {
    Critical,
    High,
    Medium,
    Low,
}

pub struct IntegrationValidator;

impl IntegrationValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_profile(&self, profile: &ModelProfile) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if profile.id.is_empty() {
            errors.push(ValidationError {
                code: "PROFILE_ID_EMPTY".to_string(),
                message: "Profile ID cannot be empty".to_string(),
                field: Some("id".to_string()),
                severity: ErrorSeverity::Critical,
            });
        }

        if profile.name.is_empty() {
            errors.push(ValidationError {
                code: "PROFILE_NAME_EMPTY".to_string(),
                message: "Profile name cannot be empty".to_string(),
                field: Some("name".to_string()),
                severity: ErrorSeverity::Critical,
            });
        }

        if profile.version.is_empty() {
            errors.push(ValidationError {
                code: "PROFILE_VERSION_EMPTY".to_string(),
                message: "Profile version cannot be empty".to_string(),
                field: Some("version".to_string()),
                severity: ErrorSeverity::High,
            });
        }

        if profile.context_window == 0 {
            errors.push(ValidationError {
                code: "INVALID_CONTEXT_WINDOW".to_string(),
                message: "Context window must be greater than 0".to_string(),
                field: Some("context_window".to_string()),
                severity: ErrorSeverity::Critical,
            });
        }

        if profile.context_window < 1024 {
            warnings.push(ValidationWarning {
                code: "SMALL_CONTEXT_WINDOW".to_string(),
                message: "Context window is very small (< 1024 tokens)".to_string(),
                field: Some("context_window".to_string()),
                suggestion: Some("Consider using a profile with larger context window".to_string()),
            });
        }

        if profile.capabilities.is_empty() {
            warnings.push(ValidationWarning {
                code: "NO_CAPABILITIES".to_string(),
                message: "Profile has no capabilities defined".to_string(),
                field: Some("capabilities".to_string()),
                suggestion: Some("Add at least one capability".to_string()),
            });
        }

        if profile.supported_formats.is_empty() {
            errors.push(ValidationError {
                code: "NO_FORMATS".to_string(),
                message: "Profile must support at least one context format".to_string(),
                field: Some("supported_formats".to_string()),
                severity: ErrorSeverity::High,
            });
        }

        ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
            profile_id: profile.id.clone(),
            validated_at: chrono::Utc::now(),
        }
    }

    pub fn validate_contract(&self, contract: &AgentContract) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if contract.agent_id.is_empty() {
            errors.push(ValidationError {
                code: "CONTRACT_AGENT_ID_EMPTY".to_string(),
                message: "Agent ID cannot be empty".to_string(),
                field: Some("agent_id".to_string()),
                severity: ErrorSeverity::Critical,
            });
        }

        if contract.version.is_empty() {
            errors.push(ValidationError {
                code: "CONTRACT_VERSION_EMPTY".to_string(),
                message: "Contract version cannot be empty".to_string(),
                field: Some("version".to_string()),
                severity: ErrorSeverity::High,
            });
        }

        if contract.timeout_seconds == 0 {
            warnings.push(ValidationWarning {
                code: "ZERO_TIMEOUT".to_string(),
                message: "Timeout is set to 0 seconds".to_string(),
                field: Some("timeout_seconds".to_string()),
                suggestion: Some("Set a reasonable timeout value".to_string()),
            });
        }

        if contract.capabilities.is_empty() {
            warnings.push(ValidationWarning {
                code: "NO_CONTRACT_CAPABILITIES".to_string(),
                message: "Contract has no capabilities defined".to_string(),
                field: Some("capabilities".to_string()),
                suggestion: Some("Define agent capabilities".to_string()),
            });
        }

        ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
            profile_id: contract.agent_id.clone(),
            validated_at: chrono::Utc::now(),
        }
    }

    pub fn validate_compatibility(
        &self,
        profile: &ModelProfile,
        required_capabilities: &[ModelCapability],
        required_formats: &[ContextFormat],
    ) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        let profile_caps: HashSet<_> = profile.capabilities.iter().collect();
        let required_caps: HashSet<_> = required_capabilities.iter().collect();

        let missing_caps: Vec<_> = required_caps.difference(&profile_caps).collect();
        if !missing_caps.is_empty() {
            errors.push(ValidationError {
                code: "MISSING_CAPABILITIES".to_string(),
                message: format!("Profile missing required capabilities: {:?}", missing_caps),
                field: Some("capabilities".to_string()),
                severity: ErrorSeverity::Critical,
            });
        }

        let profile_formats: HashSet<_> = profile.supported_formats.iter().collect();
        let required_formats_set: HashSet<_> = required_formats.iter().collect();

        let missing_formats: Vec<_> = required_formats_set.difference(&profile_formats).collect();
        if !missing_formats.is_empty() {
            errors.push(ValidationError {
                code: "MISSING_FORMATS".to_string(),
                message: format!("Profile missing required formats: {:?}", missing_formats),
                field: Some("supported_formats".to_string()),
                severity: ErrorSeverity::High,
            });
        }

        if profile.context_window < 4096 {
            warnings.push(ValidationWarning {
                code: "LOW_CONTEXT_WINDOW".to_string(),
                message: "Context window may be insufficient for complex tasks".to_string(),
                field: Some("context_window".to_string()),
                suggestion: Some("Consider using a profile with at least 4096 tokens".to_string()),
            });
        }

        ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
            profile_id: profile.id.clone(),
            validated_at: chrono::Utc::now(),
        }
    }

    pub fn validate_all(
        &self,
        profiles: &[ModelProfile],
        contracts: &[AgentContract],
    ) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        for profile in profiles {
            results.push(self.validate_profile(profile));
        }

        for contract in contracts {
            results.push(self.validate_contract(contract));
        }

        results
    }
}

impl Default for IntegrationValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_profile::{ModelProfile, ModelProvider};

    #[test]
    fn test_validate_valid_profile() {
        let validator = IntegrationValidator::new();
        let profile = ModelProfile::new(
            "test",
            "Test Profile",
            "1.0.0",
            ModelProvider::Custom("test".to_string()),
        )
        .with_context_window(4096)
        .with_capabilities(vec![ModelCapability::CodeGeneration]);

        let result = validator.validate_profile(&profile);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_invalid_profile() {
        let validator = IntegrationValidator::new();
        let profile = ModelProfile::new("", "", "", ModelProvider::Custom("test".to_string()));

        let result = validator.validate_profile(&profile);
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_compatibility() {
        let validator = IntegrationValidator::new();
        let profile = ModelProfile::new(
            "test",
            "Test",
            "1.0.0",
            ModelProvider::Custom("test".to_string()),
        )
        .with_capabilities(vec![ModelCapability::CodeGeneration])
        .with_context_window(4096);

        let result =
            validator.validate_compatibility(&profile, &[ModelCapability::CodeGeneration], &[]);

        assert!(result.valid);
    }

    #[test]
    fn test_validate_missing_capabilities() {
        let validator = IntegrationValidator::new();
        let profile = ModelProfile::new(
            "test",
            "Test",
            "1.0.0",
            ModelProvider::Custom("test".to_string()),
        )
        .with_capabilities(vec![ModelCapability::CodeGeneration]);

        let result = validator.validate_compatibility(
            &profile,
            &[
                ModelCapability::CodeGeneration,
                ModelCapability::TestGeneration,
            ],
            &[],
        );

        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == "MISSING_CAPABILITIES"));
    }
}

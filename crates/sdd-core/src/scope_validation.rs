use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeValidationHook {
    pub allowed_paths: Vec<String>,
    pub denied_paths: Vec<String>,
    pub allowed_extensions: Vec<String>,
    pub max_file_size: Option<u64>,
    pub require_test_files: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub file: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub file: String,
    pub reason: String,
}

impl ScopeValidationHook {
    pub fn new() -> Self {
        Self {
            allowed_paths: vec![
                "crates/".to_string(),
                "src/".to_string(),
                "tests/".to_string(),
            ],
            denied_paths: vec![
                "target/".to_string(),
                ".git/".to_string(),
                "node_modules/".to_string(),
            ],
            allowed_extensions: vec![
                ".rs".to_string(),
                ".toml".to_string(),
                ".md".to_string(),
            ],
            max_file_size: Some(1_000_000),
            require_test_files: true,
        }
    }

    pub fn with_allowed_paths(mut self, paths: Vec<String>) -> Self {
        self.allowed_paths = paths;
        self
    }

    pub fn with_denied_paths(mut self, paths: Vec<String>) -> Self {
        self.denied_paths = paths;
        self
    }

    pub fn with_allowed_extensions(mut self, extensions: Vec<String>) -> Self {
        self.allowed_extensions = extensions;
        self
    }

    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = Some(size);
        self
    }

    pub fn with_require_test_files(mut self, require: bool) -> Self {
        self.require_test_files = require;
        self
    }

    pub fn validate_files(&self, files: &[String]) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for file in files {
            if let Some(error) = self.validate_file(file) {
                errors.push(error);
            }

            if let Some(warning) = self.check_file_warnings(file) {
                warnings.push(warning);
            }
        }

        if self.require_test_files {
            let has_test_files = files.iter().any(|f| {
                f.contains("test") || f.contains("spec") || f.ends_with("_test.rs")
            });

            if !has_test_files && !files.is_empty() {
                warnings.push(ValidationWarning {
                    file: "all".to_string(),
                    reason: "No test files included in changes".to_string(),
                });
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    fn validate_file(&self, file: &str) -> Option<ValidationError> {
        if self.is_denied_path(file) {
            return Some(ValidationError {
                file: file.to_string(),
                reason: "File is in denied path".to_string(),
            });
        }

        if !self.is_allowed_path(file) {
            return Some(ValidationError {
                file: file.to_string(),
                reason: "File is not in allowed paths".to_string(),
            });
        }

        if !self.has_allowed_extension(file) {
            return Some(ValidationError {
                file: file.to_string(),
                reason: "File extension not allowed".to_string(),
            });
        }

        None
    }

    fn check_file_warnings(&self, file: &str) -> Option<ValidationWarning> {
        if let Some(max_size) = self.max_file_size {
            if let Ok(metadata) = std::fs::metadata(file) {
                if metadata.len() > max_size {
                    return Some(ValidationWarning {
                        file: file.to_string(),
                        reason: format!("File size exceeds {} bytes", max_size),
                    });
                }
            }
        }

        None
    }

    fn is_denied_path(&self, file: &str) -> bool {
        self.denied_paths.iter().any(|denied| file.starts_with(denied))
    }

    fn is_allowed_path(&self, file: &str) -> bool {
        if self.allowed_paths.is_empty() {
            return true;
        }
        self.allowed_paths.iter().any(|allowed| file.starts_with(allowed))
    }

    fn has_allowed_extension(&self, file: &str) -> bool {
        if self.allowed_extensions.is_empty() {
            return true;
        }
        self.allowed_extensions
            .iter()
            .any(|ext| file.ends_with(ext))
    }
}

impl Default for ScopeValidationHook {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PreCommitHook {
    scope_validator: ScopeValidationHook,
}

impl PreCommitHook {
    pub fn new(scope_validator: ScopeValidationHook) -> Self {
        Self { scope_validator }
    }

    pub fn run(&self, staged_files: &[String]) -> ValidationResult {
        self.scope_validator.validate_files(staged_files)
    }
}

pub struct PrePushHook {
    scope_validator: ScopeValidationHook,
}

impl PrePushHook {
    pub fn new(scope_validator: ScopeValidationHook) -> Self {
        Self { scope_validator }
    }

    pub fn run(&self, files_to_push: &[String]) -> ValidationResult {
        self.scope_validator.validate_files(files_to_push)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_validation_hook_creation() {
        let hook = ScopeValidationHook::new();
        assert_eq!(hook.allowed_paths.len(), 3);
        assert_eq!(hook.denied_paths.len(), 3);
    }

    #[test]
    fn test_validate_allowed_file() {
        let hook = ScopeValidationHook::new();
        let files = vec!["crates/sdd-core/src/lib.rs".to_string()];
        let result = hook.validate_files(&files);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_denied_file() {
        let hook = ScopeValidationHook::new();
        let files = vec!["target/debug/binary".to_string()];
        let result = hook.validate_files(&files);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_validate_not_allowed_path() {
        let hook = ScopeValidationHook::new();
        let files = vec!["random/file.txt".to_string()];
        let result = hook.validate_files(&files);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_wrong_extension() {
        let hook = ScopeValidationHook::new();
        let files = vec!["crates/test.py".to_string()];
        let result = hook.validate_files(&files);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_multiple_files() {
        let hook = ScopeValidationHook::new();
        let files = vec![
            "crates/sdd-core/src/lib.rs".to_string(),
            "crates/sdd-storage/src/lib.rs".to_string(),
        ];
        let result = hook.validate_files(&files);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_with_test_files() {
        let hook = ScopeValidationHook::new().with_require_test_files(true);
        let files = vec![
            "crates/sdd-core/src/lib.rs".to_string(),
            "crates/sdd-core/tests/test.rs".to_string(),
        ];
        let result = hook.validate_files(&files);
        assert!(result.is_valid);
        assert_eq!(result.warnings.len(), 0);
    }

    #[test]
    fn test_validate_without_test_files_warning() {
        let hook = ScopeValidationHook::new().with_require_test_files(true);
        let files = vec!["crates/sdd-core/src/lib.rs".to_string()];
        let result = hook.validate_files(&files);
        assert!(result.is_valid);
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_custom_allowed_paths() {
        let hook = ScopeValidationHook::new()
            .with_allowed_paths(vec!["custom/".to_string()]);
        
        let files = vec!["custom/file.rs".to_string()];
        let result = hook.validate_files(&files);
        assert!(result.is_valid);
    }

    #[test]
    fn test_custom_denied_paths() {
        let hook = ScopeValidationHook::new()
            .with_denied_paths(vec!["secret/".to_string()]);
        
        let files = vec!["secret/config.rs".to_string()];
        let result = hook.validate_files(&files);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_custom_extensions() {
        let hook = ScopeValidationHook::new()
            .with_allowed_extensions(vec![".py".to_string(), ".js".to_string()]);
        
        let files = vec!["crates/test.py".to_string()];
        let result = hook.validate_files(&files);
        assert!(result.is_valid);
    }

    #[test]
    fn test_pre_commit_hook() {
        let validator = ScopeValidationHook::new();
        let hook = PreCommitHook::new(validator);
        
        let files = vec!["crates/sdd-core/src/lib.rs".to_string()];
        let result = hook.run(&files);
        assert!(result.is_valid);
    }

    #[test]
    fn test_pre_push_hook() {
        let validator = ScopeValidationHook::new();
        let hook = PrePushHook::new(validator);
        
        let files = vec!["crates/sdd-core/src/lib.rs".to_string()];
        let result = hook.run(&files);
        assert!(result.is_valid);
    }
}

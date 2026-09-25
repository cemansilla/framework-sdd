use crate::context_bundle::{ContextFragment, ContextPriority, ContextSource};
use crate::task::Task;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ScopeResolver {
    project_root: PathBuf,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
}

impl ScopeResolver {
    pub fn new(project_root: impl AsRef<Path>) -> Self {
        Self {
            project_root: project_root.as_ref().to_path_buf(),
            include_patterns: Vec::new(),
            exclude_patterns: vec![
                "target/**".to_string(),
                ".git/**".to_string(),
                "node_modules/**".to_string(),
                "**/*.lock".to_string(),
            ],
        }
    }

    pub fn with_include_patterns(mut self, patterns: Vec<String>) -> Self {
        self.include_patterns = patterns;
        self
    }

    pub fn with_exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns = patterns;
        self
    }

    pub fn resolve_scope(&self, task: &Task) -> ScopeResult {
        let mut files = HashSet::new();
        let mut fragments = Vec::new();

        let task_files = self.extract_files_from_task(task);
        for file in task_files {
            if self.is_in_scope(&file) {
                files.insert(file);
            }
        }

        for file in &files {
            if let Ok(content) = std::fs::read_to_string(file) {
                let fragment = ContextFragment::new(
                    content,
                    ContextSource::Code,
                    file.to_string_lossy().to_string(),
                    ContextPriority::P1,
                    "File in task scope".to_string(),
                );
                fragments.push(fragment);
            }
        }

        ScopeResult {
            files: files.into_iter().collect(),
            fragments,
        }
    }

    pub fn resolve_file_scope(&self, paths: &[PathBuf]) -> ScopeResult {
        let mut files = HashSet::new();
        let mut fragments = Vec::new();

        for path in paths {
            if self.is_in_scope(path) {
                files.insert(path.clone());
                if let Ok(content) = std::fs::read_to_string(path) {
                    let fragment = ContextFragment::new(
                        content,
                        ContextSource::Code,
                        path.to_string_lossy().to_string(),
                        ContextPriority::P1,
                        "Explicitly scoped file".to_string(),
                    );
                    fragments.push(fragment);
                }
            }
        }

        ScopeResult {
            files: files.into_iter().collect(),
            fragments,
        }
    }

    pub fn is_in_scope(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.exclude_patterns {
            if self.matches_pattern(&path_str, pattern) {
                return false;
            }
        }

        if self.include_patterns.is_empty() {
            return true;
        }

        for pattern in &self.include_patterns {
            if self.matches_pattern(&path_str, pattern) {
                return true;
            }
        }

        false
    }

    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        if pattern.contains("**") {
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                let suffix_trimmed = suffix.trim_start_matches('/');
                if path.starts_with(prefix.trim_end_matches('/')) {
                    if suffix_trimmed.is_empty() {
                        return true;
                    }
                    let remaining = &path[prefix.len()..];
                    return self.matches_glob(remaining, suffix_trimmed);
                }
            }
        }

        if pattern.contains('*') {
            return self.matches_glob(path, pattern);
        }

        path.contains(pattern)
    }

    fn matches_glob(&self, text: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1];
            return text.starts_with(prefix)
                && text.ends_with(suffix)
                && text.len() >= prefix.len() + suffix.len();
        }

        text.contains(pattern)
    }

    fn extract_files_from_task(&self, task: &Task) -> Vec<PathBuf> {
        let mut files = Vec::new();

        let description_lower = task.description.to_lowercase();
        let words: Vec<&str> = description_lower.split_whitespace().collect();

        for word in words {
            if word.ends_with(".rs") || word.ends_with(".toml") || word.ends_with(".md") {
                let path = self.project_root.join(word);
                if path.exists() {
                    files.push(path);
                }
            }
        }

        files
    }
}

#[derive(Debug, Clone)]
pub struct ScopeResult {
    pub files: Vec<PathBuf>,
    pub fragments: Vec<ContextFragment>,
}

impl ScopeResult {
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn total_tokens(&self) -> usize {
        self.fragments.iter().map(|f| f.estimated_tokens).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Task;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scope_resolver_creation() {
        let resolver = ScopeResolver::new("/tmp/project");
        assert_eq!(resolver.project_root, PathBuf::from("/tmp/project"));
    }

    #[test]
    fn test_scope_resolver_with_patterns() {
        let resolver = ScopeResolver::new("/tmp/project")
            .with_include_patterns(vec!["src/**/*.rs".to_string()])
            .with_exclude_patterns(vec!["target/**".to_string()]);

        assert_eq!(resolver.include_patterns.len(), 1);
        assert_eq!(resolver.exclude_patterns.len(), 1);
    }

    #[test]
    fn test_is_in_scope_exclude() {
        let resolver = ScopeResolver::new("/tmp/project");

        assert!(!resolver.is_in_scope(Path::new("target/debug/binary")));
        assert!(!resolver.is_in_scope(Path::new(".git/config")));
        assert!(resolver.is_in_scope(Path::new("src/main.rs")));
    }

    #[test]
    fn test_is_in_scope_include() {
        let resolver = ScopeResolver::new("/tmp/project")
            .with_include_patterns(vec!["src/**/*.rs".to_string()]);

        assert!(resolver.is_in_scope(Path::new("src/main.rs")));
        assert!(!resolver.is_in_scope(Path::new("tests/test.rs")));
    }

    #[test]
    fn test_resolve_scope_with_temp_files() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let main_rs = src_dir.join("main.rs");
        fs::write(&main_rs, "fn main() {}").unwrap();

        let resolver = ScopeResolver::new(temp_dir.path());
        let task = Task::new("TASK-001", "Update main", "Modify src/main.rs");

        let result = resolver.resolve_scope(&task);
        assert_eq!(result.file_count(), 1);
    }

    #[test]
    fn test_scope_result_total_tokens() {
        let fragment = ContextFragment::new(
            "fn main() {}".to_string(),
            ContextSource::Code,
            "main.rs".to_string(),
            ContextPriority::P1,
            "Test file".to_string(),
        );

        let result = ScopeResult {
            files: vec![PathBuf::from("main.rs")],
            fragments: vec![fragment],
        };

        assert!(result.total_tokens() > 0);
    }
}

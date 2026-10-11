use crate::manifest::Manifest;
use crate::project_templates;
use crate::storage_port::{StorageError, StoragePort};
use crate::structure::{default_templates, SddStructure};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Debug)]
pub struct FilesystemAdapter {
    base_path: PathBuf,
}

impl FilesystemAdapter {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    pub fn structure(&self) -> SddStructure {
        SddStructure::new(&self.base_path)
    }

    /// Create the `.sdd/` structure with template artifacts.
    ///
    /// Existing files (templates and manifest) are never overwritten, so
    /// running `initialize` on a populated project is idempotent and never
    /// destroys user content.
    pub async fn initialize(&self, project_id: &str) -> Result<Manifest, StorageError> {
        let structure = self.structure();

        for dir in structure.all_directories() {
            fs::create_dir_all(&dir).await.map_err(StorageError::Io)?;
        }

        for (relative_path, content) in default_templates() {
            let full_path = structure.sdd_dir().join(&relative_path);
            if content.is_empty() {
                fs::create_dir_all(&full_path)
                    .await
                    .map_err(StorageError::Io)?;
            } else if !full_path.exists() {
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent).await.map_err(StorageError::Io)?;
                }
                fs::write(&full_path, content)
                    .await
                    .map_err(StorageError::Io)?;
            }
        }

        let manifest_path = structure.layout().manifest;
        if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path)
                .await
                .map_err(StorageError::Io)?;
            return serde_json::from_str(&content).map_err(|e| {
                StorageError::Serialization(format!("existing manifest is invalid: {e}"))
            });
        }

        let manifest = Manifest::new(project_id);
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        fs::write(&manifest_path, manifest_json)
            .await
            .map_err(StorageError::Io)?;

        Ok(manifest)
    }

    /// Install project-level files (AGENTS.md, .gitignore, .opencode/).
    ///
    /// Existing files are never overwritten, so running this on a populated
    /// project is idempotent and never destroys user content.
    pub async fn install_project_files(&self) -> Result<(), StorageError> {
        for (relative_path, content) in project_templates::project_templates() {
            let full_path = self.base_path.join(&relative_path);
            if !full_path.exists() {
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent).await.map_err(StorageError::Io)?;
                }
                fs::write(&full_path, content)
                    .await
                    .map_err(StorageError::Io)?;
            }
        }
        Ok(())
    }

    /// Overwrite project-level files with the current templates.
    ///
    /// Unlike [`Self::install_project_files`] this replaces AGENTS.md,
    /// .gitignore and .opencode/ unconditionally, refreshing them after a
    /// framework upgrade. User artifacts under `.sdd/` are never touched.
    pub async fn update_project_files(&self) -> Result<(), StorageError> {
        for (relative_path, content) in project_templates::project_templates() {
            let full_path = self.base_path.join(&relative_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).await.map_err(StorageError::Io)?;
            }
            self.atomic_write(&full_path, content.as_bytes()).await?;
        }
        Ok(())
    }

    pub async fn load_manifest(&self) -> Result<Manifest, StorageError> {
        let structure = self.structure();
        let manifest_path = &structure.layout().manifest;

        if !manifest_path.exists() {
            return Err(StorageError::NotFound {
                path: manifest_path.to_string_lossy().to_string(),
            });
        }

        let content = fs::read_to_string(manifest_path)
            .await
            .map_err(StorageError::Io)?;

        serde_json::from_str(&content).map_err(|e| StorageError::Serialization(e.to_string()))
    }

    pub async fn save_manifest(&self, manifest: &Manifest) -> Result<(), StorageError> {
        let structure = self.structure();
        let manifest_json = serde_json::to_string_pretty(manifest)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        self.atomic_write(&structure.layout().manifest, manifest_json.as_bytes())
            .await
    }

    async fn atomic_write(&self, path: &Path, data: &[u8]) -> Result<(), StorageError> {
        let temp_path = path.with_extension("tmp");

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(StorageError::Io)?;
        }

        let mut file = fs::File::create(&temp_path)
            .await
            .map_err(StorageError::Io)?;

        file.write_all(data).await.map_err(StorageError::Io)?;
        file.flush().await.map_err(StorageError::Io)?;
        drop(file);

        fs::rename(&temp_path, path)
            .await
            .map_err(StorageError::Io)?;

        Ok(())
    }
}

#[async_trait]
impl StoragePort for FilesystemAdapter {
    async fn read(&self, path: &str) -> Result<Vec<u8>, StorageError> {
        let full_path = self.base_path.join(path);
        fs::read(&full_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StorageError::NotFound {
                    path: path.to_string(),
                }
            } else {
                StorageError::Io(e)
            }
        })
    }

    async fn write(&self, path: &str, data: &[u8]) -> Result<(), StorageError> {
        let full_path = self.base_path.join(path);
        self.atomic_write(&full_path, data).await
    }

    async fn exists(&self, path: &str) -> bool {
        let full_path = self.base_path.join(path);
        full_path.exists()
    }

    async fn delete(&self, path: &str) -> Result<(), StorageError> {
        let full_path = self.base_path.join(path);
        if full_path.is_dir() {
            fs::remove_dir_all(&full_path)
                .await
                .map_err(StorageError::Io)
        } else if full_path.is_file() {
            fs::remove_file(&full_path).await.map_err(StorageError::Io)
        } else {
            Err(StorageError::NotFound {
                path: path.to_string(),
            })
        }
    }

    async fn list(&self, path: &str) -> Result<Vec<String>, StorageError> {
        let full_path = self.base_path.join(path);
        let mut entries = Vec::new();

        let mut dir = fs::read_dir(&full_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StorageError::NotFound {
                    path: path.to_string(),
                }
            } else {
                StorageError::Io(e)
            }
        })?;

        while let Some(entry) = dir.next_entry().await.map_err(StorageError::Io)? {
            if let Some(name) = entry.file_name().to_str() {
                entries.push(name.to_string());
            }
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_initialize() {
        let temp = TempDir::new().unwrap();
        let adapter = FilesystemAdapter::new(temp.path());

        let manifest = adapter.initialize("test-project").await.unwrap();
        assert_eq!(manifest.project_id, "test-project");

        let structure = adapter.structure();
        assert!(structure.sdd_dir().exists());
        assert!(structure.layout().manifest.exists());
    }

    /// TASK-FW-205: two fresh initializations with the same project_id must
    /// produce identical structure and template content; the manifest may
    /// differ only in its timestamps.
    #[tokio::test]
    async fn test_initialize_is_reproducible() {
        let dir_a = TempDir::new().unwrap();
        let dir_b = TempDir::new().unwrap();

        FilesystemAdapter::new(dir_a.path())
            .initialize("same-project")
            .await
            .unwrap();
        FilesystemAdapter::new(dir_b.path())
            .initialize("same-project")
            .await
            .unwrap();

        let entries_a = collect_entries(&dir_a.path().join(".sdd")).unwrap();
        let entries_b = collect_entries(&dir_b.path().join(".sdd")).unwrap();
        assert_eq!(entries_a, entries_b, "file/dir structure must match");

        for (relative, is_dir) in &entries_a {
            if *is_dir {
                continue;
            }
            let content_a = std::fs::read(dir_a.path().join(".sdd").join(relative)).unwrap();
            let content_b = std::fs::read(dir_b.path().join(".sdd").join(relative)).unwrap();

            if relative == Path::new("manifest.json") {
                let normalize = |bytes: &[u8]| {
                    let mut value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
                    value["created_at"] = serde_json::Value::Null;
                    value["updated_at"] = serde_json::Value::Null;
                    value.to_string()
                };
                assert_eq!(normalize(&content_a), normalize(&content_b));
            } else {
                assert_eq!(
                    content_a, content_b,
                    "content must match for {:?}",
                    relative
                );
            }
        }
    }

    /// TASK-FW-205: re-running initialize on a populated project must not
    /// overwrite user artifacts or regenerate the manifest.
    #[tokio::test]
    async fn test_initialize_is_idempotent() {
        let temp = TempDir::new().unwrap();
        let adapter = FilesystemAdapter::new(temp.path());

        let first = adapter.initialize("keep-me").await.unwrap();
        let structure = adapter.structure();

        // Simulate user content.
        let brief = structure.layout().brief.join("brief.md");
        tokio::fs::write(&brief, "# My custom brief\n")
            .await
            .unwrap();

        let second = adapter.initialize("different-id").await.unwrap();

        let brief_after = tokio::fs::read_to_string(&brief).await.unwrap();
        assert_eq!(brief_after, "# My custom brief\n");
        assert_eq!(second.project_id, "keep-me");
        assert_eq!(first.project_id, second.project_id);

        let manifest_after = adapter.load_manifest().await.unwrap();
        assert_eq!(manifest_after.project_id, "keep-me");
    }

    fn collect_entries(root: &std::path::Path) -> std::io::Result<Vec<(std::path::PathBuf, bool)>> {
        fn walk(
            dir: &std::path::Path,
            root: &std::path::Path,
            out: &mut Vec<(std::path::PathBuf, bool)>,
        ) -> std::io::Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let path = entry?.path();
                let relative = path.strip_prefix(root).unwrap().to_path_buf();
                if path.is_dir() {
                    out.push((relative.clone(), true));
                    walk(&path, root, out)?;
                } else {
                    out.push((relative, false));
                }
            }
            Ok(())
        }

        let mut entries = Vec::new();
        walk(root, root, &mut entries)?;
        entries.sort();
        Ok(entries)
    }

    #[tokio::test]
    async fn test_install_project_files() {
        let temp = TempDir::new().unwrap();
        let adapter = FilesystemAdapter::new(temp.path());

        adapter.initialize("test-project").await.unwrap();
        adapter.install_project_files().await.unwrap();

        assert!(temp.path().join("AGENTS.md").exists());
        assert!(temp.path().join(".gitignore").exists());
        assert!(temp
            .path()
            .join(".opencode/agents/orchestrator.md")
            .exists());
        assert!(temp
            .path()
            .join(".opencode/skills/branching-sdd/SKILL.md")
            .exists());
        assert!(temp.path().join(".opencode/commands/plan.md").exists());
    }

    /// Re-running install must never clobber edits the user made to the
    /// installed project files.
    #[tokio::test]
    async fn test_install_project_files_is_idempotent() {
        let temp = TempDir::new().unwrap();
        let adapter = FilesystemAdapter::new(temp.path());

        adapter.initialize("test-project").await.unwrap();
        adapter.install_project_files().await.unwrap();

        let agents = temp.path().join("AGENTS.md");
        tokio::fs::write(&agents, "# Mine\n").await.unwrap();

        adapter.install_project_files().await.unwrap();

        assert_eq!(
            tokio::fs::read_to_string(&agents).await.unwrap(),
            "# Mine\n"
        );
    }

    /// `--update` must refresh project files that drifted from the templates,
    /// including files the user never edited.
    #[tokio::test]
    async fn test_update_project_files_restores_templates() {
        let temp = TempDir::new().unwrap();
        let adapter = FilesystemAdapter::new(temp.path());

        adapter.initialize("test-project").await.unwrap();
        adapter.install_project_files().await.unwrap();

        let agents = temp.path().join("AGENTS.md");
        let skill = temp.path().join(".opencode/skills/rust-quality/SKILL.md");
        tokio::fs::write(&agents, "# Mine\n").await.unwrap();
        tokio::fs::write(&skill, "# Stale skill\n").await.unwrap();

        adapter.update_project_files().await.unwrap();

        let agents_after = tokio::fs::read_to_string(&agents).await.unwrap();
        let skill_after = tokio::fs::read_to_string(&skill).await.unwrap();
        assert!(agents_after.contains("SDD AI Harness"));
        assert_eq!(
            skill_after,
            project_templates::project_templates()
                .into_iter()
                .find(|(path, _)| path.ends_with("rust-quality/SKILL.md"))
                .map(|(_, content)| content.to_string())
                .unwrap()
        );
    }

    /// `--update` refreshes project files but must never touch `.sdd/`.
    #[tokio::test]
    async fn test_update_project_files_preserves_sdd_artifacts() {
        let temp = TempDir::new().unwrap();
        let adapter = FilesystemAdapter::new(temp.path());

        adapter.initialize("keep-me").await.unwrap();
        adapter.install_project_files().await.unwrap();

        let brief = adapter.structure().layout().brief.join("brief.md");
        tokio::fs::write(&brief, "# My brief\n").await.unwrap();

        adapter.update_project_files().await.unwrap();

        assert_eq!(
            tokio::fs::read_to_string(&brief).await.unwrap(),
            "# My brief\n"
        );
        assert_eq!(adapter.load_manifest().await.unwrap().project_id, "keep-me");
    }

    #[tokio::test]
    async fn test_load_manifest() {
        let temp = TempDir::new().unwrap();
        let adapter = FilesystemAdapter::new(temp.path());

        adapter.initialize("test-project").await.unwrap();
        let manifest = adapter.load_manifest().await.unwrap();
        assert_eq!(manifest.project_id, "test-project");
    }

    #[tokio::test]
    async fn test_read_write() {
        let temp = TempDir::new().unwrap();
        let adapter = FilesystemAdapter::new(temp.path());

        adapter.write("test.txt", b"hello").await.unwrap();
        let content = adapter.read("test.txt").await.unwrap();
        assert_eq!(content, b"hello");
    }

    #[tokio::test]
    async fn test_atomic_write() {
        let temp = TempDir::new().unwrap();
        let adapter = FilesystemAdapter::new(temp.path());

        adapter.write("test.txt", b"content1").await.unwrap();
        adapter.write("test.txt", b"content2").await.unwrap();

        let content = adapter.read("test.txt").await.unwrap();
        assert_eq!(content, b"content2");

        assert!(!temp.path().join("test.tmp").exists());
    }

    #[tokio::test]
    async fn test_exists() {
        let temp = TempDir::new().unwrap();
        let adapter = FilesystemAdapter::new(temp.path());

        assert!(!adapter.exists("test.txt").await);
        adapter.write("test.txt", b"hello").await.unwrap();
        assert!(adapter.exists("test.txt").await);
    }

    #[tokio::test]
    async fn test_delete() {
        let temp = TempDir::new().unwrap();
        let adapter = FilesystemAdapter::new(temp.path());

        adapter.write("test.txt", b"hello").await.unwrap();
        assert!(adapter.exists("test.txt").await);

        adapter.delete("test.txt").await.unwrap();
        assert!(!adapter.exists("test.txt").await);
    }

    #[tokio::test]
    async fn test_list() {
        let temp = TempDir::new().unwrap();
        let adapter = FilesystemAdapter::new(temp.path());

        adapter.write("file1.txt", b"1").await.unwrap();
        adapter.write("file2.txt", b"2").await.unwrap();

        let entries = adapter.list("").await.unwrap();
        assert_eq!(entries.len(), 2);
    }
}

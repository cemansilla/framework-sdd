use crate::filesystem::FilesystemAdapter;
use crate::storage_port::StorageError;
use sdd_core::model_chain::{ModelChainConfig, ModelHealthFile};
use tokio::fs;

/// Relative paths (inside `.sdd/`) for the model fallback system.
pub const MODEL_CHAIN_CONFIG_PATH: &str = "config/models.json";
pub const MODEL_HEALTH_PATH: &str = "config/model-health.json";

/// Outcome of rewriting an agent's pinned model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentModelChange {
    /// Previous `model:` value, or `None` when the agent was unpinned.
    pub previous: Option<String>,
    /// New `model:` value, or `None` when the agent was un-pinned (`inherit`).
    pub current: Option<String>,
}

impl FilesystemAdapter {
    /// Load the per-role fallback chains. A missing file is a loud error:
    /// `sdd init` installs the default config, so its absence means the
    /// project was initialized with an older framework version.
    pub async fn load_model_config(&self) -> Result<ModelChainConfig, StorageError> {
        let path = self.structure().sdd_dir().join(MODEL_CHAIN_CONFIG_PATH);
        if !path.exists() {
            return Err(StorageError::NotFound {
                path: format!(
                    ".sdd/{MODEL_CHAIN_CONFIG_PATH} (run 'sdd init' to install the default model chain config)"
                ),
            });
        }
        let content = fs::read_to_string(&path).await.map_err(StorageError::Io)?;
        serde_json::from_str(&content).map_err(|e| {
            StorageError::Serialization(format!(".sdd/{MODEL_CHAIN_CONFIG_PATH}: {e}"))
        })
    }

    /// Load the last-known model health. A missing file means "nothing
    /// observed yet": return an empty file instead of erroring.
    pub async fn load_model_health(&self) -> Result<ModelHealthFile, StorageError> {
        let path = self.structure().sdd_dir().join(MODEL_HEALTH_PATH);
        if !path.exists() {
            return Ok(ModelHealthFile::default());
        }
        let content = fs::read_to_string(&path).await.map_err(StorageError::Io)?;
        serde_json::from_str(&content)
            .map_err(|e| StorageError::Serialization(format!(".sdd/{MODEL_HEALTH_PATH}: {e}")))
    }

    /// Persist the health file atomically.
    pub async fn save_model_health(&self, health: &ModelHealthFile) -> Result<(), StorageError> {
        let path = self.structure().sdd_dir().join(MODEL_HEALTH_PATH);
        let json = serde_json::to_string_pretty(health)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        self.atomic_write(&path, json.as_bytes()).await
    }

    /// Read the pinned `model:` of `.opencode/agents/<role>.md`.
    ///
    /// Returns `None` when the agent has no pin (follows the global model).
    pub async fn agent_model(&self, role: &str) -> Result<Option<String>, StorageError> {
        let path = self.agent_file(role)?;
        let content = fs::read_to_string(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StorageError::NotFound {
                    path: agent_relative(role),
                }
            } else {
                StorageError::Io(e)
            }
        })?;
        Ok(extract_frontmatter_model(&content))
    }

    /// Rewrite the pinned `model:` of `.opencode/agents/<role>.md`.
    ///
    /// `Some(model)` pins the agent to that model; `None` removes the pin so
    /// the agent follows the globally active model (`inherit`). The write is
    /// atomic: a failed write never leaves a half-written agent file.
    pub async fn set_agent_model(
        &self,
        role: &str,
        model: Option<&str>,
    ) -> Result<AgentModelChange, StorageError> {
        let path = self.agent_file(role)?;
        let content = fs::read_to_string(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StorageError::NotFound {
                    path: agent_relative(role),
                }
            } else {
                StorageError::Io(e)
            }
        })?;
        let previous = extract_frontmatter_model(&content);
        let updated = rewrite_frontmatter_model(&content, model).ok_or_else(|| {
            StorageError::Serialization(format!(
                "{} has no frontmatter; refusing to guess where model: belongs",
                agent_relative(role)
            ))
        })?;
        if previous.as_deref() == model {
            return Ok(AgentModelChange {
                previous,
                current: model.map(str::to_string),
            });
        }
        self.atomic_write(&path, updated.as_bytes()).await?;
        Ok(AgentModelChange {
            previous,
            current: model.map(str::to_string),
        })
    }

    fn agent_file(&self, role: &str) -> Result<std::path::PathBuf, StorageError> {
        if role.is_empty() || role.contains('/') || role.contains('\\') || role.contains("..") {
            return Err(StorageError::Serialization(format!(
                "invalid agent role name: {role:?}"
            )));
        }
        Ok(self.structure().root.join(agent_relative(role)))
    }
}

fn agent_relative(role: &str) -> String {
    format!(".opencode/agents/{role}.md")
}

/// Extract the top-level `model:` value from a markdown frontmatter block.
///
/// Only lines inside the leading `--- ... ---` block are considered, so a
/// `model:` mention in the agent body can never be mistaken for the pin.
fn extract_frontmatter_model(content: &str) -> Option<String> {
    frontmatter_lines(content).find_map(|line| {
        line.strip_prefix("model:")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

/// Rewrite (or remove) the top-level `model:` line inside the frontmatter.
///
/// Returns `None` when the file has no frontmatter block.
fn rewrite_frontmatter_model(content: &str, model: Option<&str>) -> Option<String> {
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();
    if lines.first().map(String::as_str) != Some("---") {
        return None;
    }
    let end = lines.iter().skip(1).position(|l| l == "---")? + 1;
    match lines.iter().position(|l| l.starts_with("model:")) {
        Some(idx) if idx < end => match model {
            Some(value) => lines[idx] = format!("model: {value}"),
            None => {
                lines.remove(idx);
            }
        },
        _ => {
            if let Some(value) = model {
                lines.insert(1, format!("model: {value}"));
            }
        }
    }
    let mut out = lines.join("\n");
    if content.ends_with('\n') {
        out.push('\n');
    }
    Some(out)
}

/// Iterate the lines of the leading `--- ... ---` frontmatter block.
fn frontmatter_lines(content: &str) -> impl Iterator<Item = &str> {
    let mut lines = content.lines();
    let opened = lines.next() == Some("---");
    lines
        .take_while(move |_| opened)
        .take_while(|line| *line != "---")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdd_core::model_chain::{ModelHealth, RoleHealth};
    use tempfile::TempDir;

    async fn write_agent(dir: &TempDir, role: &str, content: &str) {
        let agents = dir.path().join(".opencode/agents");
        tokio::fs::create_dir_all(&agents).await.unwrap();
        tokio::fs::write(agents.join(format!("{role}.md")), content)
            .await
            .unwrap();
    }

    const ORCHESTRATOR_PINNED: &str = "---\nname: orchestrator\nmodel: opencode/longcat-2.5-preview-free\nmode: primary\n---\n\nBody mentions model: opencode/other here.\n";

    #[test]
    fn test_extract_frontmatter_model_ignores_body() {
        assert_eq!(
            extract_frontmatter_model(ORCHESTRATOR_PINNED),
            Some("opencode/longcat-2.5-preview-free".to_string())
        );
    }

    #[test]
    fn test_extract_frontmatter_model_missing() {
        assert_eq!(extract_frontmatter_model("---\nname: x\n---\n"), None);
        assert_eq!(extract_frontmatter_model("no frontmatter\n"), None);
    }

    #[tokio::test]
    async fn test_set_agent_model_replaces_pin() {
        let temp = TempDir::new().unwrap();
        write_agent(&temp, "orchestrator", ORCHESTRATOR_PINNED).await;
        let adapter = FilesystemAdapter::new(temp.path());

        let change = adapter
            .set_agent_model("orchestrator", Some("opencode/space-bunny-free"))
            .await
            .unwrap();
        assert_eq!(
            change.previous,
            Some("opencode/longcat-2.5-preview-free".to_string())
        );
        assert_eq!(
            change.current,
            Some("opencode/space-bunny-free".to_string())
        );
        assert_eq!(
            adapter.agent_model("orchestrator").await.unwrap(),
            Some("opencode/space-bunny-free".to_string())
        );
        // The body mention must survive untouched.
        let raw = tokio::fs::read_to_string(temp.path().join(".opencode/agents/orchestrator.md"))
            .await
            .unwrap();
        assert!(raw.contains("Body mentions model: opencode/other here."));
    }

    #[tokio::test]
    async fn test_set_agent_model_none_removes_pin() {
        let temp = TempDir::new().unwrap();
        write_agent(&temp, "coder", ORCHESTRATOR_PINNED).await;
        let adapter = FilesystemAdapter::new(temp.path());

        let change = adapter.set_agent_model("coder", None).await.unwrap();
        assert!(change.current.is_none());
        assert_eq!(adapter.agent_model("coder").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_set_agent_model_inserts_pin_when_missing() {
        let temp = TempDir::new().unwrap();
        write_agent(
            &temp,
            "tester",
            "---\nname: tester\nmode: subagent\n---\nBody.\n",
        )
        .await;
        let adapter = FilesystemAdapter::new(temp.path());

        adapter
            .set_agent_model("tester", Some("opencode/x-free"))
            .await
            .unwrap();
        assert_eq!(
            adapter.agent_model("tester").await.unwrap(),
            Some("opencode/x-free".to_string())
        );
    }

    #[tokio::test]
    async fn test_set_agent_model_noop_when_same() {
        let temp = TempDir::new().unwrap();
        write_agent(&temp, "coder", ORCHESTRATOR_PINNED).await;
        let adapter = FilesystemAdapter::new(temp.path());

        let change = adapter
            .set_agent_model("coder", Some("opencode/longcat-2.5-preview-free"))
            .await
            .unwrap();
        assert_eq!(change.previous, change.current);
    }

    #[tokio::test]
    async fn test_set_agent_model_missing_file() {
        let temp = TempDir::new().unwrap();
        let adapter = FilesystemAdapter::new(temp.path());
        let err = adapter
            .set_agent_model("ghost", Some("opencode/x"))
            .await
            .unwrap_err();
        assert!(matches!(err, StorageError::NotFound { .. }));
    }

    #[tokio::test]
    async fn test_set_agent_model_rejects_path_traversal() {
        let temp = TempDir::new().unwrap();
        let adapter = FilesystemAdapter::new(temp.path());
        let err = adapter
            .set_agent_model("../evil", Some("opencode/x"))
            .await
            .unwrap_err();
        assert!(matches!(err, StorageError::Serialization(_)));
    }

    #[tokio::test]
    async fn test_model_health_round_trip() {
        let temp = TempDir::new().unwrap();
        let adapter = FilesystemAdapter::new(temp.path());
        adapter.initialize("health-test").await.unwrap();

        // Missing file reads as empty, never as an error.
        assert!(adapter.load_model_health().await.unwrap().roles.is_empty());

        let mut file = ModelHealthFile::default();
        file.record(RoleHealth {
            role: "coder".to_string(),
            model: "opencode/big-pickle".to_string(),
            health: ModelHealth::Unhealthy,
            error: Some("429 rate limit".to_string()),
            checked_at: chrono::Utc::now(),
        });
        adapter.save_model_health(&file).await.unwrap();
        let back = adapter.load_model_health().await.unwrap();
        let entry = back.get("coder").unwrap();
        assert_eq!(entry.health, ModelHealth::Unhealthy);
        assert_eq!(entry.error.as_deref(), Some("429 rate limit"));
    }

    #[tokio::test]
    async fn test_load_model_config_missing_is_loud() {
        let temp = TempDir::new().unwrap();
        let adapter = FilesystemAdapter::new(temp.path());
        adapter.initialize("cfg-test").await.unwrap();
        // Simulate a project initialized with an older framework version.
        tokio::fs::remove_file(temp.path().join(".sdd/config/models.json"))
            .await
            .unwrap();

        let err = adapter.load_model_config().await.unwrap_err();
        assert!(matches!(err, StorageError::NotFound { .. }));
        assert!(err.to_string().contains("sdd init"));
    }

    #[tokio::test]
    async fn test_initialize_installs_default_model_config() {
        let temp = TempDir::new().unwrap();
        let adapter = FilesystemAdapter::new(temp.path());
        adapter.initialize("cfg-test").await.unwrap();

        let config = adapter.load_model_config().await.unwrap();
        assert_eq!(config.roles.len(), 6);
        let chain = sdd_core::model_chain::chain_for(&config, "orchestrator").unwrap();
        assert!(matches!(
            chain[0],
            sdd_core::model_chain::ChainEntry::Pinned(_)
        ));
        assert!(config.providers.contains_key("opencode"));
    }
}

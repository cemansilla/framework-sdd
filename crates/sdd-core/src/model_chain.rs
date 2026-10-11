use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Chain entry that un-pins the agent: it follows the globally active model
/// of the host tool instead of a concrete `provider/model`.
pub const INHERIT_ENTRY: &str = "inherit";

/// A concrete `provider/model` reference, e.g. `opencode/longcat-2.5-preview-free`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRef {
    pub provider: String,
    pub model: String,
}

/// One step of a role's fallback chain: either a pinned model or `inherit`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainEntry {
    Inherit,
    Pinned(ModelRef),
}

/// Per-role fallback chains, from `.sdd/config/models.json`.
///
/// Chains are ordered: the first entry is the preferred model, the last one
/// should be `inherit` so a role always lands on the host's active model when
/// every pinned model is down.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelChainConfig {
    /// Provider-agnostic connection defaults used by health probes.
    #[serde(default)]
    pub providers: HashMap<String, ProviderDef>,
    #[serde(default)]
    pub roles: HashMap<String, Vec<String>>,
}

/// How to reach one provider's API. Pure data: credential resolution and the
/// HTTP probe itself live in the CLI layer, never in the domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDef {
    /// Base URL, e.g. `https://opencode.ai/zen/v1`.
    pub api_base: String,
    /// Request shape: `openai-chat-completions` or `anthropic-messages`.
    #[serde(default = "default_request_style")]
    pub request_style: String,
    /// Credential sources, tried in order; the first that yields a token wins.
    pub auth: AuthSpec,
    /// Extra headers sent with every probe (e.g. client fingerprint).
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

fn default_request_style() -> String {
    "openai-chat-completions".to_string()
}

/// One or several credential sources, tried in order.
///
/// A list lets a provider degrade gracefully: an explicit API key first, a
/// host-tool session token file second. When nothing yields a token, probes
/// report `Untestable` instead of failing — the fallback walk then rotates by
/// chain order with a full audit trail rather than pretending to verify.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AuthSpec {
    Single(ProviderAuth),
    Ordered(Vec<ProviderAuth>),
}

impl AuthSpec {
    pub fn sources(&self) -> Vec<&ProviderAuth> {
        match self {
            AuthSpec::Single(source) => vec![source],
            AuthSpec::Ordered(sources) => sources.iter().collect(),
        }
    }
}

/// Credential source. Literals are intentionally unsupported: secrets never
/// belong in a versioned config file. When an object carries both keys, `env`
/// wins.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProviderAuth {
    /// Read the token from an environment variable.
    Env { env: String },
    /// Read the token from a JSON file (`~` expands to the home directory),
    /// following a dotted path such as `opencode.access`.
    File { file: String, token_path: String },
}

/// Last-known health of a role's model, from `.sdd/config/model-health.json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelHealth {
    /// A probe succeeded.
    Healthy,
    /// A probe reached the provider and the model is down (or the pin is bad).
    Unhealthy,
    /// Nothing verified: never probed, role unpinned, or no usable
    /// credential to probe with. Unknown is *not* failure.
    Unknown,
}

/// One recorded health observation per role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleHealth {
    pub role: String,
    /// Model that was probed, or `inherit` when the role is unpinned.
    pub model: String,
    pub health: ModelHealth,
    pub error: Option<String>,
    pub checked_at: DateTime<Utc>,
}

/// Health file: last-known state per role. Written by `sdd model health`,
/// `sdd model fallback` and `sdd model report`; read by `sdd model status`
/// and by agents at session start.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelHealthFile {
    #[serde(default)]
    pub roles: HashMap<String, RoleHealth>,
}

#[derive(Debug, Error)]
pub enum ModelChainError {
    #[error("unknown role '{role}'; known roles: {known}")]
    UnknownRole { role: String, known: String },
    #[error("role '{role}' has an empty fallback chain")]
    EmptyChain { role: String },
    #[error("invalid chain entry '{entry}': expected 'provider/model' or 'inherit'")]
    InvalidEntry { entry: String },
}

/// Parse one chain entry. `inherit` (exact, lowercase) is the terminal entry;
/// anything else must be `provider/model` with both parts non-empty.
pub fn parse_chain_entry(entry: &str) -> Result<ChainEntry, ModelChainError> {
    if entry == INHERIT_ENTRY {
        return Ok(ChainEntry::Inherit);
    }
    let mut parts = entry.splitn(2, '/');
    match (parts.next(), parts.next()) {
        (Some(provider), Some(model)) if !provider.is_empty() && !model.is_empty() => {
            Ok(ChainEntry::Pinned(ModelRef {
                provider: provider.to_string(),
                model: model.to_string(),
            }))
        }
        _ => Err(ModelChainError::InvalidEntry {
            entry: entry.to_string(),
        }),
    }
}

/// Look up a role's chain, validating every entry.
///
/// Errors loudly on unknown roles (listing the known ones) and on empty
/// chains, so misconfiguration can never fail silently.
pub fn chain_for(
    config: &ModelChainConfig,
    role: &str,
) -> Result<Vec<ChainEntry>, ModelChainError> {
    let raw = config.roles.get(role).ok_or_else(|| {
        let mut known: Vec<&str> = config.roles.keys().map(String::as_str).collect();
        known.sort_unstable();
        ModelChainError::UnknownRole {
            role: role.to_string(),
            known: known.join(", "),
        }
    })?;
    if raw.is_empty() {
        return Err(ModelChainError::EmptyChain {
            role: role.to_string(),
        });
    }
    raw.iter().map(|e| parse_chain_entry(e)).collect()
}

impl ModelHealthFile {
    /// Record an observation, replacing any previous one for the role.
    pub fn record(&mut self, entry: RoleHealth) {
        self.roles.insert(entry.role.clone(), entry);
    }

    pub fn get(&self, role: &str) -> Option<&RoleHealth> {
        self.roles.get(role)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> ModelChainConfig {
        let mut roles = HashMap::new();
        roles.insert(
            "orchestrator".to_string(),
            vec![
                "opencode/longcat-2.5-preview-free".to_string(),
                "opencode/space-bunny-free".to_string(),
                "inherit".to_string(),
            ],
        );
        roles.insert("coder".to_string(), vec!["inherit".to_string()]);
        ModelChainConfig {
            providers: HashMap::new(),
            roles,
        }
    }

    #[test]
    fn test_parse_pinned_entry() {
        let entry = parse_chain_entry("opencode/longcat-2.5-preview-free").unwrap();
        assert_eq!(
            entry,
            ChainEntry::Pinned(ModelRef {
                provider: "opencode".to_string(),
                model: "longcat-2.5-preview-free".to_string(),
            })
        );
    }

    #[test]
    fn test_parse_inherit_entry() {
        assert_eq!(parse_chain_entry("inherit").unwrap(), ChainEntry::Inherit);
    }

    #[test]
    fn test_parse_invalid_entries() {
        for bad in ["", "noprovider", "/nomodel", "provider/", "/"] {
            assert!(
                parse_chain_entry(bad).is_err(),
                "entry {bad:?} must be rejected"
            );
        }
    }

    #[test]
    fn test_chain_for_known_role() {
        let chain = chain_for(&sample_config(), "orchestrator").unwrap();
        assert_eq!(chain.len(), 3);
        assert!(matches!(chain[0], ChainEntry::Pinned(_)));
        assert_eq!(chain[2], ChainEntry::Inherit);
    }

    #[test]
    fn test_chain_for_unknown_role_lists_known_roles() {
        let err = chain_for(&sample_config(), "nope").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("nope"), "must name the unknown role");
        assert!(msg.contains("coder"), "must list known roles");
        assert!(msg.contains("orchestrator"), "must list known roles");
    }

    #[test]
    fn test_chain_for_empty_chain() {
        let mut config = sample_config();
        config.roles.insert("empty".to_string(), vec![]);
        let err = chain_for(&config, "empty").unwrap_err();
        assert!(matches!(err, ModelChainError::EmptyChain { .. }));
    }

    #[test]
    fn test_health_file_record_replaces_previous() {
        let mut file = ModelHealthFile::default();
        let stamp = Utc::now();
        file.record(RoleHealth {
            role: "coder".to_string(),
            model: "opencode/big-pickle".to_string(),
            health: ModelHealth::Unhealthy,
            error: Some("429 rate limit".to_string()),
            checked_at: stamp,
        });
        file.record(RoleHealth {
            role: "coder".to_string(),
            model: "opencode/big-pickle".to_string(),
            health: ModelHealth::Healthy,
            error: None,
            checked_at: stamp,
        });
        assert_eq!(file.get("coder").unwrap().health, ModelHealth::Healthy);
        assert!(file.get("missing").is_none());
    }

    #[test]
    fn test_config_round_trip() {
        let config = sample_config();
        let json = serde_json::to_string(&config).unwrap();
        let back: ModelChainConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.roles.len(), 2);
    }

    #[test]
    fn test_provider_auth_shapes() {
        let env: ProviderAuth = serde_json::from_str(r#"{"env": "OPENAI_API_KEY"}"#).unwrap();
        assert!(matches!(env, ProviderAuth::Env { .. }));
        let file: ProviderAuth =
            serde_json::from_str(r#"{"file": "~/.auth.json", "token_path": "a.b"}"#).unwrap();
        assert!(matches!(file, ProviderAuth::File { .. }));
        assert!(serde_json::from_str::<ProviderAuth>(r#"{"other": 1}"#).is_err());
    }

    #[test]
    fn test_auth_spec_single_or_ordered() {
        let single: AuthSpec = serde_json::from_str(r#"{"env": "X"}"#).unwrap();
        assert_eq!(single.sources().len(), 1);
        let ordered: AuthSpec =
            serde_json::from_str(r#"[{"env": "X"}, {"file": "f", "token_path": "a"}]"#).unwrap();
        assert_eq!(ordered.sources().len(), 2);
        let empty: AuthSpec = serde_json::from_str(r#"[]"#).unwrap();
        assert!(empty.sources().is_empty());
    }

    #[test]
    fn test_config_ignores_unknown_fields() {
        // Forward compatibility: the CLI may add fields later.
        let config: ModelChainConfig =
            serde_json::from_str(r#"{"roles": {}, "future": true}"#).unwrap();
        assert!(config.roles.is_empty());
    }
}

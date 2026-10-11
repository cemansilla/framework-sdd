use async_trait::async_trait;
use sdd_core::model_chain::{AuthSpec, ProviderAuth, ProviderDef};
use std::time::{Duration, Instant};

/// Request shapes understood by probes. Anything else is a loud config
/// error, never a silent skip.
pub const OPENAI_CHAT_COMPLETIONS: &str = "openai-chat-completions";
pub const ANTHROPIC_MESSAGES: &str = "anthropic-messages";

/// What a probe established about one `provider/model`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeVerdict {
    /// The provider answered 200: the model works right now.
    Healthy,
    /// The provider answered and the model is down (or the pin/credential is
    /// bad in a verified way): safe to skip in the walk.
    Unhealthy,
    /// Nothing could be verified: no usable credential, unreachable network,
    /// unknown shape. Unknown is *not* failure — the walk may still rotate
    /// to such a model, but must say so out loud.
    Untestable,
}

/// Outcome of probing one `provider/model`.
#[derive(Debug, Clone)]
pub struct ProbeOutcome {
    pub verdict: ProbeVerdict,
    pub error: Option<String>,
    pub latency_ms: u64,
}

impl ProbeOutcome {
    pub fn healthy(latency_ms: u64) -> Self {
        Self {
            verdict: ProbeVerdict::Healthy,
            error: None,
            latency_ms,
        }
    }

    pub fn untestable(error: impl Into<String>, latency_ms: u64) -> Self {
        Self {
            verdict: ProbeVerdict::Untestable,
            error: Some(error.into()),
            latency_ms,
        }
    }

    pub fn failed_status(status: u16, snippet: impl Into<String>, latency_ms: u64) -> Self {
        let mut error = format!("HTTP {status}");
        let snippet = snippet.into();
        if !snippet.is_empty() {
            error.push_str(&format!(": {snippet}"));
        }
        if status == 401 || status == 403 {
            error.push_str(" (check credentials: re-authenticate or refresh the token)");
        }
        Self {
            verdict: ProbeVerdict::Unhealthy,
            error: Some(error),
            latency_ms,
        }
    }
}

/// Probes model health over plain HTTP, without any host-tool involvement.
///
/// The trait exists so the fallback walk is testable offline with a mock;
/// `HttpProber` is the real, provider-agnostic implementation.
#[async_trait]
pub trait ModelProber: Send + Sync {
    async fn probe(&self, provider: &ProviderDef, model: &str) -> ProbeOutcome;
}

/// Real probe: one minimal completion request per provider request style.
pub struct HttpProber {
    client: reqwest::Client,
}

impl HttpProber {
    pub fn new(timeout: Duration) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder().timeout(timeout).build()?;
        Ok(Self { client })
    }
}

/// Resolve a bearer credential, trying each auth source in order.
///
/// Returns the first token found. When nothing yields a token, the caller
/// gets every observed reason — the probe then reports `Untestable`, never a
/// fabricated failure.
pub fn resolve_credentials(auth: &AuthSpec) -> Result<String, String> {
    let mut errors = Vec::new();
    for source in auth.sources() {
        match resolve_one(source) {
            Ok(token) if !token.trim().is_empty() => return Ok(token),
            Ok(_) => errors.push("empty token".to_string()),
            Err(error) => errors.push(error),
        }
    }
    if errors.is_empty() {
        return Err("no auth sources configured".to_string());
    }
    Err(errors.join("; "))
}

fn resolve_one(auth: &ProviderAuth) -> Result<String, String> {
    match auth {
        ProviderAuth::Env { env } => {
            std::env::var(env).map_err(|_| format!("env {env} is not set"))
        }
        ProviderAuth::File { file, token_path } => {
            let path = expand_home(file);
            let content =
                std::fs::read_to_string(&path).map_err(|e| format!("cannot read {path}: {e}"))?;
            let doc: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| format!("{path} is not valid JSON: {e}"))?;
            let mut current = &doc;
            for key in token_path.split('.') {
                current = current
                    .get(key)
                    .ok_or_else(|| format!("{path} has no JSON path '{token_path}'"))?;
            }
            current
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("{path} path '{token_path}' is not a string"))
        }
    }
}

fn expand_home(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        match std::env::var("HOME") {
            Ok(home) => return format!("{home}/{rest}"),
            Err(_) => return path.to_string(),
        }
    }
    path.to_string()
}

fn endpoint_path(style: &str) -> Result<&'static str, String> {
    match style {
        OPENAI_CHAT_COMPLETIONS => Ok("/chat/completions"),
        ANTHROPIC_MESSAGES => Ok("/v1/messages"),
        other => Err(format!(
            "unknown request_style '{other}' (supported: {OPENAI_CHAT_COMPLETIONS}, {ANTHROPIC_MESSAGES})"
        )),
    }
}

#[async_trait]
impl ModelProber for HttpProber {
    async fn probe(&self, provider: &ProviderDef, model: &str) -> ProbeOutcome {
        let started = Instant::now();
        let latency = || started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;

        let token = match resolve_credentials(&provider.auth) {
            Ok(token) => token,
            Err(error) => return ProbeOutcome::untestable(format!("auth: {error}"), latency()),
        };
        let path = match endpoint_path(&provider.request_style) {
            Ok(path) => path,
            Err(error) => return ProbeOutcome::untestable(error, latency()),
        };
        let url = format!("{}{path}", provider.api_base.trim_end_matches('/'));

        let mut request = match provider.request_style.as_str() {
            OPENAI_CHAT_COMPLETIONS => {
                self.client
                    .post(&url)
                    .bearer_auth(token)
                    .json(&serde_json::json!({
                        "model": model,
                        "messages": [{"role": "user", "content": "ping"}],
                        "max_tokens": 1,
                        "stream": false,
                    }))
            }
            ANTHROPIC_MESSAGES => self
                .client
                .post(&url)
                .header("x-api-key", token)
                .header("anthropic-version", "2023-06-01")
                .json(&serde_json::json!({
                    "model": model,
                    "max_tokens": 1,
                    "messages": [{"role": "user", "content": "ping"}],
                })),
            // endpoint_path already rejected anything else.
            _ => unreachable!(),
        };
        for (name, value) in &provider.headers {
            request = request.header(name, value);
        }

        let response = match request.send().await {
            Ok(response) => response,
            Err(error) if error.is_timeout() => {
                return ProbeOutcome::untestable("timeout waiting for provider", latency())
            }
            Err(error) => {
                return ProbeOutcome::untestable(format!("connection: {error}"), latency())
            }
        };
        let status = response.status().as_u16();
        if status == 200 {
            return ProbeOutcome::healthy(latency());
        }
        let snippet: String = response
            .text()
            .await
            .unwrap_or_default()
            .chars()
            .take(200)
            .collect();
        ProbeOutcome::failed_status(status, snippet.trim().to_string(), latency())
    }
}

/// Offline prober for the fallback walk tests.
///
/// `healthy` keys (`api_base/model`) probe Healthy; `untestable` keys probe
/// Untestable; anything else probes Unhealthy. An empty `healthy` map means
/// "everything is healthy" (unless listed in `untestable`).
#[cfg(test)]
pub struct MockProber {
    pub healthy: std::collections::HashMap<String, bool>,
    pub untestable: Vec<String>,
    pub failure: String,
}

#[cfg(test)]
#[async_trait]
impl ModelProber for MockProber {
    async fn probe(&self, provider: &ProviderDef, model: &str) -> ProbeOutcome {
        let key = format!("{}/{}", provider.api_base, model);
        if self.untestable.iter().any(|u| u == &key) {
            return ProbeOutcome::untestable("mock: no credential".to_string(), 1);
        }
        let healthy = if self.healthy.is_empty() {
            true
        } else {
            self.healthy.get(&key).copied().unwrap_or(false)
        };
        if healthy {
            ProbeOutcome::healthy(1)
        } else {
            ProbeOutcome {
                verdict: ProbeVerdict::Unhealthy,
                error: Some(self.failure.clone()),
                latency_ms: 1,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_credentials_prefers_first_working_source() {
        std::env::set_var("SDD_TEST_TOKEN_ENV", "secret-123");
        let auth: AuthSpec = serde_json::from_str(
            r#"[{"env": "SDD_TEST_TOKEN_DEFINITELY_MISSING"}, {"env": "SDD_TEST_TOKEN_ENV"}]"#,
        )
        .unwrap();
        assert_eq!(resolve_credentials(&auth).unwrap(), "secret-123");
        std::env::remove_var("SDD_TEST_TOKEN_ENV");
    }

    #[test]
    fn test_resolve_credentials_reports_every_source() {
        let auth: AuthSpec = serde_json::from_str(
            r#"[{"env": "SDD_TEST_TOKEN_DEFINITELY_MISSING"}, {"file": "/nonexistent/auth.json", "token_path": "a"}]"#,
        )
        .unwrap();
        let err = resolve_credentials(&auth).unwrap_err();
        assert!(
            err.contains("SDD_TEST_TOKEN_DEFINITELY_MISSING"),
            "got: {err}"
        );
        assert!(err.contains("/nonexistent/auth.json"), "got: {err}");
    }

    #[test]
    fn test_resolve_credentials_empty_is_loud() {
        let auth = AuthSpec::Ordered(vec![]);
        assert!(resolve_credentials(&auth).is_err());
    }

    #[test]
    fn test_resolve_token_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("auth.json");
        std::fs::write(&path, r#"{"a": {"b": "tok-xyz"}}"#).unwrap();
        let auth = AuthSpec::Single(ProviderAuth::File {
            file: path.to_string_lossy().to_string(),
            token_path: "a.b".to_string(),
        });
        assert_eq!(resolve_credentials(&auth).unwrap(), "tok-xyz");

        let bad_path = AuthSpec::Single(ProviderAuth::File {
            file: path.to_string_lossy().to_string(),
            token_path: "a.missing".to_string(),
        });
        assert!(resolve_credentials(&bad_path).is_err());
    }

    #[test]
    fn test_endpoint_path_rejects_unknown_styles() {
        assert_eq!(
            endpoint_path(OPENAI_CHAT_COMPLETIONS).unwrap(),
            "/chat/completions"
        );
        assert_eq!(endpoint_path(ANTHROPIC_MESSAGES).unwrap(), "/v1/messages");
        let err = endpoint_path("grpc-nope").unwrap_err();
        assert!(err.contains("supported"), "unexpected: {err}");
    }
}

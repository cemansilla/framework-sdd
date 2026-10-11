pub mod probe;

use anyhow::{bail, Context, Result};
use chrono::Utc;
use sdd_core::model_chain::{
    chain_for, ChainEntry, ModelChainConfig, ModelHealth, RoleHealth, INHERIT_ENTRY,
};
use sdd_storage::filesystem::FilesystemAdapter;
use std::path::PathBuf;
use std::time::Duration;

use probe::{HttpProber, ModelProber, ProbeVerdict};

/// Per-probe network timeout. Chains are short; worst case is a few timeouts.
const PROBE_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(clap::Subcommand)]
pub enum ModelAction {
    /// Show each role's chain, pinned model and last-known health (no network).
    Status,
    /// Probe the current model of a role (or every role with --all) and
    /// record the result. Exits non-zero when anything probed is down.
    Health {
        /// Role to probe (e.g. orchestrator, coder).
        #[arg(short, long)]
        role: Option<String>,
        /// Probe every role in the chain config.
        #[arg(long)]
        all: bool,
    },
    /// Walk a role's chain and pin the first healthy model.
    ///
    /// Touches only `.opencode/agents/<role>.md` and
    /// `.sdd/config/model-health.json`: tasks, code and git are never
    /// modified, so interrupted work resumes where it stopped. When every
    /// entry fails, nothing is written and the command exits non-zero.
    Fallback {
        /// Role whose model should fall back (e.g. orchestrator).
        #[arg(short, long)]
        role: String,
    },
    /// Record an observed model failure without probing.
    ///
    /// For agents (or humans) that just hit a provider error and want it on
    /// the record so the next session announces it instead of rediscovering
    /// it.
    Report {
        /// Role whose model failed (e.g. orchestrator).
        #[arg(short, long)]
        role: String,
        /// The provider error, verbatim when possible.
        #[arg(short, long)]
        error: String,
    },
}

pub async fn execute(path: Option<PathBuf>, action: ModelAction) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let adapter = FilesystemAdapter::new(&project_path);
    match action {
        ModelAction::Status => execute_status(&adapter).await,
        ModelAction::Health { role, all } => {
            let prober = HttpProber::new(PROBE_TIMEOUT)?;
            execute_health(&adapter, &prober, role, all).await
        }
        ModelAction::Fallback { role } => {
            let prober = HttpProber::new(PROBE_TIMEOUT)?;
            execute_fallback(&adapter, &prober, &role).await
        }
        ModelAction::Report { role, error } => execute_report(&adapter, &role, &error).await,
    }
}

async fn execute_status(adapter: &FilesystemAdapter) -> Result<()> {
    let config = adapter.load_model_config().await?;
    let health = adapter.load_model_health().await?;

    println!("Model fallback chains");
    println!("=====================");
    let mut roles: Vec<&String> = config.roles.keys().collect();
    roles.sort();
    for role in roles {
        let pinned = adapter
            .agent_model(role)
            .await
            .with_context(|| format!("cannot read .opencode/agents/{role}.md"))?;
        let chain = config.roles.get(role).cloned().unwrap_or_default();
        println!("\n[{role}]");
        println!(
            "  pinned:  {}",
            pinned
                .as_deref()
                .unwrap_or("inherit (follows global model)")
        );
        println!("  chain:   {}", chain.join(" → "));
        match health.get(role) {
            Some(entry) => println!(
                "  health:  {} ({}){}",
                entry.model,
                health_label(entry.health),
                entry
                    .error
                    .as_ref()
                    .map(|e| format!(" — {e}"))
                    .unwrap_or_default()
            ),
            None => println!("  health:  unknown (never probed; run 'sdd model health')"),
        }
    }
    Ok(())
}

fn health_label(health: ModelHealth) -> &'static str {
    match health {
        ModelHealth::Healthy => "healthy",
        ModelHealth::Unhealthy => "UNHEALTHY",
        ModelHealth::Unknown => "unknown",
    }
}

async fn execute_health(
    adapter: &FilesystemAdapter,
    prober: &dyn ModelProber,
    role: Option<String>,
    all: bool,
) -> Result<()> {
    let config = adapter.load_model_config().await?;
    let roles: Vec<String> = match (role, all) {
        (Some(role), _) => {
            chain_for(&config, &role)?;
            vec![role]
        }
        (None, true) => {
            let mut roles: Vec<String> = config.roles.keys().cloned().collect();
            roles.sort();
            roles
        }
        (None, false) => bail!("pass --role <role> or --all"),
    };

    let mut health_file = adapter.load_model_health().await?;
    let mut failures = 0usize;
    for role in &roles {
        let pinned = adapter
            .agent_model(role)
            .await
            .with_context(|| format!("cannot read .opencode/agents/{role}.md"))?;
        match pinned {
            None => {
                println!("[{role}] inherit: follows the global model (not probed)");
                health_file.record(RoleHealth {
                    role: role.clone(),
                    model: INHERIT_ENTRY.to_string(),
                    health: ModelHealth::Unknown,
                    error: None,
                    checked_at: Utc::now(),
                });
            }
            Some(model) => {
                let entry = parse_pinned(&model)
                    .with_context(|| format!("agent {role} has an invalid model pin {model:?}"))?;
                let provider = config.providers.get(&entry.provider).ok_or_else(|| {
                    anyhow::anyhow!(
                        "role {role} is pinned to unknown provider '{}'; add it to .sdd/config/models.json under \"providers\"",
                        entry.provider
                    )
                })?;
                print!("[{role}] probing {model} ... ");
                let outcome = prober.probe(provider, &entry.model).await;
                match outcome.verdict {
                    ProbeVerdict::Healthy => {
                        println!("healthy ({} ms)", outcome.latency_ms);
                    }
                    ProbeVerdict::Unhealthy => {
                        println!(
                            "UNHEALTHY: {}",
                            outcome.error.as_deref().unwrap_or("unknown error")
                        );
                        failures += 1;
                    }
                    ProbeVerdict::Untestable => {
                        println!(
                            "UNTESTABLE: {}",
                            outcome.error.as_deref().unwrap_or("unknown error")
                        );
                    }
                }
                health_file.record(RoleHealth {
                    role: role.clone(),
                    model,
                    health: match outcome.verdict {
                        ProbeVerdict::Healthy => ModelHealth::Healthy,
                        ProbeVerdict::Unhealthy => ModelHealth::Unhealthy,
                        ProbeVerdict::Untestable => ModelHealth::Unknown,
                    },
                    error: outcome.error,
                    checked_at: Utc::now(),
                });
            }
        }
    }
    adapter.save_model_health(&health_file).await?;

    if failures > 0 {
        bail!("{failures} model(s) verified unhealthy; run 'sdd model fallback --role <role>' to switch");
    }
    println!("No verified failures. (Entries reported UNTESTABLE could not be checked; see the reasons above.)");
    Ok(())
}

/// One step of a fallback walk, kept for the loud final report.
#[derive(Debug, Clone)]
pub struct FallbackAttempt {
    pub entry: String,
    pub verdict: ProbeVerdict,
    pub error: Option<String>,
}

/// Result of [`run_fallback`].
#[derive(Debug, Clone)]
pub struct FallbackReport {
    pub attempts: Vec<FallbackAttempt>,
    /// Model the role was pinned to, or `None` when the winner was `inherit`.
    pub winner: Option<String>,
    /// True when the winner was selected without verification (no usable
    /// credential to probe with). The report says so out loud.
    pub unverified: bool,
    /// Pin before the walk (`None` = was already unpinned).
    pub previous: Option<String>,
    /// True when every entry was verified down and nothing was written.
    pub exhausted: bool,
}

/// Walk a role's chain and pin the first healthy model.
///
/// Selection order: verified-healthy first; then the first unverifiable
/// candidate (rotation with an explicit unverified mark); then `inherit`
/// (un-pin); exhausted only when every pinned entry was verified down and no
/// `inherit` terminal exists. The currently pinned model is never re-selected
/// unless it probes healthy right now (transient recovery).
///
/// Config problems (unknown role, unknown provider) abort before any write.
/// When the walk exhausts the chain, the agent file is left untouched and the
/// report carries every observed error.
pub async fn run_fallback(
    adapter: &FilesystemAdapter,
    prober: &dyn ModelProber,
    role: &str,
) -> Result<FallbackReport> {
    let config = adapter.load_model_config().await?;
    let chain = chain_for(&config, role)?;
    let previous = adapter
        .agent_model(role)
        .await
        .with_context(|| format!("cannot read .opencode/agents/{role}.md"))?;

    let mut attempts = Vec::new();
    let mut winner: Option<String> = None;
    let mut unverified_candidate: Option<String> = None;
    let mut inherit_seen = false;

    for entry in &chain {
        match entry {
            ChainEntry::Inherit => {
                attempts.push(FallbackAttempt {
                    entry: INHERIT_ENTRY.to_string(),
                    verdict: ProbeVerdict::Untestable,
                    error: Some("terminal entry: un-pin and follow the global model".to_string()),
                });
                inherit_seen = true;
            }
            ChainEntry::Pinned(model_ref) => {
                let full = format!("{}/{}", model_ref.provider, model_ref.model);
                if previous.as_deref() == Some(full.as_str())
                    && attempts.iter().any(|a: &FallbackAttempt| a.entry == full)
                {
                    continue;
                }
                let provider = config.providers.get(&model_ref.provider).ok_or_else(|| {
                    anyhow::anyhow!(
                        "chain of role {role} references unknown provider '{}'; add it to .sdd/config/models.json under \"providers\"",
                        model_ref.provider
                    )
                })?;
                let outcome = prober.probe(provider, &model_ref.model).await;
                attempts.push(FallbackAttempt {
                    entry: full.clone(),
                    verdict: outcome.verdict,
                    error: outcome.error.clone(),
                });
                match outcome.verdict {
                    ProbeVerdict::Healthy => {
                        winner = Some(full);
                        break;
                    }
                    ProbeVerdict::Untestable => {
                        if unverified_candidate.is_none()
                            && previous.as_deref() != Some(full.as_str())
                        {
                            unverified_candidate = Some(full);
                        }
                    }
                    ProbeVerdict::Unhealthy => {}
                }
            }
        }
    }
    // Resolution order: verified-healthy (already selected above) first,
    // then rotation to the first unverifiable candidate, then `inherit`
    // (un-pin). Exhausted only when every pinned entry was verified down
    // and no `inherit` terminal exists.
    let (winner, unverified) = match (winner, unverified_candidate) {
        (Some(model), _) => (Some(model), false),
        (None, Some(candidate)) => (Some(candidate), true),
        (None, None) if inherit_seen => (None, false),
        (None, None) => {
            return Ok(FallbackReport {
                attempts,
                winner: None,
                unverified: false,
                previous,
                exhausted: true,
            });
        }
    };

    let change = adapter.set_agent_model(role, winner.as_deref()).await?;
    let mut health_file = adapter.load_model_health().await?;
    match &winner {
        Some(model) if !unverified => health_file.record(RoleHealth {
            role: role.to_string(),
            model: model.clone(),
            health: ModelHealth::Healthy,
            error: None,
            checked_at: Utc::now(),
        }),
        Some(model) => health_file.record(RoleHealth {
            role: role.to_string(),
            model: model.clone(),
            health: ModelHealth::Unknown,
            error: Some("selected unverified: no usable credential to probe with".to_string()),
            checked_at: Utc::now(),
        }),
        None => health_file.record(RoleHealth {
            role: role.to_string(),
            model: INHERIT_ENTRY.to_string(),
            health: ModelHealth::Unknown,
            error: None,
            checked_at: Utc::now(),
        }),
    }
    adapter.save_model_health(&health_file).await?;

    Ok(FallbackReport {
        attempts,
        winner,
        unverified,
        previous: change.previous,
        exhausted: false,
    })
}

async fn execute_fallback(
    adapter: &FilesystemAdapter,
    prober: &dyn ModelProber,
    role: &str,
) -> Result<()> {
    println!("Model fallback for role '{role}'");
    println!("===============================");
    let report = run_fallback(adapter, prober, role).await?;

    for attempt in &report.attempts {
        if attempt.entry == INHERIT_ENTRY {
            println!("  inherit → terminal entry: un-pin, follow the global model");
            continue;
        }
        match attempt.verdict {
            ProbeVerdict::Healthy => println!("  {} → healthy, selected", attempt.entry),
            ProbeVerdict::Unhealthy => println!(
                "  {} → FAILED: {}",
                attempt.entry,
                attempt.error.as_deref().unwrap_or("unknown error")
            ),
            ProbeVerdict::Untestable => println!(
                "  {} → UNTESTABLE: {}",
                attempt.entry,
                attempt.error.as_deref().unwrap_or("unknown error")
            ),
        }
    }

    if report.exhausted {
        bail!(
            "no healthy model for role '{role}': every pinned entry was verified down and the chain has no inherit terminal — nothing was changed. \
             Check credentials/network, extend the chain in .sdd/config/models.json, and retry."
        );
    }

    match (&report.previous, &report.winner) {
        (Some(previous), Some(winner)) if previous == winner => {
            println!("\nRole '{role}' stays on {winner} (verified healthy).");
        }
        _ => {
            println!(
                "\nRole '{role}': {} → {}.",
                report.previous.as_deref().unwrap_or("inherit (unpinned)"),
                report.winner.as_deref().unwrap_or("inherit (unpinned)")
            );
            if report.unverified {
                println!(
                    "NOTE: the new model could not be verified (see UNTESTABLE reasons above); \
                     it was selected by chain order. If it fails too, run fallback again to advance."
                );
            }
        }
    }
    println!("Tasks, code and git were not touched.");
    println!("Resume with 'sdd status' and continue the IN_PROGRESS task.");
    Ok(())
}

async fn execute_report(adapter: &FilesystemAdapter, role: &str, error: &str) -> Result<()> {
    let config: ModelChainConfig = adapter.load_model_config().await?;
    chain_for(&config, role)?;
    let pinned = adapter
        .agent_model(role)
        .await
        .with_context(|| format!("cannot read .opencode/agents/{role}.md"))?;

    let mut health_file = adapter.load_model_health().await?;
    health_file.record(RoleHealth {
        role: role.to_string(),
        model: pinned.unwrap_or_else(|| INHERIT_ENTRY.to_string()),
        health: ModelHealth::Unhealthy,
        error: Some(error.to_string()),
        checked_at: Utc::now(),
    });
    adapter.save_model_health(&health_file).await?;
    println!("Recorded failure for role '{role}': {error}");
    println!("The next session will announce it; switch with 'sdd model fallback --role {role}'.");
    Ok(())
}

fn parse_pinned(model: &str) -> Result<sdd_core::model_chain::ModelRef> {
    match sdd_core::model_chain::parse_chain_entry(model)? {
        ChainEntry::Pinned(model_ref) => Ok(model_ref),
        ChainEntry::Inherit => anyhow::bail!("'{model}' is not a concrete provider/model"),
    }
}

#[cfg(test)]
mod tests {
    use super::probe::MockProber;
    use super::*;
    use sdd_storage::models::MODEL_CHAIN_CONFIG_PATH;
    use std::collections::HashMap;
    use tempfile::TempDir;

    const CONFIG: &str = r#"{
      "providers": {
        "opencode": {
          "api_base": "https://probe.invalid",
          "request_style": "openai-chat-completions",
          "auth": {"env": "SDD_TEST_AUTH"}
        }
      },
      "roles": {
        "orchestrator": [
          "opencode/first-free",
          "opencode/second-free",
          "inherit"
        ],
        "coder": ["opencode/only-free"],
        "reviewer": [
          "opencode/r1",
          "opencode/r2",
          "opencode/r3",
          "inherit"
        ]
      }
    }"#;

    async fn project() -> (TempDir, FilesystemAdapter) {
        let temp = TempDir::new().unwrap();
        let adapter = FilesystemAdapter::new(temp.path());
        adapter.initialize("model-test").await.unwrap();
        let agents = temp.path().join(".opencode/agents");
        tokio::fs::create_dir_all(&agents).await.unwrap();
        tokio::fs::write(agents.join("orchestrator.md"), ORCHESTRATOR_AGENT)
            .await
            .unwrap();
        tokio::fs::write(agents.join("coder.md"), CODER_AGENT)
            .await
            .unwrap();
        tokio::fs::write(
            agents.join("reviewer.md"),
            "---\nname: reviewer\nmodel: opencode/r1\nmode: subagent\n---\nBody.\n",
        )
        .await
        .unwrap();
        tokio::fs::write(
            temp.path().join(".sdd").join(MODEL_CHAIN_CONFIG_PATH),
            CONFIG,
        )
        .await
        .unwrap();
        (temp, adapter)
    }

    const ORCHESTRATOR_AGENT: &str =
        "---\nname: orchestrator\nmodel: opencode/first-free\nmode: primary\n---\nBody.\n";
    const CODER_AGENT: &str = "---\nname: coder\nmode: subagent\n---\nBody.\n";

    fn mock_prober(healthy: &[&str]) -> MockProber {
        let mut map = HashMap::new();
        for key in healthy {
            map.insert(key.to_string(), true);
        }
        // Keys a mock understands: "<api_base>/<model>".
        MockProber {
            healthy: map
                .into_iter()
                .map(|(model, v)| (format!("https://probe.invalid/{model}"), v))
                .collect(),
            untestable: vec![],
            failure: "mock 429".to_string(),
        }
    }

    fn mock_prober_untestable(untestable: &[&str]) -> MockProber {
        MockProber {
            healthy: [("https://probe.invalid/__none__".to_string(), true)]
                .into_iter()
                .collect(),
            untestable: untestable
                .iter()
                .map(|model| format!("https://probe.invalid/{model}"))
                .collect(),
            failure: "mock 429".to_string(),
        }
    }

    #[tokio::test]
    async fn test_fallback_picks_first_healthy() {
        let (_temp, adapter) = project().await;
        let prober = mock_prober(&["second-free"]);

        let report = run_fallback(&adapter, &prober, "orchestrator")
            .await
            .unwrap();
        assert!(!report.exhausted);
        assert!(!report.unverified);
        assert_eq!(report.winner.as_deref(), Some("opencode/second-free"));
        assert_eq!(report.previous.as_deref(), Some("opencode/first-free"));
        assert_eq!(report.attempts.len(), 2);
        assert_eq!(report.attempts[0].verdict, ProbeVerdict::Unhealthy);
        assert_eq!(report.attempts[1].verdict, ProbeVerdict::Healthy);
        assert_eq!(
            adapter
                .agent_model("orchestrator")
                .await
                .unwrap()
                .as_deref(),
            Some("opencode/second-free")
        );
        let health = adapter.load_model_health().await.unwrap();
        assert_eq!(
            health.get("orchestrator").unwrap().health,
            ModelHealth::Healthy
        );
    }

    #[tokio::test]
    async fn test_fallback_first_entry_healthy_is_noop_switch() {
        let (_temp, adapter) = project().await;
        let prober = mock_prober(&["first-free"]);

        let report = run_fallback(&adapter, &prober, "orchestrator")
            .await
            .unwrap();
        assert!(!report.exhausted);
        assert_eq!(report.winner, report.previous);
        assert_eq!(report.attempts.len(), 1);
    }

    #[tokio::test]
    async fn test_fallback_lands_on_inherit_when_all_pinned_fail() {
        let (_temp, adapter) = project().await;
        let prober = MockProber {
            healthy: [("https://probe.invalid/unrelated".to_string(), true)]
                .into_iter()
                .collect(),
            untestable: vec![],
            failure: "mock 429".to_string(),
        };

        let report = run_fallback(&adapter, &prober, "orchestrator")
            .await
            .unwrap();
        assert!(!report.exhausted);
        assert_eq!(report.winner, None);
        assert_eq!(adapter.agent_model("orchestrator").await.unwrap(), None);
        assert_eq!(report.attempts.len(), 3);
    }

    #[tokio::test]
    async fn test_fallback_rotates_to_unverified_when_nothing_verifiable() {
        let (_temp, adapter) = project().await;
        // Current pin and next entry both untestable, no inherit reached yet:
        // rotation to the next entry, explicitly marked unverified.
        let prober = mock_prober_untestable(&["first-free", "second-free"]);

        // coder's chain is a single entry, so use a dedicated config shape:
        // orchestrator has inherit terminal — rotation candidate must win
        // over inherit.
        let report = run_fallback(&adapter, &prober, "orchestrator")
            .await
            .unwrap();
        assert!(!report.exhausted);
        assert!(report.unverified);
        assert_eq!(report.winner.as_deref(), Some("opencode/second-free"));
        assert_eq!(
            adapter
                .agent_model("orchestrator")
                .await
                .unwrap()
                .as_deref(),
            Some("opencode/second-free")
        );
        let health = adapter.load_model_health().await.unwrap();
        let entry = health.get("orchestrator").unwrap();
        assert_eq!(entry.health, ModelHealth::Unknown);
        assert!(entry.error.as_deref().unwrap().contains("unverified"));
    }

    #[tokio::test]
    async fn test_fallback_rotation_beats_inherit_terminal() {
        let (_temp, adapter) = project().await;
        // first-free (current): verified down. second-free: untestable.
        // Rotation to the unverified candidate must win over the inherit
        // terminal: a concrete model on a different rate-limit bucket beats
        // un-pinning to a possibly identical global.
        let mut prober = mock_prober_untestable(&["second-free"]);
        prober
            .healthy
            .insert("https://probe.invalid/first-free".to_string(), false);

        let report = run_fallback(&adapter, &prober, "orchestrator")
            .await
            .unwrap();
        assert!(!report.exhausted);
        assert!(report.unverified);
        assert_eq!(report.winner.as_deref(), Some("opencode/second-free"));
    }

    #[tokio::test]
    async fn test_fallback_prefers_verified_healthy_over_earlier_untestable() {
        let (_temp, adapter) = project().await;
        // r1 (current) and r2 untestable, r3 verified healthy: the walk must
        // skip both unverifiable entries and select r3.
        let mut prober = mock_prober_untestable(&["r1", "r2"]);
        prober
            .healthy
            .insert("https://probe.invalid/r3".to_string(), true);

        let report = run_fallback(&adapter, &prober, "reviewer").await.unwrap();
        assert!(!report.exhausted);
        assert!(!report.unverified);
        assert_eq!(report.winner.as_deref(), Some("opencode/r3"));
        assert_eq!(report.attempts.len(), 3);
    }

    #[tokio::test]
    async fn test_fallback_exhausted_chain_writes_nothing() {
        let (_temp, adapter) = project().await;
        // coder's chain has no inherit terminal: total failure possible.
        let prober = MockProber {
            healthy: [("https://probe.invalid/only-free".to_string(), false)]
                .into_iter()
                .collect(),
            untestable: vec![],
            failure: "mock 429".to_string(),
        };

        let report = run_fallback(&adapter, &prober, "coder").await.unwrap();
        assert!(report.exhausted);
        assert_eq!(report.winner, None);
        assert_eq!(report.attempts.len(), 1);
        assert_eq!(report.attempts[0].error.as_deref(), Some("mock 429"));
        // Agent file untouched (coder was unpinned) and no health recorded.
        assert_eq!(adapter.agent_model("coder").await.unwrap(), None);
        assert!(adapter.load_model_health().await.unwrap().roles.is_empty());
    }

    #[tokio::test]
    async fn test_fallback_unknown_role_is_loud() {
        let (_temp, adapter) = project().await;
        let prober = mock_prober(&[]);
        let err = run_fallback(&adapter, &prober, "ghost").await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("ghost"), "unexpected: {msg}");
        assert!(msg.contains("orchestrator"), "unexpected: {msg}");
    }

    #[tokio::test]
    async fn test_fallback_unknown_provider_aborts_before_writing() {
        let (_temp, adapter) = project().await;
        let agents = adapter.agent_model("orchestrator").await.unwrap();
        assert!(agents.is_some());

        // Rewrite the config so the chain references a missing provider.
        let bad = CONFIG.replace("\"opencode/first-free\"", "\"nope/some-model\"");
        tokio::fs::write(_temp.path().join(".sdd").join(MODEL_CHAIN_CONFIG_PATH), bad)
            .await
            .unwrap();
        let prober = mock_prober(&[]);
        let err = run_fallback(&adapter, &prober, "orchestrator")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("nope"), "unexpected: {err}");
        // Pin untouched.
        assert_eq!(
            adapter
                .agent_model("orchestrator")
                .await
                .unwrap()
                .as_deref(),
            Some("opencode/first-free")
        );
    }

    #[tokio::test]
    async fn test_report_records_failure() {
        let (_temp, adapter) = project().await;
        execute_report(&adapter, "orchestrator", "429 rate limit")
            .await
            .unwrap();
        let health = adapter.load_model_health().await.unwrap();
        let entry = health.get("orchestrator").unwrap();
        assert_eq!(entry.health, ModelHealth::Unhealthy);
        assert_eq!(entry.model, "opencode/first-free");
        assert_eq!(entry.error.as_deref(), Some("429 rate limit"));
    }

    #[tokio::test]
    async fn test_report_unknown_role_is_loud() {
        let (_temp, adapter) = project().await;
        let err = execute_report(&adapter, "ghost", "boom").await.unwrap_err();
        assert!(err.to_string().contains("ghost"));
    }
}

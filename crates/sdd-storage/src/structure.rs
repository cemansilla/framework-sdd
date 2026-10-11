use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Debug, Clone)]
pub struct SddStructure {
    pub root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SddDirectoryLayout {
    pub manifest: PathBuf,
    pub config: PathBuf,
    pub brief: PathBuf,
    pub discovery: PathBuf,
    pub requirements: PathBuf,
    pub domain: PathBuf,
    pub architecture: PathBuf,
    pub design: PathBuf,
    pub tasks: PathBuf,
    pub agents: PathBuf,
    pub skills: PathBuf,
    pub tests: PathBuf,
    pub changes: PathBuf,
    pub context: PathBuf,
    pub index: PathBuf,
}

impl SddStructure {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn sdd_dir(&self) -> PathBuf {
        self.root.join(".sdd")
    }

    pub fn layout(&self) -> SddDirectoryLayout {
        let sdd = self.sdd_dir();
        SddDirectoryLayout {
            manifest: sdd.join("manifest.json"),
            config: sdd.join("config"),
            brief: sdd.join("brief"),
            discovery: sdd.join("discovery"),
            requirements: sdd.join("requirements"),
            domain: sdd.join("domain"),
            architecture: sdd.join("architecture"),
            design: sdd.join("design"),
            tasks: sdd.join("tasks"),
            agents: sdd.join("agents"),
            skills: sdd.join("skills"),
            tests: sdd.join("tests"),
            changes: sdd.join("changes"),
            context: sdd.join("context"),
            index: sdd.join("index"),
        }
    }

    pub fn artifact_path(&self, category: &str, filename: &str) -> PathBuf {
        let sdd = self.sdd_dir();
        match category {
            "brief" => sdd.join("brief").join(filename),
            "discovery" => sdd.join("discovery").join(filename),
            "requirements" => sdd.join("requirements").join(filename),
            "domain" => sdd.join("domain").join(filename),
            "architecture" => sdd.join("architecture").join(filename),
            "design" => sdd.join("design").join(filename),
            "tasks" => sdd.join("tasks").join(filename),
            "agents" => sdd.join("agents").join(filename),
            "skills" => sdd.join("skills").join(filename),
            "tests" => sdd.join("tests").join(filename),
            "changes" => sdd.join("changes").join(filename),
            "context" => sdd.join("context").join(filename),
            "index" => sdd.join("index").join(filename),
            _ => sdd.join(filename),
        }
    }

    pub fn all_directories(&self) -> Vec<PathBuf> {
        let layout = self.layout();
        vec![
            self.sdd_dir(),
            layout.config,
            layout.brief,
            layout.discovery,
            layout.requirements,
            layout.domain,
            layout.architecture,
            layout.design,
            layout.tasks,
            layout.agents,
            layout.skills,
            layout.tests,
            layout.changes,
            layout.context,
            layout.index,
        ]
    }

    pub fn exists(&self) -> bool {
        self.sdd_dir().exists()
    }
}

pub fn default_templates() -> Vec<(PathBuf, &'static str)> {
    vec![
        (PathBuf::from("brief/brief.md"), BRIEF_TEMPLATE),
        (PathBuf::from("discovery/questions.md"), QUESTIONS_TEMPLATE),
        (
            PathBuf::from("discovery/assumptions.md"),
            ASSUMPTIONS_TEMPLATE,
        ),
        (PathBuf::from("discovery/risks.md"), RISKS_TEMPLATE),
        (
            PathBuf::from("architecture/architecture.md"),
            ARCHITECTURE_TEMPLATE,
        ),
        (PathBuf::from("architecture/adr/"), ""),
        (PathBuf::from("changes/CHANGELOG.md"), CHANGELOG_TEMPLATE),
        (PathBuf::from("config/project.md"), PROJECT_TEMPLATE),
        (PathBuf::from("config/models.json"), MODELS_TEMPLATE),
    ]
}

const BRIEF_TEMPLATE: &str = r#"# Project Brief

## Vision

## Problem Statement

## Value Proposition

## Scope

## Stakeholders
"#;

const QUESTIONS_TEMPLATE: &str = r#"# Questions

<!-- Track open questions here -->
"#;

const ASSUMPTIONS_TEMPLATE: &str = r#"# Assumptions

<!-- Track assumptions here -->
"#;

const RISKS_TEMPLATE: &str = r#"# Risks

<!-- Track identified risks here -->
"#;

const ARCHITECTURE_TEMPLATE: &str = r#"# Architecture

## Overview

## Components

## Interfaces

## Decisions
"#;

const CHANGELOG_TEMPLATE: &str = r#"# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]
"#;

const PROJECT_TEMPLATE: &str = r#"# Project Configuration

## Name

## Version

## Description
"#;

/// Parse the bundled default model chain config (the `config/models.json`
/// template installed by `sdd init`). Single source of truth for the default
/// chains; panics only if the shipped template is corrupt (guarded by test).
pub fn default_model_chain_config() -> sdd_core::model_chain::ModelChainConfig {
    serde_json::from_str(MODELS_TEMPLATE).expect("bundled models.json template is invalid")
}
const MODELS_TEMPLATE: &str = r#"{
  "providers": {
    "opencode": {
      "api_base": "https://opencode.ai/zen/v1",
      "request_style": "openai-chat-completions",
      "auth": [
        {"env": "OPENCODE_ZEN_API_KEY"},
        {"file": "~/.local/share/opencode/auth.json", "token_path": "openai.access"}
      ]
    }
  },
  "roles": {
    "orchestrator": [
      "opencode/longcat-2.5-preview-free",
      "opencode/space-bunny-free",
      "opencode/nemotron-3-ultra-free",
      "inherit"
    ],
    "architect": [
      "opencode/nemotron-3-ultra-free",
      "opencode/space-bunny-free",
      "inherit"
    ],
    "planner": [
      "opencode/ling-3.0-flash-fin-free",
      "opencode/space-bunny-free",
      "inherit"
    ],
    "coder": [
      "opencode/big-pickle",
      "opencode/space-bunny-free",
      "inherit"
    ],
    "tester": [
      "opencode/nemotron-3.5-lightning-free",
      "opencode/space-bunny-free",
      "inherit"
    ],
    "reviewer": [
      "opencode/muse-spark-1.3-contributor-free",
      "opencode/space-bunny-free",
      "inherit"
    ]
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdd_structure() {
        let structure = SddStructure::new("/tmp/test");
        assert_eq!(structure.sdd_dir(), PathBuf::from("/tmp/test/.sdd"));
    }

    #[test]
    fn test_layout() {
        let structure = SddStructure::new("/tmp/test");
        let layout = structure.layout();
        assert_eq!(
            layout.manifest,
            PathBuf::from("/tmp/test/.sdd/manifest.json")
        );
    }

    #[test]
    fn test_artifact_path() {
        let structure = SddStructure::new("/tmp/test");
        let path = structure.artifact_path("requirements", "REQ-001.md");
        assert_eq!(
            path,
            PathBuf::from("/tmp/test/.sdd/requirements/REQ-001.md")
        );
    }

    #[test]
    fn test_all_directories() {
        let structure = SddStructure::new("/tmp/test");
        let dirs = structure.all_directories();
        assert!(dirs.len() > 10);
    }

    #[test]
    fn test_default_templates() {
        let templates = default_templates();
        assert!(!templates.is_empty());
    }

    #[test]
    fn test_default_model_chain_config_is_valid() {
        use sdd_core::model_chain::{chain_for, ChainEntry, INHERIT_ENTRY};
        let config = default_model_chain_config();
        assert_eq!(config.roles.len(), 6);
        assert!(config.providers.contains_key("opencode"));
        for role in [
            "orchestrator",
            "architect",
            "planner",
            "coder",
            "tester",
            "reviewer",
        ] {
            let chain = chain_for(&config, role).unwrap();
            assert!(
                chain.len() >= 2,
                "role {role} needs a fallback, not a single model"
            );
            assert!(
                matches!(chain.last(), Some(ChainEntry::Inherit)),
                "role {role} chain must end with '{INHERIT_ENTRY}'"
            );
        }
        let head = chain_for(&config, "orchestrator").unwrap();
        assert!(matches!(head[0], ChainEntry::Pinned(_)));
    }
}

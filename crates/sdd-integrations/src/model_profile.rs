use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelProfile {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub provider: ModelProvider,
    pub capabilities: Vec<ModelCapability>,
    pub context_window: usize,
    pub max_output_tokens: usize,
    pub supported_formats: Vec<ContextFormat>,
    pub invocation_mechanism: InvocationMechanism,
    pub mcp_capabilities: McpCapabilities,
    pub limitations: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelProvider {
    OpenAI,
    Anthropic,
    Google,
    OpenCode,
    Warp,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelCapability {
    CodeGeneration,
    CodeReview,
    TestGeneration,
    Documentation,
    Refactoring,
    Debugging,
    Architecture,
    Planning,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContextFormat {
    Markdown,
    Json,
    Yaml,
    PlainText,
    Structured,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InvocationMechanism {
    ApiKey { endpoint: String },
    Mcp { server_url: String },
    Cli { command: String },
    WebSocket { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct McpCapabilities {
    pub supports_tools: bool,
    pub supports_resources: bool,
    pub supports_prompts: bool,
    pub supports_sampling: bool,
    pub supports_roots: bool,
}

impl ModelProfile {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        provider: ModelProvider,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            description: String::new(),
            provider,
            capabilities: Vec::new(),
            context_window: 4096,
            max_output_tokens: 2048,
            supported_formats: vec![ContextFormat::Markdown],
            invocation_mechanism: InvocationMechanism::ApiKey {
                endpoint: String::new(),
            },
            mcp_capabilities: McpCapabilities::default(),
            limitations: Vec::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self.updated_at = Utc::now();
        self
    }

    pub fn with_capabilities(mut self, capabilities: Vec<ModelCapability>) -> Self {
        self.capabilities = capabilities;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_context_window(mut self, window: usize) -> Self {
        self.context_window = window;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_max_output_tokens(mut self, tokens: usize) -> Self {
        self.max_output_tokens = tokens;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_supported_formats(mut self, formats: Vec<ContextFormat>) -> Self {
        self.supported_formats = formats;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_invocation_mechanism(mut self, mechanism: InvocationMechanism) -> Self {
        self.invocation_mechanism = mechanism;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_mcp_capabilities(mut self, capabilities: McpCapabilities) -> Self {
        self.mcp_capabilities = capabilities;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_limitations(mut self, limitations: Vec<String>) -> Self {
        self.limitations = limitations;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn supports_capability(&self, capability: &ModelCapability) -> bool {
        self.capabilities.contains(capability)
    }

    pub fn supports_format(&self, format: &ContextFormat) -> bool {
        self.supported_formats.contains(format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_profile_creation() {
        let profile = ModelProfile::new("gpt-4", "GPT-4", "1.0.0", ModelProvider::OpenAI);

        assert_eq!(profile.id, "gpt-4");
        assert_eq!(profile.name, "GPT-4");
        assert_eq!(profile.provider, ModelProvider::OpenAI);
    }

    #[test]
    fn test_model_profile_builder() {
        let profile = ModelProfile::new("claude-3", "Claude 3", "2.0.0", ModelProvider::Anthropic)
            .with_description("Advanced AI assistant")
            .with_capabilities(vec![
                ModelCapability::CodeGeneration,
                ModelCapability::CodeReview,
            ])
            .with_context_window(8192)
            .with_max_output_tokens(4096);

        assert_eq!(profile.description, "Advanced AI assistant");
        assert_eq!(profile.capabilities.len(), 2);
        assert_eq!(profile.context_window, 8192);
        assert_eq!(profile.max_output_tokens, 4096);
    }

    #[test]
    fn test_capability_support() {
        let profile = ModelProfile::new(
            "test",
            "Test",
            "1.0.0",
            ModelProvider::Custom("test".to_string()),
        )
        .with_capabilities(vec![ModelCapability::CodeGeneration]);

        assert!(profile.supports_capability(&ModelCapability::CodeGeneration));
        assert!(!profile.supports_capability(&ModelCapability::TestGeneration));
    }
}

use crate::model_profile::{
    ContextFormat, InvocationMechanism, McpCapabilities, ModelCapability, ModelProfile,
    ModelProvider,
};
use std::collections::HashMap;

pub fn create_opencode_profile() -> ModelProfile {
    ModelProfile::new("opencode", "OpenCode", "1.0.0", ModelProvider::OpenCode)
        .with_description("OpenCode AI development harness integration")
        .with_capabilities(vec![
            ModelCapability::CodeGeneration,
            ModelCapability::CodeReview,
            ModelCapability::TestGeneration,
            ModelCapability::Documentation,
            ModelCapability::Refactoring,
            ModelCapability::Debugging,
            ModelCapability::Architecture,
            ModelCapability::Planning,
        ])
        .with_context_window(16384)
        .with_max_output_tokens(8192)
        .with_supported_formats(vec![
            ContextFormat::Markdown,
            ContextFormat::Json,
            ContextFormat::Structured,
        ])
        .with_invocation_mechanism(InvocationMechanism::Mcp {
            server_url: "http://localhost:8080".to_string(),
        })
        .with_mcp_capabilities(McpCapabilities {
            supports_tools: true,
            supports_resources: true,
            supports_prompts: true,
            supports_sampling: true,
            supports_roots: true,
        })
        .with_limitations(vec![
            "Requires MCP server running".to_string(),
            "Context window limited by model".to_string(),
        ])
        .with_metadata("framework", "SDD")
        .with_metadata("type", "development-harness")
}

pub fn create_warp_profile() -> ModelProfile {
    ModelProfile::new("warp", "Warp", "1.0.0", ModelProvider::Warp)
        .with_description("Warp terminal AI integration")
        .with_capabilities(vec![
            ModelCapability::CodeGeneration,
            ModelCapability::CodeReview,
            ModelCapability::Documentation,
            ModelCapability::Planning,
        ])
        .with_context_window(8192)
        .with_max_output_tokens(4096)
        .with_supported_formats(vec![ContextFormat::Markdown, ContextFormat::PlainText])
        .with_invocation_mechanism(InvocationMechanism::Cli {
            command: "warp".to_string(),
        })
        .with_mcp_capabilities(McpCapabilities::default())
        .with_limitations(vec![
            "Terminal-based interface".to_string(),
            "Limited context window".to_string(),
        ])
        .with_metadata("framework", "SDD")
        .with_metadata("type", "terminal-integration")
}

pub fn get_builtin_profiles() -> HashMap<String, ModelProfile> {
    let mut profiles = HashMap::new();
    profiles.insert("opencode".to_string(), create_opencode_profile());
    profiles.insert("warp".to_string(), create_warp_profile());
    profiles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opencode_profile() {
        let profile = create_opencode_profile();

        assert_eq!(profile.id, "opencode");
        assert_eq!(profile.name, "OpenCode");
        assert_eq!(profile.provider, ModelProvider::OpenCode);
        assert!(profile.supports_capability(&ModelCapability::CodeGeneration));
        assert!(profile.mcp_capabilities.supports_tools);
    }

    #[test]
    fn test_warp_profile() {
        let profile = create_warp_profile();

        assert_eq!(profile.id, "warp");
        assert_eq!(profile.name, "Warp");
        assert_eq!(profile.provider, ModelProvider::Warp);
        assert!(profile.supports_capability(&ModelCapability::CodeGeneration));
    }

    #[test]
    fn test_builtin_profiles() {
        let profiles = get_builtin_profiles();

        assert!(profiles.contains_key("opencode"));
        assert!(profiles.contains_key("warp"));
        assert_eq!(profiles.len(), 2);
    }
}

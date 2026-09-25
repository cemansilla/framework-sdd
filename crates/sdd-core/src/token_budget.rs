use crate::agent::Agent;
use crate::context_bundle::TokenBudget;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudgetConfig {
    pub default_budget: usize,
    pub per_agent_overrides: Vec<(String, usize)>,
    pub per_model_overrides: Vec<(String, usize)>,
    pub safety_margin_percent: f64,
}

impl TokenBudgetConfig {
    pub fn new(default_budget: usize) -> Self {
        Self {
            default_budget,
            per_agent_overrides: Vec::new(),
            per_model_overrides: Vec::new(),
            safety_margin_percent: 0.1,
        }
    }

    pub fn with_agent_override(mut self, agent_name: &str, budget: usize) -> Self {
        self.per_agent_overrides
            .push((agent_name.to_string(), budget));
        self
    }

    pub fn with_model_override(mut self, model_name: &str, budget: usize) -> Self {
        self.per_model_overrides
            .push((model_name.to_string(), budget));
        self
    }

    pub fn with_safety_margin(mut self, margin_percent: f64) -> Self {
        self.safety_margin_percent = margin_percent;
        self
    }
}

impl Default for TokenBudgetConfig {
    fn default() -> Self {
        Self::new(4096)
    }
}

pub struct TokenBudgetCalculator {
    config: TokenBudgetConfig,
}

impl TokenBudgetCalculator {
    pub fn new(config: TokenBudgetConfig) -> Self {
        Self { config }
    }

    pub fn calculate_for_agent(&self, agent: &Agent) -> TokenBudget {
        let base_budget = self
            .config
            .per_agent_overrides
            .iter()
            .find(|(name, _)| name == &agent.name)
            .map(|(_, budget)| *budget)
            .unwrap_or(self.config.default_budget);

        let effective_budget =
            (base_budget as f64 * (1.0 - self.config.safety_margin_percent)) as usize;

        TokenBudget::new(effective_budget)
    }

    pub fn calculate_for_model(&self, model_name: &str) -> TokenBudget {
        let base_budget = self
            .config
            .per_model_overrides
            .iter()
            .find(|(name, _)| name == model_name)
            .map(|(_, budget)| *budget)
            .unwrap_or(self.config.default_budget);

        let effective_budget =
            (base_budget as f64 * (1.0 - self.config.safety_margin_percent)) as usize;

        TokenBudget::new(effective_budget)
    }

    pub fn calculate_for_task(&self, agent: Option<&Agent>, model: Option<&str>) -> TokenBudget {
        let base = if let Some(agent) = agent {
            self.calculate_for_agent(agent)
        } else if let Some(model) = model {
            self.calculate_for_model(model)
        } else {
            TokenBudget::new(self.config.default_budget)
        };

        let effective = (base.total as f64 * (1.0 - self.config.safety_margin_percent)) as usize;
        TokenBudget::new(effective)
    }
}

impl Default for TokenBudgetCalculator {
    fn default() -> Self {
        Self::new(TokenBudgetConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{Agent, AgentType};

    fn create_test_agent(name: &str) -> Agent {
        Agent::new(name, "Test Agent", AgentType::Custom("test".to_string()))
    }

    #[test]
    fn test_budget_config_creation() {
        let config = TokenBudgetConfig::new(8192);
        assert_eq!(config.default_budget, 8192);
    }

    #[test]
    fn test_budget_config_with_overrides() {
        let config = TokenBudgetConfig::new(4096)
            .with_agent_override("planner", 8192)
            .with_model_override("gpt-4", 16384);

        assert_eq!(config.per_agent_overrides.len(), 1);
        assert_eq!(config.per_model_overrides.len(), 1);
    }

    #[test]
    fn test_calculate_for_agent_default() {
        let calculator = TokenBudgetCalculator::default();
        let agent = create_test_agent("test");
        let budget = calculator.calculate_for_agent(&agent);

        assert!(budget.total > 0);
    }

    #[test]
    fn test_calculate_for_agent_override() {
        let config = TokenBudgetConfig::new(4096).with_agent_override("planner", 8192);
        let calculator = TokenBudgetCalculator::new(config);

        let agent = create_test_agent("planner");
        let budget = calculator.calculate_for_agent(&agent);

        assert!(budget.total > 4000);
    }

    #[test]
    fn test_calculate_with_safety_margin() {
        let config = TokenBudgetConfig::new(10000).with_safety_margin(0.2);
        let calculator = TokenBudgetCalculator::new(config);

        let agent = create_test_agent("test");
        let budget = calculator.calculate_for_agent(&agent);

        assert!(budget.total < 10000);
    }
}

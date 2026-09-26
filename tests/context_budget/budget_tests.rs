use sdd_core::*;
use uuid::Uuid;

#[test]
fn test_context_budget_creation() {
    let budget = TokenBudget::new(4000);
    assert_eq!(budget.total, 4000);
    assert_eq!(budget.used, 0);
    assert_eq!(budget.remaining(), 4000);
}

#[test]
fn test_context_budget_utilization() {
    let mut budget = TokenBudget::new(4000);
    budget.used = 2000;

    assert_eq!(budget.remaining(), 2000);
    assert_eq!(budget.utilization_percent(), 50.0);
}

#[test]
fn test_context_bundle_respects_budget() {
    let task_id = Uuid::new_v4();
    let budget = TokenBudget::new(5); // Very small budget
    let mut bundle = ContextBundle::new(task_id, budget);

    let fragment = ContextFragment::new(
        "This is a very long content that will definitely exceed the tiny budget of only 5 tokens which is extremely small".to_string(),
        ContextSource::Task,
        "test-id".to_string(),
        ContextPriority::P0,
        "Test".to_string(),
    );

    let result = bundle.add_fragment(fragment);
    assert!(result.is_err());
}

#[test]
fn test_context_bundle_multiple_fragments() {
    let task_id = Uuid::new_v4();
    let budget = TokenBudget::new(10000);
    let mut bundle = ContextBundle::new(task_id, budget);

    for i in 0..5 {
        let fragment = ContextFragment::new(
            format!("Content {}", i),
            ContextSource::Task,
            format!("test-id-{}", i),
            ContextPriority::P0,
            "Test".to_string(),
        );
        bundle.add_fragment(fragment).unwrap();
    }

    assert_eq!(bundle.fragment_count(), 5);
    assert!(bundle.total_tokens() > 0);
}

#[test]
fn test_context_prioritization() {
    let prioritizer = ContextPrioritizer::new();
    let mut budget = TokenBudget::new(1000);

    let fragments = vec![
        ContextFragment::new(
            "P0 content".to_string(),
            ContextSource::Task,
            "p0".to_string(),
            ContextPriority::P0,
            "Test".to_string(),
        ),
        ContextFragment::new(
            "P1 content".to_string(),
            ContextSource::Requirement,
            "p1".to_string(),
            ContextPriority::P1,
            "Test".to_string(),
        ),
        ContextFragment::new(
            "P2 content".to_string(),
            ContextSource::Documentation,
            "p2".to_string(),
            ContextPriority::P2,
            "Test".to_string(),
        ),
    ];

    let result = prioritizer.prioritize(fragments, &mut budget, None);
    assert!(!result.is_empty());

    // P0 should be first
    assert_eq!(result[0].priority, ContextPriority::P0);
}

#[test]
fn test_context_truncation() {
    let prioritizer = ContextPrioritizer::new();
    let task_id = Uuid::new_v4();
    let mut bundle = ContextBundle::new(task_id, TokenBudget::new(10000));

    for i in 0..10 {
        let fragment = ContextFragment::new(
            format!("Content {} with some text to increase token count", i),
            ContextSource::Task,
            format!("test-id-{}", i),
            ContextPriority::P2,
            "Test".to_string(),
        );
        bundle.add_fragment(fragment).unwrap();
    }

    let initial_tokens = bundle.total_tokens();
    let truncated = prioritizer.truncate_to_budget(&mut bundle, 50);

    assert!(bundle.total_tokens() <= 50);
    assert!(!truncated.is_empty());
    assert!(bundle.total_tokens() < initial_tokens);
}

#[test]
fn test_token_budget_calculator() {
    let config = TokenBudgetConfig::new(4096)
        .with_agent_override("planner", 8192)
        .with_model_override("gpt-4", 16384);

    let calculator = TokenBudgetCalculator::new(config);

    let agent = Agent::new(
        "planner",
        "Planner Agent",
        AgentType::Custom("planner".to_string()),
    );

    let budget = calculator.calculate_for_agent(&agent);
    assert!(budget.total > 4000);
}

#[test]
fn test_token_budget_with_safety_margin() {
    let config = TokenBudgetConfig::new(10000).with_safety_margin(0.2);
    let calculator = TokenBudgetCalculator::new(config);

    let agent = Agent::new("test", "Test Agent", AgentType::Custom("test".to_string()));

    let budget = calculator.calculate_for_agent(&agent);
    assert!(budget.total < 10000);
    assert!(budget.total > 7000);
}

#[test]
fn test_context_fragment_token_estimation() {
    let content = "This is a test content with multiple words to estimate tokens";
    let tokens = ContextFragment::estimate_tokens(content);

    assert!(tokens > 0);
    assert!(tokens < content.len());
}

#[test]
fn test_context_bundle_rendering() {
    let task_id = Uuid::new_v4();
    let budget = TokenBudget::new(4000);
    let mut bundle = ContextBundle::new(task_id, budget);

    let fragment = ContextFragment::new(
        "Test content".to_string(),
        ContextSource::Task,
        "test-id".to_string(),
        ContextPriority::P0,
        "Test reason".to_string(),
    );

    bundle.add_fragment(fragment).unwrap();

    let rendered = bundle.render();
    assert!(rendered.contains("Context Bundle"));
    assert!(rendered.contains("P0"));
    assert!(rendered.contains("test-id"));
}

#[test]
fn test_context_inspection() {
    let inspector = ContextInspector::new();
    let task_id = Uuid::new_v4();
    let mut bundle = ContextBundle::new(task_id, TokenBudget::new(4000));

    let fragment = ContextFragment::new(
        "Test content".to_string(),
        ContextSource::Task,
        "test-id".to_string(),
        ContextPriority::P0,
        "Test".to_string(),
    );

    bundle.add_fragment(fragment).unwrap();

    let inspection = inspector.inspect(&bundle);
    assert_eq!(inspection.fragment_count, 1);
    assert!(inspection.total_tokens > 0);
}

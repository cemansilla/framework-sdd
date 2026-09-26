use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{MarginBreakdown, MarginContext, MarginPolicy, Money};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginRule {
    pub id: Uuid,
    pub name: String,
    pub condition: MarginCondition,
    pub adjustment: MarginAdjustment,
    pub priority: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarginCondition {
    Always,
    WeightRange {
        min_kg: f64,
        max_kg: f64,
    },
    PriceRange {
        min_amount: Decimal,
        max_amount: Decimal,
    },
    Zone {
        zones: Vec<String>,
    },
}

impl MarginCondition {
    pub fn matches(&self, context: &MarginContext) -> bool {
        match self {
            MarginCondition::Always => true,
            MarginCondition::WeightRange { min_kg, max_kg } => {
                context.weight_kg >= *min_kg && context.weight_kg <= *max_kg
            }
            MarginCondition::PriceRange {
                min_amount,
                max_amount,
            } => {
                context.base_price.amount >= *min_amount && context.base_price.amount <= *max_amount
            }
            MarginCondition::Zone { zones } => {
                zones.contains(&context.origin_zone) || zones.contains(&context.destination_zone)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarginAdjustment {
    Percentage(f64),
    FixedAmount { amount: Decimal, currency: String },
}

impl MarginAdjustment {
    pub fn calculate(&self, base_price: &Money, _context: &MarginContext) -> Money {
        match self {
            MarginAdjustment::Percentage(percentage) => base_price.multiply(percentage / 100.0),
            MarginAdjustment::FixedAmount { amount, currency } => {
                Money::new(*amount, currency.clone())
            }
        }
    }
}

pub struct MarginEngine {
    rules: Vec<MarginRule>,
}

impl MarginEngine {
    #[allow(dead_code)]
    pub fn new(rules: Vec<MarginRule>) -> Self {
        Self { rules }
    }

    pub fn with_default_margin(default_percentage: f64) -> Self {
        let default_rule = MarginRule {
            id: Uuid::new_v4(),
            name: "Default margin".to_string(),
            condition: MarginCondition::Always,
            adjustment: MarginAdjustment::Percentage(default_percentage),
            priority: 1000, // Low priority (applied last)
            enabled: true,
        };
        Self {
            rules: vec![default_rule],
        }
    }
}

impl MarginPolicy for MarginEngine {
    fn apply(&self, base_price: Money, context: &MarginContext) -> (Money, MarginBreakdown) {
        let mut current_price = base_price.clone();
        let mut breakdown = MarginBreakdown::new(base_price);

        // Sort rules by priority (lower number = higher priority)
        let mut applicable_rules: Vec<&MarginRule> = self
            .rules
            .iter()
            .filter(|rule| rule.enabled && rule.condition.matches(context))
            .collect();

        applicable_rules.sort_by_key(|rule| rule.priority);

        for rule in applicable_rules {
            let adjustment = rule.adjustment.calculate(&current_price, context);
            current_price = current_price.add(&adjustment);
            breakdown.add_adjustment(rule.name.clone(), adjustment);
        }

        (current_price, breakdown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_margin_condition_always() {
        let condition = MarginCondition::Always;
        let context = MarginContext {
            weight_kg: 5.0,
            origin_zone: "BA".to_string(),
            destination_zone: "BA".to_string(),
            base_price: Money::from_f64(1000.0, "ARS"),
        };
        assert!(condition.matches(&context));
    }

    #[test]
    fn test_margin_condition_weight_range() {
        let condition = MarginCondition::WeightRange {
            min_kg: 5.0,
            max_kg: 10.0,
        };

        let context_in_range = MarginContext {
            weight_kg: 7.0,
            origin_zone: "BA".to_string(),
            destination_zone: "BA".to_string(),
            base_price: Money::from_f64(1000.0, "ARS"),
        };
        assert!(condition.matches(&context_in_range));

        let context_out_of_range = MarginContext {
            weight_kg: 15.0,
            origin_zone: "BA".to_string(),
            destination_zone: "BA".to_string(),
            base_price: Money::from_f64(1000.0, "ARS"),
        };
        assert!(!condition.matches(&context_out_of_range));
    }

    #[test]
    fn test_margin_engine_percentage() {
        let engine = MarginEngine::with_default_margin(10.0);

        let base_price = Money::from_f64(1000.0, "ARS");
        let context = MarginContext {
            weight_kg: 5.0,
            origin_zone: "BA".to_string(),
            destination_zone: "BA".to_string(),
            base_price: base_price.clone(),
        };

        let (final_price, breakdown) = engine.apply(base_price, &context);
        assert_eq!(final_price.amount, Decimal::from(1100));
        assert_eq!(breakdown.adjustments.len(), 1);
    }

    #[test]
    fn test_margin_engine_multiple_rules() {
        let rules = vec![
            MarginRule {
                id: Uuid::new_v4(),
                name: "Heavy package".to_string(),
                condition: MarginCondition::WeightRange {
                    min_kg: 10.0,
                    max_kg: 50.0,
                },
                adjustment: MarginAdjustment::Percentage(15.0),
                priority: 1,
                enabled: true,
            },
            MarginRule {
                id: Uuid::new_v4(),
                name: "Default margin".to_string(),
                condition: MarginCondition::Always,
                adjustment: MarginAdjustment::Percentage(10.0),
                priority: 1000,
                enabled: true,
            },
        ];

        let engine = MarginEngine::new(rules);

        let base_price = Money::from_f64(1000.0, "ARS");
        let context = MarginContext {
            weight_kg: 15.0, // In heavy package range
            origin_zone: "BA".to_string(),
            destination_zone: "BA".to_string(),
            base_price: base_price.clone(),
        };

        let (final_price, breakdown) = engine.apply(base_price, &context);
        // 1000 + 15% (150) + 10% of 1150 (115) = 1265
        assert_eq!(final_price.amount, Decimal::from(1265));
        assert_eq!(breakdown.adjustments.len(), 2);
    }
}

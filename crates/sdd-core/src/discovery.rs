use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: Uuid,
    pub question_id: String,
    pub title: String,
    pub description: String,
    pub status: QuestionStatus,
    pub answer: Option<Answer>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QuestionStatus {
    Open,
    Answered,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    pub content: String,
    pub answered_at: DateTime<Utc>,
}

impl Question {
    pub fn new(
        question_id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            question_id: question_id.into(),
            title: title.into(),
            description: description.into(),
            status: QuestionStatus::Open,
            answer: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn answer(mut self, content: impl Into<String>) -> Self {
        self.answer = Some(Answer {
            content: content.into(),
            answered_at: Utc::now(),
        });
        self.status = QuestionStatus::Answered;
        self.updated_at = Utc::now();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: Uuid,
    pub decision_id: String,
    pub title: String,
    pub context: String,
    pub decision: String,
    pub consequences: Vec<String>,
    pub alternatives: Vec<Alternative>,
    pub status: DecisionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecisionStatus {
    Proposed,
    Accepted,
    Superseded { by: String },
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub title: String,
    pub description: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

impl Decision {
    pub fn new(
        decision_id: impl Into<String>,
        title: impl Into<String>,
        context: impl Into<String>,
        decision: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            decision_id: decision_id.into(),
            title: title.into(),
            context: context.into(),
            decision: decision.into(),
            consequences: Vec::new(),
            alternatives: Vec::new(),
            status: DecisionStatus::Proposed,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_consequence(mut self, consequence: impl Into<String>) -> Self {
        self.consequences.push(consequence.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn add_alternative(mut self, alternative: Alternative) -> Self {
        self.alternatives.push(alternative);
        self.updated_at = Utc::now();
        self
    }

    pub fn accept(mut self) -> Self {
        self.status = DecisionStatus::Accepted;
        self.updated_at = Utc::now();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assumption {
    pub id: Uuid,
    pub assumption_id: String,
    pub title: String,
    pub description: String,
    pub validated: bool,
    pub validation_evidence: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Assumption {
    pub fn new(
        assumption_id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            assumption_id: assumption_id.into(),
            title: title.into(),
            description: description.into(),
            validated: false,
            validation_evidence: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn validate(mut self, evidence: impl Into<String>) -> Self {
        self.validated = true;
        self.validation_evidence = Some(evidence.into());
        self.updated_at = Utc::now();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub id: Uuid,
    pub risk_id: String,
    pub title: String,
    pub description: String,
    pub probability: RiskLevel,
    pub impact: RiskLevel,
    pub mitigation: Option<String>,
    pub status: RiskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskStatus {
    Identified,
    Mitigating,
    Resolved,
    Accepted,
}

impl Risk {
    pub fn new(
        risk_id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        probability: RiskLevel,
        impact: RiskLevel,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            risk_id: risk_id.into(),
            title: title.into(),
            description: description.into(),
            probability,
            impact,
            mitigation: None,
            status: RiskStatus::Identified,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_mitigation(mut self, mitigation: impl Into<String>) -> Self {
        self.mitigation = Some(mitigation.into());
        self.status = RiskStatus::Mitigating;
        self.updated_at = Utc::now();
        self
    }

    pub fn resolve(mut self) -> Self {
        self.status = RiskStatus::Resolved;
        self.updated_at = Utc::now();
        self
    }

    pub fn risk_score(&self) -> u8 {
        let prob = match self.probability {
            RiskLevel::Low => 1,
            RiskLevel::Medium => 2,
            RiskLevel::High => 3,
            RiskLevel::Critical => 4,
        };
        let imp = match self.impact {
            RiskLevel::Low => 1,
            RiskLevel::Medium => 2,
            RiskLevel::High => 3,
            RiskLevel::Critical => 4,
        };
        prob * imp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_creation() {
        let q = Question::new("Q-001", "API Design", "How should we design the API?");
        assert_eq!(q.status, QuestionStatus::Open);
        assert!(q.answer.is_none());
    }

    #[test]
    fn test_question_answer() {
        let q = Question::new("Q-001", "API Design", "How should we design the API?")
            .answer("Use REST with JSON");
        assert_eq!(q.status, QuestionStatus::Answered);
        assert!(q.answer.is_some());
    }

    #[test]
    fn test_decision_creation() {
        let d = Decision::new(
            "DEC-001",
            "Use Hexagonal",
            "Need clean architecture",
            "Adopt hexagonal architecture",
        );
        assert_eq!(d.status, DecisionStatus::Proposed);
    }

    #[test]
    fn test_decision_accept() {
        let d = Decision::new(
            "DEC-001",
            "Use Hexagonal",
            "Need clean architecture",
            "Adopt hexagonal architecture",
        )
        .accept();
        assert_eq!(d.status, DecisionStatus::Accepted);
    }

    #[test]
    fn test_assumption_validation() {
        let a = Assumption::new("A-001", "Performance", "System handles 1000 rps")
            .validate("Load test passed");
        assert!(a.validated);
        assert_eq!(a.validation_evidence, Some("Load test passed".to_string()));
    }

    #[test]
    fn test_risk_score() {
        let r = Risk::new(
            "R-001",
            "Data Loss",
            "Risk of data loss",
            RiskLevel::Medium,
            RiskLevel::High,
        );
        assert_eq!(r.risk_score(), 6);
    }
}

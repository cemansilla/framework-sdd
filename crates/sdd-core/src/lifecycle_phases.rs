use crate::discovery::{Assumption, Decision, Question, Risk};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainstormingSession {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub ideas: Vec<Idea>,
    pub status: BrainstormingStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Idea {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub category: IdeaCategory,
    pub priority: IdeaPriority,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IdeaCategory {
    Feature,
    Improvement,
    Technical,
    Process,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IdeaPriority {
    High,
    Medium,
    Low,
    Unprioritized,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BrainstormingStatus {
    Active,
    Completed,
    Archived,
}

impl BrainstormingSession {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            description: description.into(),
            ideas: Vec::new(),
            status: BrainstormingStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_idea(
        &mut self,
        title: impl Into<String>,
        description: impl Into<String>,
        category: IdeaCategory,
    ) -> &mut Idea {
        let idea = Idea {
            id: Uuid::new_v4(),
            title: title.into(),
            description: description.into(),
            category,
            priority: IdeaPriority::Unprioritized,
            tags: Vec::new(),
            created_at: Utc::now(),
        };
        self.ideas.push(idea);
        self.updated_at = Utc::now();
        self.ideas.last_mut().unwrap()
    }

    pub fn complete(&mut self) {
        self.status = BrainstormingStatus::Completed;
        self.updated_at = Utc::now();
    }

    pub fn archive(&mut self) {
        self.status = BrainstormingStatus::Archived;
        self.updated_at = Utc::now();
    }

    pub fn ideas_by_priority(&self) -> Vec<&Idea> {
        let mut sorted: Vec<&Idea> = self.ideas.iter().collect();
        sorted.sort_by(|a, b| {
            let priority_val = |p: &IdeaPriority| match p {
                IdeaPriority::High => 0,
                IdeaPriority::Medium => 1,
                IdeaPriority::Low => 2,
                IdeaPriority::Unprioritized => 3,
            };
            priority_val(&a.priority).cmp(&priority_val(&b.priority))
        });
        sorted
    }
}

impl Idea {
    pub fn with_priority(mut self, priority: IdeaPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brief {
    pub id: Uuid,
    pub title: String,
    pub vision: String,
    pub problem_statement: String,
    pub value_proposition: String,
    pub scope: BriefScope,
    pub stakeholders: Vec<Stakeholder>,
    pub constraints: Vec<String>,
    pub success_criteria: Vec<String>,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefScope {
    pub in_scope: Vec<String>,
    pub out_of_scope: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stakeholder {
    pub name: String,
    pub role: String,
    pub responsibilities: Vec<String>,
}

impl Brief {
    pub fn new(
        title: impl Into<String>,
        vision: impl Into<String>,
        problem_statement: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            vision: vision.into(),
            problem_statement: problem_statement.into(),
            value_proposition: String::new(),
            scope: BriefScope {
                in_scope: Vec::new(),
                out_of_scope: Vec::new(),
            },
            stakeholders: Vec::new(),
            constraints: Vec::new(),
            success_criteria: Vec::new(),
            version: 1,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_value_proposition(mut self, value_prop: impl Into<String>) -> Self {
        self.value_proposition = value_prop.into();
        self.updated_at = Utc::now();
        self
    }

    pub fn add_in_scope(mut self, item: impl Into<String>) -> Self {
        self.scope.in_scope.push(item.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn add_out_of_scope(mut self, item: impl Into<String>) -> Self {
        self.scope.out_of_scope.push(item.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn add_stakeholder(mut self, stakeholder: Stakeholder) -> Self {
        self.stakeholders.push(stakeholder);
        self.updated_at = Utc::now();
        self
    }

    pub fn add_constraint(mut self, constraint: impl Into<String>) -> Self {
        self.constraints.push(constraint.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn add_success_criterion(mut self, criterion: impl Into<String>) -> Self {
        self.success_criteria.push(criterion.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn bump_version(&mut self) {
        self.version += 1;
        self.updated_at = Utc::now();
    }
}

impl Stakeholder {
    pub fn new(name: impl Into<String>, role: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            role: role.into(),
            responsibilities: Vec::new(),
        }
    }

    pub fn with_responsibility(mut self, responsibility: impl Into<String>) -> Self {
        self.responsibilities.push(responsibility.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryContext {
    pub id: Uuid,
    pub questions: Vec<Question>,
    pub decisions: Vec<Decision>,
    pub assumptions: Vec<Assumption>,
    pub risks: Vec<Risk>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DiscoveryContext {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            questions: Vec::new(),
            decisions: Vec::new(),
            assumptions: Vec::new(),
            risks: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_question(&mut self, question: Question) {
        self.questions.push(question);
        self.updated_at = Utc::now();
    }

    pub fn add_decision(&mut self, decision: Decision) {
        self.decisions.push(decision);
        self.updated_at = Utc::now();
    }

    pub fn add_assumption(&mut self, assumption: Assumption) {
        self.assumptions.push(assumption);
        self.updated_at = Utc::now();
    }

    pub fn add_risk(&mut self, risk: Risk) {
        self.risks.push(risk);
        self.updated_at = Utc::now();
    }

    pub fn open_questions(&self) -> Vec<&Question> {
        use crate::discovery::QuestionStatus;
        self.questions
            .iter()
            .filter(|q| q.status == QuestionStatus::Open)
            .collect()
    }

    pub fn high_risks(&self) -> Vec<&Risk> {
        use crate::discovery::{RiskLevel, RiskStatus};
        self.risks
            .iter()
            .filter(|r| {
                matches!(r.status, RiskStatus::Identified | RiskStatus::Mitigating)
                    && matches!(r.probability, RiskLevel::High | RiskLevel::Critical)
            })
            .collect()
    }

    pub fn is_ready(&self) -> bool {
        self.open_questions().is_empty()
            && self.high_risks().is_empty()
            && !self.decisions.is_empty()
    }
}

impl Default for DiscoveryContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::{QuestionStatus, RiskLevel};

    #[test]
    fn test_brainstorming_session() {
        let mut session = BrainstormingSession::new("Sprint Planning", "Ideas for next sprint");
        let idea = session.add_idea(
            "Add dark mode",
            "Implement dark mode support",
            IdeaCategory::Feature,
        );
        idea.priority = IdeaPriority::High;

        assert_eq!(session.ideas.len(), 1);
        assert_eq!(session.status, BrainstormingStatus::Active);
    }

    #[test]
    fn test_ideas_by_priority() {
        let mut session = BrainstormingSession::new("Test", "Test session");
        session.add_idea(
            "Low",
            "Low priority",
            IdeaCategory::Other("test".to_string()),
        );
        session.add_idea("High", "High priority", IdeaCategory::Feature);
        session.add_idea("Medium", "Medium priority", IdeaCategory::Improvement);

        let sorted = session.ideas_by_priority();
        assert_eq!(sorted.len(), 3);
    }

    #[test]
    fn test_brief_creation() {
        let brief = Brief::new(
            "SDD Framework",
            "AI-powered development harness",
            "Context overload",
        )
        .with_value_proposition("Structured SDD workflow")
        .add_in_scope("CLI tool")
        .add_out_of_scope("Cloud hosting")
        .add_constraint("Must be local-first");

        assert_eq!(brief.title, "SDD Framework");
        assert_eq!(brief.scope.in_scope.len(), 1);
        assert_eq!(brief.constraints.len(), 1);
    }

    #[test]
    fn test_brief_versioning() {
        let mut brief = Brief::new("Test", "Vision", "Problem");
        assert_eq!(brief.version, 1);
        brief.bump_version();
        assert_eq!(brief.version, 2);
    }

    #[test]
    fn test_discovery_context() {
        let mut context = DiscoveryContext::new();
        let question = Question::new("Q-001", "API Design", "How to design the API?");
        context.add_question(question);

        assert_eq!(context.questions.len(), 1);
        assert_eq!(context.open_questions().len(), 1);
        assert!(!context.is_ready());
    }

    #[test]
    fn test_discovery_ready() {
        let mut context = DiscoveryContext::new();
        let mut question = Question::new("Q-001", "API Design", "How to design the API?");
        question.status = QuestionStatus::Answered;
        context.add_question(question);

        let decision = Decision::new("DEC-001", "Use REST", "Need API", "Use REST API").accept();
        context.add_decision(decision);

        assert!(context.is_ready());
    }

    #[test]
    fn test_high_risks() {
        let mut context = DiscoveryContext::new();
        let risk = Risk::new(
            "R-001",
            "Data Loss",
            "Risk of data loss",
            RiskLevel::High,
            RiskLevel::Critical,
        );
        context.add_risk(risk);

        assert_eq!(context.high_risks().len(), 1);
    }
}

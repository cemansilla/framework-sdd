pub mod agent;
pub mod agent_registry;
pub mod architecture;
pub mod artifact;
pub mod change;
pub mod discovery;
pub mod lifecycle;
pub mod lifecycle_engine;
pub mod lifecycle_phases;
pub mod planning;
pub mod project;
pub mod repository;
pub mod requirement;
pub mod resolution;
pub mod skill;
pub mod skill_registry;
pub mod task;
pub mod traceability;
pub mod verification;

pub use agent::{
    Agent, AgentAction, AgentPermissions, AgentStatus, AgentType, Capability, ContextLimits,
    OutputContract, OutputFormat, QualityGate, TaskType,
};
pub use agent_registry::AgentRegistry;
pub use architecture::{
    Adr, AdrStatus, Architecture, ArchitectureStyle, Component, Interface, Operation, PortType,
};
pub use artifact::{Artifact, ArtifactCategory, ArtifactOrigin, ArtifactRelation, RelationType};
pub use change::{
    Change, ChangeOrigin, ChangeStatus, ChangeType, ChangelogEntry, ChangelogItem, ImpactAnalysis,
    RiskLevel,
};
pub use discovery::{
    Alternative, Answer, Assumption, Decision, DecisionStatus, Question, QuestionStatus, Risk,
    RiskStatus,
};
pub use lifecycle::LifecycleState;
pub use lifecycle_engine::{LifecycleError, ProjectContext, StateTransition};
pub use lifecycle_phases::{
    BrainstormingSession, BrainstormingStatus, Brief, BriefScope, DiscoveryContext, Idea,
    IdeaCategory, IdeaPriority, Stakeholder,
};
pub use planning::{
    DecompositionStrategy, ImplementationPlan, PlanStatus, PlanningError, TaskDecomposition,
};
pub use project::{Project, ProjectConfig};
pub use repository::{
    ArchitectureRepository, ArtifactRepository, ChangeRepository, ProjectRepository,
    RepositoryError, RequirementRepository, TaskRepository, TraceabilityRepository,
    VerificationRepository,
};
pub use requirement::{
    AcceptanceCriterion, Priority, Requirement, RequirementStatus, RequirementType,
};
pub use resolution::{
    AgentResolver, ResolutionReasoning, ResolutionRequest, ResolutionResult, SkillResolver,
    TaskResolver,
};
pub use skill::{
    InputType, OutputType, Skill, SkillApplicability, SkillCompatibility, SkillExample, SkillInput,
    SkillOutput, SkillStatus, SkillType,
};
pub use skill_registry::{SkillRegistry, SkillResolutionError};
pub use task::{Effort, Task, TaskOutputs, TaskScope, TaskStatus};
pub use traceability::{EdgeType, GraphEdge, GraphNode, NodeType, TraceabilityGraph};
pub use verification::{
    Finding, FindingCategory, FindingSeverity, Review, ReviewVerdict, TestCase, TestResult,
    TestStatus, TestSuite, TestType, VerificationReport, VerificationStatus,
};

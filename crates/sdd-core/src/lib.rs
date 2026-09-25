pub mod agent;
pub mod agent_registry;
pub mod approval_gates;
pub mod architecture;
pub mod artifact;
pub mod branch_convention;
pub mod change;
pub mod change_detection;
pub mod change_history;
pub mod change_propagation;
pub mod change_rollback;
pub mod change_validation;
pub mod commit_metadata;
pub mod conflict_resolution;
pub mod context_bundle;
pub mod context_cache;
pub mod context_inspector;
pub mod context_prioritizer;
pub mod context_resolver;
pub mod discovery;
pub mod execution_history;
pub mod impact_analysis;
pub mod issue_tracker;
pub mod lifecycle;
pub mod lifecycle_engine;
pub mod lifecycle_phases;
pub mod orchestrator;
pub mod output_contract;
pub mod permission_engine;
pub mod planning;
pub mod project;
pub mod repository;
pub mod requirement;
pub mod resolution;
pub mod scope_resolver;
pub mod scope_validation;
pub mod semantic_ranker;
pub mod skill;
pub mod skill_registry;
pub mod task;
pub mod task_commit_mapping;
pub mod token_budget;
pub mod traceability;
pub mod verification;

pub use agent::{
    Agent, AgentAction, AgentPermissions, AgentStatus, AgentType, Capability, ContextLimits,
    OutputContract, OutputFormat, QualityGate, TaskType,
};
pub use agent_registry::AgentRegistry;
pub use approval_gates::{ApprovalError, ApprovalGate, ApprovalManager, ApprovalStatus};
pub use architecture::{
    Adr, AdrStatus, Architecture, ArchitectureStyle, Component, Interface, Operation, PortType,
};
pub use artifact::{Artifact, ArtifactCategory, ArtifactOrigin, ArtifactRelation, RelationType};
pub use branch_convention::{BranchConvention, BranchError, BranchInfo, BranchPattern, BranchType};
pub use change::{
    Change, ChangeOrigin, ChangeStatus, ChangeType, ChangelogEntry, ChangelogItem, ImpactAnalysis,
    RiskLevel,
};
pub use change_detection::{ChangeDetection, ChangeDetector, DetectedChange, DetectedChangeType};
pub use change_history::{ChangeHistory, HistoryAction, HistoryEntry};
pub use change_propagation::{
    ChangePropagator, PropagationAction, PropagationPlan, PropagationPriority, PropagationStatus,
    PropagationStep, StepStatus,
};
pub use change_rollback::{
    ChangeRollback, RollbackAction, RollbackPlan, RollbackResult, RollbackStatus, RollbackStep,
    RollbackStepStatus,
};
pub use change_validation::{
    ChangeValidator, ValidationCheck, ValidationResult as ChangeValidationResult,
};
pub use commit_metadata::{
    CommitError, CommitMessage, CommitMetadata, CommitMetadataExtractor, CommitType,
};
pub use conflict_resolution::{
    Conflict, ConflictResolution, ConflictResolver, ConflictType, ResolutionStrategy,
};
pub use context_bundle::{
    ContextBundle, ContextError, ContextFragment, ContextPriority, ContextSource, DiscardedElement,
    TokenBudget,
};
pub use context_cache::{CachedBundle, ContextCache};
pub use context_inspector::{ContextInspection, ContextInspector, FragmentInspection};
pub use context_prioritizer::ContextPrioritizer;
pub use context_resolver::ExplicitRelationResolver;
pub use discovery::{
    Alternative, Answer, Assumption, Decision, DecisionStatus, Question, QuestionStatus, Risk,
    RiskStatus,
};
pub use execution_history::{
    ExecutionHistory, ExecutionMetrics, ExecutionRecord, ExecutionRecordStatus,
};
pub use impact_analysis::{
    EffortEstimate, ImpactAnalyzer, ImpactLevel, ImpactNode, ImpactResult,
    PropagationStep as ImpactPropagationStep, RiskAssessment,
};
pub use issue_tracker::{
    GitHubAdapter, Issue, IssuePriority, IssueStatus, IssueTrackerAdapter, IssueTrackerError,
    IssueUpdate, JiraAdapter, MockIssueTracker,
};
pub use lifecycle::LifecycleState;
pub use lifecycle_engine::{LifecycleError, ProjectContext, StateTransition};
pub use lifecycle_phases::{
    BrainstormingSession, BrainstormingStatus, Brief, BriefScope, DiscoveryContext, Idea,
    IdeaCategory, IdeaPriority, Stakeholder,
};
pub use orchestrator::{
    ApprovalRequest, Checkpoint, ExecutionRequest, ExecutionResult, ExecutionStatus, Orchestrator,
    OrchestratorConfig, OrchestratorError, OrchestratorState, ValidationResult,
};
pub use output_contract::{OutputCheck, OutputContractValidator, OutputValidation};
pub use permission_engine::{PermissionAction, PermissionCheck, PermissionEngine};
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
pub use scope_resolver::{ScopeResolver, ScopeResult};
pub use scope_validation::{
    PreCommitHook, PrePushHook, ScopeValidationHook, ValidationError,
    ValidationResult as ScopeValidationResult, ValidationWarning,
};
pub use semantic_ranker::{RankedFragment, SemanticRanker};
pub use skill::{
    InputType, OutputType, Skill, SkillApplicability, SkillCompatibility, SkillExample, SkillInput,
    SkillOutput, SkillStatus, SkillType,
};
pub use skill_registry::{SkillRegistry, SkillResolutionError};
pub use task::{Effort, Task, TaskOutputs, TaskScope, TaskStatus};
pub use task_commit_mapping::{CommitReference, MappingError, TaskCommitMapper, TaskCommitMapping};
pub use token_budget::{TokenBudgetCalculator, TokenBudgetConfig};
pub use traceability::{EdgeType, GraphEdge, GraphNode, NodeType, TraceabilityGraph};
pub use verification::{
    Finding, FindingCategory, FindingSeverity, Review, ReviewVerdict, TestCase, TestResult,
    TestStatus, TestSuite, TestType, VerificationReport, VerificationStatus,
};

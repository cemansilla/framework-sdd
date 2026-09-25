use crate::agent::{Agent, AgentAction};
use crate::agent_registry::AgentRegistry;
use crate::context_bundle::{ContextBundle, ContextError, TokenBudget};
use crate::context_cache::ContextCache;
use crate::context_inspector::ContextInspector;
use crate::context_prioritizer::ContextPrioritizer;
use crate::context_resolver::ExplicitRelationResolver;
use crate::resolution::{ResolutionRequest, TaskResolver};
use crate::skill_registry::SkillRegistry;
use crate::task::{Task, TaskStatus};
use crate::traceability::TraceabilityGraph;
use crate::verification::ReviewVerdict;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrchestratorState {
    Idle,
    Resolving,
    BuildingContext,
    Executing,
    Validating,
    AwaitingApproval,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub task_id: Uuid,
    pub agent_id: Option<Uuid>,
    pub force: bool,
    pub dry_run: bool,
    pub skip_validation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub execution_id: Uuid,
    pub task_id: Uuid,
    pub agent_id: Uuid,
    pub status: ExecutionStatus,
    pub output: Option<String>,
    pub validation: Option<ValidationResult>,
    pub context_hash: String,
    pub tokens_used: usize,
    pub duration_ms: u64,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub error: Option<String>,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Success,
    Failed,
    Cancelled,
    AwaitingApproval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub verdict: ReviewVerdict,
    pub findings: Vec<String>,
    pub reviewed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub task_id: Uuid,
    pub state: OrchestratorState,
    pub context_bundle: Option<ContextBundle>,
    pub partial_output: Option<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum OrchestratorError {
    #[error("task not found: {0}")]
    TaskNotFound(Uuid),
    #[error("agent not found: {0}")]
    AgentNotFound(Uuid),
    #[error("task not ready: {0}")]
    TaskNotReady(Uuid),
    #[error("context error: {0}")]
    ContextError(#[from] ContextError),
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("approval required: {0}")]
    ApprovalRequired(String),
    #[error("execution failed: {0}")]
    ExecutionFailed(String),
    #[error("checkpoint not found: {0}")]
    CheckpointNotFound(Uuid),
    #[error("orchestrator busy")]
    Busy,
}

pub struct Orchestrator {
    state: OrchestratorState,
    agent_registry: AgentRegistry,
    skill_registry: SkillRegistry,
    graph: TraceabilityGraph,
    context_cache: ContextCache,
    checkpoints: HashMap<Uuid, Checkpoint>,
    execution_history: Vec<ExecutionResult>,
    pending_approvals: HashMap<Uuid, ApprovalRequest>,
    config: OrchestratorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub max_retries: u32,
    pub default_token_budget: usize,
    pub enable_caching: bool,
    pub enable_checkpoints: bool,
    pub approval_timeout_seconds: u64,
    pub cache_dir: PathBuf,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            default_token_budget: 4096,
            enable_caching: true,
            enable_checkpoints: true,
            approval_timeout_seconds: 3600,
            cache_dir: PathBuf::from(".sdd/cache"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub task_id: Uuid,
    pub action: AgentAction,
    pub reason: String,
    pub requested_at: DateTime<Utc>,
    pub resolved: bool,
    pub approved: Option<bool>,
    pub resolved_at: Option<DateTime<Utc>>,
}

impl Orchestrator {
    pub fn new(
        agent_registry: AgentRegistry,
        skill_registry: SkillRegistry,
        graph: TraceabilityGraph,
        config: OrchestratorConfig,
    ) -> Self {
        let context_cache = ContextCache::new(&config.cache_dir);

        Self {
            state: OrchestratorState::Idle,
            agent_registry,
            skill_registry,
            graph,
            context_cache,
            checkpoints: HashMap::new(),
            execution_history: Vec::new(),
            pending_approvals: HashMap::new(),
            config,
        }
    }

    pub fn state(&self) -> OrchestratorState {
        self.state
    }

    pub fn execute_task(
        &mut self,
        task: &mut Task,
        request: ExecutionRequest,
    ) -> Result<ExecutionResult, OrchestratorError> {
        if self.state != OrchestratorState::Idle && !request.force {
            return Err(OrchestratorError::Busy);
        }

        if task.status != TaskStatus::Ready && task.status != TaskStatus::InProgress {
            return Err(OrchestratorError::TaskNotReady(task.id));
        }

        let started_at = Utc::now();
        let execution_id = Uuid::new_v4();

        self.state = OrchestratorState::Resolving;

        let resolver = TaskResolver::new(&self.agent_registry, &self.skill_registry);
        let resolution_request = ResolutionRequest {
            task: task.clone(),
            required_capabilities: Vec::new(),
            preferred_skills: Vec::new(),
            language: None,
            phase: None,
        };

        let resolution = resolver.resolve(&resolution_request);

        let agent_id = request
            .agent_id
            .or(resolution.selected_agent)
            .ok_or_else(|| {
                OrchestratorError::ExecutionFailed("No suitable agent found".to_string())
            })?;

        let agent = self
            .agent_registry
            .get(agent_id)
            .ok_or(OrchestratorError::AgentNotFound(agent_id))?
            .clone();

        self.state = OrchestratorState::BuildingContext;

        let context_bundle = self.build_context(task, &agent)?;

        if request.dry_run {
            let inspector = ContextInspector::new();
            let _inspection = inspector.inspect(&context_bundle);
            let output = inspector.render_full(&context_bundle);

            self.state = OrchestratorState::Completed;
            return Ok(ExecutionResult {
                execution_id,
                task_id: task.id,
                agent_id,
                status: ExecutionStatus::Success,
                output: Some(output),
                validation: None,
                context_hash: context_bundle.hash.clone(),
                tokens_used: context_bundle.total_tokens(),
                duration_ms: 0,
                started_at,
                completed_at: Utc::now(),
                error: None,
                retry_count: 0,
            });
        }

        if self.config.enable_checkpoints {
            self.create_checkpoint(execution_id, task.id, &context_bundle)?;
        }

        self.state = OrchestratorState::Executing;

        task.status = TaskStatus::InProgress;

        let output = format!(
            "Agent '{}' executing task '{}'\nContext: {} tokens\nSkills: {:?}",
            agent.name,
            task.task_id,
            context_bundle.total_tokens(),
            resolution.selected_skills
        );

        self.state = OrchestratorState::Validating;

        let validation = if !request.skip_validation {
            Some(self.validate_output(task, &output)?)
        } else {
            None
        };

        task.status = TaskStatus::Done;
        self.state = OrchestratorState::Completed;

        let result = ExecutionResult {
            execution_id,
            task_id: task.id,
            agent_id,
            status: ExecutionStatus::Success,
            output: Some(output),
            validation,
            context_hash: context_bundle.hash.clone(),
            tokens_used: context_bundle.total_tokens(),
            duration_ms: Utc::now()
                .signed_duration_since(started_at)
                .num_milliseconds() as u64,
            started_at,
            completed_at: Utc::now(),
            error: None,
            retry_count: 0,
        };

        self.execution_history.push(result.clone());
        self.state = OrchestratorState::Idle;

        Ok(result)
    }

    fn build_context(
        &mut self,
        task: &Task,
        agent: &Agent,
    ) -> Result<ContextBundle, OrchestratorError> {
        if self.config.enable_caching {
            if let Some(cached) = self.context_cache.get(&task.id) {
                return Ok(cached);
            }
        }

        let mut budget = TokenBudget::new(
            agent.context_limits.max_tokens as usize,
        );

        let relation_resolver = ExplicitRelationResolver::new(&self.graph);
        let fragments = relation_resolver.resolve_task_context(task, &mut budget)?;

        let prioritizer = ContextPrioritizer::new();
        let prioritized = prioritizer.prioritize(fragments, &mut budget, Some(&task.title));

        let mut bundle = ContextBundle::new(task.id, budget);
        for fragment in prioritized {
            let _ = bundle.add_fragment(fragment);
        }

        if self.config.enable_caching {
            self.context_cache.put(bundle.clone(), vec![]);
        }

        Ok(bundle)
    }

    fn validate_output(
        &self,
        _task: &Task,
        output: &str,
    ) -> Result<ValidationResult, OrchestratorError> {
        let passed = !output.is_empty();

        Ok(ValidationResult {
            passed,
            verdict: if passed {
                ReviewVerdict::Approved
            } else {
                ReviewVerdict::Rejected
            },
            findings: if passed {
                vec!["Output validated successfully".to_string()]
            } else {
                vec!["Output is empty".to_string()]
            },
            reviewed_at: Utc::now(),
        })
    }

    fn create_checkpoint(
        &mut self,
        execution_id: Uuid,
        task_id: Uuid,
        context: &ContextBundle,
    ) -> Result<(), OrchestratorError> {
        let checkpoint = Checkpoint {
            id: Uuid::new_v4(),
            execution_id,
            task_id,
            state: self.state,
            context_bundle: Some(context.clone()),
            partial_output: None,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };

        self.checkpoints.insert(checkpoint.id, checkpoint);
        Ok(())
    }

    pub fn resume_from_checkpoint(
        &mut self,
        checkpoint_id: Uuid,
    ) -> Result<Checkpoint, OrchestratorError> {
        self.checkpoints
            .get(&checkpoint_id)
            .cloned()
            .ok_or(OrchestratorError::CheckpointNotFound(checkpoint_id))
    }

    pub fn request_approval(
        &mut self,
        execution_id: Uuid,
        task_id: Uuid,
        action: AgentAction,
        reason: String,
    ) -> Uuid {
        let approval = ApprovalRequest {
            id: Uuid::new_v4(),
            execution_id,
            task_id,
            action,
            reason,
            requested_at: Utc::now(),
            resolved: false,
            approved: None,
            resolved_at: None,
        };

        let id = approval.id;
        self.pending_approvals.insert(id, approval);
        self.state = OrchestratorState::AwaitingApproval;
        id
    }

    pub fn resolve_approval(
        &mut self,
        approval_id: Uuid,
        approved: bool,
    ) -> Result<(), OrchestratorError> {
        let approval = self
            .pending_approvals
            .get_mut(&approval_id)
            .ok_or_else(|| {
                OrchestratorError::ExecutionFailed(format!("Approval {} not found", approval_id))
            })?;

        approval.resolved = true;
        approval.approved = Some(approved);
        approval.resolved_at = Some(Utc::now());

        if !self.pending_approvals.values().any(|a| !a.resolved) {
            self.state = OrchestratorState::Executing;
        }

        Ok(())
    }

    pub fn get_pending_approvals(&self) -> Vec<&ApprovalRequest> {
        self.pending_approvals
            .values()
            .filter(|a| !a.resolved)
            .collect()
    }

    pub fn get_execution_history(&self) -> &[ExecutionResult] {
        &self.execution_history
    }

    pub fn get_execution(&self, execution_id: Uuid) -> Option<&ExecutionResult> {
        self.execution_history
            .iter()
            .find(|e| e.execution_id == execution_id)
    }

    pub fn retry_execution(
        &mut self,
        execution_id: Uuid,
        task: &mut Task,
    ) -> Result<ExecutionResult, OrchestratorError> {
        let prev = self
            .get_execution(execution_id)
            .ok_or_else(|| {
                OrchestratorError::ExecutionFailed(format!("Execution {} not found", execution_id))
            })?
            .clone();

        if prev.retry_count >= self.config.max_retries {
            return Err(OrchestratorError::ExecutionFailed(
                "Max retries exceeded".to_string(),
            ));
        }

        let request = ExecutionRequest {
            task_id: task.id,
            agent_id: Some(prev.agent_id),
            force: true,
            dry_run: false,
            skip_validation: false,
        };

        let mut result = self.execute_task(task, request)?;
        result.retry_count = prev.retry_count + 1;

        Ok(result)
    }

    pub fn cancel_execution(&mut self, execution_id: Uuid) -> Result<(), OrchestratorError> {
        if let Some(result) = self
            .execution_history
            .iter_mut()
            .find(|e| e.execution_id == execution_id)
        {
            result.status = ExecutionStatus::Cancelled;
            result.completed_at = Utc::now();
        }

        self.state = OrchestratorState::Idle;
        Ok(())
    }

    pub fn checkpoint_count(&self) -> usize {
        self.checkpoints.len()
    }

    pub fn execution_count(&self) -> usize {
        self.execution_history.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Task;

    fn create_test_orchestrator() -> Orchestrator {
        let agent_registry = AgentRegistry::with_built_in_agents();
        let skill_registry = SkillRegistry::with_built_in_skills();
        let graph = TraceabilityGraph::new();
        let config = OrchestratorConfig::default();

        Orchestrator::new(agent_registry, skill_registry, graph, config)
    }

    #[test]
    fn test_orchestrator_creation() {
        let orchestrator = create_test_orchestrator();
        assert_eq!(orchestrator.state(), OrchestratorState::Idle);
    }

    #[test]
    fn test_execute_task() {
        let mut orchestrator = create_test_orchestrator();
        let mut task = Task::new("TASK-001", "Test task", "A test task");
        task.status = TaskStatus::Ready;

        let request = ExecutionRequest {
            task_id: task.id,
            agent_id: None,
            force: false,
            dry_run: false,
            skip_validation: false,
        };

        let result = orchestrator.execute_task(&mut task, request);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.status, ExecutionStatus::Success);
        assert_eq!(result.task_id, task.id);
    }

    #[test]
    fn test_execute_task_not_ready() {
        let mut orchestrator = create_test_orchestrator();
        let mut task = Task::new("TASK-001", "Test task", "A test task");

        let request = ExecutionRequest {
            task_id: task.id,
            agent_id: None,
            force: false,
            dry_run: false,
            skip_validation: false,
        };

        let result = orchestrator.execute_task(&mut task, request);
        assert!(matches!(result, Err(OrchestratorError::TaskNotReady(_))));
    }

    #[test]
    fn test_dry_run() {
        let mut orchestrator = create_test_orchestrator();
        let mut task = Task::new("TASK-001", "Test task", "A test task");
        task.status = TaskStatus::Ready;

        let request = ExecutionRequest {
            task_id: task.id,
            agent_id: None,
            force: false,
            dry_run: true,
            skip_validation: false,
        };

        let result = orchestrator.execute_task(&mut task, request).unwrap();
        assert_eq!(result.status, ExecutionStatus::Success);
        assert!(result.output.unwrap().contains("Context Bundle"));
    }

    #[test]
    fn test_approval_flow() {
        let mut orchestrator = create_test_orchestrator();

        let approval_id = orchestrator.request_approval(
            Uuid::new_v4(),
            Uuid::new_v4(),
            AgentAction::DeleteFile,
            "Need approval to delete file".to_string(),
        );

        assert_eq!(orchestrator.state(), OrchestratorState::AwaitingApproval);
        assert_eq!(orchestrator.get_pending_approvals().len(), 1);

        orchestrator.resolve_approval(approval_id, true).unwrap();
        assert_eq!(orchestrator.state(), OrchestratorState::Executing);
    }

    #[test]
    fn test_execution_history() {
        let mut orchestrator = create_test_orchestrator();
        let mut task = Task::new("TASK-001", "Test task", "A test task");
        task.status = TaskStatus::Ready;

        let request = ExecutionRequest {
            task_id: task.id,
            agent_id: None,
            force: false,
            dry_run: false,
            skip_validation: false,
        };

        orchestrator.execute_task(&mut task, request).unwrap();
        assert_eq!(orchestrator.execution_count(), 1);
    }

    #[test]
    fn test_checkpoint_creation() {
        let mut orchestrator = create_test_orchestrator();
        let mut task = Task::new("TASK-001", "Test task", "A test task");
        task.status = TaskStatus::Ready;

        let request = ExecutionRequest {
            task_id: task.id,
            agent_id: None,
            force: false,
            dry_run: false,
            skip_validation: false,
        };

        orchestrator.execute_task(&mut task, request).unwrap();
        assert!(orchestrator.checkpoint_count() > 0);
    }

    #[test]
    fn test_cancel_execution() {
        let mut orchestrator = create_test_orchestrator();
        let execution_id = Uuid::new_v4();

        orchestrator.cancel_execution(execution_id).unwrap();
        assert_eq!(orchestrator.state(), OrchestratorState::Idle);
    }
}

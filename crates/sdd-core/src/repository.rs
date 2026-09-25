use crate::architecture::Architecture;
use crate::artifact::Artifact;
use crate::change::Change;
use crate::project::Project;
use crate::requirement::Requirement;
use crate::task::Task;
use crate::traceability::TraceabilityGraph;
use crate::verification::VerificationReport;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("not found: {entity} with id {id}")]
    NotFound { entity: String, id: String },
    #[error("already exists: {entity} with id {id}")]
    AlreadyExists { entity: String, id: String },
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("lock error: {0}")]
    Lock(String),
    #[error("migration error: {0}")]
    Migration(String),
}

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn get(&self, id: Uuid) -> Result<Project, RepositoryError>;
    async fn save(&self, project: &Project) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
    async fn list(&self) -> Result<Vec<Project>, RepositoryError>;
}

#[async_trait]
pub trait ArtifactRepository: Send + Sync {
    async fn get(&self, id: Uuid) -> Result<Artifact, RepositoryError>;
    async fn get_by_path(&self, path: &str) -> Result<Artifact, RepositoryError>;
    async fn save(&self, artifact: &Artifact) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
    async fn list_by_category(&self, category: &str) -> Result<Vec<Artifact>, RepositoryError>;
    async fn list_all(&self) -> Result<Vec<Artifact>, RepositoryError>;
}

#[async_trait]
pub trait RequirementRepository: Send + Sync {
    async fn get(&self, id: Uuid) -> Result<Requirement, RepositoryError>;
    async fn get_by_id(&self, req_id: &str) -> Result<Requirement, RepositoryError>;
    async fn save(&self, requirement: &Requirement) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
    async fn list_all(&self) -> Result<Vec<Requirement>, RepositoryError>;
}

#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn get(&self, id: Uuid) -> Result<Task, RepositoryError>;
    async fn get_by_id(&self, task_id: &str) -> Result<Task, RepositoryError>;
    async fn save(&self, task: &Task) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
    async fn list_all(&self) -> Result<Vec<Task>, RepositoryError>;
    async fn list_by_status(&self, status: &str) -> Result<Vec<Task>, RepositoryError>;
}

#[async_trait]
pub trait ArchitectureRepository: Send + Sync {
    async fn get(&self, id: Uuid) -> Result<Architecture, RepositoryError>;
    async fn save(&self, architecture: &Architecture) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait TraceabilityRepository: Send + Sync {
    async fn get(&self) -> Result<TraceabilityGraph, RepositoryError>;
    async fn save(&self, graph: &TraceabilityGraph) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait VerificationRepository: Send + Sync {
    async fn get(&self, id: Uuid) -> Result<VerificationReport, RepositoryError>;
    async fn save(&self, report: &VerificationReport) -> Result<(), RepositoryError>;
    async fn list_by_task(&self, task_id: &str)
        -> Result<Vec<VerificationReport>, RepositoryError>;
}

#[async_trait]
pub trait ChangeRepository: Send + Sync {
    async fn get(&self, id: Uuid) -> Result<Change, RepositoryError>;
    async fn get_by_id(&self, change_id: &str) -> Result<Change, RepositoryError>;
    async fn save(&self, change: &Change) -> Result<(), RepositoryError>;
    async fn list_all(&self) -> Result<Vec<Change>, RepositoryError>;
}

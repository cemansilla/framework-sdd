pub mod changelog;
pub mod filesystem;
pub mod lock;
pub mod manifest;
pub mod migration;
pub mod models;
pub mod project_templates;
pub mod storage_port;
pub mod structure;

pub use changelog::{parse_changelog, ChangelogEntry};
pub use filesystem::FilesystemAdapter;
pub use lock::{FileLock, LockError, LockInfo, LockManager};
pub use manifest::{Manifest, CURRENT_FORMAT_VERSION};
pub use migration::{Migration, MigrationError, MigrationFn, MigrationManager};
pub use models::{AgentModelChange, MODEL_CHAIN_CONFIG_PATH, MODEL_HEALTH_PATH};
pub use storage_port::{StorageError, StoragePort};
pub use structure::{SddDirectoryLayout, SddStructure};

use crate::manifest::{Manifest, CURRENT_FORMAT_VERSION};
use crate::storage_port::StorageError;
use async_trait::async_trait;

pub struct Migration {
    pub version: u32,
    pub description: String,
    pub up: Box<dyn MigrationFn>,
}

impl std::fmt::Debug for Migration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Migration")
            .field("version", &self.version)
            .field("description", &self.description)
            .finish()
    }
}

#[async_trait]
pub trait MigrationFn: Send + Sync {
    async fn execute(&self, manifest: &mut Manifest) -> Result<(), MigrationError>;
}

#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("migration failed: {0}")]
    Failed(String),
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("no migration path from {from} to {to}")]
    NoPath { from: u32, to: u32 },
}

#[derive(Debug)]
pub struct MigrationManager {
    migrations: Vec<Migration>,
}

impl MigrationManager {
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    pub fn register<F>(&mut self, version: u32, description: impl Into<String>, migration: F)
    where
        F: MigrationFn + 'static,
    {
        self.migrations.push(Migration {
            version,
            description: description.into(),
            up: Box::new(migration),
        });

        self.migrations.sort_by_key(|a| a.version);
    }

    pub fn pending_migrations(&self, manifest: &Manifest) -> Vec<&Migration> {
        self.migrations
            .iter()
            .filter(|m| m.version > manifest.format_version)
            .collect()
    }

    pub async fn run_migrations(
        &self,
        manifest: &mut Manifest,
    ) -> Result<Vec<u32>, MigrationError> {
        let mut applied = Vec::new();

        let pending = self.pending_migrations(manifest);
        if pending.is_empty() {
            return Ok(applied);
        }

        for migration in pending {
            migration.up.execute(manifest).await?;
            manifest.apply_migration(migration.version);
            applied.push(migration.version);
        }

        Ok(applied)
    }

    pub fn current_version(&self) -> u32 {
        self.migrations
            .last()
            .map(|m| m.version)
            .unwrap_or(CURRENT_FORMAT_VERSION)
    }

    pub fn needs_migration(&self, manifest: &Manifest) -> bool {
        manifest.format_version < self.current_version()
    }
}

impl Default for MigrationManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct NoOpMigration;

#[async_trait]
impl MigrationFn for NoOpMigration {
    async fn execute(&self, _manifest: &mut Manifest) -> Result<(), MigrationError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestMigration {
        version: u32,
    }

    #[async_trait]
    impl MigrationFn for TestMigration {
        async fn execute(&self, manifest: &mut Manifest) -> Result<(), MigrationError> {
            manifest
                .metadata
                .insert(format!("migrated_to_{}", self.version), "true".to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_migration_manager() {
        let mut manager = MigrationManager::new();
        manager.register(2, "Test migration 2", TestMigration { version: 2 });
        manager.register(3, "Test migration 3", TestMigration { version: 3 });

        let mut manifest = Manifest::new("test");
        manifest.format_version = 1;

        assert!(manager.needs_migration(&manifest));
        assert_eq!(manager.pending_migrations(&manifest).len(), 2);
    }

    #[tokio::test]
    async fn test_run_migrations() {
        let mut manager = MigrationManager::new();
        manager.register(2, "Migration 2", TestMigration { version: 2 });
        manager.register(3, "Migration 3", TestMigration { version: 3 });

        let mut manifest = Manifest::new("test");
        manifest.format_version = 1;

        let applied = manager.run_migrations(&mut manifest).await.unwrap();
        assert_eq!(applied, vec![2, 3]);
        assert_eq!(manifest.format_version, 3);
    }

    #[tokio::test]
    async fn test_no_pending_migrations() {
        let manager = MigrationManager::new();

        let mut manifest = Manifest::new("test");
        manifest.format_version = CURRENT_FORMAT_VERSION;

        assert!(!manager.needs_migration(&manifest));
        assert!(manager.pending_migrations(&manifest).is_empty());
    }

    #[tokio::test]
    async fn test_partial_migration() {
        let mut manager = MigrationManager::new();
        manager.register(2, "Migration 2", TestMigration { version: 2 });
        manager.register(3, "Migration 3", TestMigration { version: 3 });

        let mut manifest = Manifest::new("test");
        manifest.format_version = 2;

        let applied = manager.run_migrations(&mut manifest).await.unwrap();
        assert_eq!(applied, vec![3]);
        assert_eq!(manifest.format_version, 3);
    }
}

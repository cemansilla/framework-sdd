use async_trait::async_trait;

#[async_trait]
pub trait StoragePort: Send + Sync {
    async fn read(&self, path: &str) -> Result<Vec<u8>, StorageError>;
    async fn write(&self, path: &str, data: &[u8]) -> Result<(), StorageError>;
    async fn exists(&self, path: &str) -> bool;
    async fn delete(&self, path: &str) -> Result<(), StorageError>;
    async fn list(&self, path: &str) -> Result<Vec<String>, StorageError>;
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("file not found: {path}")]
    NotFound { path: String },
    #[error("permission denied: {path}")]
    PermissionDenied { path: String },
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

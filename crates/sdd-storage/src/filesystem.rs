use crate::storage_port::{StorageError, StoragePort};
use async_trait::async_trait;

pub struct FilesystemAdapter {
    base_path: std::path::PathBuf,
}

impl FilesystemAdapter {
    pub fn new(base_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }
}

#[async_trait]
impl StoragePort for FilesystemAdapter {
    async fn read(&self, path: &str) -> Result<Vec<u8>, StorageError> {
        let full_path = self.base_path.join(path);
        tokio::fs::read(&full_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StorageError::NotFound {
                    path: path.to_string(),
                }
            } else {
                StorageError::Io(e)
            }
        })
    }

    async fn write(&self, path: &str, data: &[u8]) -> Result<(), StorageError> {
        let full_path = self.base_path.join(path);
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&full_path, data).await?;
        Ok(())
    }
}

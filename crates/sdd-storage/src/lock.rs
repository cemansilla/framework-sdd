use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
pub struct FileLock {
    lock_path: PathBuf,
    acquired: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub pid: u32,
    pub acquired_at: DateTime<Utc>,
    pub owner: String,
}

#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("lock already held by {0}")]
    AlreadyLocked(String),
    #[error("failed to acquire lock: {0}")]
    AcquireFailed(String),
    #[error("lock timeout")]
    Timeout,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl FileLock {
    pub fn new(lock_path: impl Into<PathBuf>) -> Self {
        Self {
            lock_path: lock_path.into(),
            acquired: false,
        }
    }

    pub async fn acquire(&mut self, owner: &str) -> Result<(), LockError> {
        self.acquire_with_timeout(owner, Duration::from_secs(30))
            .await
    }

    pub async fn acquire_with_timeout(
        &mut self,
        owner: &str,
        timeout: Duration,
    ) -> Result<(), LockError> {
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(100);

        loop {
            match self.try_acquire(owner).await {
                Ok(()) => {
                    self.acquired = true;
                    return Ok(());
                }
                Err(LockError::AlreadyLocked(_)) => {
                    if start.elapsed() >= timeout {
                        return Err(LockError::Timeout);
                    }
                    sleep(poll_interval).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    fn try_acquire<'a>(
        &'a self,
        owner: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), LockError>> + Send + 'a>>
    {
        Box::pin(async move {
            let lock_info = LockInfo {
                pid: std::process::id(),
                acquired_at: Utc::now(),
                owner: owner.to_string(),
            };

            let lock_json = serde_json::to_string_pretty(&lock_info)
                .map_err(|e| LockError::AcquireFailed(e.to_string()))?;

            match fs::write(&self.lock_path, &lock_json).await {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    if let Ok(existing) = fs::read_to_string(&self.lock_path).await {
                        if let Ok(info) = serde_json::from_str::<LockInfo>(&existing) {
                            if self.is_stale(&info).await {
                                fs::remove_file(&self.lock_path).await.ok();
                                return self.try_acquire(owner).await;
                            }
                            return Err(LockError::AlreadyLocked(info.owner));
                        }
                    }
                    Err(LockError::AcquireFailed(e.to_string()))
                }
                Err(e) => Err(LockError::AcquireFailed(e.to_string())),
            }
        })
    }

    async fn is_stale(&self, info: &LockInfo) -> bool {
        let age = Utc::now()
            .signed_duration_since(info.acquired_at)
            .num_seconds();
        age > 3600
    }

    pub async fn release(&mut self) -> Result<(), LockError> {
        if self.acquired {
            fs::remove_file(&self.lock_path)
                .await
                .map_err(LockError::Io)?;
            self.acquired = false;
        }
        Ok(())
    }

    pub fn is_acquired(&self) -> bool {
        self.acquired
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if self.acquired {
            let lock_path = self.lock_path.clone();
            std::fs::remove_file(&lock_path).ok();
        }
    }
}

#[derive(Debug)]
pub struct LockManager {
    base_path: PathBuf,
}

impl LockManager {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    pub fn lock_for(&self, resource: &str) -> FileLock {
        let lock_name = format!("{}.lock", resource.replace('/', "_"));
        FileLock::new(self.base_path.join(lock_name))
    }

    pub async fn with_lock<F, T, E>(&self, resource: &str, owner: &str, f: F) -> Result<T, E>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: From<LockError>,
    {
        let mut lock = self.lock_for(resource);
        lock.acquire(owner).await?;

        let result = f().await;

        lock.release().await?;

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_acquire_release() {
        let temp = TempDir::new().unwrap();
        let mut lock = FileLock::new(temp.path().join("test.lock"));

        lock.acquire("test-owner").await.unwrap();
        assert!(lock.is_acquired());

        lock.release().await.unwrap();
        assert!(!lock.is_acquired());
    }

    #[tokio::test]
    async fn test_lock_info() {
        let temp = TempDir::new().unwrap();
        let lock_path = temp.path().join("test.lock");

        let mut lock = FileLock::new(&lock_path);
        lock.acquire("test-owner").await.unwrap();

        let content = fs::read_to_string(&lock_path).await.unwrap();
        let info: LockInfo = serde_json::from_str(&content).unwrap();
        assert_eq!(info.owner, "test-owner");
    }

    #[tokio::test]
    async fn test_lock_manager() {
        let temp = TempDir::new().unwrap();
        let manager = LockManager::new(temp.path());

        let mut lock = manager.lock_for("test-resource");
        lock.acquire("owner").await.unwrap();
        assert!(lock.is_acquired());
    }

    #[tokio::test]
    async fn test_stale_lock_detection() {
        let temp = TempDir::new().unwrap();
        let lock_path = temp.path().join("stale.lock");

        let stale_info = LockInfo {
            pid: 12345,
            acquired_at: Utc::now() - chrono::Duration::hours(2),
            owner: "stale-owner".to_string(),
        };

        fs::write(&lock_path, serde_json::to_string(&stale_info).unwrap())
            .await
            .unwrap();

        let mut lock = FileLock::new(&lock_path);
        lock.acquire("new-owner").await.unwrap();

        let content = fs::read_to_string(&lock_path).await.unwrap();
        let info: LockInfo = serde_json::from_str(&content).unwrap();
        assert_eq!(info.owner, "new-owner");
    }
}

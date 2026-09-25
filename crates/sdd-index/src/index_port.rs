use async_trait::async_trait;

#[async_trait]
pub trait IndexPort: Send + Sync {
    async fn search(&self, query: &str) -> Result<Vec<String>, IndexError>;
}

#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error("index error: {0}")]
    Generic(String),
}

use crate::index_port::{IndexError, IndexPort};
use async_trait::async_trait;

pub struct VectorAdapter;

#[async_trait]
impl IndexPort for VectorAdapter {
    async fn search(&self, _query: &str) -> Result<Vec<String>, IndexError> {
        Ok(vec![])
    }
}

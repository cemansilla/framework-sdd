use crate::index_port::{IndexDocument, IndexError, IndexPort, SearchResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    pub values: Vec<f32>,
}

impl Vector {
    pub fn new(values: Vec<f32>) -> Self {
        Self { values }
    }

    pub fn zeros(dim: usize) -> Self {
        Self {
            values: vec![0.0; dim],
        }
    }

    pub fn dot(&self, other: &Vector) -> f64 {
        self.values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| (*a as f64) * (*b as f64))
            .sum()
    }

    pub fn magnitude(&self) -> f64 {
        self.values
            .iter()
            .map(|v| (*v as f64) * (*v as f64))
            .sum::<f64>()
            .sqrt()
    }

    pub fn cosine_similarity(&self, other: &Vector) -> f64 {
        let dot = self.dot(other);
        let mag_self = self.magnitude();
        let mag_other = other.magnitude();

        if mag_self == 0.0 || mag_other == 0.0 {
            0.0
        } else {
            dot / (mag_self * mag_other)
        }
    }
}

pub trait EmbeddingModel: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vector, IndexError>;
    fn dimension(&self) -> usize;
}

pub struct SimpleHashEmbedding {
    dimension: usize,
}

impl SimpleHashEmbedding {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

impl Default for SimpleHashEmbedding {
    fn default() -> Self {
        Self::new(128)
    }
}

impl EmbeddingModel for SimpleHashEmbedding {
    fn embed(&self, text: &str) -> Result<Vector, IndexError> {
        let mut values = vec![0.0f32; self.dimension];

        for chunk in text.as_bytes().chunks(4) {
            let hash = chunk
                .iter()
                .fold(0u32, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u32));
            let idx = (hash as usize) % self.dimension;
            values[idx] += 1.0;
            let idx2 = (idx + 1) % self.dimension;
            values[idx2] += 0.5;
        }

        let magnitude: f32 = values.iter().map(|v| v * v).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for v in &mut values {
                *v /= magnitude;
            }
        }

        Ok(Vector::new(values))
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    pub id: String,
    pub content: String,
    pub vector: Vector,
    pub metadata: HashMap<String, String>,
}

pub struct LocalVectorIndex {
    documents: Vec<VectorDocument>,
    embedding_model: Box<dyn EmbeddingModel>,
}

impl LocalVectorIndex {
    pub fn new(embedding_model: Box<dyn EmbeddingModel>) -> Self {
        Self {
            documents: Vec::new(),
            embedding_model,
        }
    }

    pub fn with_default_model() -> Self {
        Self::new(Box::new(SimpleHashEmbedding::default()))
    }

    pub fn add_document(
        &mut self,
        id: String,
        content: String,
        metadata: HashMap<String, String>,
    ) -> Result<(), IndexError> {
        let vector = self.embedding_model.embed(&content)?;
        self.documents.push(VectorDocument {
            id,
            content,
            vector,
            metadata,
        });
        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, IndexError> {
        let query_vector = self.embedding_model.embed(query)?;

        let mut results: Vec<SearchResult> = self
            .documents
            .iter()
            .map(|doc| {
                let score = query_vector.cosine_similarity(&doc.vector);
                SearchResult {
                    document_id: doc.id.clone(),
                    score,
                    snippet: doc.content.chars().take(200).collect(),
                    metadata: doc.metadata.clone(),
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);

        Ok(results)
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let initial_len = self.documents.len();
        self.documents.retain(|d| d.id != id);
        self.documents.len() < initial_len
    }

    pub fn count(&self) -> usize {
        self.documents.len()
    }
}

pub struct VectorAdapter {
    index: LocalVectorIndex,
}

impl VectorAdapter {
    pub fn new() -> Self {
        Self {
            index: LocalVectorIndex::with_default_model(),
        }
    }

    pub fn with_model(embedding_model: Box<dyn EmbeddingModel>) -> Self {
        Self {
            index: LocalVectorIndex::new(embedding_model),
        }
    }
}

impl Default for VectorAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IndexPort for VectorAdapter {
    async fn index_document(&self, _doc: IndexDocument) -> Result<(), IndexError> {
        Err(IndexError::Generic(
            "use mutable methods on LocalVectorIndex".to_string(),
        ))
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, IndexError> {
        self.index.search(query, limit)
    }

    async fn remove(&self, _id: &str) -> Result<(), IndexError> {
        Err(IndexError::Generic(
            "use mutable methods on LocalVectorIndex".to_string(),
        ))
    }

    async fn count(&self) -> Result<usize, IndexError> {
        Ok(self.index.count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_creation() {
        let v = Vector::new(vec![1.0, 2.0, 3.0]);
        assert_eq!(v.values.len(), 3);
    }

    #[test]
    fn test_vector_dot_product() {
        let v1 = Vector::new(vec![1.0, 0.0, 0.0]);
        let v2 = Vector::new(vec![1.0, 0.0, 0.0]);
        assert!((v1.dot(&v2) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_vector_cosine_similarity() {
        let v1 = Vector::new(vec![1.0, 0.0]);
        let v2 = Vector::new(vec![1.0, 0.0]);
        assert!((v1.cosine_similarity(&v2) - 1.0).abs() < 0.001);

        let v3 = Vector::new(vec![0.0, 1.0]);
        assert!(v1.cosine_similarity(&v3).abs() < 0.001);
    }

    #[test]
    fn test_simple_hash_embedding() {
        let model = SimpleHashEmbedding::new(64);
        let vector = model.embed("hello world").unwrap();
        assert_eq!(vector.values.len(), 64);
    }

    #[test]
    fn test_local_vector_index() {
        let mut index = LocalVectorIndex::with_default_model();

        index
            .add_document(
                "doc1".to_string(),
                "Rust programming language".to_string(),
                HashMap::new(),
            )
            .unwrap();

        index
            .add_document(
                "doc2".to_string(),
                "Python scripting language".to_string(),
                HashMap::new(),
            )
            .unwrap();

        assert_eq!(index.count(), 2);

        let results = index.search("rust", 10).unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_vector_index_remove() {
        let mut index = LocalVectorIndex::with_default_model();

        index
            .add_document(
                "doc1".to_string(),
                "Test content".to_string(),
                HashMap::new(),
            )
            .unwrap();

        assert_eq!(index.count(), 1);
        assert!(index.remove("doc1"));
        assert_eq!(index.count(), 0);
    }

    #[test]
    fn test_vector_adapter() {
        let adapter = VectorAdapter::new();
        assert_eq!(adapter.index.count(), 0);
    }
}

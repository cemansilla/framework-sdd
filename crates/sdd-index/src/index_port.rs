use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document_id: String,
    pub score: f64,
    pub snippet: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDocument {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
pub trait IndexPort: Send + Sync {
    async fn index_document(&self, doc: IndexDocument) -> Result<(), IndexError>;
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, IndexError>;
    async fn remove(&self, id: &str) -> Result<(), IndexError>;
    async fn count(&self) -> Result<usize, IndexError>;
}

#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error("index error: {0}")]
    Generic(String),
    #[error("document not found: {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    IoError(String),
}

pub struct LexicalIndex {
    documents: HashMap<String, IndexDocument>,
    inverted_index: HashMap<String, Vec<String>>,
}

impl LexicalIndex {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            inverted_index: HashMap::new(),
        }
    }

    pub fn index_document_sync(&mut self, doc: IndexDocument) {
        let words = self.tokenize(&doc.content);
        for word in words {
            self.inverted_index
                .entry(word)
                .or_default()
                .push(doc.id.clone());
        }
        self.documents.insert(doc.id.clone(), doc);
    }

    pub fn search_sync(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let query_words = self.tokenize(query);
        let mut scores: HashMap<String, f64> = HashMap::new();

        for word in &query_words {
            if let Some(doc_ids) = self.inverted_index.get(word) {
                for doc_id in doc_ids {
                    *scores.entry(doc_id.clone()).or_insert(0.0) += 1.0;
                }
            }
        }

        let mut results: Vec<SearchResult> = scores
            .into_iter()
            .filter_map(|(doc_id, score)| {
                self.documents.get(&doc_id).map(|doc| SearchResult {
                    document_id: doc_id,
                    score,
                    snippet: doc.content.chars().take(200).collect(),
                    metadata: doc.metadata.clone(),
                })
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);
        results
    }

    pub fn remove_sync(&mut self, id: &str) -> bool {
        if let Some(doc) = self.documents.remove(id) {
            let words = self.tokenize(&doc.content);
            for word in words {
                if let Some(doc_ids) = self.inverted_index.get_mut(&word) {
                    doc_ids.retain(|d| d != id);
                }
            }
            true
        } else {
            false
        }
    }

    pub fn count_sync(&self) -> usize {
        self.documents.len()
    }

    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2)
            .map(|w| w.to_string())
            .collect()
    }
}

impl Default for LexicalIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IndexPort for LexicalIndex {
    async fn index_document(&self, _doc: IndexDocument) -> Result<(), IndexError> {
        Err(IndexError::Generic(
            "use index_document_sync for mutable access".to_string(),
        ))
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, IndexError> {
        Ok(self.search_sync(query, limit))
    }

    async fn remove(&self, _id: &str) -> Result<(), IndexError> {
        Err(IndexError::Generic(
            "use remove_sync for mutable access".to_string(),
        ))
    }

    async fn count(&self) -> Result<usize, IndexError> {
        Ok(self.count_sync())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexical_index_creation() {
        let index = LexicalIndex::new();
        assert_eq!(index.count_sync(), 0);
    }

    #[test]
    fn test_index_document() {
        let mut index = LexicalIndex::new();
        let doc = IndexDocument {
            id: "doc1".to_string(),
            content: "Hello world this is a test document".to_string(),
            metadata: HashMap::new(),
        };

        index.index_document_sync(doc);
        assert_eq!(index.count_sync(), 1);
    }

    #[test]
    fn test_search() {
        let mut index = LexicalIndex::new();

        index.index_document_sync(IndexDocument {
            id: "doc1".to_string(),
            content: "Rust programming language is fast".to_string(),
            metadata: HashMap::new(),
        });

        index.index_document_sync(IndexDocument {
            id: "doc2".to_string(),
            content: "Python is a scripting language".to_string(),
            metadata: HashMap::new(),
        });

        let results = index.search_sync("rust language", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].document_id, "doc1");
    }

    #[test]
    fn test_remove_document() {
        let mut index = LexicalIndex::new();

        index.index_document_sync(IndexDocument {
            id: "doc1".to_string(),
            content: "Test content".to_string(),
            metadata: HashMap::new(),
        });

        assert_eq!(index.count_sync(), 1);
        assert!(index.remove_sync("doc1"));
        assert_eq!(index.count_sync(), 0);
    }

    #[test]
    fn test_search_limit() {
        let mut index = LexicalIndex::new();

        for i in 0..10 {
            index.index_document_sync(IndexDocument {
                id: format!("doc{}", i),
                content: format!("Document {} with common keyword", i),
                metadata: HashMap::new(),
            });
        }

        let results = index.search_sync("document", 5);
        assert_eq!(results.len(), 5);
    }
}

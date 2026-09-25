pub mod index_port;
pub mod vector_adapter;

pub use index_port::{IndexDocument, IndexError, IndexPort, LexicalIndex, SearchResult};
pub use vector_adapter::{
    EmbeddingModel, LocalVectorIndex, SimpleHashEmbedding, Vector, VectorAdapter, VectorDocument,
};

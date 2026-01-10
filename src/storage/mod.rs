// Storage module for vector embeddings and data persistence
mod error;
mod vector_store;
mod embedders;

pub use error::{StorageError, StorageResult};
pub use vector_store::{
    VectorStore, VectorStoreConfig, VectorSearchOptions,
    VectorSearchResult, Paper, PaperMetadata, VectorMetric
};
pub use embedders::{
    Embedder, EmbeddingProvider, OpenAIEmbedder, LocalEmbedder,
    EmbeddingConfig, EmbeddingRequest, EmbeddingResponse
};

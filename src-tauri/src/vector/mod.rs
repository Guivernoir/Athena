pub mod client;
pub mod schema;
pub mod insert;
pub mod query;
pub mod embedding;

pub use client::QdrantClient;
pub use schema::*;
pub use insert::*;
pub use query::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VectorDbError {
    #[error("Client connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Insert operation failed: {0}")]
    InsertFailed(String),
    #[error("Query operation failed: {0}")]
    QueryFailed(String),
    #[error("Embedding generation failed: {0}")]
    EmbeddingFailed(String),
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    #[error("Invalid vector dimensions: expected {expected}, got {actual}")]
    InvalidVectorDimensions { expected: usize, actual: usize },
}
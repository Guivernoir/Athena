pub mod db;
pub mod ws;
pub mod merger;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RetrievalError {
    #[error("Database error: {0}")]
    Database(#[from] db::DatabaseError),
    #[error("Web search error: {0}")]
    WebSearch(#[from] ws::WebSearchError),
    #[error("Merging error: {0}")]
    Merging(#[from] merger::MergeError),
}

pub type RetrievalResult<T> = Result<T, RetrievalError>;
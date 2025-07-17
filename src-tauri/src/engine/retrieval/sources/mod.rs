//! Common traits + source registry.

pub mod cache;
pub mod memory;
pub mod web;

use async_trait::async_trait;
use super::result::SearchResult;

#[async_trait]
pub trait Source: Send + Sync {
    /// Human-readable name for logging.
    fn name(&self) -> &'static str;

    /// Perform the actual search.
    async fn search(&self, query: &str, top_k: usize) -> anyhow::Result<Vec<SearchResult>>;
}
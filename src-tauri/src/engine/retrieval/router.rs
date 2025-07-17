//! Smart dispatcher: decides which sources to query.

use crate::{query::SearchQuery, result::SearchResult, sources::*};
use std::sync::Arc;

pub struct Router {
    cache: Arc<sources::cache::CacheSource>,
    memory: Arc<sources::memory::MemorySource>,
    web: Arc<sources::web::WebSource>,
}

impl Router {
    pub fn new(
        cache: sources::cache::CacheSource,
        memory: sources::memory::MemorySource,
        web: sources::web::WebSource,
    ) -> Self {
        Self {
            cache: Arc::new(cache),
            memory: Arc::new(memory),
            web: Arc::new(web),
        }
    }

    /// Run all enabled sources concurrently.
    pub async fn search(&self, query: &SearchQuery) -> anyhow::Result<Vec<SearchResult>> {
        use futures::future::join_all;

        let tasks = vec![
            self.cache.search(&query.text, query.top_k),
            self.memory.search(&query.text, query.top_k),
            self.web.search(&query.text, query.top_k),
        ];

        let batches: Vec<_> = join_all(tasks)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        Ok(crate::merger::merge(batches))
    }
}
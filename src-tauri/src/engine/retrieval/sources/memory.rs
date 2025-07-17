//! Disk-based memory search (delegates to memory::search).

use super::Source;
use super::super::result::SearchResult;
use async_trait::async_trait;
use memory::{Searcher, Store};

pub struct MemorySource {
    searcher: Searcher,
}

impl MemorySource {
    pub fn new() -> anyhow::Result<Self> {
        let store = Store::open("memory.vec")?;
        let mmap  = store.mmap()?; // read-only mmap
        // dimension discovery: read header or use 384 (bge-small)
        let dim = 384;
        let searcher = Searcher::new(mmap, dim);
        Ok(Self { searcher })
    }
}

#[async_trait]
impl Source for MemorySource {
    fn name(&self) -> &'static str { "memory" }

    async fn search(&self, query: &str, top_k: usize) -> anyhow::Result<Vec<SearchResult>> {
        let q_vec = embedding::Engine::new().embed(query);
        let hits = self.searcher.search(&q_vec, top_k);
        Ok(hits
            .into_iter()
            .map(|(score, rec)| SearchResult {
                score,
                content: String::from_utf8_lossy(&rec.payload).into_owned(),
                source: self.name(),
            })
            .collect())
    }
}
//! Converts retrieval results into structured prompt blocks.

use crate::schema::{Block, PromptPayload};
use engine::retrieval::result::SearchResult;

pub fn format_results(
    system: String,
    memory: Vec<SearchResult>,
    cache: Vec<SearchResult>,
    web: Vec<SearchResult>,
) -> PromptPayload {
    let mut payload = PromptPayload {
        system,
        blocks: Vec::new(),
    };

    if !memory.is_empty() {
        let texts: Vec<_> = memory.into_iter().map(|r| r.content).collect();
        payload.blocks.push(("Memory".into(), Block::Memory(texts)));
    }

    if !cache.is_empty() {
        let texts: Vec<_> = cache.into_iter().map(|r| r.content).collect();
        payload.blocks.push(("Cache".into(), Block::Cache(texts)));
    }

    if !web.is_empty() {
        let texts: Vec<_> = web.into_iter().map(|r| r.content).collect();
        payload.blocks.push(("Web".into(), Block::Web(texts)));
    }

    payload
}
//! Weighted merge, deduplication, re-ranking.

use crate::{result::SearchResult, scorer::rerank};
use std::collections::HashSet;

pub fn merge(mut sources: Vec<Vec<SearchResult>>) -> Vec<SearchResult> {
    let mut seen = HashSet::new();
    let mut merged = Vec::new();

    for batch in sources {
        for item in batch {
            if seen.contains(&item.content) {
                continue;
            }
            seen.insert(item.content.clone());
            merged.push(item);
        }
    }
    rerank(merged)
}
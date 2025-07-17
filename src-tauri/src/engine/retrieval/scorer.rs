//! Post-processing: cosine + recency boost (stub).

use crate::result::SearchResult;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn rerank(mut results: Vec<SearchResult>) -> Vec<SearchResult> {
    // Naïve: keep score as is.  Later add recency, authority, etc.
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    results
}
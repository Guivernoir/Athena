//! Unified result type returned by every source.

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub score: f32,
    pub content: String,
    pub source: &'static str,
}
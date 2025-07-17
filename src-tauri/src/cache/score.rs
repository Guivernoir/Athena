//! Optional: relevance or priority scoring (for smart flush).
//! Currently unused — stub for future LRU / importance signals.

pub fn score(_entry: &crate::message::CacheEntry) -> f32 {
    1.0
}
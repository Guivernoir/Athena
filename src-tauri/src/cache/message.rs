//! Chat message structs, with optional metadata.

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,       // "user" | "assistant"
    pub content: String,
    pub timestamp: u64,     // epoch millis
}

impl ChatMessage {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        Self {
            role: role.into(),
            content: content.into(),
            timestamp: ts,
        }
    }
}

/// Internal representation of an (input, output) pair.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub input: ChatMessage,
    pub output: ChatMessage,
}
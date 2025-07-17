//! In-memory staging area for chat messages.
//! Buffers (input, output) pairs, embeds them 1 s after display,
//! then flushes to memory in configurable batches.

pub mod embedder;
pub mod flusher;
pub mod manager;
pub mod message;
pub mod scorer;

#[cfg(test)]
mod tests;
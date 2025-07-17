//! Runs embedding on messages when flush is triggered.
//! Thin wrapper around `embedding::Engine`.

use crate::message::CacheEntry;
use embedding::Engine;

pub fn embed_batch(engine: &Engine, batch: Vec<CacheEntry>) -> Vec<memory::MemoryRecord> {
    let mut records = Vec::with_capacity(batch.len() * 2);

    for entry in batch {
        let input_vec = engine.embed(&entry.input.content);
        let output_vec = engine.embed(&entry.output.content);

        let now_secs = (entry.output.timestamp / 1000) as u64;

        records.push(memory::MemoryRecord {
            vector: input_vec,
            timestamp: now_secs,
            payload: serde_json::to_vec(&entry.input).unwrap(),
        });

        records.push(memory::MemoryRecord {
            vector: output_vec,
            timestamp: now_secs,
            payload: serde_json::to_vec(&entry.output).unwrap(),
        });
    }

    records
}
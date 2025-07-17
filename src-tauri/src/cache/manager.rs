//! High-level cache logic: push, flush check, reset.

use crate::message::{CacheEntry, ChatMessage};
use crate::flusher::FlusherHandle;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Public handle to the cache.
#[derive(Clone)]
pub struct Cache {
    inner: std::sync::Arc<std::sync::Mutex<CacheInner>>,
}

struct CacheInner {
    buffer: VecDeque<CacheEntry>,
    capacity: usize,           // max #messages before forced flush
    ttl: Duration,             // max time before flush
    last_flush: Instant,
    flusher: FlusherHandle,
}

impl Cache {
    pub fn new(capacity: usize, ttl_secs: u64) -> Self {
        let (flusher, handle) = crate::flusher::spawn();
        Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(CacheInner {
                buffer: VecDeque::new(),
                capacity,
                ttl: Duration::from_secs(ttl_secs),
                last_flush: Instant::now(),
                flusher: handle,
            })),
        }
    }

    /// Push a new (input, output) pair into the buffer.
    pub fn push(&self, input: ChatMessage, output: ChatMessage) {
        let mut inner = self.inner.lock().unwrap();
        inner.buffer.push_back(CacheEntry { input, output });
        inner.maybe_flush();
    }

    /// Manual flush (e.g. shutdown).
    pub fn flush(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.do_flush();
    }
}

impl CacheInner {
    fn maybe_flush(&mut self) {
        if self.buffer.len() >= self.capacity || self.last_flush.elapsed() >= self.ttl {
            self.do_flush();
        }
    }

    fn do_flush(&mut self) {
        let batch: Vec<_> = self.buffer.drain(..).collect();
        if !batch.is_empty() {
            self.flusher.send(batch);
            self.last_flush = Instant::now();
        }
    }
}
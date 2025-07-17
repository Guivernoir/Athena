//! Handles triggering + passing embedded records to memory.
//! Spawns a background thread that:
//!   1. waits 1 s after receiving a batch,
//!   2. embeds via `embedder`,
//!   3. appends to the global `memory::Store`.

use crate::embedder::embed_batch;
use crate::message::CacheEntry;
use std::sync::mpsc::{channel, Sender};
use std::thread;

#[derive(Clone)]
pub struct FlusherHandle(Sender<Vec<CacheEntry>>);

impl FlusherHandle {
    fn send(&self, batch: Vec<CacheEntry>) {
        let _ = self.0.send(batch);
    }
}

pub fn spawn() -> (thread::JoinHandle<()>, FlusherHandle) {
    let (tx, rx) = channel::<Vec<CacheEntry>>();
    let handle = thread::spawn(move || {
        let engine = embedding::Engine::new(); // cheap clone
        let mut store = memory::Store::open("memory.vec").expect("open store");

        while let Ok(batch) = rx.recv() {
            // 1. wait 1 s
            thread::sleep(std::time::Duration::from_secs(1));
            // 2. embed
            let records = embed_batch(&engine, batch);
            // 3. append
            for rec in records {
                store.append(&rec).expect("append");
            }
            store.flush().expect("flush");
        }
    });

    (handle, FlusherHandle(tx))
}
//! Disk-level vector search: cosine similarity (no PQ yet).

use crate::record::MemoryRecord;
use memmap2::Mmap;
use std::sync::Arc;

pub struct Searcher {
    mmap: Arc<Mmap>,
    dim: usize,
}

impl Searcher {
    pub fn new(mmap: Mmap, dim: usize) -> Self {
        Self {
            mmap: Arc::new(mmap),
            dim,
        }
    }

    /// Compute cosine(q, v) = dot(q,v) / (||q||·||v||)
    fn cosine(q: &[f32], v: &[f32]) -> f32 {
        let mut dot = 0.0;
        let mut q_norm = 0.0;
        let mut v_norm = 0.0;
        for (&a, &b) in q.iter().zip(v) {
            dot += a * b;
            q_norm += a * a;
            v_norm += b * b;
        }
        dot / (q_norm.sqrt() * v_norm.sqrt()).max(f32::EPSILON)
    }

    /// Return top-k records by cosine similarity.
    pub fn search(&self, query: &[f32], k: usize) -> Vec<(f32, MemoryRecord)> {
        // Naive linear scan; later PQ + IVF.
        let mut results = Vec::new();
        let mut offset = 0;
        while offset + self.record_size() <= self.mmap.len() {
            let slice = &self.mmap[offset..offset + self.record_size()];
            if let Ok(rec) = MemoryRecord::from_bytes(slice, self.dim) {
                let score = Self::cosine(query, &rec.vector);
                results.push((score, rec));
            }
            offset += self.record_size();
        }
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        results.into_iter().take(k).collect()
    }

    fn record_size(&self) -> usize {
        self.dim * 4 + 8 + 4
    }
}
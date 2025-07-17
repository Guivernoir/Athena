//! Optional in-memory offset/index builder for O(1) seeks.
//! Not used yet; stub for future IVF/HNSW work.

#[derive(Default)]
pub struct Index {
    offsets: Vec<u64>,
}

impl Index {
    pub fn add(&mut self, offset: u64) {
        self.offsets.push(offset);
    }

    pub fn len(&self) -> usize {
        self.offsets.len()
    }
}
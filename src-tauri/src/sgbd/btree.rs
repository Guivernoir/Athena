use std::collections::BTreeMap;
use super::types::{Key, SGBDError, Result};

pub struct BTreeIndex {
    index: BTreeMap<Key, u64>, // Key -> file offset
}

impl BTreeIndex {
    pub fn new() -> Self {
        Self {
            index: BTreeMap::new(),
        }
    }
    
    pub fn insert(&mut self, key: Key, offset: u64) {
        self.index.insert(key, offset);
    }
    
    pub fn get(&self, key: &Key) -> Option<u64> {
        self.index.get(key).copied()
    }
    
    pub fn remove(&mut self, key: &Key) -> Option<u64> {
        self.index.remove(key)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (&Key, &u64)> {
        self.index.iter()
    }
    
    pub fn keys(&self) -> impl Iterator<Item = &Key> {
        self.index.keys()
    }
    
    pub fn range(&self, start: &Key, end: &Key) -> impl Iterator<Item = (&Key, &u64)> {
        self.index.range(start..=end)
    }
    
    pub fn len(&self) -> usize {
        self.index.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }
}
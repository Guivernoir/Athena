use crate::sgbd::{
    Bytes, DatabaseConfig, Key, Result, SGBDError, SerializationStrategy, StorageBackend,
    StorageError, Value,
};
use std::{
    cmp::Ordering,
    collections::VecDeque,
    fmt,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering as AtomicOrdering},
        Arc,
    },
};
use tokio::sync::{Mutex, RwLock};

/// B-Tree configuration
#[derive(Debug, Clone)]
pub struct BTreeConfig {
    pub order: usize, // Minimum degree (t). Node has at least t-1 keys, at most 2t-1 keys
    pub node_size_bytes: usize,
    pub cache_size_nodes: usize,
    pub serialization_format: SerializationFormat,
}

/// B-Tree metrics
#[derive(Debug, Default)]
pub struct BTreeMetrics {
    read_count: AtomicU64,
    write_count: AtomicU64,
    node_splits: AtomicU64,
    node_merges: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    tree_height: AtomicUsize,
    leaf_nodes: AtomicUsize,
    internal_nodes: AtomicUsize,
}

/// B-Tree metrics snapshot
pub struct BTreeMetricsSnapshot {
    pub read_count: u64,
    pub write_count: u64,
    pub node_splits: u64,
    pub node_merges: u64,
    pub cache_hit_ratio: f64,
    pub tree_height: usize,
    pub leaf_nodes: usize,
    pub internal_nodes: usize,
}

/// B-Tree node structure
struct BTreeNode {
    keys: Vec<Key>,
    values: Vec<ValueLocation>,            // Only in leaf nodes
    children: Vec<Arc<RwLock<BTreeNode>>>, // Only in internal nodes
    is_leaf: bool,
    parent: Option<Arc<RwLock<BTreeNode>>>,
}

/// Location of a value in storage
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValueLocation {
    pub segment_id: uuid::Uuid,
    pub offset: u64,
    pub size: usize,
}

/// B-Tree index
pub struct BTreeIndex {
    root: Arc<RwLock<BTreeNode>>,
    config: BTreeConfig,
    metrics: BTreeMetrics,
    cache: Mutex<lru::LruCache<Key, ValueLocation>>,
}

impl BTreeIndex {
    /// Create a new B-Tree index
    pub fn new(config: BTreeConfig) -> Result<Self> {
        let root = Arc::new(RwLock::new(BTreeNode {
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
            is_leaf: true,
            parent: None,
        }));

        Ok(Self {
            root,
            config,
            metrics: BTreeMetrics::default(),
            cache: Mutex::new(lru::LruCache::new(config.cache_size_nodes)),
        })
    }

    /// Insert a key-value location pair
    pub async fn insert(&self, key: Key, location: ValueLocation) -> Result<()> {
        let mut cache = self.cache.lock().await;
        cache.put(key.clone(), location.clone());
        drop(cache);

        let mut root = self.root.write().await;
        self.metrics
            .write_count
            .fetch_add(1, AtomicOrdering::Relaxed);

        // If root is full, split it and create a new root
        if root.keys.len() == 2 * self.config.order - 1 {
            let mut new_root = BTreeNode {
                keys: Vec::new(),
                values: Vec::new(),
                children: vec![Arc::clone(&self.root)],
                is_leaf: false,
                parent: None,
            };

            // Transfer ownership of old root to new_root
            let old_root = std::mem::replace(&mut *root, new_root);
            let old_root = Arc::new(RwLock::new(old_root));

            self.split_child(&mut *root, 0, &old_root).await?;
            self.metrics
                .node_splits
                .fetch_add(1, AtomicOrdering::Relaxed);
        }

        self.insert_non_full(&mut *root, key, location).await
    }

    /// Lookup a key
    pub async fn get(&self, key: &Key) -> Result<Option<ValueLocation>> {
        // Check cache first
        {
            let mut cache = self.cache.lock().await;
            if let Some(location) = cache.get(key) {
                self.metrics
                    .cache_hits
                    .fetch_add(1, AtomicOrdering::Relaxed);
                return Ok(Some(location.clone()));
            }
        }

        self.metrics
            .cache_misses
            .fetch_add(1, AtomicOrdering::Relaxed);
        self.metrics
            .read_count
            .fetch_add(1, AtomicOrdering::Relaxed);

        let root = self.root.read().await;
        self.search(&*root, key).await
    }

    /// Delete a key
    pub async fn delete(&self, key: &Key) -> Result<()> {
        let mut cache = self.cache.lock().await;
        cache.pop(key);
        drop(cache);

        let mut root = self.root.write().await;
        self.delete_key(&mut *root, key).await
    }

    /// Range query (start inclusive, end exclusive)
    pub async fn range_scan(&self, start: &Key, end: &Key) -> Result<Vec<(Key, ValueLocation)>> {
        self.metrics
            .read_count
            .fetch_add(1, AtomicOrdering::Relaxed);

        let root = self.root.read().await;
        let mut results = Vec::new();
        self.scan_node(&*root, start, end, &mut results).await?;
        Ok(results)
    }

    /// Get index metrics
    pub fn metrics(&self) -> BTreeMetricsSnapshot {
        let cache_hits = self.metrics.cache_hits.load(AtomicOrdering::Relaxed);
        let cache_misses = self.metrics.cache_misses.load(AtomicOrdering::Relaxed);
        let total_cache = cache_hits + cache_misses;

        BTreeMetricsSnapshot {
            read_count: self.metrics.read_count.load(AtomicOrdering::Relaxed),
            write_count: self.metrics.write_count.load(AtomicOrdering::Relaxed),
            node_splits: self.metrics.node_splits.load(AtomicOrdering::Relaxed),
            node_merges: self.metrics.node_merges.load(AtomicOrdering::Relaxed),
            cache_hit_ratio: if total_cache > 0 {
                cache_hits as f64 / total_cache as f64
            } else {
                0.0
            },
            tree_height: self.metrics.tree_height.load(AtomicOrdering::Relaxed),
            leaf_nodes: self.metrics.leaf_nodes.load(AtomicOrdering::Relaxed),
            internal_nodes: self.metrics.internal_nodes.load(AtomicOrdering::Relaxed),
        }
    }

    // Internal implementation methods
    async fn search(&self, node: &BTreeNode, key: &Key) -> Result<Option<ValueLocation>> {
        match node.keys.binary_search_by(|k| k.cmp(key)) {
            Ok(i) => {
                if node.is_leaf {
                    Ok(Some(node.values[i].clone()))
                } else {
                    // Internal nodes don't store values
                    Err(SGBDError::Index {
                        operation: "search".to_string(),
                        index_type: "B-Tree".to_string(),
                        backtrace: Some(std::backtrace::Backtrace::capture()),
                    })
                }
            }
            Err(i) => {
                if node.is_leaf {
                    Ok(None)
                } else {
                    let child = node.children[i].read().await;
                    self.search(&*child, key).await
                }
            }
        }
    }

    async fn insert_non_full(
        &self,
        node: &mut BTreeNode,
        key: Key,
        location: ValueLocation,
    ) -> Result<()> {
        match node.keys.binary_search_by(|k| k.cmp(&key)) {
            Ok(_) => {
                // Key already exists - update value
                if node.is_leaf {
                    // Update value in leaf node
                    let idx = node
                        .keys
                        .iter()
                        .position(|k| k == &key)
                        .expect("Key should exist");
                    node.values[idx] = location;
                    Ok(())
                } else {
                    // Internal nodes don't store values
                    Err(SGBDError::Index {
                        operation: "insert_non_full".to_string(),
                        index_type: "B-Tree".to_string(),
                        backtrace: Some(std::backtrace::Backtrace::capture()),
                    })
                }
            }
            Err(i) => {
                if node.is_leaf {
                    // Insert into leaf node
                    node.keys.insert(i, key);
                    node.values.insert(i, location);
                    Ok(())
                } else {
                    // Handle internal node
                    let mut child = node.children[i].write().await;
                    if child.keys.len() == 2 * self.config.order - 1 {
                        self.split_child(node, i, &node.children[i]).await?;
                        self.metrics
                            .node_splits
                            .fetch_add(1, AtomicOrdering::Relaxed);

                        // After split, decide which child to go into
                        match node.keys[i].cmp(&key) {
                            Ordering::Less => child = node.children[i + 1].write().await,
                            Ordering::Greater => child = node.children[i].write().await,
                            Ordering::Equal => {
                                return Err(SGBDError::Index {
                                    operation: "insert_non_full".to_string(),
                                    index_type: "B-Tree".to_string(),
                                    backtrace: Some(std::backtrace::Backtrace::capture()),
                                })
                            }
                        }
                    }
                    self.insert_non_full(&mut *child, key, location).await
                }
            }
        }
    }

    async fn split_child(
        &self,
        parent: &mut BTreeNode,
        index: usize,
        child: &Arc<RwLock<BTreeNode>>,
    ) -> Result<()> {
        let mut child = child.write().await;
        let order = self.config.order;
        let mut new_child = BTreeNode {
            keys: Vec::with_capacity(2 * order - 1),
            values: Vec::with_capacity(2 * order - 1),
            children: Vec::with_capacity(2 * order),
            is_leaf: child.is_leaf,
            parent: Some(Arc::new(RwLock::new(parent.clone()))),
        };

        // Move second half of child to new_child
        new_child.keys = child.keys.split_off(order);
        if child.is_leaf {
            new_child.values = child.values.split_off(order);
        } else {
            new_child.children = child.children.split_off(order);
        }

        // Move median key up to parent
        let median_key = child.keys.pop().unwrap();
        parent.keys.insert(index, median_key);

        // Insert new child into parent
        parent
            .children
            .insert(index + 1, Arc::new(RwLock::new(new_child)));

        // Update metrics
        if parent.children.len() == 1 {
            self.metrics
                .tree_height
                .fetch_add(1, AtomicOrdering::Relaxed);
        }
        if child.is_leaf {
            self.metrics
                .leaf_nodes
                .fetch_add(1, AtomicOrdering::Relaxed);
        } else {
            self.metrics
                .internal_nodes
                .fetch_add(1, AtomicOrdering::Relaxed);
        }

        Ok(())
    }

    async fn delete_key(&self, node: &mut BTreeNode, key: &Key) -> Result<()> {
        let idx = match node.keys.binary_search_by(|k| k.cmp(key)) {
            Ok(i) => i,
            Err(i) => {
                if node.is_leaf {
                    // Key not found
                    return Ok(());
                }
                // Key not in this node - recurse to child
                return self.delete_key_in_child(node, i, key).await;
            }
        };

        // Case 1: Key is in a leaf node
        if node.is_leaf {
            node.keys.remove(idx);
            node.values.remove(idx);
            return Ok(());
        }

        // Case 2: Key is in an internal node
        self.delete_key_internal(node, idx, key).await
    }

    async fn delete_key_in_child(&self, node: &mut BTreeNode, idx: usize, key: &Key) -> Result<()> {
        let mut child = node.children[idx].write().await;

        // If child has minimum keys, we need to ensure it has at least order keys
        if child.keys.len() < self.config.order {
            self.fill_child(node, idx).await?;
        }

        // After filling, child might have changed
        let child = node.children[idx].write().await;
        let idx = match child.keys.binary_search_by(|k| k.cmp(key)) {
            Ok(i) => i,
            Err(i) => {
                // Recurse to the appropriate child
                return self.delete_key(&mut *child, key).await;
            }
        };

        self.delete_key(&mut *child, key).await
    }

    async fn delete_key_internal(&self, node: &mut BTreeNode, idx: usize, key: &Key) -> Result<()> {
        // Case 3a: The child that precedes key has at least order keys
        let left_child = node.children[idx].write().await;
        if left_child.keys.len() >= self.config.order {
            let predecessor = self.get_predecessor(&*left_child).await?;
            node.keys[idx] = predecessor.0;
            self.delete_key(&mut *left_child, &predecessor.0).await?;
            return Ok(());
        }

        // Case 3b: The child that follows key has at least order keys
        let right_child = node.children[idx + 1].write().await;
        if right_child.keys.len() >= self.config.order {
            let successor = self.get_successor(&*right_child).await?;
            node.keys[idx] = successor.0;
            self.delete_key(&mut *right_child, &successor.0).await?;
            return Ok(());
        }

        // Case 3c: Both children have minimum keys - merge them
        self.merge_children(node, idx).await?;
        self.metrics
            .node_merges
            .fetch_add(1, AtomicOrdering::Relaxed);
        self.delete_key(&mut *node.children[idx].write().await, key)
            .await
    }

    async fn get_predecessor(&self, node: &BTreeNode) -> Result<(Key, ValueLocation)> {
        if node.is_leaf {
            Ok((
                node.keys.last().unwrap().clone(),
                node.values.last().unwrap().clone(),
            ))
        } else {
            let last_child = node.children.last().unwrap().read().await;
            self.get_predecessor(&*last_child).await
        }
    }

    async fn get_successor(&self, node: &BTreeNode) -> Result<(Key, ValueLocation)> {
        if node.is_leaf {
            Ok((
                node.keys.first().unwrap().clone(),
                node.values.first().unwrap().clone(),
            ))
        } else {
            let first_child = node.children.first().unwrap().read().await;
            self.get_successor(&*first_child).await
        }
    }

    async fn fill_child(&self, parent: &mut BTreeNode, idx: usize) -> Result<()> {
        // Try to borrow from left sibling
        if idx > 0 {
            let left_sibling = parent.children[idx - 1].write().await;
            if left_sibling.keys.len() >= self.config.order {
                return self.borrow_from_left(parent, idx).await;
            }
        }

        // Try to borrow from right sibling
        if idx < parent.children.len() - 1 {
            let right_sibling = parent.children[idx + 1].write().await;
            if right_sibling.keys.len() >= self.config.order {
                return self.borrow_from_right(parent, idx).await;
            }
        }

        // Merge with sibling
        if idx > 0 {
            self.merge_children(parent, idx - 1).await
        } else {
            self.merge_children(parent, idx).await
        }
    }

    async fn borrow_from_left(&self, parent: &mut BTreeNode, idx: usize) -> Result<()> {
        let mut child = parent.children[idx].write().await;
        let mut left_sibling = parent.children[idx - 1].write().await;

        // Move key from parent down to child
        child.keys.insert(0, parent.keys[idx - 1].clone());

        // Move last key from left sibling up to parent
        parent.keys[idx - 1] = left_sibling.keys.pop().unwrap();

        // Move value if leaf
        if child.is_leaf {
            child.values.insert(0, left_sibling.values.pop().unwrap());
        }

        // Move child pointer if internal
        if !left_sibling.is_leaf {
            let last_child = left_sibling.children.pop().unwrap();
            child.children.insert(0, last_child);
        }

        Ok(())
    }

    async fn borrow_from_right(&self, parent: &mut BTreeNode, idx: usize) -> Result<()> {
        let mut child = parent.children[idx].write().await;
        let mut right_sibling = parent.children[idx + 1].write().await;

        // Move key from parent down to child
        child.keys.push(parent.keys[idx].clone());

        // Move first key from right sibling up to parent
        parent.keys[idx] = right_sibling.keys.remove(0);

        // Move value if leaf
        if child.is_leaf {
            child.values.push(right_sibling.values.remove(0));
        }

        // Move child pointer if internal
        if !right_sibling.is_leaf {
            let first_child = right_sibling.children.remove(0);
            child.children.push(first_child);
        }

        Ok(())
    }

    async fn merge_children(&self, parent: &mut BTreeNode, idx: usize) -> Result<()> {
        let mut left_child = parent.children[idx].write().await;
        let mut right_child = parent.children[idx + 1].write().await;

        // Move key from parent to left child
        left_child.keys.push(parent.keys.remove(idx));

        // Move keys and values from right child
        left_child.keys.append(&mut right_child.keys);
        if left_child.is_leaf {
            left_child.values.append(&mut right_child.values);
        }

        // Move children if internal
        if !left_child.is_leaf {
            left_child.children.append(&mut right_child.children);
        }

        // Remove right child from parent
        parent.children.remove(idx + 1);

        // Update metrics
        self.metrics
            .leaf_nodes
            .fetch_sub(1, AtomicOrdering::Relaxed);

        Ok(())
    }

    async fn scan_node(
        &self,
        node: &BTreeNode,
        start: &Key,
        end: &Key,
        results: &mut Vec<(Key, ValueLocation)>,
    ) -> Result<()> {
        if node.is_leaf {
            // Collect keys in range
            for (i, key) in node.keys.iter().enumerate() {
                if key >= start && key < end {
                    results.push((key.clone(), node.values[i].clone()));
                }
            }
        } else {
            // Find start index
            let mut i = 0;
            while i < node.keys.len() && &node.keys[i] < start {
                i += 1;
            }

            // Scan children
            while i < node.children.len() {
                let child = node.children[i].read().await;
                self.scan_node(&*child, start, end, results).await?;

                // Check if we have keys in this node that are in range
                if i < node.keys.len() {
                    let key = &node.keys[i];
                    if key >= start && key < end {
                        // For internal nodes, we don't store values
                        // This is a limitation of our current design
                    }
                }

                i += 1;
            }
        }
        Ok(())
    }
}

// Serialization implementation for persistence
impl BTreeIndex {
    /// Serialize the entire index for persistence
    pub async fn serialize(&self, strategy: &dyn SerializationStrategy) -> Result<Vec<u8>> {
        let root = self.root.read().await;
        let mut queue = VecDeque::new();
        queue.push_back(Arc::clone(&self.root));

        let mut nodes = Vec::new();

        while let Some(node_ref) = queue.pop_front() {
            let node = node_ref.read().await;
            nodes.push(BTreeNodeSerialized {
                keys: node.keys.clone(),
                values: node.values.clone(),
                children: Vec::new(), // We don't serialize children pointers
                is_leaf: node.is_leaf,
            });

            // Add children to queue
            if !node.is_leaf {
                for child in &node.children {
                    queue.push_back(Arc::clone(child));
                }
            }
        }

        strategy
            .serialize(&nodes)
            .map_err(|e| SGBDError::Serialization {
                context: format!("B-Tree serialization: {}", e),
                format: strategy.format(),
                backtrace: Some(std::backtrace::Backtrace::capture()),
            })
    }

    /// Deserialize the index from bytes
    pub async fn deserialize(
        bytes: &[u8],
        strategy: &dyn SerializationStrategy,
        config: BTreeConfig,
    ) -> Result<Self> {
        let nodes: Vec<BTreeNodeSerialized> =
            strategy
                .deserialize(bytes)
                .map_err(|e| SGBDError::Serialization {
                    context: format!("B-Tree deserialization: {}", e),
                    format: strategy.format(),
                    backtrace: Some(std::backtrace::Backtrace::capture()),
                })?;

        if nodes.is_empty() {
            return Err(SGBDError::Index {
                operation: "deserialize".to_string(),
                index_type: "B-Tree".to_string(),
                backtrace: Some(std::backtrace::Backtrace::capture()),
            });
        }

        // Reconstruct the tree
        let mut node_refs = Vec::with_capacity(nodes.len());
        for node in &nodes {
            node_refs.push(Arc::new(RwLock::new(BTreeNode {
                keys: node.keys.clone(),
                values: node.values.clone(),
                children: Vec::new(),
                is_leaf: node.is_leaf,
                parent: None,
            })));
        }

        // Rebuild parent-child relationships
        let mut child_index = 0;
        for (i, node) in nodes.iter().enumerate() {
            if !node.is_leaf {
                let children_count = node.keys.len() + 1;
                let mut node_ref = node_refs[i].write().await;

                for _ in 0..children_count {
                    if child_index >= node_refs.len() {
                        return Err(SGBDError::Index {
                            operation: "deserialize".to_string(),
                            index_type: "B-Tree".to_string(),
                            backtrace: Some(std::backtrace::Backtrace::capture()),
                        });
                    }

                    let child_ref = Arc::clone(&node_refs[child_index]);
                    {
                        let mut child = child_ref.write().await;
                        child.parent = Some(Arc::clone(&node_refs[i]));
                    }
                    node_ref.children.push(child_ref);
                    child_index += 1;
                }
            }
        }

        Ok(Self {
            root: Arc::clone(&node_refs[0]),
            config,
            metrics: BTreeMetrics::default(),
            cache: Mutex::new(lru::LruCache::new(config.cache_size_nodes)),
        })
    }
}

/// Serialized node format for persistence
#[derive(Serialize, Deserialize)]
struct BTreeNodeSerialized {
    keys: Vec<Key>,
    values: Vec<ValueLocation>,
    is_leaf: bool,
}

// Implementations for B-Tree visualization (for debugging)
impl fmt::Debug for BTreeNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BTreeNode")
            .field("keys", &self.keys)
            .field("is_leaf", &self.is_leaf)
            .field("num_children", &self.children.len())
            .finish()
    }
}

impl BTreeIndex {
    /// Print tree structure for debugging
    pub async fn print_tree(&self) {
        let root = self.root.read().await;
        self.print_node(&*root, 0).await;
    }

    async fn print_node(&self, node: &BTreeNode, level: usize) {
        let indent = "  ".repeat(level);
        println!("{}Node: {:?}", indent, node.keys);

        if !node.is_leaf {
            for child in &node.children {
                let child = child.read().await;
                self.print_node(&*child, level + 1).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Key, KeyId, Timestamp};
    use rand::{rngs::StdRng, Rng, SeedableRng};

    fn create_key(id: u64) -> Key {
        Key {
            id: KeyId::Numeric(id),
            timestamp: Timestamp::now(),
            schema_version: 1,
            tenant_id: None,
        }
    }

    fn create_location() -> ValueLocation {
        ValueLocation {
            segment_id: uuid::Uuid::new_v4(),
            offset: rand::random::<u64>(),
            size: rand::random::<usize>() % 1024,
        }
    }

    #[tokio::test]
    async fn test_btree_insert_and_search() {
        let config = BTreeConfig {
            order: 3,
            node_size_bytes: 4096,
            cache_size_nodes: 100,
            serialization_format: SerializationFormat::Bincode,
        };
        let btree = BTreeIndex::new(config).unwrap();

        // Insert 100 keys
        for i in 0..100 {
            let key = create_key(i);
            let location = create_location();
            btree.insert(key.clone(), location.clone()).await.unwrap();

            // Verify immediate lookup
            let result = btree.get(&key).await.unwrap();
            assert_eq!(result, Some(location));
        }

        // Verify all keys exist
        for i in 0..100 {
            let key = create_key(i);
            let result = btree.get(&key).await.unwrap();
            assert!(result.is_some());
        }

        // Verify non-existent key
        let non_existent = create_key(1000);
        assert!(btree.get(&non_existent).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_btree_delete() {
        let config = BTreeConfig {
            order: 3,
            node_size_bytes: 4096,
            cache_size_nodes: 100,
            serialization_format: SerializationFormat::Bincode,
        };
        let btree = BTreeIndex::new(config).unwrap();

        // Insert keys
        let keys: Vec<Key> = (0..50).map(create_key).collect();
        for key in &keys {
            btree.insert(key.clone(), create_location()).await.unwrap();
        }

        // Delete every other key
        for (i, key) in keys.iter().enumerate() {
            if i % 2 == 0 {
                btree.delete(key).await.unwrap();
            }
        }

        // Verify deletions
        for (i, key) in keys.iter().enumerate() {
            let result = btree.get(key).await.unwrap();
            if i % 2 == 0 {
                assert!(result.is_none());
            } else {
                assert!(result.is_some());
            }
        }
    }

    #[tokio::test]
    async fn test_btree_range_scan() {
        let config = BTreeConfig {
            order: 4,
            node_size_bytes: 4096,
            cache_size_nodes: 100,
            serialization_format: SerializationFormat::Bincode,
        };
        let btree = BTreeIndex::new(config).unwrap();

        // Insert keys in random order
        let mut rng = StdRng::seed_from_u64(42);
        let mut keys: Vec<u64> = (0..100).collect();
        keys.shuffle(&mut rng);

        for &key in &keys {
            btree
                .insert(create_key(key), create_location())
                .await
                .unwrap();
        }

        // Range scan [20, 50)
        let start = create_key(20);
        let end = create_key(50);
        let results = btree.range_scan(&start, &end).await.unwrap();

        // Verify all results are in range
        for (key, _) in &results {
            assert!(key >= &start && key < &end);
        }

        // Verify count
        assert_eq!(results.len(), 30);

        // Verify sorted order
        for i in 1..results.len() {
            assert!(results[i - 1].0 < results[i].0);
        }
    }

    #[tokio::test]
    async fn test_btree_serialization() {
        let config = BTreeConfig {
            order: 3,
            node_size_bytes: 4096,
            cache_size_nodes: 100,
            serialization_format: SerializationFormat::Bincode,
        };
        let btree = BTreeIndex::new(config).unwrap();

        // Insert keys
        for i in 0..100 {
            btree
                .insert(create_key(i), create_location())
                .await
                .unwrap();
        }

        // Serialize
        let strategy = BincodeStrategy;
        let bytes = btree.serialize(&strategy).await.unwrap();

        // Deserialize
        let new_btree = BTreeIndex::deserialize(&bytes, &strategy, config)
            .await
            .unwrap();

        // Verify all keys exist
        for i in 0..100 {
            let key = create_key(i);
            let original = btree.get(&key).await.unwrap();
            let restored = new_btree.get(&key).await.unwrap();
            assert_eq!(original, restored);
        }
    }

    #[tokio::test]
    async fn test_btree_cache() {
        let config = BTreeConfig {
            order: 3,
            node_size_bytes: 4096,
            cache_size_nodes: 10,
            serialization_format: SerializationFormat::Bincode,
        };
        let btree = BTreeIndex::new(config).unwrap();

        // Insert 20 keys
        for i in 0..20 {
            btree
                .insert(create_key(i), create_location())
                .await
                .unwrap();
        }

        // Access first 15 keys
        for i in 0..15 {
            btree.get(&create_key(i)).await.unwrap();
        }

        // Check cache hits
        let metrics = btree.metrics();
        assert_eq!(metrics.cache_hit_ratio > 0.0, true);
        assert_eq!(metrics.cache_hits > 0, true);
    }

    #[tokio::test]
    async fn test_btree_node_split_merge() {
        let config = BTreeConfig {
            order: 2, // Small order to force splits/merges
            node_size_bytes: 256,
            cache_size_nodes: 10,
            serialization_format: SerializationFormat::Bincode,
        };
        let btree = BTreeIndex::new(config).unwrap();

        // Insert keys to force splits
        for i in 0..10 {
            btree
                .insert(create_key(i), create_location())
                .await
                .unwrap();
        }

        let metrics = btree.metrics();
        assert!(metrics.node_splits > 0);
        assert_eq!(metrics.tree_height > 1, true);

        // Delete keys to force merges
        for i in 0..10 {
            if i % 2 == 0 {
                btree.delete(&create_key(i)).await.unwrap();
            }
        }

        let metrics = btree.metrics();
        assert!(metrics.node_merges > 0);
    }
}

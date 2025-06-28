use crate::sgbd::{
    DatabaseConfig, IsolationLevel, Key, LockInfo, LockType, Result, SGBDError, Savepoint,
    Timestamp, TransactionContext,
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{Mutex, Notify, RwLock},
    time,
};
use uuid::Uuid;

/// Transaction manager state
#[derive(Debug)]
struct TxManagerState {
    active_transactions: HashMap<Uuid, TransactionContext>,
    lock_table: HashMap<Key, LockInfo>,
    wait_for_graph: HashMap<Uuid, HashSet<Uuid>>,
    transaction_queue: VecDeque<Uuid>,
    metrics: TxMetrics,
}

/// Transaction manager
#[derive(Debug, Clone)]
pub struct TransactionManager {
    state: Arc<RwLock<TxManagerState>>,
    config: Arc<DatabaseConfig>,
    deadlock_detector: Arc<Notify>,
    shutdown: Arc<Notify>,
    key_notify: Arc<Mutex<HashMap<Key, Arc<Notify>>>>,
}

/// Transaction metrics
#[derive(Debug, Default)]
struct TxMetrics {
    active_count: AtomicU64,
    committed_count: AtomicU64,
    rolled_back_count: AtomicU64,
    deadlock_count: AtomicU64,
    lock_wait_time_us: AtomicU64,
    lock_wait_count: AtomicU64,
}

/// Transaction metrics snapshot
pub struct TxMetricsSnapshot {
    pub active: u64,
    pub committed: u64,
    pub rolled_back: u64,
    pub deadlocks: u64,
    pub avg_lock_wait_us: u64,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new(config: Arc<DatabaseConfig>) -> Self {
        let state = TxManagerState {
            active_transactions: HashMap::new(),
            lock_table: HashMap::new(),
            wait_for_graph: HashMap::new(),
            transaction_queue: VecDeque::new(),
            metrics: TxMetrics::default(),
        };

        let manager = Self {
            state: Arc::new(RwLock::new(state)),
            config: config.clone(),
            deadlock_detector: Arc::new(Notify::new()),
            shutdown: Arc::new(Notify::new()),
            key_notify: Arc::new(Mutex::new(HashMap::new())),
        };

        // Start background deadlock detector
        tokio::spawn(manager.clone().deadlock_detection_task());

        manager
    }

    /// Begin a new transaction
    pub async fn begin(&self, isolation_level: IsolationLevel) -> Result<TransactionContext> {
        let mut state = self.state.write().await;

        // Enforce max connections
        if state.active_transactions.len() >= self.config.max_connections as usize {
            return Err(SGBDError::ResourceExhausted {
                resource: "database connections".to_string(),
                backtrace: Some(std::backtrace::Backtrace::capture()),
            });
        }

        let tx = TransactionContext::new(isolation_level);
        state.active_transactions.insert(tx.id, tx.clone());
        state.transaction_queue.push_back(tx.id);
        state.metrics.active_count.fetch_add(1, Ordering::Relaxed);

        Ok(tx)
    }

    /// Commit a transaction
    pub async fn commit(&self, tx_id: Uuid) -> Result<()> {
        let mut state = self.state.write().await;
        let tx = state
            .active_transactions
            .get(&tx_id)
            .ok_or(SGBDError::Transaction {
                reason: "Transaction not found".to_string(),
                tx_id: Some(tx_id),
                backtrace: Some(std::backtrace::Backtrace::capture()),
            })?;

        // Release all locks and notify waiters
        for lock in &tx.locks {
            state.lock_table.remove(&lock.key);

            // Notify any waiters for this key
            let mut key_notify = self.key_notify.lock().await;
            if let Some(notify) = key_notify.remove(&lock.key) {
                notify.notify_waiters();
            }
        }

        // Remove from active transactions
        state.active_transactions.remove(&tx_id);
        state.metrics.active_count.fetch_sub(1, Ordering::Relaxed);
        state
            .metrics
            .committed_count
            .fetch_add(1, Ordering::Relaxed);

        // Remove from queue and wait-for graph
        if let Some(pos) = state.transaction_queue.iter().position(|id| *id == tx_id) {
            state.transaction_queue.remove(pos);
        }
        state.wait_for_graph.remove(&tx_id);

        Ok(())
    }

    /// Rollback a transaction
    pub async fn rollback(&self, tx_id: Uuid) -> Result<()> {
        let mut state = self.state.write().await;
        let tx = state
            .active_transactions
            .get(&tx_id)
            .ok_or(SGBDError::Transaction {
                reason: "Transaction not found".to_string(),
                tx_id: Some(tx_id),
                backtrace: Some(std::backtrace::Backtrace::capture()),
            })?;

        // Release all locks and notify waiters
        for lock in &tx.locks {
            state.lock_table.remove(&lock.key);

            // Notify any waiters for this key
            let mut key_notify = self.key_notify.lock().await;
            if let Some(notify) = key_notify.remove(&lock.key) {
                notify.notify_waiters();
            }
        }

        // Remove from active transactions
        state.active_transactions.remove(&tx_id);
        state.metrics.active_count.fetch_sub(1, Ordering::Relaxed);
        state
            .metrics
            .rolled_back_count
            .fetch_add(1, Ordering::Relaxed);

        // Remove from queue and wait-for graph
        if let Some(pos) = state.transaction_queue.iter().position(|id| *id == tx_id) {
            state.transaction_queue.remove(pos);
        }
        state.wait_for_graph.remove(&tx_id);

        Ok(())
    }

    /// Rollback to a savepoint
    pub async fn rollback_to_savepoint(&self, tx_id: Uuid, savepoint_name: &str) -> Result<()> {
        let mut state = self.state.write().await;
        let tx = state
            .active_transactions
            .get_mut(&tx_id)
            .ok_or(SGBDError::Transaction {
                reason: "Transaction not found".to_string(),
                tx_id: Some(tx_id),
                backtrace: Some(std::backtrace::Backtrace::capture()),
            })?;

        // Find savepoint position
        let savepoint_pos = tx
            .savepoints
            .iter()
            .position(|sp| sp.name == savepoint_name)
            .ok_or(SGBDError::Transaction {
                reason: format!("Savepoint '{}' not found", savepoint_name),
                tx_id: Some(tx_id),
                backtrace: Some(std::backtrace::Backtrace::capture()),
            })?;

        // Release locks acquired after this savepoint
        let locks_to_release: Vec<_> = tx.locks.drain(savepoint_pos..).collect();

        for lock in locks_to_release {
            state.lock_table.remove(&lock.key);

            // Notify any waiters for this key
            let mut key_notify = self.key_notify.lock().await;
            if let Some(notify) = key_notify.remove(&lock.key) {
                notify.notify_waiters();
            }
        }

        // Remove savepoints after this one
        tx.savepoints.truncate(savepoint_pos + 1);

        Ok(())
    }

    /// Acquire a lock with timeout and deadlock detection
    pub async fn acquire_lock(&self, tx_id: Uuid, key: Key, lock_type: LockType) -> Result<()> {
        let start = Instant::now();
        let mut state = self.state.write().await;

        // Check transaction validity
        if !state.active_transactions.contains_key(&tx_id) {
            return Err(SGBDError::Transaction {
                reason: "Transaction not active".to_string(),
                tx_id: Some(tx_id),
                backtrace: Some(std::backtrace::Backtrace::capture()),
            });
        }

        // Check if lock is already held by this transaction
        if let Some(existing_lock) = state.lock_table.get(&key) {
            if existing_lock.tx_id == tx_id {
                // Upgrade lock if needed
                if existing_lock.lock_type.can_upgrade_to(&lock_type) {
                    return Ok(());
                }
            }
        }

        // Check for conflicts
        if let Some(existing_lock) = state.lock_table.get(&key) {
            if existing_lock.lock_type.conflicts_with(&lock_type) {
                // Get transaction holding the lock
                let holder_id = existing_lock.tx_id;

                // Add to wait-for graph
                state
                    .wait_for_graph
                    .entry(tx_id)
                    .or_default()
                    .insert(holder_id);

                // Check for deadlock
                if self.detect_cycle(&state.wait_for_graph, tx_id).await {
                    state.metrics.deadlock_count.fetch_add(1, Ordering::Relaxed);
                    return Err(SGBDError::Concurrency {
                        operation: "lock acquisition (deadlock detected)".to_string(),
                        backtrace: Some(std::backtrace::Backtrace::capture()),
                    });
                }

                // Prepare to wait for lock release
                let notify = {
                    let mut key_notify = self.key_notify.lock().await;
                    key_notify
                        .entry(key.clone())
                        .or_insert_with(|| Arc::new(Notify::new()))
                        .clone()
                };

                // Release state lock while waiting
                drop(state);

                // Wait for notification or timeout
                let timeout = Duration::from_millis(self.config.lock_timeout_ms);
                tokio::select! {
                    _ = notify.notified() => {
                        // Lock released, try again
                        return self.acquire_lock(tx_id, key, lock_type).await;
                    }
                    _ = time::sleep(timeout) => {
                        return Err(SGBDError::Concurrency {
                            operation: "lock acquisition (timeout)".to_string(),
                            backtrace: Some(std::backtrace::Backtrace::capture()),
                        });
                    }
                }
            }
        }

        // Acquire lock
        let lock_info = LockInfo {
            key: key.clone(),
            lock_type,
            tx_id,
            acquired_at: Timestamp::now(),
        };

        state.lock_table.insert(key, lock_info.clone());

        // Add to transaction
        if let Some(tx) = state.active_transactions.get_mut(&tx_id) {
            tx.locks.push(lock_info);
        }

        // Remove from wait-for graph since we got the lock
        state.wait_for_graph.remove(&tx_id);

        // Update metrics
        let wait_time = start.elapsed().as_micros() as u64;
        state
            .metrics
            .lock_wait_time_us
            .fetch_add(wait_time, Ordering::Relaxed);
        state
            .metrics
            .lock_wait_count
            .fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Detect cycles in wait-for graph using DFS
    async fn detect_cycle(&self, graph: &HashMap<Uuid, HashSet<Uuid>>, start: Uuid) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![start];
        let mut on_stack = HashSet::new();
        on_stack.insert(start);

        while let Some(node) = stack.last().copied() {
            visited.insert(node);

            if let Some(neighbors) = graph.get(&node) {
                let mut has_unvisited = false;

                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        if !on_stack.insert(neighbor) {
                            // Found a cycle
                            return true;
                        }
                        stack.push(neighbor);
                        has_unvisited = true;
                        break;
                    } else if on_stack.contains(&neighbor) {
                        // Found a cycle
                        return true;
                    }
                }

                if !has_unvisited {
                    on_stack.remove(&stack.pop().unwrap());
                }
            } else {
                stack.pop();
            }
        }

        false
    }

    /// Background deadlock detection task
    async fn deadlock_detection_task(self) {
        let interval = Duration::from_millis(self.config.deadlock_detection_interval_ms);

        loop {
            tokio::select! {
                _ = time::sleep(interval) => {
                    if let Err(e) = self.resolve_deadlocks().await {
                        log::error!("Deadlock detection failed: {}", e);
                    }
                }
                _ = self.deadlock_detector.notified() => {
                    if let Err(e) = self.resolve_deadlocks().await {
                        log::error!("Immediate deadlock detection failed: {}", e);
                    }
                }
                _ = self.shutdown.notified() => break,
            }
        }
    }

    /// Detect and resolve deadlocks
    async fn resolve_deadlocks(&self) -> Result<()> {
        let state = self.state.read().await;
        let graph = &state.wait_for_graph;

        // Find all transactions in cycles
        let mut victims = Vec::new();
        let mut visited = HashSet::new();

        for &tx_id in graph.keys() {
            if visited.contains(&tx_id) {
                continue;
            }

            let mut path = Vec::new();
            if self.find_cycle_dfs(graph, tx_id, &mut visited, &mut path, &mut HashSet::new()) {
                if let Some(victim) = self.select_victim(&path, &state).await {
                    victims.push(victim);
                }
            }
        }

        // Release lock before rolling back
        drop(state);

        // Abort victim transactions
        for victim in victims {
            self.rollback(victim).await?;
        }

        Ok(())
    }

    /// DFS cycle detection with path tracking
    fn find_cycle_dfs(
        &self,
        graph: &HashMap<Uuid, HashSet<Uuid>>,
        node: Uuid,
        visited: &mut HashSet<Uuid>,
        path: &mut Vec<Uuid>,
        current_path: &mut HashSet<Uuid>,
    ) -> bool {
        visited.insert(node);
        path.push(node);
        current_path.insert(node);

        if let Some(neighbors) = graph.get(&node) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    if self.find_cycle_dfs(graph, neighbor, visited, path, current_path) {
                        return true;
                    }
                } else if current_path.contains(&neighbor) {
                    // Found cycle
                    return true;
                }
            }
        }

        path.pop();
        current_path.remove(&node);
        false
    }

    /// Select victim transaction to abort
    async fn select_victim(&self, cycle: &[Uuid], state: &TxManagerState) -> Option<Uuid> {
        cycle
            .iter()
            .min_by_key(|&&tx_id| {
                state
                    .active_transactions
                    .get(&tx_id)
                    .map(|tx| tx.read_timestamp)
            })
            .copied()
    }

    /// Create a savepoint
    pub async fn create_savepoint(
        &self,
        tx_id: Uuid,
        name: String,
        wal_position: u64,
    ) -> Result<()> {
        let mut state = self.state.write().await;
        let tx = state
            .active_transactions
            .get_mut(&tx_id)
            .ok_or(SGBDError::Transaction {
                reason: "Transaction not found".to_string(),
                tx_id: Some(tx_id),
                backtrace: Some(std::backtrace::Backtrace::capture()),
            })?;

        tx.create_savepoint(name, wal_position);
        Ok(())
    }

    /// Get transaction metrics
    pub async fn metrics(&self) -> TxMetricsSnapshot {
        let state = self.state.read().await;
        let wait_count = state.metrics.lock_wait_count.load(Ordering::Relaxed);
        let total_wait = state.metrics.lock_wait_time_us.load(Ordering::Relaxed);

        TxMetricsSnapshot {
            active: state.metrics.active_count.load(Ordering::Relaxed),
            committed: state.metrics.committed_count.load(Ordering::Relaxed),
            rolled_back: state.metrics.rolled_back_count.load(Ordering::Relaxed),
            deadlocks: state.metrics.deadlock_count.load(Ordering::Relaxed),
            avg_lock_wait_us: if wait_count > 0 {
                total_wait / wait_count
            } else {
                0
            },
        }
    }

    /// Gracefully shutdown transaction manager
    pub async fn shutdown(&self) {
        self.shutdown.notify_one();

        // Wait for active transactions to complete
        let timeout = Duration::from_millis(self.config.connection_timeout_ms);
        let start = Instant::now();

        while self.metrics().await.active > 0 {
            if start.elapsed() > timeout {
                log::warn!(
                    "Transaction shutdown timed out with {} active transactions",
                    self.metrics().await.active
                );
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

impl LockType {
    /// Check if two lock types conflict
    pub fn conflicts_with(&self, other: &LockType) -> bool {
        match (self, other) {
            (LockType::Exclusive, _) => true,
            (_, LockType::Exclusive) => true,
            (LockType::IntentExclusive, LockType::Shared) => true,
            (LockType::Shared, LockType::IntentExclusive) => true,
            _ => false,
        }
    }

    /// Check if lock can be upgraded
    pub fn can_upgrade_to(&self, new_type: &LockType) -> bool {
        match (self, new_type) {
            (LockType::Shared, LockType::Exclusive) => true,
            (LockType::IntentShared, LockType::Shared) => true,
            (LockType::IntentShared, LockType::Exclusive) => true,
            (LockType::IntentExclusive, LockType::Exclusive) => true,
            (a, b) if a == b => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DatabaseConfig, Key};

    async fn test_manager() -> TransactionManager {
        let config = DatabaseConfig::development();
        TransactionManager::new(Arc::new(config))
    }

    #[tokio::test]
    async fn test_begin_commit() {
        let manager = test_manager().await;
        let tx = manager.begin(IsolationLevel::ReadCommitted).await.unwrap();
        manager.commit(tx.id).await.unwrap();

        let metrics = manager.metrics().await;
        assert_eq!(metrics.active, 0);
        assert_eq!(metrics.committed, 1);
    }

    #[tokio::test]
    async fn test_begin_rollback() {
        let manager = test_manager().await;
        let tx = manager.begin(IsolationLevel::ReadCommitted).await.unwrap();
        manager.rollback(tx.id).await.unwrap();

        let metrics = manager.metrics().await;
        assert_eq!(metrics.active, 0);
        assert_eq!(metrics.rolled_back, 1);
    }

    #[tokio::test]
    async fn test_lock_acquisition() {
        let manager = test_manager().await;
        let tx1 = manager.begin(IsolationLevel::ReadCommitted).await.unwrap();
        let key = Key::new_uuid();

        // First lock should succeed
        manager
            .acquire_lock(tx1.id, key.clone(), LockType::Shared)
            .await
            .unwrap();

        let tx2 = manager.begin(IsolationLevel::ReadCommitted).await.unwrap();

        // Conflicting lock should fail after timeout
        assert!(manager
            .acquire_lock(tx2.id, key.clone(), LockType::Exclusive)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_lock_upgrade() {
        let manager = test_manager().await;
        let tx = manager.begin(IsolationLevel::ReadCommitted).await.unwrap();
        let key = Key::new_uuid();

        manager
            .acquire_lock(tx.id, key.clone(), LockType::Shared)
            .await
            .unwrap();

        // Upgrade to exclusive should succeed
        manager
            .acquire_lock(tx.id, key.clone(), LockType::Exclusive)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_deadlock_detection() {
        let manager = test_manager().await;
        let key1 = Key::new_uuid();
        let key2 = Key::new_uuid();

        let tx1 = manager.begin(IsolationLevel::ReadCommitted).await.unwrap();
        let tx2 = manager.begin(IsolationLevel::ReadCommitted).await.unwrap();

        // TX1 locks key1
        manager
            .acquire_lock(tx1.id, key1.clone(), LockType::Exclusive)
            .await
            .unwrap();

        // TX2 locks key2
        manager
            .acquire_lock(tx2.id, key2.clone(), LockType::Exclusive)
            .await
            .unwrap();

        // TX1 tries to lock key2 (will wait)
        let handle1 = tokio::spawn({
            let manager = manager.clone();
            async move {
                manager
                    .acquire_lock(tx1.id, key2.clone(), LockType::Exclusive)
                    .await
            }
        });

        // TX2 tries to lock key1 (will cause deadlock)
        let handle2 = tokio::spawn({
            let manager = manager.clone();
            async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                manager
                    .acquire_lock(tx2.id, key1.clone(), LockType::Exclusive)
                    .await
            }
        });

        // One of them should be aborted due to deadlock
        let result1 = handle1.await.unwrap();
        let result2 = handle2.await.unwrap();

        assert!(
            result1.is_err() || result2.is_err(),
            "One transaction should have been aborted"
        );

        let metrics = manager.metrics().await;
        assert_eq!(metrics.deadlocks, 1);
    }

    #[tokio::test]
    async fn test_savepoints() {
        let manager = test_manager().await;
        let tx = manager.begin(IsolationLevel::ReadCommitted).await.unwrap();
        let key = Key::new_uuid();

        manager
            .create_savepoint(tx.id, "sp1".to_string(), 100)
            .await
            .unwrap();

        // Acquire lock after savepoint
        manager
            .acquire_lock(tx.id, key.clone(), LockType::Shared)
            .await
            .unwrap();

        // Rollback to savepoint
        manager.rollback_to_savepoint(tx.id, "sp1").await.unwrap();

        // Lock should have been released
        let state = manager.state.read().await;
        assert!(!state.lock_table.contains_key(&key));
    }

    #[tokio::test]
    async fn test_shutdown_with_active_transactions() {
        let manager = test_manager().await;
        let tx = manager.begin(IsolationLevel::ReadCommitted).await.unwrap();
        let key = Key::new_uuid();

        // Acquire and hold lock
        manager
            .acquire_lock(tx.id, key.clone(), LockType::Exclusive)
            .await
            .unwrap();

        // Start shutdown
        let handle = tokio::spawn({
            let manager = manager.clone();
            async move {
                manager.shutdown().await;
            }
        });

        // Verify shutdown completes
        tokio::time::timeout(Duration::from_secs(1), handle)
            .await
            .expect("Shutdown timed out")
            .expect("Shutdown failed");
    }
}

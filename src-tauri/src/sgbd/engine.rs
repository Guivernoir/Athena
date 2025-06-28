use crate::llm::{sanitized_input, Mode, ParsedInput, Proficiency, RawOutput};
use crate::quantization::{QuantizationConfig, Quantizer};
use crate::sgbd::*;
use crate::Engine::ParsedOutput;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;
use uuid::Uuid;

/// Strategic Database Management Engine
///
/// A precision-engineered SGBD that treats data like classified intelligence:
/// - Every byte quantized for maximum compression efficiency
/// - Multi-layered caching with battlefield-tested eviction strategies
/// - ACID transactions with KGB-level reliability
/// - Index structures that would make MI6 proud
pub struct Engine {
    storage: Arc<RwLock<StorageEngine>>,
    tx_manager: Arc<TransactionManager>,
    wal: Arc<Mutex<Wal>>,
    index: Arc<RwLock<BTreeIndex>>,
    quantizer: Arc<Quantizer>,
    config: DatabaseConfig,
    state: Arc<RwLock<EngineState>>,
    metrics: Arc<RwLock<EngineMetrics>>,
    background_tasks: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

/// Engine operational state - because knowing your position is half the battle
#[derive(Debug, Clone)]
pub struct EngineState {
    pub started: bool,
    pub healthy: bool,
    pub last_compaction: Option<Timestamp>,
    pub recovery_mode: bool,
    pub shutdown_requested: bool,
}

/// Engine performance metrics - tactical intelligence on our operations
#[derive(Debug, Clone, Default)]
pub struct EngineMetrics {
    pub operations_total: u64,
    pub operations_successful: u64,
    pub operations_failed: u64,
    pub transactions_committed: u64,
    pub transactions_aborted: u64,
    pub quantization_ratio: f64,
    pub cache_hit_rate: f64,
    pub average_response_time_ms: f64,
    pub last_updated: Option<Timestamp>,
}

/// Query execution context - mission parameters for data operations
#[derive(Debug, Clone)]
pub struct QueryExecution {
    pub tx_id: Uuid,
    pub timeout: Duration,
    pub proficiency: Proficiency,
    pub mode: Mode,
    pub quantization_config: QuantizationConfig,
}

impl Engine {
    /// Initialize the engine with the precision of a Swiss watchmaker
    /// and the paranoia of a Cold War intelligence operative
    pub async fn new(config: DatabaseConfig) -> Result<Self, SGBDError> {
        // Phase 1: Initialize core components with military precision
        let storage = Arc::new(RwLock::new(
            StorageEngine::new(config.clone()).await.map_err(|e| {
                SGBDError::ResourceExhausted(format!("Storage initialization failed: {}", e))
            })?,
        ));

        let tx_manager = Arc::new(TransactionManager::new(config.clone()).await.map_err(|e| {
            SGBDError::Concurrency(format!("Transaction manager initialization failed: {}", e))
        })?);

        let wal = Arc::new(Mutex::new(Wal::new(config.clone()).await.map_err(|e| {
            SGBDError::Wal(format!("WAL initialization failed: {}", e))
        })?));

        let index = Arc::new(RwLock::new(BTreeIndex::new(config.clone()).await.map_err(
            |e| SGBDError::Index(format!("Index initialization failed: {}", e)),
        )?));

        let quantizer = Arc::new(Quantizer::new(config.quantization_config.clone()).map_err(
            |e| SGBDError::Quantization(format!("Quantizer initialization failed: {}", e)),
        )?);

        let state = Arc::new(RwLock::new(EngineState {
            started: false,
            healthy: false,
            last_compaction: None,
            recovery_mode: false,
            shutdown_requested: false,
        }));

        let metrics = Arc::new(RwLock::new(EngineMetrics::default()));
        let background_tasks = Arc::new(Mutex::new(Vec::new()));

        Ok(Engine {
            storage,
            tx_manager,
            wal,
            index,
            quantizer,
            config,
            state,
            metrics,
            background_tasks,
        })
    }

    /// Start the engine with the ceremony of a military parade
    pub async fn start(&self) -> Result<(), SGBDError> {
        let mut state = self.state.write().await;

        if state.started {
            return Err(SGBDError::Concurrency(
                "Engine already started - double deployment detected".into(),
            ));
        }

        // Phase 1: WAL Recovery - reconstruct from the battlefield logs
        state.recovery_mode = true;
        self.perform_wal_recovery().await?;

        // Phase 2: Index reconstruction - rebuild our intelligence network
        self.rebuild_indexes_from_storage().await?;

        // Phase 3: Start background operations - deploy the sleeper agents
        self.start_background_tasks().await?;

        // Phase 4: Health validation - confirm all systems operational
        let health_check = self.validate_system_health().await?;
        if !health_check {
            return Err(SGBDError::ResourceExhausted(
                "System failed health validation".into(),
            ));
        }

        state.started = true;
        state.healthy = true;
        state.recovery_mode = false;

        Ok(())
    }

    /// Execute a strategic data retrieval operation
    pub async fn get(&self, key: Key, context: QueryExecution) -> Result<Option<Value>, SGBDError> {
        let start_time = Instant::now();
        self.ensure_operational().await?;

        // Begin transaction with the discipline of a chess opening
        let tx_context = self.begin_transaction(context.clone()).await?;

        let result = timeout(context.timeout, async {
            // Acquire shared lock for read operation
            self.tx_manager
                .acquire_lock(tx_context.transaction_id, key.clone(), LockType::Shared)
                .await?;

            // Check cache first - intelligence from recent operations
            if let Some(cached_value) = self.check_cache(&key).await? {
                self.update_cache_metrics(true).await;
                return Ok(Some(cached_value));
            }
            self.update_cache_metrics(false).await;

            // Consult the index for target location
            let index = self.index.read().await;
            let location = match index.search(&key).await? {
                Some(loc) => loc,
                None => return Ok(None), // Target not in our intelligence files
            };

            // Retrieve from storage with full decompression pipeline
            let storage = self.storage.read().await;
            let compressed_value = storage.get_at_location(&location).await?;

            // Dequantize the retrieved intelligence
            let value = self.quantizer.dequantize(compressed_value)?;

            // Update cache with fresh intelligence
            self.update_cache(key.clone(), value.clone()).await?;

            Ok(Some(value))
        })
        .await;

        let elapsed = start_time.elapsed();

        match result {
            Ok(Ok(value)) => {
                self.commit_transaction(tx_context).await?;
                self.update_operation_metrics(true, "get", elapsed).await;
                Ok(value)
            }
            Ok(Err(e)) => {
                self.rollback_transaction(tx_context).await?;
                self.update_operation_metrics(false, "get", elapsed).await;
                Err(e)
            }
            Err(_) => {
                self.rollback_transaction(tx_context).await?;
                self.update_operation_metrics(false, "get", elapsed).await;
                Err(SGBDError::ResourceExhausted(
                    "Query timeout - mission aborted".into(),
                ))
            }
        }
    }

    /// Execute a tactical data insertion operation
    pub async fn set(
        &self,
        key: Key,
        value: Value,
        context: QueryExecution,
    ) -> Result<(), SGBDError> {
        let start_time = Instant::now();
        self.ensure_operational().await?;

        let tx_context = self.begin_transaction(context.clone()).await?;

        let result = timeout(context.timeout, async {
            // Acquire exclusive lock - we need full control of this asset
            self.tx_manager
                .acquire_lock(tx_context.transaction_id, key.clone(), LockType::Exclusive)
                .await?;

            // Quantize the intelligence for optimal storage
            let quantized_value = self.quantizer.quantize(&value)?;

            // Record the operation in our permanent logs
            let wal_record = WalRecord {
                operation: Operation::Insert {
                    key: key.clone(),
                    value: quantized_value.clone(),
                },
                timestamp: Timestamp::now(),
            };

            let mut wal = self.wal.lock().await;
            wal.append(wal_record).await?;
            drop(wal); // Release WAL lock early

            // Store in the main archive
            let storage_location = {
                let mut storage = self.storage.write().await;
                storage.set(key.clone(), quantized_value).await?
            };

            // Update our intelligence index
            {
                let mut index = self.index.write().await;
                index.insert(key.clone(), storage_location).await?;
            }

            // Update cache with the new intelligence
            self.update_cache(key, value).await?;

            Ok(())
        })
        .await;

        let elapsed = start_time.elapsed();

        match result {
            Ok(Ok(_)) => {
                self.commit_transaction(tx_context).await?;
                self.update_operation_metrics(true, "set", elapsed).await;
                Ok(())
            }
            Ok(Err(e)) => {
                self.rollback_transaction(tx_context).await?;
                self.update_operation_metrics(false, "set", elapsed).await;
                Err(e)
            }
            Err(_) => {
                self.rollback_transaction(tx_context).await?;
                self.update_operation_metrics(false, "set", elapsed).await;
                Err(SGBDError::ResourceExhausted(
                    "Operation timeout - mission aborted".into(),
                ))
            }
        }
    }

    /// Execute a precision strike data deletion
    pub async fn delete(&self, key: Key, context: QueryExecution) -> Result<bool, SGBDError> {
        let start_time = Instant::now();
        self.ensure_operational().await?;

        let tx_context = self.begin_transaction(context.clone()).await?;

        let result = timeout(context.timeout, async {
            // Acquire exclusive lock for deletion operation
            self.tx_manager
                .acquire_lock(tx_context.transaction_id, key.clone(), LockType::Exclusive)
                .await?;

            // Check if target exists in our intelligence files
            let index = self.index.read().await;
            let exists = index.search(&key).await?.is_some();
            drop(index);

            if !exists {
                return Ok(false); // Target was never in our files
            }

            // Record the elimination order in permanent logs
            let wal_record = WalRecord {
                operation: Operation::Delete { key: key.clone() },
                timestamp: Timestamp::now(),
            };

            let mut wal = self.wal.lock().await;
            wal.append(wal_record).await?;
            drop(wal);

            // Execute the deletion from storage (tombstone marker)
            {
                let mut storage = self.storage.write().await;
                storage.delete(key.clone()).await?;
            }

            // Remove from index
            {
                let mut index = self.index.write().await;
                index.delete(&key).await?;
            }

            // Purge from cache
            self.invalidate_cache(&key).await?;

            Ok(true)
        })
        .await;

        let elapsed = start_time.elapsed();

        match result {
            Ok(Ok(deleted)) => {
                self.commit_transaction(tx_context).await?;
                self.update_operation_metrics(true, "delete", elapsed).await;
                Ok(deleted)
            }
            Ok(Err(e)) => {
                self.rollback_transaction(tx_context).await?;
                self.update_operation_metrics(false, "delete", elapsed)
                    .await;
                Err(e)
            }
            Err(_) => {
                self.rollback_transaction(tx_context).await?;
                self.update_operation_metrics(false, "delete", elapsed)
                    .await;
                Err(SGBDError::ResourceExhausted(
                    "Deletion timeout - target remains active".into(),
                ))
            }
        }
    }

    /// Execute a reconnaissance mission across key ranges
    pub async fn range_scan(
        &self,
        start_key: Key,
        end_key: Key,
        limit: Option<usize>,
        context: QueryExecution,
    ) -> Result<Vec<(Key, Value)>, SGBDError> {
        let start_time = Instant::now();
        self.ensure_operational().await?;

        let tx_context = self.begin_transaction(context.clone()).await?;

        let result = timeout(context.timeout, async {
            // Acquire shared locks on the entire range - we need read access to the territory
            // Note: In a real implementation, we'd use range locking or intent locks
            // For now, we'll proceed with the scan and handle conflicts as they arise

            let index = self.index.read().await;
            let key_locations = index.range_scan(start_key, end_key, limit).await?;
            drop(index);

            let mut results = Vec::with_capacity(key_locations.len());
            let storage = self.storage.read().await;

            for (key, location) in key_locations {
                // Acquire shared lock for each key during scan
                self.tx_manager
                    .acquire_lock(tx_context.transaction_id, key.clone(), LockType::Shared)
                    .await?;

                // Retrieve and dequantize each piece of intelligence
                let compressed_value = storage.get_at_location(&location).await?;
                let value = self.quantizer.dequantize(compressed_value)?;

                results.push((key, value));
            }

            Ok(results)
        })
        .await;

        let elapsed = start_time.elapsed();

        match result {
            Ok(Ok(results)) => {
                self.commit_transaction(tx_context).await?;
                self.update_operation_metrics(true, "range_scan", elapsed)
                    .await;
                Ok(results)
            }
            Ok(Err(e)) => {
                self.rollback_transaction(tx_context).await?;
                self.update_operation_metrics(false, "range_scan", elapsed)
                    .await;
                Err(e)
            }
            Err(_) => {
                self.rollback_transaction(tx_context).await?;
                self.update_operation_metrics(false, "range_scan", elapsed)
                    .await;
                Err(SGBDError::ResourceExhausted(
                    "Reconnaissance timeout - mission incomplete".into(),
                ))
            }
        }
    }

    /// Execute batch operations with atomic precision
    pub async fn batch_execute(
        &self,
        operations: Vec<BatchOperation>,
        context: QueryExecution,
    ) -> Result<BatchResult, SGBDError> {
        let start_time = Instant::now();
        self.ensure_operational().await?;

        let tx_context = self.begin_transaction(context.clone()).await?;

        let result = timeout(context.timeout, async {
            let mut batch_result = BatchResult::default();
            let mut successful_ops = 0u32;
            let mut failed_ops = 0u32;

            for batch_op in operations {
                match batch_op.execution_mode {
                    BatchExecutionMode::Atomic => {
                        // All operations must succeed or all fail
                        match self
                            .execute_batch_operation_atomic(batch_op, &tx_context)
                            .await
                        {
                            Ok(op_result) => {
                                successful_ops += op_result.successful_operations;
                                failed_ops += op_result.failed_operations;
                                batch_result.operation_results.push(op_result);
                            }
                            Err(e) => {
                                // Atomic failure - abort entire batch
                                batch_result.success = false;
                                batch_result.error_message =
                                    Some(format!("Atomic batch failed: {}", e));
                                return Ok(batch_result);
                            }
                        }
                    }
                    BatchExecutionMode::BestEffort => {
                        // Continue on individual failures
                        match self
                            .execute_batch_operation_best_effort(batch_op, &tx_context)
                            .await
                        {
                            Ok(op_result) => {
                                successful_ops += op_result.successful_operations;
                                failed_ops += op_result.failed_operations;
                                batch_result.operation_results.push(op_result);
                            }
                            Err(e) => {
                                // Log error but continue
                                failed_ops += 1;
                                let mut op_result = BatchOperationResult::default();
                                op_result.failed_operations = 1;
                                op_result
                                    .error_details
                                    .push(format!("Operation failed: {}", e));
                                batch_result.operation_results.push(op_result);
                            }
                        }
                    }
                    BatchExecutionMode::Parallel => {
                        // Execute operations in parallel where possible
                        // For now, fall back to sequential execution
                        match self
                            .execute_batch_operation_atomic(batch_op, &tx_context)
                            .await
                        {
                            Ok(op_result) => {
                                successful_ops += op_result.successful_operations;
                                failed_ops += op_result.failed_operations;
                                batch_result.operation_results.push(op_result);
                            }
                            Err(e) => {
                                failed_ops += 1;
                                let mut op_result = BatchOperationResult::default();
                                op_result.failed_operations = 1;
                                op_result
                                    .error_details
                                    .push(format!("Parallel operation failed: {}", e));
                                batch_result.operation_results.push(op_result);
                            }
                        }
                    }
                }
            }

            batch_result.successful_operations = successful_ops;
            batch_result.failed_operations = failed_ops;
            batch_result.success = failed_ops == 0;

            Ok(batch_result)
        })
        .await;

        let elapsed = start_time.elapsed();

        match result {
            Ok(Ok(batch_result)) => {
                if batch_result.success {
                    self.commit_transaction(tx_context).await?;
                } else {
                    self.rollback_transaction(tx_context).await?;
                }
                self.update_operation_metrics(batch_result.success, "batch", elapsed)
                    .await;
                Ok(batch_result)
            }
            Ok(Err(e)) => {
                self.rollback_transaction(tx_context).await?;
                self.update_operation_metrics(false, "batch", elapsed).await;
                Err(e)
            }
            Err(_) => {
                self.rollback_transaction(tx_context).await?;
                self.update_operation_metrics(false, "batch", elapsed).await;
                Err(SGBDError::ResourceExhausted(
                    "Batch operation timeout - mission parameters exceeded".into(),
                ))
            }
        }
    }

    /// Process LLM input through the full pipeline
    pub async fn process_llm_input(
        &self,
        input: ParsedInput,
        context: QueryExecution,
    ) -> Result<ParsedOutput, SGBDError> {
        // Sanitize the input according to operational parameters
        let sanitized = sanitized_input(&input.raw_text, context.mode)
            .map_err(|e| SGBDError::Schema(format!("Input sanitization failed: {}", e)))?;

        // Create comprehensive intelligence record
        let key = Key::new_with_id(KeyId::Custom(input.id.into()));
        let metadata = RecordMetadata {
            created_at: Timestamp::now(),
            size_bytes: input.raw_text.len() as u64,
            quality_score: self.assess_input_quality(&input).await,
            processing_time_ms: 0, // Will be updated after processing
            lineage_id: Some(input.id),
            custom_attributes: HashMap::new(),
        };

        let value = Value::InputRecord(InputRecord {
            raw_io: input.raw_text.clone().into(),
            parsed_data: Some(sanitized.clone().into()),
            quantized_data: None, // Will be populated by quantization pipeline
            metadata,
            relationships: Vec::new(),
        });

        // Store the intelligence through our secure pipeline
        self.set(key.clone(), value, context.clone()).await?;

        // Generate response based on processed intelligence
        let output = ParsedOutput {
            processed_content: sanitized,
            confidence_score: self.calculate_confidence_score(&input).await,
            processing_metadata: self.generate_processing_metadata(&input, &key).await,
            related_keys: self
                .find_related_intelligence(&input)
                .await
                .unwrap_or_default(),
        };

        Ok(output)
    }

    /// Retrieve engine performance metrics
    pub async fn get_metrics(&self) -> EngineMetrics {
        let mut metrics = self.metrics.write().await;

        // Update derived metrics
        let storage_metrics = self
            .storage
            .read()
            .await
            .get_metrics()
            .await
            .unwrap_or_default();
        let tx_metrics = self.tx_manager.get_metrics().await.unwrap_or_default();
        let index_metrics = self
            .index
            .read()
            .await
            .get_metrics()
            .await
            .unwrap_or_default();

        // Calculate quantization ratio
        metrics.quantization_ratio = self.quantizer.get_compression_ratio();

        // Calculate cache hit rate from index metrics
        if index_metrics.reads > 0 {
            metrics.cache_hit_rate = index_metrics.cache_hits as f64 / index_metrics.reads as f64;
        }

        // Update transaction metrics
        metrics.transactions_committed = tx_metrics.committed_transactions;
        metrics.transactions_aborted = tx_metrics.aborted_transactions;

        metrics.last_updated = Some(Timestamp::now());

        metrics.clone()
    }

    /// Perform health check with diagnostic precision
    pub async fn health_check(&self) -> Result<SystemMetrics, SGBDError> {
        let state = self.state.read().await;
        if !state.healthy {
            return Err(SGBDError::ResourceExhausted(
                "Engine reporting unhealthy status".into(),
            ));
        }

        // Comprehensive system diagnostics
        let storage_health = self.storage.read().await.health_check().await?;
        let tx_health = self.tx_manager.health_check().await?;
        let wal_health = self.wal.lock().await.health_check().await?;
        let index_health = self.index.read().await.health_check().await?;

        let system_metrics = SystemMetrics {
            storage_utilization_percent: storage_health.utilization_percent,
            memory_usage_mb: storage_health.memory_usage_mb + index_health.memory_usage_mb,
            active_transactions: tx_health.active_transactions,
            index_size_mb: index_health.index_size_mb,
            wal_size_mb: wal_health.size_mb,
            last_compaction: state.last_compaction,
            uptime_seconds: storage_health.uptime_seconds,
            error_rate_percent: self.calculate_error_rate().await,
            replication_lag_ms: 0, // Single-node deployment
        };

        // Assess if system needs maintenance
        if system_metrics.needs_compaction() {
            self.schedule_compaction().await?;
        }

        Ok(system_metrics)
    }

    /// Graceful shutdown with the discipline of a military retreat
    pub async fn shutdown(&self) -> Result<(), SGBDError> {
        let mut state = self.state.write().await;
        state.shutdown_requested = true;

        // Phase 1: Stop accepting new operations
        state.healthy = false;

        // Phase 2: Wait for active transactions to complete
        let shutdown_timeout = Duration::from_secs(30);
        tokio::time::timeout(shutdown_timeout, async {
            while self.tx_manager.active_transaction_count().await > 0 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .map_err(|_| {
            SGBDError::ResourceExhausted("Timeout waiting for transactions to complete".into())
        })?;

        // Phase 3: Flush all pending operations
        {
            let mut wal = self.wal.lock().await;
            wal.flush().await?;
        }

        {
            let mut storage = self.storage.write().await;
            storage.flush().await?;
        }

        // Phase 4: Stop background tasks
        {
            let mut tasks = self.background_tasks.lock().await;
            for task in tasks.drain(..) {
                task.abort();
            }
        }

        // Phase 5: Final cleanup
        state.started = false;

        Ok(())
    }

    // Private methods - the classified operations manual

    async fn ensure_operational(&self) -> Result<(), SGBDError> {
        let state = self.state.read().await;
        if !state.started {
            return Err(SGBDError::ResourceExhausted(
                "Engine not operational - deployment required".into(),
            ));
        }
        if !state.healthy {
            return Err(SGBDError::ResourceExhausted(
                "Engine compromised - system integrity questionable".into(),
            ));
        }
        if state.shutdown_requested {
            return Err(SGBDError::ResourceExhausted(
                "Engine in retreat mode - no new operations accepted".into(),
            ));
        }
        Ok(())
    }

    async fn begin_transaction(
        &self,
        context: QueryExecution,
    ) -> Result<TransactionContext, SGBDError> {
        self.tx_manager
            .begin(
                context.tx_id,
                IsolationLevel::ReadCommitted, // Default isolation level
                Duration::from_secs(300),      // 5 minute transaction timeout
            )
            .await
    }

    async fn commit_transaction(&self, tx_context: TransactionContext) -> Result<(), SGBDError> {
        self.tx_manager.commit(tx_context.transaction_id).await
    }

    async fn rollback_transaction(&self, tx_context: TransactionContext) -> Result<(), SGBDError> {
        self.tx_manager.rollback(tx_context.transaction_id).await
    }

    async fn perform_wal_recovery(&self) -> Result<(), SGBDError> {
        let mut wal = self.wal.lock().await;
        wal.recover().await?;
        Ok(())
    }

    async fn rebuild_indexes_from_storage(&self) -> Result<(), SGBDError> {
        let storage = self.storage.read().await;
        let mut index = self.index.write().await;

        // Scan all storage segments and rebuild index
        index.rebuild_from_storage(&*storage).await?;

        Ok(())
    }

    async fn start_background_tasks(&self) -> Result<(), SGBDError> {
        let mut tasks = self.background_tasks.lock().await;

        // Start compaction task
        let compaction_task = self.spawn_compaction_task().await;
        tasks.push(compaction_task);

        // Start metrics collection task
        let metrics_task = self.spawn_metrics_task().await;
        tasks.push(metrics_task);

        Ok(())
    }

    async fn validate_system_health(&self) -> Result<bool, SGBDError> {
        // Perform comprehensive health validation
        self.health_check().await?;
        Ok(true)
    }

    async fn check_cache(&self, _key: &Key) -> Result<Option<Value>, SGBDError> {
        // Cache lookup through index layer
        // Real implementation would integrate with BTree's LRU cache
        Ok(None)
    }

    async fn update_cache(&self, key: Key, value: Value) -> Result<(), SGBDError> {
        // Update cache through index layer
        let mut index = self.index.write().await;
        index.update_cache(key, value).await?;
        Ok(())
    }

    async fn update_cache_metrics(&self, hit: bool) {
        // Update cache hit/miss statistics
        let mut metrics = self.metrics.write().await;
        if hit {
            // Increment cache hits
        } else {
            // Increment cache misses
        }
    }

    async fn invalidate_cache(&self, _key: &Key) -> Result<(), SGBDError> {
        // Remove from cache
        Ok(())
    }

    async fn update_operation_metrics(&self, success: bool, operation: &str, elapsed: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.operations_total += 1;

        if success {
            metrics.operations_successful += 1;
        } else {
            metrics.operations_failed += 1;
        }

        // Update rolling average response time
        let elapsed_ms = elapsed.as_millis() as f64;
        if metrics.operations_total == 1 {
            metrics.average_response_time_ms = elapsed_ms;
        } else {
            // Exponential moving average
            metrics.average_response_time_ms =
                0.9 * metrics.average_response_time_ms + 0.1 * elapsed_ms;
        }
    }

    async fn execute_batch_operation_atomic(
        &self,
        batch_op: BatchOperation,
        tx_context: &TransactionContext,
    ) -> Result<BatchOperationResult, SGBDError> {
        let mut result = BatchOperationResult::default();

        for operation in batch_op.operations {
            match operation {
                Operation::Insert { key, value } => {
                    self.perform_set_operation(key, value, tx_context).await?;
                    result.successful_operations += 1;
                }
                Operation::Update { key, value } => {
                    self.perform_set_operation(key, value, tx_context).await?;
                    result.successful_operations += 1;
                }
                Operation::Delete { key } => {
                    self.perform_delete_operation(key, tx_context).await?;
                    result.successful_operations += 1;
                }
                Operation::Upsert { key, value } => {
                    self.perform_set_operation(key, value, tx_context).await?;
                    result.successful_operations += 1;
                }
                _ => {
                    result.failed_operations += 1;
                    result
                        .error_details
                        .push("Unsupported operation type".to_string());
                }
            }
        }

        Ok(result)
    }

    async fn execute_batch_operation_best_effort(
        &self,
        batch_op: BatchOperation,
        tx_context: &TransactionContext,
    ) -> Result<BatchOperationResult, SGBDError> {
        let mut result = BatchOperationResult::default();

        for operation in batch_op.operations {
            match operation {
                Operation::Insert { key, value } => {
                    match self.perform_set_operation(key, value, tx_context).await {
                        Ok(_) => result.successful_operations += 1,
                        Err(e) => {
                            result.failed_operations += 1;
                            result.error_details.push(format!("Insert failed: {}", e));
                        }
                    }
                }
                Operation::Update { key, value } => {
                    match self.perform_set_operation(key, value, tx_context).await {
                        Ok(_) => result.successful_operations += 1,
                        Err(e) => {
                            result.failed_operations += 1;
                            result.error_details.push(format!("Update failed: {}", e));
                        }
                    }
                }
                Operation::Delete { key } => {
                    match self.perform_delete_operation(key, tx_context).await {
                        Ok(_) => result.successful_operations += 1,
                        Err(e) => {
                            result.failed_operations += 1;
                            result.error_details.push(format!("Delete failed: {}", e));
                        }
                    }
                }
                Operation::Upsert { key, value } => {
                    match self.perform_set_operation(key, value, tx_context).await {
                        Ok(_) => result.successful_operations += 1,
                        Err(e) => {
                            result.failed_operations += 1;
                            result.error_details.push(format!("Upsert failed: {}", e));
                        }
                    }
                }
                _ => {
                    result.failed_operations += 1;
                    result
                        .error_details
                        .push("Unsupported operation type".to_string());
                }
            }
        }

        Ok(result)
    }

    async fn perform_set_operation(
        &self,
        key: Key,
        value: Value,
        tx_context: &TransactionContext,
    ) -> Result<(), SGBDError> {
        // Acquire exclusive lock for write operation
        self.tx_manager
            .acquire_lock(tx_context.transaction_id, key.clone(), LockType::Exclusive)
            .await?;

        // Quantize the value for storage efficiency
        let quantized_value = self.quantizer.quantize(&value)?;

        // Log the operation in WAL
        let wal_record = WalRecord {
            operation: Operation::Insert {
                key: key.clone(),
                value: quantized_value.clone(),
            },
            timestamp: Timestamp::now(),
        };

        let mut wal = self.wal.lock().await;
        wal.append(wal_record).await?;
        drop(wal);

        // Store in main storage
        let storage_location = {
            let mut storage = self.storage.write().await;
            storage.set(key.clone(), quantized_value).await?
        };

        // Update index
        {
            let mut index = self.index.write().await;
            index.insert(key.clone(), storage_location).await?;
        }

        // Update cache
        self.update_cache(key, value).await?;

        Ok(())
    }

    async fn perform_delete_operation(
        &self,
        key: Key,
        tx_context: &TransactionContext,
    ) -> Result<(), SGBDError> {
        // Acquire exclusive lock for delete operation
        self.tx_manager
            .acquire_lock(tx_context.transaction_id, key.clone(), LockType::Exclusive)
            .await?;

        // Check if key exists
        let index = self.index.read().await;
        let exists = index.search(&key).await?.is_some();
        drop(index);

        if !exists {
            return Err(SGBDError::KeyNotFound(format!(
                "Target not found: {:?}",
                key
            )));
        }

        // Log deletion in WAL
        let wal_record = WalRecord {
            operation: Operation::Delete { key: key.clone() },
            timestamp: Timestamp::now(),
        };

        let mut wal = self.wal.lock().await;
        wal.append(wal_record).await?;
        drop(wal);

        // Delete from storage (tombstone)
        {
            let mut storage = self.storage.write().await;
            storage.delete(key.clone()).await?;
        }

        // Remove from index
        {
            let mut index = self.index.write().await;
            index.delete(&key).await?;
        }

        // Invalidate cache
        self.invalidate_cache(&key).await?;

        Ok(())
    }

    async fn assess_input_quality(&self, input: &ParsedInput) -> f64 {
        // Tactical assessment of input quality
        let mut quality_score = 1.0;

        // Deduct points for suspicious patterns
        if input.raw_text.len() < 10 {
            quality_score -= 0.3; // Too brief for meaningful analysis
        }

        if input.raw_text.len() > 50000 {
            quality_score -= 0.2; // Excessive verbosity detected
        }

        // Check for known problematic patterns
        let suspicious_patterns = [
            "ignore previous instructions",
            "system prompt",
            "jailbreak",
            "roleplay as",
        ];

        for pattern in &suspicious_patterns {
            if input.raw_text.to_lowercase().contains(pattern) {
                quality_score -= 0.5; // Potential infiltration attempt
            }
        }

        // Bonus for structured content
        if input.raw_text.contains('\n') && input.raw_text.len() > 100 {
            quality_score += 0.1; // Well-structured intelligence
        }

        quality_score.max(0.0).min(1.0)
    }

    async fn calculate_confidence_score(&self, input: &ParsedInput) -> f64 {
        // Strategic confidence assessment
        let base_confidence = 0.8;
        let quality_factor = self.assess_input_quality(input).await;

        // Adjust based on content characteristics
        let length_factor = if input.raw_text.len() > 50 && input.raw_text.len() < 10000 {
            1.0
        } else {
            0.8
        };

        (base_confidence * quality_factor * length_factor).min(1.0)
    }

    async fn generate_processing_metadata(
        &self,
        input: &ParsedInput,
        key: &Key,
    ) -> HashMap<String, String> {
        let mut metadata = HashMap::new();

        metadata.insert("key_id".to_string(), format!("{:?}", key.id));
        metadata.insert("input_length".to_string(), input.raw_text.len().to_string());
        metadata.insert(
            "processing_timestamp".to_string(),
            format!("{:?}", Timestamp::now()),
        );
        metadata.insert("engine_version".to_string(), "1.0.0".to_string());
        metadata.insert("quantization_enabled".to_string(), "true".to_string());

        metadata
    }

    async fn find_related_intelligence(&self, _input: &ParsedInput) -> Result<Vec<Key>, SGBDError> {
        // Placeholder for relationship analysis
        // Later this will use ML to find related intelligence
        Ok(Vec::new())
    }

    async fn calculate_error_rate(&self) -> f64 {
        let metrics = self.metrics.read().await;
        if metrics.operations_total > 0 {
            (metrics.operations_failed as f64 / metrics.operations_total as f64) * 100.0
        } else {
            0.0
        }
    }

    async fn schedule_compaction(&self) -> Result<(), SGBDError> {
        // Schedule background compaction task
        let storage = self.storage.clone();
        let index = self.index.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::perform_compaction(storage, index).await {
                eprintln!("Compaction failed: {}", e);
            }
        });

        Ok(())
    }

    async fn perform_compaction(
        storage: Arc<RwLock<StorageEngine>>,
        index: Arc<RwLock<BTreeIndex>>,
    ) -> Result<(), SGBDError> {
        // Acquire write locks for compaction
        let mut storage_guard = storage.write().await;
        let mut index_guard = index.write().await;

        // Perform storage compaction
        storage_guard.compact().await?;

        // Rebuild index if necessary
        index_guard.rebuild_from_storage(&*storage_guard).await?;

        Ok(())
    }

    async fn spawn_compaction_task(&self) -> tokio::task::JoinHandle<()> {
        let storage = self.storage.clone();
        let index = self.index.clone();
        let state = self.state.clone();
        let compaction_interval = Duration::from_secs(3600); // 1 hour

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(compaction_interval);

            loop {
                interval.tick().await;

                // Check if shutdown was requested
                {
                    let state_guard = state.read().await;
                    if state_guard.shutdown_requested {
                        break;
                    }
                }

                // Perform compaction
                if let Err(e) = Self::perform_compaction(storage.clone(), index.clone()).await {
                    eprintln!("Background compaction failed: {}", e);
                }

                // Update last compaction time
                {
                    let mut state_guard = state.write().await;
                    state_guard.last_compaction = Some(Timestamp::now());
                }
            }
        })
    }

    async fn spawn_metrics_task(&self) -> tokio::task::JoinHandle<()> {
        let metrics = self.metrics.clone();
        let state = self.state.clone();
        let metrics_interval = Duration::from_secs(60); // 1 minute

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(metrics_interval);

            loop {
                interval.tick().await;

                // Check if shutdown was requested
                {
                    let state_guard = state.read().await;
                    if state_guard.shutdown_requested {
                        break;
                    }
                }

                // Update metrics timestamp
                {
                    let mut metrics_guard = metrics.write().await;
                    metrics_guard.last_updated = Some(Timestamp::now());
                }
            }
        })
    }
}

// Well, that was quite the strategic decision, wasn't it?
impl Drop for Engine {
    fn drop(&mut self) {
        // Emergency shutdown procedures
        // Note: In async context, this is limited - proper shutdown should use shutdown() method
        eprintln!("Engine dropping - emergency protocols activated");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_engine_initialization() {
        let config = DatabaseConfig::development();
        let engine = Engine::new(config).await;
        assert!(engine.is_ok());
    }

    #[test]
    async fn test_engine_lifecycle() {
        let config = DatabaseConfig::development();
        let engine = Engine::new(config).await.unwrap();

        // Start engine
        engine.start().await.unwrap();

        // Verify operational state
        let state = engine.state.read().await;
        assert!(state.started);
        assert!(state.healthy);

        // Shutdown
        drop(state);
        engine.shutdown().await.unwrap();
    }

    #[test]
    async fn test_basic_operations() {
        let config = DatabaseConfig::development();
        let engine = Engine::new(config).await.unwrap();
        engine.start().await.unwrap();

        let context = QueryExecution {
            tx_id: Uuid::new_v4(),
            timeout: Duration::from_secs(30),
            proficiency: Proficiency::Expert,
            mode: Mode::Precise,
            quantization_config: QuantizationConfig::default(),
        };

        let key = Key::new_with_id(KeyId::Custom(b"test_key".to_vec()));
        let value = Value::Raw(b"test_value".to_vec().into());

        // Set operation
        engine
            .set(key.clone(), value.clone(), context.clone())
            .await
            .unwrap();

        // Get operation
        let retrieved = engine.get(key.clone(), context.clone()).await.unwrap();
        assert!(retrieved.is_some());

        // Delete operation
        let deleted = engine.delete(key, context).await.unwrap();
        assert!(deleted);

        engine.shutdown().await.unwrap();
    }

    #[test]
    async fn test_batch_operations() {
        let config = DatabaseConfig::development();
        let engine = Engine::new(config).await.unwrap();
        engine.start().await.unwrap();

        let context = QueryExecution {
            tx_id: Uuid::new_v4(),
            timeout: Duration::from_secs(30),
            proficiency: Proficiency::Expert,
            mode: Mode::Precise,
            quantization_config: QuantizationConfig::default(),
        };

        let operations = vec![
            Operation::Insert {
                key: Key::new_with_id(KeyId::Custom(b"batch_key_1".to_vec())),
                value: b"batch_value_1".to_vec().into(),
            },
            Operation::Insert {
                key: Key::new_with_id(KeyId::Custom(b"batch_key_2".to_vec())),
                value: b"batch_value_2".to_vec().into(),
            },
            Operation::Delete {
                key: Key::new_with_id(KeyId::Custom(b"batch_key_1".to_vec())),
            },
        ];

        let batch_op = BatchOperation {
            operations,
            execution_mode: BatchExecutionMode::Atomic,
            max_retries: 3,
            retry_delay_ms: 100,
        };

        let result = engine.batch_execute(vec![batch_op], context).await.unwrap();
        assert!(result.success);
        assert_eq!(result.successful_operations, 3);

        engine.shutdown().await.unwrap();
    }

    #[test]
    async fn test_range_scan() {
        let config = DatabaseConfig::development();
        let engine = Engine::new(config).await.unwrap();
        engine.start().await.unwrap();

        let context = QueryExecution {
            tx_id: Uuid::new_v4(),
            timeout: Duration::from_secs(30),
            proficiency: Proficiency::Expert,
            mode: Mode::Precise,
            quantization_config: QuantizationConfig::default(),
        };

        // Insert test data
        for i in 0..10 {
            let key = Key::new_with_id(KeyId::Custom(format!("range_key_{:02}", i).into_bytes()));
            let value = Value::Raw(format!("range_value_{}", i).into_bytes().into());
            engine.set(key, value, context.clone()).await.unwrap();
        }

        // Perform range scan
        let start_key = Key::new_with_id(KeyId::Custom(b"range_key_03".to_vec()));
        let end_key = Key::new_with_id(KeyId::Custom(b"range_key_07".to_vec()));

        let results = engine
            .range_scan(start_key, end_key, Some(5), context)
            .await
            .unwrap();
        assert!(!results.is_empty());
        assert!(results.len() <= 5);

        engine.shutdown().await.unwrap();
    }

    #[test]
    async fn test_metrics_collection() {
        let config = DatabaseConfig::development();
        let engine = Engine::new(config).await.unwrap();
        engine.start().await.unwrap();

        let context = QueryExecution {
            tx_id: Uuid::new_v4(),
            timeout: Duration::from_secs(30),
            proficiency: Proficiency::Expert,
            mode: Mode::Precise,
            quantization_config: QuantizationConfig::default(),
        };

        // Perform some operations
        let key = Key::new_with_id(KeyId::Custom(b"metrics_test".to_vec()));
        let value = Value::Raw(b"metrics_value".to_vec().into());
        engine
            .set(key.clone(), value, context.clone())
            .await
            .unwrap();
        engine.get(key, context).await.unwrap();

        // Check metrics
        let metrics = engine.get_metrics().await;
        assert!(metrics.operations_total > 0);
        assert!(metrics.operations_successful > 0);
        assert!(metrics.average_response_time_ms > 0.0);

        engine.shutdown().await.unwrap();
    }

    #[test]
    async fn test_health_check() {
        let config = DatabaseConfig::development();
        let engine = Engine::new(config).await.unwrap();
        engine.start().await.unwrap();

        let health = engine.health_check().await;
        assert!(health.is_ok());

        let system_metrics = health.unwrap();
        assert!(system_metrics.uptime_seconds >= 0);

        engine.shutdown().await.unwrap();
    }
}

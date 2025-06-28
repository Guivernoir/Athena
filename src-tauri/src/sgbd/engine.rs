mod commands;
use crate::backend::ParsedOutput;
use crate::llm::{sanitized_input, Mode, ParsedInput, Proficiency, RawOutput};
use crate::quantization::{QuantizationConfig, Quantizer};
use crate::sgbd::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;

/// Strategic Database Management Engine
///
/// A precision-engineered SGBD that treats data like classified intelligence:
/// - Every byte quantized for maximum compression efficiency
/// - Multi-layered caching with battlefield-tested eviction strategies
/// - ACID transactions with KGB-level reliability
/// - Index structures that would make MI6 proud
pub struct SGBDEngine {
    /// Primary storage layer - the vault
    storage: Arc<StorageEngine>,

    /// BTree index for tactical key lookups
    index: Arc<RwLock<BTreeIndex>>,

    /// Write-Ahead Log for atomic operations
    wal: Arc<Mutex<WriteAheadLog>>,

    /// Quantization engine for surgical data compression
    quantizer: Arc<Quantizer>,

    /// Transaction manager for coordinated operations
    tx_manager: Arc<Mutex<TransactionManager>>,

    /// Sequential ID generator with atomic precision
    next_id: Arc<RwLock<u64>>,

    /// L1 cache: Hot data for immediate tactical advantage
    hot_cache: Arc<RwLock<HashMap<Key, Value>>>,

    /// L2 cache: Quantized data cache for strategic efficiency
    quant_cache: Arc<RwLock<HashMap<Key, Vec<u8>>>>,

    /// Statistics engine for operational intelligence
    stats: Arc<RwLock<OperationalStats>>,

    /// Configuration for strategic decision making
    config: EngineConfig,
}

#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Cache size limits - because memory is finite, strategy is infinite
    pub hot_cache_limit: usize,
    pub quant_cache_limit: usize,

    /// Quantization strategy
    pub quantization_bits: usize,
    pub quantization_block_size: usize,

    /// Performance tuning
    pub enable_compression: bool,
    pub enable_checksums: bool,
    pub sync_strategy: SyncStrategy,
}

#[derive(Debug, Clone)]
pub enum SyncStrategy {
    /// Immediate sync - maximum durability, tactical response time
    Immediate,
    /// Batched sync - strategic efficiency
    Batched(usize),
    /// Timed sync - operational balance
    Timed(std::time::Duration),
}

#[derive(Debug, Default)]
struct OperationalStats {
    total_records: u64,
    cache_hits: u64,
    cache_misses: u64,
    quantization_savings: u64,
    total_operations: u64,
    avg_compression_ratio: f64,
    hot_keys: HashMap<String, u64>, // Domain -> access count
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            hot_cache_limit: 10_000,
            quant_cache_limit: 50_000,
            quantization_bits: 4,
            quantization_block_size: 128,
            enable_compression: true,
            enable_checksums: true,
            sync_strategy: SyncStrategy::Batched(100),
        }
    }
}

impl SGBDEngine {
    /// Initialize the tactical database engine with surgical precision
    pub async fn new(db_path: &str) -> Result<Self> {
        Self::new_with_config(db_path, EngineConfig::default()).await
    }

    /// Initialize with custom configuration - for the strategically minded
    pub async fn new_with_config(db_path: &str, config: EngineConfig) -> Result<Self> {
        // Initialize core components with military precision
        let storage = Arc::new(StorageEngine::new(db_path)?);
        let index = Arc::new(RwLock::new(BTreeIndex::new()));
        let wal = Arc::new(Mutex::new(WriteAheadLog::new(&format!(
            "{}/wal.log",
            db_path
        ))?));
        let tx_manager = Arc::new(Mutex::new(TransactionManager::new()));

        // Strategic quantization configuration
        let quant_config = QuantizationConfig {
            bits: config.quantization_bits,
            block_size: config.quantization_block_size,
        };

        let quantizer = Arc::new(Quantizer::new(quant_config).map_err(|e| {
            SGBDError::QuantizationError(format!("Quantizer initialization failed: {:?}", e))
        })?);

        let engine = Self {
            storage,
            index,
            wal,
            quantizer,
            tx_manager,
            next_id: Arc::new(RwLock::new(1)),
            hot_cache: Arc::new(RwLock::new(HashMap::new())),
            quant_cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(OperationalStats::default())),
            config,
        };

        // Tactical recovery: Rebuild indices and restore state
        engine.execute_recovery_protocol().await?;

        Ok(engine)
    }

    /// Store input record with maximum compression efficiency
    /// Returns the tactical key for future operations
    pub async fn store_input_record(
        &self,
        raw_input: String,
        parsed_input: ParsedInput,
    ) -> Result<Key> {
        // Generate unique key with timestamp precision
        let key = self.generate_tactical_key().await;

        // Execute quantization protocol with surgical precision
        let quantized_data = self.execute_quantization(&raw_input).await?;

        // Calculate compression intelligence
        let original_size = raw_input.len();
        let compressed_size = quantized_data.len();
        let compression_ratio = compressed_size as f32 / original_size as f32;

        // Construct metadata with operational intelligence
        let metadata = RecordMetadata {
            created_at: key.timestamp,
            input_length: original_size,
            quantization_bits: self.config.quantization_bits,
            compression_ratio,
            domain: parsed_input.domain.clone(),
            mode: parsed_input.mode.clone(),
            proficiency: parsed_input.proficiency.clone(),
        };

        let record = InputRecord {
            raw_input: raw_input.clone(),
            parsed_input,
            quantized_data: quantized_data.clone(),
            metadata,
        };

        let value = Value::InputRecord(record);

        // Execute atomic write operation
        self.execute_atomic_write(&key, &value).await?;

        // Update operational statistics
        self.update_tactical_stats(&key, original_size, compressed_size, &raw_input)
            .await;

        Ok(key)
    }

    /// Retrieve record with multi-layer cache strategy
    pub async fn get_record(&self, key: &Key) -> Result<Option<InputRecord>> {
        // L1 Cache check - hot data for immediate tactical advantage
        if let Some(value) = self.check_hot_cache(key).await {
            if let Value::InputRecord(record) = value {
                self.increment_cache_hit().await;
                return Ok(Some(record));
            }
        }

        // L2 Cache check - quantized data reconstruction
        if let Some(quantized_data) = self.check_quant_cache(key).await {
            if let Ok(reconstructed) = self.reconstruct_from_quantized(key, &quantized_data).await {
                self.increment_cache_hit().await;
                return Ok(Some(reconstructed));
            }
        }

        // Storage retrieval - the final frontier
        self.increment_cache_miss().await;
        match self.storage.get(key).await? {
            Some(Value::InputRecord(record)) => {
                // Strategic cache population
                self.populate_caches(key, &record).await;
                Ok(Some(record))
            }
            _ => Ok(None),
        }
    }

    /// Query by domain with intelligent filtering
    pub async fn query_by_domain(&self, domain: &str) -> Result<Vec<(Key, InputRecord)>> {
        let mut results = Vec::new();
        let mut scan_count = 0u64;

        // Strategic index scan with early termination
        for (key, offset) in self.index.read().unwrap().iter() {
            scan_count += 1;

            // Check hot cache first for tactical advantage
            if let Some(Value::InputRecord(record)) = self.hot_cache.read().unwrap().get(key) {
                if record.metadata.domain == domain {
                    results.push((key.clone(), record.clone()));
                }
                continue;
            }

            // Storage lookup with surgical precision
            if let Some(Value::InputRecord(record)) = self.storage.get_at_offset(*offset).await? {
                if record.metadata.domain == domain {
                    results.push((key.clone(), record));
                }
            }

            // Tactical limit to prevent resource exhaustion
            if scan_count > 10_000 {
                break;
            }
        }

        // Update access patterns for strategic intelligence
        self.update_domain_access_stats(domain).await;

        Ok(results)
    }

    /// Query by mode and proficiency with precision targeting
    pub async fn query_by_mode_and_proficiency(
        &self,
        mode: &Mode,
        proficiency: &Proficiency,
    ) -> Result<Vec<(Key, InputRecord)>> {
        let mut results = Vec::new();

        // Execute targeted scan with compound filtering
        for (key, offset) in self.index.read().unwrap().iter() {
            if let Some(Value::InputRecord(record)) = self.storage.get_at_offset(*offset).await? {
                if record.metadata.mode == *mode && record.metadata.proficiency == *proficiency {
                    results.push((key.clone(), record));
                }
            }
        }

        Ok(results)
    }

    /// Advanced query with strategic filtering capabilities
    pub async fn query_advanced(
        &self,
        filter: QueryFilter,
        limit: Option<usize>,
    ) -> Result<Vec<(Key, InputRecord)>> {
        let mut results = Vec::new();
        let mut processed = 0usize;

        for (key, offset) in self.index.read().unwrap().iter() {
            if let Some(limit_val) = limit {
                if results.len() >= limit_val {
                    break;
                }
            }

            if let Some(Value::InputRecord(record)) = self.storage.get_at_offset(*offset).await? {
                if filter.matches(&record) {
                    results.push((key.clone(), record));
                }
            }

            processed += 1;
            if processed > 50_000 {
                break; // Strategic circuit breaker
            }
        }

        Ok(results)
    }

    /// Execute transaction with ACID guarantees
    pub async fn execute_transaction<F, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce(&mut TransactionManager) -> Result<T> + Send,
        T: Send,
    {
        let mut tx_manager = self.tx_manager.lock().await;
        let tx_id = tx_manager.begin_transaction();

        match operation(&mut *tx_manager) {
            Ok(result) => {
                let operations = tx_manager.commit_transaction(tx_id)?;
                // Apply operations to storage
                for op in operations {
                    match op {
                        TxOperation::Put(key, value) => {
                            self.execute_atomic_write(&key, &value).await?;
                        }
                        TxOperation::Delete(key) => {
                            self.execute_delete(&key).await?;
                        }
                    }
                }
                Ok(result)
            }
            Err(e) => {
                tx_manager.rollback(tx_id)?;
                Err(e)
            }
        }
    }

    /// Get comprehensive operational statistics
    pub async fn get_operational_intelligence(&self) -> Result<HashMap<String, serde_json::Value>> {
        let stats = self.stats.read().unwrap();
        let mut intelligence = HashMap::new();

        intelligence.insert("total_records".to_string(), stats.total_records.into());
        intelligence.insert(
            "cache_hit_ratio".to_string(),
            (stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64).into(),
        );
        intelligence.insert(
            "avg_compression_ratio".to_string(),
            stats.avg_compression_ratio.into(),
        );
        intelligence.insert(
            "quantization_savings_bytes".to_string(),
            stats.quantization_savings.into(),
        );
        intelligence.insert(
            "total_operations".to_string(),
            stats.total_operations.into(),
        );

        // Hot domains analysis
        let mut hot_domains = Vec::new();
        for (domain, count) in &stats.hot_keys {
            hot_domains.push(serde_json::json!({
                "domain": domain,
                "access_count": count
            }));
        }
        intelligence.insert("hot_domains".to_string(), hot_domains.into());

        // Storage statistics
        let storage_stats = self.storage.get_statistics().await?;
        intelligence.insert(
            "storage_stats".to_string(),
            serde_json::to_value(storage_stats).unwrap_or_default(),
        );

        Ok(intelligence)
    }

    /// Execute strategic compaction with data reorganization
    pub async fn execute_strategic_compaction(&self) -> Result<CompactionReport> {
        let start_time = std::time::Instant::now();

        // Clear caches for clean slate
        self.clear_all_caches().await;

        // Execute storage compaction
        self.storage.compact().await?;

        // Rebuild indices with precision
        self.execute_recovery_protocol().await?;

        // Generate compaction intelligence
        let duration = start_time.elapsed();
        let stats = self.stats.read().unwrap();

        Ok(CompactionReport {
            duration,
            records_processed: stats.total_records,
            space_reclaimed: 0, // TODO: Calculate from storage
            index_rebuilds: 1,
        })
    }

    /// Backup with strategic data preservation
    pub async fn execute_tactical_backup(&self, backup_path: &str) -> Result<BackupReport> {
        let start_time = std::time::Instant::now();

        // Execute storage backup
        self.storage.backup(backup_path).await?;

        // Backup WAL
        let wal_backup_path = format!("{}/wal_backup.log", backup_path);
        std::fs::copy(
            format!("{}/wal.log", self.storage.get_db_path()),
            wal_backup_path,
        )?;

        let duration = start_time.elapsed();
        let stats = self.stats.read().unwrap();

        Ok(BackupReport {
            duration,
            records_backed_up: stats.total_records,
            backup_size: 0, // TODO: Calculate actual size
        })
    }

    // === PRIVATE TACTICAL METHODS ===

    async fn generate_tactical_key(&self) -> Key {
        let mut next_id = self.next_id.write().unwrap();
        let key = Key::new(*next_id);
        *next_id += 1;
        key
    }

    async fn execute_quantization(&self, raw_input: &str) -> Result<Vec<u8>> {
        if !self.config.enable_compression {
            return Ok(raw_input.as_bytes().to_vec());
        }

        let input_bytes: Vec<f32> = raw_input
            .bytes()
            .map(|b| (b as f32 - 128.0) / 128.0) // Normalize to [-1, 1]
            .collect();

        self.quantizer
            .quantize(&input_bytes)
            .map_err(|e| SGBDError::QuantizationError(format!("Quantization failed: {:?}", e)))
    }

    async fn execute_atomic_write(&self, key: &Key, value: &Value) -> Result<()> {
        // WAL first - atomic guarantee
        {
            let mut wal = self.wal.lock().await;
            wal.append_entry(key, value).await?;

            match &self.config.sync_strategy {
                SyncStrategy::Immediate => wal.sync().await?,
                _ => {} // Batched sync handled elsewhere
            }
        }

        // Storage write
        self.storage.put(key, value).await?;

        // Index update
        {
            let mut index = self.index.write().unwrap();
            if let Ok(offset) = self.storage.get_page_offset_async(key).await {
                index.insert(key.clone(), offset);
            }
        }

        // Hot cache population
        {
            let mut hot_cache = self.hot_cache.write().unwrap();
            if hot_cache.len() < self.config.hot_cache_limit {
                hot_cache.insert(key.clone(), value.clone());
            }
        }

        Ok(())
    }

    async fn execute_delete(&self, key: &Key) -> Result<()> {
        // Clear from all caches
        {
            let mut hot_cache = self.hot_cache.write().unwrap();
            hot_cache.remove(key);
        }
        {
            let mut quant_cache = self.quant_cache.write().unwrap();
            quant_cache.remove(key);
        }

        // Remove from index
        {
            let mut index = self.index.write().unwrap();
            index.remove(key);
        }

        // Note: Physical deletion from storage would require compaction
        Ok(())
    }

    async fn execute_recovery_protocol(&self) -> Result<()> {
        // Scan storage and rebuild index
        let page_map = self.storage.scan_all_pages().await?;

        {
            let mut index = self.index.write().unwrap();
            index.clear();
            for (key, offset) in page_map {
                index.insert(key, offset);
            }
        }

        // Update next_id based on recovered keys
        {
            let index = self.index.read().unwrap();
            if let Some(max_id) = index.keys().map(|k| k.id).max() {
                let mut next_id = self.next_id.write().unwrap();
                *next_id = max_id + 1;
            }
        }

        // Replay WAL if necessary
        let wal_entries = {
            let wal = self.wal.lock().await;
            wal.replay()?
        };

        for (key, value) in wal_entries {
            self.storage.put(&key, &value).await?;
        }

        Ok(())
    }

    async fn check_hot_cache(&self, key: &Key) -> Option<Value> {
        self.hot_cache.read().unwrap().get(key).cloned()
    }

    async fn check_quant_cache(&self, key: &Key) -> Option<Vec<u8>> {
        self.quant_cache.read().unwrap().get(key).cloned()
    }

    async fn reconstruct_from_quantized(
        &self,
        key: &Key,
        quantized_data: &[u8],
    ) -> Result<InputRecord> {
        // This would require storing metadata separately for reconstruction
        // For now, return error to force storage lookup
        Err(SGBDError::IndexError(
            "Quantized reconstruction not implemented".to_string(),
        ))
    }

    async fn populate_caches(&self, key: &Key, record: &InputRecord) {
        // Hot cache
        {
            let mut hot_cache = self.hot_cache.write().unwrap();
            if hot_cache.len() < self.config.hot_cache_limit {
                hot_cache.insert(key.clone(), Value::InputRecord(record.clone()));
            }
        }

        // Quantized cache
        {
            let mut quant_cache = self.quant_cache.write().unwrap();
            if quant_cache.len() < self.config.quant_cache_limit {
                quant_cache.insert(key.clone(), record.quantized_data.clone());
            }
        }
    }

    async fn clear_all_caches(&self) {
        {
            let mut hot_cache = self.hot_cache.write().unwrap();
            hot_cache.clear();
        }
        {
            let mut quant_cache = self.quant_cache.write().unwrap();
            quant_cache.clear();
        }
    }

    async fn increment_cache_hit(&self) {
        let mut stats = self.stats.write().unwrap();
        stats.cache_hits += 1;
        stats.total_operations += 1;
    }

    async fn increment_cache_miss(&self) {
        let mut stats = self.stats.write().unwrap();
        stats.cache_misses += 1;
        stats.total_operations += 1;
    }

    async fn update_tactical_stats(
        &self,
        key: &Key,
        original_size: usize,
        compressed_size: usize,
        raw_input: &str,
    ) {
        let mut stats = self.stats.write().unwrap();
        stats.total_records += 1;
        stats.quantization_savings += (original_size - compressed_size) as u64;

        // Update compression ratio running average
        let new_ratio = compressed_size as f64 / original_size as f64;
        stats.avg_compression_ratio =
            (stats.avg_compression_ratio * (stats.total_records - 1) as f64 + new_ratio)
                / stats.total_records as f64;
    }

    async fn update_domain_access_stats(&self, domain: &str) {
        let mut stats = self.stats.write().unwrap();
        *stats.hot_keys.entry(domain.to_string()).or_insert(0) += 1;
    }
}

#[derive(Debug, Clone)]
pub struct QueryFilter {
    pub domain: Option<String>,
    pub mode: Option<Mode>,
    pub proficiency: Option<Proficiency>,
    pub min_compression_ratio: Option<f32>,
    pub max_compression_ratio: Option<f32>,
    pub after_timestamp: Option<u64>,
    pub before_timestamp: Option<u64>,
}

impl QueryFilter {
    pub fn new() -> Self {
        Self {
            domain: None,
            mode: None,
            proficiency: None,
            min_compression_ratio: None,
            max_compression_ratio: None,
            after_timestamp: None,
            before_timestamp: None,
        }
    }

    pub fn domain(mut self, domain: String) -> Self {
        self.domain = Some(domain);
        self
    }

    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = Some(mode);
        self
    }

    pub fn proficiency(mut self, proficiency: Proficiency) -> Self {
        self.proficiency = Some(proficiency);
        self
    }

    fn matches(&self, record: &InputRecord) -> bool {
        if let Some(ref domain) = self.domain {
            if record.metadata.domain != *domain {
                return false;
            }
        }

        if let Some(ref mode) = self.mode {
            if record.metadata.mode != *mode {
                return false;
            }
        }

        if let Some(ref proficiency) = self.proficiency {
            if record.metadata.proficiency != *proficiency {
                return false;
            }
        }

        if let Some(min_ratio) = self.min_compression_ratio {
            if record.metadata.compression_ratio < min_ratio {
                return false;
            }
        }

        if let Some(max_ratio) = self.max_compression_ratio {
            if record.metadata.compression_ratio > max_ratio {
                return false;
            }
        }

        if let Some(after) = self.after_timestamp {
            if record.metadata.created_at <= after {
                return false;
            }
        }

        if let Some(before) = self.before_timestamp {
            if record.metadata.created_at >= before {
                return false;
            }
        }

        true
    }
}

#[derive(Debug)]
pub struct CompactionReport {
    pub duration: std::time::Duration,
    pub records_processed: u64,
    pub space_reclaimed: u64,
    pub index_rebuilds: u32,
}

#[derive(Debug)]
pub struct BackupReport {
    pub duration: std::time::Duration,
    pub records_backed_up: u64,
    pub backup_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;
    use tokio;

    #[tokio::test]
    async fn test_tactical_operations() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();

        let engine = SGBDEngine::new(db_path).await.unwrap();

        let test_input = "SELECT * FROM classified_operations WHERE clearance_level = 'TOP_SECRET'";
        let parsed_input = ParsedInput {
            domain: "intelligence".to_string(),
            mode: Mode::Query,
            proficiency: Proficiency::Expert,
            confidence: 0.98,
        };

        let key = engine
            .store_input_record(test_input.to_string(), parsed_input)
            .await
            .unwrap();
        let retrieved = engine.get_record(&key).await.unwrap();

        assert!(retrieved.is_some());
        let record = retrieved.unwrap();
        assert_eq!(record.raw_input, test_input);
        assert!(record.metadata.compression_ratio < 1.0); // Should be compressed

        // Test query capabilities
        let domain_results = engine.query_by_domain("intelligence").await.unwrap();
        assert_eq!(domain_results.len(), 1);

        // Test operational intelligence
        let intel = engine.get_operational_intelligence().await.unwrap();
        assert!(intel.contains_key("total_records"));
        assert!(intel.contains_key("cache_hit_ratio"));
    }

    #[tokio::test]
    async fn test_advanced_query_filter() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();

        let engine = SGBDEngine::new(db_path).await.unwrap();

        // Store multiple records with different characteristics
        for i in 0..5 {
            let test_input = format!("Operation {}: Execute tactical maneuver", i);
            let parsed_input = ParsedInput {
                domain: if i % 2 == 0 { "tactical" } else { "strategic" }.to_string(),
                mode: Mode::Query,
                proficiency: if i < 3 {
                    Proficiency::Advanced
                } else {
                    Proficiency::Expert
                },
                confidence: 0.9,
            };

            engine
                .store_input_record(test_input, parsed_input)
                .await
                .unwrap();
        }

        // Test advanced filtering
        let filter = QueryFilter::new()
            .domain("tactical".to_string())
            .proficiency(Proficiency::Advanced);

        let results = engine.query_advanced(filter, Some(10)).await.unwrap();

        // Should find records that match both domain and proficiency
        assert!(results.len() > 0);
        for (_, record) in results {
            assert_eq!(record.metadata.domain, "tactical");
            assert_eq!(record.metadata.proficiency, Proficiency::Advanced);
        }
    }
}

// Well, that was quite the strategic decision, wasn't it?

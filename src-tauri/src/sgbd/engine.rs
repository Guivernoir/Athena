use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;
use crate::quantization::{Quantizer, QuantizationConfig};
use crate::storage::StorageEngine;
use crate::btree::BTreeIndex;
use crate::wal::WriteAheadLog;
use crate::types::{Key, Value, InputRecord, RecordMetadata, SGBDError, Result};
use crate::llm::{ParsedInput, Mode, Proficiency};

pub struct SGBDEngine {
    storage: Arc<StorageEngine>,
    index: Arc<RwLock<BTreeIndex>>,
    wal: Arc<Mutex<WriteAheadLog>>,
    quantizer: Arc<Quantizer>,
    next_id: Arc<RwLock<u64>>,
    cache: Arc<RwLock<HashMap<Key, Value>>>,
}

impl SGBDEngine {
    pub async fn new(db_path: &str) -> Result<Self> {
        let storage = Arc::new(StorageEngine::new(db_path)?);
        let index = Arc::new(RwLock::new(BTreeIndex::new()));
        let wal = Arc::new(Mutex::new(WriteAheadLog::new(&format!("{}/wal.log", db_path))?));
        
        // Initialize quantizer with strategic 4-bit quantization
        let quant_config = QuantizationConfig {
            bits: 4,
            block_size: 128,
        };
        let quantizer = Arc::new(
            Quantizer::new(quant_config)
                .map_err(|e| SGBDError::QuantizationError(format!("{:?}", e)))?
        );
        
        let engine = Self {
            storage,
            index,
            wal,
            quantizer,
            next_id: Arc::new(RwLock::new(1)),
            cache: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Load existing index from storage - tactical recovery
        engine.recover_index().await?;
        
        Ok(engine)
    }
    
    pub async fn store_input_record(
        &self,
        raw_input: String,
        parsed_input: ParsedInput,
    ) -> Result<Key> {
        // Generate unique key with current timestamp
        let key = {
            let mut next_id = self.next_id.write().unwrap();
            let key = Key::new(*next_id);
            *next_id += 1;
            key
        };
        
        // Quantize the raw input - converting text to embeddings first would be more strategic,
        // but for now we'll quantize the byte representation
        let input_bytes: Vec<f32> = raw_input.bytes().map(|b| b as f32 / 255.0).collect();
        let quantized_data = self.quantizer.quantize(&input_bytes)
            .map_err(|e| SGBDError::QuantizationError(format!("{:?}", e)))?;
        
        let compression_ratio = quantized_data.len() as f32 / input_bytes.len() as f32;
        
        let metadata = RecordMetadata {
            created_at: key.timestamp,
            input_length: raw_input.len(),
            quantization_bits: 4,
            compression_ratio,
            domain: parsed_input.domain.clone(),
            mode: parsed_input.mode.clone(),
            proficiency: parsed_input.proficiency.clone(),
        };
        
        let record = InputRecord {
            raw_input,
            parsed_input,
            quantized_data,
            metadata,
        };
        
        let value = Value::InputRecord(record);
        
        // Write to WAL first - atomic guarantee
        {
            let mut wal = self.wal.lock().await;
            wal.append_entry(&key, &value).await?;
            wal.sync().await?;
        }
        
        // Store in main storage
        self.storage.put(&key, &value).await?;
        
        // Update index
        {
            let mut index = self.index.write().unwrap();
            index.insert(key.clone(), self.storage.get_page_offset(&key)?);
        }
        
        // Cache the record for quick access
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(key.clone(), value);
        }
        
        Ok(key)
    }
    
    pub async fn get_record(&self, key: &Key) -> Result<Option<InputRecord>> {
        // Check cache first - tactical optimization
        {
            let cache = self.cache.read().unwrap();
            if let Some(value) = cache.get(key) {
                if let Value::InputRecord(record) = value {
                    return Ok(Some(record.clone()));
                }
            }
        }
        
        // Check main storage
        match self.storage.get(key).await? {
            Some(Value::InputRecord(record)) => {
                // Update cache
                {
                    let mut cache = self.cache.write().unwrap();
                    cache.insert(key.clone(), Value::InputRecord(record.clone()));
                }
                Ok(Some(record))
            }
            _ => Ok(None),
        }
    }
    
    pub async fn query_by_domain(&self, domain: &str) -> Result<Vec<(Key, InputRecord)>> {
        let mut results = Vec::new();
        
        // This is a full scan - in production, we'd want domain-specific indices
        for (key, offset) in self.index.read().unwrap().iter() {
            if let Some(Value::InputRecord(record)) = self.storage.get_at_offset(*offset).await? {
                if record.metadata.domain == domain {
                    results.push((key.clone(), record));
                }
            }
        }
        
        Ok(results)
    }
    
    pub async fn query_by_mode_and_proficiency(
        &self,
        mode: &Mode,
        proficiency: &Proficiency,
    ) -> Result<Vec<(Key, InputRecord)>> {
        let mut results = Vec::new();
        
        for (key, offset) in self.index.read().unwrap().iter() {
            if let Some(Value::InputRecord(record)) = self.storage.get_at_offset(*offset).await? {
                if record.metadata.mode == *mode && record.metadata.proficiency == *proficiency {
                    results.push((key.clone(), record));
                }
            }
        }
        
        Ok(results)
    }
    
    pub async fn get_quantized_data(&self, key: &Key) -> Result<Option<Vec<u8>>> {
        if let Some(record) = self.get_record(key).await? {
            Ok(Some(record.quantized_data))
        } else {
            Ok(None)
        }
    }
    
    pub async fn dequantize_record(&self, key: &Key) -> Result<Option<Vec<f32>>> {
        if let Some(quantized_data) = self.get_quantized_data(key).await? {
            let dequantized = self.quantizer.dequantize(&quantized_data)
                .map_err(|e| SGBDError::QuantizationError(format!("{:?}", e)))?;
            Ok(Some(dequantized))
        } else {
            Ok(None)
        }
    }
    
    pub async fn get_statistics(&self) -> Result<HashMap<String, u64>> {
        let mut stats = HashMap::new();
        let mut domain_counts = HashMap::new();
        let mut mode_counts = HashMap::new();
        let mut total_records = 0u64;
        let mut total_compressed_size = 0u64;
        let mut total_original_size = 0u64;
        
        for (_, offset) in self.index.read().unwrap().iter() {
            if let Some(Value::InputRecord(record)) = self.storage.get_at_offset(*offset).await? {
                total_records += 1;
                total_compressed_size += record.quantized_data.len() as u64;
                total_original_size += record.metadata.input_length as u64;
                
                *domain_counts.entry(record.metadata.domain.clone()).or_insert(0) += 1;
                *mode_counts.entry(format!("{:?}", record.metadata.mode)).or_insert(0) += 1;
            }
        }
        
        stats.insert("total_records".to_string(), total_records);
        stats.insert("total_compressed_size".to_string(), total_compressed_size);
        stats.insert("total_original_size".to_string(), total_original_size);
        
        if total_original_size > 0 {
            let compression_ratio = (total_compressed_size as f64 / total_original_size as f64 * 100.0) as u64;
            stats.insert("compression_percentage".to_string(), compression_ratio);
        }
        
        Ok(stats)
    }
    
    async fn recover_index(&self) -> Result<()> {
        // Scan storage and rebuild index - battlefield recovery protocol
        let page_map = self.storage.scan_all_pages().await?;
        
        {
            let mut index = self.index.write().unwrap();
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
        
        Ok(())
    }
    
    pub async fn compact(&self) -> Result<()> {
        // Strategic compaction - remove fragmentation
        self.storage.compact().await?;
        self.recover_index().await?;
        Ok(())
    }
    
    pub async fn backup(&self, backup_path: &str) -> Result<()> {
        self.storage.backup(backup_path).await
    }
}
use crate::sgbd::{
    Bytes, CompressionType, DatabaseConfig, Key, Result, SGBDError, SerializationFormat,
    SerializationStrategy, StorageBackend, StorageError, Timestamp, Value, Wal,
};
use bytes::{Buf, BufMut, BytesMut};
use crc32fast::Hasher as Crc32;
use lz4::EncoderBuilder;
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{Mutex, Notify, RwLock},
    time,
};
use uuid::Uuid;
use zstd::stream::decode_all;

/// Storage engine configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub max_segment_size: u64,
    pub max_value_size: usize,
    pub compression: CompressionType,
    pub serialization_format: SerializationFormat,
    pub enable_checksums: bool,
    pub buffer_pool_size: usize,
    pub compaction_threshold: f32,
    pub background_compaction_interval: Duration,
    pub base_path: PathBuf,
}

/// Storage engine metrics
#[derive(Debug, Default)]
pub struct StorageMetrics {
    read_count: AtomicU64,
    write_count: AtomicU64,
    bytes_read: AtomicU64,
    bytes_written: AtomicU64,
    compaction_count: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    error_count: AtomicU64,
    active_segments: AtomicUsize,
}

/// Storage metrics snapshot
pub struct StorageMetricsSnapshot {
    pub read_count: u64,
    pub write_count: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub compaction_count: u64,
    pub cache_hit_ratio: f64,
    pub error_count: u64,
    pub active_segments: usize,
}

/// Segment file metadata
struct Segment {
    id: Uuid,
    path: PathBuf,
    file: Mutex<BufWriter<File>>,
    size: AtomicU64,
    max_size: u64,
    start_offset: u64,
}

/// Storage engine state
struct StorageState {
    segments: Vec<Arc<Segment>>,
    active_segment: Arc<Segment>,
    offset_index: RwLock<HashMap<Key, (Uuid, u64)>>, // (segment_id, offset)
    config: StorageConfig,
    metrics: StorageMetrics,
}

/// Storage engine
#[derive(Clone)]
pub struct StorageEngine {
    state: Arc<RwLock<StorageState>>,
    wal: Option<Arc<Wal>>,
    shutdown: Arc<Notify>,
}

// Record header format (fixed size)
const HEADER_SIZE: usize = 32;
struct RecordHeader {
    key_size: u32,
    value_size: u32,
    timestamp: u64,
    flags: u8,
    compression: u8,
    checksum: u32,
    reserved: [u8; 10],
}

impl StorageEngine {
    /// Create a new storage engine
    pub async fn new(config: StorageConfig, wal: Option<Arc<Wal>>) -> Result<Self> {
        fs::create_dir_all(&config.base_path).map_err(|e| SGBDError::Storage {
            operation: format!("create base directory: {}", e),
            key: None,
            backtrace: Some(std::backtrace::Backtrace::capture()),
        })?;

        let (active_segment, segments) = Self::load_segments(&config).await?;
        let offset_index = Self::build_index(&segments, &config).await?;

        let state = StorageState {
            segments,
            active_segment: Arc::clone(&active_segment),
            offset_index: RwLock::new(offset_index),
            config,
            metrics: StorageMetrics::default(),
        };

        let engine = Self {
            state: Arc::new(RwLock::new(state)),
            wal,
            shutdown: Arc::new(Notify::new()),
        };

        // Start background compactor
        tokio::spawn(engine.clone().compaction_task());

        Ok(engine)
    }

    /// Write a key-value pair
    pub async fn write(&self, key: Key, value: Value) -> Result<()> {
        let start = Instant::now();
        let mut state = self.state.write().await;

        // Check value size
        let value_size = value.size();
        if value_size > state.config.max_value_size {
            return Err(SGBDError::ResourceExhausted {
                resource: format!("value size (max {} bytes)", state.config.max_value_size),
                backtrace: Some(std::backtrace::Backtrace::capture()),
            });
        }

        // Write to WAL first if enabled
        if let Some(wal) = &self.wal {
            wal.append(crate::types::Operation::Insert {
                key: key.clone(),
                value: value.clone(),
            })
            .await
            .map_err(|e| SGBDError::Storage {
                operation: format!("WAL append: {}", e),
                key: Some(key.clone()),
                backtrace: Some(std::backtrace::Backtrace::capture()),
            })?;
        }

        // Serialize key and value
        let serialized_key = key.to_bytes().map_err(|e| SGBDError::Storage {
            operation: format!("key serialization: {}", e),
            key: Some(key.clone()),
            backtrace: Some(std::backtrace::Backtrace::capture()),
        })?;

        let serialized_value = value.to_bytes().map_err(|e| SGBDError::Storage {
            operation: format!("value serialization: {}", e),
            key: Some(key.clone()),
            backtrace: Some(std::backtrace::Backtrace::capture()),
        })?;

        // Compress if configured
        let (compressed_value, compression_type) =
            if state.config.compression != CompressionType::None {
                let compressed = Self::compress(&serialized_value, state.config.compression)
                    .map_err(|e| SGBDError::Storage {
                        operation: format!("compression: {}", e),
                        key: Some(key.clone()),
                        backtrace: Some(std::backtrace::Backtrace::capture()),
                    })?;
                (compressed, state.config.compression)
            } else {
                (serialized_value, CompressionType::None)
            };

        // Prepare record
        let record = StorageRecord {
            key: serialized_key,
            value: compressed_value,
            compression: compression_type,
            timestamp: Timestamp::now(),
            deleted: false,
        };

        // Write to active segment
        let offset =
            self.write_record(&mut state, &record)
                .await
                .map_err(|e| SGBDError::Storage {
                    operation: format!("write record: {}", e),
                    key: Some(key.clone()),
                    backtrace: Some(std::backtrace::Backtrace::capture()),
                })?;

        // Update index
        state
            .offset_index
            .write()
            .await
            .insert(key, (state.active_segment.id, offset));

        // Update metrics
        state.metrics.write_count.fetch_add(1, Ordering::Relaxed);
        state
            .metrics
            .bytes_written
            .fetch_add(record.size() as u64, Ordering::Relaxed);

        Ok(())
    }

    /// Read a value by key
    pub async fn read(&self, key: &Key) -> Result<Option<Value>> {
        let start = Instant::now();
        let state = self.state.read().await;

        // Lookup in index
        let offset_index = state.offset_index.read().await;
        let Some((segment_id, offset)) = offset_index.get(key) else {
            state.metrics.cache_misses.fetch_add(1, Ordering::Relaxed);
            return Ok(None);
        };

        // Find segment
        let segment = state
            .segments
            .iter()
            .find(|s| s.id == *segment_id)
            .ok_or_else(|| SGBDError::Storage {
                operation: format!("segment lookup for id {}", segment_id),
                key: Some(key.clone()),
                backtrace: Some(std::backtrace::Backtrace::capture()),
            })?;

        // Read record
        let record = self
            .read_record(segment, *offset)
            .await
            .map_err(|e| SGBDError::Storage {
                operation: format!("read record: {}", e),
                key: Some(key.clone()),
                backtrace: Some(std::backtrace::Backtrace::capture()),
            })?;

        // Handle deleted marker
        if record.deleted {
            return Ok(None);
        }

        // Decompress if needed
        let value_bytes = if record.compression != CompressionType::None {
            Self::decompress(&record.value, record.compression).map_err(|e| SGBDError::Storage {
                operation: format!("decompression: {}", e),
                key: Some(key.clone()),
                backtrace: Some(std::backtrace::Backtrace::capture()),
            })?
        } else {
            record.value
        };

        // Deserialize value
        let value = Value::from_bytes(&value_bytes).map_err(|e| SGBDError::Storage {
            operation: format!("value deserialization: {}", e),
            key: Some(key.clone()),
            backtrace: Some(std::backtrace::Backtrace::capture()),
        })?;

        // Update metrics
        state.metrics.read_count.fetch_add(1, Ordering::Relaxed);
        state
            .metrics
            .bytes_read
            .fetch_add(record.size() as u64, Ordering::Relaxed);
        state.metrics.cache_hits.fetch_add(1, Ordering::Relaxed);

        Ok(Some(value))
    }

    /// Delete a key
    pub async fn delete(&self, key: &Key) -> Result<()> {
        let start = Instant::now();
        let mut state = self.state.write().await;

        // Write to WAL first if enabled
        if let Some(wal) = &self.wal {
            wal.append(crate::types::Operation::Delete { key: key.clone() })
                .await
                .map_err(|e| SGBDError::Storage {
                    operation: format!("WAL append: {}", e),
                    key: Some(key.clone()),
                    backtrace: Some(std::backtrace::Backtrace::capture()),
                })?;
        }

        // Write tombstone record
        let record = StorageRecord {
            key: key.to_bytes().map_err(|e| SGBDError::Storage {
                operation: format!("key serialization: {}", e),
                key: Some(key.clone()),
                backtrace: Some(std::backtrace::Backtrace::capture()),
            })?,
            value: Bytes::new(),
            compression: CompressionType::None,
            timestamp: Timestamp::now(),
            deleted: true,
        };

        // Write to active segment
        self.write_record(&mut state, &record)
            .await
            .map_err(|e| SGBDError::Storage {
                operation: format!("write record: {}", e),
                key: Some(key.clone()),
                backtrace: Some(std::backtrace::Backtrace::capture()),
            })?;

        // Update index
        state.offset_index.write().await.remove(key);

        // Update metrics
        state.metrics.write_count.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Write record to storage
    async fn write_record(&self, state: &mut StorageState, record: &StorageRecord) -> Result<u64> {
        let mut segment = state.active_segment.file.lock().await;
        let offset = segment.stream_position().map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("get stream position: {}", e))
        })?;

        // Check if we need to rotate segment
        if offset + record.size() as u64 > state.active_segment.max_size {
            self.rotate_segment(state).await?;
            segment = state.active_segment.file.lock().await;
        }

        // Serialize record with header
        let mut buffer = BytesMut::with_capacity(record.size());
        record.serialize(&mut buffer);

        // Write to file
        segment
            .write_all(&buffer)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("write record: {}", e)))?;

        segment
            .flush()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("flush segment: {}", e)))?;

        // Update segment size
        state
            .active_segment
            .size
            .fetch_add(buffer.len() as u64, Ordering::Relaxed);

        Ok(offset)
    }

    /// Read record from storage
    async fn read_record(&self, segment: &Segment, offset: u64) -> Result<StorageRecord> {
        let mut file = segment.file.lock().await;
        file.seek(SeekFrom::Start(offset)).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("seek to offset {}: {}", offset, e),
            )
        })?;

        // Read header
        let mut header = [0u8; HEADER_SIZE];
        file.read_exact(&mut header)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("read header: {}", e)))?;

        // Parse header
        let key_size = u32::from_be_bytes(header[0..4].try_into().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "invalid key size in header")
        })?) as usize;

        let value_size = u32::from_be_bytes(header[4..8].try_into().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "invalid value size in header")
        })?) as usize;

        let timestamp = u64::from_be_bytes(header[8..16].try_into().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "invalid timestamp in header")
        })?);

        let flags = header[16];
        let compression = CompressionType::from_u8(header[17]).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid compression type in header",
            )
        })?;

        let checksum = u32::from_be_bytes(header[18..22].try_into().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "invalid checksum in header")
        })?);

        let _reserved = &header[22..HEADER_SIZE];

        // Read key and value
        let mut key_buf = vec![0u8; key_size];
        file.read_exact(&mut key_buf)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("read key: {}", e)))?;

        let mut value_buf = vec![0u8; value_size];
        file.read_exact(&mut value_buf)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("read value: {}", e)))?;

        // Verify checksum
        if state.config.enable_checksums {
            let mut hasher = Crc32::new();
            hasher.update(&key_buf);
            hasher.update(&value_buf);
            if hasher.finalize() != checksum {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "checksum mismatch").into());
            }
        }

        Ok(StorageRecord {
            key: Bytes::from(key_buf),
            value: Bytes::from(value_buf),
            compression,
            timestamp: Timestamp::from_u64(timestamp),
            deleted: flags & 0x01 != 0,
        })
    }

    /// Rotate to a new segment
    async fn rotate_segment(&self, state: &mut StorageState) -> Result<()> {
        let new_segment = Self::create_segment(&state.config).await?;

        // Flush current segment
        let mut current = state.active_segment.file.lock().await;
        current
            .flush()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("flush segment: {}", e)))?;

        // Add to segments list
        state.segments.push(Arc::clone(&state.active_segment));

        // Set new active segment
        state.active_segment = new_segment;

        // Update metrics
        state
            .metrics
            .active_segments
            .store(state.segments.len() + 1, Ordering::Relaxed);

        Ok(())
    }

    /// Background compaction task
    async fn compaction_task(self) {
        let interval = {
            let state = self.state.read().await;
            state.config.background_compaction_interval
        };

        loop {
            tokio::select! {
                _ = time::sleep(interval) => {
                    if let Err(e) = self.compact().await {
                        log::error!("Compaction failed: {}", e);
                    }
                }
                _ = self.shutdown.notified() => break,
            }
        }
    }

    /// Perform storage compaction
    async fn compact(&self) -> Result<()> {
        let start = Instant::now();
        let state = self.state.read().await;

        // Check if compaction is needed
        let total_size: u64 = state
            .segments
            .iter()
            .map(|s| s.size.load(Ordering::Relaxed))
            .sum();
        let active_size = state.active_segment.size.load(Ordering::Relaxed);
        let total_bytes = total_size + active_size;
        let utilization = if total_bytes > 0 {
            total_size as f64 / total_bytes as f64
        } else {
            0.0
        };

        if utilization > state.config.compaction_threshold as f64 {
            log::debug!(
                "Skipping compaction, utilization {:.2} below threshold {:.2}",
                utilization,
                state.config.compaction_threshold
            );
            return Ok(());
        }

        log::info!(
            "Starting compaction (utilization: {:.2} > threshold: {:.2})",
            utilization,
            state.config.compaction_threshold
        );

        // Create new segment for compacted data
        let new_segment = Self::create_segment(&state.config).await?;
        let mut new_file = new_segment.file.lock().await;

        // Build new index
        let mut new_index = HashMap::new();
        let mut new_offset = 0;

        // Iterate through all keys in current index
        let current_index = state.offset_index.read().await.clone();
        for (key, (segment_id, offset)) in current_index.iter() {
            // Find segment
            let segment = state
                .segments
                .iter()
                .find(|s| s.id == *segment_id)
                .ok_or_else(|| SGBDError::Storage {
                    operation: format!("segment lookup for compaction: {}", segment_id),
                    key: Some(key.clone()),
                    backtrace: Some(std::backtrace::Backtrace::capture()),
                })?;

            // Read record
            let record =
                self.read_record(segment, *offset)
                    .await
                    .map_err(|e| SGBDError::Storage {
                        operation: format!("read record for compaction: {}", e),
                        key: Some(key.clone()),
                        backtrace: Some(std::backtrace::Backtrace::capture()),
                    })?;

            // Skip deleted records
            if record.deleted {
                continue;
            }

            // Serialize record
            let mut buffer = BytesMut::with_capacity(record.size());
            record.serialize(&mut buffer);

            // Write to new segment
            new_file
                .write_all(&buffer)
                .map_err(|e| SGBDError::Storage {
                    operation: format!("write during compaction: {}", e),
                    key: Some(key.clone()),
                    backtrace: Some(std::backtrace::Backtrace::capture()),
                })?;

            // Update index
            new_index.insert(key.clone(), (new_segment.id, new_offset));

            // Update offset
            new_offset += buffer.len() as u64;
        }

        // Flush new segment
        new_file.flush().map_err(|e| SGBDError::Storage {
            operation: format!("flush compacted segment: {}", e),
            key: None,
            backtrace: Some(std::backtrace::Backtrace::capture()),
        })?;

        new_segment.size.store(new_offset, Ordering::Relaxed);

        // Switch to new segment
        let mut state = self.state.write().await;
        state.segments = vec![Arc::new(new_segment)];
        *state.offset_index.write().await = new_index;

        // Update metrics
        state
            .metrics
            .compaction_count
            .fetch_add(1, Ordering::Relaxed);
        state.metrics.active_segments.store(1, Ordering::Relaxed);

        log::info!(
            "Compaction completed in {:?}, new size: {} bytes",
            start.elapsed(),
            new_offset
        );
        Ok(())
    }

    /// Get storage metrics
    pub async fn metrics(&self) -> StorageMetricsSnapshot {
        let state = self.state.read().await;
        let cache_hits = state.metrics.cache_hits.load(Ordering::Relaxed);
        let cache_misses = state.metrics.cache_misses.load(Ordering::Relaxed);
        let total_cache = cache_hits + cache_misses;

        StorageMetricsSnapshot {
            read_count: state.metrics.read_count.load(Ordering::Relaxed),
            write_count: state.metrics.write_count.load(Ordering::Relaxed),
            bytes_read: state.metrics.bytes_read.load(Ordering::Relaxed),
            bytes_written: state.metrics.bytes_written.load(Ordering::Relaxed),
            compaction_count: state.metrics.compaction_count.load(Ordering::Relaxed),
            cache_hit_ratio: if total_cache > 0 {
                cache_hits as f64 / total_cache as f64
            } else {
                0.0
            },
            error_count: state.metrics.error_count.load(Ordering::Relaxed),
            active_segments: state.metrics.active_segments.load(Ordering::Relaxed),
        }
    }

    /// Gracefully shutdown storage engine
    pub async fn shutdown(&self) {
        self.shutdown.notify_one();

        // Flush active segment
        let state = self.state.read().await;
        let mut file = state.active_segment.file.lock().await;
        if let Err(e) = file.flush() {
            log::error!("Failed to flush storage during shutdown: {}", e);
        }
    }

    // Helper functions
    async fn load_segments(config: &StorageConfig) -> Result<(Arc<Segment>, Vec<Arc<Segment>>)> {
        let mut segments = Vec::new();
        let mut latest_segment: Option<Arc<Segment>> = None;
        let mut max_offset = 0;

        // List segment files
        for entry in fs::read_dir(&config.base_path).map_err(|e| SGBDError::Storage {
            operation: format!("read storage directory: {}", e),
            key: None,
            backtrace: Some(std::backtrace::Backtrace::capture()),
        })? {
            let path = entry
                .map_err(|e| SGBDError::Storage {
                    operation: format!("read directory entry: {}", e),
                    key: None,
                    backtrace: Some(std::backtrace::Backtrace::capture()),
                })?
                .path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "seg") {
                let file = OpenOptions::new()
                    .read(true)
                    .append(true)
                    .open(&path)
                    .map_err(|e| SGBDError::Storage {
                        operation: format!("open segment file: {}", e),
                        key: None,
                        backtrace: Some(std::backtrace::Backtrace::capture()),
                    })?;

                let size = file
                    .metadata()
                    .map_err(|e| SGBDError::Storage {
                        operation: format!("get segment metadata: {}", e),
                        key: None,
                        backtrace: Some(std::backtrace::Backtrace::capture()),
                    })?
                    .len();

                let id = Uuid::parse_str(path.file_stem().and_then(|s| s.to_str()).ok_or_else(
                    || SGBDError::Storage {
                        operation: "invalid segment filename".to_string(),
                        key: None,
                        backtrace: Some(std::backtrace::Backtrace::capture()),
                    },
                )?)
                .map_err(|e| SGBDError::Storage {
                    operation: format!("parse segment UUID: {}", e),
                    key: None,
                    backtrace: Some(std::backtrace::Backtrace::capture()),
                })?;

                let segment = Arc::new(Segment {
                    id,
                    path: path.clone(),
                    file: Mutex::new(BufWriter::new(file)),
                    size: AtomicU64::new(size),
                    max_size: config.max_segment_size,
                    start_offset: max_offset,
                });

                segments.push(Arc::clone(&segment));
                max_offset += size;

                // Track the latest segment (highest offset)
                if let Some(current) = &latest_segment {
                    if segment.start_offset > current.start_offset {
                        latest_segment = Some(Arc::clone(&segment));
                    }
                } else {
                    latest_segment = Some(Arc::clone(&segment));
                }
            }
        }

        // If no segments found, create a new one
        let active_segment = match latest_segment {
            Some(seg) => seg,
            None => Self::create_segment(config).await?,
        };

        Ok((active_segment, segments))
    }

    async fn create_segment(config: &StorageConfig) -> Result<Arc<Segment>> {
        let id = Uuid::new_v4();
        let path = config.base_path.join(format!("{}.seg", id));
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&path)
            .map_err(|e| SGBDError::Storage {
                operation: format!("create segment file: {}", e),
                key: None,
                backtrace: Some(std::backtrace::Backtrace::capture()),
            })?;

        Ok(Arc::new(Segment {
            id,
            path,
            file: Mutex::new(BufWriter::new(file)),
            size: AtomicU64::new(0),
            max_size: config.max_segment_size,
            start_offset: 0,
        }))
    }

    async fn build_index(
        segments: &[Arc<Segment>],
        config: &StorageConfig,
    ) -> Result<HashMap<Key, (Uuid, u64)>> {
        let mut index = HashMap::new();

        for segment in segments {
            let mut offset = 0;
            let file_size = segment.size.load(Ordering::Relaxed);

            while offset < file_size {
                let mut file = segment.file.lock().await;
                file.seek(SeekFrom::Start(offset))
                    .map_err(|e| SGBDError::Storage {
                        operation: format!("seek during index build: {}", e),
                        key: None,
                        backtrace: Some(std::backtrace::Backtrace::capture()),
                    })?;

                // Read header
                let mut header = [0u8; HEADER_SIZE];
                file.read_exact(&mut header)
                    .map_err(|e| SGBDError::Storage {
                        operation: format!("read header during index build: {}", e),
                        key: None,
                        backtrace: Some(std::backtrace::Backtrace::capture()),
                    })?;

                let key_size =
                    u32::from_be_bytes(header[0..4].try_into().map_err(|_| SGBDError::Storage {
                        operation: "invalid key size in header".to_string(),
                        key: None,
                        backtrace: Some(std::backtrace::Backtrace::capture()),
                    })?) as usize;

                let value_size =
                    u32::from_be_bytes(header[4..8].try_into().map_err(|_| SGBDError::Storage {
                        operation: "invalid value size in header".to_string(),
                        key: None,
                        backtrace: Some(std::backtrace::Backtrace::capture()),
                    })?) as usize;

                let flags = header[16];
                let deleted = flags & 0x01 != 0;

                // Read key
                let mut key_buf = vec![0u8; key_size];
                file.read_exact(&mut key_buf)
                    .map_err(|e| SGBDError::Storage {
                        operation: format!("read key during index build: {}", e),
                        key: None,
                        backtrace: Some(std::backtrace::Backtrace::capture()),
                    })?;

                // Skip value
                file.seek(SeekFrom::Current(value_size as i64))
                    .map_err(|e| SGBDError::Storage {
                        operation: format!("skip value during index build: {}", e),
                        key: None,
                        backtrace: Some(std::backtrace::Backtrace::capture()),
                    })?;

                // Deserialize key
                let key = Key::from_bytes(&key_buf).map_err(|e| SGBDError::Storage {
                    operation: format!("deserialize key during index build: {}", e),
                    key: None,
                    backtrace: Some(std::backtrace::Backtrace::capture()),
                })?;

                // Only keep the latest version of non-deleted keys
                if !deleted {
                    index.insert(key, (segment.id, offset));
                }

                // Move to next record
                offset += (HEADER_SIZE + key_size + value_size) as u64;
            }
        }

        Ok(index)
    }

    fn compress(data: &[u8], compression: CompressionType) -> Result<Bytes> {
        match compression {
            CompressionType::Lz4 => {
                let mut encoder = EncoderBuilder::new()
                    .build(Vec::new())
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                encoder.write_all(data)?;
                let (compressed, result) = encoder.finish();
                result?;
                Ok(Bytes::from(compressed))
            }
            CompressionType::Zstd => {
                let compressed = zstd::stream::encode_all(std::io::Cursor::new(data), 3)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                Ok(Bytes::from(compressed))
            }
            CompressionType::Snappy => {
                let mut encoder = snap::write::FrameEncoder::new(Vec::new());
                encoder.write_all(data)?;
                let compressed = encoder.into_inner()?;
                Ok(Bytes::from(compressed))
            }
            _ => Ok(Bytes::copy_from_slice(data)),
        }
    }

    fn decompress(data: &[u8], compression: CompressionType) -> Result<Bytes> {
        match compression {
            CompressionType::Lz4 => {
                let mut decoder = lz4::Decoder::new(data)?;
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(Bytes::from(decompressed))
            }
            CompressionType::Zstd => {
                let decompressed = decode_all(data)?;
                Ok(Bytes::from(decompressed))
            }
            CompressionType::Snappy => {
                let mut decoder = snap::read::FrameDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(Bytes::from(decompressed))
            }
            _ => Ok(Bytes::copy_from_slice(data)),
        }
    }
}

/// Storage record format
struct StorageRecord {
    key: Bytes,
    value: Bytes,
    compression: CompressionType,
    timestamp: Timestamp,
    deleted: bool,
}

impl StorageRecord {
    fn size(&self) -> usize {
        HEADER_SIZE + self.key.len() + self.value.len()
    }

    fn serialize(&self, buf: &mut BytesMut) {
        // Calculate checksum
        let mut hasher = Crc32::new();
        hasher.update(&self.key);
        hasher.update(&self.value);
        let checksum = hasher.finalize();

        // Write header
        buf.put_u32(self.key.len() as u32);
        buf.put_u32(self.value.len() as u32);
        buf.put_u64(self.timestamp.as_u64());
        buf.put_u8(if self.deleted { 0x01 } else { 0x00 });
        buf.put_u8(self.compression.to_u8());
        buf.put_u32(checksum);
        buf.put_bytes(0, 10); // Reserved

        // Write data
        buf.put_slice(&self.key);
        buf.put_slice(&self.value);
    }
}

impl CompressionType {
    fn to_u8(&self) -> u8 {
        match self {
            CompressionType::None => 0,
            CompressionType::Lz4 => 1,
            CompressionType::Zstd => 2,
            CompressionType::Snappy => 3,
            CompressionType::Brotli => 4,
        }
    }

    fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(CompressionType::None),
            1 => Ok(CompressionType::Lz4),
            2 => Ok(CompressionType::Zstd),
            3 => Ok(CompressionType::Snappy),
            4 => Ok(CompressionType::Brotli),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "invalid compression type").into()),
        }
    }
}

impl Timestamp {
    fn as_u64(&self) -> u64 {
        (self.seconds << 32) | self.nanos as u64
    }

    fn from_u64(value: u64) -> Self {
        Self {
            seconds: value >> 32,
            nanos: (value & 0xFFFFFFFF) as u32,
        }
    }
}

// Implement serialization strategies from types.rs
struct BincodeStrategy;
struct JsonStrategy;
struct MessagePackStrategy;

impl SerializationStrategy for BincodeStrategy {
    fn serialize<T: serde::Serialize>(&self, value: &T) -> Result<Vec<u8>> {
        bincode::serialize(value).map_err(|e| e.into())
    }

    fn deserialize<T: for<'de> serde::Deserialize<'de>>(&self, bytes: &[u8]) -> Result<T> {
        bincode::deserialize(bytes).map_err(|e| e.into())
    }

    fn format(&self) -> SerializationFormat {
        SerializationFormat::Bincode
    }

    fn name(&self) -> &'static str {
        "bincode"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Key, Value};
    use tempfile::tempdir;

    async fn test_storage() -> (StorageEngine, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let config = StorageConfig {
            max_segment_size: 1024 * 1024,    // 1MB
            max_value_size: 10 * 1024 * 1024, // 10MB
            compression: CompressionType::None,
            serialization_format: SerializationFormat::Bincode,
            enable_checksums: true,
            buffer_pool_size: 16,
            compaction_threshold: 0.7,
            background_compaction_interval: Duration::from_secs(60),
            base_path: dir.path().to_path_buf(),
        };

        let storage = StorageEngine::new(config, None).await.unwrap();
        (storage, dir)
    }

    #[tokio::test]
    async fn test_write_read() {
        let (storage, _dir) = test_storage().await;
        let key = Key::new_uuid();
        let value = Value::Metadata(Default::default());

        storage.write(key.clone(), value.clone()).await.unwrap();
        let result = storage.read(&key).await.unwrap();

        assert!(result.is_some());
        if let Value::Metadata(meta) = result.unwrap() {
            assert!(meta.is_empty());
        } else {
            panic!("Unexpected value type");
        }
    }

    #[tokio::test]
    async fn test_delete() {
        let (storage, _dir) = test_storage().await;
        let key = Key::new_uuid();
        let value = Value::Metadata(Default::default());

        storage.write(key.clone(), value.clone()).await.unwrap();
        storage.delete(&key).await.unwrap();
        let result = storage.read(&key).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_segment_rotation() {
        let (storage, _dir) = test_storage().await;
        let state = storage.state.read().await;
        let initial_segment = state.active_segment.id;

        // Write enough data to fill segment
        for _ in 0..1000 {
            let key = Key::new_uuid();
            let value = Value::Metadata(HashMap::new());
            storage.write(key, value).await.unwrap();
        }

        let state = storage.state.read().await;
        assert_ne!(state.active_segment.id, initial_segment);
        assert!(state.segments.len() > 0);
    }

    #[tokio::test]
    async fn test_compression() {
        let dir = tempdir().unwrap();
        let config = StorageConfig {
            compression: CompressionType::Lz4,
            base_path: dir.path().to_path_buf(),
            ..StorageConfig {
                max_segment_size: 1024 * 1024,
                max_value_size: 10 * 1024 * 1024,
                serialization_format: SerializationFormat::Bincode,
                enable_checksums: true,
                buffer_pool_size: 16,
                compaction_threshold: 0.7,
                background_compaction_interval: Duration::from_secs(60),
                base_path: dir.path().to_path_buf(),
            }
        };

        let storage = StorageEngine::new(config, None).await.unwrap();
        let key = Key::new_uuid();

        // Create large value
        let mut data = vec![0u8; 1024 * 1024]; // 1MB
        for i in 0..data.len() {
            data[i] = rand::random();
        }
        let value = Value::Raw(Bytes::from(data));

        storage.write(key.clone(), value.clone()).await.unwrap();
        let result = storage.read(&key).await.unwrap().unwrap();

        match result {
            Value::Raw(bytes) => assert_eq!(bytes.len(), 1024 * 1024),
            _ => panic!("Unexpected value type"),
        }
    }

    #[tokio::test]
    async fn test_compaction() {
        let (storage, _dir) = test_storage().await;

        // Write many keys
        for i in 0..1000 {
            let key = Key::new_numeric(i as u64);
            let value = Value::Metadata(Default::default());
            storage.write(key.clone(), value).await.unwrap();
        }

        // Delete half of them
        for i in 0..500 {
            let key = Key::new_numeric(i as u64);
            storage.delete(&key).await.unwrap();
        }

        // Force compaction
        storage.compact().await.unwrap();

        // Verify remaining keys
        for i in 0..1000 {
            let key = Key::new_numeric(i as u64);
            let result = storage.read(&key).await.unwrap();
            if i < 500 {
                assert!(result.is_none());
            } else {
                assert!(result.is_some());
            }
        }

        // Verify metrics
        let metrics = storage.metrics().await;
        assert_eq!(metrics.compaction_count, 1);
        assert_eq!(metrics.active_segments, 1);
    }

    #[tokio::test]
    async fn test_recovery() {
        let dir = tempdir().unwrap();
        let config = StorageConfig {
            base_path: dir.path().to_path_buf(),
            ..StorageConfig {
                max_segment_size: 1024 * 1024,
                max_value_size: 10 * 1024 * 1024,
                compression: CompressionType::None,
                serialization_format: SerializationFormat::Bincode,
                enable_checksums: true,
                buffer_pool_size: 16,
                compaction_threshold: 0.7,
                background_compaction_interval: Duration::from_secs(60),
                base_path: dir.path().to_path_buf(),
            }
        };

        // First create and write data
        {
            let storage = StorageEngine::new(config.clone(), None).await.unwrap();
            let key = Key::new_uuid();
            let value = Value::Metadata(Default::default());
            storage.write(key.clone(), value.clone()).await.unwrap();
            storage.shutdown().await;
        }

        // Reopen storage
        let storage = StorageEngine::new(config, None).await.unwrap();
        let key = Key::new_uuid();
        let result = storage.read(&key).await.unwrap();
        assert!(result.is_some());
    }
}

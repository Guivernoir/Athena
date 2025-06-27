use crate::sgbd::{
    DatabaseConfig, Operation, Result, SGBDError, SerializationStrategy, Timestamp, WalError,
};
use bytes::{Bytes, BytesMut};
use crc32fast::Hasher as Crc32;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    sync::{Mutex, Notify},
    time,
};
use uuid::Uuid;

/// Write-Ahead Log manager
pub struct Wal {
    writer: Mutex<BufWriter<File>>,
    current_segment_path: String,
    config: Arc<DatabaseConfig>,
    strategy: Arc<dyn SerializationStrategy>,
    metrics: Arc<WalMetrics>,
    flush_notify: Arc<Notify>,
    shutdown: Arc<Notify>,
}

/// WAL metrics container
#[derive(Default)]
struct WalMetrics {
    write_count: AtomicU64,
    flush_count: AtomicU64,
    rotation_count: AtomicU64,
    bytes_written: AtomicU64,
    error_count: AtomicU64,
    last_flush_duration: AtomicU64, // microseconds
}

/// WAL record header (fixed size)
#[repr(C)]
struct RecordHeader {
    length: u32,
    checksum: u32,
    timestamp: u64,
}

/// WAL metrics snapshot
pub struct WalMetricsSnapshot {
    pub write_count: u64,
    pub flush_count: u64,
    pub rotation_count: u64,
    pub bytes_written: u64,
    pub error_count: u64,
    pub last_flush_duration_us: u64,
}

impl Wal {
    /// Initialize WAL from configuration
    pub async fn new(config: Arc<DatabaseConfig>) -> Result<Self> {
        let wal_dir = config
            .wal_directory
            .as_deref()
            .unwrap_or(&config.data_directory);
        fs::create_dir_all(wal_dir).map_err(|e| SGBDError::Wal {
            operation: format!("create WAL directory: {}", e),
            offset: None,
            backtrace: Some(std::backtrace::Backtrace::capture()),
        })?;

        let (file_path, file) = Self::create_segment_file(wal_dir)?;
        let strategy = Self::select_serialization_strategy(&config);
        let metrics = Arc::new(WalMetrics::default());
        let flush_notify = Arc::new(Notify::new());
        let shutdown = Arc::new(Notify::new());

        let wal = Self {
            writer: Mutex::new(BufWriter::with_capacity(
                config.wal_buffer_size_kb as usize * 1024,
                file,
            )),
            current_segment_path: file_path,
            config,
            strategy,
            metrics,
            flush_notify: flush_notify.clone(),
            shutdown: shutdown.clone(),
        };

        // Start background flusher
        tokio::spawn(Self::background_flusher(
            wal.clone_for_background(),
            flush_notify,
            shutdown,
        ));

        Ok(wal)
    }

    /// Append an operation to the WAL
    pub async fn append(&self, operation: Operation) -> Result<u64> {
        let record = WalRecord {
            operation,
            timestamp: Timestamp::now(),
        };

        let mut serialized = self.strategy.serialize(&record).map_err(|e| {
            self.metrics.error_count.fetch_add(1, Ordering::Relaxed);
            SGBDError::Wal {
                operation: format!("serialization: {}", e),
                offset: None,
                backtrace: Some(std::backtrace::Backtrace::capture()),
            }
        })?;

        let mut writer = self.writer.lock().await;
        let offset = writer.stream_position().map_err(|e| SGBDError::Wal {
            operation: format!("get position: {}", e),
            offset: None,
            backtrace: Some(std::backtrace::Backtrace::capture()),
        })? + self.base_offset();

        self.write_record(&mut writer, &mut serialized).await?;

        // Update metrics
        self.metrics.write_count.fetch_add(1, Ordering::Relaxed);
        self.metrics
            .bytes_written
            .fetch_add(serialized.len() as u64, Ordering::Relaxed);

        // Notify flusher if buffer is full
        if writer.buffer().len() >= self.config.wal_buffer_size_kb as usize * 1024 {
            self.flush_notify.notify_one();
        }

        Ok(offset)
    }

    /// Write record with header
    async fn write_record(&self, writer: &mut BufWriter<File>, data: &mut Vec<u8>) -> Result<()> {
        // Calculate checksum
        let mut hasher = Crc32::new();
        hasher.update(data);
        let checksum = hasher.finalize();

        // Build header
        let header = RecordHeader {
            length: data.len() as u32,
            checksum,
            timestamp: Timestamp::now().as_u64(),
        };

        // Write header + data
        let header_bytes = unsafe {
            std::slice::from_raw_parts(
                &header as *const _ as *const u8,
                std::mem::size_of::<RecordHeader>(),
            )
        };

        writer.write_all(header_bytes).map_err(|e| {
            self.metrics.error_count.fetch_add(1, Ordering::Relaxed);
            SGBDError::Wal {
                operation: format!("write header: {}", e),
                offset: None,
                backtrace: Some(std::backtrace::Backtrace::capture()),
            }
        })?;

        writer.write_all(data).map_err(|e| {
            self.metrics.error_count.fetch_add(1, Ordering::Relaxed);
            SGBDError::Wal {
                operation: format!("write data: {}", e),
                offset: None,
                backtrace: Some(std::backtrace::Backtrace::capture()),
            }
        })?;

        Ok(())
    }

    /// Background flush task
    async fn background_flusher(wal: Arc<Self>, flush_notify: Arc<Notify>, shutdown: Arc<Notify>) {
        let flush_interval = Duration::from_millis(wal.config.wal_sync_interval_ms);

        loop {
            tokio::select! {
                _ = time::sleep(flush_interval) => {
                    if let Err(e) = wal.flush().await {
                        log::error!("Background flush failed: {}", e);
                    }
                }
                _ = flush_notify.notified() => {
                    if let Err(e) = wal.flush().await {
                        log::error!("Buffer-full flush failed: {}", e);
                    }
                }
                _ = shutdown.notified() => break,
            }
        }
    }

    /// Flush buffered data to disk
    pub async fn flush(&self) -> Result<()> {
        let start = std::time::Instant::now();
        let mut writer = self.writer.lock().await;

        writer.flush().map_err(|e| {
            self.metrics.error_count.fetch_add(1, Ordering::Relaxed);
            SGBDError::Wal {
                operation: format!("flush: {}", e),
                offset: None,
                backtrace: Some(std::backtrace::Backtrace::capture()),
            }
        })?;

        // Handle segment rotation
        let metadata = writer.get_ref().metadata().map_err(|e| SGBDError::Wal {
            operation: format!("metadata: {}", e),
            offset: None,
            backtrace: Some(std::backtrace::Backtrace::capture()),
        })?;

        if metadata.len() >= self.config.wal_segment_size_mb * 1_000_000 {
            self.rotate_segment().await?;
        }

        // Update metrics
        let duration = start.elapsed().as_micros() as u64;
        self.metrics.flush_count.fetch_add(1, Ordering::Relaxed);
        self.metrics
            .last_flush_duration
            .store(duration, Ordering::Relaxed);

        Ok(())
    }

    /// Rotate to a new WAL segment
    pub async fn rotate_segment(&self) -> Result<()> {
        let mut writer = self.writer.lock().await;
        writer.flush().map_err(|e| SGBDError::Wal {
            operation: format!("flush before rotate: {}", e),
            offset: None,
            backtrace: Some(std::backtrace::Backtrace::capture()),
        })?;

        let wal_dir = self
            .config
            .wal_directory
            .as_deref()
            .unwrap_or(&self.config.data_directory);
        let (new_path, new_file) = Self::create_segment_file(wal_dir)?;

        *writer =
            BufWriter::with_capacity(self.config.wal_buffer_size_kb as usize * 1024, new_file);

        // Update metadata
        self.current_segment_path = new_path;
        self.metrics.rotation_count.fetch_add(1, Ordering::Relaxed);

        // Start retention cleanup
        self.start_retention_cleanup(wal_dir);

        Ok(())
    }

    /// Start retention cleanup in background
    fn start_retention_cleanup(&self, wal_dir: &str) {
        if self.config.wal_retention_hours > 0 {
            let retention = Duration::from_secs(self.config.wal_retention_hours * 3600);
            let wal_dir = wal_dir.to_string();
            let metrics = self.metrics.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::cleanup_old_segments(&wal_dir, retention).await {
                    metrics.error_count.fetch_add(1, Ordering::Relaxed);
                    log::error!("WAL cleanup failed: {}", e);
                }
            });
        }
    }

    /// Cleanup old segments
    async fn cleanup_old_segments(wal_dir: &str, retention: Duration) -> Result<()> {
        let entries = fs::read_dir(wal_dir).map_err(|e| SGBDError::Wal {
            operation: format!("read dir: {}", e),
            offset: None,
            backtrace: Some(std::backtrace::Backtrace::capture()),
        })?;

        let now = std::time::SystemTime::now();
        for entry in entries {
            let entry = entry.map_err(|e| SGBDError::Wal {
                operation: format!("read entry: {}", e),
                offset: None,
                backtrace: Some(std::backtrace::Backtrace::capture()),
            })?;

            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if now.duration_since(modified).unwrap_or_default() > retention {
                        fs::remove_file(entry.path()).map_err(|e| SGBDError::Wal {
                            operation: format!("remove file: {}", e),
                            offset: None,
                            backtrace: Some(std::backtrace::Backtrace::capture()),
                        })?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Get current metrics snapshot
    pub fn metrics(&self) -> WalMetricsSnapshot {
        WalMetricsSnapshot {
            write_count: self.metrics.write_count.load(Ordering::Relaxed),
            flush_count: self.metrics.flush_count.load(Ordering::Relaxed),
            rotation_count: self.metrics.rotation_count.load(Ordering::Relaxed),
            bytes_written: self.metrics.bytes_written.load(Ordering::Relaxed),
            error_count: self.metrics.error_count.load(Ordering::Relaxed),
            last_flush_duration_us: self.metrics.last_flush_duration.load(Ordering::Relaxed),
        }
    }

    /// Gracefully shutdown WAL
    pub async fn shutdown(&self) -> Result<()> {
        self.shutdown.notify_one();
        self.flush().await?;
        Ok(())
    }

    // Helper functions
    fn base_offset(&self) -> u64 {
        // Implementation for offset tracking would go here
        0
    }

    fn create_segment_file(wal_dir: &str) -> Result<(String, File)> {
        let file_path = format!("{}/segment_{}.wal", wal_dir, Uuid::new_v4());
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .map_err(|e| SGBDError::Wal {
                operation: format!("create segment: {}", e),
                offset: None,
                backtrace: Some(std::backtrace::Backtrace::capture()),
            })?;
        Ok((file_path, file))
    }

    fn select_serialization_strategy(config: &DatabaseConfig) -> Arc<dyn SerializationStrategy> {
        match config.serialization_format {
            SerializationFormat::Json => Arc::new(JsonStrategy),
            SerializationFormat::MessagePack => Arc::new(MessagePackStrategy),
            _ => Arc::new(BincodeStrategy), // Default
        }
    }

    fn clone_for_background(&self) -> Arc<Self> {
        Arc::new(Self {
            writer: Mutex::new(BufWriter::new(
                File::open(&self.current_segment_path).unwrap(),
            )),
            current_segment_path: self.current_segment_path.clone(),
            config: self.config.clone(),
            strategy: self.strategy.clone(),
            metrics: self.metrics.clone(),
            flush_notify: self.flush_notify.clone(),
            shutdown: self.shutdown.clone(),
        })
    }
}

/// Internal record structure
#[derive(serde::Serialize, serde::Deserialize)]
struct WalRecord {
    operation: Operation,
    timestamp: Timestamp,
}

// Serialization strategy implementations (from types.rs)
struct BincodeStrategy;
struct JsonStrategy;
struct MessagePackStrategy;

impl SerializationStrategy for BincodeStrategy {
    fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>> {
        bincode::serialize(value).map_err(SGBDError::from)
    }

    fn deserialize<T: for<'de> Deserialize<'de>>(&self, bytes: &[u8]) -> Result<T> {
        bincode::deserialize(bytes).map_err(SGBDError::from)
    }

    fn format(&self) -> SerializationFormat {
        SerializationFormat::Bincode
    }

    fn name(&self) -> &'static str {
        "bincode"
    }
}

impl SerializationStrategy for JsonStrategy {
    fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>> {
        serde_json::to_vec(value).map_err(SGBDError::from)
    }

    fn deserialize<T: for<'de> Deserialize<'de>>(&self, bytes: &[u8]) -> Result<T> {
        serde_json::from_slice(bytes).map_err(SGBDError::from)
    }

    fn format(&self) -> SerializationFormat {
        SerializationFormat::Json
    }

    fn name(&self) -> &'static str {
        "json"
    }
}

impl SerializationStrategy for MessagePackStrategy {
    fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>> {
        rmp_serde::to_vec(value).map_err(SGBDError::from)
    }

    fn deserialize<T: for<'de> Deserialize<'de>>(&self, bytes: &[u8]) -> Result<T> {
        rmp_serde::from_slice(bytes).map_err(SGBDError::from)
    }

    fn format(&self) -> SerializationFormat {
        SerializationFormat::MessagePack
    }

    fn name(&self) -> &'static str {
        "message_pack"
    }
}

impl Timestamp {
    /// Convert to u64 for efficient storage
    fn as_u64(&self) -> u64 {
        (self.seconds << 32) | self.nanos as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DatabaseConfig, Key, Value};
    use tempfile::tempdir;

    async fn test_wal() -> (Wal, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let mut config = DatabaseConfig::development();
        config.data_directory = dir.path().to_str().unwrap().to_string();
        config.wal_directory = Some(dir.path().to_str().unwrap().to_string());

        let wal = Wal::new(Arc::new(config)).await.unwrap();
        (wal, dir)
    }

    #[tokio::test]
    async fn test_append_and_flush() {
        let (wal, _dir) = test_wal().await;
        let key = Key::new_uuid();
        let value = Value::Metadata(Default::default());
        let operation = Operation::Insert { key, value };

        wal.append(operation).await.unwrap();
        wal.flush().await.unwrap();

        let metrics = wal.metrics();
        assert_eq!(metrics.write_count, 1);
        assert_eq!(metrics.flush_count, 1);
        assert!(metrics.bytes_written > 0);
    }

    #[tokio::test]
    async fn test_segment_rotation() {
        let (wal, _dir) = test_wal().await;
        wal.config.wal_segment_size_mb = 1; // 1MB

        // Write enough data to trigger rotation
        for _ in 0..1000 {
            let key = Key::new_uuid();
            let value = Value::Metadata(Default::default());
            wal.append(Operation::Insert { key, value }).await.unwrap();
        }
        wal.flush().await.unwrap();

        let metrics = wal.metrics();
        assert!(metrics.rotation_count > 0);
    }

    #[tokio::test]
    async fn test_retention_cleanup() {
        let (wal, dir) = test_wal().await;
        wal.config.wal_retention_hours = 0; // Disable for test

        // Create dummy segments
        for _ in 0..3 {
            let _ = wal.rotate_segment().await;
        }

        // Manually test cleanup
        let entries = fs::read_dir(dir.path()).unwrap().count();
        assert!(entries > 1);
    }
}

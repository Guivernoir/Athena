use crate::sgbd::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use bytes::{BufMut, Bytes, BytesMut};
use crc64fast::Digest as Crc64;
use snap::raw::{Decoder, Encoder};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::{Mutex, Notify, RwLock, Semaphore};
use tokio::time::{sleep, timeout};

const SEGMENT_SIZE: u64 = 64 * 1024 * 1024; // 64MB segments
const GROUP_COMMIT_THRESHOLD: usize = 32; // Entries per group commit
const GROUP_COMMIT_TIMEOUT: u64 = 10; // Milliseconds
const COMPRESSION_THRESHOLD: usize = 1024; // Compress values >1KB
const MAX_CONCURRENT_READERS: usize = 16; // Reader concurrency limit
const FRAME_MAGIC: u32 = 0xDEADBEEF; // Entry frame magic number
const SEGMENT_MAGIC: &[u8; 4] = b"WAL\x00"; // Segment magic
const MAX_ENTRY_SIZE: u32 = 16 * 1024 * 1024; // 16MB max entry
const RECOVERY_BATCH_SIZE: usize = 1000; // Entries per recovery batch

/// Entry frame structure for bulletproof parsing
#[derive(Debug, Clone)]
struct EntryFrame {
    magic: u32,     // Frame identification
    length: u32,    // Total frame length (header + payload)
    crc32: u32,     // CRC32 of payload
    timestamp: u64, // Entry timestamp
    sequence: u64,  // Monotonic sequence number
    flags: u32,     // Compression, encryption, etc.
    key_len: u32,   // Key length
    value_len: u32, // Value length
                    // Payload follows: [key_data][value_data]
}

impl EntryFrame {
    const HEADER_SIZE: usize = 44; // All fields above

    fn validate(&self) -> Result<()> {
        if self.magic != FRAME_MAGIC {
            return Err(SGBDError::Serialization {
                context: format!("Invalid magic: {:08x}", self.magic),
                format: SerializationFormat::Bincode,
                backtrace: None,
            });
        }

        if self.length > MAX_ENTRY_SIZE {
            return Err(SGBDError::Serialization {
                context: format!("Entry too large: {} > {}", self.length, MAX_ENTRY_SIZE),
                format: SerializationFormat::Bincode,
                backtrace: None,
            });
        }

        if self.key_len == 0 {
            return Err(SGBDError::Serialization {
                context: "Zero key length".to_string(),
                format: SerializationFormat::Bincode,
                backtrace: None,
            });
        }

        Ok(())
    }

    fn to_bytes(&self) -> Vec<u8> {
        // Use the serialization strategy from types.rs (BincodeStrategy)
        BincodeStrategy.serialize(self).unwrap_or_default()
    }

    fn from_bytes(data: &[u8]) -> Result<Self> {
        // Use the deserialization strategy from types.rs (BincodeStrategy)
        BincodeStrategy.deserialize(data)
    }
}

pub struct WriteAheadLog {
    directory: PathBuf,
    current_segment: Arc<Mutex<SegmentWriter>>,
    commit_notify: Arc<Notify>,
    metrics: Arc<WalMetrics>,
    sequence: AtomicU64,
    reader_semaphore: Arc<Semaphore>,
    config: WalConfig,
    recovery_state: Arc<RwLock<RecoveryState>>,
    background_tasks: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

#[derive(Clone)]
pub struct WalConfig {
    pub segment_size: u64,
    pub group_commit_threshold: usize,
    pub group_commit_timeout: Duration,
    pub compression_threshold: usize,
    pub max_concurrent_readers: usize,
    pub sync_on_commit: bool,
    pub enable_compression: bool,
    pub recovery_parallelism: usize,
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            segment_size: SEGMENT_SIZE,
            group_commit_threshold: GROUP_COMMIT_THRESHOLD,
            group_commit_timeout: Duration::from_millis(GROUP_COMMIT_TIMEOUT),
            compression_threshold: COMPRESSION_THRESHOLD,
            max_concurrent_readers: MAX_CONCURRENT_READERS,
            sync_on_commit: true,
            enable_compression: true,
            recovery_parallelism: 4,
        }
    }
}

struct SegmentWriter {
    writer: BufWriter<File>,
    file_path: PathBuf,
    temp_path: PathBuf,
    position: u64,
    pending_entries: usize,
    last_sync: Instant,
    sequence_start: u64,
    is_finalized: bool,
}

#[derive(Debug, Default)]
struct RecoveryState {
    last_valid_position: HashMap<PathBuf, u64>,
    corrupted_segments: Vec<PathBuf>,
    recovery_stats: RecoveryStats,
}

#[derive(Debug, Default, Clone)]
pub struct RecoveryStats {
    pub segments_processed: u32,
    pub entries_recovered: u64,
    pub bytes_recovered: u64,
    pub corrupted_entries: u32,
    pub recovery_duration: Duration,
}

#[derive(Debug)]
pub struct ReplayResult {
    pub entries: Vec<(Key, Value)>,
    pub valid_bytes: u64,
    pub segment_metrics: Vec<SegmentMetrics>,
    pub recovery_stats: RecoveryStats,
}

#[derive(Debug, Default, Clone)]
pub struct SegmentMetrics {
    pub path: PathBuf,
    pub entries: u64,
    pub bytes: u64,
    pub checksum_errors: u32,
    pub corrupted_frames: u32,
    pub last_sequence: u64,
}

#[derive(Default)]
struct WalMetrics {
    // Write metrics
    write_bytes: AtomicU64,
    write_entries: AtomicU64,
    write_batches: AtomicU64,

    // Sync metrics
    sync_count: AtomicU64,
    sync_duration: AtomicU64,
    sync_bytes: AtomicU64,

    // Segment metrics
    rotation_count: AtomicU64,
    active_segments: AtomicU32,

    // Error metrics
    checksum_errors: AtomicU32,
    frame_errors: AtomicU32,
    io_errors: AtomicU32,

    // Concurrency metrics
    active_readers: AtomicU32,
    active_writers: AtomicU32,

    // Performance metrics
    compression_ratio: AtomicU64, // x1000 for precision
    avg_entry_size: AtomicU64,
}

impl WriteAheadLog {
    pub async fn open(directory: &Path) -> Result<Self> {
        Self::open_with_config(directory, WalConfig::default()).await
    }

    pub async fn open_with_config(directory: &Path, config: WalConfig) -> Result<Self> {
        fs::create_dir_all(directory).await?;
        let metrics = Arc::new(WalMetrics::default());
        let recovery_state = Arc::new(RwLock::new(RecoveryState::default()));

        // Perform crash recovery
        let sequence = Self::perform_crash_recovery(directory, &recovery_state).await?;

        let wal = Self {
            directory: directory.to_path_buf(),
            current_segment: Arc::new(Mutex::new(SegmentWriter {
                writer: BufWriter::new(File::create("").await?), // Placeholder
                file_path: PathBuf::new(),
                temp_path: PathBuf::new(),
                position: 0,
                pending_entries: 0,
                last_sync: Instant::now(),
                sequence_start: sequence,
                is_finalized: false,
            })),
            commit_notify: Arc::new(Notify::new()),
            metrics: metrics.clone(),
            sequence: AtomicU64::new(sequence),
            reader_semaphore: Arc::new(Semaphore::new(config.max_concurrent_readers)),
            config,
            recovery_state,
            background_tasks: Arc::new(Mutex::new(Vec::new())),
        };

        wal.create_new_segment().await?;
        wal.start_background_tasks().await;

        Ok(wal)
    }

    async fn perform_crash_recovery(
        directory: &Path,
        recovery_state: &Arc<RwLock<RecoveryState>>,
    ) -> Result<u64> {
        let start_time = Instant::now();
        let mut state = recovery_state.write().await;
        let mut max_sequence = 0;

        let segments = Self::list_segments(directory).await?;

        for (seq, path) in segments {
            if seq > max_sequence {
                max_sequence = seq;
            }

            match Self::validate_segment(&path).await {
                Ok(last_valid_pos) => {
                    state
                        .last_valid_position
                        .insert(path.clone(), last_valid_pos);
                    state.recovery_stats.segments_processed += 1;
                }
                Err(_) => {
                    state.corrupted_segments.push(path);
                }
            }
        }

        state.recovery_stats.recovery_duration = start_time.elapsed();
        Ok(max_sequence)
    }

    async fn validate_segment(path: &Path) -> Result<u64> {
        let mut file = File::open(path).await?;
        let file_size = file.metadata().await?.len();
        let mut reader = BufReader::new(file);
        let mut position = 0u64;

        // Validate header
        let mut header = [0u8; 8];
        reader.read_exact(&mut header).await?;
        position += 8;

        if &header[0..4] != SEGMENT_MAGIC {
            return Err(SGBDError::Serialization {
                context: "Invalid segment magic".to_string(),
                format: SerializationFormat::Bincode,
                backtrace: None,
            });
        }

        // Scan for valid entries
        let mut last_valid_position = position;

        while position < file_size {
            let frame_start = position;

            // Try to read frame header
            let mut frame_header = vec![0u8; EntryFrame::HEADER_SIZE];
            match reader.read_exact(&mut frame_header).await {
                Ok(_) => {
                    match EntryFrame::from_bytes(&frame_header) {
                        Ok(frame) => {
                            // Validate frame integrity
                            let payload_size = (frame.key_len + frame.value_len) as usize;
                            let mut payload = vec![0u8; payload_size];

                            if reader.read_exact(&mut payload).await.is_ok() {
                                // Verify CRC
                                let crc = crc32fast::hash(&payload);
                                if crc == frame.crc32 {
                                    position +=
                                        EntryFrame::HEADER_SIZE as u64 + payload_size as u64;
                                    last_valid_position = position;
                                    continue;
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
                Err(_) => break,
            }

            // If we get here, we hit corruption
            break;
        }

        Ok(last_valid_position)
    }

    async fn create_new_segment(&self) -> Result<()> {
        let next_seq = self.sequence.fetch_add(1, Ordering::SeqCst) + 1;
        let file_path = self.directory.join(format!("wal-{:020}.log", next_seq));
        let temp_path = self.directory.join(format!("wal-{:020}.tmp", next_seq));

        // Atomic segment creation: write to temp file first
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temp_path)
            .await?;

        // Write segment header
        let mut header = Vec::with_capacity(16);
        header.extend_from_slice(SEGMENT_MAGIC);
        header.extend_from_slice(&1u16.to_le_bytes()); // Version
        header.extend_from_slice(&0u16.to_le_bytes()); // Flags
        header.extend_from_slice(&next_seq.to_le_bytes()); // Sequence

        file.write_all(&header).await?;
        file.sync_all().await?;

        // Atomically rename temp file to final name
        fs::rename(&temp_path, &file_path)
            .await
            .map_err(|e| SGBDError::Io {
                context: format!("Failed to rename segment: {}", e),
                source: Some(e.to_string()),
                backtrace: None,
            })?;

        // Open the finalized file
        let file = OpenOptions::new().append(true).open(&file_path).await?;

        let writer = BufWriter::new(file);

        let mut current_segment = self.current_segment.lock().await;

        // Finalize previous segment if exists
        if !current_segment.file_path.as_os_str().is_empty() && !current_segment.is_finalized {
            self.finalize_segment_locked(&mut current_segment).await?;
        }

        *current_segment = SegmentWriter {
            writer,
            file_path,
            temp_path,
            position: header.len() as u64,
            pending_entries: 0,
            last_sync: Instant::now(),
            sequence_start: next_seq,
            is_finalized: false,
        };

        self.metrics.rotation_count.fetch_add(1, Ordering::Relaxed);
        self.metrics.active_segments.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    async fn finalize_segment_locked(&self, segment: &mut SegmentWriter) -> Result<()> {
        if segment.is_finalized {
            return Ok(());
        }

        // Write segment footer with metadata
        let footer = self.create_segment_footer(segment).await?;
        segment.writer.write_all(&footer).await?;
        segment.writer.flush().await?;
        segment.writer.get_ref().sync_all().await?;
        segment.is_finalized = true;

        Ok(())
    }

    async fn create_segment_footer(&self, segment: &SegmentWriter) -> Result<Vec<u8>> {
        let mut footer = Vec::new();

        // Footer format: [entry_count: 8][last_sequence: 8][checksum: 8][magic: 4]
        footer.extend_from_slice(&(segment.pending_entries as u64).to_le_bytes());
        footer.extend_from_slice(&self.sequence.load(Ordering::Relaxed).to_le_bytes());

        // Calculate footer checksum
        let checksum = crc64fast::hash(&footer);
        footer.extend_from_slice(&checksum.to_le_bytes());
        footer.extend_from_slice(b"FOOT");

        Ok(footer)
    }

    pub async fn append(&self, key: &Key, value: &Value) -> Result<()> {
        let batch = vec![(key.clone(), value.clone())];
        self.append_batch(&batch).await
    }

    pub async fn append_batch(&self, entries: &[(Key, Value)]) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }

        let batch_start = Instant::now();
        self.metrics.active_writers.fetch_add(1, Ordering::Relaxed);

        let result = self.append_batch_internal(entries).await;

        self.metrics.active_writers.fetch_sub(1, Ordering::Relaxed);
        self.metrics.write_batches.fetch_add(1, Ordering::Relaxed);

        // Update performance metrics
        let batch_duration = batch_start.elapsed().as_micros() as u64;
        self.update_performance_metrics(entries.len(), batch_duration);

        result
    }

    async fn append_batch_internal(&self, entries: &[(Key, Value)]) -> Result<()> {
        let mut frames = Vec::with_capacity(entries.len());
        let mut total_size = 0;

        // Pre-serialize all entries
        for (key, value) in entries {
            let frame_data = self.serialize_entry_with_frame(key, value).await?;
            total_size += frame_data.len();
            frames.push(frame_data);
        }

        // Write batch atomically
        let mut segment = self.current_segment.lock().await;

        for frame_data in frames {
            segment.writer.write_all(&frame_data).await?;
            segment.position += frame_data.len() as u64;
        }

        segment.pending_entries += entries.len();

        // Update metrics
        self.metrics
            .write_bytes
            .fetch_add(total_size as u64, Ordering::Relaxed);
        self.metrics
            .write_entries
            .fetch_add(entries.len() as u64, Ordering::Relaxed);

        // Group commit logic
        let should_sync = segment.pending_entries >= self.config.group_commit_threshold
            || segment.last_sync.elapsed() > self.config.group_commit_timeout;

        if should_sync && self.config.sync_on_commit {
            self.sync_locked(&mut segment).await?;
        }

        // Rotate if needed
        if segment.position >= self.config.segment_size {
            drop(segment);
            self.create_new_segment().await?;
        }

        Ok(())
    }

    async fn serialize_entry_with_frame(&self, key: &Key, value: &Value) -> Result<Vec<u8>> {
        let key_bytes = key.to_bytes()?; // Uses types.rs serialization
        let mut value_bytes = value.to_bytes()?; // Uses types.rs serialization
        let mut flags = 0u32;

        // Compression
        if self.config.enable_compression && value_bytes.len() > self.config.compression_threshold {
            let mut encoder = Encoder::new();
            let compressed =
                encoder
                    .compress_vec(&value_bytes)
                    .map_err(|e| SGBDError::Serialization {
                        context: e.to_string(),
                        format: SerializationFormat::Bincode,
                        backtrace: None,
                    })?;

            if compressed.len() < value_bytes.len() {
                value_bytes = compressed;
                flags |= 0x01; // Compression flag

                // Update compression ratio metric
                let ratio = (value_bytes.len() * 1000) / key_bytes.len();
                self.metrics
                    .compression_ratio
                    .store(ratio as u64, Ordering::Relaxed);
            }
        }

        // Create payload
        let mut payload = Vec::with_capacity(key_bytes.len() + value_bytes.len());
        payload.extend_from_slice(&key_bytes);
        payload.extend_from_slice(&value_bytes);

        // Calculate CRC32 of payload
        let crc32 = crc32fast::hash(&payload);

        // Create frame
        let frame = EntryFrame {
            magic: FRAME_MAGIC,
            length: (EntryFrame::HEADER_SIZE + payload.len()) as u32,
            crc32,
            timestamp: key.timestamp,
            sequence: self.sequence.fetch_add(1, Ordering::SeqCst),
            flags,
            key_len: key_bytes.len() as u32,
            value_len: value_bytes.len() as u32,
        };

        // Serialize complete frame
        let mut frame_data = frame.to_bytes();
        frame_data.extend_from_slice(&payload);

        Ok(frame_data)
    }

    pub async fn sync(&self) -> Result<()> {
        let mut segment = self.current_segment.lock().await;
        self.sync_locked(&mut segment).await
    }

    async fn sync_locked(&self, segment: &mut SegmentWriter) -> Result<()> {
        let start = Instant::now();
        let bytes_before = segment.position;

        segment.writer.flush().await?;
        segment.writer.get_ref().sync_all().await?;

        let duration = start.elapsed().as_micros() as u64;
        segment.last_sync = Instant::now();
        segment.pending_entries = 0;

        // Update sync metrics
        self.metrics.sync_count.fetch_add(1, Ordering::Relaxed);
        self.metrics
            .sync_duration
            .fetch_add(duration, Ordering::Relaxed);
        self.metrics
            .sync_bytes
            .fetch_add(bytes_before, Ordering::Relaxed);

        self.commit_notify.notify_waiters();
        Ok(())
    }

    pub async fn replay(directory: &Path) -> Result<ReplayResult> {
        Self::replay_with_config(directory, WalConfig::default()).await
    }

    pub async fn replay_with_config(directory: &Path, config: WalConfig) -> Result<ReplayResult> {
        let start_time = Instant::now();
        let mut entries = Vec::new();
        let mut segment_metrics = Vec::new();
        let mut total_bytes = 0;
        let mut recovery_stats = RecoveryStats::default();

        let mut segments = Self::list_segments(directory).await?;
        segments.sort_by_key(|(seq, _)| *seq);

        // Parallel replay with controlled concurrency
        let semaphore = Arc::new(Semaphore::new(config.recovery_parallelism));
        let mut tasks = Vec::new();

        for (seq, path) in segments {
            let path_clone = path.clone();
            let sem = semaphore.clone();

            let task = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                Self::replay_segment_with_recovery(&path_clone).await
            });

            tasks.push((seq, path, task));
        }

        // Collect results in sequence order
        for (seq, path, task) in tasks {
            match task.await.unwrap() {
                Ok((mut seg_entries, metrics)) => {
                    entries.append(&mut seg_entries);
                    total_bytes += metrics.bytes;
                    recovery_stats.entries_recovered += metrics.entries;
                    recovery_stats.corrupted_entries +=
                        metrics.checksum_errors + metrics.corrupted_frames;
                    segment_metrics.push(metrics);
                }
                Err(e) => {
                    // Log error but continue with other segments
                    recovery_stats.corrupted_entries += 1;
                    segment_metrics.push(SegmentMetrics {
                        path,
                        checksum_errors: 1,
                        ..Default::default()
                    });
                }
            }
        }

        recovery_stats.segments_processed = segment_metrics.len() as u32;
        recovery_stats.bytes_recovered = total_bytes;
        recovery_stats.recovery_duration = start_time.elapsed();

        Ok(ReplayResult {
            entries,
            valid_bytes: total_bytes,
            segment_metrics,
            recovery_stats,
        })
    }

    async fn replay_segment_with_recovery(
        path: &Path,
    ) -> Result<(Vec<(Key, Value)>, SegmentMetrics)> {
        let mut file = File::open(path).await?;
        let file_size = file.metadata().await?.len();
        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();
        let mut position = 0u64;

        let mut metrics = SegmentMetrics {
            path: path.to_path_buf(),
            ..Default::default()
        };

        // Skip segment header
        let mut header = [0u8; 16];
        reader.read_exact(&mut header).await?;
        position += 16;

        // Process entries with robust error handling
        while position < file_size {
            let entry_start = position;

            match Self::read_framed_entry(&mut reader, &mut position).await {
                Ok((key, value, sequence)) => {
                    entries.push((key, value));
                    metrics.entries += 1;
                    metrics.last_sequence = sequence;
                }
                Err(WalError::ChecksumMismatch { .. }) => {
                    metrics.checksum_errors += 1;
                    // Try to skip to next valid frame
                    if let Ok(next_pos) = Self::find_next_valid_frame(&mut reader, position).await {
                        position = next_pos;
                        continue;
                    }
                    break;
                }
                Err(WalError::InvalidFrame { .. }) => {
                    metrics.corrupted_frames += 1;
                    break;
                }
                Err(_) => break,
            }
        }

        metrics.bytes = position;
        Ok((entries, metrics))
    }

    async fn read_framed_entry<R: AsyncReadExt + Unpin>(
        reader: &mut R,
        position: &mut u64,
    ) -> Result<(Key, Value, u64)> {
        // Read frame header
        let mut frame_header = vec![0u8; EntryFrame::HEADER_SIZE];
        reader.read_exact(&mut frame_header).await?;
        *position += EntryFrame::HEADER_SIZE as u64;

        let frame = EntryFrame::from_bytes(&frame_header)?; // Uses types.rs deserialization

        // Read payload
        let payload_size = (frame.key_len + frame.value_len) as usize;
        let mut payload = vec![0u8; payload_size];
        reader.read_exact(&mut payload).await?;
        *position += payload_size as u64;

        // Verify CRC
        let crc = crc32fast::hash(&payload);
        if crc != frame.crc32 {
            return Err(SGBDError::Serialization {
                context: format!("Checksum mismatch at offset {}", *position),
                format: SerializationFormat::Bincode,
                backtrace: None,
            });
        }

        // Split payload
        let key_bytes = &payload[..frame.key_len as usize];
        let value_bytes = &payload[frame.key_len as usize..];

        // Decompress if needed
        let final_value_bytes = if frame.flags & 0x01 != 0 {
            let mut decoder = Decoder::new();
            decoder
                .decompress_vec(value_bytes)
                .map_err(|e| SGBDError::Serialization {
                    context: e.to_string(),
                    format: SerializationFormat::Bincode,
                    backtrace: None,
                })?
        } else {
            value_bytes.to_vec()
        };

        // Deserialize
        let key = Key::from_bytes(key_bytes)?; // Uses types.rs deserialization
        let value = Value::from_bytes(&final_value_bytes)?; // Uses types.rs deserialization

        Ok((key, value, frame.sequence))
    }

    async fn find_next_valid_frame<R: AsyncReadExt + Unpin>(
        reader: &mut R,
        start_position: u64,
    ) -> Result<u64> {
        let mut buffer = vec![0u8; 4096];
        let mut position = start_position;

        loop {
            let bytes_read = reader.read(&mut buffer).await?;
            if bytes_read == 0 {
                return Err(SGBDError::Serialization {
                    context: "No more valid frames found".to_string(),
                    format: SerializationFormat::Bincode,
                    backtrace: None,
                });
            }

            // Scan for magic number
            for i in 0..bytes_read.saturating_sub(4) {
                let magic =
                    u32::from_le_bytes([buffer[i], buffer[i + 1], buffer[i + 2], buffer[i + 3]]);

                if magic == FRAME_MAGIC {
                    return Ok(position + i as u64);
                }
            }

            position += bytes_read as u64;
        }
    }

    async fn list_segments(directory: &Path) -> Result<Vec<(u64, PathBuf)>> {
        let mut segments = Vec::new();
        let mut entries = fs::read_dir(directory).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "log") {
                if let Some(seq) = path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .and_then(|s| s.strip_prefix("wal-"))
                    .and_then(|s| s.parse::<u64>().ok())
                {
                    segments.push((seq, path));
                }
            }
        }

        Ok(segments)
    }

    async fn start_background_tasks(&self) {
        let mut tasks = self.background_tasks.lock().await;

        // Metrics collection task
        let metrics_task = {
            let metrics = self.metrics.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    Self::log_metrics(&metrics).await;
                }
            })
        };

        // Segment compaction task
        let compaction_task = {
            let directory = self.directory.clone();
            let config = self.config.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
                loop {
                    interval.tick().await;
                    if let Err(e) = Self::compact_segments(&directory, &config).await {
                        eprintln!("Compaction failed: {}", e);
                    }
                }
            })
        };

        tasks.push(metrics_task);
        tasks.push(compaction_task);
    }

    async fn log_metrics(metrics: &WalMetrics) {
        let write_throughput = metrics.write_bytes.load(Ordering::Relaxed) as f64 / 1024.0 / 1024.0; // MB
        let write_ops = metrics.write_entries.load(Ordering::Relaxed);
        let sync_count = metrics.sync_count.load(Ordering::Relaxed);
        let avg_sync_latency = if sync_count > 0 {
            metrics.sync_duration.load(Ordering::Relaxed) as f64 / sync_count as f64 / 1000.0
        // ms
        } else {
            0.0
        };

        println!(
            "WAL Metrics: {:.2}MB written, {} entries, {} syncs, {:.2}ms avg sync latency",
            write_throughput, write_ops, sync_count, avg_sync_latency
        );
    }

    async fn compact_segments(directory: &Path, config: &WalConfig) -> Result<()> {
        // Simple compaction: remove old segments beyond a threshold
        let mut segments = Self::list_segments(directory).await?;
        segments.sort_by_key(|(seq, _)| *seq);

        // Keep last 10 segments
        if segments.len() > 10 {
            let to_remove = segments.len() - 10;
            for (_, path) in segments.iter().take(to_remove) {
                fs::remove_file(path).await?;
            }
        }

        Ok(())
    }

    pub async fn checkpoint(&self) -> Result<()> {
        self.sync().await?;
        self.truncate_old_segments().await
    }

    pub async fn truncate_old_segments(&self) -> Result<()> {
        let mut segments = Self::list_segments(&self.directory).await?;
        segments.sort_by_key(|(seq, _)| *seq);

        // Keep current segment and previous one
        if segments.len() > 2 {
            let to_remove = segments.len() - 2;
            for (_, path) in segments.iter().take(to_remove) {
                fs::remove_file(path).await?;
            }
        }

        Ok(())
    }

    pub async fn wait_for_commit(&self) {
        self.commit_notify.notified().await;
    }

    pub async fn wait_for_commit_timeout(&self, timeout_duration: Duration) -> Result<()> {
        timeout(timeout_duration, self.commit_notify.notified())
            .await
            .map_err(|_| WalError::Timeout {
                operation: "wait_for_commit".to_string(),
            })?;
        Ok(())
    }

    pub async fn read_range(&self, start_seq: u64, end_seq: u64) -> Result<Vec<(Key, Value)>> {
        let _permit = self
            .reader_semaphore
            .acquire()
            .await
            .map_err(|_| WalError::ConcurrencyLimit)?;

        self.metrics.active_readers.fetch_add(1, Ordering::Relaxed);
        let result = self.read_range_internal(start_seq, end_seq).await;
        self.metrics.active_readers.fetch_sub(1, Ordering::Relaxed);

        result
    }

    async fn read_range_internal(&self, start_seq: u64, end_seq: u64) -> Result<Vec<(Key, Value)>> {
        let mut entries = Vec::new();
        let segments = Self::list_segments(&self.directory).await?;

        for (_, path) in segments {
            let segment_entries = Self::read_segment_range(&path, start_seq, end_seq).await?;
            entries.extend(segment_entries);
        }

        // Sort by sequence number
        entries.sort_by_key(|(_, _, seq)| *seq);

        // Remove sequence numbers and return
        Ok(entries.into_iter().map(|(k, v, _)| (k, v)).collect())
    }

    async fn read_segment_range(
        path: &Path,
        start_seq: u64,
        end_seq: u64,
    ) -> Result<Vec<(Key, Value, u64)>> {
        let mut file = File::open(path).await?;
        let file_size = file.metadata().await?.len();
        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();
        let mut position = 16u64; // Skip header

        reader.seek(tokio::io::SeekFrom::Start(position)).await?;

        while position < file_size {
            match Self::read_framed_entry(&mut reader, &mut position).await {
                Ok((key, value, sequence)) => {
                    if sequence >= start_seq && sequence <= end_seq {
                        entries.push((key, value, sequence));
                    }
                    if sequence > end_seq {
                        break; // Sequences are monotonic
                    }
                }
                Err(_) => break, // Stop on any error
            }
        }

        Ok(entries)
    }

    fn update_performance_metrics(&self, entry_count: usize, duration_micros: u64) {
        // Update average entry size
        let current_entries = self.metrics.write_entries.load(Ordering::Relaxed);
        let current_bytes = self.metrics.write_bytes.load(Ordering::Relaxed);

        if current_entries > 0 {
            let avg_size = current_bytes / current_entries;
            self.metrics
                .avg_entry_size
                .store(avg_size, Ordering::Relaxed);
        }
    }

    // Observability methods
    pub fn metrics(&self) -> WalMetricsSnapshot {
        WalMetricsSnapshot {
            write_bytes: self.metrics.write_bytes.load(Ordering::Relaxed),
            write_entries: self.metrics.write_entries.load(Ordering::Relaxed),
            write_batches: self.metrics.write_batches.load(Ordering::Relaxed),
            sync_count: self.metrics.sync_count.load(Ordering::Relaxed),
            sync_duration_total: self.metrics.sync_duration.load(Ordering::Relaxed),
            rotation_count: self.metrics.rotation_count.load(Ordering::Relaxed),
            active_segments: self.metrics.active_segments.load(Ordering::Relaxed),
            checksum_errors: self.metrics.checksum_errors.load(Ordering::Relaxed),
            frame_errors: self.metrics.frame_errors.load(Ordering::Relaxed),
            io_errors: self.metrics.io_errors.load(Ordering::Relaxed),
            active_readers: self.metrics.active_readers.load(Ordering::Relaxed),
            active_writers: self.metrics.active_writers.load(Ordering::Relaxed),
            compression_ratio: self.metrics.compression_ratio.load(Ordering::Relaxed) as f64
                / 1000.0,
            avg_entry_size: self.metrics.avg_entry_size.load(Ordering::Relaxed),
        }
    }

    pub fn avg_sync_latency_ms(&self) -> f64 {
        let count = self.metrics.sync_count.load(Ordering::Relaxed);
        if count == 0 {
            return 0.0;
        }
        let total = self.metrics.sync_duration.load(Ordering::Relaxed);
        total as f64 / count as f64 / 1000.0 // Convert to milliseconds
    }

    pub fn write_throughput_mbps(&self) -> f64 {
        let bytes = self.metrics.write_bytes.load(Ordering::Relaxed);
        bytes as f64 / 1024.0 / 1024.0 // Convert to MB
    }

    pub async fn health_check(&self) -> HealthStatus {
        let metrics = self.metrics();
        let mut status = HealthStatus {
            is_healthy: true,
            issues: Vec::new(),
            metrics: metrics.clone(),
        };

        // Check error rates
        if metrics.checksum_errors > 0 {
            status.is_healthy = false;
            status.issues.push(format!(
                "Checksum errors detected: {}",
                metrics.checksum_errors
            ));
        }

        if metrics.frame_errors > 0 {
            status.is_healthy = false;
            status
                .issues
                .push(format!("Frame errors detected: {}", metrics.frame_errors));
        }

        if metrics.io_errors > 0 {
            status.is_healthy = false;
            status
                .issues
                .push(format!("I/O errors detected: {}", metrics.io_errors));
        }

        // Check sync latency
        let avg_latency = self.avg_sync_latency_ms();
        if avg_latency > 100.0 {
            status
                .issues
                .push(format!("High sync latency: {:.2}ms", avg_latency));
        }

        // Check segment count
        if let Ok(segments) = Self::list_segments(&self.directory).await {
            if segments.len() > 100 {
                status
                    .issues
                    .push(format!("Too many segments: {}", segments.len()));
            }
        }

        status
    }
}

#[derive(Debug, Clone)]
pub struct WalMetricsSnapshot {
    pub write_bytes: u64,
    pub write_entries: u64,
    pub write_batches: u64,
    pub sync_count: u64,
    pub sync_duration_total: u64,
    pub rotation_count: u64,
    pub active_segments: u32,
    pub checksum_errors: u32,
    pub frame_errors: u32,
    pub io_errors: u32,
    pub active_readers: u32,
    pub active_writers: u32,
    pub compression_ratio: f64,
    pub avg_entry_size: u64,
}

#[derive(Debug)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub issues: Vec<String>,
    pub metrics: WalMetricsSnapshot,
}

// Drop implementation for clean shutdown
impl Drop for WriteAheadLog {
    fn drop(&mut self) {
        // Cancel background tasks
        if let Ok(mut tasks) = self.background_tasks.try_lock() {
            for task in tasks.drain(..) {
                task.abort();
            }
        }
    }
}

// Testing utilities
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_wal() -> (WriteAheadLog, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let wal = WriteAheadLog::open(temp_dir.path()).await.unwrap();
        (wal, temp_dir)
    }

    #[tokio::test]
    async fn test_basic_append_and_replay() {
        let (wal, temp_dir) = create_test_wal().await;

        // Use Key and Value constructors from types.rs
        let key = Key::new_uuid();
        let value = Value::Raw(Bytes::from("value1"));

        wal.append(&key, &value).await.unwrap();
        wal.sync().await.unwrap();

        let result = WriteAheadLog::replay(temp_dir.path()).await.unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].0.namespace, "test");
        assert_eq!(result.entries[0].1.data, Bytes::from("value1"));
    }

    #[tokio::test]
    async fn test_batch_append() {
        let (wal, temp_dir) = create_test_wal().await;

        let entries = vec![
            (
                Key::new("test".to_string(), b"key1".to_vec()),
                Value::new(Bytes::from("value1")),
            ),
            (
                Key::new("test".to_string(), b"key2".to_vec()),
                Value::new(Bytes::from("value2")),
            ),
            (
                Key::new("test".to_string(), b"key3".to_vec()),
                Value::new(Bytes::from("value3")),
            ),
        ];

        wal.append_batch(&entries).await.unwrap();
        wal.sync().await.unwrap();

        let result = WriteAheadLog::replay(temp_dir.path()).await.unwrap();
        assert_eq!(result.entries.len(), 3);
    }

    #[tokio::test]
    async fn test_segment_rotation() {
        let mut config = WalConfig::default();
        config.segment_size = 1024; // Small segments for testing

        let temp_dir = TempDir::new().unwrap();
        let wal = WriteAheadLog::open_with_config(temp_dir.path(), config)
            .await
            .unwrap();

        // Write enough data to trigger rotation
        for i in 0..100 {
            let key = Key::new("test".to_string(), format!("key{}", i).into_bytes());
            let value = Value::new(Bytes::from(vec![0u8; 100])); // 100 bytes each
            wal.append(&key, &value).await.unwrap();
        }

        wal.sync().await.unwrap();

        // Check that multiple segments were created
        let segments = WriteAheadLog::list_segments(temp_dir.path()).await.unwrap();
        assert!(segments.len() > 1);
    }

    #[tokio::test]
    async fn test_compression() {
        let mut config = WalConfig::default();
        config.compression_threshold = 100; // Low threshold for testing

        let temp_dir = TempDir::new().unwrap();
        let wal = WriteAheadLog::open_with_config(temp_dir.path(), config)
            .await
            .unwrap();

        let key = Key::new("test".to_string(), b"key1".to_vec());
        let large_data = vec![0u8; 2000]; // Should trigger compression
        let value = Value::new(Bytes::from(large_data));

        wal.append(&key, &value).await.unwrap();
        wal.sync().await.unwrap();

        let result = WriteAheadLog::replay(temp_dir.path()).await.unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].1.data.len(), 2000);
    }

    #[tokio::test]
    async fn test_crash_recovery() {
        let temp_dir = TempDir::new().unwrap();

        // Create and write to WAL
        {
            let wal = WriteAheadLog::open(temp_dir.path()).await.unwrap();

            for i in 0..10 {
                let key = Key::new("test".to_string(), format!("key{}", i).into_bytes());
                let value = Value::new(Bytes::from(format!("value{}", i)));
                wal.append(&key, &value).await.unwrap();
            }

            wal.sync().await.unwrap();
        } // WAL drops here

        // Reopen and verify recovery
        let wal = WriteAheadLog::open(temp_dir.path()).await.unwrap();
        let result = WriteAheadLog::replay(temp_dir.path()).await.unwrap();

        assert_eq!(result.entries.len(), 10);
        assert_eq!(result.recovery_stats.segments_processed, 1);
    }

    #[tokio::test]
    async fn test_metrics_and_health() {
        let (wal, _temp_dir) = create_test_wal().await;

        let key = Key::new("test".to_string(), b"key1".to_vec());
        let value = Value::new(Bytes::from("value1"));

        wal.append(&key, &value).await.unwrap();
        wal.sync().await.unwrap();

        let metrics = wal.metrics();
        assert_eq!(metrics.write_entries, 1);
        assert!(metrics.write_bytes > 0);
        assert_eq!(metrics.sync_count, 1);

        let health = wal.health_check().await;
        assert!(health.is_healthy);
        assert!(health.issues.is_empty());
    }
}

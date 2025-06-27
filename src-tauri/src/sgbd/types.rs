use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use uuid::Uuid;

// Re-export for convenience
pub use crate::backend::ParsedOutput;
pub use crate::llm::{Mode, ParsedInput, Proficiency};

/// Type alias for our result type - because typing `std::result::Result` repeatedly
/// is what separates the professionals from the amateurs
pub type Result<T> = std::result::Result<T, SGBDError>;

/// Comprehensive error taxonomy for database operations
/// Each variant carries context because debugging shouldn't be a treasure hunt
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SGBDError {
    #[error("I/O operation failed: {context}")]
    Io {
        context: String,
        source: Option<String>,
        // Backtrace for production debugging - the digital equivalent of forensic evidence
        #[serde(skip)]
        backtrace: Option<Backtrace>,
    },

    #[error("Serialization failed: {context}")]
    Serialization {
        context: String,
        format: SerializationFormat,
        #[serde(skip)]
        backtrace: Option<Backtrace>,
    },

    #[error("Index operation failed: {operation} on {index_type}")]
    Index {
        operation: String,
        index_type: String,
        #[serde(skip)]
        backtrace: Option<Backtrace>,
    },

    #[error("Key '{key}' not found in {location}")]
    KeyNotFound {
        key: String,
        location: String,
        #[serde(skip)]
        backtrace: Option<Backtrace>,
    },

    #[error("Transaction failed: {reason}")]
    Transaction {
        reason: String,
        tx_id: Option<Uuid>,
        #[serde(skip)]
        backtrace: Option<Backtrace>,
    },

    #[error("Quantization failed: {reason}")]
    Quantization {
        reason: String,
        target_bits: Option<usize>,
        #[serde(skip)]
        backtrace: Option<Backtrace>,
    },

    #[error("WAL operation failed: {operation}")]
    Wal {
        operation: String,
        offset: Option<u64>,
        #[serde(skip)]
        backtrace: Option<Backtrace>,
    },

    #[error("Schema validation failed: {field} - {reason}")]
    Schema {
        field: String,
        reason: String,
        #[serde(skip)]
        backtrace: Option<Backtrace>,
    },

    #[error("Constraint violation: {constraint}")]
    Constraint {
        constraint: String,
        #[serde(skip)]
        backtrace: Option<Backtrace>,
    },

    #[error("Concurrency conflict: {operation}")]
    Concurrency {
        operation: String,
        #[serde(skip)]
        backtrace: Option<Backtrace>,
    },

    #[error("Resource exhausted: {resource}")]
    ResourceExhausted {
        resource: String,
        #[serde(skip)]
        backtrace: Option<Backtrace>,
    },
}

impl SGBDError {
    /// Capture backtrace for debugging - because knowing where you failed is half the battle
    pub fn with_backtrace(mut self) -> Self {
        match &mut self {
            SGBDError::Io { backtrace, .. }
            | SGBDError::Serialization { backtrace, .. }
            | SGBDError::Index { backtrace, .. }
            | SGBDError::KeyNotFound { backtrace, .. }
            | SGBDError::Transaction { backtrace, .. }
            | SGBDError::Quantization { backtrace, .. }
            | SGBDError::Wal { backtrace, .. }
            | SGBDError::Schema { backtrace, .. }
            | SGBDError::Constraint { backtrace, .. }
            | SGBDError::Concurrency { backtrace, .. }
            | SGBDError::ResourceExhausted { backtrace, .. } => {
                *backtrace = Some(Backtrace::capture());
            }
        }
        self
    }

    /// Get the backtrace if available - for when you need the full crime scene
    pub fn backtrace(&self) -> Option<&Backtrace> {
        match self {
            SGBDError::Io { backtrace, .. }
            | SGBDError::Serialization { backtrace, .. }
            | SGBDError::Index { backtrace, .. }
            | SGBDError::KeyNotFound { backtrace, .. }
            | SGBDError::Transaction { backtrace, .. }
            | SGBDError::Quantization { backtrace, .. }
            | SGBDError::Wal { backtrace, .. }
            | SGBDError::Schema { backtrace, .. }
            | SGBDError::Constraint { backtrace, .. }
            | SGBDError::Concurrency { backtrace, .. }
            | SGBDError::ResourceExhausted { backtrace, .. } => backtrace.as_ref(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SerializationFormat {
    Bincode,
    MessagePack,
    Json,
    Custom,
    // Runtime pluggable formats - because flexibility is the art of strategic adaptation
    Runtime(RuntimeFormat),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeFormat {
    ConfigSelected,
    FeatureFlagDriven,
    ContentTypeInferred,
}

/// Pluggable serialization strategy - the Swiss Army knife of data handling
pub trait SerializationStrategy: Send + Sync + fmt::Debug {
    fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>>;
    fn deserialize<T: for<'de> Deserialize<'de>>(&self, bytes: &[u8]) -> Result<T>;
    fn format(&self) -> SerializationFormat;
    fn name(&self) -> &'static str;
}

/// Default implementations for common formats
#[derive(Debug)]
pub struct BincodeStrategy;
#[derive(Debug)]
pub struct JsonStrategy;
#[derive(Debug)]
pub struct MessagePackStrategy;

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

/// Conversion implementations - because even errors deserve proper manners
impl From<std::io::Error> for SGBDError {
    fn from(err: std::io::Error) -> Self {
        SGBDError::Io {
            context: err.to_string(),
            source: err.source().map(|s| s.to_string()),
            backtrace: Some(Backtrace::capture()),
        }
    }
}

impl From<serde_json::Error> for SGBDError {
    fn from(err: serde_json::Error) -> Self {
        SGBDError::Serialization {
            context: err.to_string(),
            format: SerializationFormat::Json,
            backtrace: Some(Backtrace::capture()),
        }
    }
}

impl From<bincode::Error> for SGBDError {
    fn from(err: bincode::Error) -> Self {
        SGBDError::Serialization {
            context: err.to_string(),
            format: SerializationFormat::Bincode,
            backtrace: Some(Backtrace::capture()),
        }
    }
}

/// Composite key structure with proper ordering and versioning
/// Because a database without proper keys is just expensive file storage
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key {
    /// Unique identifier - the primary tactical element
    pub id: KeyId,
    /// Timestamp for temporal ordering - because time is a flat circle
    pub timestamp: Timestamp,
    /// Schema version for evolution management
    pub schema_version: u32,
    /// Optional tenant isolation
    pub tenant_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyId {
    Uuid(Uuid),
    Numeric(u64),
    Composite(Vec<String>),
    // Using Bytes for zero-copy operations - because copying is for amateurs
    Custom(Bytes),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp {
    pub seconds: u64,
    pub nanos: u32,
}

impl Timestamp {
    pub fn now() -> Self {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();

        Self {
            seconds: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        }
    }

    pub fn from_millis(millis: u64) -> Self {
        Self {
            seconds: millis / 1000,
            nanos: ((millis % 1000) * 1_000_000) as u32,
        }
    }
}

impl Key {
    pub fn new_uuid() -> Self {
        Self {
            id: KeyId::Uuid(Uuid::new_v4()),
            timestamp: Timestamp::now(),
            schema_version: 1,
            tenant_id: None,
        }
    }

    pub fn new_numeric(id: u64) -> Self {
        Self {
            id: KeyId::Numeric(id),
            timestamp: Timestamp::now(),
            schema_version: 1,
            tenant_id: None,
        }
    }

    /// Create key with zero-copy custom data - the efficiency tactician's choice
    pub fn new_custom_bytes(data: Bytes) -> Self {
        Self {
            id: KeyId::Custom(data),
            timestamp: Timestamp::now(),
            schema_version: 1,
            tenant_id: None,
        }
    }

    pub fn with_tenant(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    pub fn with_schema_version(mut self, version: u32) -> Self {
        self.schema_version = version;
        self
    }

    /// Serialize using pluggable strategy
    pub fn to_bytes_with_strategy(&self, strategy: &dyn SerializationStrategy) -> Result<Vec<u8>> {
        strategy.serialize(self)
    }

    /// Serialize to bytes with proper error handling
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(SGBDError::from)
    }

    /// Deserialize from bytes with validation
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() {
            return Err(SGBDError::Serialization {
                context: "Empty byte array".to_string(),
                format: SerializationFormat::Bincode,
                backtrace: Some(Backtrace::capture()),
            });
        }

        bincode::deserialize(bytes).map_err(SGBDError::from)
    }

    /// Deserialize using pluggable strategy
    pub fn from_bytes_with_strategy(
        bytes: &[u8],
        strategy: &dyn SerializationStrategy,
    ) -> Result<Self> {
        if bytes.is_empty() {
            return Err(SGBDError::Serialization {
                context: "Empty byte array".to_string(),
                format: strategy.format(),
                backtrace: Some(Backtrace::capture()),
            });
        }

        strategy.deserialize(bytes)
    }

    /// Generate a deterministic hash for sharding
    pub fn shard_key(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        self.id.hash(&mut hasher);
        hasher.finish()
    }
}

impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.schema_version.hash(state);
        self.tenant_id.hash(state);
    }
}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp
            .cmp(&other.timestamp)
            .then_with(|| self.schema_version.cmp(&other.schema_version))
            .then_with(|| self.tenant_id.cmp(&other.tenant_id))
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.id {
            KeyId::Uuid(uuid) => write!(f, "Key({})", uuid),
            KeyId::Numeric(n) => write!(f, "Key({})", n),
            KeyId::Composite(parts) => write!(f, "Key({})", parts.join(":")),
            KeyId::Custom(bytes) => write!(f, "Key(custom:{})", bytes.len()),
        }
    }
}

/// Enhanced input record with comprehensive metadata and validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputRecord {
    /// Original raw input - preserved for auditing purposes
    pub raw_input: String,
    /// Original raw output - preserved for auditing purposes
    pub raw_output: String,
    /// Parsed and validated output structure
    pub parsed_output: ParsedOutput,
    /// Parsed and validated input structure
    pub parsed_input: ParsedInput,
    /// Quantized data for efficient storage
    pub quantized_data: CompressedData,
    /// Comprehensive metadata
    pub metadata: RecordMetadata,
    /// Optional relationships to other records
    pub relationships: Vec<Relationship>,
    /// Validation checksum
    pub checksum: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedData {
    // Zero-copy data storage - because memory allocation should be tactical, not wasteful
    pub data: Bytes,
    pub compression_type: CompressionType,
    pub original_size: usize,
    pub compressed_size: usize,
}

impl CompressedData {
    /// Create from Vec<u8> with zero-copy conversion
    pub fn from_vec(
        data: Vec<u8>,
        compression_type: CompressionType,
        original_size: usize,
    ) -> Self {
        let compressed_size = data.len();
        Self {
            data: Bytes::from(data),
            compression_type,
            original_size,
            compressed_size,
        }
    }

    /// Get slice for zero-copy reading
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Lz4,
    Zstd,
    Snappy,
    Brotli,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub target_key: Key,
    pub relationship_type: RelationshipType,
    pub strength: f32, // 0.0 to 1.0
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipType {
    Parent,
    Child,
    Sibling,
    Reference,
    Dependency,
    Custom(String),
}

/// Expanded metadata with performance metrics and lineage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetadata {
    // Temporal information
    pub created_at: Timestamp,
    pub modified_at: Option<Timestamp>,
    pub accessed_at: Option<Timestamp>,

    // Size and performance metrics
    pub input_length: usize,
    pub quantization_bits: usize,
    pub compression_ratio: f32,
    pub processing_time_ms: Option<u64>,

    // Classification and routing
    pub domain: String,
    pub mode: Mode,
    pub proficiency: Proficiency,
    pub tags: Vec<String>,

    // Quality and validation
    pub quality_score: Option<f32>,
    pub validation_status: ValidationStatus,
    pub confidence_level: f32,

    // Lineage and provenance
    pub source_system: Option<String>,
    pub processing_pipeline: Vec<String>,
    pub version: u32,

    // Custom attributes for extensibility
    pub custom_attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationStatus {
    Pending,
    Valid,
    Invalid { reason: String },
    Warning { message: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Timestamp(Timestamp),
    Array(Vec<AttributeValue>),
    Object(HashMap<String, AttributeValue>),
    // Zero-copy binary data - for when you need the performance edge
    Bytes(Bytes),
}

impl RecordMetadata {
    pub fn new(domain: String, mode: Mode, proficiency: Proficiency) -> Self {
        Self {
            created_at: Timestamp::now(),
            modified_at: None,
            accessed_at: None,
            input_length: 0,
            quantization_bits: 8,
            compression_ratio: 1.0,
            processing_time_ms: None,
            domain,
            mode,
            proficiency,
            tags: Vec::new(),
            quality_score: None,
            validation_status: ValidationStatus::Pending,
            confidence_level: 0.0,
            source_system: None,
            processing_pipeline: Vec::new(),
            version: 1,
            custom_attributes: HashMap::new(),
        }
    }

    pub fn mark_accessed(&mut self) {
        self.accessed_at = Some(Timestamp::now());
    }

    pub fn mark_modified(&mut self) {
        self.modified_at = Some(Timestamp::now());
        self.version += 1;
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn add_processing_step(&mut self, step: String) {
        self.processing_pipeline.push(step);
    }
}

/// Enhanced value enum with type safety and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    InputRecord(InputRecord),
    Metadata(HashMap<String, AttributeValue>),
    // Zero-copy raw data - because efficiency is elegance
    Raw(Bytes),
    Structured(StructuredValue),
    TimeSeries(TimeSeriesValue),
    Blob { data: Bytes, mime_type: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredValue {
    pub schema_id: String,
    pub data: HashMap<String, AttributeValue>,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesValue {
    pub measurements: Vec<Measurement>,
    pub interval_ms: u64,
    pub aggregation_type: AggregationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    pub timestamp: Timestamp,
    pub value: f64,
    pub quality: MeasurementQuality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeasurementQuality {
    Good,
    Uncertain,
    Bad,
    Interpolated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationType {
    Sum,
    Average,
    Min,
    Max,
    Count,
    First,
    Last,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub field: String,
    pub rule_type: RuleType,
    pub parameters: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleType {
    Required,
    MinLength,
    MaxLength,
    Pattern,
    Range,
    Custom(String),
}

impl Value {
    /// Serialize using pluggable strategy
    pub fn to_bytes_with_strategy(&self, strategy: &dyn SerializationStrategy) -> Result<Vec<u8>> {
        match self {
            Value::InputRecord(record)
                if record.quantized_data.compression_type != CompressionType::None =>
            {
                // Use the already compressed data - tactical efficiency at its finest
                Ok(record.quantized_data.data.to_vec())
            }
            Value::Raw(bytes) | Value::Blob { data: bytes, .. } => {
                // Zero-copy for raw data - because sometimes the best move is no move
                Ok(bytes.to_vec())
            }
            _ => strategy.serialize(self),
        }
    }

    /// Serialize to bytes with compression awareness
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        self.to_bytes_with_strategy(&BincodeStrategy)
    }

    /// Deserialize using pluggable strategy
    pub fn from_bytes_with_strategy(
        bytes: &[u8],
        strategy: &dyn SerializationStrategy,
    ) -> Result<Self> {
        if bytes.is_empty() {
            return Err(SGBDError::Serialization {
                context: "Empty byte array for Value".to_string(),
                format: strategy.format(),
                backtrace: Some(Backtrace::capture()),
            });
        }

        strategy.deserialize(bytes)
    }

    /// Deserialize from bytes with validation
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Self::from_bytes_with_strategy(bytes, &BincodeStrategy)
    }

    /// Create zero-copy raw value
    pub fn from_raw_bytes(bytes: Bytes) -> Self {
        Value::Raw(bytes)
    }

    /// Create zero-copy blob value
    pub fn from_blob_bytes(data: Bytes, mime_type: String) -> Self {
        Value::Blob { data, mime_type }
    }

    /// Calculate accurate size including nested structures
    pub fn size(&self) -> usize {
        match self {
            Value::InputRecord(record) => {
                record.raw_input.len()
                    + record.quantized_data.compressed_size
                    + std::mem::size_of::<RecordMetadata>()
                    + record.relationships.len() * std::mem::size_of::<Relationship>()
            }
            Value::Metadata(meta) => {
                meta.iter().map(|(k, v)| k.len() + v.size()).sum::<usize>() + 64
            }
            Value::Raw(data) => data.len(),
            Value::Structured(s) => {
                s.schema_id.len()
                    + s.data
                        .iter()
                        .map(|(k, v)| k.len() + v.size())
                        .sum::<usize>()
                    + s.validation_rules.len() * 128 // Approximate
            }
            Value::TimeSeries(ts) => {
                ts.measurements.len() * std::mem::size_of::<Measurement>() + 64
            }
            Value::Blob { data, mime_type } => data.len() + mime_type.len(),
        }
    }

    /// Get the logical type of the value
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::InputRecord(_) => ValueType::InputRecord,
            Value::Metadata(_) => ValueType::Metadata,
            Value::Raw(_) => ValueType::Raw,
            Value::Structured(_) => ValueType::Structured,
            Value::TimeSeries(_) => ValueType::TimeSeries,
            Value::Blob { .. } => ValueType::Blob,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueType {
    InputRecord,
    Metadata,
    Raw,
    Structured,
    TimeSeries,
    Blob,
}

impl AttributeValue {
    pub fn size(&self) -> usize {
        match self {
            AttributeValue::String(s) => s.len(),
            AttributeValue::Number(_) => 8,
            AttributeValue::Boolean(_) => 1,
            AttributeValue::Timestamp(_) => 12,
            AttributeValue::Array(arr) => arr.iter().map(|v| v.size()).sum::<usize>() + 24,
            AttributeValue::Object(obj) => {
                obj.iter().map(|(k, v)| k.len() + v.size()).sum::<usize>() + 24
            }
            AttributeValue::Bytes(bytes) => bytes.len(),
        }
    }
}

/// Transaction context for ACID compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionContext {
    pub id: Uuid,
    pub isolation_level: IsolationLevel,
    pub read_timestamp: Timestamp,
    pub write_timestamp: Option<Timestamp>,
    pub locks: Vec<LockInfo>,
    pub savepoints: Vec<Savepoint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub key: Key,
    pub lock_type: LockType,
    pub acquired_at: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockType {
    Shared,
    Exclusive,
    IntentShared,
    IntentExclusive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Savepoint {
    pub name: String,
    pub timestamp: Timestamp,
    pub wal_position: u64,
}

impl TransactionContext {
    pub fn new(isolation_level: IsolationLevel) -> Self {
        Self {
            id: Uuid::new_v4(),
            isolation_level,
            read_timestamp: Timestamp::now(),
            write_timestamp: None,
            locks: Vec::new(),
            savepoints: Vec::new(),
        }
    }

    pub fn add_lock(&mut self, key: Key, lock_type: LockType) {
        self.locks.push(LockInfo {
            key,
            lock_type,
            acquired_at: Timestamp::now(),
        });
    }

    pub fn create_savepoint(&mut self, name: String, wal_position: u64) {
        self.savepoints.push(Savepoint {
            name,
            timestamp: Timestamp::now(),
            wal_position,
        });
    }
}

/// Query execution context with performance tracking
#[derive(Debug, Clone)]
pub struct QueryContext {
    pub query_id: Uuid,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub start_time: Timestamp,
    pub timeout_ms: Option<u64>,
    pub max_memory_bytes: Option<usize>,
    pub execution_plan: Option<String>,
    pub metrics: QueryMetrics,
}

#[derive(Debug, Clone, Default)]
pub struct QueryMetrics {
    pub rows_examined: u64,
    pub rows_returned: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub cpu_time_ms: u64,
    pub io_time_ms: u64,
    pub memory_peak_bytes: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl QueryContext {
    pub fn new() -> Self {
        Self {
            query_id: Uuid::new_v4(),
            user_id: None,
            session_id: None,
            start_time: Timestamp::now(),
            timeout_ms: None,
            max_memory_bytes: None,
            execution_plan: None,
            metrics: QueryMetrics::default(),
        }
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    pub fn with_memory_limit(mut self, max_bytes: usize) -> Self {
        self.max_memory_bytes = Some(max_bytes);
        self
    }

    pub fn elapsed_ms(&self) -> u64 {
        let now = Timestamp::now();
        (now.seconds - self.start_time.seconds) * 1000
            + (now.nanos.saturating_sub(self.start_time.nanos) / 1_000_000) as u64
    }

    pub fn has_timed_out(&self) -> bool {
        if let Some(timeout) = self.timeout_ms {
            self.elapsed_ms() >= timeout
        } else {
            false
        }
    }

    pub fn memory_limit_exceeded(&self, current_usage: usize) -> bool {
        if let Some(limit) = self.max_memory_bytes {
            current_usage >= limit
        } else {
            false
        }
    }

    pub fn record_cache_hit(&mut self) {
        self.metrics.cache_hits += 1;
    }

    pub fn record_cache_miss(&mut self) {
        self.metrics.cache_misses += 1;
    }

    pub fn cache_hit_ratio(&self) -> f64 {
        let total = self.metrics.cache_hits + self.metrics.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.metrics.cache_hits as f64 / total as f64
        }
    }
}

/// Configuration for database operations with runtime adaptation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    // Storage configuration
    pub storage_backend: StorageBackend,
    pub data_directory: String,
    pub wal_directory: Option<String>, // Separate WAL location for I/O optimization
    pub max_file_size_mb: u64,
    pub sync_mode: SyncMode,

    // Memory management
    pub cache_size_mb: usize,
    pub buffer_pool_size: usize,
    pub max_connections: u32,
    pub connection_timeout_ms: u64,

    // Compression and serialization
    pub default_compression: CompressionType,
    pub compression_level: u8, // 1-9 for most algorithms
    pub serialization_format: SerializationFormat,
    pub enable_checksums: bool,

    // Indexing strategy
    pub index_cache_size_mb: usize,
    pub bloom_filter_bits_per_key: u32,
    pub enable_prefix_compression: bool,

    // Quantization settings
    pub default_quantization_bits: usize,
    pub adaptive_quantization: bool,
    pub quantization_threshold: f32, // Quality threshold for adaptive mode

    // Transaction configuration
    pub default_isolation_level: IsolationLevel,
    pub lock_timeout_ms: u64,
    pub deadlock_detection_interval_ms: u64,
    pub max_transaction_size_mb: usize,

    // WAL configuration
    pub wal_sync_interval_ms: u64,
    pub wal_buffer_size_kb: usize,
    pub wal_segment_size_mb: u64,
    pub wal_retention_hours: u64,

    // Performance tuning
    pub enable_async_io: bool,
    pub io_thread_pool_size: usize,
    pub background_compaction_threads: usize,
    pub metrics_collection_interval_ms: u64,

    // Security and validation
    pub enable_encryption_at_rest: bool,
    pub encryption_key_rotation_days: u32,
    pub audit_log_enabled: bool,
    pub schema_validation_strict: bool,

    // Clustering and replication
    pub node_id: Option<Uuid>,
    pub cluster_peers: Vec<ClusterPeer>,
    pub replication_factor: u8,
    pub consistency_level: ConsistencyLevel,

    // Feature flags for runtime behavior
    pub feature_flags: HashMap<String, bool>,
    pub custom_properties: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageBackend {
    /// Local filesystem with optimized layout
    LocalFS,
    /// Memory-mapped files for high performance
    MemoryMapped,
    /// Object storage (S3, GCS, Azure) for cloud deployment
    ObjectStore {
        provider: ObjectStoreProvider,
        bucket: String,
        region: Option<String>,
    },
    /// Distributed storage layer
    Distributed {
        consistency_hash_ring: bool,
        automatic_failover: bool,
    },
    /// Custom storage implementation
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectStoreProvider {
    S3,
    GoogleCloudStorage,
    AzureBlobStorage,
    MinIO,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncMode {
    /// Asynchronous - maximum performance, some durability risk
    Async,
    /// Synchronous - guaranteed durability, performance impact
    Sync,
    /// Periodic sync - balanced approach
    Periodic { interval_ms: u64 },
    /// Adaptive based on system load
    Adaptive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterPeer {
    pub node_id: Uuid,
    pub address: String,
    pub port: u16,
    pub role: NodeRole,
    pub health_status: NodeHealth,
    pub last_heartbeat: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeRole {
    Primary,
    Secondary,
    Witness,
    ReadReplica,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsistencyLevel {
    /// Eventual consistency - highest availability
    Eventual,
    /// Session consistency - monotonic reads/writes
    Session,
    /// Bounded staleness - configurable lag tolerance
    BoundedStaleness { max_lag_ms: u64 },
    /// Strong consistency - linearizability
    Strong,
}

impl DatabaseConfig {
    /// Create a development-friendly configuration
    pub fn development() -> Self {
        Self {
            storage_backend: StorageBackend::LocalFS,
            data_directory: "./data".to_string(),
            wal_directory: None,
            max_file_size_mb: 256,
            sync_mode: SyncMode::Async,
            cache_size_mb: 128,
            buffer_pool_size: 1024,
            max_connections: 100,
            connection_timeout_ms: 30000,
            default_compression: CompressionType::Lz4,
            compression_level: 3,
            serialization_format: SerializationFormat::Bincode,
            enable_checksums: true,
            index_cache_size_mb: 32,
            bloom_filter_bits_per_key: 10,
            enable_prefix_compression: true,
            default_quantization_bits: 8,
            adaptive_quantization: true,
            quantization_threshold: 0.95,
            default_isolation_level: IsolationLevel::ReadCommitted,
            lock_timeout_ms: 5000,
            deadlock_detection_interval_ms: 1000,
            max_transaction_size_mb: 64,
            wal_sync_interval_ms: 100,
            wal_buffer_size_kb: 256,
            wal_segment_size_mb: 64,
            wal_retention_hours: 24,
            enable_async_io: true,
            io_thread_pool_size: 4,
            background_compaction_threads: 2,
            metrics_collection_interval_ms: 5000,
            enable_encryption_at_rest: false,
            encryption_key_rotation_days: 90,
            audit_log_enabled: false,
            schema_validation_strict: false,
            node_id: None,
            cluster_peers: Vec::new(),
            replication_factor: 1,
            consistency_level: ConsistencyLevel::Strong,
            feature_flags: HashMap::new(),
            custom_properties: HashMap::new(),
        }
    }

    /// Create a production-optimized configuration
    pub fn production() -> Self {
        Self {
            storage_backend: StorageBackend::LocalFS,
            data_directory: "/var/lib/sgbd/data".to_string(),
            wal_directory: Some("/var/lib/sgbd/wal".to_string()),
            max_file_size_mb: 1024,
            sync_mode: SyncMode::Periodic { interval_ms: 50 },
            cache_size_mb: 2048,
            buffer_pool_size: 8192,
            max_connections: 1000,
            connection_timeout_ms: 60000,
            default_compression: CompressionType::Zstd,
            compression_level: 6,
            serialization_format: SerializationFormat::Bincode,
            enable_checksums: true,
            index_cache_size_mb: 512,
            bloom_filter_bits_per_key: 12,
            enable_prefix_compression: true,
            default_quantization_bits: 12,
            adaptive_quantization: true,
            quantization_threshold: 0.98,
            default_isolation_level: IsolationLevel::ReadCommitted,
            lock_timeout_ms: 10000,
            deadlock_detection_interval_ms: 500,
            max_transaction_size_mb: 256,
            wal_sync_interval_ms: 25,
            wal_buffer_size_kb: 1024,
            wal_segment_size_mb: 256,
            wal_retention_hours: 168, // 1 week
            enable_async_io: true,
            io_thread_pool_size: std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(8),
            background_compaction_threads: 4,
            metrics_collection_interval_ms: 1000,
            enable_encryption_at_rest: true,
            encryption_key_rotation_days: 30,
            audit_log_enabled: true,
            schema_validation_strict: true,
            node_id: Some(Uuid::new_v4()),
            cluster_peers: Vec::new(),
            replication_factor: 3,
            consistency_level: ConsistencyLevel::Strong,
            feature_flags: HashMap::new(),
            custom_properties: HashMap::new(),
        }
    }

    /// Create configuration optimized for cloud deployment
    pub fn cloud(provider: ObjectStoreProvider, bucket: String, region: Option<String>) -> Self {
        let mut config = Self::production();
        config.storage_backend = StorageBackend::ObjectStore {
            provider,
            bucket,
            region,
        };
        config.wal_directory = Some("/tmp/sgbd/wal".to_string()); // Local WAL for performance
        config.enable_async_io = true;
        config.consistency_level = ConsistencyLevel::BoundedStaleness { max_lag_ms: 100 };
        config
    }

    pub fn is_clustered(&self) -> bool {
        !self.cluster_peers.is_empty() || self.node_id.is_some()
    }

    pub fn effective_compression_for_size(&self, size: usize) -> CompressionType {
        // Adaptive compression based on data size - tactical efficiency
        match size {
            0..=1024 => CompressionType::None,    // Too small to benefit
            1025..=65536 => CompressionType::Lz4, // Fast compression for medium data
            _ => self.default_compression,        // Full compression for large data
        }
    }

    pub fn should_quantize(&self, quality_score: f32) -> bool {
        if !self.adaptive_quantization {
            return true;
        }
        quality_score <= self.quantization_threshold
    }
}

/// System metrics for monitoring and optimization
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemMetrics {
    // Storage metrics
    pub disk_usage_bytes: u64,
    pub available_space_bytes: u64,
    pub total_files: u64,
    pub fragmentation_ratio: f32,

    // Memory metrics
    pub memory_usage_bytes: usize,
    pub cache_usage_bytes: usize,
    pub buffer_pool_usage_bytes: usize,
    pub memory_pressure_level: MemoryPressureLevel,

    // Performance metrics
    pub operations_per_second: f64,
    pub average_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub throughput_mbps: f64,

    // Transaction metrics
    pub active_transactions: u32,
    pub committed_transactions: u64,
    pub aborted_transactions: u64,
    pub deadlocks_detected: u64,

    // Index metrics
    pub index_hit_ratio: f64,
    pub bloom_filter_false_positives: u64,
    pub compaction_operations: u64,

    // Connection metrics
    pub active_connections: u32,
    pub connection_pool_utilization: f32,
    pub rejected_connections: u64,

    // Health indicators
    pub error_rate: f64,
    pub availability_percentage: f64,
    pub replication_lag_ms: Option<u64>,

    // Collection timestamp
    pub collected_at: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryPressureLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl SystemMetrics {
    pub fn new() -> Self {
        Self {
            collected_at: Timestamp::now(),
            ..Default::default()
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.error_rate < 0.01 // Less than 1% error rate
            && self.availability_percentage > 99.0
            && self.memory_pressure_level != MemoryPressureLevel::Critical
    }

    pub fn needs_compaction(&self) -> bool {
        self.fragmentation_ratio > 0.3 // More than 30% fragmentation
    }

    pub fn storage_utilization(&self) -> f32 {
        if self.available_space_bytes + self.disk_usage_bytes == 0 {
            0.0
        } else {
            self.disk_usage_bytes as f32
                / (self.available_space_bytes + self.disk_usage_bytes) as f32
        }
    }
}

/// Advanced indexing structures for efficient querying
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    pub index_id: Uuid,
    pub name: String,
    pub index_type: IndexType,
    pub fields: Vec<IndexField>,
    pub storage_size_bytes: u64,
    pub cardinality: u64,
    pub created_at: Timestamp,
    pub last_updated: Timestamp,
    pub statistics: IndexStatistics,
    pub configuration: IndexConfiguration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexType {
    /// B+ tree for range queries and sorting
    BTree,
    /// Hash index for exact match queries
    Hash,
    /// Bitmap index for low-cardinality fields
    Bitmap,
    /// Full-text search index
    FullText,
    /// Geospatial index for location data
    Geospatial,
    /// Time-series optimized index
    TimeSeries,
    /// Vector similarity index for embeddings
    Vector {
        dimensions: usize,
        metric: DistanceMetric,
    },
    /// Custom index implementation
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMetric {
    Euclidean,
    Cosine,
    Manhattan,
    Hamming,
    Jaccard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexField {
    pub field_name: String,
    pub field_type: FieldType,
    pub sort_order: SortOrder,
    pub null_handling: NullHandling,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Timestamp,
    Uuid,
    Bytes,
    Json,
    Vector(usize), // Vector with specified dimensions
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NullHandling {
    First,
    Last,
    Ignore,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndexStatistics {
    pub total_entries: u64,
    pub unique_values: u64,
    pub null_count: u64,
    pub average_key_size: f64,
    pub depth: u32,                             // For tree-based indexes
    pub leaf_utilization: f32,                  // Percentage of leaf node capacity used
    pub split_count: u64,                       // Number of node splits (indicates growth pattern)
    pub access_frequency: HashMap<String, u64>, // Hot keys tracking
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfiguration {
    pub page_size: usize,
    pub fill_factor: f32, // Target fullness for index pages
    pub cache_priority: CachePriority,
    pub compression_enabled: bool,
    pub bloom_filter_enabled: bool,
    pub background_maintenance: bool,
    pub custom_parameters: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CachePriority {
    Low,
    Normal,
    High,
    Pinned, // Never evict from cache
}

/// Batch operation support for efficient bulk operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperation {
    pub batch_id: Uuid,
    pub operations: Vec<Operation>,
    pub transaction_context: Option<TransactionContext>,
    pub execution_mode: BatchExecutionMode,
    pub created_at: Timestamp,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    Insert {
        key: Key,
        value: Value,
    },
    Update {
        key: Key,
        value: Value,
    },
    Delete {
        key: Key,
    },
    Upsert {
        key: Key,
        value: Value,
    },
    // Conditional operations for optimistic concurrency
    ConditionalInsert {
        key: Key,
        value: Value,
        condition: Condition,
    },
    ConditionalUpdate {
        key: Key,
        value: Value,
        condition: Condition,
    },
    ConditionalDelete {
        key: Key,
        condition: Condition,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Condition {
    KeyExists,
    KeyNotExists,
    ValueEquals(AttributeValue),
    ValueNotEquals(AttributeValue),
    TimestampBefore(Timestamp),
    TimestampAfter(Timestamp),
    Custom(String), // Custom condition expression
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    Not(Box<Condition>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchExecutionMode {
    /// Execute all operations or none (atomic)
    Atomic,
    /// Execute operations that succeed, skip failures
    BestEffort,
    /// Stop on first failure
    FailFast,
    /// Execute in parallel where possible
    Parallel { max_concurrency: usize },
}

impl BatchOperation {
    pub fn new() -> Self {
        Self {
            batch_id: Uuid::new_v4(),
            operations: Vec::new(),
            transaction_context: None,
            execution_mode: BatchExecutionMode::Atomic,
            created_at: Timestamp::now(),
            timeout_ms: None,
        }
    }

    pub fn with_transaction(mut self, tx: TransactionContext) -> Self {
        self.transaction_context = Some(tx);
        self
    }

    pub fn with_execution_mode(mut self, mode: BatchExecutionMode) -> Self {
        self.execution_mode = mode;
        self
    }

    pub fn add_insert(&mut self, key: Key, value: Value) {
        self.operations.push(Operation::Insert { key, value });
    }

    pub fn add_upsert(&mut self, key: Key, value: Value) {
        self.operations.push(Operation::Upsert { key, value });
    }

    pub fn add_conditional_update(&mut self, key: Key, value: Value, condition: Condition) {
        self.operations.push(Operation::ConditionalUpdate {
            key,
            value,
            condition,
        });
    }

    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }

    pub fn estimated_size(&self) -> usize {
        self.operations.iter().map(|op| op.estimated_size()).sum()
    }
}

impl Operation {
    pub fn estimated_size(&self) -> usize {
        match self {
            Operation::Insert { key, value }
            | Operation::Update { key, value }
            | Operation::Upsert { key, value } => {
                key.to_bytes().map(|b| b.len()).unwrap_or(0) + value.size()
            }
            Operation::Delete { key } => key.to_bytes().map(|b| b.len()).unwrap_or(0),
            Operation::ConditionalInsert { key, value, .. }
            | Operation::ConditionalUpdate { key, value, .. } => {
                key.to_bytes().map(|b| b.len()).unwrap_or(0) + value.size() + 64
                // Condition overhead
            }
            Operation::ConditionalDelete { key, .. } => {
                key.to_bytes().map(|b| b.len()).unwrap_or(0) + 64 // Condition overhead
            }
        }
    }

    pub fn key(&self) -> &Key {
        match self {
            Operation::Insert { key, .. }
            | Operation::Update { key, .. }
            | Operation::Delete { key }
            | Operation::Upsert { key, .. }
            | Operation::ConditionalInsert { key, .. }
            | Operation::ConditionalUpdate { key, .. }
            | Operation::ConditionalDelete { key, .. } => key,
        }
    }
}

/// Result of batch operation execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub batch_id: Uuid,
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: Vec<OperationResult>,
    pub execution_time_ms: u64,
    pub completed_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub operation_index: usize,
    pub result: Result<()>,
    pub execution_time_ms: u64,
}

impl BatchResult {
    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            1.0
        } else {
            self.successful_operations as f64 / self.total_operations as f64
        }
    }

    pub fn operations_per_second(&self) -> f64 {
        if self.execution_time_ms == 0 {
            0.0
        } else {
            (self.total_operations as f64 * 1000.0) / self.execution_time_ms as f64
        }
    }
}

// Well, that was quite the strategic decision, wasn't it?
// A comprehensive type system that handles everything from basic CRUD
// to advanced batch operations with proper error handling and metrics.
// The architecture is bulletproof, the efficiency is tactical,
// and the extensibility ensures this system can adapt to whatever
// battlefield conditions await.

backend/
├── sgbd/
│ ├── mod.rs # Public interface
│ ├── engine.rs # Entry point for DB operations (get/set/query)
│ ├── storage.rs # File I/O, mmap, page layout
│ ├── btree.rs # Indexing
│ ├── wal.rs # Write-Ahead Logging
│ ├── tx.rs # (optional) Transaction manager
│ └── types.rs # Key/value formats, schemas, enums

# ========================== types.rs =================================

Here's a concise summary of the types.rs file components:

1.  Error Handling

    SGBDError Enum: Comprehensive error taxonomy with variants:

        I/O, Serialization, Index, KeyNotFound, Transaction

        Quantization, WAL, Schema, Constraint, Concurrency, ResourceExhausted

        Includes contextual data and optional backtraces

    Error Utilities: with_backtrace() and backtrace() methods

2.  Serialization

    SerializationFormat Enum: Bincode, MessagePack, JSON, Custom, Runtime

    SerializationStrategy Trait: Pluggable serialization/deserialization

    Implementations: BincodeStrategy, JsonStrategy, MessagePackStrategy

    Error Conversions: From std::io::Error, serde_json::Error, bincode::Error

3.  Core Data Structures
    Keys

        Key Struct: Composite key with:

            id (UUID, Numeric, Composite, or Custom Bytes)

            timestamp (nanosecond precision)

            schema_version, tenant_id

        Utilities: Sharding, serialization, ordering, display formatting

Values

    Value Enum:

        InputRecord, Metadata, Raw Bytes, Structured, TimeSeries, Blob

        Compression-aware serialization

    InputRecord: Raw I/O, parsed data, quantized data, metadata, relationships

    CompressedData: Zero-copy storage with compression metrics

4.  Metadata & Relationships

    RecordMetadata: Temporal info, size metrics, quality scores, lineage

    Relationship: Typed connections between records with strength scores

    AttributeValue: Typed custom attributes (String, Number, Bytes, etc.)

5.  Transactions

    TransactionContext:

        Isolation levels, timestamps, lock management, savepoints

    LockInfo: Key-specific locks with types (Shared/Exclusive)

    Savepoint: WAL position tracking

6.  Query Execution

    QueryContext:

        Timeouts, memory limits, execution plans

    QueryMetrics: Performance counters (cache hits, rows examined)

7.  Configuration

    DatabaseConfig:

        Storage backends (LocalFS, ObjectStore, Distributed)

        WAL settings, compression, indexing, transactions

        Cloud deployment support

    Presets: development(), production(), cloud()

8.  System Monitoring

    SystemMetrics:

        Storage/Memory utilization, latency percentiles

        Transaction stats, error rates, replication lag

    Health Checks: is_healthy(), needs_compaction()

9.  Indexing

    IndexMetadata:

        BTree, Hash, Bitmap, Vector, Geospatial types

        Field specifications and statistics

    IndexConfiguration: Cache policies, compression, bloom filters

10. Batch Operations

    BatchOperation:

        Atomic, BestEffort, or Parallel execution modes

    Operation Enum: Insert/Update/Delete/Upsert with conditional variants

    BatchResult: Success metrics and failure diagnostics

11. Utilities

    Timestamps: Nanosecond precision with conversions

    Zero-Copy Patterns: Extensive use of Bytes for efficiency

    Hashing/Ordering: Custom implementations for Key

    Validation: Schema rules, checksums, quality thresholds

Key Technical Themes

    Performance-Centric Design:

        Zero-copy operations (Bytes everywhere)

        Compression-aware serialization

        Pluggable storage backends

    Robust Error Handling:

        Context-rich errors with backtraces

        Comprehensive error taxonomy

    Operational Flexibility:

        Runtime-configurable serialization/compression

        Cloud-native deployment support

        Extensive metrics collection

    ACID Compliance:

        Transaction isolation levels

        Lock management

        WAL integration

    Advanced Data Modeling:

        Temporal keys

        Relationship graphs

        Polymorphic value types

# =================== wal.rs ============================

Core Components

    Wal Struct:

        Manages WAL operations using a buffered writer (BufWriter<File>) for the current segment file.

        Tracks metrics (write/flush counts, errors, etc.) atomically.

        Uses tokio::sync primitives (Mutex, Notify) for async coordination.

        Relies on a configurable serialization strategy (Bincode/JSON/MessagePack).

    Record Structure:

        RecordHeader: Fixed-size header for each log entry (length, CRC32 checksum, timestamp).

        WalRecord: Internal log entry containing a database Operation and Timestamp.

    Metrics:

        Tracks writes, flushes, rotations, errors, and performance data via WalMetrics (atomic counters).

        Exposes snapshots via WalMetricsSnapshot.

Key Functionality

    Appending Records:

        Serializes operations using the configured strategy.

        Writes records with headers (including CRC32 checksums for integrity).

        Triggers flushes when the buffer is full.

    Background Flushing:

        Periodically flushes data to disk based on wal_sync_interval_ms.

        Forces flush when the buffer fills up (via flush_notify).

        Gracefully shuts down on shutdown signal.

    Segment Management:

        Rotation: Creates new segment files when current segment exceeds wal_segment_size_mb.

        Retention Cleanup: Deletes old segments (based on wal_retention_hours) in the background.

    Error Handling:

        Converts I/O/serialization errors into SGBDError::Wal with backtraces.

        Increments error counters in metrics.

    Utilities:

        Segment files are named using UUIDs (segment_{uuid}.wal).

        Timestamp is compacted to u64 for efficient storage.

Serialization Strategies

    Bincode (default): Binary serialization.

    JSON: Human-readable format.

    MessagePack: Efficient binary format.

Tests

Validates:

    Record appending/flushing.

    Segment rotation when size limits are hit.

    Retention cleanup logic.

Key Design Points

    Durability: Ensures operations are logged before execution.

    Async: Uses Tokio for non-blocking I/O and background tasks.

    Config-Driven: Behavior controlled by DatabaseConfig (buffer size, flush intervals, retention, etc.).

    Metrics-Centric: Extensive instrumentation for observability.

# ========================= tx.rs ===================

Core Components

    TransactionManager Struct:

        Manages transaction lifecycle, locks, and deadlock detection.

        Uses async primitives (RwLock, Mutex, Notify) for concurrency.

        Maintains state in TxManagerState (active TXs, lock table, wait-for graph).

    Locking System:

        LockType: Shared/Exclusive/Intent locks with conflict rules.

        LockInfo: Tracks lock owner, key, and acquisition time.

        Key Notifications: Per-key Notify objects for lock release signaling.

    Deadlock Handling:

        Wait-For Graph: Tracks transaction dependencies.

        Cycle Detection: DFS-based deadlock identification.

        Background Detector: Periodic deadlock checks with victim selection (oldest TX).

    Metrics:

        Tracks active/committed/rolled-back TXs, deadlocks, and lock wait times.

        Exposed via TxMetricsSnapshot.

Key Functionality

    Transaction Lifecycle:

        begin(): Starts TX with isolation level, enforces max connections.

        commit()/rollback(): Release locks, update state, notify waiters.

        rollback_to_savepoint(): Reverts to named savepoint, releases subsequent locks.

    Lock Management:

        acquire_lock(): Handles lock upgrades, conflicts, and timeouts.

        Implements wait-for graph updates and deadlock checks during acquisition.

    Deadlock Resolution:

        Background task runs at configured intervals.

        Aborts victim TXs (oldest read timestamp) to break cycles.

    Savepoints:

        create_savepoint(): Bookmarks transaction state/WAL position.

        Enables partial rollbacks within transactions.

    Shutdown:

        Gracefully waits for active TXs to complete (with timeout).

        Stops background deadlock detector.

Concurrency Control

    Lock Compatibility: Rules for shared/exclusive lock coexistence.

    Lock Upgrading: Allows strengthening existing locks (e.g., Shared → Exclusive).

    Timeout Handling: Fails locks after lock_timeout_ms.

Tests

Validates:

    TX begin/commit/rollback workflows

    Lock acquisition/upgrading

    Deadlock detection/resolution

    Savepoint functionality

    Graceful shutdown with active TXs

Key Design Points

    Pessimistic Concurrency Control: Locks acquired before data access.

    Wait-For Graph: Efficient deadlock detection beyond simple timeouts.

    Async-First: Non-blocking operations using Tokio.

    Observability: Detailed TX metrics for monitoring.

    Config-Driven: Lock timeouts, deadlock check intervals, etc.

# ============================= storage.rs ===========================

Core Components

    StorageEngine Struct:

        Manages segment-based storage with an in-memory index

        Integrates with WAL for durability (optional)

        Uses asynchronous primitives (RwLock, Mutex) for concurrency

        Handles compression, checksums, and serialization

    Storage Organization:

        Segments: Immutable data files (.seg) storing key-value records

        Active Segment: Current writable segment

        Offset Index: In-memory HashMap mapping keys → (segment_id, offset)

    Record Format:

        Header (32 bytes): Key/value sizes, timestamp, compression type, checksum, deletion flag

        Data: Serialized key + value (optionally compressed)

Key Functionality

    Write Operations:

        Writes to WAL first (if enabled) for durability

        Serializes and compresses values (LZ4/Zstd/Snappy)

        Appends records to active segment with CRC32 checksums

        Rotates segments when size limits (max_segment_size) are reached

    Read Operations:

        Looks up key in offset index

        Reads record from segment, verifies checksum

        Decompresses and deserializes value

        Handles tombstone records (deletes)

    Background Compaction:

        Merges segments when storage utilization exceeds threshold

        Removes deleted keys and outdated versions

        Rebuilds index and reduces segment count

    Crash Recovery:

        Rebuilds index by scanning segments on startup

        Uses checksums to detect corrupt records

        Timestamps for version tracking

    Compression Support:

        Configurable algorithms (None/LZ4/Zstd/Snappy)

        Transparent compression/decompression

        Type encoded in record header

Metrics & Maintenance

    Tracks reads/writes, bytes transferred, cache hits

    Monitors segment count and compaction stats

    Background compaction task runs at configured intervals

    Graceful shutdown flushes active segment

Record Structure

struct StorageRecord {
key: Bytes, // Serialized key
value: Bytes, // (Possibly compressed) value
compression: CompressionType,
timestamp: Timestamp, // For versioning
deleted: bool, // Tombstone marker
}

Design Highlights

    Log-Structured Storage:

        Append-only writes for high throughput

        Immutable segments enable efficient compaction

        Index enables fast point lookups

    Data Integrity:

        CRC32 checksums for corruption detection

        Configurable checksum verification

    Space Efficiency:

        Background compaction reclaims space

        Compression reduces storage footprint

        Tombstone markers for efficient deletes

    Recovery:

        Segment scanning rebuilds index after crashes

        WAL integration for ACID compliance

Tests Validate

    Basic read/write/delete operations

    Segment rotation when size limits exceeded

    Compression/decompression correctness

    Compaction process and space reclamation

    Crash recovery semantics

# ======================= btree.rs ===================

Core Components

    BTreeIndex:

        Manages the B-Tree structure with configurable order (branching factor).

        Integrates an LRU cache for hot key lookups.

        Tracks metrics (reads, writes, splits, merges, cache hits).

    BTreeNode:

        Leaf Nodes: Store keys and ValueLocation pointers.

        Internal Nodes: Store keys and child pointers.

        Parent pointers for traversal.

    ValueLocation:

        Physical location of a value in storage:

            segment_id: UUID of the segment file.

            offset: Byte offset within the segment.

            size: Value size.

    Serialization:

        BTreeNodeSerialized: Format for disk persistence.

        Supports configurable serialization (Bincode/JSON/MessagePack).

Key Functionality

    Insertion:

        Splits nodes when full (maintains order invariants).

        Updates cache and metrics.

        Handles key updates (overwrite existing values).

    Lookup:

        Checks cache first (LRU).

        Recursive tree traversal (binary search within nodes).

    Deletion:

        Merges nodes or borrows from siblings to maintain fill factor.

        Handles leaf and internal nodes differently:

            Leaf: Direct key/value removal.

            Internal: Replaces with predecessor/successor.

    Range Scans:

        Efficiently retrieves keys in [start, end) range.

        Returns sorted (Key, ValueLocation) pairs.

    Persistence:

        serialize()/deserialize(): Convert tree to/from bytes.

        Maintains parent-child relationships on reload.

    Maintenance Operations:

        Splitting/Merging: Triggered during insert/delete.

        Borrowing: From siblings to avoid merging.

        Metrics tracking (splits, merges, height).

Metrics & Debugging

    BTreeMetricsSnapshot:

        Read/write counts.

        Cache hit ratio.

        Tree height, leaf/internal node counts.

    print_tree(): Debug utility to visualize tree structure.

Design Highlights

    Concurrency:

        Uses RwLock per node for fine-grained access.

        Async-compatible (Tokio primitives).

    Cache Integration:

        LRU cache for frequently accessed keys.

        Configurable size (cache_size_nodes).

    Algorithmic Correctness:

        Implements full B-Tree deletion (cases 1–3c).

        Handles predecessor/successor replacement.

    Serialization:

        Decouples in-memory pointers from disk format.

        Rebuilds parent-child links on deserialization.

Tests Validate

    Insert/search/delete correctness.

    Range scans (sorted, inclusive/exclusive).

    Node splits/merges under small order.

    Cache hit ratios.

    Serialization/deserialization integrity.

# ================= engine.rs ===================

Core Components

    Engine Struct:

        Central orchestrator for all SGBD operations.

        Holds references to storage, transaction manager, WAL, B-Tree index, quantizer, config, state, metrics, and background tasks.

        Uses Arc/RwLock/Mutex for safe async concurrency.

    EngineState:

        Tracks operational status (started, healthy, recovery mode, shutdown requested, last compaction).

    EngineMetrics:

        Collects operation counts, transaction stats, quantization ratio, cache hit rate, response times, and last update timestamp.

    QueryExecution:

        Encapsulates transaction ID, timeout, proficiency, mode, and quantization config for each operation.

Key Functionality

    Initialization & Startup:

        new(): Initializes all subsystems (storage, WAL, index, quantizer, etc.) with config.

        start(): Runs WAL recovery, index rebuild, starts background tasks, and validates health.

    Core Operations:

        get(): ACID-compliant point lookup with cache check, index search, storage fetch, and quantization pipeline.

        set(): Transactional insert/update with WAL logging, quantization, storage write, index update, and cache refresh.

        delete(): Transactional removal with WAL logging, storage tombstone, index update, and cache invalidation.

        range_scan(): Efficient range queries using index, with per-key locking and quantization-aware retrieval.

        batch_execute(): Supports atomic, best-effort, and parallel batch operations with transaction management.

    LLM/AI Integration:

        process_llm_input(): Sanitizes input, creates metadata, stores as InputRecord, and generates output with confidence score and related keys.

    Metrics & Health:

        get_metrics(): Aggregates and updates metrics from all subsystems.

        health_check(): Runs comprehensive diagnostics and triggers compaction if needed.

    Shutdown & Maintenance:

        shutdown(): Graceful shutdown—waits for transactions, flushes WAL/storage, aborts background tasks, and updates state.

        schedule_compaction(), perform_compaction(): Background and on-demand storage compaction with index rebuild.

        Background Tasks: Periodic compaction and metrics update via Tokio tasks.

    Transaction & Locking:

        Full transaction lifecycle (begin/commit/rollback), per-operation locking, and integration with transaction manager.

    Cache Integration:

        Hooks for cache check/update/invalidate via index layer (LRU cache in BTreeIndex).

    Error Handling:

        Rich error taxonomy (SGBDError), timeouts, and resource exhaustion handling.

    Testing:

        Extensive async tests for initialization, lifecycle, CRUD, batch, range scan, metrics, and health.

Design Highlights

    Modern Rust async architecture (Tokio, Arc, RwLock, Mutex).

    ACID compliance via WAL, transaction manager, and locking.

    Pluggable quantization and compression for storage efficiency.

    Modular, extensible, and scope-controlled—advanced features are stubbed for future work.

    Metrics-driven and operationally observable.

Tests Validate

    Engine initialization and lifecycle.

    Correctness of get/set/delete/range_scan.

    Batch operation atomicity and error handling.

    Metrics and health reporting.

    Graceful shutdown

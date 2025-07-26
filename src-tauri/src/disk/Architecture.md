disk/
├── mod.rs # Public façade — init/load, insert, query, delete
├── config.rs # Compile-time constants: vector size, limits, tuning
├── index/ # Vector index algorithms
│ ├── mod.rs # Index trait + dynamic dispatch enum
│ ├── flat.rs # Linear scan fallback (O(n), only used when tiny)
│ ├── ivf.rs # Inverted File Index (clustering-based)
│ └── pq.rs # Product Quantization support (for IVF-PQ)
├── storage/ # Persistent, secure data layout
│ ├── mod.rs # Memory-mapped or in-memory abstraction
│ ├── mmap.rs # mmap backend with growable structure
│ ├── inmem.rs # In-memory (testing or embedded-only mode)
│ └── file_format.rs # Bootstrapped data format (versioned)
├── types.rs # Core types: VectorId, SecureVector, Entry
├── query.rs # Search logic — kNN, radius, etc.
├── cleanup.rs # (Optional) pruning irrelevant data
├── ffi.rs # C-ABI symbols (opt-in via cfg)
├── benches/ # Microbenchmarks (if enabled)
│ └── bench_query.rs
├── tests/ # Internal unit tests
│ ├── mod.rs
│ ├── insert_query.rs
│ └── load_bootstrap.rs

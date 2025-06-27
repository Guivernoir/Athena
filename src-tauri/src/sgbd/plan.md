backend/
├── sgbd/
│   ├── mod.rs              # Public interface
│   ├── engine.rs           # Entry point for DB operations (get/set/query)
│   ├── storage.rs          # File I/O, mmap, page layout
│   ├── btree.rs            # Indexing (or LSM if you’re fancy)
│   ├── wal.rs              # Write-Ahead Logging
│   ├── tx.rs               # (optional) Transaction manager
│   └── types.rs            # Key/value formats, schemas, enums
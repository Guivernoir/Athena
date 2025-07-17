memory/
├── mod.rs # Root module
├── store.rs # Core writer/reader logic (append, flush, mmap)
├── layout.rs # Binary format definitions (headers, record schema)
├── io.rs # Handles file I/O, mmap logic, compression pipeline
├── search.rs # Disk-level vector search (cosine, PQ later)
├── record.rs # Defines `MemoryRecord`, serialization/deserialization
├── index.rs # Optional: in-memory offset/index builder (O(1) seek)
└── tests.rs # Unit + integration tests

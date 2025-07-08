.
├── Architecture.md
├── client.rs // Qdrant client setup and connection
├── embedding
│   ├── engine.cpp
│   ├── engine.hpp
│   ├── ffi.rs
│   ├── models
│   │   └── bge-small-en-v1.5-q8_0.gguf
│   └── mod.rs
├── insert.rs // Insert vectors + payload
├── mod.rs
├── query.rs // Search similar vectors
└── schema.rs // Structs for your documents

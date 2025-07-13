# Engine Module Architecture

```
engine/
├── mod.rs                 # Module declarations and public API
├── core.rs               # Main engine orchestration and request handling
├── processor.rs          # Input processing and output formatting coordination
├── retrieval/
│   ├── mod.rs           # Retrieval subsystem public interface
│   ├── database.rs      # Database query operations
│   ├── websearch.rs     # Web search integration
│   └── merger.rs        # Knowledge source result merging
├── output/
│   ├── mod.rs           # Output generation public interface
│   ├── formatter.rs     # Response formatting logic
│   └── builder.rs       # Response construction and assembly
├── cache/
│   ├── mod.rs           # Caching subsystem interface
│   ├── memory.rs        # In-memory cache implementation
│   └── storage.rs       # Persistent cache storage
└── ffi/                 # C++ integration (if performance demands it)
    ├── mod.rs           # FFI bindings
    ├── engine.cpp       # C++ performance-critical implementations
    └── engine.hpp       # C++ headers
```

## Module Responsibilities

### `core.rs`

- **Mission**: Central command center for all engine operations
- **Duties**: Request orchestration, error handling, performance monitoring
- **Dependencies**: All engine subsystems

### `processor.rs`

- **Mission**: Input/output coordination without the messy details
- **Duties**: Delegate to retrieval, coordinate with output generation
- **Dependencies**: `retrieval/`, `output/`, `preprocessing/` (external)

### `retrieval/`

- **`database.rs`**: Vector database query execution
- **`websearch.rs`**: External search API integration
- **`merger.rs`**: Intelligent result combination (no duplicate intelligence)

### `output/`

- **`formatter.rs`**: Response structure and formatting
- **`builder.rs`**: Assembly line for coherent responses

### `cache/`

- **`memory.rs`**: Hot cache for frequent queries
- **`storage.rs`**: Cold storage for persistent caching

### `ffi/` (Optional - Deploy when Rust alone isn't sufficient)

- **`engine.cpp`**: Performance-critical operations in C++
- **`engine.hpp`**: Interface definitions
- **`mod.rs`**: Safe Rust wrappers around unsafe operations

## Integration Points

- **Security**: Leverages `../security/` for encryption/compression
- **Vector**: Interfaces with `../vector/` for database operations
- **Preprocessing**: Receives formatted input from `../preprocessing/`
- **Personalities**: Remains blissfully unaware of personality quirks

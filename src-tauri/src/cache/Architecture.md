cache/
├── mod.rs # Entry point to expose modules
├── manager.rs # High-level cache logic (push, flush check, reset)
├── embedder.rs # Runs embedding on messages when flush is triggered
├── flusher.rs # Handles triggering + passing embedded records to memory
├── message.rs # Chat message structs, with optional metadata
├── scorer.rs # Optional: relevance or priority scoring (for smart flush)
├── tests.rs # Unit tests for cache logic + flush behavior

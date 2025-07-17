engine/
├── mod.rs # Re-exports everything
├── retrieval/ # Handles searching cache, memory, web
│ ├── mod.rs
│ ├── sources/
│ ├── merger.rs
│ ├── scorer.rs
│ ├── query.rs
│ ├── result.rs
│ ├── router.rs
│ └── tests.rs
├── output/ # Prepares structured LLM input context
│ ├── mod.rs
│ ├── builder.rs
│ ├── formatter.rs
│ ├── injector.rs
│ ├── schema.rs
│ ├── templates.rs
│ ├── traits.rs
│ └── tests.rs
├── orchestrator.rs # High-level coordinator for flow: input → retrieval → output
├── core.rs# Internal logic: conversation logic, input parsing, internal commands
├── traits.rs # Shared traits (e.g., EngineStep, ContextProvider)
├── types.rs # Shared structs: engine state, task types, persona state
└── tests.rs # Cross-module integration tests

.
├── Architecture.md
├── cache
│   ├── Architecture.md
│   ├── embedder.rs
│   ├── flusher.rs
│   ├── manager.rs
│   ├── message.rs
│   ├── mod.rs
│   ├── score.rs
│   └── tests.rs
├── commands.rs
├── embedding
│   ├── engine.cpp
│   ├── engine.hpp
│   ├── ffi.rs
│   ├── models
│   │   └── bge-small-en-v1.5-q8_0.gguf
│   └── mod.rs
├── engine
│   ├── Architecture.md
│   ├── core.rs
│   ├── mod.rs
│   ├── orchestrator.rs
│   ├── output
│   │   ├── Architecture.md
│   │   ├── builder.rs
│   │   ├── formatter.rs
│   │   ├── injector.rs
│   │   ├── mod.rs
│   │   ├── schema.rs
│   │   ├── templates.rs
│   │   ├── tests.rs
│   │   └── traits.rs
│   ├── retrieval
│   │   ├── Architecture.md
│   │   ├── merger.rs
│   │   ├── mod.rs
│   │   ├── query.rs
│   │   ├── result.rs
│   │   ├── router.rs
│   │   ├── scorer.rs
│   │   ├── sources
│   │   │   ├── cache.rs
│   │   │   ├── memory.rs
│   │   │   ├── mod.rs
│   │   │   └── web.rs
│   │   └── tests.rs
│   ├── tests.rs
│   ├── traits.rs
│   └── types.rs
├── lib.rs
├── llama
│   ├── engine.cpp
│   ├── engine.hpp
│   ├── ffi.rs
│   ├── models
│   │   └── qwen2.5-0.5b-instruct-q5_k_m.gguf
│   └── mod.rs
├── main.rs
├── memory
│   ├── Architecture.md
│   ├── index.rs
│   ├── io.rs
│   ├── layout.rs
│   ├── mod.rs
│   ├── record.rs
│   ├── search.rs
│   ├── store.rs
│   └── tests.rs
├── personalities
│   ├── Aurora.toml
│   ├── Ekaterina.toml
│   ├── Erika.toml
│   ├── mod.rs
│   └── Viktor.toml
├── postprocessing
│   ├── Architecture.md
│   ├── formatter.rs
│   ├── interpreter.rs
│   ├── mod.rs
│   ├── persona.rs
│   ├── templates.rs
│   ├── tests.rs
│   ├── traits.rs
│   └── validator.rs
├── preprocessing
│   ├── Architecture.md
│   ├── cleaner.rs
│   ├── context.rs
│   ├── formatter.rs
│   ├── mod.rs
│   ├── router.rs
│   └── tokenizer.rs
├── security
│   ├── compression.rs
│   ├── encryption.rs
│   ├── mod.rs
│   └── quantization
│   ├── bridge.rs
│   ├── config.rs
│   ├── cpp
│   │   ├── CMakeList.txt
│   │   ├── kernels.cpp
│   │   ├── kernels.hpp
│   │   ├── quantizer.cpp
│   │   └── quantizer.hpp
│   └── mod.rs
├── test2.txt
└── test.txt

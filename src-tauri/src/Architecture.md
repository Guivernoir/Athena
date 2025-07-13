.
├── Architecture.md
├── commands.rs
├── engine
│   ├── Architecture.md
│   ├── cache
│   │   ├── memory.rs
│   │   ├── mod.rs
│   │   └── storage.rs
│   ├── core.rs
│   ├── ffi
│   │   ├── engine.cpp
│   │   ├── engine.hpp
│   │   └── mod.rs
│   ├── mod.rs
│   ├── output
│   │   ├── builder.rs
│   │   ├── formatter.rs
│   │   └── mod.rs
│   ├── processor.rs
│   └── retrieval
│   ├── db.rs
│   ├── merger.rs
│   ├── mod.rs
│   └── ws.rs
├── lib.rs
├── llama
│   ├── engine.cpp
│   ├── engine.hpp
│   ├── ffi.rs
│   ├── models
│   │   └── qwen2.5-0.5b-instruct-q5_k_m.gguf
│   └── mod.rs
├── main.rs
├── personalities
│   ├── Aurora.toml
│   ├── Ekaterina.toml
│   ├── Erika.toml
│   ├── mod.rs
│   └── Viktor.toml
├── postprocessing
│   ├── context.rs
│   ├── mod.rs
│   └── persona.rs
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
└── vector
├── Architecture.md
├── client.rs
├── embedding
│   ├── engine.cpp
│   ├── engine.hpp
│   ├── ffi.rs
│   ├── models
│   │   └── bge-small-en-v1.5-q8_0.gguf
│   └── mod.rs
├── insert.rs
├── mod.rs
├── query.rs
├── schema.rs
└── transformation.rs

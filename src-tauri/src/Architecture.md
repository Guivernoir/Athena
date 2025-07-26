guivernoir@guivernoir-Latitude-E6410:~/Desktop/Athena/src-tauri/src$ tree
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
├── disk
│   ├── Architecture.md
│   ├── benches
│   │   └── bench_query.rs
│   ├── cleanup.rs
│   ├── config.rs
│   ├── ffi.rs
│   ├── index
│   │   ├── flat.rs
│   │   ├── ivf.rs
│   │   ├── mod.rs
│   │   └── pq.rs
│   ├── mod.rs
│   ├── query.rs
│   ├── storage
│   │   ├── file_format.rs
│   │   ├── inmem.rs
│   │   ├── mmap.rs
│   │   └── mod.rs
│   ├── tests
│   │   ├── insert_query.rs
│   │   ├── load_bootstrap.rs
│   │   └── mod.rs
│   └── types.rs
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
│   │   │   ├── tests.rs
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
├── personalities
│   ├── Aurora.toml
│   ├── Ekaterina.toml
│   ├── Erika.toml
│   ├── mod.rs
│   └── Viktor.toml
├── postprocessing
│   ├── Architecture.md
│   ├── context.rs
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
│   ├── Architecture.md
│   ├── compression
│   │   ├── Architecture.md
│   │   ├── bitpacking
│   │   │   ├── decoder.rs
│   │   │   ├── encoder.rs
│   │   │   ├── mod.rs
│   │   │   └── utils.rs
│   │   ├── codec.rs
│   │   ├── config.rs
│   │   ├── entropy
│   │   │   ├── ans.rs
│   │   │   ├── huffman.rs
│   │   │   ├── mod.rs
│   │   │   ├── rle.rs
│   │   │   └── zstd.rs
│   │   ├── errors.rs
│   │   ├── mod.rs
│   │   ├── tests
│   │   │   ├── mod.rs
│   │   │   ├── test_bitpacking.rs
│   │   │   ├── test_codec.rs
│   │   │   └── test_entropy.rs
│   │   ├── types.rs
│   │   └── utils.rs
│   ├── encryption
│   │   ├── aead
│   │   │   ├── aes_gcm.rs
│   │   │   ├── chacha20poly1305.rs
│   │   │   └── mod.rs
│   │   ├── Architecture.md
│   │   ├── errors.rs
│   │   ├── kdf
│   │   │   ├── argon2.rs
│   │   │   ├── hkdf.rs
│   │   │   └── mod.rs
│   │   ├── key.rs
│   │   ├── mod.rs
│   │   ├── nonce.rs
│   │   ├── tests
│   │   │   ├── mod.rs
│   │   │   ├── test_aead.rs
│   │   │   ├── test_integration.rs
│   │   │   └── test_kdf.rs
│   │   ├── types.rs
│   │   └── utils.rs
│   ├── mod.rs
│   └── quantization
│   ├── Architecture.md
│   ├── cpp
│   │   ├── CMakeLists.txt
│   │   ├── include
│   │   │   └── faiss_wrapper.h
│   │   ├── README.md
│   │   ├── src
│   │   │   ├── platform.cpp
│   │   │   ├── wrapper.cpp
│   │   │   └── wrapper.h
│   │   └── tests
│   │   ├── CMakeLists.txt
│   │   └── smoke.cpp
│   ├── decoder.rs
│   ├── encoder.rs
│   ├── ffi.rs
│   ├── mod.rs
│   ├── preprocessor.rs
│   ├── tests
│   │   ├── mod.rs
│   │   ├── test_decoder.rs
│   │   ├── test_encoder.rs
│   │   ├── test_integration.rs
│   │   ├── test_preprocessor.rs
│   │   ├── test_trainer.rs
│   │   └── test_types.rs
│   ├── trainer.rs
│   ├── types.rs
│   └── wrapper.rs
├── test2.txt
└── test.txt

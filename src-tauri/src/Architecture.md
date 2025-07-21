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
│   │   ├── faiss_wrapper.cpp
│   │   └── faiss_wrapper.hpp
│   ├── decoder.rs
│   ├── encoder.rs
│   ├── ffi.rs
│   ├── mod.rs
│   ├── preprocessor.rs
│   ├── tests
│   │   ├── mod.rs
│   │   ├── test_encoding.rs
│   │   ├── test_faiss_integration.rs
│   │   └── test_training.rs
│   ├── trainer.rs
│   ├── types.rs
│   └── wrapper.rs

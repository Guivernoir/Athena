.
├── Architecture.md
├── compression
│   ├── Architecture.md
│   ├── bitpacking
│   │   ├── decoder.rs
│   │   ├── encoder.rs
│   │   ├── mod.rs
│   │   └── utils.rs
│   ├── codec.rs
│   ├── config.rs
│   ├── entropy
│   │   ├── ans.rs
│   │   ├── huffman.rs
│   │   ├── mod.rs
│   │   ├── rle.rs
│   │   └── zstd.rs
│   ├── errors.rs
│   ├── mod.rs
│   ├── tests
│   │   ├── mod.rs
│   │   ├── test_bitpacking.rs
│   │   ├── test_codec.rs
│   │   └── test_entropy.rs
│   ├── types.rs
│   └── utils.rs
├── encryption
│   ├── aead
│   │   ├── aes_gcm.rs
│   │   ├── chacha20poly1305.rs
│   │   └── mod.rs
│   ├── Architecture.md
│   ├── errors.rs
│   ├── kdf
│   │   ├── argon2.rs
│   │   ├── hkdf.rs
│   │   └── mod.rs
│   ├── key.rs
│   ├── mod.rs
│   ├── nonce.rs
│   ├── tests
│   │   ├── mod.rs
│   │   ├── test_aead.rs
│   │   ├── test_integration.rs
│   │   └── test_kdf.rs
│   ├── types.rs
│   └── utils.rs
├── mod.rs
└── quantization
├── Architecture.md
├── cpp
│   ├── CMakeLists.txt
│   ├── faiss_wrapper.cpp
│   └── faiss_wrapper.hpp
├── decoder.rs
├── encoder.rs
├── ffi.rs
├── mod.rs
├── preprocessor.rs
├── tests
│   ├── mod.rs
│   ├── test_encoding.rs
│   ├── test_faiss_integration.rs
│   └── test_training.rs
├── trainer.rs
├── types.rs
└── wrapper.rs

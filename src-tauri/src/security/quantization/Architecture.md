quantization/
├── mod.rs # Public Rust API: re-exports encoder, trainer, etc.
├── types.rs # Core types: Vector, QuantizedVector, Codebook
├── preprocessor.rs # Optional: PCA, L2 norm, whitening
├── trainer.rs # KMeans, OPQ, etc. (Rust or FAISS-backed)
├── encoder.rs # PQ / IVFPQ encoding interface
├── decoder.rs # Vector reconstruction (optional)
├── ffi.rs # Rust bindings to C++ wrapper (unsafe FFI)
├── wrapper.rs # Safe Rust abstraction over the FFI
├── tests/
│ ├── mod.rs
│ ├── test_training.rs
│ ├── test_encoding.rs
│ └── test_faiss_integration.rs
├── cpp/ # FAISS bridge implementation in C++
│ ├── faiss_wrapper.hpp # C-compatible API header
│ ├── faiss_wrapper.cpp # Implements C-style API over FAISS C++
│ └── CMakeLists.txt # Builds static/dynamic libfaiss_wrapper.a/.so

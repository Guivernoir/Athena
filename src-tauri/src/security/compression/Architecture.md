compression/
├── mod.rs # Public API: exposes encoders, utils
├── types.rs # Core types: CompressedVec, CodecOptions, etc.
├── bitpacking/
│ ├── mod.rs # Public interface for bitpacking
│ ├── encoder.rs # Packs vectors to bits (4/5/6/8-bit)
│ ├── decoder.rs # Reverses packing
│ └── utils.rs # Bitwise helpers, masks, etc.
├── entropy/
│ ├── mod.rs # Entropy codec interface (trait)
│ ├── rle.rs # Run-length encoding
│ ├── huffman.rs # Huffman coding
│ ├── ans.rs # Asymmetric Numeral Systems
│ └── zstd.rs # Optional: wrapper for zstd crate
├── codec.rs # High-level compressor combining bitpacking + entropy
├── utils.rs # Shared helpers: byte I/O, alignment, zero-padding
├── errors.rs # Custom error types for clean API
├── config.rs # Structs and enums for config options
└── tests/
├── mod.rs
├── test_bitpacking.rs
├── test_entropy.rs
└── test_codec.rs

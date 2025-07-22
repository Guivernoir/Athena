//! **Single-call** compression façade — zero-copy, deterministic, ≤ 8 KB

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub use crate::errors::CompressionError;
pub use crate::types::{CompressedVec, CodecConfig, Codec};

// ------------------------------------------------------------------
// 1. One-line compress / decompress
// ------------------------------------------------------------------
use crate::codec::{compress, decompress, compress_into, decompress_into};

// ------------------------------------------------------------------
// 2. Public API (already exported)
// ------------------------------------------------------------------
/// Compress bytes with a chosen codec.
/// Returns `Vec<u8>` (heap) or `Result<usize, CompressionError>` for zero-copy.
pub use compress as compress_bytes;
pub use decompress as decompress_bytes;

/// Zero-copy variants (caller supplies `&mut [u8]`).
pub use compress_into as compress_bytes_into;
pub use decompress_into as decompress_bytes_into;

// ------------------------------------------------------------------
// 3. Convenience shortcuts
// ------------------------------------------------------------------
/// Compress with **default** settings (`Codec::Zstd`, level 3).
#[inline(always)]
pub fn compress_default(src: &[u8]) -> Result<Vec<u8>, CompressionError> {
    compress(src, CodecConfig::default())
}

/// Decompress with **default** settings (auto-detect via codec tag).
#[inline(always)]
pub fn decompress_default(src: &[u8]) -> Result<Vec<u8>, CompressionError> {
    decompress(src, CodecConfig::default())
}
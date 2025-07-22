//! State-of-art compressed-vector types for the security layer.
//! Const-generic bit-width, zero-copy views, ≤ 30 KB public surface.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use alloc::vec::Vec;
use core::marker::PhantomData;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// 1. Compression Error
// ------------------------------------------------------------------
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CompressionError {
    BitWidthUnsupported { bits: u8 },
    InvalidLength,
    Overflow,
    EntropyError,
}

// ------------------------------------------------------------------
// 2. Codec Options (builder-friendly)
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EntropyMode {
    #[default]
    None,
    Huffman,
    Rle,
    Ans,
    Zstd,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CodecOptions {
    pub bits: u8,
    pub entropy: EntropyMode,
    #[cfg(feature = "simd")]
    pub simd: bool,
}

impl CodecOptions {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }
    #[inline(always)]
    pub fn with_bits(mut self, bits: u8) -> Result<Self, CompressionError> {
        if bits == 0 || bits > 8 {
            return Err(CompressionError::BitWidthUnsupported { bits });
        }
        self.bits = bits;
        Ok(self)
    }
    #[inline(always)]
    pub fn with_entropy(mut self, mode: EntropyMode) -> Self {
        self.entropy = mode;
        self
    }
    #[cfg(feature = "simd")]
    #[inline(always)]
    pub fn enable_simd(mut self, yes: bool) -> Self {
        self.simd = yes;
        self
    }
}

// ------------------------------------------------------------------
// 3. CompressedVec (owned)
// ------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CompressedVec<const BITS: u8> {
    data: Vec<u8>,
    len_bits: usize,
    _marker: PhantomData<[(); BITS as usize]>,
}

impl<const BITS: u8> CompressedVec<BITS> {
    /// Create from raw bytes and exact bit length.
    #[inline(always)]
    pub fn new(data: Vec<u8>, len_bits: usize) -> Result<Self, CompressionError> {
        if BITS == 0 || BITS > 8 {
            return Err(CompressionError::BitWidthUnsupported { bits: BITS });
        }
        Ok(Self {
            data,
            len_bits,
            _marker: PhantomData,
        })
    }

    /// Bit length (not byte length).
    #[inline(always)]
    pub fn len_bits(&self) -> usize {
        self.len_bits
    }

    /// Total byte length (may include padding).
    #[inline(always)]
    pub fn len_bytes(&self) -> usize {
        self.data.len()
    }

    /// Zero-copy byte slice.
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Mutable slice for in-place decompression.
    #[inline(always)]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

// ------------------------------------------------------------------
// 4. Zero-copy views
// ------------------------------------------------------------------
/// Read-only slice of compressed bits.
#[derive(Debug, Clone, Copy)]
pub struct CompressedSlice<'a> {
    bytes: &'a [u8],
    len_bits: usize,
}

impl<'a> CompressedSlice<'a> {
    #[inline(always)]
    pub fn new(bytes: &'a [u8], len_bits: usize) -> Self {
        Self { bytes, len_bits }
    }
    #[inline(always)]
    pub fn len_bits(&self) -> usize { self.len_bits }
    #[inline(always)]
    pub fn as_bytes(&self) -> &'a [u8] { self.bytes }
}

/// Mutable slice for in-place update.
#[derive(Debug)]
pub struct CompressedSliceMut<'a> {
    bytes: &'a mut [u8],
    len_bits: usize,
}

impl<'a> CompressedSliceMut<'a> {
    #[inline(always)]
    pub fn new(bytes: &'a mut [u8], len_bits: usize) -> Self {
        Self { bytes, len_bits }
    }
    #[inline(always)]
    pub fn len_bits(&self) -> usize { self.len_bits }
    #[inline(always)]
    pub fn as_bytes_mut(&mut self) -> &'a mut [u8] { self.bytes }
}

// ------------------------------------------------------------------
// 5. Memory layout notes
// ------------------------------------------------------------------
// - Bits are packed LSB-first within each byte.
// - Padding bits are zero-filled.
// - `len_bits` is the exact original bit count before padding.
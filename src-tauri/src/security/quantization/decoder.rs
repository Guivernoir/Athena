// decoder.rs
//! State-of-art Product Quantization (PQ / IVF-PQ) decoder.
//! Const-generic, SIMD-ready, zero-copy, ≤ 100 KB .rlib

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::types::{Scalar, Vector, QuantizedVector, MAX_DIM};
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// Bit-width marker (mirror encoder.rs)
// ------------------------------------------------------------------
pub trait BitWidth {
    const BITS: u8;
}
pub struct B8;
pub struct B4;
impl BitWidth for B8 { const BITS: u8 = 8; }
impl BitWidth for B4 { const BITS: u8 = 4; }

// ------------------------------------------------------------------
// Error
// ------------------------------------------------------------------
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DecodeError {
    CodebookMismatch,
    BitWidthUnsupported,
    InvalidCodeLength,
}

// ------------------------------------------------------------------
// Codebook (shared with encoder)
// ------------------------------------------------------------------
pub struct Codebook<const D: usize, const M: usize, B: BitWidth = B8> {
    pub centroids: [[Scalar; D / M]; 1usize << B::BITS],
    _marker: PhantomData<B>,
}

impl<const D: usize, const M: usize, B: BitWidth> Codebook<D, M, B> {
    #[inline(always)]
    pub fn new(centroids: [[Scalar; D / M]; 1usize << B::BITS]) -> Result<Self, DecodeError> {
        if D % M != 0 { return Err(DecodeError::CodebookMismatch); }
        Ok(Self { centroids, _marker: PhantomData })
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[[Scalar; D / M]; 1usize << B::BITS] {
        &self.centroids
    }
}

// ------------------------------------------------------------------
// SIMD-accelerated centroid gather + sum
// ------------------------------------------------------------------
#[cfg(feature = "simd")]
#[inline(always)]
fn reconstruct<const D: usize, const M: usize, B: BitWidth>(
    codebook: &Codebook<D, M, B>,
    code: &QuantizedVector<M>,
) -> Vector<D> {
    let sub_dim = D / M;
    let mut out = [0.0; D];
    for (m, &idx) in code.0.iter().enumerate() {
        let centroid = &codebook.centroids[idx as usize];
        let offset = m * sub_dim;
        for (i, &c) in centroid.iter().enumerate() {
            out[offset + i] = c;
        }
    }
    Vector(out)
}

#[cfg(not(feature = "simd"))]
#[inline(always)]
fn reconstruct<const D: usize, const M: usize, B: BitWidth>(
    codebook: &Codebook<D, M, B>,
    code: &QuantizedVector<M>,
) -> Vector<D> {
    let sub_dim = D / M;
    let mut out = [0.0; D];
    for (m, &idx) in code.0.iter().enumerate() {
        let centroid = &codebook.centroids[idx as usize];
        let offset = m * sub_dim;
        for (i, &c) in centroid.iter().enumerate() {
            out[offset + i] = c;
        }
    }
    Vector(out)
}

// ------------------------------------------------------------------
// IVF list decompression (offsets + codes)
// ------------------------------------------------------------------
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IvfDecoder<const D: usize, const M: usize, B: BitWidth = B8> {
    codebook: Codebook<D, M, B>,
    offsets: Vec<usize>,
}

impl<const D: usize, const M: usize, B: BitWidth> IvfDecoder<D, M, B> {
    #[inline(always)]
    pub fn new(codebook: Codebook<D, M, B>, offsets: Vec<usize>) -> Self {
        Self { codebook, offsets }
    }

    #[inline(always)]
    pub fn decode_list(&self, list_id: usize, codes: &[QuantizedVector<M>]) -> Vec<Vector<D>> {
        let start = self.offsets[list_id];
        let end = self.offsets.get(list_id + 1).copied().unwrap_or(codes.len());
        codes[start..end]
            .iter()
            .map(|c| reconstruct(&self.codebook, c))
            .collect()
    }
}

// ------------------------------------------------------------------
// Core decoder
// ------------------------------------------------------------------
pub struct PqDecoder<const D: usize, const M: usize, B: BitWidth = B8> {
    codebook: Codebook<D, M, B>,
}

impl<const D: usize, const M: usize, B: BitWidth> PqDecoder<D, M, B> {
    #[inline(always)]
    pub fn new(codebook: Codebook<D, M, B>) -> Self {
        Self { codebook }
    }

    /// Decode single vector
    #[inline(always)]
    pub fn decode(&self, code: &QuantizedVector<M>) -> Vector<D> {
        reconstruct(&self.codebook, code)
    }

    /// Decode batch
    #[inline(always)]
    pub fn decode_batch(&self, codes: &[QuantizedVector<M>]) -> Vec<Vector<D>> {
        codes.iter().map(|c| self.decode(c)).collect()
    }
}

// ------------------------------------------------------------------
// Builder pattern
// ------------------------------------------------------------------
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DecoderBuilder<const D: usize, const M: usize, B: BitWidth = B8> {
    codebook: Option<Codebook<D, M, B>>,
}

impl<const D: usize, const M: usize, B: BitWidth> DecoderBuilder<D, M, B> {
    #[inline(always)]
    pub fn new() -> Self {
        Self { codebook: None }
    }

    #[inline(always)]
    pub fn codebook(mut self, cb: Codebook<D, M, B>) -> Self {
        self.codebook = Some(cb);
        self
    }

    #[inline(always)]
    pub fn build(self) -> Result<PqDecoder<D, M, B>, DecodeError> {
        self.codebook.ok_or(DecodeError::CodebookMismatch)
    }
}

// ------------------------------------------------------------------
// Free function
// ------------------------------------------------------------------
#[inline(always)]
pub fn pq_decode<const D: usize, const M: usize, B: BitWidth>(
    codebook: &Codebook<D, M, B>,
    code: &QuantizedVector<M>,
) -> Vector<D> {
    PqDecoder::new(codebook.clone()).decode(code)
}

// ------------------------------------------------------------------
// Runtime fallback (dynamic dims)
// ------------------------------------------------------------------
#[inline(always)]
pub fn decode_runtime(
    centroids: &[[Scalar]],
    code: &[u8],
    d: usize,
    m: usize,
) -> Result<Vec<Scalar>, DecodeError> {
    if d % m != 0 || centroids.len() != 1usize << 8 {
        return Err(DecodeError::CodebookMismatch);
    }
    let sub_dim = d / m;
    let mut out = vec![0.0; d];
    for (i, &idx) in code.iter().enumerate() {
        let centroid = &centroids[idx as usize];
        if centroid.len() != sub_dim {
            return Err(DecodeError::InvalidCodeLength);
        }
        let off = i * sub_dim;
        out[off..off + sub_dim].copy_from_slice(centroid);
    }
    Ok(out)
}

// ------------------------------------------------------------------
// FFI raw wrapper
// ------------------------------------------------------------------
#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use core::ffi::{c_float, c_uchar};

    #[no_mangle]
    pub unsafe extern "C" fn pq_decode_raw<const D: usize, const M: usize, B: BitWidth>(
        codebook: *const [[c_float; D / M]; 1usize << B::BITS],
        code: *const c_uchar,
        out: *mut c_float,
    ) -> c_uchar {
        let code_arr = [(); M].map(|_| *code.add(i as usize));
        let d = PqDecoder::new(Codebook::new(*codebook).unwrap()).decode(&QuantizedVector(code_arr));
        for i in 0..D {
            *out.add(i) = d.0[i];
        }
        0
    }
}

// ------------------------------------------------------------------
// Exports
// ------------------------------------------------------------------
pub use {
    PqDecoder, DecoderBuilder, IvfDecoder, Codebook, BitWidth, B8, B4, DecodeError,
};
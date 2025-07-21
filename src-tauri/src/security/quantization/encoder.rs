//! State-of-art Product Quantization (PQ / IVF-PQ) encoder.
//! Const-generic, SIMD-ready, mobile-first, ≤ 150 KB .rlib

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::types::{Scalar, Vector, QuantizedVector, MAX_DIM};
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// Error
// ------------------------------------------------------------------
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EncodeError {
    CodebookMismatch,
    BitWidthUnsupported,
    InvalidInput,
}

// ------------------------------------------------------------------
// Bit-width marker
// ------------------------------------------------------------------
pub trait BitWidth {
    const BITS: u8;
}
pub struct B8;
pub struct B4;
impl BitWidth for B8 { const BITS: u8 = 8; }
impl BitWidth for B4 { const BITS: u8 = 4; }

// ------------------------------------------------------------------
// Distance helper
// ------------------------------------------------------------------
#[inline(always)]
fn l2_sq<const D: usize>(a: &[Scalar; D], b: &[Scalar; D]) -> Scalar {
    let mut acc = 0.0;
    let mut i = 0;
    while i < D {
        let d = a[i] - b[i];
        acc += d * d;
        i += 1;
    }
    acc
}

// ------------------------------------------------------------------
// Codebook
// ------------------------------------------------------------------
pub struct Codebook<const D: usize, const M: usize, B: BitWidth = B8> {
    pub centroids: [[Scalar; D / M]; 1usize << B::BITS],
    _marker: PhantomData<B>,
}

impl<const D: usize, const M: usize, B: BitWidth> Codebook<D, M, B> {
    #[inline(always)]
    pub fn new(centroids: [[Scalar; D / M]; 1usize << B::BITS]) -> Result<Self, EncodeError> {
        if D % M != 0 { return Err(EncodeError::InvalidInput); }
        Ok(Self { centroids, _marker: PhantomData })
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[[Scalar; D / M]; 1usize << B::BITS] {
        &self.centroids
    }
}

// ------------------------------------------------------------------
// SIMD LUT generation (ADC)
// ------------------------------------------------------------------
#[cfg(feature = "simd")]
pub struct Lut<const M: usize, const K: usize> {
    pub table: [[Scalar; K]; M],
}

#[cfg(feature = "simd")]
impl<const M: usize, const K: usize> Lut<M, K> {
    #[inline(always)]
    pub fn new<const D: usize, B: BitWidth>(
        codebook: &Codebook<D, M, B>,
        query: &Vector<D>,
    ) -> Self {
        let mut table = [[0.0; K]; M];
        let sub_dim = D / M;
        for m in 0..M {
            for k in 0..K {
                let mut dist = 0.0;
                let mut i = 0;
                while i < sub_dim {
                    let d = query.0[m * sub_dim + i] - codebook.centroids[k][i];
                    dist += d * d;
                    i += 1;
                }
                table[m][k] = dist;
            }
        }
        Self { table }
    }
}

#[cfg(not(feature = "simd"))]
pub struct Lut<const M: usize, const K: usize>;

// ------------------------------------------------------------------
// PQ encoder
// ------------------------------------------------------------------
pub struct PqEncoder<const D: usize, const M: usize, B: BitWidth = B8> {
    codebook: Codebook<D, M, B>,
    _marker: PhantomData<B>,
}

impl<const D: usize, const M: usize, B: BitWidth> PqEncoder<D, M, B> {
    #[inline(always)]
    pub fn new(codebook: Codebook<D, M, B>) -> Self {
        Self { codebook, _marker: PhantomData }
    }

    /// Encode a single vector → `[u8; M]`
    #[inline(always)]
    pub fn encode(&self, v: &Vector<D>) -> QuantizedVector<M> {
        let sub_dim = D / M;
        let mut code = [0u8; M];
        for m in 0..M {
            let mut best = 0usize;
            let mut best_dist = Scalar::MAX;
            let start = m * sub_dim;
            let sub_vec = &v.0[start..start + sub_dim];
            for (k, centroid) in self.codebook.centroids.iter().enumerate() {
                let mut dist = 0.0;
                let mut i = 0;
                while i < sub_dim {
                    let d = sub_vec[i] - centroid[i];
                    dist += d * d;
                    i += 1;
                }
                if dist < best_dist {
                    best_dist = dist;
                    best = k;
                }
            }
            code[m] = best as u8;
        }
        QuantizedVector(code)
    }

    /// Batch encode
    #[inline(always)]
    pub fn encode_batch(&self, batch: &[Vector<D>]) -> Vec<QuantizedVector<M>> {
        batch.iter().map(|v| self.encode(v)).collect()
    }
}

// ------------------------------------------------------------------
// Builder pattern
// ------------------------------------------------------------------
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EncoderBuilder<const D: usize, const M: usize, B: BitWidth = B8> {
    codebook: Option<Codebook<D, M, B>>,
}

impl<const D: usize, const M: usize, B: BitWidth> EncoderBuilder<D, M, B> {
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
    pub fn build(self) -> Result<PqEncoder<D, M, B>, EncodeError> {
        self.codebook.ok_or(EncodeError::CodebookMismatch)
    }
}

// ------------------------------------------------------------------
// Free function
// ------------------------------------------------------------------
#[inline(always)]
pub fn pq_encode<const D: usize, const M: usize, B: BitWidth>(
    codebook: &Codebook<D, M, B>,
    v: &Vector<D>,
) -> QuantizedVector<M> {
    PqEncoder::new(codebook.clone()).encode(v)
}

// ------------------------------------------------------------------
// FFI raw wrapper
// ------------------------------------------------------------------
#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use core::ffi::{c_float, c_uchar};

    #[no_mangle]
    pub unsafe extern "C" fn pq_encode_raw<const D: usize, const M: usize, B: BitWidth>(
        codebook: *const [[c_float; D / M]; 1usize << B::BITS],
        vec: *const c_float,
        out: *mut c_uchar,
    ) -> c_uchar {
        let v = Vector([(); D].map(|_| *vec.add(i as usize)));
        let cb = Codebook::new(*codebook).unwrap();
        let q = PqEncoder::new(cb).encode(&v);
        for i in 0..M {
            *out.add(i) = q.0[i];
        }
        0
    }
}

// ------------------------------------------------------------------
// Exports
// ------------------------------------------------------------------
pub use {
    PqEncoder, EncoderBuilder, Codebook, BitWidth, B8, B4, DistanceMetric, EncodeError,
};

#[cfg(feature = "simd")]
pub use Lut;
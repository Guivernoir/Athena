// mod.rs
//! State-of-art public API for quantization.
//! Single-call `quantize` / `dequantize` with optional preprocessing.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub use crate::types::{Vector, QuantizedVector, Codebook, BitWidth, B8, B4};
pub use crate::types::{PreprocessError as ValidationError, TrainError, EncodeError, DecodeError};

use crate::preprocessor::{Preprocessor, L2Norm, MeanCenter};
use crate::encoder::{PqEncoder, DecodeError as EncError};
use crate::decoder::{PqDecoder, DecodeError as DecError};
use crate::trainer::{KMeans, InitMethod, DistanceMetric, EmptyClusterAction};

use core::marker::PhantomData;
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// Unified error
// ------------------------------------------------------------------
#[non_exhaustive]
#[derive(Debug)]
pub enum QuantizeError {
    Encode(EncError),
    Decode(DecError),
    Train(TrainError),
    Validation(ValidationError),
}

impl From<EncError> for QuantizeError { fn from(e: EncError) -> Self { Self::Encode(e) } }
impl From<DecError> for QuantizeError { fn from(e: DecError) -> Self { Self::Decode(e) } }
impl From<TrainError> for QuantizeError { fn from(e: TrainError) -> Self { Self::Train(e) } }
impl From<ValidationError> for QuantizeError { fn from(e: ValidationError) -> Self { Self::Validation(e) } }

// ------------------------------------------------------------------
// Default pipeline
// ------------------------------------------------------------------
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Quantizer<const D: usize, const M: usize, B: BitWidth = B8> {
    preprocessor: Preprocessor<D>,
    encoder: PqEncoder<D, M, B>,
    decoder: PqDecoder<D, M, B>,
}

impl<const D: usize, const M: usize, B: BitWidth> Quantizer<D, M, B> {
    /// Ready-to-use quantizer with default preprocessing (L2 + mean-center).
    #[inline(always)]
    pub fn new(codebook: Codebook<D, M, B>) -> Self {
        let encoder = PqEncoder::new(codebook.clone());
        let decoder = PqDecoder::new(codebook);
        Self {
            preprocessor: Preprocessor::<D>::new()
                .l2_normalize(true)
                .mean_center(true),
            encoder,
            decoder,
        }
    }

    /// Single-call quantize.
    #[inline(always)]
    pub fn quantize(&self, v: &Vector<D>) -> Result<QuantizedVector<M>, QuantizeError> {
        let prepped = self.preprocessor.apply(v);
        Ok(self.encoder.encode(&prepped))
    }

    /// Single-call dequantize.
    #[inline(always)]
    pub fn dequantize(&self, q: &QuantizedVector<M>) -> Result<Vector<D>, QuantizeError> {
        Ok(self.decoder.decode(q))
    }

    /// Batch helpers.
    #[inline(always)]
    pub fn quantize_batch(&self, batch: &[Vector<D>]) -> Result<Vec<QuantizedVector<M>>, QuantizeError> {
        batch.iter().map(|v| self.quantize(v)).collect()
    }

    #[inline(always)]
    pub fn dequantize_batch(&self, codes: &[QuantizedVector<M>]) -> Result<Vec<Vector<D>>, QuantizeError> {
        codes.iter().map(|q| self.dequantize(q)).collect()
    }
}

// ------------------------------------------------------------------
// Builder for power users
// ------------------------------------------------------------------
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuantizerBuilder<const D: usize, const M: usize, B: BitWidth = B8> {
    init: InitMethod,
    max_iters: usize,
    tolerance: Scalar,
    metric: DistanceMetric,
    empty_action: EmptyClusterAction,
    l2_norm: bool,
    mean_center: bool,
    _marker: PhantomData<B>,
}
type Scalar = crate::types::Scalar;

impl<const D: usize, const M: usize, B: BitWidth> QuantizerBuilder<D, M, B> {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn init(mut self, method: InitMethod) -> Self {
        self.init = method;
        self
    }

    #[inline(always)]
    pub fn max_iters(mut self, iters: usize) -> Self {
        self.max_iters = iters;
        self
    }

    #[inline(always)]
    pub fn tolerance(mut self, tol: Scalar) -> Self {
        self.tolerance = tol;
        self
    }

    #[inline(always)]
    pub fn metric(mut self, m: DistanceMetric) -> Self {
        self.metric = m;
        self
    }

    #[inline(always)]
    pub fn empty_action(mut self, action: EmptyClusterAction) -> Self {
        self.empty_action = action;
        self
    }

    #[inline(always)]
    pub fn l2_norm(mut self, enable: bool) -> Self {
        self.l2_norm = enable;
        self
    }

    #[inline(always)]
    pub fn mean_center(mut self, enable: bool) -> Self {
        self.mean_center = enable;
        self
    }

    /// Train codebook then build quantizer.
    #[inline(always)]
    pub fn fit(self, data: &[Vector<D>]) -> Result<Quantizer<D, M, B>, QuantizeError> {
        let kmeans = KMeans::<D, 1usize << B::BITS>::builder()
            .init(self.init)
            .max_iters(self.max_iters)
            .tolerance(self.tolerance)
            .metric(self.metric)
            .empty_action(self.empty_action)
            .build();
        let codebook = kmeans.fit(data)?;
        Ok(Quantizer::new(codebook))
    }
}

// ------------------------------------------------------------------
// Runtime helpers
// ------------------------------------------------------------------
#[inline(always)]
pub fn quantize_runtime(
    data: &[Scalar],
    d: usize,
    m: usize,
    centroids: &[[Scalar]],
) -> Result<Vec<u8>, QuantizeError> {
    if d % m != 0 || centroids.len() != 256 {
        return Err(QuantizeError::Validation(ValidationError::DimMismatch));
    }
    let mut out = vec![0u8; m * (data.len() / d)];
    let sub_dim = d / m;
    for (chunk_idx, chunk) in data.chunks_exact(d).enumerate() {
        for m_idx in 0..m {
            let mut best = 0usize;
            let mut best_dist = Scalar::MAX;
            let start = m_idx * sub_dim;
            let sub_vec = &chunk[start..start + sub_dim];
            for (k, centroid) in centroids.iter().enumerate() {
                let mut dist = 0.0;
                for (a, &b) in sub_vec.iter().zip(centroid.iter()) {
                    let d = a - b;
                    dist += d * d;
                }
                if dist < best_dist {
                    best_dist = dist;
                    best = k;
                }
            }
            out[chunk_idx * m + m_idx] = best as u8;
        }
    }
    Ok(out)
}

#[inline(always)]
pub fn dequantize_runtime(
    codes: &[u8],
    d: usize,
    m: usize,
    centroids: &[[Scalar]],
) -> Result<Vec<Scalar>, QuantizeError> {
    if d % m != 0 || centroids.len() != 256 {
        return Err(QuantizeError::Validation(ValidationError::DimMismatch));
    }
    let sub_dim = d / m;
    let mut out = vec![0.0; codes.len() / m * d];
    for (chunk_idx, chunk) in codes.chunks_exact(m).enumerate() {
        for (m_idx, &idx) in chunk.iter().enumerate() {
            let centroid = &centroids[idx as usize];
            let offset = chunk_idx * d + m_idx * sub_dim;
            out[offset..offset + sub_dim].copy_from_slice(centroid);
        }
    }
    Ok(out)
}

// ------------------------------------------------------------------
// FFI bridge
// ------------------------------------------------------------------
#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use core::ffi::{c_float, c_uchar};

    #[no_mangle]
    pub unsafe extern "C" fn quantize_raw(
        data: *const c_float,
        len: usize,
        d: usize,
        m: usize,
        centroids: *const *const c_float,
        centroids_len: usize,
        out: *mut c_uchar,
    ) -> c_uchar {
        if len % d != 0 || centroids_len != 256 {
            return 1;
        }
        let data_slice = core::slice::from_raw_parts(data, len);
        let centroids_slice = (0..256)
            .map(|i| {
                core::slice::from_raw_parts(
                    *centroids.add(i),
                    d / m,
                )
            })
            .collect::<Vec<_>>();
        match quantize_runtime(data_slice, d, m, &centroids_slice) {
            Ok(bytes) => {
                core::ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len());
                0
            }
            Err(_) => 1,
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn dequantize_raw(
        codes: *const c_uchar,
        len: usize,
        d: usize,
        m: usize,
        centroids: *const *const c_float,
        centroids_len: usize,
        out: *mut c_float,
    ) -> c_uchar {
        if len % m != 0 || centroids_len != 256 {
            return 1;
        }
        let codes_slice = core::slice::from_raw_parts(codes, len);
        let centroids_slice = (0..256)
            .map(|i| {
                core::slice::from_raw_parts(
                    *centroids.add(i),
                    d / m,
                )
            })
            .collect::<Vec<_>>();
        match dequantize_runtime(codes_slice, d, m, &centroids_slice) {
            Ok(vec) => {
                core::ptr::copy_nonoverlapping(vec.as_ptr(), out, vec.len());
                0
            }
            Err(_) => 1,
        }
    }
}

// ------------------------------------------------------------------
// Convenience exports
// ------------------------------------------------------------------
pub use {
    Quantizer, QuantizerBuilder, quantize_runtime, dequantize_runtime, QuantizeError,
};

#[cfg(feature = "ffi")]
pub use ffi::{quantize_raw, dequantize_raw};

// ------------------------------------------------------------------
// Zero-cost re-exports from types
pub use crate::types::{Vector, QuantizedVector, Codebook};
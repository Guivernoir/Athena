//! **State-of-art** safe façade over `crate::ffi`.  
//! Zero-copy, panic-safe, `Send + Sync`, ≤ 5 MB binary.

use crate::types::{Scalar, Vector, QuantizedVector, DIM, CODE_BYTES};
use crate::ffi::{Quantizer as RawQuantizer, TrainError};
use core::sync::Arc;

/// High-level, thread-safe quantizer handle.
#[derive(Debug, Clone)]
pub struct Quantizer {
    inner: Arc<RawQuantizer>,
}

impl Quantizer {
    /// Creates a fresh quantizer for the **current** `DIM`.
    #[inline(always)]
    pub fn new() -> Result<Self, TrainError> {
        let inner = RawQuantizer::new()?;
        Ok(Self { inner: Arc::new(inner) })
    }

    /// Train on a batch of vectors.  
    /// Exclusive access enforced via `&mut self`.
    #[inline]
    pub fn train<I>(&mut self, batch: I) -> Result<(), TrainError>
    where
        I: IntoIterator<Item = Vector<DIM>>,
    {
        let flat: Vec<Scalar> = batch.into_iter().flat_map(|v| v.0).collect();
        let n_vectors = flat.len() / DIM;

        Arc::get_mut(&mut self.inner)
            .ok_or(TrainError::Other)?
            .train(&flat, n_vectors)
    }

    /// Encode a single vector → 48-byte code.
    #[inline(always)]
    pub fn encode(&self, v: &Vector<DIM>) -> Result<QuantizedVector<CODE_BYTES>, TrainError> {
        self.inner.encode(v)
    }

    /// Decode 48-byte code → vector.
    #[inline(always)]
    pub fn decode(&self, q: &QuantizedVector<CODE_BYTES>) -> Result<Vector<DIM>, TrainError> {
        self.inner.decode(q)
    }

    /// Runtime OpenMP knob (global).
    #[inline(always)]
    pub fn set_threads(n: u32) {
        RawQuantizer::set_threads(n);
    }

    #[inline(always)]
    pub fn max_threads() -> u32 {
        RawQuantizer::max_threads()
    }
}
// types.rs
//! State-of-art, zero-copy core types for vector quantization.
//! Const-generic, serde-ready, panic-safe, Send + Sync.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_snake_case)]

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use core::fmt;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::slice;
use alloc::sync::Arc;
use alloc::boxed::Box;
use alloc::vec::Vec;

/// Floating-point scalar used throughout the crate.
pub type Scalar = f32;

/// Maximum supported dimensionality (compile-time ceiling).
pub const MAX_DIM: usize = 1024;

/// Number of centroids per sub-quantizer (2⁸ = 256).
pub const K: usize = 256;

/// Number of sub-quantizers (bytes) for the current default dimension.
/// Derived: DIM / (256 centroids → 1 byte each) = 48.
pub const CODE_BYTES: usize = 48;

// ------------------------------------------------------------------
// 1. Vector & QuantizedVector
// ------------------------------------------------------------------

/// Const-generic, fixed-length vector.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vector<const D: usize>([Scalar; D]);

impl<const D: usize> Vector<D> {
    /// Create from array (compile-time length check).
    #[inline(always)]
    pub const fn new(data: [Scalar; D]) -> Self {
        Self(data)
    }

    /// Zero-copy slice view usable in unsafe contexts.
    #[inline(always)]
    pub fn as_slice(&self) -> VectorView<'_> {
        VectorView(&self.0)
    }

    /// Mutable slice view.
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> VectorViewMut<'_> {
        VectorViewMut(&mut self.0)
    }
}

impl<const D: usize> From<[Scalar; D]> for Vector<D> {
    #[inline(always)]
    fn from(arr: [Scalar; D]) -> Self {
        Self(arr)
    }
}

/// Const-generic quantized code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuantizedVector<const BYTES: usize = CODE_BYTES>([u8; BYTES]);

impl<const BYTES: usize> QuantizedVector<BYTES> {
    /// Create from array.
    #[inline(always)]
    pub const fn new(code: [u8; BYTES]) -> Self {
        Self(code)
    }

    /// Zero-copy byte slice view.
    #[inline(always)]
    pub fn as_bytes(&self) -> QuantizedView<'_> {
        QuantizedView(&self.0)
    }
}

// ------------------------------------------------------------------
// 2. Zero-copy view types for unsafe & FFI contexts
// ------------------------------------------------------------------

/// Lightweight read-only slice wrapper for `Vector`.
#[derive(Debug, Clone, Copy)]
pub struct VectorView<'a>(&'a [Scalar]);

impl<'a> Deref for VectorView<'a> {
    type Target = [Scalar];
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Lightweight mutable slice wrapper.
pub struct VectorViewMut<'a>(&'a mut [Scalar]);

impl<'a> Deref for VectorViewMut<'a> {
    type Target = [Scalar];
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> DerefMut for VectorViewMut<'a> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

/// Lightweight slice wrapper for quantized bytes.
#[derive(Debug, Clone, Copy)]
pub struct QuantizedView<'a>(&'a [u8]);

impl<'a> Deref for QuantizedView<'a> {
    type Target = [u8];
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

// ------------------------------------------------------------------
// 3. Codebook
// ------------------------------------------------------------------

/// Immutable codebook: `[K]` centroids for each sub-quantizer.
/// Stored as `Box<[[Scalar; K]]>` for contiguous memory and cheap clone via `Arc`.
#[derive(Debug, Clone)]
pub struct Codebook<const M: usize> {
    centroids: Arc<[[Scalar; K]; M]>,
}

impl<const M: usize> Codebook<M> {
    /// Construct from raw centroids (validated outside this type).
    #[inline(always)]
    pub fn new(centroids: Box<[[Scalar; K]; M]>) -> Self {
        Self {
            centroids: Arc::from(centroids),
        }
    }

    /// Zero-copy slice of centroids.
    #[inline(always)]
    pub fn as_slice(&self) -> &[[Scalar; K]; M] {
        &self.centroids
    }
}

// ------------------------------------------------------------------
// 4. Meta wrapper for strided data
// ------------------------------------------------------------------

/// Metadata carrier for future strided / batched data.
/// Keeps `Vector` itself un-bloated.
#[derive(Debug, Clone, Copy)]
pub struct Meta {
    /// Logical length (may differ from physical storage).
    pub len: usize,
    /// Byte stride between successive vectors.
    pub stride: usize,
}

impl Meta {
    /// Safe construction.
    #[inline(always)]
    pub const fn new(len: usize, stride: usize) -> Self {
        Self { len, stride }
    }
}

// ------------------------------------------------------------------
// 5. Error taxonomy (non-exhaustive)
// ------------------------------------------------------------------

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationError {
    DimensionTooLarge,
    DimensionMisaligned,
    StrideTooSmall,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrideError {
    OverlappingSlice,
    OutOfBounds,
}

// ------------------------------------------------------------------
// 6. Trait alias for mmap future-proofing
// ------------------------------------------------------------------

/// Marker trait for types that can be backed by memory-mapped storage.
pub trait Mappable: Send + Sync {}

impl<const D: usize> Mappable for Vector<D> {}
impl<const BYTES: usize> Mappable for QuantizedVector<BYTES> {}
impl<const M: usize> Mappable for Codebook<M> {}
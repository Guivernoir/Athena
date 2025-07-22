//! Lightweight, zero-copy compression errors.
//! ≤ 2 KB .rlib, FFI-safe, serde-ready.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::types::ValidationError;

#[cfg(feature = "thiserror")]
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// 1. Stable numeric codes for FFI
// ------------------------------------------------------------------
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CompressionErrorCode {
    Ok = 0,
    BitWidthUnsupported = 1,
    InvalidLength = 2,
    Overflow = 3,
    EntropyError = 4,
    Validation = 5,
}

// ------------------------------------------------------------------
// 2. Detailed Rust-side error
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CompressionError {
    BitWidthUnsupported { bits: u8 },
    InvalidLength { expected: usize, found: usize },
    Overflow,
    EntropyError,
    Validation(ValidationError),
}

impl CompressionError {
    /// FFI numeric code
    #[inline(always)]
    pub fn code(&self) -> CompressionErrorCode {
        match self {
            Self::BitWidthUnsupported { .. } => CompressionErrorCode::BitWidthUnsupported,
            Self::InvalidLength { .. } => CompressionErrorCode::InvalidLength,
            Self::Overflow => CompressionErrorCode::Overflow,
            Self::EntropyError => CompressionErrorCode::EntropyError,
            Self::Validation(_) => CompressionErrorCode::Validation,
        }
    }
}

// ------------------------------------------------------------------
// 3. Optional std::error::Error impl
// ------------------------------------------------------------------
#[cfg(feature = "thiserror")]
#[cfg_attr(docsrs, doc(cfg(feature = "thiserror")))]
impl core::fmt::Display for CompressionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BitWidthUnsupported { bits } => write!(f, "Bit width {} not supported", bits),
            Self::InvalidLength { expected, found } => write!(
                f,
                "Invalid length: expected {}, found {}",
                expected, found
            ),
            Self::Overflow => write!(f, "Bit-stream overflow"),
            Self::EntropyError => write!(f, "Entropy codec error"),
            Self::Validation(e) => write!(f, "Validation error: {:?}", e),
        }
    }
}

#[cfg(feature = "thiserror")]
#[cfg_attr(docsrs, doc(cfg(feature = "thiserror")))]
impl Error for CompressionError {}

// ------------------------------------------------------------------
// 4. Re-export for convenience
// ------------------------------------------------------------------
pub use CompressionError;
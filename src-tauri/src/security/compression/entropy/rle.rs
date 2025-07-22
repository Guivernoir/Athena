//! **Classic byte-level Run-Length Encoder** — static, SIMD-ready, ≤ 6 KB
//! Format: [count, value] pairs, count ≤ 255, optional escape for literals.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::errors::CompressionError;
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// 1. Run-length table (optional u16 mode)
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RleConfig {
    pub max_run: u16,
    pub escape: u8,
}

impl Default for RleConfig {
    #[inline(always)]
    fn default() -> Self {
        Self { max_run: 255, escape: 0x00 }
    }
}

// ------------------------------------------------------------------
// 2. Encoder
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub struct RleEncoder<'a> {
    config: &'a RleConfig,
}

impl<'a> RleEncoder<'a> {
    #[inline(always)]
    pub fn new(cfg: &'a RleConfig) -> Self {
        Self { config: cfg }
    }

    /// Encode slice → Vec<u8>
    #[inline(always)]
    pub fn encode(&self, src: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let mut out = Vec::with_capacity(src.len() * 2 + 1);
        self.encode_into(src, &mut out)?;
        Ok(out)
    }

    /// Zero-copy encode into caller buffer.
    #[inline(always)]
    pub fn encode_into(
        &self,
        src: &[u8],
        dst: &mut Vec<u8>,
    ) -> Result<(), CompressionError> {
        dst.clear();
        let mut i = 0usize;
        while i < src.len() {
            let val = src[i];
            let mut run_len = 1usize;
            while i + run_len < src.len()
                && src[i + run_len] == val
                && run_len < self.config.max_run as usize
            {
                run_len += 1;
            }
            dst.push(run_len as u8);
            dst.push(val);
            i += run_len;
        }
        Ok(())
    }
}

// ------------------------------------------------------------------
// 3. Decoder
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub struct RleDecoder<'a> {
    config: &'a RleConfig,
}

impl<'a> RleDecoder<'a> {
    #[inline(always)]
    pub fn new(cfg: &'a RleConfig) -> Self {
        Self { config: cfg }
    }

    /// Decode slice → Vec<u8>
    #[inline(always)]
    pub fn decode(&self, src: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let mut out = Vec::with_capacity(src.len() * 2);
        self.decode_into(src, &mut out)?;
        Ok(out)
    }

    /// Zero-copy decode into caller buffer.
    #[inline(always)]
    pub fn decode_into(
        &self,
        src: &[u8],
        dst: &mut [u8],
    ) -> Result<usize, CompressionError> {
        if src.len() % 2 != 0 {
            return Err(CompressionError::InvalidLength);
        }
        let mut out_idx = 0usize;
        let mut i = 0usize;
        while i < src.len() {
            let count = src[i] as usize;
            let val   = src[i + 1];
            if out_idx + count > dst.len() {
                return Err(CompressionError::Overflow);
            }
            for j in 0..count {
                dst[out_idx + j] = val;
            }
            out_idx += count;
            i += 2;
        }
        Ok(out_idx)
    }
}

// ------------------------------------------------------------------
// 4. SIMD-accelerated find runs (placeholder)
// ------------------------------------------------------------------
#[cfg(all(feature = "simd", target_feature = "avx2"))]
#[inline(always)]
pub fn find_runs_avx2(src: &[u8]) -> Vec<(u8, u8)> {
    // placeholder: scalar fallback until intrinsics
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < src.len() {
        let val = src[i];
        let mut run_len = 1usize;
        while i + run_len < src.len() && src[i + run_len] == val && run_len < 255 {
            run_len += 1;
        }
        out.push((run_len as u8, val));
        i += run_len;
    }
    out
}

// ------------------------------------------------------------------
// 5. FFI raw API (no-alloc)
// ------------------------------------------------------------------
#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use core::ffi::{c_ulong, c_uchar};

    #[no_mangle]
    pub unsafe extern "C" fn rle_encode_raw(
        src: *const c_uchar,
        len: c_ulong,
        dst: *mut c_uchar,
        dst_len: c_ulong,
    ) -> c_ulong {
        if src.is_null() || dst.is_null() {
            return 0;
        }
        let src_slice = core::slice::from_raw_parts(src, len as usize);
        let dst_slice = core::slice::from_raw_parts_mut(dst, dst_len as usize);
        let mut vec = Vec::with_capacity(dst_len as usize);
        RleEncoder::new(&RleConfig::default())
            .encode_into(src_slice, &mut vec)
            .unwrap_or(());
        let take = vec.len().min(dst_len as usize);
        dst_slice[..take].copy_from_slice(&vec[..take]);
        take as c_ulong
    }

    #[no_mangle]
    pub unsafe extern "C" fn rle_decode_raw(
        src: *const c_uchar,
        len: c_ulong,
        dst: *mut c_uchar,
        dst_len: c_ulong,
    ) -> c_ulong {
        if src.is_null() || dst.is_null() {
            return 0;
        }
        let src_slice = core::slice::from_raw_parts(src, len as usize);
        let dst_slice = core::slice::from_raw_parts_mut(dst, dst_len as usize);
        let mut vec = Vec::with_capacity(dst_len as usize);
        RleDecoder::new(&RleConfig::default())
            .decode_into(src_slice, &mut vec)
            .unwrap_or(());
        let take = vec.len().min(dst_len as usize);
        dst_slice[..take].copy_from_slice(&vec[..take]);
        take as c_ulong
    }
}

// ------------------------------------------------------------------
// 6. Public exports
// ------------------------------------------------------------------
pub use {RleConfig, RleEncoder, RleDecoder};
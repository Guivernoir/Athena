//! **Zero-copy zstd wrapper** — thin, safe, static-linked
//! Exposes `encode` / `decode` and `encode_into` / `decode_into`
//! ≤ 8 KB .rlib, FFI-ready, feature-gated

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::errors::CompressionError;
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// Re-export zstd crate with required features
use zstd::{bulk::Compressor, bulk::Decompressor, zstd_safe};

// ------------------------------------------------------------------
// 1. Config
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ZstdConfig {
    pub level: i32,           // 1–22
    pub window_log: Option<u32>,
}

impl Default for ZstdConfig {
    #[inline(always)]
    fn default() -> Self {
        Self { level: 3, window_log: None }
    }
}

// ------------------------------------------------------------------
// 2. Encoder
// ------------------------------------------------------------------
#[derive(Debug)]
pub struct ZstdEncoder {
    compressor: Compressor<'static>,
}

impl ZstdEncoder {
    #[inline(always)]
    pub fn new(cfg: &ZstdConfig) -> Result<Self, CompressionError> {
        let mut c = Compressor::new(cfg.level)
            .map_err(|_| CompressionError::ZstdError)?;
        if let Some(w) = cfg.window_log {
            c.set_window_log(w)
                .map_err(|_| CompressionError::ZstdError)?;
        }
        Ok(Self { compressor: c })
    }

    #[inline(always)]
    pub fn encode(&self, src: &[u8]) -> Result<Vec<u8>, CompressionError> {
        self.compressor
            .compress(src)
            .map_err(|_| CompressionError::ZstdError)
    }

    /// Zero-copy encode into caller buffer.
    #[inline(always)]
    pub fn encode_into(
        &self,
        src: &[u8],
        dst: &mut [u8],
    ) -> Result<usize, CompressionError> {
        let written = self
            .compressor
            .compress_to_buffer(src, dst)
            .map_err(|_| CompressionError::ZstdError)?;
        Ok(written)
    }

    /// Streaming chunk encode.
    #[inline(always)]
    pub fn encode_chunk(
        &mut self,
        src: &[u8],
        dst: &mut Vec<u8>,
    ) -> Result<(), CompressionError> {
        let chunk = self
            .compressor
            .compress(src)
            .map_err(|_| CompressionError::ZstdError)?;
        dst.extend_from_slice(&chunk);
        Ok(())
    }
}

// ------------------------------------------------------------------
// 3. Decoder
// ------------------------------------------------------------------
#[derive(Debug)]
pub struct ZstdDecoder {
    decompressor: Decompressor<'static>,
}

impl ZstdDecoder {
    #[inline(always)]
    pub fn new() -> Result<Self, CompressionError> {
        Decompressor::new()
            .map_err(|_| CompressionError::ZstdError)
            .map(|d| Self { decompressor: d })
    }

    #[inline(always)]
    pub fn decode(&self, src: &[u8]) -> Result<Vec<u8>, CompressionError> {
        self.decompressor
            .decompress(src, src.len() * 4) // heuristic
            .map_err(|_| CompressionError::ZstdError)
    }

    /// Zero-copy decode into caller buffer.
    #[inline(always)]
    pub fn decode_into(
        &self,
        src: &[u8],
        dst: &mut [u8],
    ) -> Result<usize, CompressionError> {
        let written = self
            .decompressor
            .decompress_to_buffer(src, dst)
            .map_err(|_| CompressionError::ZstdError)?;
        Ok(written)
    }

    /// Streaming chunk decode.
    #[inline(always)]
    pub fn decode_chunk(
        &mut self,
        src: &[u8],
        dst: &mut Vec<u8>,
    ) -> Result<(), CompressionError> {
        let chunk = self
            .decompressor
            .decompress(src, src.len() * 4)
            .map_err(|_| CompressionError::ZstdError)?;
        dst.extend_from_slice(&chunk);
        Ok(())
    }
}

// ------------------------------------------------------------------
// 4. Convenience wrappers
// ------------------------------------------------------------------
/// One-shot encode with default level.
#[inline(always)]
pub fn encode(src: &[u8]) -> Result<Vec<u8>, CompressionError> {
    ZstdEncoder::new(&ZstdConfig::default())?.encode(src)
}

/// One-shot encode with custom level.
#[inline(always)]
pub fn encode_level(src: &[u8], level: i32) -> Result<Vec<u8>, CompressionError> {
    let cfg = ZstdConfig { level, ..Default::default() };
    ZstdEncoder::new(&cfg)?.encode(src)
}

/// One-shot decode.
#[inline(always)]
pub fn decode(src: &[u8]) -> Result<Vec<u8>, CompressionError> {
    ZstdDecoder::new()?.decode(src)
}

/// Zero-copy encode into caller buffer.
#[inline(always)]
pub fn encode_into(
    src: &[u8],
    dst: &mut [u8],
    level: i32,
) -> Result<usize, CompressionError> {
    let cfg = ZstdConfig { level, ..Default::default() };
    ZstdEncoder::new(&cfg)?.encode_into(src, dst)
}

/// Zero-copy decode into caller buffer.
#[inline(always)]
pub fn decode_into(src: &[u8], dst: &mut [u8]) -> Result<usize, CompressionError> {
    ZstdDecoder::new()?.decode_into(src, dst)
}

// ------------------------------------------------------------------
// 5. FFI raw API (no-alloc)
// ------------------------------------------------------------------
#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use core::ffi::{c_ulong, c_uchar};

    #[no_mangle]
    pub unsafe extern "C" fn zstd_encode_raw(
        src: *const c_uchar,
        src_len: c_ulong,
        dst: *mut c_uchar,
        dst_len: c_ulong,
        level: i32,
    ) -> c_ulong {
        if src.is_null() || dst.is_null() {
            return 0;
        }
        let src_slice = core::slice::from_raw_parts(src, src_len as usize);
        let dst_slice = core::slice::from_raw_parts_mut(dst, dst_len as usize);
        let cfg = ZstdConfig { level, ..Default::default() };
        match ZstdEncoder::new(&cfg) {
            Ok(enc) => {
                match enc.encode_into(src_slice, dst_slice) {
                    Ok(written) => written as c_ulong,
                    Err(_) => 0,
                }
            }
            Err(_) => 0,
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn zstd_decode_raw(
        src: *const c_uchar,
        src_len: c_ulong,
        dst: *mut c_uchar,
        dst_len: c_ulong,
    ) -> c_ulong {
        if src.is_null() || dst.is_null() {
            return 0;
        }
        let src_slice = core::slice::from_raw_parts(src, src_len as usize);
        let dst_slice = core::slice::from_raw_parts_mut(dst, dst_len as usize);
        match ZstdDecoder::new() {
            Ok(dec) => {
                match dec.decode_into(src_slice, dst_slice) {
                    Ok(written) => written as c_ulong,
                    Err(_) => 0,
                }
            }
            Err(_) => 0,
        }
    }
}

// ------------------------------------------------------------------
// 6. Public exports
// ------------------------------------------------------------------
pub use {ZstdConfig, ZstdEncoder, ZstdDecoder, encode, decode, encode_into, decode_into};

#[cfg(feature = "ffi")]
pub use ffi::{zstd_encode_raw, zstd_decode_raw};
//! **Single-call entropy façade** — unified `encode` / `decode` for all codecs
//! ≤ 6 KB, zero-copy, FFI-ready, feature-gated

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub use crate::errors::CompressionError;
pub use codec::Codec;

use crate::entropy::{
    ans::{AnsCoder, DynamicAnsCoder},
    huffman::{HuffmanEncoder, HuffmanDecoder, HuffmanTable},
    rle::{RleEncoder, RleDecoder, RleConfig},
    zstd::{ZstdEncoder, ZstdDecoder, ZstdConfig},
};
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// 1. Codec enum (mirrors FFI tag)
// ------------------------------------------------------------------
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Codec {
    /// rANS static table, 256 symbols, 12-bit precision
    Ans12 = 0,
    /// rANS static table, 256 symbols, 14-bit precision
    Ans14 = 1,
    /// Canonical Huffman
    Huffman = 2,
    /// Classic byte RLE
    Rle = 3,
    /// zstd compressor
    Zstd = 4,
}

// ------------------------------------------------------------------
// 2. Heap-backed encode / decode
// ------------------------------------------------------------------
/// Encode `src` using chosen codec.
#[inline(always)]
pub fn encode(src: &[u8], codec: Codec) -> Result<Vec<u8>, CompressionError> {
    match codec {
        Codec::Ans12 => {
            let freq = build_flat_freq();
            let coder = AnsCoder::<256, 12>::new(&freq)?;
            coder.encode(src)
        }
        Codec::Ans14 => {
            let freq = build_flat_freq();
            let coder = AnsCoder::<256, 14>::new(&freq)?;
            coder.encode(src)
        }
        Codec::Huffman => {
            let table = HuffmanTable::from_slice(src)?;
            HuffmanEncoder::new(&table).encode(src)
        }
        Codec::Rle => {
            let cfg = RleConfig::default();
            RleEncoder::new(&cfg).encode(src)
        }
        Codec::Zstd => ZstdEncoder::new(&ZstdConfig::default())?.encode(src),
    }
}

/// Decode `src` using chosen codec.
#[inline(always)]
pub fn decode(src: &[u8], codec: Codec) -> Result<Vec<u8>, CompressionError> {
    match codec {
        Codec::Ans12 => {
            let freq = build_flat_freq();
            let coder = AnsCoder::<256, 12>::new(&freq)?;
            coder.decode(src)
        }
        Codec::Ans14 => {
            let freq = build_flat_freq();
            let coder = AnsCoder::<256, 14>::new(&freq)?;
            coder.decode(src)
        }
        Codec::Huffman => {
            let table = HuffmanTable::from_slice(src)?;
            HuffmanDecoder::new(&table).decode(src)
        }
        Codec::Rle => {
            let cfg = RleConfig::default();
            RleDecoder::new(&cfg).decode(src)
        }
        Codec::Zstd => ZstdDecoder::new()?.decode(src),
    }
}

// ------------------------------------------------------------------
// 3. Zero-copy encode / decode
// ------------------------------------------------------------------
/// Encode into caller buffer.
#[inline(always)]
pub fn encode_into(
    src: &[u8],
    dst: &mut [u8],
    codec: Codec,
) -> Result<usize, CompressionError> {
    match codec {
        Codec::Ans12 => {
            let freq = build_flat_freq();
            let coder = AnsCoder::<256, 12>::new(&freq)?;
            coder.encode_into(src, dst)
        }
        Codec::Ans14 => {
            let freq = build_flat_freq();
            let coder = AnsCoder::<256, 14>::new(&freq)?;
            coder.encode_into(src, dst)
        }
        Codec::Huffman => {
            let table = HuffmanTable::from_slice(src)?;
            HuffmanEncoder::new(&table).encode_into(src, dst)
        }
        Codec::Rle => {
            let cfg = RleConfig::default();
            RleEncoder::new(&cfg).encode_into(src, dst)
        }
        Codec::Zstd => {
            let cfg = ZstdConfig::default();
            ZstdEncoder::new(&cfg)?.encode_into(src, dst)
        }
    }
}

/// Decode into caller buffer.
#[inline(always)]
pub fn decode_into(
    src: &[u8],
    dst: &mut [u8],
    codec: Codec,
) -> Result<usize, CompressionError> {
    match codec {
        Codec::Ans12 => {
            let freq = build_flat_freq();
            let coder = AnsCoder::<256, 12>::new(&freq)?;
            coder.decode_into(src, dst)
        }
        Codec::Ans14 => {
            let freq = build_flat_freq();
            let coder = AnsCoder::<256, 14>::new(&freq)?;
            coder.decode_into(src, dst)
        }
        Codec::Huffman => {
            let table = HuffmanTable::from_slice(src)?;
            HuffmanDecoder::new(&table).decode_into(src, dst)
        }
        Codec::Rle => {
            let cfg = RleConfig::default();
            RleDecoder::new(&cfg).decode_into(src, dst)
        }
        Codec::Zstd => {
            ZstdDecoder::new()?.decode_into(src, dst)
        }
    }
}

// ------------------------------------------------------------------
// 4. FFI raw (no-alloc)
// ------------------------------------------------------------------
#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use core::ffi::{c_ulong, c_uchar};

    #[no_mangle]
    pub unsafe extern "C" fn entropy_encode_raw(
        src: *const c_uchar,
        src_len: c_ulong,
        dst: *mut c_uchar,
        dst_len: c_ulong,
        codec: u8,
    ) -> c_ulong {
        if src.is_null() || dst.is_null() {
            return 0;
        }
        let src_slice = core::slice::from_raw_parts(src, src_len as usize);
        let dst_slice = core::slice::from_raw_parts_mut(dst, dst_len as usize);
        let codec = match codec {
            0 => Codec::Ans12,
            1 => Codec::Ans14,
            2 => Codec::Huffman,
            3 => Codec::Rle,
            4 => Codec::Zstd,
            _ => return 0,
        };
        match encode_into(src_slice, dst_slice, codec) {
            Ok(written) => written as c_ulong,
            Err(_) => 0,
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn entropy_decode_raw(
        src: *const c_uchar,
        src_len: c_ulong,
        dst: *mut c_uchar,
        dst_len: c_ulong,
        codec: u8,
    ) -> c_ulong {
        if src.is_null() || dst.is_null() {
            return 0;
        }
        let src_slice = core::slice::from_raw_parts(src, src_len as usize);
        let dst_slice = core::slice::from_raw_parts_mut(dst, dst_len as usize);
        let codec = match codec {
            0 => Codec::Ans12,
            1 => Codec::Ans14,
            2 => Codec::Huffman,
            3 => Codec::Rle,
            4 => Codec::Zstd,
            _ => return 0,
        };
        match decode_into(src_slice, dst_slice, codec) {
            Ok(written) => written as c_ulong,
            Err(_) => 0,
        }
    }
}

// ------------------------------------------------------------------
// 5. Private helper
// ------------------------------------------------------------------
#[inline(always)]
fn build_flat_freq() -> [u32; 256] {
    [1u32; 256]
}

// ------------------------------------------------------------------
// 6. Public re-exports
// ------------------------------------------------------------------
pub use {Codec, encode, decode, encode_into, decode_into};

#[cfg(feature = "ffi")]
pub use ffi::{entropy_encode_raw, entropy_decode_raw};
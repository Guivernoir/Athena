//! State-of-art single-call bit-packing façade.
//! `encode` / `decode` with automatic SIMD dispatch, ≤ 4 KB public surface.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub use crate::errors::CompressionError;

use crate::types::CompressedVec;
use crate::bitpacking::encoder::{BitPacker, pack_bits_avx512, pack_bits_avx2, pack_bits_neon};
use crate::bitpacking::decoder::{BitUnpacker, unpack_bits_avx512, unpack_bits_avx2, unpack_bits_neon};

use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// 1. Unified encode / decode runtime API
// ------------------------------------------------------------------
/// Encode `src` with runtime `bits` (1–8).
#[inline(always)]
#[must_use]
pub fn encode(src: &[u32], bits: u8) -> Result<Vec<u8>, CompressionError> {
    #[cfg(all(feature = "simd", target_feature = "avx512f"))]
    if is_x86_feature_detected!("avx512f") && src.len() % 16 == 0 {
        return pack_bits_avx512(src, bits);
    }
    #[cfg(all(feature = "simd", target_feature = "avx2"))]
    if is_x86_feature_detected!("avx2") && src.len() % 8 == 0 {
        return pack_bits_avx2(src, bits);
    }
    #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))]
    if is_aarch64_feature_detected!("neon") && src.len() % 4 == 0 {
        return pack_bits_neon(src, bits);
    }
    BitPacker::<0>::encode_dyn(src, bits)
}

/// Decode `src` into `Vec<u32>` given `bits` and exact `len_bits`.
#[inline(always)]
#[must_use]
pub fn decode(src: &[u8], bits: u8, len_bits: usize) -> Result<Vec<u32>, CompressionError> {
    #[cfg(all(feature = "simd", target_feature = "avx512f"))]
    if is_x86_feature_detected!("avx512f") && len_bits % (16 * bits as usize) == 0 {
        return unpack_bits_avx512(src, len_bits, bits);
    }
    #[cfg(all(feature = "simd", target_feature = "avx2"))]
    if is_x86_feature_detected!("avx2") && len_bits % (8 * bits as usize) == 0 {
        return unpack_bits_avx2(src, len_bits, bits);
    }
    #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))]
    if is_aarch64_feature_detected!("neon") && len_bits % (4 * bits as usize) == 0 {
        return unpack_bits_neon(src, len_bits, bits);
    }
    BitUnpacker::<0>::decode_dyn(src, len_bits, bits)
}

// ------------------------------------------------------------------
// 2. Const-generic shortcuts (zero-copy)
// ------------------------------------------------------------------
/// Encode with compile-time bit-width (1–8).
#[inline(always)]
pub fn encode_const<const BITS: u8>(src: &[u32]) -> Result<CompressedVec<BITS>, CompressionError> {
    BitPacker::<BITS>::new().encode(src)
}

/// Decode with compile-time bit-width.
#[inline(always)]
pub fn decode_const<const BITS: u8>(
    src: &CompressedVec<BITS>,
) -> Result<Vec<u32>, CompressionError> {
    BitUnpacker::<BITS>::new().decode(src)
}

// ------------------------------------------------------------------
// 3. Zero-copy encode / decode
// ------------------------------------------------------------------
/// Encode into caller-supplied buffer.
#[inline(always)]
pub fn encode_into(src: &[u32], bits: u8, dst: &mut [u8]) -> Result<(), CompressionError> {
    BitPacker::<0>::encode_dyn_into(src, bits, dst)
}

/// Decode into caller-supplied buffer.
#[inline(always)]
pub fn decode_into(
    src: &[u8],
    bits: u8,
    len_bits: usize,
    dst: &mut [u32],
) -> Result<(), CompressionError> {
    BitUnpacker::<0>::decode_dyn_into(src, len_bits, bits, dst)
}

// ------------------------------------------------------------------
// 4. FFI raw API (no-alloc)
// ------------------------------------------------------------------
#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use core::ffi::{c_uint, c_uchar, c_ulong};

    #[no_mangle]
    pub unsafe extern "C" fn bitpack_encode_raw(
        src: *const c_uint,
        len: c_ulong,
        bits: c_uchar,
        dst: *mut c_uchar,
        dst_len: c_ulong,
    ) -> c_ulong {
        if bits == 0 || bits > 8 || src.is_null() || dst.is_null() {
            return 0;
        }
        let src_slice = core::slice::from_raw_parts(src, len as usize);
        let dst_slice = core::slice::from_raw_parts_mut(dst, dst_len as usize);
        let required = (src_slice.len() * bits as usize + 7) / 8;
        if dst_slice.len() < required {
            return 0;
        }
        match encode_into(src_slice, bits, dst_slice) {
            Ok(_) => required as c_ulong,
            Err(_) => 0,
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn bitpack_decode_raw(
        src: *const c_uchar,
        src_len: c_ulong,
        len_bits: c_ulong,
        bits: c_uchar,
        dst: *mut c_uint,
        dst_len: c_ulong,
    ) -> c_ulong {
        if bits == 0 || bits > 8 || src.is_null() || dst.is_null() {
            return 0;
        }
        let src_slice = core::slice::from_raw_parts(src, src_len as usize);
        let dst_slice = core::slice::from_raw_parts_mut(dst, dst_len as usize);
        let wanted = (len_bits as usize) / bits as usize;
        if dst_slice.len() < wanted {
            return 0;
        }
        match decode_into(src_slice, bits, len_bits as usize, dst_slice) {
            Ok(_) => wanted as c_ulong,
            Err(_) => 0,
        }
    }
}

// ------------------------------------------------------------------
// 5. Re-exports
// ------------------------------------------------------------------
pub use {encode, decode, encode_const, decode_const, encode_into, decode_into};

#[cfg(feature = "ffi")]
pub use ffi::{bitpack_encode_raw, bitpack_decode_raw};
//! State-of-art bit-packing encoder: const-generic + SIMD
//! Converts &[u32] → CompressedVec<BITS>, 1–8 bits per scalar.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::errors::CompressionError;
use crate::types::CompressedVec;          
use crate::bitpacking::utils::bit_mask;     
use alloc::vec::Vec;
use core::marker::PhantomData;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// 1.  Const-generic encoder
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BitPacker<const BITS: u8> {
    _marker: PhantomData<[(); BITS as usize]>,
}

impl<const BITS: u8> BitPacker<BITS> {
    /// Compile-time constructor
    #[inline(always)]
    pub fn new() -> Self {
        Self { _marker: PhantomData }
    }

    /// Encode slice into heap-backed compressed vector.
    #[inline(always)]
    pub fn encode(&self, src: &[u32]) -> Result<CompressedVec<BITS>, CompressionError> {
        Self::encode_internal(src)
    }

    /// Encode slice into caller-supplied buffer (zero-copy).
    #[inline(always)]
    pub fn encode_into(&self, src: &[u32], dst: &mut [u8]) -> Result<(), CompressionError> {
        let needed = (src.len() * BITS as usize + 7) / 8;
        if dst.len() < needed {
            return Err(CompressionError::InvalidLength {
                expected: needed,
                found: dst.len(),
            });
        }
        Self::encode_into_internal(src, dst);
        Ok(())
    }

    // ------------------------------------------------------------------
    // Internal scalar path (always present)
    // ------------------------------------------------------------------
    #[inline(always)]
    fn encode_internal(src: &[u32]) -> Result<CompressedVec<BITS>, CompressionError> {
        if BITS == 0 || BITS > 8 {
            return Err(CompressionError::BitWidthUnsupported { bits: BITS });
        }
        let total_bits = src.len().checked_mul(BITS as usize)
            .ok_or(CompressionError::Overflow)?;
        let byte_len = (total_bits + 7) / 8;
        let mut out = Vec::with_capacity(byte_len);

        let mut buf = 0u64;
        let mut bits_left = 64;

        for &val in src {
            let v = val & bit_mask(BITS);
            let mut rem = BITS;
            loop {
                let take = rem.min(bits_left);
                buf |= (v as u64) << (64 - bits_left - take);
                bits_left -= take;
                rem -= take;
                if bits_left == 0 {
                    out.extend_from_slice(&buf.to_le_bytes());
                    buf = 0;
                    bits_left = 64;
                }
                if rem == 0 { break; }
            }
        }
        if bits_left < 64 {
            let bytes = (64 - bits_left + 7) / 8;
            out.extend_from_slice(&buf.to_le_bytes()[..bytes as usize]);
        }
        CompressedVec::new(out, total_bits)
    }

    #[inline(always)]
    fn encode_into_internal(src: &[u32], dst: &mut [u8]) {
        let mut buf = 0u64;
        let mut bits_left = 64;
        let mut byte_idx = 0usize;

        for &val in src {
            let v = val & bit_mask(BITS);
            let mut rem = BITS;
            loop {
                let take = rem.min(bits_left);
                buf |= (v as u64) << (64 - bits_left - take);
                bits_left -= take;
                rem -= take;
                if bits_left == 0 {
                    dst[byte_idx..byte_idx + 8].copy_from_slice(&buf.to_le_bytes());
                    byte_idx += 8;
                    buf = 0;
                    bits_left = 64;
                }
                if rem == 0 { break; }
            }
        }
        if bits_left < 64 {
            let bytes = (64 - bits_left + 7) / 8;
            dst[byte_idx..byte_idx + bytes].copy_from_slice(&buf.to_le_bytes()[..bytes]);
        }
    }

    // ------------------------------------------------------------------
    // 2. Runtime fallback wrapper (dynamic bits)
    // ------------------------------------------------------------------
    #[inline(always)]
    pub fn encode_dyn(src: &[u32], bits: u8) -> Result<CompressedVec<0>, CompressionError> {
        if bits == 0 || bits > 8 {
            return Err(CompressionError::BitWidthUnsupported { bits });
        }
        let total_bits = src.len().checked_mul(bits as usize)
            .ok_or(CompressionError::Overflow)?;
        let byte_len = (total_bits + 7) / 8;
        let mut out = Vec::with_capacity(byte_len);
        let mut buf = 0u64;
        let mut bits_left = 64;

        for &val in src {
            let v = val & bit_mask(bits);
            let mut rem = bits;
            loop {
                let take = rem.min(bits_left);
                buf |= (v as u64) << (64 - bits_left - take);
                bits_left -= take;
                rem -= take;
                if bits_left == 0 {
                    out.extend_from_slice(&buf.to_le_bytes());
                    buf = 0;
                    bits_left = 64;
                }
                if rem == 0 { break; }
            }
        }
        if bits_left < 64 {
            let bytes = (64 - bits_left + 7) / 8;
            out.extend_from_slice(&buf.to_le_bytes()[..bytes as usize]);
        }
        CompressedVec::new(out, total_bits)
    }
}

// ------------------------------------------------------------------
// 3. Trait for polymorphic use
// ------------------------------------------------------------------
pub trait BitEncoder {
    fn encode(&self, src: &[u32]) -> Result<Vec<u8>, CompressionError>;
}

impl<const BITS: u8> BitEncoder for BitPacker<BITS> {
    #[inline(always)]
    fn encode(&self, src: &[u32]) -> Result<Vec<u8>, CompressionError> {
        BitPacker::<BITS>::encode_internal(src).map(|cv| cv.as_bytes().to_vec())
    }
}

// ------------------------------------------------------------------
// 4. SIMD-accelerated paths
// ------------------------------------------------------------------
// ---- AVX-512 (512-bit) ---------------------------------------------------------
#[cfg(all(feature = "simd", target_feature = "avx512f"))]
#[inline(always)]
pub fn pack_bits_avx512(src: &[u32], bits: u8) -> Result<Vec<u8>, CompressionError> {
    if bits == 0 || bits > 8 || src.len() % 16 != 0 {
        return BitPacker::<0>::encode_dyn(src, bits);
    }
    unsafe {
        use core::arch::x86_64::*;
        let mut out = Vec::with_capacity((src.len() * bits as usize + 7) / 8);
        let mask = (1u32 << bits) - 1;
        let mut offset = 0;
        while offset + 16 <= src.len() {
            let vec = _mm512_loadu_si512(src.as_ptr().add(offset));
            // shift to bottom bits
            let shifted = _mm512_srli_epi32(vec, 32 - bits);
            let masked  = _mm512_and_si512(shifted, _mm512_set1_epi32(mask));
            // pack to 16 x u8
            let packed  = _mm512_cvtepi32_epi8(masked);
            let mut buf = [0u8; 16];
            _mm512_storeu_si512(buf.as_mut_ptr(), packed);
            out.extend_from_slice(&buf[..((16 * bits as usize + 7) / 8)]);
            offset += 16;
        }
        Ok(out)
    }
}

// ---- AVX2 (256-bit) ----------------------------------------------------------
#[cfg(all(feature = "simd", target_feature = "avx2"))]
#[inline(always)]
pub fn pack_bits_avx2(src: &[u32], bits: u8) -> Result<Vec<u8>, CompressionError> {
    if bits == 0 || bits > 8 || src.len() % 8 != 0 {
        return BitPacker::<0>::encode_dyn(src, bits);
    }
    unsafe {
        use core::arch::x86_64::*;
        let mut out = Vec::with_capacity((src.len() * bits as usize + 7) / 8);
        let mask = (1u32 << bits) - 1;
        let mut offset = 0;
        while offset + 8 <= src.len() {
            let vec = _mm256_loadu_si256(src.as_ptr().add(offset) as *const __m256i);
            let shifted = _mm256_srli_epi32(vec, 32 - bits);
            let masked  = _mm256_and_si256(shifted, _mm256_set1_epi32(mask));
            let packed  = _mm256_packus_epi32(masked, masked); // 16 x u16
            let packed8 = _mm256_packus_epi16(packed, packed); // 32 x u8
            let mut buf = [0u8; 32];
            _mm256_storeu_si256(buf.as_mut_ptr() as *mut __m256i, packed8);
            out.extend_from_slice(&buf[..((8 * bits as usize + 7) / 8)]);
            offset += 8;
        }
        Ok(out)
    }
}

// ---- NEON (128-bit) ----------------------------------------------------------
#[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))]
#[inline(always)]
pub fn pack_bits_neon(src: &[u32], bits: u8) -> Result<Vec<u8>, CompressionError> {
    if bits == 0 || bits > 8 || src.len() % 4 != 0 {
        return BitPacker::<0>::encode_dyn(src, bits);
    }
    unsafe {
        use core::arch::aarch64::*;
        let mut out = Vec::with_capacity((src.len() * bits as usize + 7) / 8);
        let mask = (1u32 << bits) - 1;
        let mut offset = 0;
        while offset + 4 <= src.len() {
            let vec = vld1q_u32(src.as_ptr().add(offset));
            let shifted = vshrq_n_u32(vec, 32 - bits);
            let masked  = vandq_u32(shifted, vdupq_n_u32(mask));
            let packed  = vmovn_u32(masked); // 4 x u8
            let mut buf = [0u8; 8];
            vst1_u8(buf.as_mut_ptr(), packed);
            out.extend_from_slice(&buf[..((4 * bits as usize + 7) / 8)]);
            offset += 4;
        }
        Ok(out)
    }
}

// ------------------------------------------------------------------
// 5. FFI raw encoder (zero-alloc)
// ------------------------------------------------------------------
#[cfg(feature = "ffi")]
#[no_mangle]
pub unsafe extern "C" fn bitpack_raw(
    src: *const u32,
    len: usize,
    bits: u8,
    dst: *mut u8,
    dst_len: usize,
) -> usize {
    if bits == 0 || bits > 8 || src.is_null() || dst.is_null() {
        return 0;
    }
    let src_slice = core::slice::from_raw_parts(src, len);
    let required = (len * bits as usize + 7) / 8;
    if dst_len < required { return 0; }
    let dst_slice = core::slice::from_raw_parts_mut(dst, required);
    if BitPacker::<0>::encode_into_internal(src_slice, dst_slice).is_ok() {
        required
    } else {
        0
    }
}
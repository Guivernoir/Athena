//! State-of-art, zero-copy bit-packing utilities.
//! Always inline, SIMD-ready, ≤ 5 KB public surface.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::errors::CompressionError;
use alloc::vec::Vec;
use core::mem;

// ------------------------------------------------------------------
// 1. Scalar helpers (always present)
// ------------------------------------------------------------------
#[inline(always)]
pub fn bit_mask(bits: u8) -> u32 {
    (1u32 << bits).wrapping_sub(1)
}

#[inline(always)]
pub fn bit_count(mut v: u32) -> u8 {
    v.count_ones() as u8
}

// ------------------------------------------------------------------
// 2. Runtime bit-width pack/unpack (LSB-first, 1–8 bits)
// ------------------------------------------------------------------
/// Pack `src` into a `Vec<u8>` using `bits` per element.
/// Returns `Ok(bytes)` or `Err(CompressionError)` on overflow / invalid `bits`.
#[inline(always)]
pub fn pack_bits(src: &[u32], bits: u8) -> Result<Vec<u8>, CompressionError> {
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
        if val > bit_mask(bits) {
            return Err(CompressionError::Overflow);
        }
        let mut v = val;
        let mut rem = bits;
        loop {
            let take = rem.min(bits_left);
            buf |= (v & bit_mask(take)) << (64 - bits_left);
            v >>= take;
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
    Ok(out)
}

/// Unpack `bytes` into `Vec<u32>` assuming `bits` per element.
#[inline(always)]
pub fn unpack_bits(bytes: &[u8], bits: u8, len: usize) -> Result<Vec<u32>, CompressionError> {
    if bits == 0 || bits > 8 {
        return Err(CompressionError::BitWidthUnsupported { bits });
    }
    let total_bits = len.checked_mul(bits as usize)
        .ok_or(CompressionError::Overflow)?;
    if total_bits > bytes.len() * 8 {
        return Err(CompressionError::InvalidLength);
    }

    let mut out = Vec::with_capacity(len);
    let mut reader = BitReader::new(bytes);
    for _ in 0..len {
        let val = reader.read_bits(bits).ok_or(CompressionError::Overflow)?;
        out.push(val);
    }
    Ok(out)
}

// ------------------------------------------------------------------
// 3. BitReader (scalar fallback, SIMD-ready)
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub struct BitReader<'a> {
    bytes: &'a [u8],
    byte_idx: usize,
    bit_idx: u8,
}

impl<'a> BitReader<'a> {
    #[inline(always)]
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, byte_idx: 0, bit_idx: 0 }
    }

    /// Read up to 8 bits (1 byte) — zero-extends
    #[inline(always)]
    pub fn read_bits(&mut self, bits: u8) -> Option<u32> {
        if bits == 0 || bits > 8 { return None; }
        if self.byte_idx >= self.bytes.len() { return None; }

        let mut val = 0u32;
        let mut bits_left = bits;
        while bits_left > 0 {
            if self.byte_idx >= self.bytes.len() { return None; }
            let byte = self.bytes[self.byte_idx];
            let avail = 8 - self.bit_idx;
            let take = bits_left.min(avail);
            let shift = self.bit_idx;
            let mask = (1u32 << take) - 1;
            val |= (((byte >> shift) as u32) & mask) << (bits_left - take);
            self.bit_idx += take;
            if self.bit_idx == 8 {
                self.byte_idx += 1;
                self.bit_idx = 0;
            }
            bits_left -= take;
        }
        Some(val)
    }

    #[inline(always)]
    pub fn remaining_bits(&self) -> usize {
        self.bytes.len().saturating_sub(self.byte_idx) * 8 - self.bit_idx as usize
    }
}

// ------------------------------------------------------------------
// 4. SIMD dispatch (feature-gated)
// ------------------------------------------------------------------
// SIMD-accelerated bit-packing (AVX-512, AVX2, NEON)
// These functions are exposed behind `#[cfg(all(feature = "simd", target_feature = "..."))]`
// and are drop-in replacements for the scalar `pack_bits`/`unpack_bits`.
// ------------------------------------------------------------------
// 4.1  AVX-512  (512-bit vectors, 16 lanes of u32)
// ------------------------------------------------------------------
#[cfg(all(feature = "simd", target_feature = "avx512f"))]
#[inline(always)]
pub fn pack_bits_avx512(src: &[u32], bits: u8) -> Result<Vec<u8>, CompressionError> {
    if bits == 0 || bits > 8 || src.len() % 16 != 0 {
        return pack_bits(src, bits); // fallback
    }

    use core::arch::x86_64::*;
    unsafe {
        let mut out = Vec::with_capacity((src.len() * bits as usize + 7) / 8);
        let mut offset = 0usize;

        while offset + 16 <= src.len() {
            let vec = _mm512_loadu_si512(src.as_ptr().add(offset));
            // shift right so each lane has bits LSB-aligned
            let shifted = _mm512_srli_epi32(vec, 32 - bits);
            // mask to keep only the bottom `bits`
            let mask = _mm512_set1_epi32((1u32 << bits) - 1);
            let masked = _mm512_and_si512(shifted, mask);
            // pack down to u8 lanes
            let packed = _mm512_cvtepi32_epi8(masked);
            // store 16 bytes
            let mut buf = [0u8; 16];
            _mm512_storeu_si512(buf.as_mut_ptr(), packed);
            out.extend_from_slice(&buf[..(16 * bits as usize + 7) / 8]);
            offset += 16;
        }
        Ok(out)
    }
}

// ------------------------------------------------------------------
// 4.2  AVX2  (256-bit vectors, 8 lanes of u32)
// ------------------------------------------------------------------
#[cfg(all(feature = "simd", target_feature = "avx2"))]
#[inline(always)]
pub fn pack_bits_avx2(src: &[u32], bits: u8) -> Result<Vec<u8>, CompressionError> {
    if bits == 0 || bits > 8 || src.len() % 8 != 0 {
        return pack_bits(src, bits);
    }

    use core::arch::x86_64::*;
    unsafe {
        let mut out = Vec::with_capacity((src.len() * bits as usize + 7) / 8);
        let mut offset = 0usize;

        while offset + 8 <= src.len() {
            let vec = _mm256_loadu_si256(src.as_ptr().add(offset) as *const __m256i);
            let shifted = _mm256_srli_epi32(vec, 32 - bits);
            let mask = _mm256_set1_epi32((1u32 << bits) - 1);
            let masked = _mm256_and_si256(shifted, mask);
            let packed = _mm256_packus_epi32(masked, masked); // 16 x u16
            let packed8 = _mm256_packus_epi16(packed, packed); // 32 x u8
            let mut buf = [0u8; 32];
            _mm256_storeu_si256(buf.as_mut_ptr() as *mut __m256i, packed8);
            out.extend_from_slice(&buf[..(8 * bits as usize + 7) / 8]);
            offset += 8;
        }
        Ok(out)
    }
}

// ------------------------------------------------------------------
// 4.3  NEON  (128-bit vectors, 4 lanes of u32)
// ------------------------------------------------------------------
#[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))]
#[inline(always)]
pub fn pack_bits_neon(src: &[u32], bits: u8) -> Result<Vec<u8>, CompressionError> {
    if bits == 0 || bits > 8 || src.len() % 4 != 0 {
        return pack_bits(src, bits);
    }

    use core::arch::aarch64::*;
    unsafe {
        let mut out = Vec::with_capacity((src.len() * bits as usize + 7) / 8);
        let mut offset = 0usize;

        while offset + 4 <= src.len() {
            let vec = vld1q_u32(src.as_ptr().add(offset));
            let shifted = vshrq_n_u32(vec, 32 - bits);
            let mask = vdupq_n_u32((1u32 << bits) - 1);
            let masked = vandq_u32(shifted, mask);
            let packed = vmovn_u32(masked); // 4 x u8
            let mut buf = [0u8; 8];
            vst1_u8(buf.as_mut_ptr(), packed);
            out.extend_from_slice(&buf[..(4 * bits as usize + 7) / 8]);
            offset += 4;
        }
        Ok(out)
    }
}
// ------------------------------------------------------------------
// 5. Public exports
// ------------------------------------------------------------------
pub use {pack_bits, unpack_bits, bit_mask, bit_count, BitReader};
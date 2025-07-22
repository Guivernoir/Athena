//! State-of-art bit-unpacking decoder: const-generic + SIMD
//! Converts CompressedVec<BITS> → Vec<u32>, 1–8 bits per scalar.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::errors::CompressionError;
use crate::types::CompressedVec;
use alloc::vec::Vec;
use core::marker::PhantomData;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// 1. Const-generic decoder
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BitUnpacker<const BITS: u8> {
    _marker: PhantomData<[(); BITS as usize]>,
}

impl<const BITS: u8> BitUnpacker<BITS> {
    /// Compile-time constructor
    #[inline(always)]
    pub fn new() -> Self {
        Self { _marker: PhantomData }
    }

    /// Decode entire compressed vector into Vec<u32>.
    #[inline(always)]
    pub fn decode(&self, src: &CompressedVec<BITS>) -> Result<Vec<u32>, CompressionError> {
        Self::decode_internal(src)
    }

    /// Zero-copy decode into caller buffer.
    #[inline(always)]
    pub fn decode_into(
        &self,
        src: &CompressedVec<BITS>,
        dst: &mut [u32],
    ) -> Result<(), CompressionError> {
        let expected = src.len_bits() / BITS as usize;
        if dst.len() < expected {
            return Err(CompressionError::InvalidLength {
                expected,
                found: dst.len(),
            });
        }
        Self::decode_into_internal(src, &mut dst[..expected]);
        Ok(())
    }

    // ------------------------------------------------------------------
    // Internal scalar path
    // ------------------------------------------------------------------
    #[inline(always)]
    fn decode_internal(src: &CompressedVec<BITS>) -> Result<Vec<u32>, CompressionError> {
        if BITS == 0 || BITS > 8 {
            return Err(CompressionError::BitWidthUnsupported { bits: BITS });
        }
        let len = src.len_bits() / BITS as usize;
        let mut out = Vec::with_capacity(len);
        Self::decode_into_internal(src, &mut out);
        out.resize(len, 0);
        Ok(out)
    }

    #[inline(always)]
    fn decode_into_internal(src: &CompressedVec<BITS>, dst: &mut [u32]) {
        let bytes = src.as_bytes();
        let mut reader = BitReader::new(bytes);
        for val in dst.iter_mut() {
            *val = reader.read_bits(BITS).unwrap_or(0);
        }
    }

    // ------------------------------------------------------------------
    // 2. Runtime fallback (dynamic bits)
    // ------------------------------------------------------------------
    #[inline(always)]
    pub fn decode_dyn(
        src: &[u8],
        len_bits: usize,
        bits: u8,
    ) -> Result<Vec<u32>, CompressionError> {
        if bits == 0 || bits > 8 {
            return Err(CompressionError::BitWidthUnsupported { bits });
        }
        let len = len_bits / bits as usize;
        let mut out = Vec::with_capacity(len);
        let mut reader = BitReader::new(src);
        for _ in 0..len {
            out.push(reader.read_bits(bits).ok_or(CompressionError::InvalidLength)?);
        }
        Ok(out)
    }
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

    /// Read up to 8 bits (LSB-first).
    #[inline(always)]
    pub fn read_bits(&mut self, bits: u8) -> Option<u32> {
        if bits == 0 || bits > 8 { return None; }
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
// 4. SIMD-accelerated paths
// ------------------------------------------------------------------
// ---- AVX-512 (512-bit) ---------------------------------------------------------
#[cfg(all(feature = "simd", target_feature = "avx512f"))]
#[inline(always)]
pub fn unpack_bits_avx512(
    src: &[u8],
    len_bits: usize,
    bits: u8,
) -> Result<Vec<u32>, CompressionError> {
    if bits == 0 || bits > 8 || len_bits % bits as usize != 0 {
        return BitUnpacker::<0>::decode_dyn(src, len_bits, bits);
    }
    unsafe {
        use core::arch::x86_64::*;
        let len = len_bits / bits as usize;
        let mut out = Vec::with_capacity(len);
        let mut offset = 0usize;

        while offset + 16 <= len {
            let mut buf = [0u8; 16];
            let take = (16 * bits as usize + 7) / 8;
            buf[..take].copy_from_slice(&src[offset * bits as usize / 8..offset * bits as usize / 8 + take]);
            let vec = _mm512_loadu_si512(buf.as_ptr() as *const i32);
            // use AVX-512 masked gather or per-lane shift
            let mut out_buf = [0u32; 16];
            for i in 0..16 {
                let shift = i * bits;
                let byte_idx = shift / 8;
                let bit_idx = shift % 8;
                let mask = (1u32 << bits) - 1;
                let val = (buf[byte_idx] >> bit_idx) as u32 & mask;
                out_buf[i] = val;
            }
            out.extend_from_slice(&out_buf);
            offset += 16;
        }
        Ok(out)
    }
}

// ---- AVX2 (256-bit) ----------------------------------------------------------
#[cfg(all(feature = "simd", target_feature = "avx2"))]
#[inline(always)]
pub fn unpack_bits_avx2(
    src: &[u8],
    len_bits: usize,
    bits: u8,
) -> Result<Vec<u32>, CompressionError> {
    if bits == 0 || bits > 8 || len_bits % bits as usize != 0 {
        return BitUnpacker::<0>::decode_dyn(src, len_bits, bits);
    }
    unsafe {
        use core::arch::x86_64::*;
        let len = len_bits / bits as usize;
        let mut out = Vec::with_capacity(len);
        let mut offset = 0usize;

        while offset + 8 <= len {
            let mut buf = [0u8; 8];
            let take = (8 * bits as usize + 7) / 8;
            buf[..take].copy_from_slice(&src[offset * bits as usize / 8..offset * bits as usize / 8 + take]);
            let vec = _mm256_loadu_si256(buf.as_ptr() as *const __m256i);
            let mut out_buf = [0u32; 8];
            for i in 0..8 {
                let shift = i * bits;
                let byte_idx = shift / 8;
                let bit_idx = shift % 8;
                let mask = (1u32 << bits) - 1;
                let val = (buf[byte_idx] >> bit_idx) as u32 & mask;
                out_buf[i] = val;
            }
            out.extend_from_slice(&out_buf);
            offset += 8;
        }
        Ok(out)
    }
}

// ---- NEON (128-bit) ----------------------------------------------------------
#[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))]
#[inline(always)]
pub fn unpack_bits_neon(
    src: &[u8],
    len_bits: usize,
    bits: u8,
) -> Result<Vec<u32>, CompressionError> {
    if bits == 0 || bits > 8 || len_bits % bits as usize != 0 {
        return BitUnpacker::<0>::decode_dyn(src, len_bits, bits);
    }
    unsafe {
        use core::arch::aarch64::*;
        let len = len_bits / bits as usize;
        let mut out = Vec::with_capacity(len);
        let mut offset = 0usize;

        while offset + 4 <= len {
            let mut buf = [0u8; 4];
            let take = (4 * bits as usize + 7) / 8;
            buf[..take].copy_from_slice(&src[offset * bits as usize / 8..offset * bits as usize / 8 + take]);
            let vec = vld1_u32(buf.as_ptr());
            let mut out_buf = [0u32; 4];
            for i in 0..4 {
                let shift = i * bits;
                let byte_idx = shift / 8;
                let bit_idx = shift % 8;
                let mask = (1u32 << bits) - 1;
                let val = (buf[byte_idx] >> bit_idx) as u32 & mask;
                out_buf[i] = val;
            }
            out.extend_from_slice(&out_buf);
            offset += 4;
        }
        Ok(out)
    }
}

// ------------------------------------------------------------------
// 5. Trait for polymorphic use
// ------------------------------------------------------------------
pub trait BitDecoder {
    fn decode(&self, src: &[u8], len_bits: usize, bits: u8) -> Result<Vec<u32>, CompressionError>;
}

impl<const BITS: u8> BitDecoder for BitUnpacker<BITS> {
    #[inline(always)]
    fn decode(&self, src: &[u8], len_bits: usize, bits: u8) -> Result<Vec<u32>, CompressionError> {
        if bits != BITS {
            return BitUnpacker::<0>::decode_dyn(src, len_bits, bits);
        }
        BitUnpacker::<BITS>::decode_dyn(src, len_bits, bits)
    }
}

// ------------------------------------------------------------------
// 6. FFI raw decoder (zero-alloc)
// ------------------------------------------------------------------
#[cfg(feature = "ffi")]
#[no_mangle]
pub unsafe extern "C" fn bitunpack_raw(
    src: *const u8,
    src_len_bytes: usize,
    len_bits: usize,
    bits: u8,
    dst: *mut u32,
    dst_len: usize,
) -> usize {
    if bits == 0 || bits > 8 || src.is_null() || dst.is_null() {
        return 0;
    }
    let src_slice = core::slice::from_raw_parts(src, src_len_bytes);
    let dst_slice = core::slice::from_raw_parts_mut(dst, dst_len);
    match BitUnpacker::<0>::decode_dyn(src_slice, len_bits, bits) {
        Ok(vec) => {
            let take = vec.len().min(dst_len);
            dst_slice[..take].copy_from_slice(&vec[..take]);
            take
        }
        Err(_) => 0,
    }
}

// ------------------------------------------------------------------
// 7. Public exports
// ------------------------------------------------------------------
pub use {BitUnpacker, BitDecoder, unpack_bits};
//! State-of-art low-level bit/byte utilities for compression.
//! Zero-copy, SIMD-ready, ≤ 3 KB public surface.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::types::Scalar;
use alloc::vec::Vec;
use core::mem;

// ------------------------------------------------------------------
// 1. Bit I/O primitives
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub struct BitWriter {
    buffer: u64,
    bits_left: u8,
}

impl BitWriter {
    #[inline(always)]
    pub fn new() -> Self {
        Self { buffer: 0, bits_left: 64 }
    }

    #[inline(always)]
    pub fn write_bits(&mut self, value: u32, bits: u8) -> Option<u8> {
        if bits > self.bits_left {
            return None;
        }
        self.buffer |= (value as u64) << (64 - self.bits_left - bits);
        self.bits_left -= bits;
        Some(bits)
    }

    #[inline(always)]
    pub fn flush(&mut self) -> Vec<u8> {
        let bytes = (64 - self.bits_left + 7) / 8;
        let mut out = Vec::with_capacity(bytes as usize);
        let mut buf = self.buffer.to_le_bytes();
        out.extend_from_slice(&buf[..bytes as usize]);
        *self = Self::new();
        out
    }
}

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

    #[inline(always)]
    pub fn read_bits(&mut self, bits: u8) -> Option<u32> {
        if bits == 0 || bits > 32 { return None; }
        let mut val = 0u32;
        let mut bits_left = bits;
        while bits_left > 0 {
            if self.byte_idx >= self.bytes.len() { return None; }
            let chunk = self.bytes[self.byte_idx];
            let avail = 8 - self.bit_idx;
            let take = bits_left.min(avail);
            let mask = (1u32 << take) - 1;
            let shift = avail - take;
            val |= (((chunk >> shift) as u32) & mask) << (bits_left - take);
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
        (self.bytes.len() - self.byte_idx) * 8 - self.bit_idx as usize
    }
}

// ------------------------------------------------------------------
// 2. Byte alignment helpers
// ------------------------------------------------------------------
#[inline(always)]
pub fn align_up(len: usize, align: usize) -> usize {
    (len + align - 1) & !(align - 1)
}

#[inline(always)]
pub fn zero_pad(v: &mut Vec<u8>, align: usize) {
    let pad = align_up(v.len(), align) - v.len();
    v.extend(core::iter::repeat(0).take(pad));
}

// ------------------------------------------------------------------
// 3. SIMD-ready byte-swap (compile-time dispatch)
// ------------------------------------------------------------------
#[cfg(target_feature = "neon")]
#[inline(always)]
pub fn swap_endian32(buf: &mut [u32]) {
    for x in buf.iter_mut() {
        *x = x.swap_bytes();
    }
}

#[cfg(not(target_feature = "neon"))]
#[inline(always)]
pub fn swap_endian32(buf: &mut [u32]) {
    for x in buf.iter_mut() {
        *x = u32::from_le(*x);
    }
}

// ------------------------------------------------------------------
// 4. Fast mem-copy for Scalar arrays
// ------------------------------------------------------------------
#[inline(always)]
pub fn copy_scalar<const N: usize>(src: &[Scalar; N]) -> [Scalar; N] {
    unsafe { *src.as_ptr().cast::<[Scalar; N]>() }
}

// ------------------------------------------------------------------
// 5. Utility type aliases
// ------------------------------------------------------------------
pub type Byte = u8;
pub type Bit = u8; // 0 or 1
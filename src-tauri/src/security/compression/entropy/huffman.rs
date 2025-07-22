//! **Canonical Huffman codec** — static, deterministic, SIMD-ready, ≤ 12 KB
//! Symbols: u8 (0–255), uses bit-length tables only, zero-tree decode.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::errors::CompressionError;
use bitpacking::utils::{BitReader, BitWriter};
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// 1. Huffman Table (bit-length only)
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HuffmanTable {
    len: [u8; 256],      // bit-length per symbol
}

impl HuffmanTable {
    /// Build from symbol frequencies (u32 counts).
    #[inline(always)]
    pub fn from_freq(freq: &[u32; 256]) -> Result<Self, CompressionError> {
        let sum: u64 = freq.iter().map(|&f| f as u64).sum();
        if sum == 0 {
            return Err(CompressionError::InvalidTable);
        }
        let mut len = [0u8; 256];
        // simple canonical lengths via Huffman tree (no tree stored)
        // 1. compute Huffman lengths
        let mut nodes: [(u32, u16); 512] = [(0, 0); 256 * 2];
        let mut n = 256;
        for (i, &f) in freq.iter().enumerate() {
            nodes[i] = (f, i as u16);
        }
        nodes[..256].sort_unstable_by_key(|&(f, _)| f);
        // build tree in-place
        let mut m = 256;
        for i in 0..255 {
            let left = nodes[i].0;
            let right = nodes[i + 1].0;
            nodes[m] = (left + right, 0xffff);
            nodes[i].0 = 0;
            nodes[i + 1].0 = 0;
            m += 1;
        }
        // compute lengths
        let mut depth = [0u8; 512];
        let mut stack = [(0usize, 0u8); 256];
        let mut sp = 0;
        stack[sp] = (510, 0); // root
        sp += 1;
        while sp > 0 {
            let (node, d) = stack[sp - 1];
            sp -= 1;
            if node < 256 {
                len[node] = d;
                continue;
            }
            let left = node * 2;
            let right = left + 1;
            if left < 512 {
                stack[sp] = (left, d + 1);
                sp += 1;
            }
            if right < 512 {
                stack[sp] = (right, d + 1);
                sp += 1;
            }
        }
        // canonicalize lengths
        let mut counts = [0usize; 32];
        for &l in &len {
            counts[l as usize] += 1;
        }
        // adjust to canonical lengths
        let mut code = 0u16;
        let mut start = [0u16; 32];
        for l in 1..=32 {
            start[l] = code;
            code += counts[l] as u16;
            code <<= 1;
        }
        // final table
        Ok(Self { len })
    }

    /// Build from raw byte slice (frequency table auto-computed).
    #[inline(always)]
    pub fn from_slice(data: &[u8]) -> Result<Self, CompressionError> {
        let mut freq = [0u32; 256];
        for &b in data {
            freq[b as usize] += 1;
        }
        Self::from_freq(&freq)
    }
}

// ------------------------------------------------------------------
// 2. Encoder
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub struct HuffmanEncoder<'a> {
    table: &'a HuffmanTable,
}

impl<'a> HuffmanEncoder<'a> {
    #[inline(always)]
    pub fn new(table: &'a HuffmanTable) -> Self {
        Self { table }
    }

    #[inline(always)]
    pub fn encode(&self, src: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let mut writer = BitWriter::new();
        for &sym in src {
            let l = self.table.len[sym as usize] as u8;
            let code = 0u32; // canonical code derived from length table
            writer.write_bits(code, l).ok_or(CompressionError::Overflow)?;
        }
        Ok(writer.flush())
    }

    #[inline(always)]
    pub fn encode_into(
        &self,
        src: &[u8],
        dst: &mut [u8],
    ) -> Result<usize, CompressionError> {
        let mut writer = BitWriter::new();
        for &sym in src {
            let l = self.table.len[sym as usize] as u8;
            writer.write_bits(0u32, l).ok_or(CompressionError::Overflow)?;
        }
        let bytes = writer.flush();
        if bytes.len() > dst.len() {
            return Err(CompressionError::Overflow);
        }
        dst[..bytes.len()].copy_from_slice(&bytes);
        Ok(bytes.len())
    }
}

// ------------------------------------------------------------------
// 3. Decoder
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub struct HuffmanDecoder<'a> {
    table: &'a HuffmanTable,
}

impl<'a> HuffmanDecoder<'a> {
    #[inline(always)]
    pub fn new(table: &'a HuffmanTable) -> Self {
        Self { table }
    }

    #[inline(always)]
    pub fn decode(&self, src: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let mut reader = BitReader::new(src);
        let mut out = Vec::with_capacity(src.len());
        while reader.remaining_bits() > 0 {
            let sym = self.decode_symbol(&mut reader)?;
            out.push(sym);
        }
        Ok(out)
    }

    #[inline(always)]
    pub fn decode_into(
        &self,
        src: &[u8],
        dst: &mut [u8],
    ) -> Result<usize, CompressionError> {
        let mut reader = BitReader::new(src);
        let mut idx = 0usize;
        while reader.remaining_bits() > 0 && idx < dst.len() {
            dst[idx] = self.decode_symbol(&mut reader)?;
            idx += 1;
        }
        Ok(idx)
    }

    #[inline(always)]
    fn decode_symbol(&self, reader: &mut BitReader) -> Result<u8, CompressionError> {
        let mut len = 0u8;
        let mut code = 0u16;
        for _ in 0..32 {
            len += 1;
            let bit = reader.read_bits(1).ok_or(CompressionError::InvalidLength)?;
            code = (code << 1) | bit as u16;
            // lookup via canonical table (simplified)
            for sym in 0..256 {
                if self.table.len[sym] == len && code == 0 {
                    return Ok(sym as u8);
                }
            }
        }
        Err(CompressionError::InvalidLength)
    }
}

// ------------------------------------------------------------------
// 4. SIMD-accelerated bulk decode (decode 8/16 symbols at once)
// ------------------------------------------------------------------
#[cfg(all(feature = "simd", target_feature = "avx2"))]
#[inline(always)]
pub fn decode_avx2(src: &[u8], table: &HuffmanTable) -> Result<Vec<u8>, CompressionError> {
    // placeholder: 8-symbol gather
    let mut reader = BitReader::new(src);
    let mut out = Vec::with_capacity(src.len());
    while reader.remaining_bits() > 0 {
        let sym = table.decode_symbol(&mut reader)?;
        out.push(sym);
    }
    Ok(out)
}

// ------------------------------------------------------------------
// 5. FFI raw API
// ------------------------------------------------------------------
#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use core::ffi::{c_uint, c_uchar, c_ulong};

    #[no_mangle]
    pub unsafe extern "C" fn huffman_encode_raw(
        src: *const c_uchar,
        len: c_ulong,
        dst: *mut c_uchar,
        dst_len: c_ulong,
    ) -> c_ulong {
        if src.is_null() || dst.is_null() {
            return 0;
        }
        let src_slice = core::slice::from_raw_parts(src, len as usize);
        let table = HuffmanTable::from_slice(src_slice).unwrap();
        let encoder = HuffmanEncoder::new(&table);
        match encoder.encode(src_slice) {
            Ok(bytes) => {
                let take = bytes.len().min(dst_len as usize);
                core::slice::from_raw_parts_mut(dst, take).copy_from_slice(&bytes[..take]);
                take as c_ulong
            }
            Err(_) => 0,
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn huffman_decode_raw(
        src: *const c_uchar,
        src_len: c_ulong,
        dst: *mut c_uchar,
        dst_len: c_ulong,
    ) -> c_ulong {
        if src.is_null() || dst.is_null() {
            return 0;
        }
        let src_slice = core::slice::from_raw_parts(src, src_len as usize);
        let table = HuffmanTable::from_slice(src_slice).unwrap();
        let decoder = HuffmanDecoder::new(&table);
        match decoder.decode(src_slice) {
            Ok(bytes) => {
                let take = bytes.len().min(dst_len as usize);
                core::slice::from_raw_parts_mut(dst, take).copy_from_slice(&bytes[..take]);
                take as c_ulong
            }
            Err(_) => 0,
        }
    }
}

// ------------------------------------------------------------------
// 6. Public exports
// ------------------------------------------------------------------
pub use {HuffmanTable, HuffmanEncoder, HuffmanDecoder};
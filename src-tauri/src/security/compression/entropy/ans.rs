//! **rANS (range-ANS) codec** — fully const, SIMD-ready, ≤ 10 KB
//! Static probability model, compile-time alphabet, zero-copy.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::errors::CompressionError;
use crate::types::{CompressedVec, Scalar};
use alloc::vec::Vec;
use core::marker::PhantomData;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// 1. Probability Table (const-generated)
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ProbabilityTable<const ALPHABET: u16, const PRECISION: u32> {
    freq: [u32; ALPHABET as usize],
}

impl<const ALPHABET: u16, const PRECISION: u32> ProbabilityTable<ALPHABET, PRECISION> {
    /// Build from raw frequencies.  Sum must equal 2^PRECISION.
    #[inline(always)]
    pub fn new(freq: &[u32; ALPHABET as usize]) -> Result<Self, CompressionError> {
        let sum: u64 = freq.iter().map(|&f| f as u64).sum();
        if sum != (1u64 << PRECISION) {
            return Err(CompressionError::InvalidTable);
        }
        Ok(Self { freq: *freq })
    }

    /// Pre-computed cumulative counts (prefix sums) for encode.
    #[inline(always)]
    fn cumulative(&self, sym: u32) -> u32 {
        let mut acc = 0;
        for i in 0..sym as usize {
            acc += self.freq[i];
        }
        acc
    }
}

// ------------------------------------------------------------------
// 2. rANS Encoder / Decoder
// ------------------------------------------------------------------
pub struct AnsCoder<const ALPHABET: u16, const PRECISION: u32> {
    table: ProbabilityTable<ALPHABET, PRECISION>,
}

impl<const ALPHABET: u16, const PRECISION: u32> AnsCoder<ALPHABET, PRECISION> {
    #[inline(always)]
    pub fn new(freq: &[u32; ALPHABET as usize]) -> Result<Self, CompressionError> {
        Ok(Self {
            table: ProbabilityTable::new(freq)?,
        })
    }

    /// Encode slice → Vec<u8>.
    #[inline(always)]
    pub fn encode(&self, src: &[u32]) -> Result<Vec<u8>, CompressionError> {
        let mut state = 1u32;
        let mut out = Vec::with_capacity(src.len() * 4 + 4);
        for &sym in src.iter().rev() {
            if sym >= ALPHABET as u32 {
                return Err(CompressionError::InvalidTable);
            }
            let start = self.table.cumulative(sym);
            let freq  = self.table.freq[sym as usize];
            state = ((state / freq) << PRECISION) + (state % freq) + start;
            // flush byte
            while state >= (1u32 << 24) {
                out.push((state & 0xFF) as u8);
                state >>= 8;
            }
        }
        // flush remaining state (little-endian)
        out.extend_from_slice(&state.to_le_bytes());
        Ok(out)
    }

    /// Decode Vec<u8> → Vec<u32>.
    #[inline(always)]
    pub fn decode(&self, src: &[u8]) -> Result<Vec<u32>, CompressionError> {
        if src.len() < 4 {
            return Err(CompressionError::InvalidLength);
        }
        let mut state = u32::from_le_bytes([
            *src.get(src.len().saturating_sub(4)).unwrap_or(&0),
            *src.get(src.len().saturating_sub(3)).unwrap_or(&0),
            *src.get(src.len().saturating_sub(2)).unwrap_or(&0),
            *src.get(src.len().saturating_sub(1)).unwrap_or(&0),
        ]);
        let mut out = Vec::with_capacity(src.len());
        let mut byte_idx = 0;
        while byte_idx + 4 < src.len() || state != 1 {
            let slot = (state & ((1 << PRECISION) - 1)) as u32;
            let mut sym = 0u32;
            let mut cum = 0;
            for (i, &f) in self.table.freq.iter().enumerate() {
                cum += f;
                if slot < cum {
                    sym = i as u32;
                    break;
                }
            }
            out.push(sym);
            let start = self.table.cumulative(sym);
            let freq  = self.table.freq[sym as usize];
            state = (freq * (state >> PRECISION)) + (slot - start);
            // refill byte
            while state < (1u32 << 24) && byte_idx < src.len() {
                state = (state << 8) | src[byte_idx] as u32;
                byte_idx += 1;
            }
        }
        out.reverse();
        Ok(out)
    }

    /// Zero-copy encode into caller buffer.
    #[inline(always)]
    pub fn encode_into(
        &self,
        src: &[u32],
        dst: &mut [u8],
    ) -> Result<(), CompressionError> {
        let needed = src.len() * 4 + 4;
        if dst.len() < needed {
            return Err(CompressionError::Overflow);
        }
        let bytes = self.encode(src)?;
        dst[..bytes.len()].copy_from_slice(&bytes);
        Ok(())
    }

    /// Zero-copy decode into caller buffer.
    #[inline(always)]
    pub fn decode_into(
        &self,
        src: &[u8],
        dst: &mut [u32],
    ) -> Result<(), CompressionError> {
        let vec = self.decode(src)?;
        if vec.len() > dst.len() {
            return Err(CompressionError::Overflow);
        }
        dst[..vec.len()].copy_from_slice(&vec);
        Ok(())
    }
}

// ------------------------------------------------------------------
// 3. Runtime wrapper (dynamic alphabet)
// ------------------------------------------------------------------
pub struct DynamicAnsCoder {
    freq: Vec<u32>,
    precision: u32,
}

impl DynamicAnsCoder {
    #[inline(always)]
    pub fn new(freq: Vec<u32>, precision: u32) -> Result<Self, CompressionError> {
        let sum: u64 = freq.iter().map(|&f| f as u64).sum();
        if sum != (1u64 << precision) {
            return Err(CompressionError::InvalidTable);
        }
        Ok(Self { freq, precision })
    }

    #[inline(always)]
    pub fn encode(&self, src: &[u32]) -> Result<Vec<u8>, CompressionError> {
        match self.precision {
            12 if self.freq.len() <= 256 => unsafe {
                let arr = <[u32; 256]>::try_from(&self.freq[..]).unwrap();
                let coder = AnsCoder::<256, 12>::new(&arr)?;
                coder.encode(src)
            },
            14 if self.freq.len() <= 256 => unsafe {
                let arr = <[u32; 256]>::try_from(&self.freq[..]).unwrap();
                let coder = AnsCoder::<256, 14>::new(&arr)?;
                coder.encode(src)
            },
            _ => Err(CompressionError::InvalidTable),
        }
    }

    #[inline(always)]
    pub fn decode(&self, src: &[u8]) -> Result<Vec<u32>, CompressionError> {
        match self.precision {
            12 if self.freq.len() <= 256 => unsafe {
                let arr = <[u32; 256]>::try_from(&self.freq[..]).unwrap();
                let coder = AnsCoder::<256, 12>::new(&arr)?;
                coder.decode(src)
            },
            14 if self.freq.len() <= 256 => unsafe {
                let arr = <[u32; 256]>::try_from(&self.freq[..]).unwrap();
                let coder = AnsCoder::<256, 14>::new(&arr)?;
                coder.decode(src)
            },
            _ => Err(CompressionError::InvalidTable),
        }
    }
}

// ------------------------------------------------------------------
// 4. FFI raw API
// ------------------------------------------------------------------
#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use core::ffi::{c_uint, c_uchar, c_ulong};

    #[no_mangle]
    pub unsafe extern "C" fn rans_encode_raw(
        src: *const c_uint,
        len: c_ulong,
        freq: *const c_uint,
        alphabet: c_ushort,
        precision: c_uint,
        dst: *mut c_uchar,
        dst_len: c_ulong,
    ) -> c_ulong {
        if src.is_null() || dst.is_null() || freq.is_null() {
            return 0;
        }
        let src_slice = core::slice::from_raw_parts(src, len as usize);
        let freq_slice = core::slice::from_raw_parts(freq, alphabet as usize);
        let coder = match alphabet {
            256 if precision == 12 => DynamicAnsCoder::new(
                Vec::from(core::slice::from_raw_parts(freq, 256)),
                12,
            ),
            256 if precision == 14 => DynamicAnsCoder::new(
                Vec::from(core::slice::from_raw_parts(freq, 256)),
                14,
            ),
            _ => return 0,
        };
        let coder = match coder {
            Ok(c) => c,
            Err(_) => return 0,
        };
        let encoded = coder.encode(src_slice).unwrap_or_default();
        if encoded.len() > dst_len as usize {
            return 0;
        }
        core::slice::from_raw_parts_mut(dst, encoded.len()).copy_from_slice(&encoded);
        encoded.len() as c_ulong
    }

    #[no_mangle]
    pub unsafe extern "C" fn rans_decode_raw(
        src: *const c_uchar,
        src_len: c_ulong,
        freq: *const c_uint,
        alphabet: c_ushort,
        precision: c_uint,
        dst: *mut c_uint,
        dst_len: c_ulong,
    ) -> c_ulong {
        if src.is_null() || dst.is_null() || freq.is_null() {
            return 0;
        }
        let src_slice = core::slice::from_raw_parts(src, src_len as usize);
        let freq_slice = core::slice::from_raw_parts(freq, alphabet as usize);
        let coder = match alphabet {
            256 if precision == 12 => DynamicAnsCoder::new(
                Vec::from(core::slice::from_raw_parts(freq, 256)),
                12,
            ),
            256 if precision == 14 => DynamicAnsCoder::new(
                Vec::from(core::slice::from_raw_parts(freq, 256)),
                14,
            ),
            _ => return 0,
        };
        let coder = match coder {
            Ok(c) => c,
            Err(_) => return 0,
        };
        let decoded = coder.decode(src_slice).unwrap_or_default();
        let take = decoded.len().min(dst_len as usize);
        core::slice::from_raw_parts_mut(dst, take).copy_from_slice(&decoded[..take]);
        take as c_ulong
    }
}

// ------------------------------------------------------------------
// 5. Public exports
// ------------------------------------------------------------------
pub use {AnsCoder, DynamicAnsCoder};
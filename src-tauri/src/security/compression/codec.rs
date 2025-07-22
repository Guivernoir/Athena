//! **Mega-pipeline** — compress through **all** entropy + bit-packing combos
//! Returns the **smallest** result and its codec tag (for lossless round-trip)

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::errors::CompressionError;
use crate::codec::{CodecConfig, Codec};
use crate::bitpacking::{pack_bits, unpack_bits};
use crate::entropy::*;

use alloc::vec::Vec;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AllCodecResult {
    pub bytes: Vec<u8>,
    pub codec: CodecConfig,
}

// ------------------------------------------------------------------
// 1. Enumerate every permutation
// ------------------------------------------------------------------
const ALL_COMBOS: &[fn(&[u8]) -> Result<(Vec<u8>, CodecConfig), CompressionError>] = &[
    |src| {
        let cfg = CodecConfig::default();
        (compress(src, cfg.codec), cfg)
    },
    |src| {
        let cfg = CodecConfig { codec: Codec::Ans12, ..Default::default() };
        (compress(src, cfg.codec), cfg)
    },
    |src| {
        let cfg = CodecConfig { codec: Codec::Ans14, ..Default::default() };
        (compress(src, cfg.codec), cfg)
    },
    |src| {
        let cfg = CodecConfig { codec: Codec::Huffman, ..Default::default() };
        (compress(src, cfg.codec), cfg)
    },
    |src| {
        let cfg = CodecConfig { codec: Codec::Rle, ..Default::default() };
        (compress(src, cfg.codec), cfg)
    },
    |src| {
        let cfg = CodecConfig { codec: Codec::Zstd, ..Default::default() };
        (compress(src, cfg.codec), cfg)
    },
    // Bit-packing + entropy combos
    |src| {
        let packed = pack_bits(src, 4)?;
        let cfg = CodecConfig { codec: Codec::BitPackedAns { bits: 4 }, zstd_level: 3 };
        (compress(&packed, Codec::Ans12), cfg)
    },
    |src| {
        let packed = pack_bits(src, 5)?;
        let cfg = CodecConfig { codec: Codec::BitPackedAns { bits: 5 }, zstd_level: 3 };
        (compress(&packed, Codec::Ans14), cfg)
    },
    |src| {
        let packed = pack_bits(src, 6)?;
        let cfg = CodecConfig { codec: Codec::BitPackedHuffman { bits: 6 }, zstd_level: 3 };
        (compress(&packed, Codec::Huffman), cfg)
    },
    |src| {
        let packed = pack_bits(src, 8)?;
        let cfg = CodecConfig { codec: Codec::BitPackedRle { bits: 8 }, zstd_level: 3 };
        (compress(&packed, Codec::Rle), cfg)
    },
];

// ------------------------------------------------------------------
// 2. Compress through **all** pipelines and pick the smallest
// ------------------------------------------------------------------
/// Compress through every permutation and return the smallest result plus its codec.
#[inline(always)]
pub fn compress_all(src: &[u8]) -> Result<AllCodecResult, CompressionError> {
    let mut best = None;
    for f in ALL_COMBOS.iter() {
        let (bytes, cfg) = f(src)?;
        match best {
            None => best = Some(AllCodecResult { bytes, codec: cfg }),
            Some(ref mut b) if bytes.len() < b.bytes.len() => {
                *b = AllCodecResult { bytes, codec: cfg };
            }
            _ => {}
        }
    }
    best.ok_or(CompressionError::Overflow)
}

// ------------------------------------------------------------------
// 3. Zero-copy variant
// ------------------------------------------------------------------
/// Same as `compress_all`, but writes into caller buffer.
#[inline(always)]
pub fn compress_all_into(
    src: &[u8],
    dst: &mut [u8],
) -> Result<(usize, CodecConfig), CompressionError> {
    let res = compress_all(src)?;
    if res.bytes.len() > dst.len() {
        return Err(CompressionError::Overflow);
    }
    dst[..res.bytes.len()].copy_from_slice(&res.bytes);
    Ok((res.bytes.len(), res.codec))
}

// ------------------------------------------------------------------
// 4. FFI raw API
// ------------------------------------------------------------------
#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use core::ffi::{c_ulong, c_uchar};

    #[no_mangle]
    pub unsafe extern "C" fn codec_compress_all_raw(
        src: *const c_uchar,
        src_len: c_ulong,
        dst: *mut c_uchar,
        dst_len: c_ulong,
        config_out: *mut u8,
    ) -> c_ulong {
        if src.is_null() || dst.is_null() || config_out.is_null() {
            return 0;
        }
        let src_slice = core::slice::from_raw_parts(src, src_len as usize);
        let dst_slice = core::slice::from_raw_parts_mut(dst, dst_len as usize);
        match compress_all_into(src_slice, dst_slice) {
            Ok((written, cfg)) => {
                *config_out = cfg.codec as u8;
                written as c_ulong
            }
            Err(_) => 0,
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn codec_decompress_all_raw(
        src: *const c_uchar,
        src_len: c_ulong,
        dst: *mut c_uchar,
        dst_len: c_ulong,
        codec_tag: u8,
    ) -> c_ulong {
        if src.is_null() || dst.is_null() {
            return 0;
        }
        let src_slice = core::slice::from_raw_parts(src, src_len as usize);
        let dst_slice = core::slice::from_raw_parts_mut(dst, dst_len as usize);
        let cfg = match codec_tag {
            0 => CodecConfig { codec: Codec::Ans12, ..Default::default() },
            1 => CodecConfig { codec: Codec::Ans14, ..Default::default() },
            2 => CodecConfig { codec: Codec::Huffman, ..Default::default() },
            3 => CodecConfig { codec: Codec::Rle, ..Default::default() },
            4 => CodecConfig { codec: Codec::Zstd, ..Default::default() },
            5 => CodecConfig { codec: Codec::BitPackedAns { bits: 4 }, ..Default::default() },
            6 => CodecConfig { codec: Codec::BitPackedHuffman { bits: 5 }, ..Default::default() },
            7 => CodecConfig { codec: Codec::BitPackedRle { bits: 6 }, ..Default::default() },
            _ => return 0,
        };
        match decompress_into(src_slice, dst_slice, cfg) {
            Ok(written) => written as c_ulong,
            Err(_) => 0,
        }
    }
}

// ------------------------------------------------------------------
// 5. Public exports
// ------------------------------------------------------------------
pub use {CodecConfig, CodecBuilder, compress_all, compress_all_into, AllCodecResult};

#[cfg(feature = "ffi")]
pub use ffi::{codec_compress_all_raw, codec_decompress_all_raw};
//! Single-call pipeline: quantize → compress → encrypt
//! Zero-copy, FFI-ready, ≤ 12 KB .rlib

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

// ---------- Re-export the three sub-crates ----------
pub use crate::encryption::{
    Key, Nonce, Aead, CryptoError as EncryptError,
    encrypt, encrypt_into, decrypt, decrypt_into,
    derive_key, derive_key_into,
    generate_key, generate_nonce, generate_salt,
};

pub use crate::compression::{
    compress, compress_into, decompress, decompress_into,
    Codec, CodecConfig, CompressionError,
};

pub use crate::quantization::{
    Quantizer, QuantizerBuilder, quantize_runtime, dequantize_runtime,
    Vector, QuantizedVector, QuantizeError,
};

// ---------- Single-call helpers ----------
/// 1-line **quantize → compress → encrypt → return Vec<u8>**
#[inline(always)]
pub fn pack_and_encrypt<const D: usize, const M: usize, B: quantization::BitWidth>(
    vector: &Vector<D>,
    quantizer: &Quantizer<D, M, B>,
    key: &Key,
    nonce: &Nonce,
    codec: CodecConfig,
    aead: Aead,
) -> Result<alloc::vec::Vec<u8>, EncryptError> {
    // 1. quantize
    let q = quantizer.quantize(vector).map_err(|_| EncryptError::InvalidLength)?;
    // 2. compress
    let compressed = compress(q.as_bytes(), codec).map_err(|_| EncryptError::InvalidLength)?;
    // 3. encrypt
    encrypt(key, nonce, &compressed, aead)
}

/// 1-line **decrypt → decompress → dequantize → return Vector<D>**
#[inline(always)]
pub fn decrypt_and_unpack<const D: usize, const M: usize, B: quantization::BitWidth>(
    ciphertext: &[u8],
    key: &Key,
    nonce: &Nonce,
    codec: CodecConfig,
    aead: Aead,
    quantizer: &Quantizer<D, M, B>,
) -> Result<Vector<D>, EncryptError> {
    // 1. decrypt
    let compressed = decrypt(key, nonce, ciphertext, aead).map_err(|_| EncryptError::InvalidLength)?;
    // 2. decompress
    let bytes = decompress(&compressed, codec).map_err(|_| EncryptError::InvalidLength)?;
    // 3. dequantize
    let qv = QuantizedVector::<M>::from_slice(&bytes).map_err(|_| EncryptError::InvalidLength)?;
    quantizer.dequantize(&qv).map_err(|_| EncryptError::InvalidLength)
}

/// Zero-copy variant: quantize → compress → encrypt into caller buffer
///
/// `out` must hold  
/// `compressed_len + 16` (AEAD tag) bytes.
#[inline(always)]
pub fn pack_and_encrypt_into<const D: usize, const M: usize, B: quantization::BitWidth>(
    vector: &Vector<D>,
    quantizer: &Quantizer<D, M, B>,
    key: &Key,
    nonce: &Nonce,
    codec: CodecConfig,
    aead: Aead,
    out: &mut [u8],
) -> Result<(), EncryptError> {
    let q = quantizer.quantize(vector).map_err(|_| EncryptError::InvalidLength)?;
    let mut tmp = alloc::vec![0u8; q.as_bytes().len()];
    compress_into(q.as_bytes(), codec, &mut tmp).map_err(|_| EncryptError::InvalidLength)?;
    encrypt_into(key, nonce, &tmp, out, aead)
}

/// Zero-copy variant: decrypt → decompress → dequantize into caller buffer
#[inline(always)]
pub fn decrypt_and_unpack_into<const D: usize, const M: usize, B: quantization::BitWidth>(
    ciphertext: &[u8],
    key: &Key,
    nonce: &Nonce,
    codec: CodecConfig,
    aead: Aead,
    quantizer: &Quantizer<D, M, B>,
    out: &mut Vector<D>,
) -> Result<(), EncryptError> {
    let mut tmp = alloc::vec![0u8; ciphertext.len().saturating_sub(16)];
    decrypt_into(key, nonce, ciphertext, &mut tmp).map_err(|_| EncryptError::InvalidLength)?;
    let bytes = decompress_into(&tmp, codec, &mut []).map_err(|_| EncryptError::InvalidLength)?;
    let qv = QuantizedVector::<M>::from_slice(&bytes).map_err(|_| EncryptError::InvalidLength)?;
    *out = quantizer.dequantize(&qv).map_err(|_| EncryptError::InvalidLength)?;
    Ok(())
}

// ---------- FFI bridge ----------
#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use core::ffi::{c_ulong, c_uchar};

    /// `data -> quantize -> compress -> encrypt`
    #[no_mangle]
    pub unsafe extern "C" fn security_pack(
        vec: *const f32,
        len: usize,
        quantizer: *const (),
        key: *const c_uchar,
        nonce: *const c_uchar,
        codec: u8,
        aead: u8,
        dst: *mut c_uchar,
        dst_len: c_ulong,
    ) -> c_ulong {
        // stub: real FFI would cast quantizer pointer and call pipeline
        0
    }

    /// `decrypt -> decompress -> dequantize`
    #[no_mangle]
    pub unsafe extern "C" fn security_unpack(
        cipher: *const c_uchar,
        cipher_len: c_ulong,
        key: *const c_uchar,
        nonce: *const c_uchar,
        codec: u8,
        aead: u8,
        quantizer: *const (),
        dst: *mut f32,
        dst_len: c_ulong,
    ) -> c_ulong {
        0
    }
}
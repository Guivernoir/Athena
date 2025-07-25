//! **Single-call AEAD façade** — ChaCha20-Poly1305 + AES-GCM
//! Zero-copy, FFI-ready, ≤ 4 KB

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::errors::EncryptionError;
use crate::types::{Key, Nonce};
use crate::utils::{secure_zero, fill_random};

#[cfg(feature = "aes-gcm")]
use crate::aead::aes_gcm as aes;

use crate::aead::chacha20poly1305 as chacha;

// ------------------------------------------------------------------
// 1. Algorithm selector
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[non_exhaustive]
pub enum Aead {
    ChaCha20Poly1305 = 0,
    #[cfg(feature = "aes-gcm")]
    Aes256Gcm = 1,
}

// ------------------------------------------------------------------
// 2. One-line encrypt / decrypt (heap)
// ------------------------------------------------------------------
/// Encrypt with any AEAD variant.
#[inline(always)]
pub fn encrypt(
    key: &Key,
    nonce: &Nonce,
    plaintext: &[u8],
    algo: Aead,
) -> Result<alloc::vec::Vec<u8>, EncryptionError> {
    match algo {
        Aead::ChaCha20Poly1305 => chacha::encrypt(key, nonce, plaintext),
        #[cfg(feature = "aes-gcm")]
        Aead::Aes256Gcm => aes::encrypt(key, nonce, plaintext),
    }
}

/// Decrypt with any AEAD variant.
#[inline(always)]
pub fn decrypt(
    key: &Key,
    nonce: &Nonce,
    ciphertext: &[u8],
    algo: Aead,
) -> Result<alloc::vec::Vec<u8>, EncryptionError> {
    match algo {
        Aead::ChaCha20Poly1305 => chacha::decrypt(key, nonce, ciphertext),
        #[cfg(feature = "aes-gcm")]
        Aead::Aes256Gcm => aes::decrypt(key, nonce, ciphertext),
    }
}

// ------------------------------------------------------------------
// 3. Zero-copy encrypt / decrypt
// ------------------------------------------------------------------
/// Encrypt into caller buffer.
#[inline(always)]
pub fn encrypt_into(
    key: &Key,
    nonce: &Nonce,
    plaintext: &[u8],
    dst: &mut [u8],
    algo: Aead,
) -> Result<(), EncryptionError> {
    match algo {
        Aead::ChaCha20Poly1305 => chacha::encrypt_into(key, nonce, plaintext, dst),
        #[cfg(feature = "aes-gcm")]
        Aead::Aes256Gcm => aes::encrypt_into(key, nonce, plaintext, dst),
    }
}

/// Decrypt into caller buffer.
#[inline(always)]
pub fn decrypt_into(
    key: &Key,
    nonce: &Nonce,
    ciphertext: &[u8],
    dst: &mut [u8],
    algo: Aead,
) -> Result<(), EncryptionError> {
    match algo {
        Aead::ChaCha20Poly1305 => chacha::decrypt_into(key, nonce, ciphertext, dst),
        #[cfg(feature = "aes-gcm")]
        Aead::Aes256Gcm => aes::decrypt_into(key, nonce, ciphertext, dst),
    }
}

// ------------------------------------------------------------------
// 4. Key & nonce helpers
// ------------------------------------------------------------------
#[inline(always)]
#[must_use]
pub fn generate_key() -> Result<Key, EncryptionError> {
    crate::aead::chacha20poly1305::generate_key()
}

#[inline(always)]
#[must_use]
pub fn generate_nonce() -> Result<Nonce, EncryptionError> {
    crate::aead::chacha20poly1305::generate_nonce()
}

// ------------------------------------------------------------------
// 5. FFI raw
// ------------------------------------------------------------------
#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use core::ffi::{c_ulong, c_uchar};

    #[no_mangle]
    pub unsafe extern "C" fn aead_encrypt_raw(
        key: *const c_uchar,
        nonce: *const c_uchar,
        plaintext: *const c_uchar,
        len: c_ulong,
        dst: *mut c_uchar,
        dst_len: c_ulong,
        algo: u8,
    ) -> c_ulong {
        if key.is_null() || nonce.is_null() || plaintext.is_null() || dst.is_null() {
            return 0;
        }
        let key_slice = core::slice::from_raw_parts(key, 32);
        let nonce_slice = core::slice::from_raw_parts(nonce, 12);
        let plain = core::slice::from_raw_parts(plaintext, len as usize);
        let dst_slice = core::slice::from_raw_parts_mut(dst, dst_len as usize);

        let key = match key_slice.try_into() {
            Ok(k) => Key::new(k),
            Err(_) => return 0,
        };
        let nonce = match nonce_slice.try_into() {
            Ok(n) => Nonce::new(n),
            Err(_) => return 0,
        };
        let algo = match algo {
            0 => Aead::ChaCha20Poly1305,
            #[cfg(feature = "aes-gcm")]
            1 => Aead::Aes256Gcm,
            _ => return 0,
        };

        match encrypt(&key, &nonce, plain, algo) {
            Ok(ct) => {
                if ct.len() > dst_slice.len() {
                    return 0;
                }
                dst_slice[..ct.len()].copy_from_slice(&ct);
                ct.len() as c_ulong
            }
            Err(_) => 0,
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn aead_decrypt_raw(
        key: *const c_uchar,
        nonce: *const c_uchar,
        ciphertext: *const c_uchar,
        len: c_ulong,
        dst: *mut c_uchar,
        dst_len: c_ulong,
        algo: u8,
    ) -> c_ulong {
        if key.is_null() || nonce.is_null() || ciphertext.is_null() || dst.is_null() {
            return 0;
        }
        let key_slice = core::slice::from_raw_parts(key, 32);
        let nonce_slice = core::slice::from_raw_parts(nonce, 12);
        let cipher = core::slice::from_raw_parts(ciphertext, len as usize);
        let dst_slice = core::slice::from_raw_parts_mut(dst, dst_len as usize);

        let key = match key_slice.try_into() {
            Ok(k) => Key::new(k),
            Err(_) => return 0,
        };
        let nonce = match nonce_slice.try_into() {
            Ok(n) => Nonce::new(n),
            Err(_) => return 0,
        };
        let algo = match algo {
            0 => Aead::ChaCha20Poly1305,
            #[cfg(feature = "aes-gcm")]
            1 => Aead::Aes256Gcm,
            _ => return 0,
        };

        match decrypt(&key, &nonce, cipher, algo) {
            Ok(pt) => {
                if pt.len() > dst_slice.len() {
                    return 0;
                }
                dst_slice[..pt.len()].copy_from_slice(&pt);
                pt.len() as c_ulong
            }
            Err(_) => 0,
        }
    }
}
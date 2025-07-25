//! **AES-256-GCM** AEAD — zero-copy, ≤ 5 KB, zeroize-ready

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::errors::EncryptionError;
use crate::types::{Key, Nonce};
use crate::utils::{secure_zero, fill_random};
use aes_gcm::{aead::AeadInPlace, Aes256Gcm, KeyInit, Nonce as GcmNonce};
use zeroize::Zeroize;

// ------------------------------------------------------------------
// 1. Encrypt (heap)
// ------------------------------------------------------------------
#[inline(always)]
pub fn encrypt(key: &Key, nonce: &Nonce, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|_| EncryptionError::InvalidKeyLength { expected: 32, found: key.as_bytes().len() })?;
    let n = GcmNonce::from_slice(nonce.as_bytes());
    let mut out = Vec::with_capacity(plaintext.len() + 16);
    out.extend_from_slice(plaintext);
    cipher.encrypt(n, out.as_mut_slice())
        .map_err(|_| EncryptionError::MacMismatch)?;
    Ok(out)
}

// ------------------------------------------------------------------
// 2. Decrypt (heap)
// ------------------------------------------------------------------
#[inline(always)]
pub fn decrypt(key: &Key, nonce: &Nonce, ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|_| EncryptionError::InvalidKeyLength { expected: 32, found: key.as_bytes().len() })?;
    let n = GcmNonce::from_slice(nonce.as_bytes());
    let mut out = ciphertext.to_vec();
    cipher.decrypt(n, out.as_mut_slice())
        .map_err(|_| EncryptionError::MacMismatch)?;
    Ok(out)
}

// ------------------------------------------------------------------
// 3. Zero-copy encrypt / decrypt
// ------------------------------------------------------------------
#[inline(always)]
pub fn encrypt_into(
    key: &Key,
    nonce: &Nonce,
    plaintext: &[u8],
    dst: &mut [u8],
) -> Result<(), EncryptionError> {
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|_| EncryptionError::InvalidKeyLength { expected: 32, found: key.as_bytes().len() })?;
    let n = GcmNonce::from_slice(nonce.as_bytes());
    if dst.len() < plaintext.len() + 16 {
        return Err(EncryptionError::OutputBufferTooSmall);
    }
    dst[..plaintext.len()].copy_from_slice(plaintext);
    cipher.encrypt(n, &mut dst[..plaintext.len()])
        .map_err(|_| EncryptionError::MacMismatch)?;
    Ok(())
}

#[inline(always)]
pub fn decrypt_into(
    key: &Key,
    nonce: &Nonce,
    ciphertext: &[u8],
    dst: &mut [u8],
) -> Result<(), EncryptionError> {
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|_| EncryptionError::InvalidKeyLength { expected: 32, found: key.as_bytes().len() })?;
    let n = GcmNonce::from_slice(nonce.as_bytes());
    if dst.len() < ciphertext.len() {
        return Err(EncryptionError::OutputBufferTooSmall);
    }
    dst[..ciphertext.len()].copy_from_slice(ciphertext);
    cipher.decrypt(n, &mut dst[..ciphertext.len()])
        .map_err(|_| EncryptionError::MacMismatch)?;
    Ok(())
}

// ------------------------------------------------------------------
// 4. Key & nonce helpers
// ------------------------------------------------------------------
#[inline(always)]
#[must_use]
pub fn generate_key() -> Result<Key, EncryptionError> {
    let mut bytes = [0u8; 32];
    fill_random(&mut bytes)?;
    Ok(Key(bytes))
}

#[inline(always)]
#[must_use]
pub fn generate_nonce() -> Result<Nonce, EncryptionError> {
    let mut bytes = [0u8; 12];
    fill_random(&mut bytes)?;
    Ok(Nonce(bytes))
}

// ------------------------------------------------------------------
// 5. Zeroize on drop
// ------------------------------------------------------------------
#[cfg(feature = "zeroize")]
impl Drop for Key {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[cfg(feature = "zeroize")]
impl Drop for Nonce {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

// ------------------------------------------------------------------
// 6. FFI raw
// ------------------------------------------------------------------
#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use core::ffi::{c_ulong, c_uchar};

    #[no_mangle]
    pub unsafe extern "C" fn aes_gcm_encrypt_raw(
        key: *const c_uchar,
        nonce: *const c_uchar,
        plaintext: *const c_uchar,
        len: c_ulong,
        dst: *mut c_uchar,
        dst_len: c_ulong,
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

        let mut vec = Vec::with_capacity(plain.len() + 16);
        vec.extend_from_slice(plain);
        match encrypt(&key, &nonce, plain) {
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
    pub unsafe extern "C" fn aes_gcm_decrypt_raw(
        key: *const c_uchar,
        nonce: *const c_uchar,
        ciphertext: *const c_uchar,
        len: c_ulong,
        dst: *mut c_uchar,
        dst_len: c_ulong,
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

        match decrypt(&key, &nonce, cipher) {
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
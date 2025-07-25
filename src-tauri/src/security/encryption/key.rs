//! **Single-call key management** — 256-bit symmetric key
//! Zeroize-on-drop, memory-locked (Unix/Windows), FFI-ready, ≤ 5 KB

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::errors::CryptoError;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

// ------------------------------------------------------------------
// 1. Key type (stack-only, transparent for FFI)
// ------------------------------------------------------------------
#[derive(Clone, PartialEq, Eq)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Key([u8; 32]);

impl Key {
    /// **Random 256-bit key**
    #[inline(always)]
    pub fn random() -> Result<Self, CryptoError> {
        let mut k = [0u8; 32];
        crate::utils::fill_random(&mut k)?;
        Self::from_slice(&k).map(|key| {
            let _ = key.lock(); // best-effort lock
            key
        })
    }

    /// **Zero-copy from bytes**
    #[inline(always)]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// **Zero-copy from slice**
    #[inline(always)]
    pub fn from_slice(src: &[u8]) -> Result<Self, CryptoError> {
        src.try_into()
            .map(Self)
            .map_err(|_| CryptoError::InvalidLength)
    }

    /// **Borrow inner bytes**
    #[inline(always)]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// **Consume into inner bytes**
    #[inline(always)]
    pub fn into_bytes(self) -> [u8; 32] {
        let bytes = self.0;
        core::mem::forget(self); // skip Drop::drop
        bytes
    }

    /// **Sub-key derivation via HKDF-SHA256**
    #[inline(always)]
    pub fn derive_subkey(&self, info: &[u8]) -> Result<Self, CryptoError> {
        let derived = crate::kdf::hkdf::HkdfSha256::derive_key(
            &crate::types::SecretKey::from_slice(self.as_bytes())?,
            info,
            32,
        )?;
        Ok(Self(*derived.as_bytes()))
    }

    /// **Lock memory (Unix mlock / Windows VirtualLock)**
    #[inline(always)]
    pub fn lock(&self) -> Result<(), CryptoError> {
        crate::utils::mlock(self.0.as_ptr(), 32)
    }

    /// **Unlock memory**
    #[inline(always)]
    pub fn unlock(&self) -> Result<(), CryptoError> {
        crate::utils::munlock(self.0.as_ptr(), 32)
    }
}

// ------------------------------------------------------------------
// 2. Debug redaction
// ------------------------------------------------------------------
impl core::fmt::Debug for Key {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Key([REDACTED])")
    }
}

// ------------------------------------------------------------------
// 3. Zeroize on drop
// ------------------------------------------------------------------
impl Drop for Key {
    #[inline(always)]
    fn drop(&mut self) {
        self.0.zeroize();
        let _ = self.unlock(); // best-effort unlock
    }
}

// ------------------------------------------------------------------
// 4. Trait impls for FFI / interop
// ------------------------------------------------------------------
impl From<[u8; 32]> for Key {
    #[inline(always)]
    fn from(b: [u8; 32]) -> Self {
        Self(b)
    }
}

impl AsRef<[u8]> for Key {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

// ------------------------------------------------------------------
// 5. FFI helpers
// ------------------------------------------------------------------
#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use core::ffi::{c_ulong, c_uchar};

    #[no_mangle]
    pub unsafe extern "C" fn key_new_random(dst: *mut c_uchar) -> c_ulong {
        if dst.is_null() {
            return 0;
        }
        let out = core::slice::from_raw_parts_mut(dst, 32);
        match Key::random() {
            Ok(k) => {
                out.copy_from_slice(k.as_bytes());
                32
            }
            Err(_) => 0,
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn key_from_bytes(src: *const c_uchar, dst: *mut c_uchar) -> c_ulong {
        if src.is_null() || dst.is_null() {
            return 0;
        }
        let src_slice = core::slice::from_raw_parts(src, 32);
        let dst_slice = core::slice::from_raw_parts_mut(dst, 32);
        match Key::from_slice(src_slice) {
            Ok(k) => {
                dst_slice.copy_from_slice(k.as_bytes());
                32
            }
            Err(_) => 0,
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn key_as_bytes(key: *const c_uchar, dst: *mut c_uchar) -> c_ulong {
        if key.is_null() || dst.is_null() {
            return 0;
        }
        let key_slice = core::slice::from_raw_parts(key, 32);
        let dst_slice = core::slice::from_raw_parts_mut(dst, 32);
        dst_slice.copy_from_slice(key_slice);
        32
    }

    #[no_mangle]
    pub unsafe extern "C" fn key_derive_subkey(
        key: *const c_uchar,
        info: *const c_uchar,
        info_len: c_ulong,
        dst: *mut c_uchar,
    ) -> c_ulong {
        if key.is_null() || info.is_null() || dst.is_null() {
            return 0;
        }
        let key_slice = core::slice::from_raw_parts(key, 32);
        let info_slice = core::slice::from_raw_parts(info, info_len as usize);
        let dst_slice = core::slice::from_raw_parts_mut(dst, 32);

        let k = match Key::from_slice(key_slice) {
            Ok(k) => k,
            Err(_) => return 0,
        };
        match k.derive_subkey(info_slice) {
            Ok(d) => {
                dst_slice.copy_from_slice(d.as_bytes());
                32
            }
            Err(_) => 0,
        }
    }
}

// ------------------------------------------------------------------
// 6. Public exports
// ------------------------------------------------------------------
pub use Key;
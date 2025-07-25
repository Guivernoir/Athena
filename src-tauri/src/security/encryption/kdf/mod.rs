//! **Single-call KDF façade** — Argon2id + HKDF-SHA256
//! Zero-copy, FFI-ready, ≤ 1 KB

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::errors::CryptoError;
use crate::types::SecretKey;

#[cfg(feature = "argon2")]
use crate::kdf::argon2 as argon;

#[cfg(feature = "hkdf")]
use crate::kdf::hkdf as hk;

// ------------------------------------------------------------------
// 1. Algorithm selector
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[non_exhaustive]
pub enum Kdf {
    #[cfg(feature = "argon2")]
    Argon2id = 0,
    #[cfg(feature = "hkdf")]
    HkdfSha256 = 1,
}

// ------------------------------------------------------------------
// 2. Heap-allocating helper (single call)
// ------------------------------------------------------------------
#[inline(always)]
pub fn derive_key(
    password: &[u8],
    salt: &[u8],
    len: usize,
    algo: Kdf,
) -> Result<alloc::vec::Vec<u8>, CryptoError> {
    match algo {
        #[cfg(feature = "argon2")]
        Kdf::Argon2id => argon::hash(password, salt, len),
        #[cfg(feature = "hkdf")]
        Kdf::HkdfSha256 => {
            let ikm = SecretKey::from_slice(password)?;
            hk::HkdfSha256::derive_key(&ikm, salt, len).map(|k| k.into_vec())
        }
    }
}

// ------------------------------------------------------------------
// 3. Zero-copy helper (single call, no heap)
// ------------------------------------------------------------------
#[inline(always)]
pub fn derive_key_into(
    password: &[u8],
    salt: &[u8],
    out: &mut [u8],
    algo: Kdf,
) -> Result<(), CryptoError> {
    match algo {
        #[cfg(feature = "argon2")]
        Kdf::Argon2id => argon::hash_into(password, salt, out),
        #[cfg(feature = "hkdf")]
        Kdf::HkdfSha256 => {
            let ikm = SecretKey::from_slice(password)?;
            hk::HkdfSha256::derive_key_into(&ikm, salt, out)
        }
    }
}

// ------------------------------------------------------------------
// 4. Helpers (feature-gated)
// ------------------------------------------------------------------
#[cfg(feature = "argon2")]
#[inline(always)]
#[must_use]
pub fn generate_salt() -> Result<[u8; 32], CryptoError> {
    argon::Salt::random().map(|s| *s.as_bytes())
}

#[cfg(not(feature = "argon2"))]
#[inline(always)]
#[must_use]
pub fn generate_salt() -> Result<[u8; 32], CryptoError> {
    crate::utils::random_32()
}

// ------------------------------------------------------------------
// 5. Feature-gated re-exports
// ------------------------------------------------------------------
#[cfg(feature = "hkdf")]
pub use hkdf::HkdfSha256;

#[cfg(feature = "argon2")]
pub use argon2::{Argon2Params, Salt, hash, verify};

// ------------------------------------------------------------------
// 6. FFI raw
// ------------------------------------------------------------------
#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use core::ffi::{c_ulong, c_uchar};

    #[no_mangle]
    pub unsafe extern "C" fn kdf_derive_raw(
        password: *const c_uchar,
        password_len: c_ulong,
        salt: *const c_uchar,
        salt_len: c_ulong,
        dst: *mut c_uchar,
        dst_len: c_ulong,
        algo: u8,
    ) -> c_ulong {
        if password.is_null() || salt.is_null() || dst.is_null() {
            return 0;
        }
        let pwd = core::slice::from_raw_parts(password, password_len as usize);
        let salt_slice = core::slice::from_raw_parts(salt, salt_len as usize);
        let dst_slice = core::slice::from_raw_parts_mut(dst, dst_len as usize);

        let kdf = match algo {
            #[cfg(feature = "argon2")]
            0 => Kdf::Argon2id,
            #[cfg(feature = "hkdf")]
            1 => Kdf::HkdfSha256,
            _ => return 0,
        };

        match derive_key_into(pwd, salt_slice, dst_slice, kdf) {
            Ok(()) => dst_len,
            Err(_) => 0,
        }
    }

    #[cfg(feature = "hkdf")]
    pub use hkdf::ffi::*;
    #[cfg(feature = "argon2")]
    pub use argon2::ffi::*;
}

// ------------------------------------------------------------------
// 7. Public surface
// ------------------------------------------------------------------
pub use {Kdf, derive_key, derive_key_into, generate_salt};
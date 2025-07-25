//! **Argon2id** key-derivation function — mobile-friendly, zero-copy, ≤ 6 KB
//! Outputs 32-byte symmetric keys from passwords.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::errors::EncryptionError;
use crate::types::Key;
use crate::utils::{fill_random, secure_zero};
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// 1. Tunable parameters (mobile-friendly defaults)
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Argon2Params {
    pub memory_cost: u32,  // KB
    pub time_cost: u32,    // iterations
    pub parallelism: u32,  // threads
}

impl Default for Argon2Params {
    #[inline(always)]
    fn default() -> Self {
        Self {
            memory_cost: 64 * 1024,
            time_cost: 3,
            parallelism: 1,
        }
    }
}

// ------------------------------------------------------------------
// 2. Salt
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Salt([u8; 32]);

impl Salt {
    #[inline(always)]
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Generate 32-byte random salt.
    #[inline(always)]
    pub fn random() -> Result<Salt, EncryptionError> {
        let mut bytes = [0u8; 32];
        fill_random(&mut bytes)?;
        Ok(Salt(bytes))
    }
}

// ------------------------------------------------------------------
// 3. Hash derivation
// ------------------------------------------------------------------
#[inline(always)]
fn argon2_instance() -> argon2::Argon2<'static> {
    argon2::Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(
            64 * 1024, // memory_cost
            3,         // time_cost
            1,         // parallelism
            Some(32),  // output_len
        )
        .unwrap(),
    )
}

/// Derive 32-byte key from password + salt.
#[inline(always)]
pub fn hash(
    password: &[u8],
    salt: &Salt,
    params: Argon2Params,
) -> Result<Key, EncryptionError> {
    let mut output = [0u8; 32];
    let params_argon = argon2::Params::new(
        params.memory_cost,
        params.time_cost,
        params.parallelism,
        Some(32),
    )
    .map_err(|_| EncryptionError::Argon2Params)?;

    let argon2 = argon2::Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params_argon,
    );

    argon2
        .hash_password_into(password, salt.as_bytes(), &mut output)
        .map_err(|_| EncryptionError::Argon2Error)?;

    Ok(Key(output))
}

/// Verify a password against an existing hash.
#[inline(always)]
pub fn verify(
    password: &[u8],
    salt: &Salt,
    expected_hash: &[u8],
    params: Argon2Params,
) -> Result<bool, EncryptionError> {
    let derived = hash(password, salt, params)?;
    Ok(crate::utils::ct_eq(expected_hash, derived.as_bytes()))
}

// ------------------------------------------------------------------
// 4. FFI raw
// ------------------------------------------------------------------
#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use core::ffi::{c_ulong, c_uchar};

    #[no_mangle]
    pub unsafe extern "C" fn argon2_hash_raw(
        password: *const c_uchar,
        password_len: c_ulong,
        salt: *const c_uchar,
        salt_len: c_ulong,
        dst: *mut c_uchar,
        dst_len: c_ulong,
        memory_cost: u32,
        time_cost: u32,
        parallelism: u32,
    ) -> c_ulong {
        if password.is_null() || salt.is_null() || dst.is_null() || salt_len != 32 || dst_len < 32 {
            return 0;
        }
        let pwd = core::slice::from_raw_parts(password, password_len as usize);
        let salt_slice = core::slice::from_raw_parts(salt, 32);
        let dst_slice = core::slice::from_raw_parts_mut(dst, 32);

        let params = Argon2Params {
            memory_cost,
            time_cost,
            parallelism,
        };
        let salt = match salt_slice.try_into() {
                Ok(arr) => Salt::new(arr),
                Err(_) => return 0,
        };
        match hash(
            pwd,
            salt,
            params,
        ) {
            Ok(key) => {
                dst_slice.copy_from_slice(key.as_bytes());
                32
            }
            Err(_) => 0,
        }
    }
}

// ------------------------------------------------------------------
// 5. Public exports
// ------------------------------------------------------------------
pub use {Argon2Params, Salt, hash, verify};
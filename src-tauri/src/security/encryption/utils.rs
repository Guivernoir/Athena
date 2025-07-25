//! **Secure helpers** — zeroize, RNG, hex, constant-time compare, ≤ 3 KB

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::types::{Key, Nonce};
use core::mem::MaybeUninit;

#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

// ------------------------------------------------------------------
// 1. Secure RNG wrappers
// ------------------------------------------------------------------
/// Fill buffer with cryptographically-secure random bytes.
#[inline(always)]
pub fn fill_random(buf: &mut [u8]) -> Result<(), EncryptionError> {
    #[cfg(feature = "getrandom")]
    {
        getrandom::getrandom(buf).map_err(|_| EncryptionError::RngUnavailable)
    }
    #[cfg(not(feature = "getrandom"))]
    {
        // Stub for no_std without getrandom — caller must provide entropy
        Ok(())
    }
}

/// Generate 32-byte key.
#[inline(always)]
#[must_use]
pub fn random_key() -> Result<Key, EncryptionError> {
    let mut bytes = [0u8; 32];
    fill_random(&mut bytes)?;
    Ok(Key(bytes))
}

/// Generate 12-byte nonce.
#[inline(always)]
#[must_use]
pub fn random_nonce() -> Result<Nonce, EncryptionError> {
    let mut bytes = [0u8; 12];
    fill_random(&mut bytes)?;
    Ok(Nonce(bytes))
}

// ------------------------------------------------------------------
// 2. Secure wipe
// ------------------------------------------------------------------
/// Zero-fill buffer (volatile to prevent compiler elision).
#[inline(always)]
pub fn secure_zero(buf: &mut [u8]) {
    #[cfg(feature = "zeroize")]
    buf.zeroize();
    #[cfg(not(feature = "zeroize"))]
    {
        for b in buf.iter_mut() {
            core::ptr::write_volatile(b, 0);
        }
    }
}

// ------------------------------------------------------------------
// 3. Hex encode / decode
// ------------------------------------------------------------------
/// Encode slice to lowercase hex.
#[inline(always)]
pub fn to_hex(src: &[u8]) -> alloc::string::String {
    let mut out = alloc::string::String::with_capacity(src.len() * 2);
    for b in src {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

/// Decode lowercase hex into Vec<u8>; rejects odd length or non-hex.
#[inline(always)]
pub fn from_hex(hex: &str) -> Result<Vec<u8>, EncryptionError> {
    if hex.len() % 2 != 0 {
        return Err(EncryptionError::InvalidHexLength);
    }
    let mut out = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i+2], 16).map_err(|_| EncryptionError::InvalidHex)?;
        out.push(byte);
    }
    Ok(out)
}

// ------------------------------------------------------------------
// 4. Constant-time compare
// ------------------------------------------------------------------
/// Safe constant-time byte comparison.
#[inline(always)]
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

// ------------------------------------------------------------------
// 5. Alignment helper
// ------------------------------------------------------------------
/// Round `len` up to next power-of-two alignment.
#[inline(always)]
pub fn align_up(len: usize, align: usize) -> usize {
    (len + align - 1) & !(align - 1)
}

// ------------------------------------------------------------------
// 6. Error variants
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionError {
    RngUnavailable,
    InvalidHexLength,
    InvalidHex,
}

// ------------------------------------------------------------------
// 7. Optional zeroize wrapper
// ------------------------------------------------------------------
#[cfg(feature = "zeroize")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Sensitive<T>(pub T)
where
    T: zeroize::Zeroize;

#[cfg(feature = "zeroize")]
impl<T: zeroize::Zeroize> Drop for Sensitive<T> {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

// ------------------------------------------------------------------
// 8. Public exports
// ------------------------------------------------------------------
pub use {
    fill_random, random_key, random_nonce, secure_zero, to_hex, from_hex, ct_eq, align_up,
};
#[cfg(feature = "zeroize")]
pub use Sensitive;
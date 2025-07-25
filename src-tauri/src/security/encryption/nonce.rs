//! **Single-call nonce management** — 96-bit (12-byte) nonces
//! Zeroize-on-drop, counter + random generation, FFI-ready, ≤ 2 KB

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::errors::CryptoError;
use zeroize::Zeroize;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// 1. Nonce type (transparent for FFI)
// ------------------------------------------------------------------
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Nonce([u8; 12]);

impl Nonce {
    /// **Random 96-bit nonce**
    #[inline(always)]
    pub fn random() -> Result<Self, CryptoError> {
        let mut n = [0u8; 12];
        crate::utils::fill_random(&mut n)?;
        Ok(Self(n))
    }

    /// **Monotonic counter nonce (12-byte BE counter in last 8 bytes)**
    #[cfg(feature = "counter")]
    #[inline(always)]
    pub fn counter() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let ctr = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut bytes = [0u8; 12];
        bytes[4..].copy_from_slice(&ctr.to_be_bytes());
        Self(bytes)
    }

    /// **Zero-copy from bytes**
    #[inline(always)]
    pub const fn from_bytes(bytes: [u8; 12]) -> Self {
        Self(bytes)
    }

    /// **Fallible slice constructor**
    #[inline(always)]
    pub fn from_slice(src: &[u8]) -> Result<Self, CryptoError> {
        src.try_into()
            .map(Self)
            .map_err(|_| CryptoError::InvalidLength)
    }

    /// **Borrow inner bytes**
    #[inline(always)]
    pub const fn as_bytes(&self) -> &[u8; 12] {
        &self.0
    }

    /// **Consume into inner bytes**
    #[inline(always)]
    pub fn into_bytes(self) -> [u8; 12] {
        let bytes = self.0;
        core::mem::forget(self); // skip Drop
        bytes
    }
}

// ------------------------------------------------------------------
// 2. Debug redaction
// ------------------------------------------------------------------
impl core::fmt::Debug for Nonce {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Nonce([REDACTED])")
    }
}

// ------------------------------------------------------------------
// 3. Zeroize on drop
// ------------------------------------------------------------------
impl Drop for Nonce {
    #[inline(always)]
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

// ------------------------------------------------------------------
// 4. Trait impls for FFI
// ------------------------------------------------------------------
impl From<[u8; 12]> for Nonce {
    #[inline(always)]
    fn from(b: [u8; 12]) -> Self {
        Self(b)
    }
}

impl AsRef<[u8]> for Nonce {
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
    pub unsafe extern "C" fn nonce_generate(ptr: *mut c_uchar) -> c_ulong {
        if ptr.is_null() {
            return 0;
        }
        let out = core::slice::from_raw_parts_mut(ptr, 12);
        match Nonce::random() {
            Ok(n) => {
                out.copy_from_slice(n.as_bytes());
                12
            }
            Err(_) => 0,
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn nonce_from_bytes(
        src: *const c_uchar,
        dst: *mut c_uchar,
    ) -> c_ulong {
        if src.is_null() || dst.is_null() {
            return 0;
        }
        let src_slice = core::slice::from_raw_parts(src, 12);
        let dst_slice = core::slice::from_raw_parts_mut(dst, 12);
        match Nonce::from_slice(src_slice) {
            Ok(n) => {
                dst_slice.copy_from_slice(n.as_bytes());
                12
            }
            Err(_) => 0,
        }
    }

    #[cfg(feature = "counter")]
    #[no_mangle]
    pub unsafe extern "C" fn nonce_counter(ptr: *mut c_uchar) -> c_ulong {
        if ptr.is_null() {
            return 0;
        }
        let out = core::slice::from_raw_parts_mut(ptr, 12);
        let n = Nonce::counter();
        out.copy_from_slice(n.as_bytes());
        12
    }
}

// ------------------------------------------------------------------
// 6. Public exports
// ------------------------------------------------------------------
pub use Nonce;
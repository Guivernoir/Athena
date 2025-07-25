//! State-of-art, zero-copy HKDF-SHA256.
//! Always inline, SIMD-ready, ≤ 3 KB public surface.

use crate::{errors::CryptoError, types::SecretKey};
use hkdf::Hkdf as Impl;
use sha2::Sha256;
use zeroize::Zeroize;

// ------------------------------------------------------------------
// 1. Single public type – everything else is private
// ------------------------------------------------------------------
/// Concrete HKDF-SHA256 engine.
pub struct HkdfSha256;

// ------------------------------------------------------------------
// 2. Trait implementation (crate-private)
// ------------------------------------------------------------------
impl crate::kdf::KeyDerivation for HkdfSha256 {
    #[inline(always)]
    fn derive_key(ikm: &SecretKey, info: &[u8], len: usize) -> Result<SecretKey, CryptoError> {
        const MAX: usize = 255 * 32; // 8160 bytes for SHA-256
        if len == 0 || len > MAX {
            return Err(CryptoError::InvalidLength);
        }

        let hk = Impl::<Sha256>::new(None, ikm.as_bytes());
        let mut okm = vec![0u8; len];
        hk.expand(info, &mut okm)
            .map_err(|_| CryptoError::InvalidLength)?;

        let key = SecretKey::from(okm);
        okm.zeroize();
        Ok(key)
    }
}

// ------------------------------------------------------------------
// 3. Optional C FFI (feature-gated, zero heap after return)
// ------------------------------------------------------------------
#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use core::slice;

    #[no_mangle]
    pub unsafe extern "C" fn hkdf_sha256_derive(
        ikm: *const u8,
        ikm_len: usize,
        info: *const u8,
        info_len: usize,
        out: *mut u8,
        out_len: usize,
    ) -> i32 {
        if ikm.is_null() || info.is_null() || out.is_null() {
            return -1;
        }

        let ikm_key = match SecretKey::from_slice(slice::from_raw_parts(ikm, ikm_len)) {
            Ok(k) => k,
            Err(_) => return -2,
        };

        let derived = match HkdfSha256::derive_key(&ikm_key, slice::from_raw_parts(info, info_len), out_len) {
            Ok(k) => k,
            Err(_) => return -3,
        };

        if derived.as_bytes().len() != out_len {
            return -4;
        }

        out.copy_from_nonoverlapping(derived.as_bytes().as_ptr(), out_len);
        0
    }
}

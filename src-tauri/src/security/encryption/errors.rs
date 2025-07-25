//! **Unified encryption error taxonomy**  
//! Covers **ChaCha20-Poly1305**, **AES-GCM**, **key derivation**, and **FFI boundaries**.

#![cfg_attr(not(feature = "std"), no_std)]

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionError {
    // --- AEAD ciphers ----------------------------------------------------------
    ChaCha20Poly1305 = 0,
    AesGcm = 1,

    // --- Key / nonce -----------------------------------------------------------
    InvalidKeyLength { expected: usize, found: usize } = 2,
    InvalidNonceLength { expected: usize, found: usize } = 3,

    // --- MAC / integrity -------------------------------------------------------
    MacMismatch = 4,

    // --- KDF / key hierarchy ---------------------------------------------------
    InvalidKdfInput = 5,
    KdfOutputTooShort = 6,

    // --- Buffer & allocation ---------------------------------------------------
    OutputBufferTooSmall = 7,
}

impl EncryptionError {
    #[inline(always)]
    pub fn code(&self) -> u8 {
        *self as u8
    }
}

#[cfg(feature = "thiserror")]
impl core::fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ChaCha20Poly1305 => write!(f, "ChaCha20-Poly1305 error"),
            Self::AesGcm => write!(f, "AES-GCM error"),
            Self::InvalidKeyLength { expected, found } => {
                write!(f, "Invalid key length: expected {}, found {}", expected, found)
            }
            Self::InvalidNonceLength { expected, found } => {
                write!(f, "Invalid nonce length: expected {}, found {}", expected, found)
            }
            Self::MacMismatch => write!(f, "MAC verification failed"),
            Self::InvalidKdfInput => write!(f, "KDF input invalid"),
            Self::KdfOutputTooShort => write!(f, "KDF output too short"),
            Self::OutputBufferTooSmall => write!(f, "Output buffer too small"),
        }
    }
}

pub use EncryptionError;
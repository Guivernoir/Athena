//! Single-call, state-of-art encryption façade
//! encrypt / decrypt / derive_key / generate_key / generate_nonce / generate_salt
//! Zero-copy variants included, FFI-ready, ≤ 6 KB .rlib

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

// ---------- Re-export all public primitives ----------
pub use crate::encryption::{
    aead::{Aead, encrypt, encrypt_into, decrypt, decrypt_into},
    errors::CryptoError,
    keys::{Key, generate_key},
    kdf::{Kdf, derive_key, derive_key_into},
    nonce::{Nonce, generate_nonce},
    utils::{generate_salt},
};

// ---------- convenience façade helpers ----------
#[inline(always)]
pub fn seal(
    key: &Key,
    nonce: &Nonce,
    plaintext: &[u8],
    algo: Aead,
) -> Result<alloc::vec::Vec<u8>, CryptoError> {
    encrypt(key, nonce, plaintext, algo)
}

#[inline(always)]
pub fn open(
    key: &Key,
    nonce: &Nonce,
    ciphertext: &[u8],
    algo: Aead,
) -> Result<alloc::vec::Vec<u8>, CryptoError> {
    decrypt(key, nonce, ciphertext, algo)
}

/// Encrypt into caller buffer (zero-copy).
#[inline(always)]
pub fn seal_into(
    key: &Key,
    nonce: &Nonce,
    plaintext: &[u8],
    dst: &mut [u8],
    algo: Aead,
) -> Result<(), CryptoError> {
    encrypt_into(key, nonce, plaintext, dst, algo)
}

/// Decrypt into caller buffer (zero-copy).
#[inline(always)]
pub fn open_into(
    key: &Key,
    nonce: &Nonce,
    ciphertext: &[u8],
    dst: &mut [u8],
    algo: Aead,
) -> Result<(), CryptoError> {
    decrypt_into(key, nonce, ciphertext, dst, algo)
}

// ---------- internal modules ----------
mod aead;
mod errors;
mod keys;
mod kdf;
mod nonce;
mod utils;
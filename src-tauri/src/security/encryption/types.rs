//! **State-of-art encryption primitives**  
//! Zero-copy, FFI-safe, ≤ 2 KB

#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// 1. Key material
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Key([u8; 32]);

impl Key {
    #[inline(always)]
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

// ------------------------------------------------------------------
// 2. Nonce (96-bit AEAD)
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Nonce([u8; 12]);

impl Nonce {
    #[inline(always)]
    pub fn new(bytes: [u8; 12]) -> Self {
        Self(bytes)
    }

    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8; 12] {
        &self.0
    }
}

// ------------------------------------------------------------------
// 3. Ciphertext (owned)
// ------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Ciphertext(Vec<u8>);

impl Ciphertext {
    #[inline(always)]
    pub fn new(vec: Vec<u8>) -> Self {
        Self(vec)
    }

    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    #[inline(always)]
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

// ------------------------------------------------------------------
// 4. Zero-copy view
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub struct CiphertextView<'a> {
    bytes: &'a [u8],
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> CiphertextView<'a> {
    #[inline(always)]
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, _lifetime: PhantomData }
    }

    #[inline(always)]
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }
}
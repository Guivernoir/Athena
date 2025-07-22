//! **Central configuration** — drives bit-packing + entropy + zstd
//! Serde-ready, zero-copy, ≤ 4 KB public surface

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::codec::Codec;
use crate::errors::CompressionError;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// 1. Configuration struct
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Config {
    /// Entropy / pipeline codec
    pub codec: Codec,
    /// Bit-packing bits (ignored for non-bit-packing codecs)
    pub bits: u8,
    /// zstd compression level (ignored for non-zstd codecs)
    pub zstd_level: i32,
    /// Enable SIMD acceleration
    #[cfg(feature = "simd")]
    pub simd_enabled: bool,
    /// Enable buffer-pool reuse
    #[cfg(feature = "buffer-pool")]
    pub use_buffer_pool: bool,
}

impl Default for Config {
    #[inline(always)]
    fn default() -> Self {
        Self {
            codec: Codec::Zstd,
            bits: 8,
            zstd_level: 3,
            #[cfg(feature = "simd")]
            simd_enabled: true,
            #[cfg(feature = "buffer-pool")]
            use_buffer_pool: false,
        }
    }
}

// ------------------------------------------------------------------
// 2. Builder
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub struct ConfigBuilder {
    inner: Config,
}

impl ConfigBuilder {
    #[inline(always)]
    pub fn new() -> Self {
        Self { inner: Config::default() }
    }

    #[inline(always)]
    pub fn codec(mut self, codec: Codec) -> Self {
        self.inner.codec = codec;
        self
    }

    #[inline(always)]
    pub fn bits(mut self, bits: u8) -> Self {
        self.inner.bits = bits;
        self
    }

    #[inline(always)]
    pub fn zstd_level(mut self, level: i32) -> Self {
        self.inner.zstd_level = level;
        self
    }

    #[cfg(feature = "simd")]
    #[inline(always)]
    pub fn simd(mut self, enable: bool) -> Self {
        self.inner.simd_enabled = enable;
        self
    }

    #[cfg(feature = "buffer-pool")]
    #[inline(always)]
    pub fn buffer_pool(mut self, enable: bool) -> Self {
        self.inner.use_buffer_pool = enable;
        self
    }

    #[inline(always)]
    pub fn build(self) -> Self {
        self
    }
}

// ------------------------------------------------------------------
// 3. Validation
// ------------------------------------------------------------------
impl Config {
    #[inline(always)]
    pub fn validate(&self) -> Result<(), CompressionError> {
        if self.bits < 1 || self.bits > 8 {
            return Err(CompressionError::BitWidthUnsupported { bits: self.bits });
        }
        if !(1..=22).contains(&self.zstd_level) {
            return Err(CompressionError::ZstdError);
        }
        Ok(())
    }
}

// ------------------------------------------------------------------
// 4. Optional runtime from env (feature-gated)
// ------------------------------------------------------------------
#[cfg(feature = "std")]
impl Config {
    /// Load from environment variables.
    /// ENV vars: COMP_CODEC, COMP_BITS, COMP_LEVEL, COMP_SIMD, COMP_POOL
    #[cfg(feature = "env")]
    #[inline(always)]
    pub fn from_env() -> Result<Self, CompressionError> {
        use std::env;
        let codec_str = env::var("COMP_CODEC").unwrap_or_else(|_| "zstd".to_string());
        let codec = match codec_str.as_str() {
            "ans12" => Codec::Ans12,
            "ans14" => Codec::Ans14,
            "huffman" => Codec::Huffman,
            "rle" => Codec::Rle,
            "zstd" => Codec::Zstd,
            _ => return Err(CompressionError::ZstdError),
        };
        let bits: u8 = env::var("COMP_BITS")
            .unwrap_or_else(|_| "8".to_string())
            .parse()
            .unwrap_or(8);
        let level: i32 = env::var("COMP_LEVEL")
            .unwrap_or_else(|_| "3".to_string())
            .parse()
            .unwrap_or(3);
        let cfg = Config {
            codec,
            bits,
            zstd_level: level,
            #[cfg(feature = "simd")]
            simd_enabled: env::var("COMP_SIMD")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            #[cfg(feature = "buffer-pool")]
            use_buffer_pool: env::var("COMP_POOL")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        };
        cfg.validate()
    }
}

// ------------------------------------------------------------------
// 5. Public exports
// ------------------------------------------------------------------
pub use {Config, ConfigBuilder};
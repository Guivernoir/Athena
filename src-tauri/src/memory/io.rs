//! File I/O helpers and compression pipeline stubs.

use std::io::{Read, Write};

/// Very small LZ4-like framing stub (no-op for now).
pub struct Compressor;

impl Compressor {
    pub fn compress(_src: &[u8], _dst: &mut Vec<u8>) -> std::io::Result<()> {
        // TODO: real LZ4
        Ok(())
    }

    pub fn decompress(_src: &[u8], _dst: &mut Vec<u8>) -> std::io::Result<()> {
        // TODO: real LZ4
        Ok(())
    }
}
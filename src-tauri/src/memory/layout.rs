//! Binary format definitions (headers, record schema).

use std::mem::size_of;

/// Fixed-size header at the start of every segment file.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SegmentHeader {
    pub magic: u32,      // b"VEC0"
    pub version: u32,    // 1
    pub dim: u32,        // vector dimension
    pub reserved: [u8; 16],
}

impl SegmentHeader {
    pub const MAGIC: u32 = u32::from_le_bytes(*b"VEC0");
    pub const SIZE: usize = size_of::<Self>();
}

/// On-disk record layout (little-endian):
/// [vector: f32 * dim][timestamp: u64][payload_len: u32][payload: bytes]
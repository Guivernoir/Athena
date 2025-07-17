//! Defines `MemoryRecord`, serialization / deserialization.

use crate::layout::SegmentHeader;
use std::io::{Cursor, Read, Write};

#[derive(Debug, Clone)]
pub struct MemoryRecord {
    pub vector: Vec<f32>,
    pub timestamp: u64,
    pub payload: Vec<u8>,
}

impl MemoryRecord {
    /// Serialize to little-endian bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        for &v in &self.vector {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        buf.extend_from_slice(&self.timestamp.to_le_bytes());
        buf.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }

    /// Deserialize from bytes (assumes correct layout).
    pub fn from_bytes(mut data: &[u8], dim: usize) -> std::io::Result<Self> {
        let mut vector = vec![0.0f32; dim];
        for v in &mut vector {
            let mut buf = [0u8; 4];
            data.read_exact(&mut buf)?;
            *v = f32::from_le_bytes(buf);
        }

        let mut ts_buf = [0u8; 8];
        data.read_exact(&mut ts_buf)?;
        let timestamp = u64::from_le_bytes(ts_buf);

        let mut len_buf = [0u8; 4];
        data.read_exact(&mut len_buf)?;
        let payload_len = u32::from_le_bytes(len_buf) as usize;

        let mut payload = vec![0u8; payload_len];
        data.read_exact(&mut payload)?;

        Ok(Self {
            vector,
            timestamp,
            payload,
        })
    }
}
use flate2::{Compression, write::ZlibEncoder, read::ZlibDecoder};
use std::io::{Read, Write};
use anyhow::Result;

/// Compresses data using zlib (good balance of speed and ratio)
pub fn compress_data(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish().context("Compression failed")
}

/// Decompresses zlib-compressed data
pub fn decompress_data(compressed_data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(compressed_data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}
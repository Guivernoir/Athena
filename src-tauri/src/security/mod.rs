mod encryption;
mod compression;
pub mod quantizer;

pub use encryption::{encrypt_data, decrypt_data, generate_key};
pub use compression::{compress_data, decompress_data};
pub use quantizer::{Quantizer, QuantizationError};

/// Security processor for embedded vector data
pub struct VectorSecurity;

impl VectorSecurity {
    /// Prepares vector data for storage (quantize -> compress -> encrypt)
    pub fn prepare_for_storage(
        weights: &[f32],
        key: &[u8; 32],
        quantizer: &Quantizer,
    ) -> anyhow::Result<Vec<u8>> {
        // Step 1: Quantization
        let quantized = quantizer.quantize(weights)
            .map_err(|e| anyhow::anyhow!("Quantization failed: {:?}", e))?;
        
        // Step 2: Compression
        let compressed = compression::compress_data(&quantized)?;
        
        // Step 3: Encryption
        encryption::encrypt_data(&compressed, key)
    }

    /// Restores vector data from secured form (decrypt -> decompress -> dequantize)
    pub fn restore_from_storage(
        secured_data: &[u8],
        key: &[u8; 32],
        quantizer: &Quantizer,
    ) -> anyhow::Result<Vec<f32>> {
        // Step 1: Decryption
        let compressed = encryption::decrypt_data(secured_data, key)?;
        
        // Step 2: Decompression
        let quantized = compression::decompress_data(&compressed)?;
        
        // Step 3: Dequantization
        quantizer.dequantize(&quantized)
            .map_err(|e| anyhow::anyhow!("Dequantization failed: {:?}", e))
    }

    /// Generates a new encryption key
    pub fn generate_key() -> [u8; 32] {
        encryption::generate_key()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quantizer::{Quantizer, QuantizationConfig};

    #[test]
    fn test_full_pipeline() -> anyhow::Result<()> {
        let key = VectorSecurity::generate_key();
        let quantizer = Quantizer::new(QuantizationConfig::default())?;
        let original_weights = vec![1.0, -0.5, 0.25, -0.125, 0.0625];
        
        // Test storage pipeline
        let secured = VectorSecurity::prepare_for_storage(&original_weights, &key, &quantizer)?;
        let recovered = VectorSecurity::restore_from_storage(&secured, &key, &quantizer)?;
        
        // Verify we got back roughly the same values
        assert_eq!(original_weights.len(), recovered.len());
        for (orig, rec) in original_weights.iter().zip(recovered.iter()) {
            assert!((orig - rec).abs() < 0.1, "Value mismatch: {} vs {}", orig, rec);
        }
        
        Ok(())
    }
}
// quantization/bridge.rs
use crate::quantization::{QuantizationConfig, QuantizationError, Result};
use std::sync::Arc;
use std::collections::HashMap;

/// High-level bridge for quantization operations
pub struct QuantizationBridge {
    configs: HashMap<String, QuantizationConfig>,
    active_quantizers: HashMap<String, Arc<super::Quantizer>>,
}

impl QuantizationBridge {
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
            active_quantizers: HashMap::new(),
        }
    }

    /// Register a quantization configuration
    pub fn register_config(&mut self, name: String, config: QuantizationConfig) -> Result<()> {
        config.validate().map_err(|_| QuantizationError::InvalidBitWidth)?;
        self.configs.insert(name, config);
        Ok(())
    }

    /// Create a quantizer from registered config
    pub fn create_quantizer(&mut self, config_name: &str) -> Result<()> {
        let config = self.configs.get(config_name)
            .ok_or(QuantizationError::InvalidBitWidth)?
            .clone();

        let quantizer = super::Quantizer::new(config)?;
        self.active_quantizers.insert(config_name.to_string(), Arc::new(quantizer));
        Ok(())
    }

    /// Quantize weights using named quantizer
    pub fn quantize_with_name(&self, name: &str, weights: &[f32]) -> Result<Vec<u8>> {
        let quantizer = self.active_quantizers.get(name)
            .ok_or(QuantizationError::NullPointer)?;
        quantizer.quantize(weights)
    }

    /// Dequantize weights using named quantizer
    pub fn dequantize_with_name(&self, name: &str, quantized: &[u8]) -> Result<Vec<f32>> {
        let quantizer = self.active_quantizers.get(name)
            .ok_or(QuantizationError::NullPointer)?;
        quantizer.dequantize(quantized)
    }

    /// Batch quantization for multiple weight tensors
    pub fn batch_quantize(&self, name: &str, weight_batches: &[&[f32]]) -> Result<Vec<Vec<u8>>> {
        let quantizer = self.active_quantizers.get(name)
            .ok_or(QuantizationError::NullPointer)?;

        let mut results = Vec::with_capacity(weight_batches.len());
        for weights in weight_batches {
            results.push(quantizer.quantize(weights)?);
        }
        Ok(results)
    }

    /// Get compression statistics
    pub fn get_compression_stats(&self, name: &str, original_size: usize) -> Option<CompressionStats> {
        let config = self.configs.get(name)?;
        let compressed_size = (original_size * config.bits + 7) / 8;
        
        Some(CompressionStats {
            original_size,
            compressed_size,
            compression_ratio: config.compression_ratio(),
            space_saved: original_size - compressed_size,
        })
    }

    /// List all registered configurations
    pub fn list_configs(&self) -> Vec<String> {
        self.configs.keys().cloned().collect()
    }

    /// Remove a quantizer
    pub fn remove_quantizer(&mut self, name: &str) -> bool {
        self.active_quantizers.remove(name).is_some()
    }
}

#[derive(Debug, Clone)]
pub struct CompressionStats {
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f32,
    pub space_saved: usize,
}

impl Default for QuantizationBridge {
    fn default() -> Self {
        let mut bridge = Self::new();
        
        // Register common configurations
        let _ = bridge.register_config("int4".to_string(), QuantizationConfig::int4());
        let _ = bridge.register_config("int8".to_string(), QuantizationConfig::int8());
        let _ = bridge.register_config("int16".to_string(), QuantizationConfig::int16());
        
        bridge
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let bridge = QuantizationBridge::new();
        assert_eq!(bridge.configs.len(), 0);
    }

    #[test]
    fn test_config_registration() {
        let mut bridge = QuantizationBridge::new();
        let config = QuantizationConfig::int8();
        
        assert!(bridge.register_config("test".to_string(), config).is_ok());
        assert_eq!(bridge.configs.len(), 1);
    }

    #[test]
    fn test_default_configs() {
        let bridge = QuantizationBridge::default();
        let configs = bridge.list_configs();
        
        assert!(configs.contains(&"int4".to_string()));
        assert!(configs.contains(&"int8".to_string()));
        assert!(configs.contains(&"int16".to_string()));
    }
}
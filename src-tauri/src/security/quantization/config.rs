use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationConfig {
    /// Number of bits for quantization (typically 4, 8, or 16)
    pub bits: usize,
    /// Block size for grouped quantization
    pub block_size: usize,
    /// Quantization method
    pub method: QuantizationMethod,
    /// Whether to use symmetric quantization
    pub symmetric: bool,
    /// Calibration dataset size (for dynamic quantization)
    pub calibration_samples: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantizationMethod {
    /// Simple linear quantization
    Linear,
    /// Dynamic range quantization
    Dynamic,
    /// Block-wise quantization with shared scales
    BlockWise,
    /// K-means clustering quantization
    KMeans,
}

impl Default for QuantizationConfig {
    fn default() -> Self {
        Self {
            bits: 8,
            block_size: 128,
            method: QuantizationMethod::BlockWise,
            symmetric: true,
            calibration_samples: Some(1000),
        }
    }
}

impl QuantizationConfig {
    /// Create a new config with validation
    pub fn new(bits: usize, block_size: usize) -> Result<Self, String> {
        if bits == 0 || bits > 16 {
            return Err("Bit width must be between 1 and 16".to_string());
        }
        
        if block_size == 0 || block_size > 1024 {
            return Err("Block size must be between 1 and 1024".to_string());
        }

        Ok(Self {
            bits,
            block_size,
            ..Default::default()
        })
    }

    /// Preset for 4-bit quantization (aggressive compression)
    pub fn int4() -> Self {
        Self {
            bits: 4,
            block_size: 64,
            method: QuantizationMethod::BlockWise,
            symmetric: true,
            calibration_samples: Some(2000),
        }
    }

    /// Preset for 8-bit quantization (balanced)
    pub fn int8() -> Self {
        Self {
            bits: 8,
            block_size: 128,
            method: QuantizationMethod::Dynamic,
            symmetric: false,
            calibration_samples: Some(1000),
        }
    }

    /// Preset for 16-bit quantization (high precision)
    pub fn int16() -> Self {
        Self {
            bits: 16,
            block_size: 256,
            method: QuantizationMethod::Linear,
            symmetric: true,
            calibration_samples: None,
        }
    }

    /// Calculate compression ratio
    pub fn compression_ratio(&self) -> f32 {
        32.0 / (self.bits as f32)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.bits == 0 || self.bits > 16 {
            return Err("Invalid bit width".to_string());
        }

        if self.block_size == 0 {
            return Err("Invalid block size".to_string());
        }

        if let Some(samples) = self.calibration_samples {
            if samples == 0 {
                return Err("Calibration samples must be > 0".to_string());
            }
        }

        Ok(())
    }
}
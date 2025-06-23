// quantization/mod.rs
pub mod bridge;
pub mod config;

use std::ffi::CString;
use std::os::raw::{c_char, c_float, c_int, c_void};

// External C++ function declarations
extern "C" {
    fn create_quantizer(bits: c_int, block_size: c_int) -> *mut c_void;
    fn destroy_quantizer(quantizer: *mut c_void);
    fn quantize_weights(
        quantizer: *mut c_void,
        weights: *const c_float,
        length: c_int,
        output: *mut c_char,
    ) -> c_int;
    fn dequantize_weights(
        quantizer: *mut c_void,
        quantized: *const c_char,
        length: c_int,
        output: *mut c_float,
    ) -> c_int;
}

pub use bridge::QuantizationBridge;
pub use config::QuantizationConfig;

#[derive(Debug)]
pub enum QuantizationError {
    InvalidBitWidth,
    InvalidBlockSize,
    QuantizationFailed,
    DequantizationFailed,
    NullPointer,
}

pub type Result<T> = std::result::Result<T, QuantizationError>;

// Main quantization interface
pub struct Quantizer {
    ptr: *mut c_void,
    config: QuantizationConfig,
}

impl Quantizer {
    pub fn new(config: QuantizationConfig) -> Result<Self> {
        let ptr = unsafe { 
            create_quantizer(config.bits as c_int, config.block_size as c_int) 
        };
        
        if ptr.is_null() {
            return Err(QuantizationError::NullPointer);
        }

        Ok(Quantizer { ptr, config })
    }

    pub fn quantize(&self, weights: &[f32]) -> Result<Vec<u8>> {
        let mut output = vec![0u8; self.calculate_output_size(weights.len())];
        
        let result = unsafe {
            quantize_weights(
                self.ptr,
                weights.as_ptr(),
                weights.len() as c_int,
                output.as_mut_ptr() as *mut c_char,
            )
        };

        if result == 0 {
            Err(QuantizationError::QuantizationFailed)
        } else {
            Ok(output)
        }
    }

    pub fn dequantize(&self, quantized: &[u8]) -> Result<Vec<f32>> {
        let output_size = self.calculate_dequant_size(quantized.len());
        let mut output = vec![0.0f32; output_size];

        let result = unsafe {
            dequantize_weights(
                self.ptr,
                quantized.as_ptr() as *const c_char,
                quantized.len() as c_int,
                output.as_mut_ptr(),
            )
        };

        if result == 0 {
            Err(QuantizationError::DequantizationFailed)
        } else {
            Ok(output)
        }
    }

    fn calculate_output_size(&self, input_len: usize) -> usize {
        // Strategic calculation: bits per weight * number of weights / 8 bits per byte
        (input_len * self.config.bits + 7) / 8
    }

    fn calculate_dequant_size(&self, quantized_len: usize) -> usize {
        // Reverse calculation with proper rounding
        (quantized_len * 8) / self.config.bits
    }
}

impl Drop for Quantizer {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { destroy_quantizer(self.ptr) };
        }
    }
}

unsafe impl Send for Quantizer {}
unsafe impl Sync for Quantizer {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantizer_creation() {
        let config = QuantizationConfig::default();
        let quantizer = Quantizer::new(config);
        assert!(quantizer.is_ok());
    }

    #[test]
    fn test_quantization_roundtrip() {
        let config = QuantizationConfig::default();
        let quantizer = Quantizer::new(config).unwrap();
        
        let weights = vec![1.0, -0.5, 0.25, -0.125];
        let quantized = quantizer.quantize(&weights).unwrap();
        let dequantized = quantizer.dequantize(&quantized).unwrap();
        
        // Allow for quantization precision loss
        for (original, recovered) in weights.iter().zip(dequantized.iter()) {
            assert!((original - recovered).abs() < 0.1);
        }
    }
}
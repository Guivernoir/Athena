pub mod engine;
pub mod storage;
pub mod btree;
pub mod wal;
pub mod tx;
pub mod types;

pub use engine::SGBDEngine;
pub use types::{Key, Value, SGBDError, Result};

// sgbd/types.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::receive::{ParsedInput, Mode, Proficiency};

pub type Result<T> = std::result::Result<T, SGBDError>;

#[derive(Debug, Clone)]
pub enum SGBDError {
    IoError(String),
    SerializationError(String),
    IndexError(String),
    KeyNotFound,
    TransactionError(String),
    QuantizationError(String),
    WalError(String),
}

impl From<std::io::Error> for SGBDError {
    fn from(err: std::io::Error) -> Self {
        SGBDError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for SGBDError {
    fn from(err: serde_json::Error) -> Self {
        SGBDError::SerializationError(err.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Key {
    pub id: u64,
    pub timestamp: u64,
}

impl Key {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(|e| SGBDError::SerializationError(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputRecord {
    pub raw_input: String,
    pub parsed_input: ParsedInput,
    pub quantized_data: Vec<u8>,
    pub metadata: RecordMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetadata {
    pub created_at: u64,
    pub input_length: usize,
    pub quantization_bits: usize,
    pub compression_ratio: f32,
    pub domain: String,
    pub mode: Mode,
    pub proficiency: Proficiency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    InputRecord(InputRecord),
    Metadata(HashMap<String, String>),
    Raw(Vec<u8>),
}

impl Value {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| SGBDError::SerializationError(e.to_string()))
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(|e| SGBDError::SerializationError(e.to_string()))
    }
    
    pub fn size(&self) -> usize {
        match self {
            Value::InputRecord(record) => {
                record.raw_input.len() + record.quantized_data.len() + 256 // Approximate overhead
            }
            Value::Metadata(meta) => {
                meta.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>() + 64
            }
            Value::Raw(data) => data.len(),
        }
    }
}
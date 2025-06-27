use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::llm::{ParsedInput, Mode, Proficiency};

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

impl std::fmt::Display for SGBDError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SGBDError::IoError(msg) => write!(f, "IO Error: {}", msg),
            SGBDError::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
            SGBDError::IndexError(msg) => write!(f, "Index Error: {}", msg),
            SGBDError::KeyNotFound => write!(f, "Key not found"),
            SGBDError::TransactionError(msg) => write!(f, "Transaction Error: {}", msg),
            SGBDError::QuantizationError(msg) => write!(f, "Quantization Error: {}", msg),
            SGBDError::WalError(msg) => write!(f, "WAL Error: {}", msg),
        }
    }
}

impl std::error::Error for SGBDError {}

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

impl From<bincode::Error> for SGBDError {
    fn from(err: bincode::Error) -> Self {
        SGBDError::SerializationError(err.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
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
        bincode::deserialize(bytes).map_err(SGBDError::from)
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
        bincode::serialize(self).map_err(SGBDError::from)
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(SGBDError::from)
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
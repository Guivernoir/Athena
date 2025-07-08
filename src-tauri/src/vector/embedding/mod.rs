use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;

mod ffi;
use ffi::RawEmbeddingEngine;

/**
 * Safe Rust Wrapper for BGE Embedding Engine
 * 
 * Strategic pivot to vectorial intelligence operations.
 * Clean, idiomatic interface for semantic understanding without the
 * unsafe wilderness bleeding into your vector database operations.
 */

#[derive(Error, Debug)]
pub enum EmbeddingError {
    #[error("Failed to initialize embedding engine with model: {model_path}")]
    InitializationFailed { model_path: String },
    #[error("Engine is not loaded or has been disposed")]
    EngineNotLoaded,
    #[error("Embedding extraction failed: {reason}")]
    EmbeddingFailed { reason: String },
    #[error("Model file not found: {path}")]
    ModelNotFound { path: String },
    #[error("Invalid input: {details}")]
    InvalidInput { details: String },
}

pub type Result<T> = std::result::Result<T, EmbeddingError>;

#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    pub vector: Vec<f32>,
    pub dimension: usize,
    pub text: String,
}

impl EmbeddingResult {
    pub fn cosine_similarity(&self, other: &EmbeddingResult) -> f32 {
        if self.dimension != other.dimension {
            return 0.0;
        }
        
        let dot_product: f32 = self.vector.iter()
            .zip(other.vector.iter())
            .map(|(a, b)| a * b)
            .sum();
        
        let norm_a: f32 = self.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = other.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        
        dot_product / (norm_a * norm_b)
    }
}

pub struct EmbeddingEngine {
    inner: Arc<Mutex<Option<RawEmbeddingEngine>>>,
    model_path: String,
    embedding_dim: usize,
}

impl EmbeddingEngine {
    pub fn new<P: AsRef<Path>>(model_path: P) -> Result<Self> {
        let path_str = model_path.as_ref().to_string_lossy().to_string();
        
        if !model_path.as_ref().exists() {
            return Err(EmbeddingError::ModelNotFound { path: path_str });
        }
        
        let raw_engine = unsafe {
            RawEmbeddingEngine::new(&path_str).ok_or_else(|| {
                EmbeddingError::InitializationFailed {
                    model_path: path_str.clone(),
                }
            })?
        };
        
        let embedding_dim = unsafe { raw_engine.get_embedding_dim() };
        
        Ok(EmbeddingEngine {
            inner: Arc::new(Mutex::new(Some(raw_engine))),
            model_path: path_str,
            embedding_dim,
        })
    }
    
    pub fn embed(&self, text: &str) -> Result<EmbeddingResult> {
        if text.trim().is_empty() {
            return Err(EmbeddingError::InvalidInput {
                details: "Empty text provided".to_string(),
            });
        }
        
        let guard = self.inner.lock().unwrap();
        match guard.as_ref() {
            Some(engine) => {
                let vector = unsafe { engine.embed(text) }
                    .ok_or_else(|| EmbeddingError::EmbeddingFailed {
                        reason: "Failed to extract embedding vector".to_string(),
                    })?;
                
                Ok(EmbeddingResult {
                    vector,
                    dimension: self.embedding_dim,
                    text: text.to_string(),
                })
            }
            None => Err(EmbeddingError::EngineNotLoaded),
        }
    }
    
    pub fn embed_batch(&self, texts: &[String]) -> Result<Vec<EmbeddingResult>> {
        texts.iter()
            .map(|text| self.embed(text))
            .collect()
    }
    
    pub fn is_loaded(&self) -> bool {
        let guard = self.inner.lock().unwrap();
        match guard.as_ref() {
            Some(engine) => unsafe { engine.is_loaded() },
            None => false,
        }
    }
    
    pub fn get_model_info(&self) -> String {
        let guard = self.inner.lock().unwrap();
        match guard.as_ref() {
            Some(engine) => unsafe { engine.get_model_info() },
            None => format!("Engine not loaded (model: {})", self.model_path),
        }
    }
    
    pub fn model_path(&self) -> &str {
        &self.model_path
    }
    
    pub fn embedding_dimension(&self) -> usize {
        self.embedding_dim
    }
    
    pub fn dispose(&self) {
        let mut guard = self.inner.lock().unwrap();
        *guard = None;
    }
}

unsafe impl Send for EmbeddingEngine {}
unsafe impl Sync for EmbeddingEngine {}

impl Clone for EmbeddingEngine {
    fn clone(&self) -> Self {
        EmbeddingEngine {
            inner: Arc::clone(&self.inner),
            model_path: self.model_path.clone(),
            embedding_dim: self.embedding_dim,
        }
    }
}

// Convenience constructors and methods
impl EmbeddingEngine {
    pub fn from_models_dir() -> Result<Self> {
        let model_path = "llama/models/bge-small-en-v1.5-q8_0.gguf";
        Self::new(model_path)
    }
    
    pub fn encode(&self, text: &str) -> Result<Vec<f32>> {
        Ok(self.embed(text)?.vector)
    }
    
    pub fn semantic_search(&self, query: &str, documents: &[String]) -> Result<Vec<(usize, f32)>> {
        let query_embedding = self.embed(query)?;
        let doc_embeddings: Result<Vec<_>> = documents.iter()
            .map(|doc| self.embed(doc))
            .collect();
        
        let doc_embeddings = doc_embeddings?;
        let mut similarities: Vec<(usize, f32)> = doc_embeddings.iter()
            .enumerate()
            .map(|(i, doc_emb)| (i, query_embedding.cosine_similarity(doc_emb)))
            .collect();
        
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(similarities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_engine_creation() {
        let engine = EmbeddingEngine::from_models_dir();
        assert!(engine.is_ok(), "Engine creation should succeed with valid model");
    }
    
    #[test]
    fn test_embedding_extraction() {
        let engine = EmbeddingEngine::from_models_dir().unwrap();
        let result = engine.embed("Hello, world!");
        assert!(result.is_ok(), "Embedding extraction should work");
        
        let embedding = result.unwrap();
        assert_eq!(embedding.dimension, 384); // BGE-small dimension
        assert_eq!(embedding.vector.len(), 384);
        println!("Embedding dimension: {}", embedding.dimension);
    }
    
    #[test]
    fn test_cosine_similarity() {
        let engine = EmbeddingEngine::from_models_dir().unwrap();
        let emb1 = engine.embed("The cat sat on the mat").unwrap();
        let emb2 = engine.embed("A feline rested on the rug").unwrap();
        let emb3 = engine.embed("Quantum mechanics is fascinating").unwrap();
        
        let sim_similar = emb1.cosine_similarity(&emb2);
        let sim_different = emb1.cosine_similarity(&emb3);
        
        assert!(sim_similar > sim_different, "Similar sentences should have higher similarity");
        println!("Similar: {:.4}, Different: {:.4}", sim_similar, sim_different);
    }
}
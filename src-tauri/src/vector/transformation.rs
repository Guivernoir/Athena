use crate::embedding::EmbeddingEngine;
use crate::preprocessor::formatter::*;
use crate::security::{VectorSecurity, Quantizer, QuantizationError};
use crate::security::quantizer::QuantizationConfig;
use thiserror::Error;
use std::sync::Arc;

#[derive(Error, Debug)]
pub enum TransformationError {
    #[error("Embedding generation failed: {0}")]
    EmbeddingFailed(#[from] crate::embedding::EmbeddingError),
    #[error("Input formatting error: {0}")]
    FormattingError(#[from] FormatterError),
    #[error("Vector dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    #[error("Vector quantization failed: {0}")]
    QuantizationFailed(#[from] QuantizationError),
    #[error("Vector security processing failed: {0}")]
    SecurityFailed(#[from] anyhow::Error),
}

/// Transforms raw input data into secured vectorized SqliteVecRecord
pub struct DataTransformer {
    embedding_engine: Arc<EmbeddingEngine>,
    quantizer: Quantizer,
    encryption_key: [u8; 32],
    expected_dim: usize,
}

impl DataTransformer {
    pub fn new(embedding_engine: Arc<EmbeddingEngine>) -> Result<Self, TransformationError> {
        let expected_dim = embedding_engine.embedding_dimension();
        let quantizer = Quantizer::new(QuantizationConfig::default())
            .map_err(TransformationError::QuantizationFailed)?;
        let encryption_key = VectorSecurity::generate_key();
        
        Ok(Self {
            embedding_engine,
            quantizer,
            encryption_key,
            expected_dim,
        })
    }

    /// Transforms raw text into a complete secured SqliteVecRecord with embeddings
    pub async fn transform_text(
        &self,
        context: Context,
        tokens: TokenInfo,
        mode: Mode,
        proficiency: Proficiency,
        personality: Personality,
    ) -> Result<SecuredFormattedInput, TransformationError> {
        // Generate embedding from cleaned text
        let embedding = self.embedding_engine
            .embed(&context.raw_input)
            .await?;

        // Validate dimensions match collection requirements
        if embedding.vector.len() != self.expected_dim {
            return Err(TransformationError::DimensionMismatch {
                expected: self.expected_dim,
                actual: embedding.vector.len(),
            });
        }

        // Apply security pipeline: quantize -> compress -> encrypt
        let secured_vector = VectorSecurity::prepare_for_storage(
            &embedding.vector,
            &self.encryption_key,
            &self.quantizer,
        ).map_err(TransformationError::SecurityFailed)?;

        // Create formatted input with the secured vector
        SecuredFormattedInput::new(
            context,
            tokens,
            mode,
            proficiency,
            personality,
            secured_vector,
        )
        .map_err(Into::into)
    }

    /// Batch version of transform_text for multiple inputs
    pub async fn transform_batch(
        &self,
        inputs: Vec<(Context, TokenInfo, Mode, Proficiency, Personality)>,
    ) -> Result<Vec<SecuredFormattedInput>, TransformationError> {
        let mut results = Vec::with_capacity(inputs.len());
        
        for (context, tokens, mode, proficiency, personality) in inputs {
            let formatted = self.transform_text(
                context,
                tokens,
                mode,
                proficiency,
                personality,
            ).await?;
            results.push(formatted);
        }
        
        Ok(results)
    }

    /// Updates an existing record with new embeddings and re-secures it
    pub async fn update_with_reembed(
        &self,
        mut secured_input: SecuredFormattedInput,
    ) -> Result<SecuredFormattedInput, TransformationError> {
        // Generate new embedding from raw input
        let new_embedding = self.embedding_engine
            .embed(&secured_input.context.raw_input)
            .await?;

        // Re-secure the new embedding
        let secured_vector = VectorSecurity::prepare_for_storage(
            &new_embedding.vector,
            &self.encryption_key,
            &self.quantizer,
        ).map_err(TransformationError::SecurityFailed)?;

        // Update the secured record
        secured_input.secured_record.vector = secured_vector;
        secured_input.secured_record.updated_at = current_timestamp();
        
        Ok(secured_input)
    }

    /// Recovers the original vector from secured storage (for similarity search or analysis)
    pub fn recover_vector(&self, secured_vector: &[u8]) -> Result<Vec<f32>, TransformationError> {
        VectorSecurity::restore_from_storage(secured_vector, &self.encryption_key, &self.quantizer)
            .map_err(TransformationError::SecurityFailed)
    }

    /// Gets the expected vector dimensions for validation
    pub fn expected_dimensions(&self) -> usize {
        self.expected_dim
    }

    /// Regenerates the encryption key (invalidates all existing secured vectors)
    pub fn regenerate_key(&mut self) {
        self.encryption_key = VectorSecurity::generate_key();
    }

    /// Gets a reference to the current encryption key (for backup/restore scenarios)
    pub fn encryption_key(&self) -> &[u8; 32] {
        &self.encryption_key
    }
}

/// Secured version of SqliteVecRecord with encrypted vector data
pub struct SecuredFormattedInput {
    pub context: Context,
    pub tokens: TokenInfo,
    pub mode: Mode,
    pub proficiency: Proficiency,
    pub personality: Personality,
    pub secured_record: SecuredSqliteVecRecord,
}

impl SecuredFormattedInput {
    pub fn new(
        context: Context,
        tokens: TokenInfo,
        mode: Mode,
        proficiency: Proficiency,
        personality: Personality,
        secured_vector: Vec<u8>,
    ) -> Result<Self, FormatterError> {
        let now = current_timestamp();
        let token_preview = tokens.tokens
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");

        let secured_record = SecuredSqliteVecRecord {
            id: generate_point_id(),
            vector: secured_vector,
            raw_input: context.raw_input.clone(),
            action: context.action.clone(),
            domain: context.domain.clone(),
            topic: context.topic.clone(),
            mode: mode.to_string(),
            proficiency: proficiency.to_string(),
            personality: personality.to_string(),
            word_count: tokens.word_count as i64,
            sentence_count: tokens.sentence_count as i64,
            token_preview,
            created_at: now,
            updated_at: now,
        };

        Ok(Self {
            context,
            tokens,
            mode,
            proficiency,
            personality,
            secured_record,
        })
    }
}

/// Secured version of SqliteVecRecord with encrypted vector data
pub struct SecuredSqliteVecRecord {
    pub id: String,
    pub vector: Vec<u8>,  // Secured vector data instead of raw f32
    pub raw_input: String,
    pub action: String,
    pub domain: String,
    pub topic: String,
    pub mode: String,
    pub proficiency: String,
    pub personality: String,
    pub word_count: i64,
    pub sentence_count: i64,
    pub token_preview: String,
    pub created_at: i64,
    pub updated_at: i64,
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn generate_point_id() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    current_timestamp().hash(&mut hasher);
    format!("point_{}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::EmbeddingEngine;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_secured_text_transformation() {
        let engine = Arc::new(EmbeddingEngine::from_models_dir().unwrap());
        let transformer = DataTransformer::new(Arc::clone(&engine)).unwrap();

        let context = Context {
            raw_input: "Explain Rust's ownership model".to_string(),
            action: "explain".to_string(),
            domain: "rust".to_string(),
            topic: "ownership".to_string(),
        };

        let tokens = TokenInfo {
            word_count: 5,
            sentence_count: 1,
            tokens: vec!["Explain", "Rust's", "ownership", "model"].into_iter().map(|s| s.to_string()).collect(),
        };

        let result = transformer.transform_text(
            context,
            tokens,
            Mode::Assistant,
            Proficiency::Intermediate,
            Personality::Aurora,
        ).await;

        assert!(result.is_ok());
        let secured = result.unwrap();
        
        // Vector should be secured (not raw f32 values)
        assert!(!secured.secured_record.vector.is_empty());
        assert_eq!(secured.secured_record.domain, "rust");
        
        // Test vector recovery
        let recovered = transformer.recover_vector(&secured.secured_record.vector).unwrap();
        assert_eq!(recovered.len(), engine.embedding_dimension());
    }

    #[tokio::test]
    async fn test_secured_batch_transformation() {
        let engine = Arc::new(EmbeddingEngine::from_models_dir().unwrap());
        let transformer = DataTransformer::new(Arc::clone(&engine)).unwrap();

        let inputs = vec![
            (
                Context {
                    raw_input: "Explain borrowing in Rust".to_string(),
                    action: "explain".to_string(),
                    domain: "rust".to_string(),
                    topic: "borrowing".to_string(),
                },
                TokenInfo {
                    word_count: 4,
                    sentence_count: 1,
                    tokens: vec!["Explain", "borrowing", "in", "Rust"].into_iter().map(|s| s.to_string()).collect(),
                },
                Mode::Assistant,
                Proficiency::Intermediate,
                Personality::Ekaterina,
            ),
            (
                Context {
                    raw_input: "Show me Python list comprehension".to_string(),
                    action: "demonstrate".to_string(),
                    domain: "python".to_string(),
                    topic: "list comprehension".to_string(),
                },
                TokenInfo {
                    word_count: 4,
                    sentence_count: 1,
                    tokens: vec!["Show", "me", "Python", "list", "comprehension"].into_iter().map(|s| s.to_string()).collect(),
                },
                Mode::Tutor,
                Proficiency::Beginner,
                Personality::Erika,
            ),
        ];

        let results = transformer.transform_batch(inputs).await.unwrap();
        assert_eq!(results.len(), 2);
        
        // Secured vectors should be different
        assert_ne!(results[0].secured_record.vector, results[1].secured_record.vector);
        
        // Both should be recoverable
        let recovered_0 = transformer.recover_vector(&results[0].secured_record.vector).unwrap();
        let recovered_1 = transformer.recover_vector(&results[1].secured_record.vector).unwrap();
        
        assert_eq!(recovered_0.len(), engine.embedding_dimension());
        assert_eq!(recovered_1.len(), engine.embedding_dimension());
        assert_ne!(recovered_0, recovered_1);
    }

    #[tokio::test]
    async fn test_vector_recovery_pipeline() {
        let engine = Arc::new(EmbeddingEngine::from_models_dir().unwrap());
        let transformer = DataTransformer::new(Arc::clone(&engine)).unwrap();
        
        // Create test vector
        let test_vector = vec![1.0, -0.5, 0.25, -0.125];
        
        // Secure it
        let secured = VectorSecurity::prepare_for_storage(
            &test_vector,
            transformer.encryption_key(),
            &transformer.quantizer,
        ).unwrap();
        
        // Recover it
        let recovered = transformer.recover_vector(&secured).unwrap();
        
        // Should have same length
        assert_eq!(test_vector.len(), recovered.len());
        
        // Values should be approximately equal (quantization introduces some loss)
        for (orig, rec) in test_vector.iter().zip(recovered.iter()) {
            assert!((orig - rec).abs() < 0.1, "Value mismatch: {} vs {}", orig, rec);
        }
    }
}
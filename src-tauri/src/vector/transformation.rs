use crate::embedding::EmbeddingEngine;
use crate::vector::security;
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
}

/// Transforms raw input data into vectorized Qdrant points
pub struct DataTransformer {
    embedding_engine: Arc<EmbeddingEngine>,
    expected_dim: usize,
}

impl DataTransformer {
    pub fn new(embedding_engine: Arc<EmbeddingEngine>) -> Self {
        let expected_dim = embedding_engine.embedding_dimension();
        Self {
            embedding_engine,
            expected_dim,
        }
    }

    /// Transforms raw text into a complete Qdrant point with embeddings
    pub async fn transform_text(
        &self,
        context: Context,
        tokens: TokenInfo,
        mode: Mode,
        proficiency: Proficiency,
        personality: Personality,
    ) -> Result<FormattedInput, TransformationError> {
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

        // Create formatted input with the generated embedding
        FormattedInput::new(
            context,
            tokens,
            mode,
            proficiency,
            personality,
            embedding.vector,
        )
        .map_err(Into::into)
    }

    /// Batch version of transform_text for multiple inputs
    pub async fn transform_batch(
        &self,
        inputs: Vec<(Context, TokenInfo, Mode, Proficiency, Personality)>,
    ) -> Result<Vec<FormattedInput>, TransformationError> {
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

    /// Updates an existing point with new embeddings
    pub async fn update_with_reembed(
        &self,
        mut formatted_input: FormattedInput,
    ) -> Result<FormattedInput, TransformationError> {
        let new_embedding = self.embedding_engine
            .embed(&formatted_input.context.raw_input)
            .await?;

        // Replace the vector while preserving other metadata
        formatted_input.qdrant_point.vector = new_embedding.vector;
        formatted_input.qdrant_point.payload.updated_at = current_timestamp();
        
        Ok(formatted_input)
    }

    /// Gets the expected vector dimensions for validation
    pub fn expected_dimensions(&self) -> usize {
        self.expected_dim
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::EmbeddingEngine;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_text_transformation() {
        let engine = Arc::new(EmbeddingEngine::from_models_dir().unwrap());
        let transformer = DataTransformer::new(Arc::clone(&engine));

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
        let formatted = result.unwrap();
        assert_eq!(formatted.qdrant_point.vector.len(), engine.embedding_dimension());
        assert_eq!(formatted.qdrant_point.payload.domain, "rust");
    }

    #[tokio::test]
    async fn test_batch_transformation() {
        let engine = Arc::new(EmbeddingEngine::from_models_dir().unwrap());
        let transformer = DataTransformer::new(Arc::clone(&engine));

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
        assert_ne!(results[0].qdrant_point.vector, results[1].qdrant_point.vector);
    }
}
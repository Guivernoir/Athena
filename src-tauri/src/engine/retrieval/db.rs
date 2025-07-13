use crate::vector::query::VectorSearcher;
use crate::vector::schema::{SearchResult, SearchFilters};
use std::sync::Arc;
use tracing::{info, instrument};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Vector search failed: {0}")]
    SearchFailed(String),
    #[error("Invalid query parameters: {0}")]
    InvalidParameters(String),
}

pub struct VectorDB {
    searcher: Arc<VectorSearcher>,
}

impl VectorDB {
    #[instrument]
    pub fn new(searcher: Arc<VectorSearcher>) -> Self {
        info!("VectorDB initialized with existing searcher");
        Self { searcher }
    }

    #[instrument(skip(self, embedding))]
    pub async fn query(
        &self,
        embedding: &[f32],
        top_k: usize,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<KnowledgeItem>, DatabaseError> {
        let results = self.searcher
            .search_similar(embedding.to_vec(), top_k as u64, filters)
            .await
            .map_err(|e| DatabaseError::SearchFailed(e.to_string()))?;

        Ok(results
            .into_iter()
            .map(|r| KnowledgeItem {
                content: r.payload.cleaned_input,
                source: "db".to_string(),
                score: r.score,
                metadata: Some(format!(
                    "domain={}, topic={}, action={}",
                    r.payload.domain, r.payload.topic, r.payload.action
                )),
            })
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct KnowledgeItem {
    pub content: String,
    pub source: String,
    pub score: f32,
    pub metadata: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::{client::VectorClient, schema::SearchPayload};
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_db_wrapper() {
        // Setup mock searcher
        let mut mock_searcher = mockall::mock! {
            VectorSearcher {
                pub async fn search_similar(
                    &self,
                    query_vector: Vec<f32>,
                    limit: u64,
                    filters: Option<SearchFilters>,
                ) -> Result<Vec<SearchResult>, VectorDbError>;
            }
        };

        // Expect single call with our test vector
        mock_searcher.expect_search_similar()
            .with(
                eq(vec![0.1, 0.2, 0.3]),
                eq(5),
                eq(None::<SearchFilters>)
            )
            .returning(|_, _, _| {
                Ok(vec![SearchResult {
                    id: "test1".to_string(),
                    score: 0.9,
                    vector: vec![0.1, 0.2, 0.3],
                    payload: SearchPayload {
                        cleaned_input: "Test content".to_string(),
                        raw_input: "Raw test".to_string(),
                        action: "explain".to_string(),
                        domain: "rust".to_string(),
                        topic: "ownership".to_string(),
                        ..Default::default()
                    },
                }])
            });

        let db = VectorDB::new(Arc::new(mock_searcher));
        let results = db.query(&[0.1, 0.2, 0.3], 5, None).await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Test content");
        assert!(results[0].metadata.unwrap().contains("domain=rust"));
    }
}
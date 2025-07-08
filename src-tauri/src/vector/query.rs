use crate::{QdrantClient, VectorDbError};
use crate::schema::{SearchResult, SearchFilters, SearchPayload};
use qdrant_client::qdrant::{SearchPoints, ScoredPoint, Value};
use std::sync::Arc;

pub struct VectorSearcher {
    client: Arc<QdrantClient>,
}

impl VectorSearcher {
    pub fn new(client: Arc<QdrantClient>) -> Self {
        Self { client }
    }
    
    pub async fn search_similar(
        &self,
        query_vector: Vec<f32>,
        limit: u64,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<SearchResult>, VectorDbError> {
        // Validate vector dimensions
        if query_vector.len() != self.client.get_vector_size() as usize {
            return Err(VectorDbError::InvalidVectorDimensions {
                expected: self.client.get_vector_size() as usize,
                actual: query_vector.len(),
            });
        }
        
        let search_points = SearchPoints {
            collection_name: self.client.get_collection_name().to_string(),
            vector: query_vector,
            limit,
            with_payload: Some(true.into()),
            with_vectors: Some(true.into()),
            filter: filters.and_then(|f| f.to_qdrant_filter()),
            ..Default::default()
        };
        
        let search_result = self.client
            .get_client()
            .search_points(&search_points)
            .await
            .map_err(|e| VectorDbError::QueryFailed(format!("Search failed: {}", e)))?;
        
        let results = search_result
            .result
            .into_iter()
            .map(|point| self.scored_point_to_search_result(point))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(results)
    }
    
    pub async fn search_by_id(&self, id: &str) -> Result<Option<SearchResult>, VectorDbError> {
        let get_points = qdrant_client::qdrant::GetPoints {
            collection_name: self.client.get_collection_name().to_string(),
            ids: vec![id.to_string().into()],
            with_payload: Some(true.into()),
            with_vectors: Some(true.into()),
            ..Default::default()
        };
        
        let get_result = self.client
            .get_client()
            .get_points(&get_points)
            .await
            .map_err(|e| VectorDbError::QueryFailed(format!("Get by ID failed: {}", e)))?;
        
        if let Some(point) = get_result.result.into_iter().next() {
            let scored_point = ScoredPoint {
                id: point.id,
                payload: point.payload,
                vectors: point.vectors,
                score: 1.0, // Perfect match since we're getting by ID
            };
            
            Ok(Some(self.scored_point_to_search_result(scored_point)?))
        } else {
            Ok(None)
        }
    }
    
    pub async fn search_with_scroll(
        &self,
        filters: Option<SearchFilters>,
        limit: u32,
        offset: Option<String>,
    ) -> Result<(Vec<SearchResult>, Option<String>), VectorDbError> {
        let scroll_points = qdrant_client::qdrant::ScrollPoints {
            collection_name: self.client.get_collection_name().to_string(),
            filter: filters.and_then(|f| f.to_qdrant_filter()),
            limit: Some(limit),
            with_payload: Some(true.into()),
            with_vectors: Some(true.into()),
            offset: offset.map(|o| o.into()),
            ..Default::default()
        };
        
        let scroll_result = self.client
            .get_client()
            .scroll(&scroll_points)
            .await
            .map_err(|e| VectorDbError::QueryFailed(format!("Scroll failed: {}", e)))?;
        
        let results = scroll_result
            .result
            .into_iter()
            .map(|point| {
                let scored_point = ScoredPoint {
                    id: point.id,
                    payload: point.payload,
                    vectors: point.vectors,
                    score: 1.0, // No scoring in scroll
                };
                self.scored_point_to_search_result(scored_point)
            })
            .collect::<Result<Vec<_>, _>>()?;
        
        let next_offset = scroll_result.next_page_offset.map(|o| o.to_string());
        
        Ok((results, next_offset))
    }
    
    pub async fn count_points(&self, filters: Option<SearchFilters>) -> Result<u64, VectorDbError> {
        let count_points = qdrant_client::qdrant::CountPoints {
            collection_name: self.client.get_collection_name().to_string(),
            filter: filters.and_then(|f| f.to_qdrant_filter()),
            exact: Some(false), // Use approximate counting for performance
            ..Default::default()
        };
        
        let count_result = self.client
            .get_client()
            .count(&count_points)
            .await
            .map_err(|e| VectorDbError::QueryFailed(format!("Count failed: {}", e)))?;
        
        Ok(count_result.result.map(|r| r.count).unwrap_or(0))
    }
    
    pub async fn search_recommendations(
        &self,
        positive_ids: Vec<String>,
        negative_ids: Vec<String>,
        limit: u64,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<SearchResult>, VectorDbError> {
        let recommend_points = qdrant_client::qdrant::RecommendPoints {
            collection_name: self.client.get_collection_name().to_string(),
            positive: positive_ids.into_iter().map(|id| id.into()).collect(),
            negative: negative_ids.into_iter().map(|id| id.into()).collect(),
            limit,
            with_payload: Some(true.into()),
            with_vectors: Some(true.into()),
            filter: filters.and_then(|f| f.to_qdrant_filter()),
            ..Default::default()
        };
        
        let recommend_result = self.client
            .get_client()
            .recommend(&recommend_points)
            .await
            .map_err(|e| VectorDbError::QueryFailed(format!("Recommend failed: {}", e)))?;
        
        let results = recommend_result
            .result
            .into_iter()
            .map(|point| self.scored_point_to_search_result(point))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(results)
    }
    
    fn scored_point_to_search_result(
        &self,
        point: ScoredPoint,
    ) -> Result<SearchResult, VectorDbError> {
        // Convert the payload to SearchPayload if needed
        let payload = point
            .payload
            .map(|p| SearchPayload::from_qdrant_payload(p))
            .transpose()?;

        // Extract vector(s)
        let vector = match point.vectors {
            Some(qdrant_client::qdrant::vectors::Vectors::Vector(v)) => Some(v.data),
            Some(qdrant_client::qdrant::vectors::Vectors::VectorsMap(mut map)) => {
                // If there are multiple named vectors, pick the first one
                map.values_mut().next().map(|v| v.data.clone())
            }
            None => None,
        };

        Ok(SearchResult {
            id: match point.id {
                qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid) => uuid,
                qdrant_client::qdrant::point_id::PointIdOptions::Num(num) => num.to_string(),
            },
            score: point.score,
            vector,
            payload,
        })
    }
}
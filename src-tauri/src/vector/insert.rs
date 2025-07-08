use crate::{QdrantClient, VectorDbError};
use crate::vector::transformation::FormattedInput;
use crate::formatter::QdrantPoint;
use crate::schema::qdrant_point_to_point_struct;
use qdrant_client::qdrant::{UpsertPoints, PointStruct};
use std::sync::Arc;

pub struct VectorInserter {
    client: Arc<QdrantClient>,
}

impl VectorInserter {
    pub fn new(client: Arc<QdrantClient>) -> Self {
        Self { client }
    }
    
    pub async fn insert_single(&self, formatted_input: &FormattedInput) -> Result<String, VectorDbError> {
        let point = &formatted_input.qdrant_point;
        
        // Validate vector dimensions
        if point.vector.len() != self.client.get_vector_size() as usize {
            return Err(VectorDbError::InvalidVectorDimensions {
                expected: self.client.get_vector_size() as usize,
                actual: point.vector.len(),
            });
        }
        
        let point_struct = qdrant_point_to_point_struct(point);
        
        let upsert_points = UpsertPoints {
            collection_name: self.client.get_collection_name().to_string(),
            points: vec![point_struct],
            ..Default::default()
        };
        
        self.client
            .get_client()
            .upsert_points(&upsert_points)
            .await
            .map_err(|e| VectorDbError::InsertFailed(format!("Failed to insert point: {}", e)))?;
        
        Ok(point.id.clone())
    }
    
    pub async fn insert_batch(&self, formatted_inputs: &[FormattedInput]) -> Result<Vec<String>, VectorDbError> {
        if formatted_inputs.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut point_structs = Vec::new();
        let mut ids = Vec::new();
        
        for formatted_input in formatted_inputs {
            let point = &formatted_input.qdrant_point;
            
            // Validate vector dimensions
            if point.vector.len() != self.client.get_vector_size() as usize {
                return Err(VectorDbError::InvalidVectorDimensions {
                    expected: self.client.get_vector_size() as usize,
                    actual: point.vector.len(),
                });
            }
            
            point_structs.push(qdrant_point_to_point_struct(point));
            ids.push(point.id.clone());
        }
        
        let upsert_points = UpsertPoints {
            collection_name: self.client.get_collection_name().to_string(),
            points: point_structs,
            ..Default::default()
        };
        
        self.client
            .get_client()
            .upsert_points(&upsert_points)
            .await
            .map_err(|e| VectorDbError::InsertFailed(format!("Failed to insert batch: {}", e)))?;
        
        Ok(ids)
    }
    
    pub async fn insert_raw_point(&self, point: &QdrantPoint) -> Result<String, VectorDbError> {
        // Validate vector dimensions
        if point.vector.len() != self.client.get_vector_size() as usize {
            return Err(VectorDbError::InvalidVectorDimensions {
                expected: self.client.get_vector_size() as usize,
                actual: point.vector.len(),
            });
        }
        
        let point_struct = qdrant_point_to_point_struct(point);
        
        let upsert_points = UpsertPoints {
            collection_name: self.client.get_collection_name().to_string(),
            points: vec![point_struct],
            ..Default::default()
        };
        
        self.client
            .get_client()
            .upsert_points(&upsert_points)
            .await
            .map_err(|e| VectorDbError::InsertFailed(format!("Failed to insert raw point: {}", e)))?;
        
        Ok(point.id.clone())
    }
    
    pub async fn update_point(&self, id: &str, formatted_input: &FormattedInput) -> Result<(), VectorDbError> {
        let point = &formatted_input.qdrant_point;
        
        // Validate vector dimensions
        if point.vector.len() != self.client.get_vector_size() as usize {
            return Err(VectorDbError::InvalidVectorDimensions {
                expected: self.client.get_vector_size() as usize,
                actual: point.vector.len(),
            });
        }
        
        // Create a new point with the specified id
        let mut updated_point = point.clone();
        updated_point.id = id.to_string();
        updated_point.payload.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let point_struct = qdrant_point_to_point_struct(&updated_point);
        
        let upsert_points = UpsertPoints {
            collection_name: self.client.get_collection_name().to_string(),
            points: vec![point_struct],
            ..Default::default()
        };
        
        self.client
            .get_client()
            .upsert_points(&upsert_points)
            .await
            .map_err(|e| VectorDbError::InsertFailed(format!("Failed to update point: {}", e)))?;
        
        Ok(())
    }
    
    pub async fn delete_point(&self, id: &str) -> Result<(), VectorDbError> {
        let delete_points = qdrant_client::qdrant::DeletePoints {
            collection_name: self.client.get_collection_name().to_string(),
            points: Some(qdrant_client::qdrant::PointsSelector {
                points_selector_one_of: Some(qdrant_client::qdrant::points_selector::PointsSelectorOneOf::Points(
                    qdrant_client::qdrant::PointsIdsList {
                        ids: vec![id.to_string().into()],
                    }
                )),
            }),
            ..Default::default()
        };
        
        self.client
            .get_client()
            .delete_points(&delete_points)
            .await
            .map_err(|e| VectorDbError::InsertFailed(format!("Failed to delete point: {}", e)))?;
        
        Ok(())
    }
    
    pub async fn delete_batch(&self, ids: &[String]) -> Result<(), VectorDbError> {
        if ids.is_empty() {
            return Ok(());
        }
        
        let delete_points = qdrant_client::qdrant::DeletePoints {
            collection_name: self.client.get_collection_name().to_string(),
            points: Some(qdrant_client::qdrant::PointsSelector {
                points_selector_one_of: Some(qdrant_client::qdrant::points_selector::PointsSelectorOneOf::Points(
                    qdrant_client::qdrant::PointsIdsList {
                        ids: ids.iter().map(|id| id.clone().into()).collect(),
                    }
                )),
            }),
            ..Default::default()
        };
        
        self.client
            .get_client()
            .delete_points(&delete_points)
            .await
            .map_err(|e| VectorDbError::InsertFailed(format!("Failed to delete batch: {}", e)))?;
        
        Ok(())
    }
}
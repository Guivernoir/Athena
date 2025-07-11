use crate::{QdrantClient, VectorDbError};
use crate::vector::transformation::FormattedInput;
use crate::formatter::QdrantPoint;
use crate::schema::qdrant_point_to_point_struct;
use qdrant_client::qdrant::{UpsertPoints, PointStruct};
use std::sync::Arc;
use crate::security::{VectorSecurity, Quantizer}; // <-- import security

pub struct VectorInserter {
    client: Arc<QdrantClient>,
    key: [u8; 32],
    quantizer: Quantizer,
}

impl VectorInserter {
    pub fn new(client: Arc<QdrantClient>, key: [u8; 32], quantizer: Quantizer) -> Self {
        Self { client, key, quantizer }
    }
    
    pub async fn insert_single(&self, formatted_input: &FormattedInput) -> Result<String, VectorDbError> {
        let mut point = formatted_input.qdrant_point.clone();

        // Validate vector dimensions
        if point.vector.len() != self.client.get_vector_size() as usize {
            return Err(VectorDbError::InvalidVectorDimensions {
                expected: self.client.get_vector_size() as usize,
                actual: point.vector.len(),
            });
        }

        // Secure the vector
        let secured = VectorSecurity::prepare_for_storage(&point.vector, &self.key, &self.quantizer)
            .map_err(|e| VectorDbError::InsertFailed(format!("Security pipeline failed: {}", e)))?;
        point.vector = vec![]; // clear original
        point.payload.secured_vector = Some(secured); // assumes payload has this field

        let point_struct = qdrant_point_to_point_struct(&point);

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
            let mut point = formatted_input.qdrant_point.clone();

            // Validate vector dimensions
            if point.vector.len() != self.client.get_vector_size() as usize {
                return Err(VectorDbError::InvalidVectorDimensions {
                    expected: self.client.get_vector_size() as usize,
                    actual: point.vector.len(),
                });
            }

            // Secure the vector
            let secured = VectorSecurity::prepare_for_storage(&point.vector, &self.key, &self.quantizer)
                .map_err(|e| VectorDbError::InsertFailed(format!("Security pipeline failed: {}", e)))?;
            point.vector = vec![];
            point.payload.secured_vector = Some(secured);

            point_structs.push(qdrant_point_to_point_struct(&point));
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
        let mut point = point.clone();

        // Validate vector dimensions
        if point.vector.len() != self.client.get_vector_size() as usize {
            return Err(VectorDbError::InvalidVectorDimensions {
                expected: self.client.get_vector_size() as usize,
                actual: point.vector.len(),
            });
        }

        // Secure the vector
        let secured = VectorSecurity::prepare_for_storage(&point.vector, &self.key, &self.quantizer)
            .map_err(|e| VectorDbError::InsertFailed(format!("Security pipeline failed: {}", e)))?;
        point.vector = vec![];
        point.payload.secured_vector = Some(secured);

        let point_struct = qdrant_point_to_point_struct(&point);

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
        let mut point = formatted_input.qdrant_point.clone();

        // Validate vector dimensions
        if point.vector.len() != self.client.get_vector_size() as usize {
            return Err(VectorDbError::InvalidVectorDimensions {
                expected: self.client.get_vector_size() as usize,
                actual: point.vector.len(),
            });
        }

        // Secure the vector
        let secured = VectorSecurity::prepare_for_storage(&point.vector, &self.key, &self.quantizer)
            .map_err(|e| VectorDbError::InsertFailed(format!("Security pipeline failed: {}", e)))?;
        point.vector = vec![];
        point.payload.secured_vector = Some(secured);

        // Create a new point with the specified id
        point.id = id.to_string();
        point.payload.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let point_struct = qdrant_point_to_point_struct(&point);

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
use crate::{SqliteVecClient, VectorDbError};
use crate::vector::transformation::FormattedInput;
use crate::formatter::QdrantPoint;
use std::sync::Arc;
use crate::security::{VectorSecurity, Quantizer};
use sqlx::Row;

pub struct VectorInserter {
    client: Arc<SqliteVecClient>,
    key: [u8; 32],
    quantizer: Quantizer,
}

impl VectorInserter {
    pub fn new(client: Arc<SqliteVecClient>, key: [u8; 32], quantizer: Quantizer) -> Self {
        Self { client, key, quantizer }
    }
    
    pub async fn insert_single(&self, formatted_input: &FormattedInput) -> Result<String, VectorDbError> {
        let mut point = formatted_input.qdrant_point.clone();

        // Validate vector dimensions
        if point.vector.len() != self.client.get_vector_size() {
            return Err(VectorDbError::InvalidVectorDimensions {
                expected: self.client.get_vector_size(),
                actual: point.vector.len(),
            });
        }

        // Secure the vector
        let secured = VectorSecurity::prepare_for_storage(&point.vector, &self.key, &self.quantizer)
            .map_err(|e| VectorDbError::InsertFailed(format!("Security pipeline failed: {}", e)))?;

        // Convert vector to bytes for sqlite-vec
        let vector_bytes = vector_to_bytes(&point.vector);

        let insert_sql = format!(
            r#"
            INSERT INTO {} (
                id, vector, raw_input, cleaned_input, action, domain, topic, mode, 
                proficiency, personality, word_count, sentence_count, token_preview,
                complexity_score, estimated_processing_time, suggested_response_length,
                domain_category, complexity_tier, proficiency_level, created_at, updated_at,
                secured_vector
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            self.client.get_table_name()
        );

        sqlx::query(&insert_sql)
            .bind(&point.id)
            .bind(&vector_bytes)
            .bind(&point.payload.raw_input)
            .bind(&point.payload.cleaned_input)
            .bind(&point.payload.action)
            .bind(&point.payload.domain)
            .bind(&point.payload.topic)
            .bind(&point.payload.mode)
            .bind(&point.payload.proficiency)
            .bind(&point.payload.personality)
            .bind(point.payload.word_count)
            .bind(point.payload.sentence_count)
            .bind(&point.payload.token_preview)
            .bind(point.payload.complexity_score)
            .bind(point.payload.estimated_processing_time)
            .bind(&point.payload.suggested_response_length)
            .bind(&point.payload.domain_category)
            .bind(&point.payload.complexity_tier)
            .bind(&point.payload.proficiency_level)
            .bind(point.payload.created_at)
            .bind(point.payload.updated_at)
            .bind(&secured)
            .execute(self.client.get_pool())
            .await
            .map_err(|e| VectorDbError::InsertFailed(format!("Failed to insert point: {}", e)))?;

        // Insert into vector index
        let index_insert_sql = format!(
            "INSERT INTO {}_vec_index (rowid, vector) VALUES (last_insert_rowid(), ?)",
            self.client.get_table_name()
        );
        
        sqlx::query(&index_insert_sql)
            .bind(&vector_bytes)
            .execute(self.client.get_pool())
            .await
            .map_err(|e| VectorDbError::InsertFailed(format!("Failed to insert vector index: {}", e)))?;

        Ok(point.id)
    }
    
    pub async fn insert_batch(&self, formatted_inputs: &[FormattedInput]) -> Result<Vec<String>, VectorDbError> {
        if formatted_inputs.is_empty() {
            return Ok(Vec::new());
        }

        let mut tx = self.client.get_pool()
            .begin()
            .await
            .map_err(|e| VectorDbError::InsertFailed(format!("Failed to begin transaction: {}", e)))?;

        let mut ids = Vec::new();

        for formatted_input in formatted_inputs {
            let mut point = formatted_input.qdrant_point.clone();

            // Validate vector dimensions
            if point.vector.len() != self.client.get_vector_size() {
                return Err(VectorDbError::InvalidVectorDimensions {
                    expected: self.client.get_vector_size(),
                    actual: point.vector.len(),
                });
            }

            // Secure the vector
            let secured = VectorSecurity::prepare_for_storage(&point.vector, &self.key, &self.quantizer)
                .map_err(|e| VectorDbError::InsertFailed(format!("Security pipeline failed: {}", e)))?;

            let vector_bytes = vector_to_bytes(&point.vector);

            let insert_sql = format!(
                r#"
                INSERT INTO {} (
                    id, vector, raw_input, cleaned_input, action, domain, topic, mode, 
                    proficiency, personality, word_count, sentence_count, token_preview,
                    complexity_score, estimated_processing_time, suggested_response_length,
                    domain_category, complexity_tier, proficiency_level, created_at, updated_at,
                    secured_vector
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                self.client.get_table_name()
            );

            sqlx::query(&insert_sql)
                .bind(&point.id)
                .bind(&vector_bytes)
                .bind(&point.payload.raw_input)
                .bind(&point.payload.cleaned_input)
                .bind(&point.payload.action)
                .bind(&point.payload.domain)
                .bind(&point.payload.topic)
                .bind(&point.payload.mode)
                .bind(&point.payload.proficiency)
                .bind(&point.payload.personality)
                .bind(point.payload.word_count)
                .bind(point.payload.sentence_count)
                .bind(&point.payload.token_preview)
                .bind(point.payload.complexity_score)
                .bind(point.payload.estimated_processing_time)
                .bind(&point.payload.suggested_response_length)
                .bind(&point.payload.domain_category)
                .bind(&point.payload.complexity_tier)
                .bind(&point.payload.proficiency_level)
                .bind(point.payload.created_at)
                .bind(point.payload.updated_at)
                .bind(&secured)
                .execute(&mut *tx)
                .await
                .map_err(|e| VectorDbError::InsertFailed(format!("Failed to insert point: {}", e)))?;

            // Get the rowid for vector index
            let rowid: i64 = sqlx::query("SELECT last_insert_rowid() as rowid")
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| VectorDbError::InsertFailed(format!("Failed to get rowid: {}", e)))?
                .get("rowid");

            // Insert into vector index
            let index_insert_sql = format!(
                "INSERT INTO {}_vec_index (rowid, vector) VALUES (?, ?)",
                self.client.get_table_name()
            );
            
            sqlx::query(&index_insert_sql)
                .bind(rowid)
                .bind(&vector_bytes)
                .execute(&mut *tx)
                .await
                .map_err(|e| VectorDbError::InsertFailed(format!("Failed to insert vector index: {}", e)))?;

            ids.push(point.id);
        }

        tx.commit()
            .await
            .map_err(|e| VectorDbError::InsertFailed(format!("Failed to commit transaction: {}", e)))?;

        Ok(ids)
    }
    
    pub async fn delete_point(&self, id: &str) -> Result<(), VectorDbError> {
        let mut tx = self.client.get_pool()
            .begin()
            .await
            .map_err(|e| VectorDbError::InsertFailed(format!("Failed to begin transaction: {}", e)))?;

        // Get rowid first
        let rowid_query = format!("SELECT rowid FROM {} WHERE id = ?", self.client.get_table_name());
        let rowid: Option<i64> = sqlx::query(&rowid_query)
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| VectorDbError::InsertFailed(format!("Failed to get rowid: {}", e)))?
            .map(|row| row.get("rowid"));

        if let Some(rowid) = rowid {
            // Delete from vector index
            let delete_index_sql = format!(
                "DELETE FROM {}_vec_index WHERE rowid = ?",
                self.client.get_table_name()
            );
            
            sqlx::query(&delete_index_sql)
                .bind(rowid)
                .execute(&mut *tx)
                .await
                .map_err(|e| VectorDbError::InsertFailed(format!("Failed to delete from vector index: {}", e)))?;
        }

        // Delete from main table
        let delete_sql = format!("DELETE FROM {} WHERE id = ?", self.client.get_table_name());
        sqlx::query(&delete_sql)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| VectorDbError::InsertFailed(format!("Failed to delete point: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| VectorDbError::InsertFailed(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }
}

fn vector_to_bytes(vector: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vector.len() * 4);
    for &value in vector {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn bytes_to_vector(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}
use crate::{SqliteVecClient, VectorDbError};
use crate::schema::{SearchResult, SearchFilters, SearchPayload};
use std::sync::Arc;
use sqlx::Row;

pub struct VectorSearcher {
    client: Arc<SqliteVecClient>,
}

impl VectorSearcher {
    pub fn new(client: Arc<SqliteVecClient>) -> Self {
        Self { client }
    }
    
    pub async fn search_similar(
        &self,
        query_vector: Vec<f32>,
        limit: u64,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<SearchResult>, VectorDbError> {
        // Validate vector dimensions
        if query_vector.len() != self.client.get_vector_size() {
            return Err(VectorDbError::InvalidVectorDimensions {
                expected: self.client.get_vector_size(),
                actual: query_vector.len(),
            });
        }

        let query_bytes = vector_to_bytes(&query_vector);
        
        // Build the search query with filters
        let (where_clause, bind_params) = self.build_filter_clause(filters);
        
        let search_sql = format!(
            r#"
            SELECT 
                t.id, t.raw_input, t.cleaned_input, t.action, t.domain, t.topic, t.mode,
                t.proficiency, t.personality, t.word_count, t.sentence_count, t.token_preview,
                t.complexity_score, t.estimated_processing_time, t.suggested_response_length,
                t.domain_category, t.complexity_tier, t.proficiency_level, t.created_at,
                t.vector, v.distance
            FROM {} t
            JOIN {}_vec_index v ON t.rowid = v.rowid
            WHERE v.vector MATCH ? {}
            ORDER BY v.distance
            LIMIT ?
            "#,
            self.client.get_table_name(),
            self.client.get_table_name(),
            where_clause
        );

        let mut query = sqlx::query(&search_sql)
            .bind(&query_bytes);

        // Bind filter parameters
        for param in bind_params {
            query = query.bind(param);
        }
        
        query = query.bind(limit as i64);

        let rows = query
            .fetch_all(self.client.get_pool())
            .await
            .map_err(|e| VectorDbError::QueryFailed(format!("Search failed: {}", e)))?;

        let results = rows
            .into_iter()
            .map(|row| self.row_to_search_result(row))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }
    
    pub async fn search_by_id(&self, id: &str) -> Result<Option<SearchResult>, VectorDbError> {
        let query_sql = format!(
            r#"
            SELECT 
                id, raw_input, cleaned_input, action, domain, topic, mode,
                proficiency, personality, word_count, sentence_count, token_preview,
                complexity_score, estimated_processing_time, suggested_response_length,
                domain_category, complexity_tier, proficiency_level, created_at, vector
            FROM {} WHERE id = ?
            "#,
            self.client.get_table_name()
        );

        let row = sqlx::query(&query_sql)
            .bind(id)
            .fetch_optional(self.client.get_pool())
            .await
            .map_err(|e| VectorDbError::QueryFailed(format!("Get by ID failed: {}", e)))?;

        if let Some(row) = row {
            Ok(Some(self.row_to_search_result_with_score(row, 1.0)?))
        } else {
            Ok(None)
        }
    }
    
    pub async fn search_with_pagination(
        &self,
        filters: Option<SearchFilters>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<SearchResult>, VectorDbError> {
        let (where_clause, bind_params) = self.build_filter_clause(filters);
        
        let query_sql = format!(
            r#"
            SELECT 
                id, raw_input, cleaned_input, action, domain, topic, mode,
                proficiency, personality, word_count, sentence_count, token_preview,
                complexity_score, estimated_processing_time, suggested_response_length,
                domain_category, complexity_tier, proficiency_level, created_at, vector
            FROM {} 
            {} 
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
            self.client.get_table_name(),
            if where_clause.is_empty() { "" } else { &format!("WHERE {}", &where_clause[4..]) }
        );

        let mut query = sqlx::query(&query_sql);

        // Bind filter parameters
        for param in bind_params {
            query = query.bind(param);
        }
        
        query = query.bind(limit as i64).bind(offset as i64);

        let rows = query
            .fetch_all(self.client.get_pool())
            .await
            .map_err(|e| VectorDbError::QueryFailed(format!("Pagination search failed: {}", e)))?;

        let results = rows
            .into_iter()
            .map(|row| self.row_to_search_result_with_score(row, 1.0))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }
    
    pub async fn count_points(&self, filters: Option<SearchFilters>) -> Result<u64, VectorDbError> {
        let (where_clause, bind_params) = self.build_filter_clause(filters);
        
        let count_sql = format!(
            "SELECT COUNT(*) as count FROM {} {}",
            self.client.get_table_name(),
            if where_clause.is_empty() { "" } else { &format!("WHERE {}", &where_clause[4..]) }
        );

        let mut query = sqlx::query(&count_sql);

        // Bind filter parameters
        for param in bind_params {
            query = query.bind(param);
        }

        let row = query
            .fetch_one(self.client.get_pool())
            .await
            .map_err(|e| VectorDbError::QueryFailed(format!("Count failed: {}", e)))?;

        Ok(row.get::<i64, _>("count") as u64)
    }
    
    fn build_filter_clause(&self, filters: Option<SearchFilters>) -> (String, Vec<String>) {
        let mut conditions = Vec::new();
        let mut params = Vec::new();

        if let Some(filters) = filters {
            if let Some(domain) = filters.domain {
                conditions.push("t.domain = ?".to_string());
                params.push(domain);
            }
            
            if let Some(category) = filters.domain_category {
                conditions.push("t.domain_category = ?".to_string());
                params.push(category);
            }
            
            if let Some(tier) = filters.complexity_tier {
                conditions.push("t.complexity_tier = ?".to_string());
                params.push(tier);
            }
            
            if let Some(level) = filters.proficiency_level {
                conditions.push("t.proficiency_level = ?".to_string());
                params.push(level);
            }
            
            if let Some(mode) = filters.mode {
                conditions.push("t.mode = ?".to_string());
                params.push(mode);
            }
            
            if let Some(personality) = filters.personality {
                conditions.push("t.personality = ?".to_string());
                params.push(personality);
            }
            
            if let (Some(min), Some(max)) = (filters.min_complexity, filters.max_complexity) {
                conditions.push("t.complexity_score BETWEEN ? AND ?".to_string());
                params.push(min.to_string());
                params.push(max.to_string());
            }
            
            if let Some(after) = filters.created_after {
                conditions.push("t.created_at > ?".to_string());
                params.push(after.to_string());
            }
            
            if let Some(before) = filters.created_before {
                conditions.push("t.created_at < ?".to_string());
                params.push(before.to_string());
            }
        }

        if conditions.is_empty() {
            ("".to_string(), params)
        } else {
            (format!("AND {}", conditions.join(" AND ")), params)
        }
    }
    
    fn row_to_search_result(&self, row: sqlx::sqlite::SqliteRow) -> Result<SearchResult, VectorDbError> {
        let distance: f32 = row.get("distance");
        let score = 1.0 - distance; // Convert distance to similarity score
        
        self.row_to_search_result_with_score(row, score)
    }
    
    fn row_to_search_result_with_score(&self, row: sqlx::sqlite::SqliteRow, score: f32) -> Result<SearchResult, VectorDbError> {
        let vector_bytes: Vec<u8> = row.get("vector");
        let vector = bytes_to_vector(&vector_bytes);
        
        let payload = SearchPayload {
            raw_input: row.get("raw_input"),
            cleaned_input: row.get("cleaned_input"),
            action: row.get("action"),
            domain: row.get("domain"),
            topic: row.get("topic"),
            mode: row.get("mode"),
            proficiency: row.get("proficiency"),
            personality: row.get("personality"),
            complexity_score: row.get("complexity_score"),
            domain_category: row.get("domain_category"),
            complexity_tier: row.get("complexity_tier"),
            proficiency_level: row.get("proficiency_level"),
            created_at: row.get("created_at"),
        };

        Ok(SearchResult {
            id: row.get("id"),
            score,
            vector,
            payload,
        })
    }
}

// Helper functions for vector conversion
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
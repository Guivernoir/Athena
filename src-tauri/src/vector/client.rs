use sqlx::{SqlitePool, sqlite::SqlitePoolOptions, Row};
use tokio::sync::OnceCell;
use std::sync::Arc;
use crate::VectorDbError;

static SQLITE_CLIENT: OnceCell<Arc<SqliteVecClient>> = OnceCell::const_new();

pub struct SqliteVecClient {
    pool: SqlitePool,
    table_name: String,
    vector_size: usize,
}

impl SqliteVecClient {
    pub async fn new(
        db_path: &str,
        table_name: String,
        vector_size: usize,
    ) -> Result<Self, VectorDbError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite:{}", db_path))
            .await
            .map_err(|e| VectorDbError::ConnectionFailed(e.to_string()))?;
        
        let client = Self {
            pool,
            table_name,
            vector_size,
        };
        
        // Initialize the database schema
        client.ensure_schema().await?;
        
        Ok(client)
    }
    
    pub async fn global(
        db_path: &str,
        table_name: String,
        vector_size: usize,
    ) -> Result<Arc<Self>, VectorDbError> {
        SQLITE_CLIENT.get_or_try_init(|| async {
            let client = Self::new(db_path, table_name, vector_size).await?;
            Ok(Arc::new(client))
        }).await.map(|c| c.clone())
    }
    
    async fn ensure_schema(&self) -> Result<(), VectorDbError> {
        // Load sqlite-vec extension
        sqlx::query("SELECT load_extension('vec0')")
            .execute(&self.pool)
            .await
            .map_err(|e| VectorDbError::ConnectionFailed(format!("Failed to load vec0 extension: {}", e)))?;
        
        // Create the main table with vector column
        let create_table_sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                vector BLOB,
                raw_input TEXT NOT NULL,
                cleaned_input TEXT NOT NULL,
                action TEXT NOT NULL,
                domain TEXT NOT NULL,
                topic TEXT NOT NULL,
                mode TEXT NOT NULL,
                proficiency TEXT NOT NULL,
                personality TEXT NOT NULL,
                word_count INTEGER NOT NULL,
                sentence_count INTEGER NOT NULL,
                token_preview TEXT NOT NULL,
                complexity_score REAL NOT NULL,
                estimated_processing_time INTEGER NOT NULL,
                suggested_response_length TEXT NOT NULL,
                domain_category TEXT NOT NULL,
                complexity_tier TEXT NOT NULL,
                proficiency_level TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                secured_vector BLOB
            )
            "#,
            self.table_name
        );
        
        sqlx::query(&create_table_sql)
            .execute(&self.pool)
            .await
            .map_err(|e| VectorDbError::ConnectionFailed(format!("Failed to create table: {}", e)))?;
        
        // Create vector index for similarity search
        let create_index_sql = format!(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS {}_vec_index USING vec0(
                vector float[{}] distance_metric=cosine
            )
            "#,
            self.table_name, self.vector_size
        );
        
        sqlx::query(&create_index_sql)
            .execute(&self.pool)
            .await
            .map_err(|e| VectorDbError::ConnectionFailed(format!("Failed to create vector index: {}", e)))?;
        
        Ok(())
    }
    
    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }
    
    pub fn get_table_name(&self) -> &str {
        &self.table_name
    }
    
    pub fn get_vector_size(&self) -> usize {
        self.vector_size
    }
    
    pub async fn table_info(&self) -> Result<Vec<String>, VectorDbError> {
        let query = format!("PRAGMA table_info({})", self.table_name);
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| VectorDbError::QueryFailed(format!("Failed to get table info: {}", e)))?;
        
        let columns: Vec<String> = rows
            .iter()
            .map(|row| row.get::<String, _>("name"))
            .collect();
        
        Ok(columns)
    }
    
    pub async fn count_vectors(&self) -> Result<i64, VectorDbError> {
        let query = format!("SELECT COUNT(*) as count FROM {}", self.table_name);
        let row = sqlx::query(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| VectorDbError::QueryFailed(format!("Failed to count vectors: {}", e)))?;
        
        Ok(row.get("count"))
    }
}
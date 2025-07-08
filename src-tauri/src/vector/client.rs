use qdrant_client::{
    client::QdrantClient as QdrantClientInner,
    qdrant::{
        CreateCollection, Distance, VectorParams, VectorsConfig,
        CollectionOperationResponse, GetCollectionInfoRequest
    },
};
use tokio::sync::OnceCell;
use std::sync::Arc;
use crate::VectorDbError;

static QDRANT_CLIENT: OnceCell<Arc<QdrantClient>> = OnceCell::const_new();

pub struct QdrantClient {
    client: QdrantClientInner,
    collection_name: String,
    vector_size: u64,
}

impl QdrantClient {
    pub async fn new(
        url: &str,
        collection_name: String,
        vector_size: u64,
    ) -> Result<Self, VectorDbError> {
        let client = QdrantClientInner::from_url(url)
            .build()
            .map_err(|e| VectorDbError::ConnectionFailed(e.to_string()))?;
        
        let qdrant_client = Self {
            client,
            collection_name,
            vector_size,
        };
        
        // Ensure collection exists
        qdrant_client.ensure_collection().await?;
        
        Ok(qdrant_client)
    }
    
    pub async fn global(
        url: &str,
        collection_name: String,
        vector_size: u64,
    ) -> Result<Arc<Self>, VectorDbError> {
        QDRANT_CLIENT.get_or_try_init(|| async {
            let client = Self::new(url, collection_name, vector_size).await?;
            Ok(Arc::new(client))
        }).await.map(|c| c.clone())
    }
    
    async fn ensure_collection(&self) -> Result<(), VectorDbError> {
        // Check if collection exists
        match self.client.get_collection_info(&self.collection_name).await {
            Ok(_) => {
                // Collection exists, validate vector size
                // Note: In a real implementation, you'd check the actual vector config
                Ok(())
            }
            Err(_) => {
                // Collection doesn't exist, create it
                self.create_collection().await
            }
        }
    }
    
    async fn create_collection(&self) -> Result<(), VectorDbError> {
        let create_collection = CreateCollection {
            collection_name: self.collection_name.clone(),
            vectors_config: Some(VectorsConfig {
                config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                    VectorParams {
                        size: self.vector_size,
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    }
                ))
            }),
            ..Default::default()
        };
        
        self.client
            .create_collection(&create_collection)
            .await
            .map_err(|e| VectorDbError::ConnectionFailed(format!("Failed to create collection: {}", e)))?;
        
        Ok(())
    }
    
    pub fn get_client(&self) -> &QdrantClientInner {
        &self.client
    }
    
    pub fn get_collection_name(&self) -> &str {
        &self.collection_name
    }
    
    pub fn get_vector_size(&self) -> u64 {
        self.vector_size
    }
    
    pub async fn collection_info(&self) -> Result<qdrant_client::qdrant::CollectionInfo, VectorDbError> {
        self.client
            .get_collection_info(&self.collection_name)
            .await
            .map_err(|e| VectorDbError::QueryFailed(format!("Failed to get collection info: {}", e)))
    }
}
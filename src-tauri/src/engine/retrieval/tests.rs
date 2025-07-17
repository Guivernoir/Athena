#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::{cache::CacheSource, memory::MemorySource, web::WebSource};

    #[tokio::test]
    async fn router_flow() {
        let cache  = CacheSource::new(cache::Cache::new(100, 60));
        let memory = MemorySource::new().unwrap();
        let web    = WebSource::new();

        let router = Router::new(cache, memory, web);
        let query  = SearchQuery::new("rust lang", 3);

        let results = router.search(&query).await.unwrap();
        assert!(!results.is_empty());
    }
}
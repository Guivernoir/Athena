#[cfg(test)]
mod tests {
    use super::*;
    use crate::{output::builder::PromptBuilder, retrieval::router::Router};
    use engine::retrieval::sources::{cache::CacheSource, memory::MemorySource, web::WebSource};

    #[tokio::test]
    async fn end_to_end_turn() {
        let cache = CacheSource::new(cache::Cache::new(10, 60));
        let memory = MemorySource::new().unwrap();
        let web = WebSource::new();

        let router = Router::new(cache, memory, web);
        let builder = PromptBuilder::new(router.clone(), "test");
        let mut orch = Orchestrator::new(router, builder, "test");

        let reply = orch.turn("hello").await.unwrap();
        assert!(!reply.is_empty());
    }
}
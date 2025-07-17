//! High-level orchestrator: query → retrieval → formatting.

use crate::{
    formatter::format_results, injector::inject, schema::PromptPayload,
    templates::persona_system,
};
use engine::retrieval::{query::SearchQuery, router::Router};

pub struct PromptBuilder {
    router: Router,
    persona: String,
}

impl PromptBuilder {
    pub fn new(router: Router, persona: &str) -> Self {
        Self {
            router,
            persona: persona.to_string(),
        }
    }

    /// Build the final prompt for the LLM.
    pub async fn build(&self, user_text: &str, top_k: usize) -> anyhow::Result<String> {
        let query = SearchQuery::new(user_text, top_k);
        let all_results = self.router.search(&query).await?;

        // Split by source
        let (memory, rest): (Vec<_>, Vec<_>) = all_results
            .into_iter()
            .partition(|r| r.source == "memory");
        let (cache, web): (Vec<_>, Vec<_>) = rest.into_iter().partition(|r| r.source == "cache");

        let system = persona_system(&self.persona);
        let payload = format_results(system, memory, cache, web);
        Ok(inject(payload))
    }
}
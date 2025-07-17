//! Optional DuckDuckGo instant-answer search as fallback.

use super::Source;
use super::super::result::SearchResult;
use async_trait::async_trait;
use reqwest::Client;

pub struct WebSource {
    client: Client,
}

impl WebSource {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Source for WebSource {
    fn name(&self) -> &'static str { "web" }

    async fn search(&self, query: &str, top_k: usize) -> anyhow::Result<Vec<SearchResult>> {
        // DuckDuckGo instant answer API — no key required
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
            urlencoding::encode(query)
        );
        let resp: serde_json::Value = self.client.get(&url).send().await?.json().await?;
        let mut results = Vec::new();

        if let Some(related) = resp["RelatedTopics"].as_array() {
            for item in related.iter().take(top_k) {
                if let Some(text) = item["Text"].as_str() {
                    results.push(SearchResult {
                        score: 0.5, // placeholder
                        content: text.to_string(),
                        source: self.name(),
                    });
                }
            }
        }
        Ok(results)
    }
}
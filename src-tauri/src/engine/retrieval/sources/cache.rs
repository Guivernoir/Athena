//! Cache-based search (in-memory staging area).

use super::Source;
use super::super::result::SearchResult;
use async_trait::async_trait;
use cache::Cache; // thin wrapper around cache::Cache

pub struct CacheSource {
    cache: Cache,
}

impl CacheSource {
    pub fn new(cache: Cache) -> Self {
        Self { cache }
    }
}

#[async_trait]
impl Source for CacheSource {
    fn name(&self) -> &'static str { "cache" }

    async fn search(&self, query: &str, top_k: usize) -> anyhow::Result<Vec<SearchResult>> {
        // TODO: naive linear scan for now; later add ANN on cache vectors
        let guard = self.cache.inner.lock().unwrap();
        let mut hits = Vec::new();
        for entry in &guard.buffer {
            let input_vec  = embedding::Engine::new().embed(&entry.input.content);
            let output_vec = embedding::Engine::new().embed(&entry.output.content);
            let q_vec      = embedding::Engine::new().embed(query);

            let input_score  = cosine(&q_vec, &input_vec);
            let output_score = cosine(&q_vec, &output_vec);

            if input_score > 0.0 {
                hits.push(SearchResult {
                    score: input_score,
                    content: entry.input.content.clone(),
                    source: self.name(),
                });
            }
            if output_score > 0.0 {
                hits.push(SearchResult {
                    score: output_score,
                    content: entry.output.content.clone(),
                    source: self.name(),
                });
            }
        }
        hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        Ok(hits.into_iter().take(top_k).collect())
    }
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot = a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>();
    let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (norm_a * norm_b).max(f32::EPSILON)
}
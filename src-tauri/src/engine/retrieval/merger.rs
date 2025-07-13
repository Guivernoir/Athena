use std::collections::HashMap;
use tracing::{info, debug};
use seahash::hash;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MergeError {
    #[error("No results to merge")]
    EmptyResults,
    #[error("Score normalization failed")]
    NormalizationFailed,
}

#[derive(Debug, Clone)]
pub struct KnowledgeItem {
    pub content: String,
    pub source: String, // "db" or "web"
    pub score: f32,
    pub metadata: Option<String>,
}

pub struct ResultMerger;

impl ResultMerger {
    #[instrument(skip_all)]
    pub fn merge(
        &self,
        db_results: Vec<(String, f32)>,
        web_results: Vec<super::ws::SearchResult>,
    ) -> Result<Vec<KnowledgeItem>, MergeError> {
        if db_results.is_empty() && web_results.is_empty() {
            return Err(MergeError::EmptyResults);
        }

        // Create content hash map for deduplication
        let mut unique_items = HashMap::new();

        // Process database results
        for (content, score) in db_results {
            let content_hash = hash(content.as_bytes());
            unique_items.entry(content_hash)
                .and_modify(|e: &mut KnowledgeItem| {
                    // If same content exists, keep higher score
                    if score > e.score {
                        e.score = score;
                        e.source = "db".to_string();
                    }
                })
                .or_insert_with(|| KnowledgeItem {
                    content,
                    source: "db".to_string(),
                    score,
                    metadata: None,
                });
        }

        // Process web results (with lower base score)
        for result in web_results {
            let content = format!("{}: {}", result.title, result.snippet);
            let content_hash = hash(content.as_bytes());
            let adjusted_score = result.relevance_score * 0.8; // Web results slightly penalized

            unique_items.entry(content_hash)
                .and_modify(|e: &mut KnowledgeItem| {
                    if adjusted_score > e.score {
                        e.score = adjusted_score;
                        e.source = "web".to_string();
                    }
                })
                .or_insert_with(|| KnowledgeItem {
                    content,
                    source: "web".to_string(),
                    score: adjusted_score,
                    metadata: Some(result.link),
                });
        }

        let mut final_results: Vec<_> = unique_items.into_values().collect();
        
        // Normalize scores
        if let Some(max_score) = final_results.iter().map(|i| i.score).max_by(|a, b| a.partial_cmp(b).unwrap_or(&1.0)) {
            if max_score > 0.0 {
                for item in &mut final_results {
                    item.score /= max_score;
                }
            }
        }

        // Sort by descending score
        final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        debug!("Merged {} knowledge items", final_results.len());
        Ok(final_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_results() {
        let merger = ResultMerger;
        let db_results = vec![
            ("Database fact 1".to_string(), 0.9),
            ("Database fact 2".to_string(), 0.7),
        ];
        
        let web_results = vec![
            super::ws::SearchResult {
                title: "Web result 1".to_string(),
                link: "https://example.com/1".to_string(),
                snippet: "Web snippet 1".to_string(),
                relevance_score: 0.8,
            },
            super::ws::SearchResult {
                title: "Database fact 1".to_string(), // Should be deduplicated
                link: "https://example.com/dup".to_string(),
                snippet: "Same as DB fact".to_string(),
                relevance_score: 0.95,
            },
        ];

        let merged = merger.merge(db_results, web_results).unwrap();
        assert_eq!(merged.len(), 3); // 2 unique from DB + 1 unique from web
        assert!(merged[0].score >= merged[1].score); // Sorted by score
    }
}
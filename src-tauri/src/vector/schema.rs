use serde::{Serialize, Deserialize};
use qdrant_client::qdrant::{PointStruct, Value, Vectors};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub payload: SearchPayload,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPayload {
    pub raw_input: String,
    pub cleaned_input: String,
    pub action: String,
    pub domain: String,
    pub topic: String,
    pub mode: String,
    pub proficiency: String,
    pub personality: String,
    pub complexity_score: f32,
    pub domain_category: String,
    pub complexity_tier: String,
    pub proficiency_level: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub domain: Option<String>,
    pub domain_category: Option<String>,
    pub complexity_tier: Option<String>,
    pub proficiency_level: Option<String>,
    pub mode: Option<String>,
    pub personality: Option<String>,
    pub min_complexity: Option<f32>,
    pub max_complexity: Option<f32>,
    pub created_after: Option<i64>,
    pub created_before: Option<i64>,
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            domain: None,
            domain_category: None,
            complexity_tier: None,
            proficiency_level: None,
            mode: None,
            personality: None,
            min_complexity: None,
            max_complexity: None,
            created_after: None,
            created_before: None,
        }
    }
}

impl SearchFilters {
    pub fn new() -> Self {
        Default::default()
    }
    
    pub fn domain(mut self, domain: String) -> Self {
        self.domain = Some(domain);
        self
    }
    
    pub fn domain_category(mut self, category: String) -> Self {
        self.domain_category = Some(category);
        self
    }
    
    pub fn complexity_tier(mut self, tier: String) -> Self {
        self.complexity_tier = Some(tier);
        self
    }
    
    pub fn proficiency_level(mut self, level: String) -> Self {
        self.proficiency_level = Some(level);
        self
    }
    
    pub fn complexity_range(mut self, min: f32, max: f32) -> Self {
        self.min_complexity = Some(min);
        self.max_complexity = Some(max);
        self
    }
    
    pub fn to_qdrant_filter(&self) -> Option<qdrant_client::qdrant::Filter> {
        let mut conditions = Vec::new();
        
        if let Some(domain) = &self.domain {
            conditions.push(qdrant_client::qdrant::Condition {
                condition_one_of: Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(
                    qdrant_client::qdrant::FieldCondition {
                        key: "domain".to_string(),
                        r#match: Some(qdrant_client::qdrant::Match {
                            match_value: Some(qdrant_client::qdrant::r#match::MatchValue::Text(domain.clone())),
                        }),
                        ..Default::default()
                    }
                )),
            });
        }
        
        if let Some(category) = &self.domain_category {
            conditions.push(qdrant_client::qdrant::Condition {
                condition_one_of: Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(
                    qdrant_client::qdrant::FieldCondition {
                        key: "domain_category".to_string(),
                        r#match: Some(qdrant_client::qdrant::Match {
                            match_value: Some(qdrant_client::qdrant::r#match::MatchValue::Text(category.clone())),
                        }),
                        ..Default::default()
                    }
                )),
            });
        }
        
        if let Some(tier) = &self.complexity_tier {
            conditions.push(qdrant_client::qdrant::Condition {
                condition_one_of: Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(
                    qdrant_client::qdrant::FieldCondition {
                        key: "complexity_tier".to_string(),
                        r#match: Some(qdrant_client::qdrant::Match {
                            match_value: Some(qdrant_client::qdrant::r#match::MatchValue::Text(tier.clone())),
                        }),
                        ..Default::default()
                    }
                )),
            });
        }
        
        if let Some(level) = &self.proficiency_level {
            conditions.push(qdrant_client::qdrant::Condition {
                condition_one_of: Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(
                    qdrant_client::qdrant::FieldCondition {
                        key: "proficiency_level".to_string(),
                        r#match: Some(qdrant_client::qdrant::Match {
                            match_value: Some(qdrant_client::qdrant::r#match::MatchValue::Text(level.clone())),
                        }),
                        ..Default::default()
                    }
                )),
            });
        }
        
        if let (Some(min), Some(max)) = (self.min_complexity, self.max_complexity) {
            conditions.push(qdrant_client::qdrant::Condition {
                condition_one_of: Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(
                    qdrant_client::qdrant::FieldCondition {
                        key: "complexity_score".to_string(),
                        range: Some(qdrant_client::qdrant::Range {
                            gte: Some(min as f64),
                            lte: Some(max as f64),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }
                )),
            });
        }
        
        if conditions.is_empty() {
            None
        } else {
            Some(qdrant_client::qdrant::Filter {
                must: conditions,
                ..Default::default()
            })
        }
    }
}

// Conversion utilities
pub fn payload_to_qdrant_map(payload: &crate::formatter::QdrantPayload) -> HashMap<String, Value> {
    let mut map = HashMap::new();
    
    map.insert("raw_input".to_string(), Value::from(payload.raw_input.clone()));
    map.insert("cleaned_input".to_string(), Value::from(payload.cleaned_input.clone()));
    map.insert("action".to_string(), Value::from(payload.action.clone()));
    map.insert("domain".to_string(), Value::from(payload.domain.clone()));
    map.insert("topic".to_string(), Value::from(payload.topic.clone()));
    map.insert("mode".to_string(), Value::from(payload.mode.clone()));
    map.insert("proficiency".to_string(), Value::from(payload.proficiency.clone()));
    map.insert("personality".to_string(), Value::from(payload.personality.clone()));
    map.insert("word_count".to_string(), Value::from(payload.word_count));
    map.insert("sentence_count".to_string(), Value::from(payload.sentence_count));
    map.insert("token_preview".to_string(), Value::from(payload.token_preview.clone()));
    map.insert("complexity_score".to_string(), Value::from(payload.complexity_score as f64));
    map.insert("estimated_processing_time".to_string(), Value::from(payload.estimated_processing_time));
    map.insert("suggested_response_length".to_string(), Value::from(payload.suggested_response_length.clone()));
    map.insert("domain_category".to_string(), Value::from(payload.domain_category.clone()));
    map.insert("complexity_tier".to_string(), Value::from(payload.complexity_tier.clone()));
    map.insert("proficiency_level".to_string(), Value::from(payload.proficiency_level.clone()));
    map.insert("created_at".to_string(), Value::from(payload.created_at));
    map.insert("updated_at".to_string(), Value::from(payload.updated_at));
    
    map
}

pub fn qdrant_point_to_point_struct(point: &crate::formatter::QdrantPoint) -> PointStruct {
    PointStruct {
        id: Some(point.id.clone().into()),
        vectors: Some(Vectors {
            vectors_options: Some(qdrant_client::qdrant::vectors::VectorsOptions::Vector(
                qdrant_client::qdrant::Vector {
                    data: point.vector.clone(),
                    ..Default::default()
                }
            )),
        }),
        payload: payload_to_qdrant_map(&point.payload),
    }
}
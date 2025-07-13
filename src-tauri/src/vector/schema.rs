use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub vector: Vec<f32>,
    pub payload: SearchPayload,
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
    
    pub fn mode(mut self, mode: String) -> Self {
        self.mode = Some(mode);
        self
    }
    
    pub fn personality(mut self, personality: String) -> Self {
        self.personality = Some(personality);
        self
    }
    
    pub fn created_after(mut self, timestamp: i64) -> Self {
        self.created_after = Some(timestamp);
        self
    }
    
    pub fn created_before(mut self, timestamp: i64) -> Self {
        self.created_before = Some(timestamp);
        self
    }
}
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::collections::HashMap;
use uuid::Uuid;
use crate::context::Context;
use crate::tokenizer::TokenInfo;
use crate::router::{Mode, Proficiency, Personality};

#[derive(Error, Debug)]
pub enum FormatterError {
    #[error("Failed to serialize formatted input: {0}")]
    SerializationFailed(String),
    #[error("Failed to generate embeddings: {0}")]
    EmbeddingFailed(String),
    #[error("Invalid payload structure: {0}")]
    InvalidPayload(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QdrantPoint {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: QdrantPayload,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QdrantPayload {
    // Core content
    pub raw_input: String,
    pub cleaned_input: String,
    pub action: String,
    pub domain: String,
    pub topic: String,
    
    // Processing metadata
    pub mode: String,
    pub proficiency: String,
    pub personality: String,
    
    // Token information
    pub word_count: i64,
    pub sentence_count: i64,
    pub token_preview: String, // First 10 tokens for quick reference
    
    // Calculated metrics
    pub complexity_score: f32,
    pub estimated_processing_time: i64,
    pub suggested_response_length: String,
    
    // Indexing fields for efficient querying
    pub domain_category: String,
    pub complexity_tier: String,
    pub proficiency_level: String,
    
    // Timestamps
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormattedInput {
    pub qdrant_point: QdrantPoint,
    pub context: Context,
    pub tokens: TokenInfo,
    pub mode: Mode,
    pub proficiency: Proficiency,
    pub personality: Personality,
    pub metadata: InputMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputMetadata {
    pub complexity_score: f32,
    pub estimated_processing_time: u32,
    pub suggested_response_length: ResponseLength,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ResponseLength {
    Brief,
    Standard,
    Detailed,
    Comprehensive,
}

impl FormattedInput {
    pub fn new(
        context: Context,
        tokens: TokenInfo,
        mode: Mode,
        proficiency: Proficiency,
        personality: Personality,
        vector: Vec<f32>,
    ) -> Result<Self, FormatterError> {
        let metadata = Self::calculate_metadata(&context, &tokens, &proficiency);
        let qdrant_point = Self::build_qdrant_point(&context, &tokens, &mode, &proficiency, &personality, &metadata, vector)?;
        
        Ok(Self {
            qdrant_point,
            context,
            tokens,
            mode,
            proficiency,
            personality,
            metadata,
        })
    }
    
    fn build_qdrant_point(
        context: &Context,
        tokens: &TokenInfo,
        mode: &Mode,
        proficiency: &Proficiency,
        personality: &Personality,
        metadata: &InputMetadata,
        vector: Vec<f32>,
    ) -> Result<QdrantPoint, FormatterError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let token_preview = tokens.tokens
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");
        
        let payload = QdrantPayload {
            raw_input: context.raw_input.clone(),
            cleaned_input: context.raw_input.clone(), // Assuming cleaned version
            action: context.action.clone(),
            domain: context.domain.clone(),
            topic: context.topic.clone(),
            
            mode: format!("{:?}", mode),
            proficiency: format!("{:?}", proficiency),
            personality: format!("{:?}", personality),
            
            word_count: tokens.word_count as i64,
            sentence_count: tokens.sentence_count as i64,
            token_preview,
            
            complexity_score: metadata.complexity_score,
            estimated_processing_time: metadata.estimated_processing_time as i64,
            suggested_response_length: format!("{:?}", metadata.suggested_response_length),
            
            domain_category: Self::categorize_domain(&context.domain),
            complexity_tier: Self::categorize_complexity(metadata.complexity_score),
            proficiency_level: Self::map_proficiency_level(proficiency),
            
            created_at: now,
            updated_at: now,
        };
        
        Ok(QdrantPoint {
            id: Uuid::new_v4().to_string(),
            vector,
            payload,
        })
    }
    
    fn categorize_domain(domain: &str) -> String {
        match domain.to_lowercase().as_str() {
            // Programming languages
            "rust" | "elixir" | "julia" | "c++" | "c" | "sql" | "go" | "typescript" | "python" | "javascript" | "scala" | "cobol" => "programming_language".to_string(),
            "html" | "css" => "markup_styling".to_string(),
            "tauri" => "framework".to_string(),

            // Natural languages
            "english" | "french" | "portuguese" | "russian" | "spanish" | "german" | "italian" | "japanese" | "mandarin" | "arabic" | "korean" | "hebrew" => "natural_language".to_string(),

            // Language mechanics and theory
            "grammar" | "syntax" | "pronunciation" | "vocabulary" | "translation" | "linguistics" | "phonetics" | "semantics" | "pragmatics" | "sociolinguistics" | "discourse_analysis" | "stylistics" | "morphology" | "phonology" | "syntax_tree" | "language_acquisition" | "general_linguistics" => "language_mechanics".to_string(),

            // Programming concepts
            "algorithms" | "data_structures" | "design_patterns" | "debugging" | "testing" | "optimization" | "concurrency" | "distributed_systems" | "functional_programming" | "object_oriented_programming" | "reactive_programming" | "metaprogramming" | "type_systems" | "memory_management" | "networking" | "security" | "performance_tuning" | "software_architecture" | "agile_development" | "general_programming" => "programming_concepts".to_string(),

            _ => "general".to_string(),
        }
    }
    
    fn categorize_complexity(score: f32) -> String {
        match score {
            s if s < 1.0 => "simple".to_string(),
            s if s < 2.5 => "moderate".to_string(),
            s if s < 4.0 => "complex".to_string(),
            _ => "advanced".to_string(),
        }
    }
    
    fn map_proficiency_level(proficiency: &Proficiency) -> String {
        match proficiency {
            Proficiency::Beginner => "novice".to_string(),
            Proficiency::Intermediate => "intermediate".to_string(),
            Proficiency::Advanced => "advanced".to_string(),
            Proficiency::Expert => "expert".to_string(),
        }
    }
    
    fn calculate_metadata(
        context: &Context,
        tokens: &TokenInfo,
        proficiency: &Proficiency,
    ) -> InputMetadata {
        let complexity_score = Self::calculate_complexity(context, tokens);
        let estimated_processing_time = Self::estimate_processing_time(&complexity_score, tokens);
        let suggested_response_length = Self::suggest_response_length(proficiency, &complexity_score);
        
        InputMetadata {
            complexity_score,
            estimated_processing_time,
            suggested_response_length,
        }
    }
    
    fn calculate_complexity(context: &Context, tokens: &TokenInfo) -> f32 {
        let base_score = tokens.word_count as f32 / 100.0;
        let domain_multiplier = match context.domain.to_lowercase().as_str() {
            // Programming languages
            "rust" => 2.0,  // Borrow checker, lifetimes, ownership
            "elixir" => 1.8, // Actor model, functional patterns
            "julia" => 1.7,  // Multiple dispatch, metaprogramming
            "c++" => 2.5,  // Pointers, templates, manual memory management
            "c" => 2.2,   // Low-level, manual memory management
            "sql" => 1.5, // Query complexity, joins
            "go" => 1.4,  // Concurrency model, garbage collection
            "typescript" => 1.6, // Static typing, complex type system
            "python" => 1.5, // Object-oriented, but readable
            "javascript" => 1.4, // Prototypal inheritance, async patterns
            "tauri" => 1.6, // Cross-platform complexity
            "scala" => 1.8, // Functional programming, type inference
            "cobol" => 1.9, // Legacy systems, verbose syntax
            "html" => 1.1, // Semantic complexity
            "css" => 1.3,  // Layout models, specificity wars
            
            // Natural language complexity varies by task
            "english" => 1.2, // Irregular verbs, idiomatic expressions
            "french" => 1.4,  // Gendered nouns, complex grammar
            "portuguese" => 1.4, // Verb conjugations, regional variations
            "russian" => 1.5, // Cyrillic script, complex grammar
            "spanish" => 1.3, // Verb tenses, gendered nouns
            "german" => 1.6, // Compound nouns, case system
            "italian" => 1.3, // Verb conjugations, gendered nouns
            "japanese" => 2.0, // Kanji complexity, honorifics
            "mandarin" => 2.1, // Tonal language, characters
            "arabic" => 2.2, // Script complexity, dialects
            "korean" => 1.8, // Hangul script, honorifics
            "hebrew" => 1.7, // Script complexity, vowel omission
            
            // Language mechanics and theory
            "grammar" | "syntax" => 1.5, // Structural analysis
            "pronunciation" => 1.3, // Phonetic complexity
            "vocabulary" => 1.1, // Memorization-based
            "translation" => 1.7, // Cultural context, nuance
            "linguistics" => 1.8, // Theoretical depth
            "phonetics" => 1.6, // Sound systems, articulatory complexity
            "semantics" => 1.5, // Meaning analysis, context sensitivity
            "pragmatics" => 1.4, // Contextual language use
            "sociolinguistics" => 1.9, // Language in social context
            "discourse_analysis" => 1.8, // Text structure, coherence
            "stylistics" => 1.7, // Literary style, rhetorical devices
            "morphology" => 1.6, // Word structure, inflection
            "phonology" => 1.5, // Sound patterns, phonemes
            "syntax_tree" => 1.4, // Hierarchical structure of sentences
            "language_acquisition" => 1.8, // Learning processes, stages
            "general_linguistics" => 1.5, // Broad theoretical concepts
            
            // Programming concepts
            "algorithms" => 1.9, // Mathematical reasoning
            "data_structures" => 1.6, // Abstract thinking
            "design_patterns" => 1.7, // Architectural concepts
            "debugging" => 1.4, // Problem-solving skills
            "testing" => 1.3, // Methodical approach
            "optimization" => 1.8, // Performance analysis
            "concurrency" => 2.0, // Parallel processing complexity
            "distributed_systems" => 2.1, // Networked systems, fault tolerance
            "functional_programming" => 1.9, // Immutable data, higher-order functions
            "object_oriented_programming" => 1.5, // Encapsulation, inheritance
            "reactive_programming" => 1.8, // Event-driven, asynchronous patterns
            "metaprogramming" => 2.0, // Code that writes code
            "type_systems" => 1.7, // Static vs dynamic typing, type inference
            "memory_management" => 2.2, // Manual vs automatic, garbage collection
            "networking" => 1.6, // Protocols, data transmission
            "security" => 1.9, // Vulnerabilities, encryption
            "performance_tuning" => 1.8, // Profiling, optimization techniques
            "software_architecture" => 1.7, // System design, modularity
            "agile_development" => 1.5, // Iterative processes, team dynamics
            "general_programming" => 1.4, // Basic coding skills
            
            _ => 1.0,
        };
        
        (base_score * domain_multiplier).min(5.0)
    }
    
    fn estimate_processing_time(complexity: &f32, tokens: &TokenInfo) -> u32 {
        let base_time = (tokens.word_count as f32 * 0.1) as u32;
        let complexity_factor = (*complexity * 10.0) as u32;
        
        (base_time + complexity_factor).max(5).min(300)
    }
    
    fn suggest_response_length(proficiency: &Proficiency, complexity: &f32) -> ResponseLength {
        match (proficiency, *complexity) {
            (Proficiency::Beginner, _) => ResponseLength::Detailed,
            (Proficiency::Intermediate, c) if c > 3.0 => ResponseLength::Comprehensive,
            (Proficiency::Intermediate, _) => ResponseLength::Standard,
            (Proficiency::Advanced, c) if c > 4.0 => ResponseLength::Comprehensive,
            (Proficiency::Advanced, _) => ResponseLength::Standard,
            (Proficiency::Expert, _) => ResponseLength::Brief,
        }
    }
    
    pub fn to_qdrant_json(&self) -> Result<String, FormatterError> {
        serde_json::to_string_pretty(&self.qdrant_point)
            .map_err(|e| FormatterError::SerializationFailed(e.to_string()))
    }
    
    pub fn to_json(&self) -> Result<String, FormatterError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| FormatterError::SerializationFailed(e.to_string()))
    }
    
    // Utility methods for Qdrant operations
    pub fn get_search_filters(&self) -> HashMap<String, serde_json::Value> {
        let mut filters = HashMap::new();
        filters.insert("domain".to_string(), serde_json::Value::String(self.qdrant_point.payload.domain.clone()));
        filters.insert("domain_category".to_string(), serde_json::Value::String(self.qdrant_point.payload.domain_category.clone()));
        filters.insert("complexity_tier".to_string(), serde_json::Value::String(self.qdrant_point.payload.complexity_tier.clone()));
        filters.insert("proficiency_level".to_string(), serde_json::Value::String(self.qdrant_point.payload.proficiency_level.clone()));
        filters
    }
    
    pub fn get_id(&self) -> &str {
        &self.qdrant_point.id
    }
    
    pub fn get_vector(&self) -> &Vec<f32> {
        &self.qdrant_point.vector
    }
    
    pub fn get_payload(&self) -> &QdrantPayload {
        &self.qdrant_point.payload
    }
}
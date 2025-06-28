use crate::llama::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub model_path: String,
    pub number_of_queries: i32,
    pub max_research_loops: u32,
    pub generation_config: GenerationConfig,
    pub security: SecurityConfig,
    pub performance: PerformanceConfig,
    pub localization: LocalizationConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SecurityConfig {
    pub max_input_length: usize,
    pub enable_injection_detection: bool,
    pub redact_sensitive_data: bool,
    pub dangerous_patterns: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PerformanceConfig {
    pub cache_size: usize,
    pub cache_ttl_seconds: u64,
    pub max_concurrent_requests: usize,
    pub request_timeout_seconds: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalizationConfig {
    pub default_locale: String,
    pub supported_locales: Vec<String>,
    pub enable_auto_detection: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model_path: "model.bin".to_string(),
            number_of_queries: 1,
            max_research_loops: 3,
            generation_config: GenerationConfig::default(),
            security: SecurityConfig::default(),
            performance: PerformanceConfig::default(),
            localization: LocalizationConfig::default(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_input_length: 10000,
            enable_injection_detection: true,
            redact_sensitive_data: true,
            dangerous_patterns: vec![
                "\\x00".to_string(),
                "SELECT.*FROM".to_string(),
                "DROP.*TABLE".to_string(),
                "SCRIPT.*SRC".to_string(),
            ],
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            cache_size: 1000,
            cache_ttl_seconds: 3600,
            max_concurrent_requests: 10,
            request_timeout_seconds: 30,
        }
    }
}

impl Default for LocalizationConfig {
    fn default() -> Self {
        Self {
            default_locale: "en-US".to_string(),
            supported_locales: vec![
                "en-US".to_string(),
                "pt-BR".to_string(),
                "fr-FR".to_string(),
            ],
            enable_auto_detection: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub mode: u32,
    pub proficiency: u32,
    pub input: String,
    #[serde(default)]
    pub locale: Option<String>,
    #[serde(default)]
    pub context: Option<ConversationContext>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConversationContext {
    pub session_id: String,
    pub previous_interactions: Vec<String>,
    pub user_preferences: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum Mode {
    Assistant,
    Tutor,
    WebSearch,
    CodeReview,
    Translation,
}

#[derive(Debug, Clone)]
pub enum Proficiency {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, thiserror::Error)]
pub enum InputProcessingError {
    #[error("Input validation failed: {reason}")]
    ValidationError { reason: String },

    #[error("Security violation detected: {violation_type}")]
    SecurityViolation { violation_type: String },

    #[error("Parsing failed after {attempts} attempts: {last_error}")]
    ParsingError { attempts: u32, last_error: String },

    #[error("LLM engine error: {source}")]
    LLMError { source: LLMError },

    #[error("Rate limit exceeded: {limit} requests per {window_seconds}s")]
    RateLimitError { limit: u32, window_seconds: u32 },

    #[error("Timeout after {seconds} seconds")]
    TimeoutError { seconds: u64 },

    #[error("Cache error: {reason}")]
    CacheError { reason: String },

    #[error("Localization error: {locale} not supported")]
    LocalizationError { locale: String },
}

pub type ProcessingResult<T> = Result<T, InputProcessingError>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParsedInput {
    pub action: String,
    pub domain: String,
    pub topic: String,
    pub mode: Mode,
    pub proficiency: Proficiency,
    pub original_input: String,
    pub confidence_score: f32,
    pub parsing_method: ParsingMethod,
    pub detected_locale: String,
    pub security_flags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub processing_time_ms: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ParsingMethod {
    LLMPrimary,
    LLMFallback,
    RuleBased,
    HybridApproach,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessedRequest {
    pub mode: Mode,
    pub proficiency: Proficiency,
    pub original_input: String,
    pub processed_input: String,
    pub config: Config,
    pub processing_metadata: ProcessingMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessingMetadata {
    pub request_id: String,
    pub processing_time_ms: u64,
    pub cache_hit: bool,
    pub security_checks_passed: bool,
    pub confidence_score: f32,
}

pub struct InputValidator {
    security_config: SecurityConfig,
}

impl InputValidator {
    pub fn new(security_config: SecurityConfig) -> Self {
        Self { security_config }
    }

    #[instrument(skip(self))]
    pub fn validate_and_sanitize(&self, input: &str) -> ProcessingResult<String> {
        let start = Instant::now();

        // Basic validation
        if input.trim().is_empty() {
            return Err(InputProcessingError::ValidationError {
                reason: "Input cannot be empty".to_string(),
            });
        }

        if input.len() > self.security_config.max_input_length {
            return Err(InputProcessingError::ValidationError {
                reason: format!(
                    "Input too long: {} > {} characters",
                    input.len(),
                    self.security_config.max_input_length
                ),
            });
        }

        // Security checks
        if self.security_config.enable_injection_detection {
            self.detect_injection_attempts(input)?;
        }

        // Sanitization
        let sanitized = self.sanitize_input(input);

        debug!(
            processing_time_ms = start.elapsed().as_millis() as u64,
            original_length = input.len(),
            sanitized_length = sanitized.len(),
            "Input validation completed"
        );

        Ok(sanitized)
    }

    fn detect_injection_attempts(&self, input: &str) -> ProcessingResult<()> {
        let input_lower = input.to_lowercase();

        for pattern in &self.security_config.dangerous_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if regex.is_match(&input_lower) {
                    warn!(pattern = pattern, "Potential injection attempt detected");
                    return Err(InputProcessingError::SecurityViolation {
                        violation_type: format!("Pattern match: {}", pattern),
                    });
                }
            }
        }

        // Check for suspicious Unicode characters
        if input
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t')
        {
            return Err(InputProcessingError::SecurityViolation {
                violation_type: "Suspicious control characters detected".to_string(),
            });
        }

        Ok(())
    }

    fn sanitize_input(&self, input: &str) -> String {
        // Normalize Unicode
        let normalized =
            unicode_normalization::UnicodeNormalization::nfc(input).collect::<String>();

        // Remove or replace dangerous characters
        normalized
            .chars()
            .filter(|&c| !c.is_control() || c == '\n' || c == '\r' || c == '\t')
            .collect::<String>()
            .trim()
            .to_string()
    }

    fn redact_sensitive_data(&self, input: &str) -> String {
        if !self.security_config.redact_sensitive_data {
            return input.to_string();
        }

        let mut result = input.to_string();

        // Redact potential email addresses
        if let Ok(email_regex) =
            regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")
        {
            result = email_regex
                .replace_all(&result, "[EMAIL_REDACTED]")
                .to_string();
        }

        // Redact potential phone numbers
        if let Ok(phone_regex) = regex::Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b") {
            result = phone_regex
                .replace_all(&result, "[PHONE_REDACTED]")
                .to_string();
        }

        // Redact potential credit card numbers
        if let Ok(cc_regex) = regex::Regex::new(r"\b\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b") {
            result = cc_regex.replace_all(&result, "[CC_REDACTED]").to_string();
        }

        result
    }
}

pub struct LocalizationManager {
    config: LocalizationConfig,
}

impl LocalizationManager {
    pub fn new(config: LocalizationConfig) -> Self {
        Self { config }
    }

    #[instrument(skip(self))]
    pub fn detect_locale(&self, input: &str) -> String {
        if !self.config.enable_auto_detection {
            return self.config.default_locale.clone();
        }

        // Simple heuristic-based language detection
        // In production, you'd use a proper language detection library
        let input_lower = input.to_lowercase();

        if input_lower.contains("é") || input_lower.contains("ç") || input_lower.contains("ã") {
            if self.config.supported_locales.contains(&"pt-BR".to_string()) {
                return "pt-BR".to_string();
            }
        }

        if input_lower.contains("é") || input_lower.contains("è") || input_lower.contains("ê") {
            if self.config.supported_locales.contains(&"fr-FR".to_string()) {
                return "fr-FR".to_string();
            }
        }

        self.config.default_locale.clone()
    }

    pub fn get_error_message(&self, error: &InputProcessingError, locale: &str) -> String {
        match locale {
            "pt-BR" => self.get_portuguese_error_message(error),
            "fr-FR" => self.get_french_error_message(error),
            _ => format!("{}", error),
        }
    }

    fn get_portuguese_error_message(&self, error: &InputProcessingError) -> String {
        match error {
            InputProcessingError::ValidationError { .. } => {
                "Erro de validação na entrada de dados".to_string()
            }
            InputProcessingError::SecurityViolation { .. } => {
                "Violação de segurança detectada".to_string()
            }
            _ => format!("{}", error),
        }
    }

    fn get_french_error_message(&self, error: &InputProcessingError) -> String {
        match error {
            InputProcessingError::ValidationError { .. } => {
                "Erreur de validation des données d'entrée".to_string()
            }
            InputProcessingError::SecurityViolation { .. } => {
                "Violation de sécurité détectée".to_string()
            }
            _ => format!("{}", error),
        }
    }
}

#[derive(Clone)]
pub struct CacheEntry {
    pub value: ParsedInput,
    pub created_at: Instant,
    pub ttl: Duration,
}

impl CacheEntry {
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

pub struct InputCache {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    ttl: Duration,
    max_size: usize,
}

impl InputCache {
    pub fn new(max_size: usize, ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_seconds),
            max_size,
        }
    }

    #[instrument(skip(self))]
    pub async fn get(&self, key: &str) -> Option<ParsedInput> {
        let cache = self.cache.read().await;

        if let Some(entry) = cache.get(key) {
            if !entry.is_expired() {
                debug!(key = key, "Cache hit");
                return Some(entry.value.clone());
            }
        }

        debug!(key = key, "Cache miss");
        None
    }

    #[instrument(skip(self, value))]
    pub async fn set(&self, key: String, value: ParsedInput) -> ProcessingResult<()> {
        let mut cache = self.cache.write().await;

        // Evict expired entries and enforce size limit
        if cache.len() >= self.max_size {
            self.evict_expired_entries(&mut cache).await;

            if cache.len() >= self.max_size {
                // Remove oldest entry
                if let Some((oldest_key, _)) = cache
                    .iter()
                    .min_by_key(|(_, entry)| entry.created_at)
                    .map(|(k, _)| k.clone())
                {
                    cache.remove(&oldest_key);
                }
            }
        }

        let entry = CacheEntry {
            value,
            created_at: Instant::now(),
            ttl: self.ttl,
        };

        cache.insert(key, entry);
        debug!("Cache entry added");
        Ok(())
    }

    async fn evict_expired_entries(&self, cache: &mut HashMap<String, CacheEntry>) {
        let expired_keys: Vec<String> = cache
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            cache.remove(&key);
        }
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Cache cleared");
    }
}

pub trait InputParser: Send + Sync {
    async fn parse(
        &self,
        input: &str,
        mode: &Mode,
        proficiency: &Proficiency,
        context: Option<&ConversationContext>,
    ) -> ProcessingResult<ParsedInput>;

    fn confidence_threshold(&self) -> f32;
    fn parser_name(&self) -> &'static str;
}

pub struct LLMParser {
    engine: Arc<LLMEngine>,
    config: GenerationConfig,
}

impl LLMParser {
    pub fn new(engine: Arc<LLMEngine>, config: GenerationConfig) -> Self {
        Self { engine, config }
    }
}

#[async_trait::async_trait]
impl InputParser for LLMParser {
    #[instrument(skip(self))]
    async fn parse(
        &self,
        input: &str,
        mode: &Mode,
        proficiency: &Proficiency,
        context: Option<&ConversationContext>,
    ) -> ProcessingResult<ParsedInput> {
        const MAX_RETRIES: u32 = 3;
        let start = Instant::now();

        if !self.engine.is_loaded() {
            return Err(InputProcessingError::LLMError {
                source: LLMError::ModelNotLoaded,
            });
        }

        let system_prompt = self.build_system_prompt(mode, proficiency, context);

        for attempt in 1..=MAX_RETRIES {
            match self
                .engine
                .simple_chat(input, Some(&system_prompt), Some(self.config.clone()))
            {
                Ok(response_content) => {
                    match self.parse_llm_response(
                        &response_content,
                        input,
                        mode,
                        proficiency,
                        start.elapsed().as_millis() as u64,
                    ) {
                        Ok(mut parsed_input) => {
                            parsed_input.parsing_method = if attempt == 1 {
                                ParsingMethod::LLMPrimary
                            } else {
                                ParsingMethod::LLMFallback
                            };
                            return Ok(parsed_input);
                        }
                        Err(parse_error) => {
                            if attempt == MAX_RETRIES {
                                warn!(
                                    attempts = attempt,
                                    error = ?parse_error,
                                    "All LLM parsing attempts failed"
                                );
                                return Err(InputProcessingError::ParsingError {
                                    attempts: MAX_RETRIES,
                                    last_error: parse_error,
                                });
                            }
                        }
                    }
                }
                Err(llm_error) => {
                    if attempt == MAX_RETRIES {
                        return Err(InputProcessingError::LLMError { source: llm_error });
                    }
                    warn!(attempt = attempt, error = ?llm_error, "LLM attempt failed");
                }
            }

            // Exponential backoff
            tokio::time::sleep(Duration::from_millis(100 * (1 << attempt))).await;
        }

        unreachable!("Should have returned or errored by now")
    }

    fn confidence_threshold(&self) -> f32 {
        0.7
    }

    fn parser_name(&self) -> &'static str {
        "LLMParser"
    }
}

impl LLMParser {
    fn build_system_prompt(
        &self,
        mode: &Mode,
        proficiency: &Proficiency,
        context: Option<&ConversationContext>,
    ) -> String {
        let mut prompt = format!(
            "You are an intelligent parsing assistant for a tutoring chatbot. Your task is to analyze user input and extract structured information.

Parse the following user input and return ONLY a valid JSON object with these exact fields:
- \"action\": The main action the user wants (e.g., \"explain\", \"teach\", \"help\", \"search\", \"analyze\", \"create\", \"debug\", \"review\", \"practice\", \"translate\")
- \"domain\": The subject area or programming/natural language (e.g., \"rust\", \"cpp\", \"python\", \"javascript\", \"italian\", \"spanish\", \"english\", \"mathematics\", \"general\")
- \"topic\": The specific topic within the domain (e.g., \"lifetimes\", \"borrowing\", \"syntax\", \"verbs\", \"grammar\", \"algorithms\", \"data structures\", \"functions\", \"variables\")
- \"confidence\": A float between 0.0 and 1.0 indicating parsing confidence

Context: {} {}",
            mode.get_system_prompt(),
            proficiency.get_system_prompt()
        );

        if let Some(ctx) = context {
            if !ctx.previous_interactions.is_empty() {
                prompt.push_str(&format!(
                    "\n\nPrevious conversation context: {}",
                    ctx.previous_interactions.join("; ")
                ));
            }
        }

        prompt.push_str(
            "\n\nRules:
1. Return ONLY valid JSON, no other text
2. If you cannot determine a field, use \"general\" or \"unknown\" as appropriate
3. Keep field values concise (1-3 words typically)
4. Use lowercase for consistency
5. Set confidence based on how certain you are about the parsing

Example output:
{\"action\": \"explain\", \"domain\": \"rust\", \"topic\": \"lifetimes\", \"confidence\": 0.9}",
        );

        prompt
    }

    fn parse_llm_response(
        &self,
        response_content: &str,
        original_input: &str,
        mode: &Mode,
        proficiency: &Proficiency,
        processing_time_ms: u64,
    ) -> Result<ParsedInput, String> {
        let json_str = extract_json_from_response(response_content)?;

        #[derive(Deserialize)]
        struct LLMParsedResponse {
            action: String,
            domain: String,
            topic: String,
            confidence: Option<f32>,
        }

        let parsed_response: LLMParsedResponse = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse LLM JSON response: {}", e))?;

        let action = sanitize_field(&parsed_response.action, "general");
        let domain = sanitize_field(&parsed_response.domain, "general");
        let topic = sanitize_field(&parsed_response.topic, "general");
        let confidence = parsed_response.confidence.unwrap_or(0.5).clamp(0.0, 1.0);

        Ok(ParsedInput {
            action,
            domain,
            topic,
            mode: mode.clone(),
            proficiency: proficiency.clone(),
            original_input: original_input.to_string(),
            confidence_score: confidence,
            parsing_method: ParsingMethod::LLMPrimary,
            detected_locale: "en-US".to_string(), // Will be set by caller
            security_flags: Vec::new(),
            metadata: HashMap::new(),
            processing_time_ms,
        })
    }
}

pub struct RuleBasedParser;

#[async_trait::async_trait]
impl InputParser for RuleBasedParser {
    #[instrument(skip(self))]
    async fn parse(
        &self,
        input: &str,
        mode: &Mode,
        proficiency: &Proficiency,
        _context: Option<&ConversationContext>,
    ) -> ProcessingResult<ParsedInput> {
        let start = Instant::now();
        let result =
            create_fallback_response(input, mode, proficiency, start.elapsed().as_millis() as u64);
        Ok(result)
    }

    fn confidence_threshold(&self) -> f32 {
        0.3
    }

    fn parser_name(&self) -> &'static str {
        "RuleBasedParser"
    }
}

pub struct ParsingPipeline {
    parsers: Vec<Box<dyn InputParser>>,
    cache: InputCache,
    validator: InputValidator,
    localization: LocalizationManager,
}

impl ParsingPipeline {
    pub fn new(
        parsers: Vec<Box<dyn InputParser>>,
        cache: InputCache,
        validator: InputValidator,
        localization: LocalizationManager,
    ) -> Self {
        Self {
            parsers,
            cache,
            validator,
            localization,
        }
    }

    #[instrument(skip(self))]
    pub async fn process(
        &self,
        input: &str,
        mode: &Mode,
        proficiency: &Proficiency,
        context: Option<&ConversationContext>,
    ) -> ProcessingResult<ParsedInput> {
        let start = Instant::now();

        // Input validation and sanitization
        let sanitized_input = self.validator.validate_and_sanitize(input)?;

        // Detect locale
        let detected_locale = self.localization.detect_locale(&sanitized_input);

        // Check cache first
        let cache_key = format!(
            "{}-{:?}-{:?}-{}",
            sanitized_input, mode, proficiency, detected_locale
        );
        if let Some(mut cached_result) = self.cache.get(&cache_key).await {
            cached_result.processing_time_ms = start.elapsed().as_millis() as u64;
            debug!("Returning cached result");
            return Ok(cached_result);
        }

        // Try parsers in order of preference
        let mut best_result: Option<ParsedInput> = None;
        let mut best_confidence = 0.0f32;

        for parser in &self.parsers {
            match parser
                .parse(&sanitized_input, mode, proficiency, context)
                .await
            {
                Ok(mut result) => {
                    result.detected_locale = detected_locale.clone();
                    result.processing_time_ms = start.elapsed().as_millis() as u64;

                    if result.confidence_score >= parser.confidence_threshold() {
                        info!(
                            parser = parser.parser_name(),
                            confidence = result.confidence_score,
                            "High-confidence parse successful"
                        );

                        // Cache the result
                        let _ = self.cache.set(cache_key, result.clone()).await;
                        return Ok(result);
                    }

                    if result.confidence_score > best_confidence {
                        best_confidence = result.confidence_score;
                        best_result = Some(result);
                    }
                }
                Err(e) => {
                    warn!(
                        parser = parser.parser_name(),
                        error = ?e,
                        "Parser failed"
                    );
                }
            }
        }

        // Return best result if we have one
        if let Some(mut result) = best_result {
            result.processing_time_ms = start.elapsed().as_millis() as u64;
            warn!(
                confidence = best_confidence,
                "Returning low-confidence result"
            );

            // Cache even low-confidence results to avoid reprocessing
            let _ = self.cache.set(cache_key, result.clone()).await;
            return Ok(result);
        }

        Err(InputProcessingError::ParsingError {
            attempts: self.parsers.len() as u32,
            last_error: "All parsers failed".to_string(),
        })
    }
}

impl Mode {
    pub fn from_u32(mode: u32) -> ProcessingResult<Self> {
        match mode {
            0 => Ok(Mode::Assistant),
            1 => Ok(Mode::Tutor),
            2 => Ok(Mode::WebSearch),
            3 => Ok(Mode::CodeReview),
            4 => Ok(Mode::Translation),
            _ => Err(InputProcessingError::ValidationError {
                reason: format!("Invalid mode: {}", mode),
            }),
        }
    }

    pub fn get_system_prompt(&self) -> String {
        let mode_context = match self {
            Mode::Assistant => "You are a helpful assistant providing general guidance and information.",
            Mode::Tutor => "You are an educational tutor focused on teaching and learning support.",
            Mode::WebSearch => "You are a research assistant with web search capabilities for finding current information.",
            Mode::CodeReview => "You are a code review specialist focused on analyzing and improving code quality.",
            Mode::Translation => "You are a translation specialist helping with language conversion and interpretation.",
        };
        mode_context.to_string()
    }

    pub fn get_default_confidence_threshold(&self) -> f32 {
        match self {
            Mode::Assistant => 0.6,
            Mode::Tutor => 0.7,
            Mode::WebSearch => 0.5,
            Mode::CodeReview => 0.8,
            Mode::Translation => 0.7,
        }
    }
}

impl Proficiency {
    pub fn from_u32(proficiency: u32) -> ProcessingResult<Self> {
        match proficiency {
            0 => Ok(Proficiency::Beginner),
            1 => Ok(Proficiency::Intermediate),
            2 => Ok(Proficiency::Advanced),
            3 => Ok(Proficiency::Expert),
            _ => Err(InputProcessingError::ValidationError {
                reason: format!("Invalid proficiency level: {}", proficiency),
            }),
        }
    }

    pub fn get_system_prompt(&self) -> String {
        let proficiency_context = match self {
            Proficiency::Beginner => "The user has basic knowledge and needs fundamental explanations with simple examples.",
            Proficiency::Intermediate => "The user has solid foundations and can handle moderate complexity with some technical details.",
            Proficiency::Advanced => "The user has deep understanding and appreciates technical depth with advanced concepts.",
            Proficiency::Expert => "The user is highly knowledgeable and expects sophisticated analysis with cutting-edge insights.",
        };
        proficiency_context.to_string()
    }

    pub fn get_complexity_multiplier(&self) -> f32 {
        match self {
            Proficiency::Beginner => 0.5,
            Proficiency::Intermediate => 0.7,
            Proficiency::Advanced => 0.9,
            Proficiency::Expert => 1.0,
        }
    }
}

// Serialization implementations remain the same but with added modes
// Serialization implementations for Mode and Proficiency
impl Serialize for Mode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mode_str = match self {
            Mode::Assistant => "assistant",
            Mode::Tutor => "tutor",
            Mode::WebSearch => "web_search",
            Mode::CodeReview => "code_review",
            Mode::Translation => "translation",
        };
        serializer.serialize_str(mode_str)
    }
}

impl<'de> Deserialize<'de> for Mode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mode_str = String::deserialize(deserializer)?;
        match mode_str.as_str() {
            "assistant" => Ok(Mode::Assistant),
            "tutor" => Ok(Mode::Tutor),
            "web_search" => Ok(Mode::WebSearch),
            "code_review" => Ok(Mode::CodeReview),
            "translation" => Ok(Mode::Translation),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid mode: {}",
                mode_str
            ))),
        }
    }
}

impl Serialize for Proficiency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let proficiency_str = match self {
            Proficiency::Beginner => "beginner",
            Proficiency::Intermediate => "intermediate",
            Proficiency::Advanced => "advanced",
            Proficiency::Expert => "expert",
        };
        serializer.serialize_str(proficiency_str)
    }
}

impl<'de> Deserialize<'de> for Proficiency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let proficiency_str = String::deserialize(deserializer)?;
        match proficiency_str.as_str() {
            "beginner" => Ok(Proficiency::Beginner),
            "intermediate" => Ok(Proficiency::Intermediate),
            "advanced" => Ok(Proficiency::Advanced),
            "expert" => Ok(Proficiency::Expert),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid proficiency: {}",
                proficiency_str
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests_per_minute: u32,
    pub max_requests_per_hour: u32,
    pub max_requests_per_day: u32,
    pub burst_capacity: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests_per_minute: 60,
            max_requests_per_hour: 1000,
            max_requests_per_day: 10000,
            burst_capacity: 10,
        }
    }
}

pub struct RateLimiter {
    config: RateLimitConfig,
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[instrument(skip(self))]
    pub async fn check_rate_limit(&self, client_id: &str) -> ProcessingResult<()> {
        let mut requests = self.requests.write().await;
        let now = Instant::now();

        let client_requests = requests
            .entry(client_id.to_string())
            .or_insert_with(Vec::new);

        // Clean up old requests
        client_requests.retain(|&request_time| {
            now.duration_since(request_time) < Duration::from_secs(86400) // 24 hours
        });

        // Check various rate limits
        let minute_ago = now - Duration::from_secs(60);
        let hour_ago = now - Duration::from_secs(3600);
        let day_ago = now - Duration::from_secs(86400);

        let requests_last_minute = client_requests
            .iter()
            .filter(|&&time| time > minute_ago)
            .count() as u32;

        let requests_last_hour = client_requests
            .iter()
            .filter(|&&time| time > hour_ago)
            .count() as u32;

        let requests_last_day = client_requests
            .iter()
            .filter(|&&time| time > day_ago)
            .count() as u32;

        if requests_last_minute >= self.config.max_requests_per_minute {
            return Err(InputProcessingError::RateLimitError {
                limit: self.config.max_requests_per_minute,
                window_seconds: 60,
            });
        }

        if requests_last_hour >= self.config.max_requests_per_hour {
            return Err(InputProcessingError::RateLimitError {
                limit: self.config.max_requests_per_hour,
                window_seconds: 3600,
            });
        }

        if requests_last_day >= self.config.max_requests_per_day {
            return Err(InputProcessingError::RateLimitError {
                limit: self.config.max_requests_per_day,
                window_seconds: 86400,
            });
        }

        // Record the request
        client_requests.push(now);
        Ok(())
    }

    pub async fn cleanup_old_entries(&self) {
        let mut requests = self.requests.write().await;
        let now = Instant::now();
        let cutoff = now - Duration::from_secs(86400);

        requests.retain(|_, client_requests| {
            client_requests.retain(|&time| time > cutoff);
            !client_requests.is_empty()
        });
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessingMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_processing_time_ms: f64,
    pub security_violations: u64,
    pub rate_limit_violations: u64,
    pub parser_success_rates: HashMap<String, f64>,
}

impl Default for ProcessingMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            cache_hits: 0,
            cache_misses: 0,
            avg_processing_time_ms: 0.0,
            security_violations: 0,
            rate_limit_violations: 0,
            parser_success_rates: HashMap::new(),
        }
    }
}

pub struct MetricsCollector {
    metrics: Arc<RwLock<ProcessingMetrics>>,
    processing_times: Arc<RwLock<Vec<u64>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(ProcessingMetrics::default())),
            processing_times: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn record_request(&self, processing_time_ms: u64, success: bool) {
        let mut metrics = self.metrics.write().await;
        let mut times = self.processing_times.write().await;

        metrics.total_requests += 1;
        if success {
            metrics.successful_requests += 1;
        } else {
            metrics.failed_requests += 1;
        }

        times.push(processing_time_ms);

        // Keep only last 1000 processing times for average calculation
        if times.len() > 1000 {
            times.drain(0..times.len() - 1000);
        }

        // Recalculate average
        metrics.avg_processing_time_ms = times.iter().sum::<u64>() as f64 / times.len() as f64;
    }

    pub async fn record_cache_hit(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_hits += 1;
    }

    pub async fn record_cache_miss(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_misses += 1;
    }

    pub async fn record_security_violation(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.security_violations += 1;
    }

    pub async fn record_rate_limit_violation(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.rate_limit_violations += 1;
    }

    pub async fn record_parser_result(&self, parser_name: String, success: bool) {
        let mut metrics = self.metrics.write().await;
        let current_rate = metrics
            .parser_success_rates
            .get(&parser_name)
            .unwrap_or(&0.0);

        // Simple exponential moving average
        let new_rate = if success {
            current_rate * 0.9 + 0.1
        } else {
            current_rate * 0.9
        };

        metrics.parser_success_rates.insert(parser_name, new_rate);
    }

    pub async fn get_metrics(&self) -> ProcessingMetrics {
        self.metrics.read().await.clone()
    }
}

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub client_id: String,
    pub timestamp: Instant,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub session_id: Option<String>,
}

impl RequestContext {
    pub fn new(client_id: String) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            client_id,
            timestamp: Instant::now(),
            user_agent: None,
            ip_address: None,
            session_id: None,
        }
    }

    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    pub fn with_ip_address(mut self, ip_address: String) -> Self {
        self.ip_address = Some(ip_address);
        self
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

fn extract_json_from_response(response: &str) -> Result<String, String> {
    let trimmed = response.trim();

    // Look for JSON block markers
    if let Some(start) = trimmed.find("```json") {
        if let Some(end) = trimmed[start + 7..].find("```") {
            return Ok(trimmed[start + 7..start + 7 + end].trim().to_string());
        }
    }

    // Look for raw JSON
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            if end > start {
                return Ok(trimmed[start..=end].to_string());
            }
        }
    }

    Err("No valid JSON found in response".to_string())
}

fn sanitize_field(field: &str, default: &str) -> String {
    if field.trim().is_empty() || field.to_lowercase() == "unknown" {
        default.to_string()
    } else {
        field.trim().to_lowercase()
    }
}

fn create_fallback_response(
    input: &str,
    mode: &Mode,
    proficiency: &Proficiency,
    processing_time_ms: u64,
) -> ParsedInput {
    let input_lower = input.to_lowercase();

    // Simple keyword-based classification
    let (action, domain, topic) = classify_input_heuristically(&input_lower);

    ParsedInput {
        action,
        domain,
        topic,
        mode: mode.clone(),
        proficiency: proficiency.clone(),
        original_input: input.to_string(),
        confidence_score: 0.3, // Low confidence for fallback
        parsing_method: ParsingMethod::RuleBased,
        detected_locale: "en-US".to_string(),
        security_flags: Vec::new(),
        metadata: HashMap::new(),
        processing_time_ms,
    }
}

fn classify_input_heuristically(input: &str) -> (String, String, String) {
    // Action classification
    let action =
        if input.contains("explain") || input.contains("what is") || input.contains("how does") {
            "explain"
        } else if input.contains("help") || input.contains("assist") {
            "help"
        } else if input.contains("teach") || input.contains("learn") || input.contains("tutorial") {
            "teach"
        } else if input.contains("search") || input.contains("find") || input.contains("look up") {
            "search"
        } else if input.contains("debug") || input.contains("fix") || input.contains("error") {
            "debug"
        } else if input.contains("review") || input.contains("check") || input.contains("analyze") {
            "review"
        } else if input.contains("translate") || input.contains("convert") {
            "translate"
        } else if input.contains("create") || input.contains("generate") || input.contains("make") {
            "create"
        } else {
            "general"
        };

    // Domain classification based on keywords
    let domain = if input.contains("rust") || input.contains("cargo") || input.contains("borrow") {
        "rust"
    } else if input.contains("python") || input.contains("pip") || input.contains("django") {
        "python"
    } else if input.contains("javascript") || input.contains("js") || input.contains("node") {
        "javascript"
    } else if input.contains("c++") || input.contains("cpp") {
        "cpp"
    } else if input.contains("html") || input.contains("css") || input.contains("web") {
        "web"
    } else if input.contains("math") || input.contains("calculus") || input.contains("algebra") {
        "mathematics"
    } else if input.contains("physics") || input.contains("chemistry") || input.contains("biology")
    {
        "science"
    } else if input.contains("french") || input.contains("spanish") || input.contains("german") {
        "language"
    } else {
        "general"
    };

    // Topic classification (simplified)
    let topic = if domain == "rust" {
        if input.contains("lifetime") || input.contains("borrow") {
            "lifetimes"
        } else if input.contains("ownership") {
            "ownership"
        } else if input.contains("trait") {
            "traits"
        } else if input.contains("async") {
            "async"
        } else {
            "general"
        }
    } else if domain == "python" {
        if input.contains("class") || input.contains("object") {
            "oop"
        } else if input.contains("function") || input.contains("def") {
            "functions"
        } else if input.contains("list") || input.contains("dict") {
            "data_structures"
        } else {
            "general"
        }
    } else if domain == "mathematics" {
        if input.contains("derivative") || input.contains("integral") {
            "calculus"
        } else if input.contains("equation") || input.contains("solve") {
            "algebra"
        } else {
            "general"
        }
    } else {
        "general"
    };

    (action.to_string(), domain.to_string(), topic.to_string())
}

pub struct InputProcessor {
    pipeline: ParsingPipeline,
    rate_limiter: RateLimiter,
    metrics: MetricsCollector,
    config: Config,
}

impl InputProcessor {
    pub fn new(llm_engine: Arc<LLMEngine>, config: Config) -> ProcessingResult<Self> {
        let validator = InputValidator::new(config.security.clone());
        let localization = LocalizationManager::new(config.localization.clone());
        let cache = InputCache::new(
            config.performance.cache_size,
            config.performance.cache_ttl_seconds,
        );

        // Create parsers in order of preference
        let parsers: Vec<Box<dyn InputParser>> = vec![
            Box::new(LLMParser::new(llm_engine, config.generation_config.clone())),
            Box::new(RuleBasedParser),
        ];

        let pipeline = ParsingPipeline::new(parsers, cache, validator, localization);
        let rate_limiter = RateLimiter::new(RateLimitConfig::default());
        let metrics = MetricsCollector::new();

        Ok(Self {
            pipeline,
            rate_limiter,
            metrics,
            config,
        })
    }

    #[instrument(skip(self, data))]
    pub async fn process_request(
        &self,
        data: Data,
        context: RequestContext,
    ) -> ProcessingResult<ProcessedRequest> {
        let start = Instant::now();

        // Rate limiting
        if let Err(e) = self.rate_limiter.check_rate_limit(&context.client_id).await {
            self.metrics.record_rate_limit_violation().await;
            return Err(e);
        }

        // Convert numeric mode and proficiency
        let mode = Mode::from_u32(data.mode)?;
        let proficiency = Proficiency::from_u32(data.proficiency)?;

        // Process the input through the pipeline
        let processing_result = self
            .pipeline
            .process(&data.input, &mode, &proficiency, data.context.as_ref())
            .await;

        let processing_time = start.elapsed().as_millis() as u64;

        match processing_result {
            Ok(parsed_input) => {
                // Record successful processing
                self.metrics.record_request(processing_time, true).await;

                let processed_request = ProcessedRequest {
                    mode,
                    proficiency,
                    original_input: data.input.clone(),
                    processed_input: parsed_input.action.clone(),
                    config: self.config.clone(),
                    processing_metadata: ProcessingMetadata {
                        request_id: context.request_id.clone(),
                        processing_time_ms: processing_time,
                        cache_hit: parsed_input.parsing_method == ParsingMethod::LLMPrimary,
                        security_checks_passed: parsed_input.security_flags.is_empty(),
                        confidence_score: parsed_input.confidence_score,
                    },
                };

                info!(
                    request_id = context.request_id,
                    processing_time_ms = processing_time,
                    confidence = parsed_input.confidence_score,
                    "Request processed successfully"
                );

                Ok(processed_request)
            }
            Err(e) => {
                // Record failed processing
                self.metrics.record_request(processing_time, false).await;

                if matches!(e, InputProcessingError::SecurityViolation { .. }) {
                    self.metrics.record_security_violation().await;
                }

                error!(
                    request_id = context.request_id,
                    processing_time_ms = processing_time,
                    error = ?e,
                    "Request processing failed"
                );

                Err(e)
            }
        }
    }

    pub async fn get_health_status(&self) -> HashMap<String, serde_json::Value> {
        let metrics = self.metrics.get_metrics().await;
        let mut status = HashMap::new();

        status.insert(
            "status".to_string(),
            serde_json::Value::String("healthy".to_string()),
        );
        status.insert(
            "total_requests".to_string(),
            serde_json::Value::Number(metrics.total_requests.into()),
        );
        status.insert(
            "success_rate".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(if metrics.total_requests > 0 {
                    metrics.successful_requests as f64 / metrics.total_requests as f64
                } else {
                    0.0
                })
                .unwrap_or(serde_json::Number::from(0)),
            ),
        );
        status.insert(
            "avg_processing_time_ms".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(metrics.avg_processing_time_ms)
                    .unwrap_or(serde_json::Number::from(0)),
            ),
        );
        status.insert(
            "cache_hit_rate".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(if metrics.cache_hits + metrics.cache_misses > 0 {
                    metrics.cache_hits as f64 / (metrics.cache_hits + metrics.cache_misses) as f64
                } else {
                    0.0
                })
                .unwrap_or(serde_json::Number::from(0)),
            ),
        );

        status
    }

    pub async fn clear_cache(&self) -> ProcessingResult<()> {
        self.pipeline.cache.clear().await;
        Ok(())
    }

    pub async fn cleanup_resources(&self) {
        self.rate_limiter.cleanup_old_entries().await;
        // Additional cleanup tasks can be added here
    }
}

pub struct InputProcessorBuilder {
    config: Config,
    rate_limit_config: Option<RateLimitConfig>,
}

impl InputProcessorBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            rate_limit_config: None,
        }
    }

    pub fn with_model_path(mut self, path: String) -> Self {
        self.config.model_path = path;
        self
    }

    pub fn with_security_config(mut self, security: SecurityConfig) -> Self {
        self.config.security = security;
        self
    }

    pub fn with_performance_config(mut self, performance: PerformanceConfig) -> Self {
        self.config.performance = performance;
        self
    }

    pub fn with_localization_config(mut self, localization: LocalizationConfig) -> Self {
        self.config.localization = localization;
        self
    }

    pub fn with_rate_limiting(mut self, rate_limit_config: RateLimitConfig) -> Self {
        self.rate_limit_config = Some(rate_limit_config);
        self
    }

    pub fn build(self, llm_engine: Arc<LLMEngine>) -> ProcessingResult<InputProcessor> {
        InputProcessor::new(llm_engine, self.config)
    }
}

impl Default for InputProcessorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_input_validation() {
        let validator = InputValidator::new(SecurityConfig::default());

        // Test normal input
        assert!(validator
            .validate_and_sanitize("Hello, how are you?")
            .is_ok());

        // Test empty input
        assert!(validator.validate_and_sanitize("").is_err());

        // Test potential SQL injection
        assert!(validator
            .validate_and_sanitize("SELECT * FROM users")
            .is_err());
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let rate_limiter = RateLimiter::new(RateLimitConfig {
            max_requests_per_minute: 2,
            ..Default::default()
        });

        let client_id = "test_client";

        // First two requests should succeed
        assert!(rate_limiter.check_rate_limit(client_id).await.is_ok());
        assert!(rate_limiter.check_rate_limit(client_id).await.is_ok());

        // Third request should fail
        assert!(rate_limiter.check_rate_limit(client_id).await.is_err());
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let cache = InputCache::new(10, 3600);

        let test_input = ParsedInput {
            action: "test".to_string(),
            domain: "test".to_string(),
            topic: "test".to_string(),
            mode: Mode::Assistant,
            proficiency: Proficiency::Beginner,
            original_input: "test input".to_string(),
            confidence_score: 0.9,
            parsing_method: ParsingMethod::LLMPrimary,
            detected_locale: "en-US".to_string(),
            security_flags: Vec::new(),
            metadata: HashMap::new(),
            processing_time_ms: 100,
        };

        // Test cache miss
        assert!(cache.get("test_key").await.is_none());

        // Test cache set and hit
        cache
            .set("test_key".to_string(), test_input.clone())
            .await
            .unwrap();
        let cached_result = cache.get("test_key").await;
        assert!(cached_result.is_some());
        assert_eq!(cached_result.unwrap().action, "test");
    }
}

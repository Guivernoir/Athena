use serde::{Serialize, Deserialize};
use crate::llama::{LLMEngine, GenerationConfig};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Context {
    pub action: String,
    pub domain: String,
    pub topic: String,
    pub raw_input: String,
}

#[derive(Error, Debug)]
pub enum ContextError {
    #[error("Failed to analyze input: {0}")]
    AnalysisFailed(String),
    #[error("Invalid analysis format: {0}")]
    InvalidFormat(String),
}

impl Context {
    const ANALYSIS_PROMPT: &'static str = r#"<|im_start|>system
You are an input analyzer. Extract exactly these REQUIRED elements:
1. Action (verb: explain, write, debug, compare, etc.)
2. Domain (subject: Rust, Python, math, etc.)
3. Topic (specific concept: lifetimes, vectors, etc.)

Respond ONLY with this JSON format:
{"action":"...","domain":"...","topic":"..."}
No commentary or additional text!<|im_end|>
"#;

    pub async fn analyze(input: String, llm: &LLMEngine) -> Result<Self, ContextError> {
        let prompt = format!(
            "{}<|im_start|>user\n{}\n<|im_end|>\n<|im_start|>assistant\n",
            Self::ANALYSIS_PROMPT,
            input
        );

        let response = llm.generate(&prompt, Some(GenerationConfig {
            max_tokens: 100,
            temperature: 0.1,
            ..Default::default()
        })).map_err(|e| ContextError::AnalysisFailed(e.to_string()))?;

        let clean_response = response.split('<').next().unwrap_or(&response).trim();
        let analysis: ContentAnalysis = serde_json::from_str(clean_response)
            .map_err(|e| ContextError::InvalidFormat(e.to_string()))?;

        Ok(Self {
            action: analysis.action,
            domain: analysis.domain,
            topic: analysis.topic,
            raw_input: input,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ContentAnalysis {
    action: String,
    domain: String,
    topic: String,
}
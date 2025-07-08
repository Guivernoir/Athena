use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;

mod ffi;
use ffi::RawEngine;

/**
 * Safe Rust Wrapper for LLM Engine
 * 
 * This is your clean, safe interface that integrates seamlessly with your
 * existing Tauri backend. No unsafe blocks bleeding into your application logic,
 * no mysterious segfaults at 3 AM - just clean, idiomatic Rust that works.
 */

#[derive(Error, Debug)]
pub enum LLMError {
    #[error("Failed to initialize engine with model: {model_path}")]
    InitializationFailed { model_path: String },
    #[error("Engine is not loaded or has been disposed")]
    EngineNotLoaded,
    #[error("Text generation failed: {reason}")]
    GenerationFailed { reason: String },
    #[error("Model file not found: {path}")]
    ModelNotFound { path: String },
    #[error("Invalid input parameters: {details}")]
    InvalidInput { details: String },
}

pub type Result<T> = std::result::Result<T, LLMError>;

#[derive(Debug, Clone)]
pub struct GenerationConfig {
    pub max_tokens: i32,
    pub temperature: f32,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

pub struct LLMEngine {
    inner: Arc<Mutex<Option<RawEngine>>>,
    model_path: String,
}

impl LLMEngine {
    pub fn new<P: AsRef<Path>>(model_path: P) -> Result<Self> {
        let path_str = model_path.as_ref().to_string_lossy().to_string();
        if !model_path.as_ref().exists() {
            return Err(LLMError::ModelNotFound { path: path_str });
        }
        let raw_engine = unsafe {
            RawEngine::new(&path_str).ok_or_else(|| {
                LLMError::InitializationFailed {
                    model_path: path_str.clone(),
                }
            })?
        };
        Ok(LLMEngine {
            inner: Arc::new(Mutex::new(Some(raw_engine))),
            model_path: path_str,
        })
    }
    pub fn generate(&self, prompt: &str, config: Option<GenerationConfig>) -> Result<String> {
        if prompt.trim().is_empty() {
            return Err(LLMError::InvalidInput {
                details: "Empty prompt provided".to_string(),
            });
        }
        let config = config.unwrap_or_default();
        let guard = self.inner.lock().unwrap();
        match guard.as_ref() {
            Some(engine) => {
                let result = unsafe {
                    engine.generate(prompt, config.max_tokens, config.temperature)
                };
                result.ok_or_else(|| LLMError::GenerationFailed {
                    reason: "C++ engine returned null result".to_string(),
                })
            }
            None => Err(LLMError::EngineNotLoaded),
        }
    }
    pub fn chat(
        &self,
        messages: &[ChatMessage],
        config: Option<GenerationConfig>,
    ) -> Result<String> {
        if messages.is_empty() {
            return Err(LLMError::InvalidInput {
                details: "No messages provided".to_string(),
            });
        }
        let config = config.unwrap_or_default();
        let system_prompt = messages
            .iter()
            .find(|msg| matches!(msg.role, MessageRole::System))
            .map(|msg| msg.content.as_str());
        let user_message = messages
            .iter()
            .rev()
            .find(|msg| matches!(msg.role, MessageRole::User))
            .ok_or_else(|| LLMError::InvalidInput {
                details: "No user message found".to_string(),
            })?;
        let guard = self.inner.lock().unwrap();
        match guard.as_ref() {
            Some(engine) => {
                let result = unsafe {
                    engine.chat(system_prompt, &user_message.content, config.max_tokens)
                };
                result.ok_or_else(|| LLMError::GenerationFailed {
                    reason: "Chat generation failed".to_string(),
                })
            }
            None => Err(LLMError::EngineNotLoaded),
        }
    }
    pub fn simple_chat(
        &self,
        user_message: &str,
        system_prompt: Option<&str>,
        config: Option<GenerationConfig>,
    ) -> Result<String> {
        let mut messages = Vec::new();
        if let Some(sys) = system_prompt {
            messages.push(ChatMessage {
                role: MessageRole::System,
                content: sys.to_string(),
            });
        }
        messages.push(ChatMessage {
            role: MessageRole::User,
            content: user_message.to_string(),
        });
        self.chat(&messages, config)
    }
    pub fn is_loaded(&self) -> bool {
        let guard = self.inner.lock().unwrap();
        match guard.as_ref() {
            Some(engine) => unsafe { engine.is_loaded() },
            None => false,
        }
    }
    pub fn get_model_info(&self) -> String {
        let guard = self.inner.lock().unwrap();
        match guard.as_ref() {
            Some(engine) => unsafe { engine.get_model_info() },
            None => format!("Engine not loaded (model: {})", self.model_path),
        }
    }
    pub fn model_path(&self) -> &str {
        &self.model_path
    }
    pub fn dispose(&self) {
        let mut guard = self.inner.lock().unwrap();
        *guard = None;
    }
}

unsafe impl Send for LLMEngine {}
unsafe impl Sync for LLMEngine {}

impl Clone for LLMEngine {
    fn clone(&self) -> Self {
        LLMEngine {
            inner: Arc::clone(&self.inner),
            model_path: self.model_path.clone(),
        }
    }
}

impl LLMEngine {
    pub fn from_models_dir() -> Result<Self> {
        let model_path = "llama/models/qwen2.5-0.5b-instruct-q5_k_m.gguf";
        Self::new(model_path)
    }
    pub fn complete(&self, prompt: &str) -> Result<String> {
        self.generate(prompt, None)
    }
    pub fn assist(&self, user_message: &str) -> Result<String> {
        let system_prompt = "You are a helpful AI assistant. Provide clear, accurate, and concise responses.";
        self.simple_chat(user_message, Some(system_prompt), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_engine_creation() {
        let engine = LLMEngine::from_models_dir();
        assert!(engine.is_ok(), "Engine creation should succeed with valid model");
    }
    #[test]
    fn test_simple_generation() {
        let engine = LLMEngine::from_models_dir().unwrap();
        let result = engine.complete("Hello, world!");
        assert!(result.is_ok(), "Simple generation should work");
        println!("Generated: {}", result.unwrap());
    }
    
}
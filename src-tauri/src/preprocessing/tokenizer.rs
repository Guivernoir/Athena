use thiserror::Error;

#[derive(Error, Debug)]
pub enum TokenizerError {
    #[error("No tokens found in input")]
    NoTokens,
    #[error("Token count exceeds limit: {0}")]
    TooManyTokens(usize),
}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub tokens: Vec<String>,
    pub word_count: usize,
    pub sentence_count: usize,
}

pub struct Tokenizer;

impl Tokenizer {
    const MAX_TOKENS: usize = 2000;
    
    pub fn tokenize(input: &str) -> Result<TokenInfo, TokenizerError> {
        let tokens: Vec<String> = input
            .split_whitespace()
            .map(|token| token.to_lowercase())
            .collect();
        
        if tokens.is_empty() {
            return Err(TokenizerError::NoTokens);
        }
        
        if tokens.len() > Self::MAX_TOKENS {
            return Err(TokenizerError::TooManyTokens(tokens.len()));
        }
        
        let sentence_count = input
            .chars()
            .filter(|&c| c == '.' || c == '!' || c == '?')
            .count()
            .max(1);
        
        Ok(TokenInfo {
            word_count: tokens.len(),
            sentence_count,
            tokens,
        })
    }
}
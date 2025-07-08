use thiserror::Error;

#[derive(Error, Debug)]
pub enum CleanerError {
    #[error("Input is empty after cleaning")]
    EmptyInput,
    #[error("Input exceeds maximum length: {0}")]
    TooLong(usize),
}

pub struct Cleaner;

impl Cleaner {
    const MAX_LENGTH: usize = 10000;
    
    pub fn clean(input: &str) -> Result<String, CleanerError> {
        if input.trim().is_empty() {
            return Err(CleanerError::EmptyInput);
        }
        
        let cleaned = input
            .trim()
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .filter(|&c| c.is_ascii() || c.is_whitespace())
            .collect::<String>();
        
        if cleaned.len() > Self::MAX_LENGTH {
            return Err(CleanerError::TooLong(cleaned.len()));
        }
        
        if cleaned.trim().is_empty() {
            return Err(CleanerError::EmptyInput);
        }
        
        Ok(cleaned)
    }
}
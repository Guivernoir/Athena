pub mod router;
pub mod context;
pub mod cleaner;
pub mod tokenizer;
pub mod formatter;

pub use router::{Mode, Proficiency, Personality};
pub use context::{Context, ContextError};
pub use cleaner::{Cleaner, CleanerError};
pub use tokenizer::{Tokenizer, TokenizerError};
pub use formatter::{FormattedInput, FormatterError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PreprocessorError {
    #[error("Context analysis failed: {0}")]
    Context(#[from] ContextError),
    #[error("Cleaning failed: {0}")]
    Cleaner(#[from] CleanerError),
    #[error("Tokenization failed: {0}")]
    Tokenizer(#[from] TokenizerError),
    #[error("Formatting failed: {0}")]
    Formatter(#[from] FormatterError),
}

pub struct Preprocessor;

impl Preprocessor {
    pub async fn process(
        input: String,
        mode: Mode,
        proficiency: Proficiency,
        personality: Personality,
        llm: &crate::llama::LLMEngine,
    ) -> Result<FormattedInput, PreprocessorError> {
        let cleaned = Cleaner::clean(&input)?;
        let context = Context::analyze(cleaned.clone(), llm).await?;
        let tokens = Tokenizer::tokenize(&cleaned)?;
        let formatted = FormattedInput::new(context, tokens, mode, proficiency, personality)?;
        
        Ok(formatted)
    }
}
//! # Erika Tutoring System
//!
//! A modular Rust application for processing user input through an AI-powered tutoring system.
//! The system consists of two main modules:
//! - `receive`: Handles input processing, parsing, and LLM communication for analysis
//! - `send`: Generates personality-driven responses using the "Erika" character
//!
//! ## Architecture
//!
//! ```text
//! User Input → receive.rs (Parse & Analyze) → send.rs (Generate Response) → Final Output
//! ```
//!
//! The system supports multiple modes (Assistant, Tutor, WebSearch) and proficiency levels
//! (Beginner, Intermediate, Advanced, Expert) to tailor responses appropriately.

pub mod receive;
pub mod send;

// Re-export commonly used types for easier access
pub use receive::*;

pub use send::*;

/// Main entry point for processing a complete request through the system
///
/// # Arguments
/// * `json_data` - JSON string containing the user input data
///
/// # Returns
/// * `Result<String, String>` - Serialized ErikaResponse or error message
///
/// # Example
/// ```rust
/// use your_crate::{process_complete_request};
///
/// #[tokio::main]
/// async fn main() {
///     let input = r#"{"mode": 1, "proficiency": 2, "input": "How do Rust lifetimes work?"}"#;
///     match process_complete_request(input).await {
///         Ok(response) => println!("Success: {}", response),
///         Err(error) => eprintln!("Error: {}", error),
///     }
/// }
/// ```
pub async fn process_complete_request(json_data: &str) -> Result<String, String> {
    send::handle_erika_request(json_data).await
}

/// Convenience function to create a basic input data structure
///
/// # Arguments
/// * `mode` - Operating mode (0=Assistant, 1=Tutor, 2=WebSearch)
/// * `proficiency` - User proficiency level (0=Beginner, 1=Intermediate, 2=Advanced, 3=Expert)
/// * `input` - The user's input text
///
/// # Returns
/// * `Data` - Structured data ready for processing
///
/// # Example
/// ```rust
/// use your_crate::create_input_data;
///
/// let data = create_input_data(1, 2, "Explain Rust ownership".to_string());
/// ```
pub fn create_input_data(mode: u32, proficiency: u32, input: String) -> Data {
    Data { mode, proficiency, input }
}

/// Validates that the required environment variables are set
///
/// # Returns
/// * `Result<(), String>` - Ok if all required variables are present, Err with missing variables
///
/// # Required Environment Variables
/// - `MODEL`: The AI model to use
/// - `API_KEY`: Authentication key for the AI service
/// - `API_URL`: Base URL for the AI API
/// - `NUMBER_OF_QUERIES`: Optional, defaults to 5
/// - `MAX_RESEARCH_LOOPS`: Optional, defaults to 3
pub fn validate_environment() -> Result<(), String> {
    let required_vars = ["MODEL", "API_KEY", "API_URL"];
    let mut missing_vars = Vec::new();

    for var in &required_vars {
        if std::env::var(var).is_err() {
            missing_vars.push(*var);
        }
    }

    if missing_vars.is_empty() {
        Ok(())
    } else {
        Err(format!("Missing required environment variables: {}", missing_vars.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_input_data() {
        let data = create_input_data(1, 2, "Test input".to_string());
        assert_eq!(data.mode, 1);
        assert_eq!(data.proficiency, 2);
        assert_eq!(data.input, "Test input");
    }

    #[test]
    fn test_mode_enum_conversion() {
        assert!(matches!(Mode::from_u32(0), Ok(Mode::Assistant)));
        assert!(matches!(Mode::from_u32(1), Ok(Mode::Tutor)));
        assert!(matches!(Mode::from_u32(2), Ok(Mode::WebSearch)));
        assert!(Mode::from_u32(3).is_err());
    }

    #[test]
    fn test_proficiency_enum_conversion() {
        assert!(matches!(Proficiency::from_u32(0), Ok(Proficiency::Beginner)));
        assert!(matches!(Proficiency::from_u32(1), Ok(Proficiency::Intermediate)));
        assert!(matches!(Proficiency::from_u32(2), Ok(Proficiency::Advanced)));
        assert!(matches!(Proficiency::from_u32(3), Ok(Proficiency::Expert)));
        assert!(Proficiency::from_u32(4).is_err());
    }
}
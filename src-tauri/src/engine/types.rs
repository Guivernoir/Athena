//! Shared structs.

#[derive(Debug, Clone)]
pub enum Task {
    Chat(String),
    Command(String),
}

#[derive(Debug, Clone)]
pub struct EngineState {
    pub persona: String,
    pub turns: Vec<(String, String)>, // (user, assistant)
}
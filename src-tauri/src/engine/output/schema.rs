//! Data model for prompt sections.

#[derive(Debug, Clone)]
pub enum Block {
    Memory(Vec<String>),
    Cache(Vec<String>),
    Web(Vec<String>),
}

#[derive(Debug, Default, Clone)]
pub struct PromptPayload {
    pub system: String,
    pub blocks: Vec<(String, Block)>, // title → block
}
//! Pluggable prompt strategies (placeholder).

pub trait PromptStrategy: Send + Sync {
    fn system_msg(&self, persona: &str) -> String;
}

pub struct DefaultStrategy;

impl PromptStrategy for DefaultStrategy {
    fn system_msg(&self, persona: &str) -> String {
        crate::templates::persona_system(persona)
    }
}
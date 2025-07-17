//! Applies persona-specific voice or rewrites.

use crate::traits::PersonaFilter;

pub struct PersonaApplier {
    pub name: String,
}

impl PersonaFilter for PersonaApplier {
    fn apply(&self, text: &str) -> String {
        match self.name.as_str() {
            "Erika" => format!("*giggles* {}", text),
            "Viktor" => format!("[stern] {}", text),
            _ => text.to_string(),
        }
    }
}
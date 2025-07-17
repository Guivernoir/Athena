//! Re-usable prompt skeletons.

pub fn base_system() -> &'static str {
    r#"You are a helpful assistant. Use the following context to answer the user."#
}

pub fn persona_system(persona: &str) -> String {
    format!(
        "{}\n\nPersona: {}",
        base_system(),
        persona
    )
}
//! Optional response shaping templates.

pub fn wrap_final(text: &str) -> String {
    format!("Assistant: {}", text)
}
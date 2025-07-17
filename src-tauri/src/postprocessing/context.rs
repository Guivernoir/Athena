//! Optional metadata injection into the LLM output (timestamp, persona).

use crate::traits::ContextInjector;

pub struct TimestampInjector;

impl ContextInjector for TimestampInjector {
    fn inject(&self, text: &str) -> String {
        let now = chrono::Utc::now().to_rfc3339();
        format!("[{}] {}", &now[..19], text)
    }
}
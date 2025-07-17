//! Pluggable post-processing traits.

pub trait ContextInjector: Send + Sync {
    fn inject(&self, text: &str) -> String;
}

pub trait PersonaFilter: Send + Sync {
    fn apply(&self, text: &str) -> String;
}
//! Shared traits.

#[async_trait::async_trait]
pub trait ContextProvider: Send + Sync {
    async fn build(&self, user_input: &str, top_k: usize) -> anyhow::Result<String>;
}
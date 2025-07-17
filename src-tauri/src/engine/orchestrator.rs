//! High-level coordinator: input → retrieval → LLM prompt → cache.

use crate::{
    core::{parse_input, update_state},
    output::builder::PromptBuilder,
    retrieval::router::Router,
    types::{EngineState, Task},
};
use anyhow::Result;

pub struct Orchestrator {
    router: Router,
    builder: PromptBuilder,
    state: EngineState,
}

impl Orchestrator {
    pub fn new(
        router: Router,
        builder: PromptBuilder,
        persona: &str,
    ) -> Self {
        Self {
            router,
            builder,
            state: EngineState {
                persona: persona.to_string(),
                turns: Vec::new(),
            },
        }
    }

    /// Drive one conversational turn.
    pub async fn turn(&mut self, user_input: &str) -> Result<String> {
        let task = parse_input(user_input);

        match task {
            Task::Command(cmd) => Ok(format!("[command: {}]", cmd)), // stub
            Task::Chat(text) => {
                let prompt = self.builder.build(&text, 5).await?;
                // TODO: feed prompt to LLM via llama::Engine
                let reply = "LLM reply placeholder".to_string();

                update_state(&mut self.state, user_input, &reply);
                Ok(reply)
            }
        }
    }
}
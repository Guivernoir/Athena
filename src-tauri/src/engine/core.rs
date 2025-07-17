//! Conversation logic, internal commands, input parsing.

use crate::types::{EngineState, Task};
use anyhow::Result;

pub fn parse_input(text: &str) -> Task {
    if text.starts_with('/') {
        Task::Command(text[1..].trim().to_string())
    } else {
        Task::Chat(text.to_string())
    }
}

pub fn update_state(state: &mut EngineState, user_msg: &str, assistant_msg: &str) {
    state.turns.push((user_msg.to_string(), assistant_msg.to_string()));
    if state.turns.len() > 20 {
        state.turns.remove(0);
    }
}
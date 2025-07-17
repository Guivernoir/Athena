//! Injects blocks into the final prompt string.

use crate::schema::PromptPayload;

pub fn inject(payload: PromptPayload) -> String {
    let mut prompt = String::new();
    prompt.push_str(&payload.system);
    prompt.push_str("\n\n");

    for (title, block) in payload.blocks {
        prompt.push_str(&format!("## {}\n", title));
        match block {
            crate::schema::Block::Memory(v)
            | crate::schema::Block::Cache(v)
            | crate::schema::Block::Web(v) => {
                for line in v {
                    prompt.push_str(&format!("- {}\n", line));
                }
            }
        }
        prompt.push('\n');
    }

    prompt
}
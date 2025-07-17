//! Optional JSON or tool-call interpreter (stub).

pub fn interpret(raw: &str) -> String {
    // If the LLM returned ```json {...}```, strip fences and pretty-print.
    if raw.trim_start().starts_with("```json") {
        raw.trim_start_matches("```json")
            .trim_end_matches("```")
            .trim()
            .to_string()
    } else {
        raw.to_string()
    }
}
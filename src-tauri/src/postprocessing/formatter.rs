//! Cleans and trims the raw LLM response.

pub fn clean(raw: &str) -> String {
    let trimmed = raw.trim();
    // Remove code-fence markers if LLM wrapped the answer.
    trimmed
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string()
}
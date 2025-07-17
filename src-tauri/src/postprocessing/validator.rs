//! Safety & policy checks.

pub fn validate(text: &str) -> Result<String, &'static str> {
    if text.len() > 4096 {
        return Err("response too long");
    }
    if text.contains("<script>") {
        return Err("unsafe html");
    }
    Ok(text.to_string())
}
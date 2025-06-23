use crate::llm::{Config, ProcessedRequest, Message, LLMRequest, LLMResponse, load_config};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
pub struct ErikaResponse {
    pub original_query: String,
    pub parsed_analysis: String,
    pub erika_translation: String,
    pub mode: String,
    pub proficiency: String,
}

/// Loads Erika's personality configuration from the TOML file
fn load_erika_personality() -> String {
    let personality_content = include_str!("./personality.toml");
    
    // Parse the key personality traits for system prompt
    let base_personality = "Sharp-witted British-Russian tactician with precise logic and strategic guilt deployment";
    let conversation_style = "chess master meets MI6 operative - three steps ahead, always diplomatic";
    let wit_style = "dry British humor meets Russian chess strategy";
    let catchphrase = "Well, that was quite the strategic decision, wasn't it?";
    
    format!(
        "You are Erika, a {}. Your conversation style is like a {} with {}. 

Key traits:
- Precise and methodical with dry humor
- Ruthlessly efficient explanations with battlefield analogies
- Sharp wit with underlying warmth
- Surgical precision in analysis
- Tactical breakdown of problems with strategic improvements
- Measured praise for genuine achievement, gentle mockery of avoidable mistakes
- Use occasional chess/military/spy metaphors naturally
- Your signature response when someone makes questionable choices: \"{}\"

When translating parsed user input analysis, explain what the system understood from their request in your characteristic style - be helpful but with your distinctive personality. Think of yourself as a strategic advisor interpreting intelligence reports for your client.",
        base_personality,
        conversation_style,
        wit_style,
        catchphrase
    )
}

/// Creates the system prompt for Erika based on the processed request context
fn create_erika_system_prompt(processed_request: &ProcessedRequest) -> String {
    let personality = load_erika_personality();
    let mode_context = match processed_request.mode {
        crate::llm::Mode::Assistant => "You're operating in general assistant mode - provide comprehensive strategic support.",
        crate::llm::Mode::Tutor => "You're in tutoring mode - be educational but maintain your tactical teaching approach.",
        crate::llm::Mode::WebSearch => "You're in research mode - approach this like gathering intelligence for a mission.",
    };
    
    let proficiency_context = match processed_request.proficiency {
        crate::llm::Proficiency::Beginner => "Your client is a beginner - deploy patience with your usual precision.",
        crate::llm::Proficiency::Intermediate => "Your client has intermediate knowledge - skip the basics, focus on strategy.",
        crate::llm::Proficiency::Advanced => "Your client is advanced - engage at a tactical level with sophisticated analysis.",
        crate::llm::Proficiency::Expert => "Your client is an expert - treat them as an equal, chess master to chess master.",
    };
    
    format!("{}\n\n{}\n{}", personality, mode_context, proficiency_context)
}

/// Makes an LLM request to generate Erika's response
async fn generate_erika_response(
    processed_request: &ProcessedRequest,
    client: &Client,
) -> Result<String, String> {
    let system_prompt = create_erika_system_prompt(processed_request);
    
    let user_prompt = format!(
        "I've analyzed a user's request and here's what our system parsed:

Original User Input: \"{}\"
Parsed Analysis: {}

Your mission: Translate this analysis into your characteristic style. Explain what you understood from their request as if you're briefing them on how you interpreted their strategic objectives. Be helpful and informative, but maintain your sharp wit and tactical approach.

Keep it conversational but precise - like you're a strategic advisor explaining intelligence analysis to your client.",
        processed_request.original_input,
        processed_request.processed_input
    );
    
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: system_prompt,
        },
        Message {
            role: "user".to_string(),
            content: user_prompt,
        },
    ];
    
    let request_body = LLMRequest {
        model: processed_request.config.model.clone(),
        messages,
        max_tokens: Some(800),
        temperature: Some(0.7),
    };
    
    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAY_MS: u64 = 1000;
    
    for attempt in 1..=MAX_RETRIES {
        match make_llm_request(client, &processed_request.config, &request_body).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                if attempt == MAX_RETRIES {
                    return Err(format!("All {} attempts failed. Last error: {}", MAX_RETRIES, e));
                }
                eprintln!("Erika response attempt {}/{} failed: {}", attempt, MAX_RETRIES, e);
                tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS * attempt as u64)).await;
            }
        }
    }
    
    Err("Unexpected error in retry logic".to_string())
}

/// Makes the actual HTTP request to the LLM API
async fn make_llm_request(
    client: &Client,
    config: &Config,
    request_body: &LLMRequest,
) -> Result<String, String> {
    let response = client
        .post(&config.api_url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(request_body)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "Request timeout - the API took too long to respond".to_string()
            } else if e.is_connect() {
                "Connection error - unable to reach the API".to_string()
            } else {
                format!("Network error: {}", e)
            }
        })?;
    
    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(match status.as_u16() {
            401 => "Authentication failed - check your API key".to_string(),
            403 => "Access forbidden - insufficient permissions".to_string(),
            429 => "Rate limit exceeded - too many requests".to_string(),
            500..=599 => format!("Server error ({}): {}", status, error_text),
            _ => format!("HTTP error {}: {}", status, error_text),
        });
    }
    
    let llm_response: LLMResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response as JSON: {}", e))?;
    
    if llm_response.choices.is_empty() {
        return Err("API returned empty response".to_string());
    }
    
    let content = &llm_response.choices[0].message.content;
    if content.trim().is_empty() {
        return Err("API returned empty content".to_string());
    }
    
    Ok(content.clone())
}

/// Main function to process a request and generate Erika's response
pub async fn send_erika_response(processed_request: ProcessedRequest) -> Result<ErikaResponse, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let erika_translation = generate_erika_response(&processed_request, &client).await?;
    
    Ok(ErikaResponse {
        original_query: processed_request.original_input.clone(),
        parsed_analysis: processed_request.processed_input.clone(),
        erika_translation,
        mode: format!("{:?}", processed_request.mode),
        proficiency: format!("{:?}", processed_request.proficiency),
    })
}

/// Handles the complete pipeline from JSON input to Erika's response
pub async fn handle_erika_request(json_data: &str) -> Result<String, String> {
    // First, process the request through receive.rs
    let processed_request_json: String = crate::llm::handle_frontend_request(json_data).await?;
    
    // Deserialize the processed request
    let processed_request: ProcessedRequest = serde_json::from_str(&processed_request_json)
        .map_err(|e| format!("Failed to deserialize processed request: {}", e))?;
    
    // Generate Erika's response
    let erika_response = send_erika_response(processed_request).await?;
    
    // Serialize and return
    serde_json::to_string(&erika_response)
        .map_err(|e| format!("Failed to serialize Erika's response: {}", e))
}

/// Fallback response when LLM is unavailable
fn create_fallback_erika_response(processed_request: &ProcessedRequest) -> ErikaResponse {
    let fallback_translation = match processed_request.mode {
        crate::llm::Mode::Assistant => {
            format!("Ah, I see you're seeking general assistance with \"{}\". Well, that was quite the strategic decision, wasn't it? My tactical analysis suggests you want me to {} something related to {} in the {} domain. Consider this mission accepted - though I must say, your briefing could use some work.", 
                processed_request.original_input,
                extract_action_from_parsed(&processed_request.processed_input),
                extract_topic_from_parsed(&processed_request.processed_input),
                extract_domain_from_parsed(&processed_request.processed_input)
            )
        },
        crate::llm::Mode::Tutor => {
            format!("Right, tutoring mode engaged. You want to learn about {} in {}? Excellent choice - knowledge is the finest weapon in any arsenal. I'll deploy my teaching protocols with surgical precision. {} level, I see - we'll calibrate accordingly.",
                extract_topic_from_parsed(&processed_request.processed_input),
                extract_domain_from_parsed(&processed_request.processed_input),
                format!("{:?}", processed_request.proficiency)
            )
        },
        crate::llm::Mode::WebSearch => {
            format!("Intelligence gathering mission detected. You need research on \"{}\"? My reconnaissance systems have parsed this as a {} operation in the {} sector, specifically targeting {}. Initiating search protocols - this should be quite the tactical exercise.",
                processed_request.original_input,
                extract_action_from_parsed(&processed_request.processed_input),
                extract_domain_from_parsed(&processed_request.processed_input),
                extract_topic_from_parsed(&processed_request.processed_input)
            )
        }
    };
    
    ErikaResponse {
        original_query: processed_request.original_input.clone(),
        parsed_analysis: processed_request.processed_input.clone(),
        erika_translation: fallback_translation,
        mode: format!("{:?}", processed_request.mode),
        proficiency: format!("{:?}", processed_request.proficiency),
    }
}

/// Utility functions to extract fields from parsed JSON
fn extract_action_from_parsed(parsed_json: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(parsed_json) {
        Ok(value) => value
            .get("action")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| "analyze".to_string()),
        Err(_) => "analyze".to_string(),
    }
}

fn extract_domain_from_parsed(parsed_json: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(parsed_json) {
        Ok(value) => value
            .get("domain")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| "general".to_string()),
        Err(_) => "general".to_string(),
    }
}

fn extract_topic_from_parsed(parsed_json: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(parsed_json) {
        Ok(value) => value
            .get("topic")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| "general topics".to_string()),
        Err(_) => "general topics".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receive::{Mode, Proficiency, Config};

    #[test]
    fn test_extract_functions() {
        let parsed_json = r#"{"action": "explain", "domain": "rust", "topic": "lifetimes"}"#;
        
        assert_eq!(extract_action_from_parsed(parsed_json), "explain");
        assert_eq!(extract_domain_from_parsed(parsed_json), "rust");
        assert_eq!(extract_topic_from_parsed(parsed_json), "lifetimes");
    }

    #[test]
    fn test_fallback_response_creation() {
        let config = Config {
            model: "test-model".to_string(),
            api_key: "test-key".to_string(),
            api_url: "test-url".to_string(),
            number_of_queries: 5,
            max_research_loops: 3,
        };
        
        let processed_request = ProcessedRequest {
            mode: Mode::Tutor,
            proficiency: Proficiency::Intermediate,
            original_input: "How do Rust lifetimes work?".to_string(),
            processed_input: r#"{"action": "explain", "domain": "rust", "topic": "lifetimes"}"#.to_string(),
            config,
        };
        
        let fallback = create_fallback_erika_response(&processed_request);
        assert!(fallback.erika_translation.contains("tutoring mode"));
        assert!(fallback.erika_translation.contains("lifetimes"));
        assert!(fallback.erika_translation.contains("rust"));
    }

    #[test]
    fn test_personality_loading() {
        let personality = load_erika_personality();
        assert!(personality.contains("Sharp-witted British-Russian tactician"));
        assert!(personality.contains("chess master meets MI6 operative"));
        assert!(personality.contains("Well, that was quite the strategic decision"));
    }
}
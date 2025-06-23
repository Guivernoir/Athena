use std::env;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use reqwest::Client;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub model: String,
    pub api_key: String,
    pub api_url: String,
    pub number_of_queries: i32,
    pub max_research_loops: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub mode: u32,
    pub proficiency: u32,
    pub input: String,
}

#[derive(Debug, Clone)]
pub enum Mode {
    Assistant,
    Tutor,
    WebSearch,
}

#[derive(Debug, Clone)]
pub enum Proficiency {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParsedInput {
    pub action: String,
    pub domain: String,
    pub topic: String,
    pub mode: Mode,
    pub proficiency: Proficiency,
    pub original_input: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessedRequest {
    pub mode: Mode,
    pub proficiency: Proficiency,
    pub original_input: String,
    pub processed_input: String,
    pub config: Config,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LLMRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LLMResponse {
    pub choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Choice {
    pub message: Message,
}

#[derive(Serialize, Deserialize, Debug)]
struct LLMParsedResponse {
    action: String,
    domain: String,
    topic: String,
}

impl Mode {
    pub fn from_u32(mode: u32) -> Result<Self, String> {
        match mode {
            0 => Ok(Mode::Assistant),
            1 => Ok(Mode::Tutor),
            2 => Ok(Mode::WebSearch),
            _ => Err("Invalid mode".to_string()),
        }
    }

    pub fn get_system_prompt(&self) -> String {
        let mode_context = match self {
            Mode::Assistant => "You are a helpful assistant.",
            Mode::Tutor => "You are an educational tutor.",
            Mode::WebSearch => "You are a research assistant with web search capabilities.",
        };
        mode_context.to_string()
    }
}

impl Proficiency {
    pub fn from_u32(proficiency: u32) -> Result<Self, String> {
        match proficiency {
            0 => Ok(Proficiency::Beginner),
            1 => Ok(Proficiency::Intermediate),
            2 => Ok(Proficiency::Advanced),
            3 => Ok(Proficiency::Expert),
            _ => Err("Invalid proficiency level".to_string()),
        }
    }

    pub fn get_system_prompt(&self) -> String {
        let proficiency_context = match self {
            Proficiency::Beginner => "The user is a beginner in this topic.",
            Proficiency::Intermediate => "The user has an intermediate understanding of this topic.",
            Proficiency::Advanced => "The user has an advanced understanding of this topic.",
            Proficiency::Expert => "The user is an expert in this topic.",
        };
        proficiency_context.to_string()
    }
}

impl Serialize for Mode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mode_str = match self {
            Mode::Assistant => "assistant",
            Mode::Tutor => "tutor",
            Mode::WebSearch => "web_search",
        };
        serializer.serialize_str(mode_str)
    }
}

impl<'de> Deserialize<'de> for Mode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "assistant" => Ok(Mode::Assistant),
            "tutor" => Ok(Mode::Tutor),
            "web_search" => Ok(Mode::WebSearch),
            _ => Err(serde::de::Error::custom("Invalid mode")),
        }
    }
}

impl Serialize for Proficiency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let proficiency_str = match self {
            Proficiency::Beginner => "beginner",
            Proficiency::Intermediate => "intermediate",
            Proficiency::Advanced => "advanced",
            Proficiency::Expert => "expert",
        };
        serializer.serialize_str(proficiency_str)
    }
}

impl<'de> Deserialize<'de> for Proficiency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "beginner" => Ok(Proficiency::Beginner),
            "intermediate" => Ok(Proficiency::Intermediate),
            "advanced" => Ok(Proficiency::Advanced),
            "expert" => Ok(Proficiency::Expert),
            _ => Err(serde::de::Error::custom("Invalid proficiency level")),
        }
    }
}

pub async fn load_config() -> Result<Config, String> {
    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| ".env".to_string());
    if let Ok(config_content) = std::fs::read_to_string(&config_path) {
        if let Ok(config) = serde_json::from_str::<Config>(&config_content) {
            return Ok(config);
        }
    }
    
    load_config_from_env().await
}

pub async fn load_config_from_env() -> Result<Config, String> {
    dotenv::dotenv().ok(); 
    
    let model = env::var("MODEL")
        .map_err(|_| "MODEL environment variable not set".to_string())?;
    let api_key = env::var("API_KEY")
        .map_err(|_| "API_KEY environment variable not set".to_string())?;
    let api_url = env::var("API_URL")
        .map_err(|_| "API_URL environment variable not set".to_string())?;
    let number_of_queries = env::var("NUMBER_OF_QUERIES")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<i32>()
        .map_err(|_| "Invalid NUMBER_OF_QUERIES format".to_string())?;
    let max_research_loops = env::var("MAX_RESEARCH_LOOPS")
        .unwrap_or_else(|_| "3".to_string())
        .parse::<u32>()
        .map_err(|_| "Invalid MAX_RESEARCH_LOOPS format".to_string())?;

    Ok(Config {
        model,
        api_key,
        api_url,
        number_of_queries,
        max_research_loops,
    })
}

pub async fn receive_input(input: String) -> Result<String, String> {
    if input.trim().is_empty() {
        return Err("Input cannot be empty".to_string());
    }
    
    let sanitized_input = input.trim().to_string();
    
    if sanitized_input.len() > 10000 {
        return Err("Input too long (max 10000 characters)".to_string());
    }
    
    Ok(sanitized_input)
}

pub async fn process_with_llm(
    input: &str,
    mode: &Mode,
    proficiency: &Proficiency,
    config: &Config,
) -> Result<ParsedInput, String> {
    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAY_MS: u64 = 1000;
    
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let system_prompt = format!(
        "You are an intelligent parsing assistant for a tutoring chatbot. Your task is to analyze user input and extract structured information.

Parse the following user input and return ONLY a valid JSON object with these exact fields:
- \"action\": The main action the user wants (e.g., \"explain\", \"teach\", \"help\", \"search\", \"analyze\", \"create\", \"debug\", \"review\", \"practice\", \"translate\")
- \"domain\": The subject area or programming/natural language (e.g., \"rust\", \"cpp\", \"python\", \"javascript\", \"italian\", \"spanish\", \"english\", \"mathematics\", \"general\")
- \"topic\": The specific topic within the domain (e.g., \"lifetimes\", \"borrowing\", \"syntax\", \"verbs\", \"grammar\", \"algorithms\", \"data structures\", \"functions\", \"variables\")

Context: {} {}

Rules:
1. Return ONLY valid JSON, no other text
2. If you cannot determine a field, use \"general\" or \"unknown\" as appropriate
3. Keep field values concise (1-3 words typically)
4. Use lowercase for consistency

Example output:
{{\"action\": \"explain\", \"domain\": \"rust\", \"topic\": \"lifetimes\"}}",
        mode.get_system_prompt(),
        proficiency.get_system_prompt()
    );
    
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: system_prompt,
        },
        Message {
            role: "user".to_string(),
            content: input.to_string(),
        },
    ];
    
    let request_body = LLMRequest {
        model: config.model.clone(),
        messages,
        max_tokens: Some(150), // Reduced since we only need JSON output
        temperature: Some(0.1), // Lower temperature for more consistent parsing
    };
    
    for attempt in 1..=MAX_RETRIES {
        match make_llm_request(&client, &config, &request_body).await {
            Ok(response_content) => {
                match parse_llm_response(&response_content, input, mode, proficiency) {
                    Ok(parsed_input) => return Ok(parsed_input),
                    Err(parse_error) => {
                        if attempt == MAX_RETRIES {
                            // On final attempt, return a fallback response
                            return Ok(create_fallback_response(input, mode, proficiency));
                        }
                        eprintln!("Parse attempt {}/{} failed: {}", attempt, MAX_RETRIES, parse_error);
                    }
                }
            }
            Err(request_error) => {
                if attempt == MAX_RETRIES {
                    return Err(format!("All {} attempts failed. Last error: {}", MAX_RETRIES, request_error));
                }
                eprintln!("Request attempt {}/{} failed: {}", attempt, MAX_RETRIES, request_error);
            }
        }
        
        // Wait before retrying
        tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS * attempt as u64)).await;
    }
    
    // This should never be reached due to the logic above, but just in case
    Err("Unexpected error in retry logic".to_string())
}

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

fn parse_llm_response(
    response_content: &str,
    original_input: &str,
    mode: &Mode,
    proficiency: &Proficiency,
) -> Result<ParsedInput, String> {
    // Try to extract JSON from the response
    let json_str = extract_json_from_response(response_content)?;
    
    let parsed_response: LLMParsedResponse = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse LLM JSON response: {}", e))?;
    
    // Validate and sanitize the parsed fields
    let action = sanitize_field(&parsed_response.action, "general");
    let domain = sanitize_field(&parsed_response.domain, "general");
    let topic = sanitize_field(&parsed_response.topic, "general");
    
    Ok(ParsedInput {
        action,
        domain,
        topic,
        mode: mode.clone(),
        proficiency: proficiency.clone(),
        original_input: original_input.to_string(),
    })
}

fn extract_json_from_response(response: &str) -> Result<String, String> {
    let response = response.trim();
    
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            if start <= end {
                return Ok(response[start..=end].to_string());
            }
        }
    }
    
    Err(format!("No valid JSON object found in response: {}", response))
}

fn sanitize_field(field: &str, default: &str) -> String {
    let sanitized = field.trim().to_lowercase();
    if sanitized.is_empty() || sanitized == "unknown" || sanitized == "null" {
        default.to_string()
    } else {
        sanitized.chars().take(50).filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-').collect()
    }
}

fn create_fallback_response(
    input: &str,
    mode: &Mode,
    proficiency: &Proficiency,
) -> ParsedInput {
    let input_lower = input.to_lowercase();
    
    let action = if input_lower.contains("explain") || input_lower.contains("what") || input_lower.contains("how") {
        "explain"
    } else if input_lower.contains("help") || input_lower.contains("assist") {
        "help"
    } else if input_lower.contains("search") || input_lower.contains("find") {
        "search"
    } else if input_lower.contains("teach") || input_lower.contains("learn") {
        "teach"
    } else {
        "general"
    };
    
    let domain = if input_lower.contains("rust") || input_lower.contains("rs") {
        "rust"
    } else if input_lower.contains("cpp") || input_lower.contains("c++") {
        "cpp"
    } else if input_lower.contains("python") {
        "python"
    } else if input_lower.contains("javascript") || input_lower.contains("js") {
        "javascript"
    } else if input_lower.contains("elixir") || input_lower.contains("ex") {
        "elixir"
    } else if input_lower.contains("julia") || input_lower.contains("jl") {
        "julia"
    } else if input_lower.contains("react") || input_lower.contains("jsx") {
        "react"
    } else if input_lower.contains("italian") {
        "italian"
    } else if input_lower.contains("spanish") {
        "spanish"
    } else if input_lower.contains("english") {
        "english"
    } else if input_lower.contains("russian") {
        "russian"
    } else if input_lower.contains("german") {
        "german"
    } else {
        "general"
    };
    
    let topic = if input_lower.contains("lifetime") {
        "lifetimes"
    } else if input_lower.contains("borrow") {
        "borrowing"
    } else if input_lower.contains("syntax") {
        "syntax"
    } else if input_lower.contains("verb") {
        "verbs"
    } else if input_lower.contains("grammar") {
        "grammar"
    } else {
        "general"
    };
    
    ParsedInput {
        action: action.to_string(),
        domain: domain.to_string(),
        topic: topic.to_string(),
        mode: mode.clone(),
        proficiency: proficiency.clone(),
        original_input: input.to_string(),
    }
}

pub async fn process_request(data: Data) -> Result<ProcessedRequest, String> {
    let config = load_config().await?;
    let mode = Mode::from_u32(data.mode)?;
    let proficiency = Proficiency::from_u32(data.proficiency)?;
    let original_input = receive_input(data.input).await?;
    let processed_input = process_with_llm(&original_input, &mode, &proficiency, &config).await?;
    
    let processed_input_string = serde_json::to_string(&processed_input)
        .map_err(|e| format!("Failed to serialize parsed input: {}", e))?;
    
    Ok(ProcessedRequest {
        mode,
        proficiency,
        original_input,
        processed_input: processed_input_string,
        config,
    })
}

pub async fn handle_frontend_request(json_data: &str) -> Result<String, String> {
    let data: Data = serde_json::from_str(json_data)
        .map_err(|e| format!("Failed to parse JSON input: {}", e))?;
    
    let processed_request = process_request(data).await?;
    
    serde_json::to_string(&processed_request)
        .map_err(|e| format!("Failed to serialize processed request: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_conversion() {
        assert!(matches!(Mode::from_u32(0), Ok(Mode::Assistant)));
        assert!(matches!(Mode::from_u32(1), Ok(Mode::Tutor)));
        assert!(matches!(Mode::from_u32(2), Ok(Mode::WebSearch)));
        assert!(Mode::from_u32(3).is_err());
    }

    #[test]
    fn test_proficiency_conversion() {
        assert!(matches!(Proficiency::from_u32(0), Ok(Proficiency::Beginner)));
        assert!(matches!(Proficiency::from_u32(1), Ok(Proficiency::Intermediate)));
        assert!(matches!(Proficiency::from_u32(2), Ok(Proficiency::Advanced)));
        assert!(matches!(Proficiency::from_u32(3), Ok(Proficiency::Expert)));
        assert!(Proficiency::from_u32(4).is_err());
    }

    #[tokio::test]
    async fn test_input_validation() {
        assert!(receive_input("valid input".to_string()).await.is_ok());
        assert!(receive_input("".to_string()).await.is_err());
        assert!(receive_input("   ".to_string()).await.is_err());
    }

    #[test]
    fn test_extract_json_from_response() {
        let response1 = r#"{"action": "explain", "domain": "rust", "topic": "lifetimes"}"#;
        assert!(extract_json_from_response(response1).is_ok());
        
        let response2 = r#"Here's the parsed result: {"action": "help", "domain": "python", "topic": "syntax"} - done!"#;
        assert!(extract_json_from_response(response2).is_ok());
        
        let response3 = "No JSON here";
        assert!(extract_json_from_response(response3).is_err());
    }

    #[test]
    fn test_sanitize_field() {
        assert_eq!(sanitize_field("  RUST  ", "default"), "rust");
        assert_eq!(sanitize_field("", "default"), "default");
        assert_eq!(sanitize_field("unknown", "default"), "default");
        assert_eq!(sanitize_field("C++/C#", "default"), "cc");
    }

    #[test]
    fn test_fallback_response() {
        let mode = Mode::Tutor;
        let proficiency = Proficiency::Beginner;
        
        let result = create_fallback_response("How do lifetimes work in Rust?", &mode, &proficiency);
        assert_eq!(result.action, "explain");
        assert_eq!(result.domain, "rust");
        assert_eq!(result.topic, "lifetimes");
    }
}
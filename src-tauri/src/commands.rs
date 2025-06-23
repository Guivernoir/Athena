use tauri::command;
use crate::llm::{self, Data, Mode, Proficiency};
use serde_json;

#[tauri::command]
pub async fn receive_input(input: String) -> Result<String, String> {
    llm::receive_input(input).await
}

#[tauri::command]
pub async fn receive_mode(mode: u32) -> Result<String, String> {
    Mode::from_u32(mode)
        .map(|m| format!("{:?}", m))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn receive_proficiency(proficiency: u32) -> Result<String, String> {
    Proficiency::from_u32(proficiency)
        .map(|p| format!("{:?}", p))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn send_output(mode: u32, proficiency: u32, input: String) -> Result<String, String> {
    let data = Data {
        mode,
        proficiency,
        input,
    };
    
    let json_data = serde_json::to_string(&data)
        .map_err(|e| format!("Failed to serialize input data: {}", e))?;
    
    llm::process_complete_request(&json_data).await
}
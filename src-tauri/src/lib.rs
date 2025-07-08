// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod commands;
pub mod preprocessing;
pub mod llama;
pub mod llm;

use crate::commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            receive_input,
            receive_mode,
            receive_proficiency,
            send_output
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

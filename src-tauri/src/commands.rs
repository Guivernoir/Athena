use crate::preprocessing::{
    router::{Mode, Proficiency, Personality},
    Preprocessor, FormattedInput,
};
use crate::postprocessing::PostProcessor;
use crate::llama::{LLMEngine, Language};
use tauri::command;
use std::sync::Mutex;

// Global state now only keeps the latest formatted payload
#[derive(Default)]
pub struct AppState {
    pub latest: Option<FormattedInput>,
}

/* ---------- 1.  SETTERS ---------- */

#[command]
pub async fn receive_mode(
    mode: u8,
) -> Result<String, String> {
    Mode::select_mode(mode).await?;
    Ok("Mode stored".to_string())
}

#[command]
pub async fn receive_proficiency(
    proficiency: u8,
) -> Result<String, String> {
    Proficiency::select_proficiency(proficiency).await?;
    Ok("Proficiency stored".to_string())
}

#[command]
pub async fn receive_personality(
    personality: u8,
) -> Result<String, String> {
    Personality::select_personality(personality).await?;
    Ok("Personality stored".to_string())
}

/* ---------- 2.  MAIN PIPELINE ---------- */

#[command]
pub async fn send_output(
    input: String,
    mode: u8,
    proficiency: u8,
    personality: u8,
    language: Language,
    llm: tauri::State<'_, Mutex<LLMEngine>>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    // 1. Convert enums
    let mode_enum      = Mode::select_mode(mode).await?;
    let prof_enum      = Proficiency::select_proficiency(proficiency).await?;
    let pers_enum      = Personality::select_personality(personality).await?;

    // 2. Run the **single** preprocessing step
    let llm_guard = llm.lock().unwrap();
    let formatted = Preprocessor::process(
        input,
        mode_enum,
        prof_enum,
        pers_enum,
        language,
        &llm_guard,
    )
    .await
    .map_err(|e| e.to_string())?;

    // 3. Cache for later retrieval
    state.lock().unwrap().latest = Some(formatted.clone());

    // 4. Post-process with persona flavor
    let post = PostProcessor::new(llm_guard.clone());
    let output = post
        .process(
            formatted.context.raw_input.clone(),
            pers_enum,
            mode_enum,
            prof_enum,
        )
        .await;

    Ok(output)
}
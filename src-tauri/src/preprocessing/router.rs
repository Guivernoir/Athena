use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Mode{
    Tutor,
    Assistant
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Proficiency{
    Beginner,
    Intermediate,
    Advanced,
    Expert
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Personality{
    Erika,
    Aurora,
    Ekaterina,
    Viktor
}

impl Mode{
    pub async fn select_mode(mode: u32) -> Result<Self, String> {
        match mode {
            0 => Ok(Mode::Tutor),
            1 => Ok(Mode::Assistant),
            _ => Err("Invalid mode selected!".to_string())
        }
    }
}

impl Proficiency{
    pub async fn select_proficiency(proficiency: u32) -> Result<Self, String>{
        match proficiency {
            0 => Ok(Proficiency::Beginner),
            1 => Ok(Proficiency::Intermediate),
            2 => Ok(Proficiency::Advanced),
            3 => Ok(Proficiency::Expert),
            _ => Err("Invalid proficiency selected!".to_string())
        }
    }
}

impl Personality{
    pub async fn select_personality(personality: u32) -> Result<Self, String>{
        match personality{
            0 => Ok(Personality::Erika),
            1 => Ok(Personality::Aurora),
            2 => Ok(Personality::Ekaterina),
            3 => Ok(Personality::Viktor),
            _ => Err("Invalid personality selected!".to_string())
        }
    }
}
use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub transcription_mode: String, // "cloud" or "local"
    pub api_provider: String,       // "gemini" or "openai"
    pub api_key: String,
    pub model_name: String,         // "small", "base", etc.
    pub hotkey: String,             // "Alt+N"
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            transcription_mode: "cloud".to_string(),
            api_provider: "gemini".to_string(),
            api_key: "".to_string(),
            model_name: "base".to_string(),
            hotkey: "Alt+N".to_string(),
        }
    }
}

pub fn get_settings_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let config_dir = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get app config dir: {}", e))?;
    Ok(config_dir.join("settings.json"))
}

pub fn load_settings(app_handle: &tauri::AppHandle) -> Result<Settings, String> {
    let path = get_settings_path(app_handle)?;
    if !path.exists() {
        return Ok(Settings::default());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;
    let settings: Settings = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Warning: Failed to parse settings JSON: {}. Falling back to default settings.", e);
            Settings::default()
        }
    };
    Ok(settings)
}

pub fn save_settings(app_handle: &tauri::AppHandle, settings: &Settings) -> Result<(), String> {
    let path = get_settings_path(app_handle)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create settings directory: {}", e))?;
    }
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;
    Ok(())
}

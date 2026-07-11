use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    pub transcription_mode: String, // "cloud" or "local"
    pub api_provider: String,       // "gemini" or "openai"
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub api_key_gemini: String,
    #[serde(default)]
    pub api_key_openai: String,
    #[serde(default)]
    pub api_key_groq: String,
    pub model_name: String,         // "small", "base", etc.
    pub hotkey: String,             // "Alt+N"
    pub streaming_enabled: bool,    // whether to type text on-the-fly or paste on release (v1.0 mode)
    pub toggle_enabled: bool,       // short tap latches recording until the next tap
    pub language: String,           // "auto" | "layout" | "ru" | "en" | "de" | "es" | "fr" | "it" | "zh" | "pt" | "tr" (dictation language bias/hint)
    pub dictionary: String,         // comma-separated custom terms passed to the recognizer as hints
    pub voice_punctuation: bool,    // convert spoken commands ("запятая", "новая строка") to punctuation
    pub autostart: bool,            // launch the app on Windows startup
    pub overlay_sounds: bool,
    pub overlay_sound_theme: String,
    pub overlay_sound_volume: f32,
    pub cloud_fallback_enabled: bool, // auto-retry locally when cloud is unreachable (VPN/network/region block)
    pub local_engine: String,         // "whisper" or "parakeet"
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            transcription_mode: "cloud".to_string(),
            api_provider: "gemini".to_string(),
            api_key: "".to_string(),
            api_key_gemini: "".to_string(),
            api_key_openai: "".to_string(),
            api_key_groq: "".to_string(),
            model_name: "base".to_string(),
            hotkey: "Alt+V".to_string(),
            streaming_enabled: false,
            toggle_enabled: false,
            language: "auto".to_string(),
            dictionary: "".to_string(),
            voice_punctuation: false,
            autostart: false,
            overlay_sounds: true,
            overlay_sound_theme: "zen".to_string(),
            overlay_sound_volume: 0.8,
            cloud_fallback_enabled: true,
            local_engine: "whisper".to_string(),
        }
    }
}

impl Settings {
    pub fn migrate_legacy_keys(&mut self) {
        if !self.api_key.is_empty() {
            match self.api_provider.as_str() {
                "gemini" if self.api_key_gemini.is_empty() => {
                    self.api_key_gemini = self.api_key.clone();
                }
                "openai" if self.api_key_openai.is_empty() => {
                    self.api_key_openai = self.api_key.clone();
                }
                "groq" if self.api_key_groq.is_empty() => {
                    self.api_key_groq = self.api_key.clone();
                }
                _ => {}
            }
        }
    }
}

pub fn get_settings_path<R: tauri::Runtime>(app_handle: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    let config_dir = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get app config dir: {}", e))?;
    Ok(config_dir.join("settings.json"))
}

pub fn load_settings<R: tauri::Runtime>(app_handle: &tauri::AppHandle<R>) -> Result<Settings, String> {
    let path = get_settings_path(app_handle)?;
    if !path.exists() {
        return Ok(Settings::default());
    }
    
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;
        
    let mut settings: Settings = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Warning: Failed to parse settings JSON: {}. Falling back to default settings.", e);
            Settings::default()
        }
    };
    
    settings.migrate_legacy_keys();
    
    settings.api_key = match settings.api_provider.as_str() {
        "gemini" => settings.api_key_gemini.clone(),
        "openai" => settings.api_key_openai.clone(),
        "groq" => settings.api_key_groq.clone(),
        _ => settings.api_key.clone(),
    };
    
    Ok(settings)
}

pub fn save_settings<R: tauri::Runtime>(app_handle: &tauri::AppHandle<R>, settings: &Settings) -> Result<(), String> {
    let mut settings_clone = settings.clone();
    
    // Trim API keys before saving to prevent trailing newlines
    settings_clone.api_key_gemini = settings_clone.api_key_gemini.trim().to_string();
    settings_clone.api_key_openai = settings_clone.api_key_openai.trim().to_string();
    settings_clone.api_key_groq = settings_clone.api_key_groq.trim().to_string();
    
    settings_clone.api_key = match settings_clone.api_provider.as_str() {
        "gemini" => settings_clone.api_key_gemini.clone(),
        "openai" => settings_clone.api_key_openai.clone(),
        "groq" => settings_clone.api_key_groq.clone(),
        _ => settings_clone.api_key.clone(),
    };
    
    let path = get_settings_path(app_handle)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create settings directory: {}", e))?;
    }
    
    let content = serde_json::to_string_pretty(&settings_clone)
        .map_err(|e| format!("Failed to format settings JSON: {}", e))?;
        
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;
        
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_defaults() {
        let settings = Settings::default();
        assert_eq!(settings.api_key, "");
        assert_eq!(settings.api_key_gemini, "");
        assert_eq!(settings.api_key_openai, "");
        assert_eq!(settings.api_key_groq, "");
        assert_eq!(settings.transcription_mode, "cloud");
        assert_eq!(settings.api_provider, "gemini");
        assert_eq!(settings.overlay_sounds, true);
        assert_eq!(settings.overlay_sound_theme, "zen");
        assert_eq!(settings.overlay_sound_volume, 0.8);
        assert_eq!(settings.local_engine, "whisper");
    }

    #[test]
    fn test_legacy_settings_deserialization() {
        let json = r#"{
            "transcription_mode": "cloud",
            "api_provider": "gemini",
            "api_key": "old-key-123",
            "model_name": "base",
            "hotkey": "Alt+V",
            "streaming_enabled": false,
            "toggle_enabled": false,
            "language": "auto",
            "dictionary": "",
            "voice_punctuation": false,
            "autostart": false
        }"#;

        let mut settings: Settings = serde_json::from_str(json).unwrap();
        
        assert_eq!(settings.api_key_gemini, "");
        assert_eq!(settings.api_key_openai, "");
        assert_eq!(settings.api_key_groq, "");

        settings.migrate_legacy_keys();

        assert_eq!(settings.api_key_gemini, "old-key-123");
        assert_eq!(settings.api_key_openai, "");
        assert_eq!(settings.api_key_groq, "");
    }


    #[test]
    fn test_save_settings_sync_logic() {
        let mut settings = Settings::default();
        settings.api_provider = "openai".to_string();
        settings.api_key_openai = "openai-key".to_string();
        settings.api_key_gemini = "gemini-key".to_string();

        let mut settings_clone = settings.clone();
        settings_clone.api_key = match settings_clone.api_provider.as_str() {
            "gemini" => settings_clone.api_key_gemini.clone(),
            "openai" => settings_clone.api_key_openai.clone(),
            "groq" => settings_clone.api_key_groq.clone(),
            _ => settings_clone.api_key.clone(),
        };

        assert_eq!(settings_clone.api_key, "openai-key");

        settings.api_provider = "gemini".to_string();
        let mut settings_clone2 = settings.clone();
        settings_clone2.api_key = match settings_clone2.api_provider.as_str() {
            "gemini" => settings_clone2.api_key_gemini.clone(),
            "openai" => settings_clone2.api_key_openai.clone(),
            "groq" => settings_clone2.api_key_groq.clone(),
            _ => settings_clone2.api_key.clone(),
        };
        assert_eq!(settings_clone2.api_key, "gemini-key");
    }
}

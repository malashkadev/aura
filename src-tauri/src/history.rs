use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use tauri::Manager;

const MAX_ENTRIES: usize = 50;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryEntry {
    pub text: String,
    pub timestamp_ms: u64,
    pub mode: String, // "cloud" or "local"
}

fn get_history_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {}", e))?;
    Ok(data_dir.join("history.json"))
}

pub fn load_history(app_handle: &tauri::AppHandle) -> Result<Vec<HistoryEntry>, String> {
    let path = get_history_path(app_handle)?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read history file: {}", e))?;
    match serde_json::from_str(&content) {
        Ok(entries) => Ok(entries),
        Err(e) => {
            eprintln!("Warning: Failed to parse history JSON: {}. Starting with empty history.", e);
            Ok(Vec::new())
        }
    }
}

fn save_history(app_handle: &tauri::AppHandle, entries: &[HistoryEntry]) -> Result<(), String> {
    let path = get_history_path(app_handle)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create history directory: {}", e))?;
    }
    let content = serde_json::to_string_pretty(entries)
        .map_err(|e| format!("Failed to serialize history: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write history file: {}", e))
}

/// Appends a transcription to the history file, keeping only the most recent entries.
pub fn add_entry(app_handle: &tauri::AppHandle, text: &str, mode: &str) -> Result<(), String> {
    let mut entries = load_history(app_handle)?;
    let timestamp_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    entries.insert(0, HistoryEntry {
        text: text.to_string(),
        timestamp_ms,
        mode: mode.to_string(),
    });
    entries.truncate(MAX_ENTRIES);
    save_history(app_handle, &entries)
}

pub fn clear_history(app_handle: &tauri::AppHandle) -> Result<(), String> {
    save_history(app_handle, &[])
}

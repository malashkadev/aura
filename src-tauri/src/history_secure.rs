use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};
use tauri::Manager;
use zeroize::Zeroize;

use crate::secure_storage;

const MAX_ENTRIES: usize = 300;
const MAX_ENTRY_CHARS: usize = 100_000;
static HISTORY_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HistoryEntry {
    pub text: String,
    pub timestamp_ms: u64,
    pub mode: String,
    /// Local recognition engine ("whisper" or "parakeet") that produced the
    /// text. None for cloud entries; absent in entries written by old builds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    /// Speech-to-text conversion time in milliseconds (local runs only).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub processing_ms: u64,
}

fn is_zero(value: &u64) -> bool {
    *value == 0
}

fn history_lock() -> &'static Mutex<()> {
    HISTORY_LOCK.get_or_init(|| Mutex::new(()))
}

fn lock_history() -> MutexGuard<'static, ()> {
    match history_lock().lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            crate::logger::log(
                "ERROR",
                "History",
                None,
                "Recovering poisoned history mutex",
            );
            poisoned.into_inner()
        }
    }
}

fn get_history_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|error| format!("Failed to get app local data directory: {error}"))?;
    Ok(data_dir.join("history.dpapi"))
}

fn get_legacy_history_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|error| format!("Failed to get app local data directory: {error}"))?;
    Ok(data_dir.join("history.json"))
}

fn encode_entries(entries: &[HistoryEntry]) -> Result<Vec<u8>, String> {
    let mut plaintext = serde_json::to_vec(entries)
        .map_err(|error| format!("Failed to serialize history: {error}"))?;
    let result = secure_storage::protect_for_current_user(&plaintext);
    plaintext.zeroize();
    result
}

fn decode_entries(ciphertext: &[u8]) -> Result<Vec<HistoryEntry>, String> {
    let mut plaintext = secure_storage::unprotect_for_current_user(ciphertext)?;
    let result = serde_json::from_slice(&plaintext)
        .map_err(|error| format!("Encrypted history file is invalid: {error}"));
    plaintext.zeroize();
    result
}

fn read_encrypted(path: &Path) -> Result<Vec<HistoryEntry>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let ciphertext =
        fs::read(path).map_err(|error| format!("Failed to read encrypted history: {error}"))?;
    decode_entries(&ciphertext).map_err(|error| {
        let backup = path.with_extension(format!("corrupt-{}", timestamp_millis()));
        let _ = fs::rename(path, &backup);
        format!(
            "History could not be decrypted and was moved to {}: {error}",
            backup.display()
        )
    })
}

fn migrate_legacy_if_needed(
    encrypted_path: &Path,
    legacy_path: &Path,
) -> Result<Option<Vec<HistoryEntry>>, String> {
    if encrypted_path.exists() || !legacy_path.exists() {
        return Ok(None);
    }
    let mut plaintext =
        fs::read(legacy_path).map_err(|error| format!("Failed to read legacy history: {error}"))?;
    let parsed: Result<Vec<HistoryEntry>, String> =
        serde_json::from_slice(&plaintext).map_err(|error| {
            let backup = legacy_path.with_extension(format!("corrupt-{}", timestamp_millis()));
            let _ = fs::rename(legacy_path, &backup);
            format!(
                "Legacy history was invalid and moved to {}: {error}",
                backup.display()
            )
        });
    plaintext.zeroize();
    let entries = parsed?;
    let encrypted = encode_entries(&entries)?;
    secure_storage::atomic_write(encrypted_path, &encrypted)?;
    fs::remove_file(legacy_path).map_err(|error| {
        format!("Encrypted history was saved but legacy cleanup failed: {error}")
    })?;
    Ok(Some(entries))
}

fn timestamp_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn load_history_unlocked(app_handle: &tauri::AppHandle) -> Result<Vec<HistoryEntry>, String> {
    let encrypted_path = get_history_path(app_handle)?;
    let legacy_path = get_legacy_history_path(app_handle)?;
    if let Some(entries) = migrate_legacy_if_needed(&encrypted_path, &legacy_path)? {
        return Ok(entries);
    }
    read_encrypted(&encrypted_path)
}

fn save_history_unlocked(
    app_handle: &tauri::AppHandle,
    entries: &[HistoryEntry],
) -> Result<(), String> {
    let encrypted = encode_entries(entries)?;
    secure_storage::atomic_write(&get_history_path(app_handle)?, &encrypted)
}

pub fn load_history(app_handle: &tauri::AppHandle) -> Result<Vec<HistoryEntry>, String> {
    let _guard = lock_history();
    load_history_unlocked(app_handle)
}

fn insert_entry(
    entries: &mut Vec<HistoryEntry>,
    text: &str,
    mode: &str,
    engine: Option<&str>,
    processing_ms: u64,
    timestamp_ms: u64,
) {
    entries.insert(
        0,
        HistoryEntry {
            text: text.to_string(),
            timestamp_ms,
            mode: mode.to_string(),
            engine: engine.map(|value| value.to_string()),
            processing_ms,
        },
    );
    entries.truncate(MAX_ENTRIES);
}

pub fn add_entry(
    app_handle: &tauri::AppHandle,
    text: &str,
    mode: &str,
    engine: Option<&str>,
    processing_ms: u64,
) -> Result<(), String> {
    if text.chars().count() > MAX_ENTRY_CHARS {
        return Err(format!(
            "History entry exceeds {MAX_ENTRY_CHARS} characters"
        ));
    }
    let _guard = lock_history();
    let mut entries = load_history_unlocked(app_handle)?;
    let timestamp_ms = timestamp_millis() as u64;
    insert_entry(
        &mut entries,
        text,
        mode,
        engine,
        processing_ms,
        timestamp_ms,
    );
    save_history_unlocked(app_handle, &entries)
}

pub fn clear_history(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let _guard = lock_history();
    save_history_unlocked(app_handle, &[])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_entry_is_newest_first_and_bounded() {
        let mut entries = Vec::new();
        for index in 0..(MAX_ENTRIES + 5) {
            insert_entry(
                &mut entries,
                &format!("entry-{index}"),
                "local",
                Some("whisper"),
                123,
                index as u64,
            );
        }
        assert_eq!(entries.len(), MAX_ENTRIES);
        assert_eq!(entries[0].text, format!("entry-{}", MAX_ENTRIES + 4));
        assert_eq!(entries.last().unwrap().text, "entry-5");
        assert_eq!(entries[0].engine.as_deref(), Some("whisper"));
        assert_eq!(entries[0].processing_ms, 123);
    }

    #[test]
    fn insert_entry_without_engine_keeps_fields_empty() {
        let mut entries = Vec::new();
        insert_entry(&mut entries, "text", "cloud", None, 0, 7);
        assert_eq!(entries[0].engine, None);
        assert_eq!(entries[0].processing_ms, 0);
    }

    #[test]
    fn old_entry_without_new_fields_decodes() {
        let legacy = r#"{"text":"old","timestamp_ms":1,"mode":"local"}"#;
        let entry: HistoryEntry = serde_json::from_str(legacy).unwrap();
        assert_eq!(entry.engine, None);
        assert_eq!(entry.processing_ms, 0);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn encrypted_history_round_trip_contains_no_plaintext() {
        let entries = vec![HistoryEntry {
            text: "private dictation".to_string(),
            timestamp_ms: 42,
            mode: "local".to_string(),
            engine: Some("whisper".to_string()),
            processing_ms: 500,
        }];
        let encoded = encode_entries(&entries).unwrap();
        assert!(!encoded
            .windows("private dictation".len())
            .any(|window| window == b"private dictation"));
        assert_eq!(decode_entries(&encoded).unwrap(), entries);
    }
}

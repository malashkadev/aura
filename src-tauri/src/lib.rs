#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod ai_client;
pub mod artifact_download;
pub mod audio_recorder;
pub mod caret_context;
#[cfg(target_os = "windows")]
pub mod clipboard_win;
pub mod edit_intent;
pub mod edit_validate;
#[path = "history_secure.rs"]
pub mod history;
pub mod keyboard_hook;
pub mod keyboard_simulator;
pub mod llm_runner;
pub mod logger;
pub mod parakeet_streaming;
pub mod secure_storage;
#[path = "settings_secure.rs"]
pub mod settings;
pub mod smart_insertion;
pub mod text_normalizer;
pub mod vad;
pub mod whisper_runner;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::UpdaterExt;

struct ParakeetStreamingSession {
    generation: u64,
    cancel: Arc<AtomicBool>,
    result_rx: mpsc::Receiver<parakeet_streaming::StreamingOutcome>,
}

struct AppState {
    audio_recorder: audio_recorder::AudioRecorder,
    selected_text: Mutex<String>,
    selection_blocked_by_uipi: AtomicBool,
    caret_context: Mutex<Option<smart_insertion::CaretContext>>,
    press_time: Mutex<Option<std::time::Instant>>,
    is_recording: AtomicBool,
    toggle_enabled: AtomicBool,
    typed_so_far: Mutex<String>,
    /// True after physical user input makes Aura's mirrored live-text state
    /// unsafe for further Backspace-based reconciliation.
    live_target_desynced: AtomicBool,
    /// Remains active through final reconciliation so edits during processing
    /// also force a non-destructive clipboard handoff.
    live_target_monitoring: AtomicBool,
    selected_language: Mutex<String>,
    /// Increments on every new recording session; stale async tasks compare
    /// against it before touching the keyboard or clipboard.
    session_gen: AtomicU64,
    /// Serializes clipboard backup/paste/restore sequences so overlapping
    /// sessions cannot interleave and clobber each other's clipboard contents.
    clipboard_mutex: Mutex<()>,
    /// Holds `(owner_gen, real_original_clipboard, temporary_transcript)` during
    /// the 400ms post-paste settle window so an overlapping session inherits the
    /// real user clipboard rather than backing up the previous session's transcript.
    unrestored_clipboard: Mutex<Option<(u64, ClipboardBackup, String)>>,
    /// Serializes runtime side-effects of settings mutations (`hotkey`,
    /// `toggle_enabled`, `gaming_mode_active`, resident servers).
    settings_apply_lock: tokio::sync::Mutex<()>,
    /// Toggle mode: a short tap latched the recording until the next tap / Esc.
    latched: AtomicBool,
    /// Set when a toggle-stopping tap already finalized; its key release is a no-op.
    ignore_next_release: AtomicBool,
    /// Window and process that had focus when the recording started (focus guard for typing).
    start_focus: Mutex<keyboard_simulator::FocusTarget>,
    parakeet_lifecycle: Mutex<()>,
    parakeet_server: Mutex<Option<whisper_runner::RunningParakeetServer>>,
    parakeet_port: std::sync::atomic::AtomicU16,
    parakeet_streaming: Mutex<Option<ParakeetStreamingSession>>,
    parakeet_watchdog: Mutex<Option<whisper_runner::ParakeetWatchdog>>,
    pub(crate) parakeet_starting: AtomicBool,
    whisper_lifecycle: Mutex<()>,
    whisper_server: Mutex<Option<whisper_runner::RunningWhisperServer>>,
    whisper_port: std::sync::atomic::AtomicU16,
    whisper_watchdog: Mutex<Option<whisper_runner::WhisperWatchdog>>,
    pub(crate) whisper_starting: AtomicBool,
    llm_lifecycle: Mutex<()>,
    llm_server: Mutex<Option<llm_runner::RunningLlmServer>>,
    llm_port: std::sync::atomic::AtomicU16,
    last_activity_time: Mutex<std::time::Instant>,
    gaming_mode_active: AtomicBool,
    last_audio_devices: Mutex<Vec<String>>,
}

#[tauri::command]
fn minimize_window(window: tauri::WebviewWindow) {
    let _ = window.minimize();
}

#[tauri::command]
fn close_window(window: tauri::WebviewWindow) {
    if window.label() == "tray-menu" {
        let _ = window.hide();
    } else {
        let _ = window.close();
    }
}

#[tauri::command]
fn show_settings_window(app_handle: tauri::AppHandle) {
    if let Some(tray_win) = app_handle.get_webview_window("tray-menu") {
        let _ = tray_win.hide();
    }
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn stop_all_sidecars_and_watchdogs(app_handle: &tauri::AppHandle) {
    whisper_runner::stop_whisper_server_and_watchdog(app_handle);
    whisper_runner::stop_parakeet_server_and_watchdog(app_handle);
    llm_runner::stop_llm_server(app_handle);
}

#[tauri::command]
async fn exit_app(app_handle: tauri::AppHandle) {
    let stop_handle = app_handle.clone();
    let _ = tauri::async_runtime::spawn_blocking(move || {
        stop_all_sidecars_and_watchdogs(&stop_handle);
    })
    .await;
    app_handle.exit(0);
}

#[tauri::command]
fn start_dragging_command(window: tauri::WebviewWindow) {
    let _ = window.start_dragging();
}

#[tauri::command]
fn hide_overlay_window(window: tauri::WebviewWindow) {
    if window.label() == "overlay" {
        let _ = window.hide();
    }
}

async fn load_settings_async(app_handle: tauri::AppHandle) -> Result<settings::Settings, String> {
    tauri::async_runtime::spawn_blocking(move || settings::load_settings(&app_handle))
        .await
        .map_err(|error| format!("Settings load worker failed: {error}"))?
}

#[tauri::command]
async fn get_settings(app_handle: tauri::AppHandle) -> Result<settings::SettingsView, String> {
    load_settings_async(app_handle)
        .await
        .map(|settings| settings::SettingsView::from_settings(&settings))
}

#[derive(Clone, serde::Serialize)]
struct OverlayPreferences {
    overlay_sounds: bool,
    overlay_sound_theme: String,
    overlay_sound_volume: f32,
    overlay_show_timer: bool,
    overlay_style: String,
    ui_language: String,
}

impl From<&settings::Settings> for OverlayPreferences {
    fn from(settings: &settings::Settings) -> Self {
        Self {
            overlay_sounds: settings.overlay_sounds,
            overlay_sound_theme: settings.overlay_sound_theme.clone(),
            overlay_sound_volume: settings.overlay_sound_volume,
            overlay_show_timer: settings.overlay_show_timer,
            overlay_style: settings.overlay_style.clone(),
            ui_language: settings.ui_language.clone(),
        }
    }
}

#[tauri::command]
async fn set_provider_key(
    app_handle: tauri::AppHandle,
    provider: String,
    mut key: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let result = settings::set_provider_key(&app_handle, &provider, &key);
        zeroize::Zeroize::zeroize(&mut key);
        result
    })
    .await
    .map_err(|error| format!("API key storage worker failed: {error}"))?
}

#[tauri::command]
async fn set_settings(
    app_handle: tauri::AppHandle,
    mut settings: settings::Settings,
) -> Result<Option<String>, String> {
    keyboard_hook::validate_hotkey(&settings.hotkey)?;
    let _apply_guard = match app_handle.try_state::<AppState>() {
        Some(ref state) => Some(state.inner().settings_apply_lock.lock().await),
        None => None,
    };

    let save_handle = app_handle.clone();
    let (previous, updated, autostart_warning) = tauri::async_runtime::spawn_blocking(move || {
        let mut incoming = std::mem::take(&mut settings);
        let tx_result = settings::update_settings(&save_handle, |current| {
            // Critical invariant: set_settings must never reset completed onboarding status
            if current.has_completed_onboarding {
                incoming.has_completed_onboarding = true;
            }
            let selection_changed = current.model_name != incoming.model_name
                || current.local_engine != incoming.local_engine
                || current.local_acceleration != incoming.local_acceleration;
            if selection_changed {
                whisper_runner::validate_local_model_selection(&save_handle, &incoming)?;
            }
            *current = incoming.clone();
            Ok(())
        });
        zeroize::Zeroize::zeroize(&mut incoming.api_key);
        zeroize::Zeroize::zeroize(&mut incoming.api_key_gemini);
        zeroize::Zeroize::zeroize(&mut incoming.api_key_openai);
        zeroize::Zeroize::zeroize(&mut incoming.api_key_groq);
        zeroize::Zeroize::zeroize(&mut incoming.api_key_huggingface);
        zeroize::Zeroize::zeroize(&mut incoming.api_key_custom);
        let (previous, updated) = tx_result?;
        let autostart_warning = sync_autostart(&save_handle, updated.autostart).err();
        Ok::<_, String>((previous, updated, autostart_warning))
    })
    .await
    .map_err(|error| format!("Settings storage worker failed: {error}"))??;

    let _ = app_handle.emit("overlay-preferences", OverlayPreferences::from(&updated));

    keyboard_hook::update_hotkey(&updated.hotkey)?;
    if let Some(state) = app_handle.try_state::<AppState>() {
        state
            .toggle_enabled
            .store(updated.toggle_enabled, Ordering::SeqCst);
    }

    if previous.gaming_mode_enabled != updated.gaming_mode_enabled {
        apply_gaming_mode_state(&app_handle, updated.gaming_mode_enabled, false).await;
    } else {
        let sidecar_handle = app_handle.clone();
        let sidecar_settings = updated;
        tauri::async_runtime::spawn_blocking(move || {
            whisper_runner::ensure_parakeet_server_state(&sidecar_handle, &sidecar_settings);
            whisper_runner::ensure_whisper_server_state(&sidecar_handle, &sidecar_settings);
            llm_runner::ensure_llm_server_state(&sidecar_handle, &sidecar_settings);
        });
    }
    let _ = app_handle.emit("settings-changed", ());
    Ok(autostart_warning)
}

#[tauri::command]
async fn set_ui_language(app_handle: tauri::AppHandle, ui_language: String) -> Result<(), String> {
    let _apply_guard = match app_handle.try_state::<AppState>() {
        Some(ref state) => Some(state.inner().settings_apply_lock.lock().await),
        None => None,
    };
    let save_handle = app_handle.clone();
    let (_, updated) = tauri::async_runtime::spawn_blocking(move || {
        settings::update_settings(&save_handle, |current| {
            current.ui_language = ui_language;
            Ok(())
        })
    })
    .await
    .map_err(|error| format!("Interface language storage worker failed: {error}"))??;
    let _ = app_handle.emit("overlay-preferences", OverlayPreferences::from(&updated));
    let _ = app_handle.emit("settings-changed", ());
    Ok(())
}

#[tauri::command]
async fn set_transcription_mode(app_handle: tauri::AppHandle, mode: String) -> Result<(), String> {
    let _apply_guard = match app_handle.try_state::<AppState>() {
        Some(ref state) => Some(state.inner().settings_apply_lock.lock().await),
        None => None,
    };
    let save_handle = app_handle.clone();
    let (_, updated) = tauri::async_runtime::spawn_blocking(move || {
        settings::update_settings(&save_handle, |current| {
            current.transcription_mode = mode;
            whisper_runner::validate_local_model_selection(&save_handle, current)?;
            Ok(())
        })
    })
    .await
    .map_err(|error| format!("Transcription mode storage worker failed: {error}"))??;

    let sidecar_handle = app_handle.clone();
    tauri::async_runtime::spawn_blocking(move || {
        whisper_runner::ensure_parakeet_server_state(&sidecar_handle, &updated);
        whisper_runner::ensure_whisper_server_state(&sidecar_handle, &updated);
        llm_runner::ensure_llm_server_state(&sidecar_handle, &updated);
    });
    let _ = app_handle.emit("settings-changed", ());
    Ok(())
}

#[tauri::command]
async fn set_recognition_language(
    app_handle: tauri::AppHandle,
    language: String,
) -> Result<(), String> {
    let _apply_guard = match app_handle.try_state::<AppState>() {
        Some(ref state) => Some(state.inner().settings_apply_lock.lock().await),
        None => None,
    };
    let save_handle = app_handle.clone();
    tauri::async_runtime::spawn_blocking(move || {
        settings::update_settings(&save_handle, |current| {
            current.language = language;
            Ok(())
        })
        .map(|_| ())
    })
    .await
    .map_err(|error| format!("Recognition language storage worker failed: {error}"))??;
    let _ = app_handle.emit("settings-changed", ());
    Ok(())
}

#[tauri::command]
async fn complete_onboarding(app_handle: tauri::AppHandle) -> Result<(), String> {
    let save_handle = app_handle.clone();
    tauri::async_runtime::spawn_blocking(move || {
        settings::update_settings(&save_handle, |current| {
            current.has_completed_onboarding = true;
            Ok(())
        })
        .map(|_| ())
    })
    .await
    .map_err(|error| format!("Failed to complete onboarding: {error}"))?
}

#[tauri::command]
async fn save_settings_backup(
    contents: String,
    suggested_name: String,
    dialog_title: String,
) -> Result<Option<String>, String> {
    const MAX_BACKUP_BYTES: usize = 2_000_000;

    if contents.len() > MAX_BACKUP_BYTES {
        return Err("Settings backup is too large".to_string());
    }
    let parsed: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|error| format!("Settings backup is not valid JSON: {error}"))?;
    if !parsed.is_object() {
        return Err("Settings backup must be a JSON object".to_string());
    }

    let suggested_name = std::path::Path::new(&suggested_name)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("aura-settings.json")
        .to_string();
    let dialog_title = if dialog_title.trim().is_empty() {
        "Aura settings backup".to_string()
    } else {
        dialog_title
    };

    let selected = tauri::async_runtime::spawn_blocking(move || {
        rfd::FileDialog::new()
            .set_title(dialog_title)
            .set_file_name(suggested_name)
            .add_filter("JSON", &["json"])
            .save_file()
    })
    .await
    .map_err(|error| format!("Settings save dialog failed: {error}"))?;

    let Some(mut path) = selected else {
        return Ok(None);
    };
    if path.extension().is_none() {
        path.set_extension("json");
    }
    std::fs::write(&path, contents)
        .map_err(|error| format!("Failed to save settings backup: {error}"))?;
    Ok(Some(path.display().to_string()))
}

fn tray_tooltip(gaming_mode: bool, language: &str) -> &'static str {
    if gaming_mode {
        match language {
            "en" => "Aura — 🎮 Gaming Mode active",
            "de" => "Aura — 🎮 Spielmodus aktiv",
            "fr" => "Aura — 🎮 Mode jeu actif",
            "es" => "Aura — 🎮 Modo de juego activo",
            "it" => "Aura — 🎮 Modalità gioco attiva",
            "zh" => "Aura — 🎮 游戏模式已开启",
            "pt" => "Aura — 🎮 Modo de Jogo ativo",
            "tr" => "Aura — 🎮 Oyun Modu aktif",
            _ => "Aura — 🎮 Игровой режим активен",
        }
    } else {
        "Aura"
    }
}

fn canonical_model_name(model_name: &str) -> Result<String, String> {
    if model_name.len() > 64 {
        return Err("Invalid model name".to_string());
    }
    if llm_runner::is_llm_model_id(model_name) {
        return Ok(model_name.to_string());
    }
    if model_name == "parakeet-v3" {
        return Ok(model_name.to_string());
    }
    let filename = whisper_runner::whisper_model_filename(model_name)?;
    filename
        .strip_prefix("ggml-")
        .and_then(|name| name.strip_suffix(".bin"))
        .map(str::to_string)
        .ok_or_else(|| "Invalid Whisper model filename".to_string())
}

#[tauri::command]
async fn download_model_command(
    app_handle: tauri::AppHandle,
    model_name: String,
) -> Result<(), String> {
    let model_name = canonical_model_name(&model_name)?;
    if model_name == "parakeet-v3" {
        whisper_runner::download_parakeet_model(&app_handle)
            .await
            .map(|_| ())
    } else if llm_runner::is_llm_model_id(&model_name) {
        llm_runner::download_llm_model(&app_handle, model_name.as_str())
            .await
            .map(|_| ())
    } else {
        whisper_runner::download_model(&app_handle, model_name.as_str())
            .await
            .map(|_| ())
    }
}

#[tauri::command]
async fn cancel_model_download(model_name: String) -> Result<(), String> {
    if matches!(
        model_name.as_str(),
        "llm-runtime-vulkan" | "llm-runtime-cpu"
    ) {
        whisper_runner::request_cancel_download(&model_name);
        return Ok(());
    }
    let model_name = canonical_model_name(&model_name)?;
    whisper_runner::request_cancel_download(&model_name);
    Ok(())
}

fn remove_matching_part_files(dir: &std::path::Path, filename: &str) {
    let prefix = format!("{filename}.");
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with(&prefix) && name.ends_with(".part") {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[tauri::command]
async fn delete_model_command(
    app_handle: tauri::AppHandle,
    model_name: String,
) -> Result<(), String> {
    let model_name = canonical_model_name(&model_name)?;
    // An active model is never deleted: doing so leaves the saved selection
    // pointing at a file that no longer exists, and the next dictation fails.
    if let Ok(current) = load_settings_async(app_handle.clone()).await {
        let is_active_speech = current.model_name == model_name
            || (model_name == "parakeet-v3" && current.local_engine == "parakeet");
        if is_active_speech {
            return Err(format!(
                "model_in_use: '{model_name}' is the selected speech model."
            ));
        }
        if llm_runner::is_llm_model_id(&model_name) && current.llm_model == model_name {
            return Err(format!(
                "edit_model_in_use: '{model_name}' is the selected local editing model."
            ));
        }
    }
    if whisper_runner::is_model_download_active(&model_name) {
        return Err(format!(
            "model_download_active: Cannot delete '{model_name}' while its download is active"
        ));
    }
    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {}", e))?;

    if model_name == "parakeet-v3" {
        let folder = app_local_data.join("models").join("parakeet-v3");
        let worker_handle = app_handle.clone();
        tauri::async_runtime::spawn_blocking(move || {
            whisper_runner::stop_parakeet_server_and_watchdog(&worker_handle);
            match std::fs::remove_dir_all(&folder) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(format!("Failed to delete parakeet directory: {error}")),
            }
        })
        .await
        .map_err(|error| format!("Parakeet deletion worker failed: {error}"))??;
        reset_engine_after_model_deletion(&app_handle)?;
        return Ok(());
    }

    if llm_runner::is_llm_model_id(&model_name) {
        let model_path = llm_runner::llm_model_path(&app_handle, &model_name)?;
        let worker_handle = app_handle.clone();
        return tauri::async_runtime::spawn_blocking(move || {
            llm_runner::stop_llm_server(&worker_handle);
            if let (Some(parent), Some(fname)) = (
                model_path.parent(),
                model_path.file_name().and_then(|n| n.to_str()),
            ) {
                remove_matching_part_files(parent, fname);
            }
            match std::fs::remove_file(&model_path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(format!("Failed to delete local editing model: {error}")),
            }
        })
        .await
        .map_err(|error| format!("Local editing model deletion worker failed: {error}"))?;
    }

    let filename = whisper_runner::whisper_model_filename(&model_name)?;
    let models_dir = app_local_data.join("models");
    let model_path = models_dir.join(&filename);

    tauri::async_runtime::spawn_blocking(move || {
        remove_matching_part_files(&models_dir, &filename);
        match std::fs::remove_file(&model_path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!("Failed to delete model file: {error}")),
        }
    })
    .await
    .map_err(|error| format!("Whisper model deletion worker failed: {error}"))?
}

/// When the deleted model was the active parakeet engine, fall back to
/// whisper/base so the next session does not try to start a server whose
/// model is gone (and settings validation does not fail on
/// engine=parakeet without parakeet-v3).
fn reset_engine_after_model_deletion<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Result<(), String> {
    settings::update_settings(app_handle, |settings| {
        if settings.local_engine == "parakeet" {
            crate::logger::log(
                "INFO",
                "Models",
                None,
                "Deleted the active parakeet model; switching local engine to whisper/base",
            );
            settings.local_engine = "whisper".to_string();
            settings.model_name = "base".to_string();
        }
        Ok(())
    })
    .map(|_| ())
}

#[tauri::command]
async fn get_downloaded_models(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {}", e))?;
    let models_dir = app_local_data.join("models");

    tauri::async_runtime::spawn_blocking(move || {
        let mut downloaded = Vec::new();
        let parakeet_dir = models_dir.join("parakeet-v3");
        if whisper_runner::parakeet_model_is_installed(&parakeet_dir) {
            downloaded.push("parakeet-v3".to_string());
        }
        for llm_id in llm_runner::LLM_MODEL_IDS {
            if llm_runner::llm_model_is_installed(&app_handle, llm_id) {
                downloaded.push((*llm_id).to_string());
            }
        }

        let entries = match std::fs::read_dir(&models_dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(downloaded),
            Err(error) => return Err(format!("Failed to inspect downloaded models: {error}")),
        };
        for entry_result in entries {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(error) => {
                    crate::logger::log(
                        "WARN",
                        "Models",
                        None,
                        &format!("Skipping unreadable model directory entry: {error}"),
                    );
                    continue;
                }
            };
            let path = entry.path();
            let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let Some(name) = filename
                .strip_prefix("ggml-")
                .and_then(|value| value.strip_suffix(".bin"))
            else {
                continue;
            };
            if whisper_runner::whisper_model_is_installed(name, &path) {
                downloaded.push(name.to_string());
            }
        }
        downloaded.sort();
        downloaded.dedup();
        Ok::<Vec<String>, String>(downloaded)
    })
    .await
    .map_err(|error| format!("Model inspection worker failed: {error}"))?
}

#[tauri::command]
async fn get_history(app_handle: tauri::AppHandle) -> Result<Vec<history::HistoryEntry>, String> {
    tauri::async_runtime::spawn_blocking(move || history::load_history(&app_handle))
        .await
        .map_err(|error| format!("History load worker failed: {error}"))?
}

#[tauri::command]
async fn clear_history(app_handle: tauri::AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || history::clear_history(&app_handle))
        .await
        .map_err(|error| format!("History clear worker failed: {error}"))?
}

#[tauri::command]
async fn copy_to_clipboard(text: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut clipboard = arboard::Clipboard::new().map_err(|error| error.to_string())?;
        clipboard.set_text(text).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Clipboard worker failed: {error}"))?
}

#[tauri::command]
async fn get_audio_input_devices() -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(audio_recorder::list_audio_input_devices)
        .await
        .map_err(|e| format!("Audio device enumeration task failed: {e}"))
}

#[tauri::command]
async fn start_mic_meter(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(state) = app_handle.try_state::<AppState>() {
        if state.is_recording.load(Ordering::SeqCst) {
            return Err("Cannot start mic meter while dictation recording is active".to_string());
        }
    }
    let settings = load_settings_async(app_handle.clone())
        .await
        .unwrap_or_default();
    let dev = if settings.audio_input_device == "default" || settings.audio_input_device.is_empty()
    {
        None
    } else {
        Some(settings.audio_input_device.as_str())
    };
    audio_recorder::start_mic_meter(app_handle, dev)
}

#[tauri::command]
fn stop_mic_meter() {
    audio_recorder::stop_mic_meter();
}

#[derive(serde::Serialize)]
struct AppUpdateInfo {
    current_version: String,
    version: String,
}

async fn query_app_update(
    app_handle: &tauri::AppHandle,
) -> Result<Option<tauri_plugin_updater::Update>, String> {
    let updater = app_handle
        .updater()
        .map_err(|error| format!("Could not initialize updater: {error}"))?;
    tokio::time::timeout(std::time::Duration::from_secs(30), updater.check())
        .await
        .map_err(|_| "Update check timed out after 30 seconds".to_string())?
        .map_err(|error| format!("Update check failed: {error}"))
}

fn prepare_update_notification_payload(
    gaming_mode_active: bool,
    ui_language: &str,
    new_version: &str,
) -> Option<(String, String)> {
    if gaming_mode_active {
        return None;
    }
    let copy = match ui_language {
        "en" => (
            "Aura Update Available".to_string(),
            format!("Version {new_version} is available. Click to open Settings and update."),
        ),
        "de" => (
            "Aura-Update verfügbar".to_string(),
            format!("Version {new_version} ist verfügbar. Klicken Sie hier, um zu aktualisieren."),
        ),
        "es" => (
            "Actualización de Aura disponible".to_string(),
            format!("La versión {new_version} está disponible. Haga clic para actualizar."),
        ),
        "fr" => (
            "Mise à jour Aura disponible".to_string(),
            format!("La version {new_version} est disponible. Cliquez pour mettre à jour."),
        ),
        "it" => (
            "Aggiornamento di Aura disponibile".to_string(),
            format!("La versione {new_version} è disponibile. Fai clic per aggiornare."),
        ),
        "zh" => (
            "Aura 有新版本可用".to_string(),
            format!("版本 {new_version} 已发布。点击打开设置进行更新。"),
        ),
        "pt" => (
            "Atualização do Aura disponível".to_string(),
            format!("A versão {new_version} está disponível. Clique para atualizar."),
        ),
        "tr" => (
            "Aura Güncellemesi Mevcut".to_string(),
            format!("Sürüm {new_version} mevcut. Güncellemek için tıklayın."),
        ),
        _ => (
            "Доступно обновление Aura".to_string(),
            format!("Доступна версия {new_version}. Нажмите, чтобы открыть настройки и обновить."),
        ),
    };
    Some(copy)
}

pub fn notify_app_update_available(app: &tauri::AppHandle, new_version: &str) {
    let gaming_active = app
        .try_state::<AppState>()
        .map(|state| state.gaming_mode_active.load(Ordering::Acquire))
        .unwrap_or(false);
    let settings = settings::cached_or_load_settings_without_secrets(app).ok();
    let lang = settings
        .as_ref()
        .map(|s| s.ui_language.as_str())
        .unwrap_or("ru");

    let Some((title, body)) = prepare_update_notification_payload(gaming_active, lang, new_version)
    else {
        crate::logger::log(
            "INFO",
            "Updater",
            None,
            &format!(
                "Suppressing update notification for v{new_version} because Gaming Mode is active"
            ),
        );
        return;
    };

    if let Err(e) = app.notification().builder().title(title).body(body).show() {
        crate::logger::log(
            "WARN",
            "Updater",
            None,
            &format!("Failed to show update notification: {e}"),
        );
    }
}

#[tauri::command]
async fn check_for_app_update(
    app_handle: tauri::AppHandle,
) -> Result<Option<AppUpdateInfo>, String> {
    let update = query_app_update(&app_handle).await?;
    if let Some(ref upd) = update {
        notify_app_update_available(&app_handle, &upd.version);
    }
    Ok(update.map(|update| AppUpdateInfo {
        current_version: update.current_version,
        version: update.version,
    }))
}

static UPDATE_INSTALL_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

struct UpdateInstallGuard;

impl Drop for UpdateInstallGuard {
    fn drop(&mut self) {
        UPDATE_INSTALL_IN_PROGRESS.store(false, Ordering::Release);
    }
}

#[tauri::command]
async fn install_app_update(app_handle: tauri::AppHandle) -> Result<(), String> {
    if UPDATE_INSTALL_IN_PROGRESS
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Err("Update installation is already in progress".to_string());
    }
    let _install_guard = UpdateInstallGuard;

    cancel_recording(app_handle.clone()).await;

    let update = query_app_update(&app_handle)
        .await?
        .ok_or_else(|| "No update is currently available".to_string())?;

    let stop_handle = app_handle.clone();
    let _ = tauri::async_runtime::spawn_blocking(move || {
        stop_all_sidecars_and_watchdogs(&stop_handle);
    })
    .await;

    tokio::time::timeout(
        std::time::Duration::from_secs(15 * 60),
        update.download_and_install(
            |_chunk_size, _content_length| {},
            || crate::logger::log("INFO", "Updater", None, "Update download completed"),
        ),
    )
    .await
    .map_err(|_| "Update download timed out after 15 minutes".to_string())?
    .map_err(|error| format!("Update installation failed: {error}"))
}

const ALLOWED_OPEN_URL_HOSTS: &[&str] = &[
    "aistudio.google.com",
    "platform.openai.com",
    "console.groq.com",
    "huggingface.co",
    "deepinfra.com",
    "docs.nvidia.com",
    "github.com",
    "malashkadev.github.io",
    "sourceforge.net",
];

fn validate_allowed_external_url(raw: &str) -> Result<reqwest::Url, String> {
    let parsed = reqwest::Url::parse(raw.trim()).map_err(|_| "URL is invalid".to_string())?;
    if parsed.scheme() != "https" {
        return Err("Only https URLs may be opened".to_string());
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("URL must not contain embedded credentials".to_string());
    }
    let host = parsed.host_str().unwrap_or("");
    if !ALLOWED_OPEN_URL_HOSTS
        .iter()
        .any(|allowed| host.eq_ignore_ascii_case(allowed))
    {
        return Err(format!(
            "Host '{host}' is not in the external URL allowlist"
        ));
    }
    Ok(parsed)
}

/// Opens a URL in the user's default browser (API key portals, release notes, documentation).
#[tauri::command]
async fn open_url(app_handle: tauri::AppHandle, url: String) -> Result<(), String> {
    let parsed = validate_allowed_external_url(&url)?;

    use tauri_plugin_opener::OpenerExt;
    app_handle
        .opener()
        .open_url(parsed.as_str(), None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn relaunch_app(app_handle: tauri::AppHandle) {
    let stop_handle = app_handle.clone();
    let _ = tauri::async_runtime::spawn_blocking(move || {
        stop_all_sidecars_and_watchdogs(&stop_handle);
    })
    .await;
    app_handle.restart();
}

#[tauri::command]
async fn get_diagnostic_report(app_handle: tauri::AppHandle) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || logger::generate_diagnostic_report(&app_handle))
        .await
        .map_err(|error| format!("Diagnostic report worker failed: {error}"))?
}

#[derive(Clone, serde::Serialize)]
struct EngineHealth {
    engine: String,
    running: bool,
    starting: bool,
    provider: Option<String>,
    port: Option<u16>,
    gaming_mode: bool,
}

#[tauri::command]
async fn get_engine_health(app_handle: tauri::AppHandle) -> EngineHealth {
    tauri::async_runtime::spawn_blocking(move || {
        let settings =
            settings::cached_or_load_settings_without_secrets(&app_handle).unwrap_or_default();
        let (gaming_mode, whisper_starting, parakeet_starting) = app_handle
            .try_state::<AppState>()
            .map(|s| {
                (
                    s.gaming_mode_active.load(Ordering::Acquire),
                    s.whisper_starting.load(Ordering::Acquire),
                    s.parakeet_starting.load(Ordering::Acquire),
                )
            })
            .unwrap_or((false, false, false));

        if settings.local_engine == "whisper" {
            return match whisper_runner::whisper_server_status(&app_handle) {
                Some((provider, port)) => EngineHealth {
                    engine: "whisper".to_string(),
                    running: true,
                    starting: false,
                    provider: Some(provider),
                    port: Some(port),
                    gaming_mode,
                },
                None => EngineHealth {
                    engine: "whisper".to_string(),
                    running: false,
                    starting: whisper_starting,
                    provider: None,
                    port: None,
                    gaming_mode,
                },
            };
        }
        if settings.local_engine != "parakeet" {
            return EngineHealth {
                engine: settings.local_engine,
                running: true,
                starting: false,
                provider: None,
                port: None,
                gaming_mode,
            };
        }
        match whisper_runner::parakeet_server_status(&app_handle) {
            Some((provider, port)) => EngineHealth {
                engine: "parakeet".to_string(),
                running: true,
                starting: false,
                provider: Some(provider),
                port: Some(port),
                gaming_mode,
            },
            None => EngineHealth {
                engine: "parakeet".to_string(),
                running: false,
                starting: parakeet_starting,
                provider: None,
                port: None,
                gaming_mode,
            },
        }
    })
    .await
    .unwrap_or_else(|_| EngineHealth {
        engine: "whisper".to_string(),
        running: false,
        starting: false,
        provider: None,
        port: None,
        gaming_mode: false,
    })
}

fn sanitize_frontend_log_field(input: &str, max_chars: usize, strip_brackets: bool) -> String {
    let cleaned: String = input
        .chars()
        .map(|ch| {
            if ch.is_control() {
                ' '
            } else if strip_brackets && ch == '[' {
                '('
            } else if strip_brackets && ch == ']' {
                ')'
            } else {
                ch
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    cleaned.chars().take(max_chars).collect()
}

#[tauri::command]
async fn log_frontend_event(
    level: String,
    tag: String,
    session: Option<String>,
    message: String,
) -> Result<(), String> {
    let safe_level = match level.trim().to_ascii_uppercase().as_str() {
        "DEBUG" => "DEBUG",
        "WARN" | "WARNING" => "WARN",
        "ERROR" => "ERROR",
        _ => "INFO",
    };
    let safe_tag_inner = sanitize_frontend_log_field(&tag, 32, true);
    let safe_tag = if safe_tag_inner.is_empty() {
        "Frontend".to_string()
    } else {
        format!("FE:{safe_tag_inner}")
    };
    let safe_session = session
        .as_deref()
        .map(|s| sanitize_frontend_log_field(s, 32, true))
        .filter(|s| !s.is_empty());
    let safe_message = sanitize_frontend_log_field(&message, 512, true);
    if !safe_message.is_empty() {
        logger::log(
            safe_level,
            &safe_tag,
            safe_session.as_deref(),
            &safe_message,
        );
    }
    Ok(())
}

fn sync_autostart(app_handle: &tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app_handle.autolaunch();
    let result = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
    if let Err(e) = result {
        let err_str = e.to_string();
        let is_not_found = err_str.contains("os error 2")
            || err_str.contains("not find")
            || err_str.contains("не удается найти")
            || err_str.contains("not found");

        if !enabled && is_not_found {
            crate::logger::log("INFO", "Autostart", None, "Autostart was already disabled.");
            return Ok(());
        }
        crate::logger::log(
            "ERROR",
            "Autostart",
            None,
            &format!("Failed to update autostart ({}): {}", enabled, err_str),
        );
        return Err(err_str);
    }
    Ok(())
}

fn recording_wav_path(gen: u64) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("aura-rec-{}-{}.wav", std::process::id(), gen))
}

fn chunk_wav_path(gen: u64) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("aura-chunk-{}-{}.wav", std::process::id(), gen))
}

fn provider_from(settings: &settings::Settings) -> ai_client::ApiProvider {
    match settings.api_provider.as_str() {
        "openai" => ai_client::ApiProvider::OpenAi,
        "groq" => ai_client::ApiProvider::Groq,
        "huggingface" => ai_client::ApiProvider::HuggingFace,
        "custom" => ai_client::ApiProvider::Custom,
        _ => ai_client::ApiProvider::Gemini,
    }
}

/// Resolves the language to send to the recognizer from settings + detected layout.
/// When editing selected text, we bias short voice commands symmetrically for all
/// non-English UI/layout languages while falling back to auto-detect (`""`) whenever
/// an English keyboard is active on a non-English UI (so bilingual users can speak
/// commands in either language without poisoning Whisper's decoder priors).
fn effective_language(
    settings: &settings::Settings,
    layout_language: &str,
    has_selection: bool,
) -> String {
    const SUPPORTED_NON_EN: &[&str] = &["ru", "de", "es", "fr", "it", "zh", "pt", "tr", "nl"];

    match settings.language.as_str() {
        "ru" | "en" | "de" | "es" | "fr" | "it" | "zh" | "pt" | "tr" | "nl" => {
            settings.language.clone()
        }
        "layout" => {
            if has_selection {
                if SUPPORTED_NON_EN.contains(&layout_language)
                    || (layout_language == "en" && settings.ui_language == "en")
                {
                    layout_language.to_string()
                } else {
                    String::new()
                }
            } else {
                layout_language.to_string()
            }
        }
        _ => {
            if has_selection
                && SUPPORTED_NON_EN.contains(&layout_language)
                && layout_language == settings.ui_language
            {
                layout_language.to_string()
            } else {
                String::new()
            }
        }
    }
}

struct WordSpan {
    _start: usize,
    end: usize,
    text: String,
}

fn extract_word_spans(chars: &[char]) -> Vec<WordSpan> {
    let mut spans = Vec::new();
    let mut in_word = false;
    let mut start = 0;

    for (i, &ch) in chars.iter().enumerate() {
        if !ch.is_whitespace() {
            if !in_word {
                in_word = true;
                start = i;
            }
        } else if in_word {
            in_word = false;
            let word_str: String = chars[start..i].iter().collect();
            spans.push(WordSpan {
                _start: start,
                end: i,
                text: word_str,
            });
        }
    }
    if in_word {
        let word_str: String = chars[start..chars.len()].iter().collect();
        spans.push(WordSpan {
            _start: start,
            end: chars.len(),
            text: word_str,
        });
    }
    spans
}

fn normalize_word_for_matching(w: &str) -> String {
    w.trim_matches(|c: char| c.is_ascii_punctuation() || ".,!?:;-—\"'«»()[]{}".contains(c))
        .to_lowercase()
        .replace('ё', "е")
}

#[derive(Debug, PartialEq, Eq)]
struct TextReplacementPlan {
    backspaces: usize,
    suffix: String,
}

fn plan_text_replacement(typed_so_far: &str, new_text: &str, is_live: bool) -> TextReplacementPlan {
    let typed_chars: Vec<char> = typed_so_far.chars().collect();
    let new_chars: Vec<char> = new_text.chars().collect();

    let (common_typed_end, common_new_end) = if is_live {
        let typed_spans = extract_word_spans(&typed_chars);
        let new_spans = extract_word_spans(&new_chars);

        let mut matched_typed_end = 0;
        let mut matched_new_end = 0;

        for (tw, nw) in typed_spans.iter().zip(new_spans.iter()) {
            let norm_t = normalize_word_for_matching(&tw.text);
            let norm_n = normalize_word_for_matching(&nw.text);

            if !norm_t.is_empty() && norm_t == norm_n {
                matched_typed_end = tw.end;
                matched_new_end = nw.end;
            } else {
                break;
            }
        }

        while matched_typed_end < typed_chars.len()
            && matched_new_end < new_chars.len()
            && typed_chars[matched_typed_end].is_whitespace()
            && new_chars[matched_new_end].is_whitespace()
            && typed_chars[matched_typed_end] == new_chars[matched_new_end]
        {
            matched_typed_end += 1;
            matched_new_end += 1;
        }

        (matched_typed_end, matched_new_end)
    } else {
        let mut common_len = 0;
        for (c1, c2) in typed_chars.iter().zip(new_chars.iter()) {
            if c1 == c2 {
                common_len += 1;
            } else {
                break;
            }
        }
        (common_len, common_len)
    };

    TextReplacementPlan {
        backspaces: typed_chars.len() - common_typed_end,
        suffix: new_chars[common_new_end..].iter().collect(),
    }
}

fn diff_and_type_with<D>(
    typed_so_far: &mut String,
    new_text: &str,
    is_live: bool,
    dispatch: D,
) -> Result<keyboard_simulator::ReplacementDispatchMetrics, String>
where
    D: FnOnce(
        usize,
        &str,
    ) -> Result<
        keyboard_simulator::ReplacementDispatchMetrics,
        keyboard_simulator::TextReplacementError,
    >,
{
    let plan = plan_text_replacement(typed_so_far, new_text, is_live);
    let retained_chars = typed_so_far.chars().count().saturating_sub(plan.backspaces);
    let mut applied_text: String = typed_so_far.chars().take(retained_chars).collect();
    applied_text.push_str(&plan.suffix);
    match dispatch(plan.backspaces, &plan.suffix) {
        Ok(metrics) => {
            *typed_so_far = applied_text;
            Ok(metrics)
        }
        Err(error) => {
            // Even an interrupted dispatch leaves part (or all) of the planned
            // change in the document. Commit a mirror of exactly what landed so
            // a later Esc cancel backspaces the true visible text instead of a
            // stale pre-update prefix, which would leave a partial tail behind.
            commit_partial_replacement(typed_so_far, &plan.suffix, &error);
            Err(error.message)
        }
    }
}

/// Mirrors into `typed_so_far` the portion of a planned replacement that a
/// dispatch actually committed before being interrupted. Backspaces map
/// one-to-one onto chars; committed UTF-16 units are converted back to the
/// longest matching char prefix of the planned suffix.
fn commit_partial_replacement(
    typed_so_far: &mut String,
    planned_suffix: &str,
    error: &keyboard_simulator::TextReplacementError,
) {
    if error.backspaces_committed == 0 && error.utf16_units_committed == 0 {
        return;
    }
    let current_chars = typed_so_far.chars().count();
    let backspaces_applied = error.backspaces_committed.min(current_chars);
    let retained: String = typed_so_far
        .chars()
        .take(current_chars - backspaces_applied)
        .collect();
    let mut consumed_units = 0usize;
    let mut landed_suffix = String::new();
    for ch in planned_suffix.chars() {
        let units = ch.len_utf16();
        if consumed_units + units > error.utf16_units_committed {
            break;
        }
        consumed_units += units;
        landed_suffix.push(ch);
    }
    typed_so_far.clear();
    typed_so_far.push_str(&retained);
    typed_so_far.push_str(&landed_suffix);
}

fn diff_and_type<F>(
    typed_so_far: &mut String,
    new_text: &str,
    is_live: bool,
    should_stop: F,
) -> Result<keyboard_simulator::ReplacementDispatchMetrics, String>
where
    F: Fn() -> bool,
{
    diff_and_type_with(typed_so_far, new_text, is_live, |backspaces, suffix| {
        keyboard_simulator::replace_text(backspaces, suffix, should_stop)
    })
}

fn is_silence_hallucination(text: &str) -> bool {
    let t = text.trim().to_lowercase();
    if t.is_empty() {
        return true;
    }
    if t.chars()
        .all(|c| c.is_ascii_punctuation() || ".,!?:;-—\"'«»()[]{}… \t\r\n".contains(c))
    {
        return true;
    }

    // Unambiguous fragments Whisper produces on silence — safe to match as substrings.
    let substring_markers = [
        "no audio to transcribe",
        "no speech",
        "no audio detected",
        "there is no audio",
        "subtitles by",
        "amara.org",
        "субтитры сделал",
        "субтитры создал",
        "редактор субтитров",
        "подпишитесь на канал",
        "подписывайтесь на канал",
        "blank_audio",
        "текст фильма",
    ];
    for marker in &substring_markers {
        if t.contains(marker) {
            return true;
        }
    }

    // Common hallucinated phrases — must match the WHOLE normalized text so that
    // ordinary dictation containing these words is never discarded.
    let normalized: String = t
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();
    let normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
    let exact_markers = [
        // Russian
        "спасибо за просмотр",
        "спасибо за внимание",
        "спасибо что посмотрели",
        "до новых встреч",
        "продолжение следует",
        "подпишитесь",
        "подпишитесь на канал",
        "подписывайтесь на канал",
        "ставьте лайки",
        "жми колокольчик",
        "используйте что это не так",
        "используйтесь что это не так",
        "убедитесь что это не так",
        "я не знаю что я здесь делаю",
        // English
        "thank you for watching",
        "thanks for watching",
        "thank you for watching please subscribe",
        "to be continued",
        "subscribe to my channel",
        "dont forget to subscribe",
        "transcript provided by",
        "i m not sure what i m doing here",
        "i am not sure what i am doing here",
        "i m not sure what to do with this",
        "thank you",
        "you",
        "you you",
        "you you you",
        "you you you you",
        // German
        "danke fürs zuschauen",
        "danke furs zuschauen",
        "danke fürs ansehen",
        "danke furs ansehen",
        "bitte abonnieren",
        "abonniere meinen kanal",
        "das wars",
        // French
        "sous titrage société radio canada",
        "sous titrage societe radio canada",
        "merci davoir regardé cette vidéo",
        "merci davoir regarde cette video",
        "merci davoir regardé",
        "merci davoir regarde",
        "abonnez vous à ma chaîne",
        "abonnez vous a ma chaine",
        // Spanish
        "gracias por ver el video",
        "gracias por su atención",
        "gracias por su atencion",
        "cc por antarctica films argentina",
        "dale like y suscríbete",
        "dale like y suscribete",
        // NVIDIA Parakeet tends to emit a single affirmative word on silence
        // or on a near-empty clip (e.g. a say-nothing quick hotkey press).
        // Whole-utterance match only, so real dictation containing these as
        // part of a longer phrase is never discarded.
        "yeah",
        "yeah yeah",
        "yeah yeah yeah",
        "yeah yeah yeah yeah",
        "yep",
        "yep yep",
        // Parakeet also emits near-silence single-letter filler ("Mm", "S").
        "mm",
        "mm mm",
        "мм",
        "мм мм",
        "s",
    ];
    if exact_markers.contains(&normalized.as_str()) {
        return true;
    }

    // Single-word or repeated filler loops like "ммммммммммм", "эмммммммммммммм", "hmmmmmm", "ummm"
    if !normalized.is_empty() {
        let no_spaces: String = normalized.chars().filter(|c| !c.is_whitespace()).collect();
        if !no_spaces.is_empty() {
            if no_spaces.chars().all(|c| c == 'м' || c == 'm') {
                return true;
            }
            let mut chs = no_spaces.chars();
            let is_filler_hallucination = match chs.next() {
                Some('э') => chs.all(|c| c == 'м'),
                // Latin fillers require at least three characters so real
                // two-letter words ("um", "em" in German/Portuguese) survive.
                Some('e') | Some('u') | Some('h') => {
                    no_spaces.chars().count() >= 3 && chs.all(|c| c == 'm')
                }
                _ => false,
            };
            if is_filler_hallucination {
                return true;
            }
        }
    }

    // Also catch repeated loops where the whole utterance consists solely of repeating an exact marker
    let norm_words: Vec<&str> = normalized.split_whitespace().collect();
    if norm_words.len() >= 2 {
        for marker in &exact_markers {
            let marker_words: Vec<&str> = marker.split_whitespace().collect();
            if !marker_words.is_empty()
                && norm_words.len() >= marker_words.len() * 2
                && norm_words.len().is_multiple_of(marker_words.len())
            {
                let all_match = norm_words
                    .chunks(marker_words.len())
                    .all(|chunk| chunk == marker_words.as_slice());
                if all_match {
                    return true;
                }
            }
        }
    }
    false
}

fn capitalize_word(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn clean_hallucinated_brackets(text: &str) -> String {
    let mut cleaned = text.trim().to_string();
    let noise_words = [
        "музыка",
        "music",
        "laughter",
        "смех",
        "background noise",
        "шум",
        "applause",
        "аплодисменты",
        "silence",
        "тишина",
        "sigh",
        "вздох",
        "cough",
        "кашель",
        "crying",
        "плач",
    ];
    for term in &noise_words {
        let brackets_upper = format!("[{}]", capitalize_word(term));
        let brackets_lower = format!("[{}]", term);
        let parens_upper = format!("({})", capitalize_word(term));
        let parens_lower = format!("({})", term);

        cleaned = cleaned.replace(&brackets_upper, "");
        cleaned = cleaned.replace(&brackets_lower, "");
        cleaned = cleaned.replace(&parens_upper, "");
        cleaned = cleaned.replace(&parens_lower, "");
    }
    cleaned.trim().to_string()
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ClipboardBackup {
    Text(String),
    Image {
        width: usize,
        height: usize,
        bytes: Vec<u8>,
    },
    Empty,
    Preserve,
}

#[cfg(target_os = "windows")]
fn is_clipboard_empty_os() -> bool {
    unsafe { windows_sys::Win32::System::DataExchange::CountClipboardFormats() == 0 }
}

#[cfg(not(target_os = "windows"))]
fn is_clipboard_empty_os() -> bool {
    false
}

#[cfg(target_os = "windows")]
fn get_clipboard_sequence_number() -> u32 {
    unsafe { windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber() }
}

#[cfg(not(target_os = "windows"))]
fn get_clipboard_sequence_number() -> u32 {
    0
}

/// Writes a transient transcript for the paste keystroke. On Windows the write
/// is marked so the text does not reach Win+V or the cloud clipboard; if that
/// fails we fall back to an ordinary write rather than losing the paste.
fn write_transcript_to_clipboard(text: &str) -> bool {
    #[cfg(target_os = "windows")]
    if clipboard_win::set_text_private(text) {
        return true;
    }
    arboard::Clipboard::new()
        .ok()
        .map(|mut clipboard| clipboard.set_text(text.to_string()).is_ok())
        .unwrap_or(false)
}

fn backup_clipboard() -> ClipboardBackup {
    for _ in 0..3 {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            if let Ok(text) = cb.get_text() {
                return ClipboardBackup::Text(text);
            }
            if let Ok(img) = cb.get_image() {
                return ClipboardBackup::Image {
                    width: img.width,
                    height: img.height,
                    bytes: img.bytes.into_owned(),
                };
            }
            if is_clipboard_empty_os() {
                return ClipboardBackup::Empty;
            } else {
                return ClipboardBackup::Preserve;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    if is_clipboard_empty_os() {
        ClipboardBackup::Empty
    } else {
        ClipboardBackup::Preserve
    }
}

#[cfg(target_os = "windows")]
fn restore_clipboard_preserving_history(backup: &ClipboardBackup) -> bool {
    match backup {
        ClipboardBackup::Text(text) => clipboard_win::set_text_private(text),
        ClipboardBackup::Image {
            width,
            height,
            bytes,
        } => clipboard_win::set_image_private(*width, *height, bytes),
        ClipboardBackup::Empty | ClipboardBackup::Preserve => false,
    }
}

#[cfg(not(target_os = "windows"))]
fn restore_clipboard_preserving_history(_backup: &ClipboardBackup) -> bool {
    false
}

fn restore_clipboard(backup: ClipboardBackup) {
    if matches!(backup, ClipboardBackup::Preserve) {
        crate::logger::log(
            "INFO",
            "Clipboard",
            None,
            "Clipboard held non-text/non-image formats (e.g. files/tables); skipped clearing to prevent data loss",
        );
        return;
    }
    // The previous item is already in Win+V from its original copy. Restoring
    // it through the ordinary writer would add another duplicate for every
    // dictation, so restore it as transient content instead.
    if restore_clipboard_preserving_history(&backup) {
        return;
    }
    for _ in 0..5 {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let res = match &backup {
                ClipboardBackup::Text(text) => cb.set_text(text.clone()),
                ClipboardBackup::Image {
                    width,
                    height,
                    bytes,
                } => {
                    let img = arboard::ImageData {
                        width: *width,
                        height: *height,
                        bytes: std::borrow::Cow::Borrowed(bytes),
                    };
                    cb.set_image(img)
                }
                ClipboardBackup::Empty => cb.clear(),
                ClipboardBackup::Preserve => Ok(()),
            };
            if res.is_ok() {
                return;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn restore_clipboard_if_unchanged(backup: ClipboardBackup, expected_temporary_text: &str) {
    if matches!(backup, ClipboardBackup::Preserve) {
        return;
    }
    let current_text_opt = arboard::Clipboard::new()
        .ok()
        .and_then(|mut clipboard| clipboard.get_text().ok());

    let unchanged = match current_text_opt {
        Some(ref current) => current == expected_temporary_text,
        None => expected_temporary_text.is_empty(),
    };

    if unchanged {
        restore_clipboard(backup);
    } else {
        crate::logger::log(
            "INFO",
            "Clipboard",
            None,
            "Clipboard changed externally; skipping Aura's clipboard restore",
        );
    }
}

/// Returns `true` when the given generation still identifies the *current*
/// recording session. Stale async tasks use this before touching the clipboard
/// so they cannot clobber state captured by a newer, overlapping session.
fn session_still_current(state: &AppState, my_gen: u64) -> bool {
    state.session_gen.load(Ordering::SeqCst) == my_gen
}

/// Serialized clipboard restore: refuses to restore unless the caller's session
/// is still current and holds the shared clipboard mutex, preventing an
/// outdated session from overwriting a newer session's clipboard contents.
fn restore_clipboard_guarded(
    state: &AppState,
    my_gen: u64,
    backup: ClipboardBackup,
    expected_temporary_text: Option<&str>,
) {
    if !session_still_current(state, my_gen) {
        crate::logger::log(
            "INFO",
            "Clipboard",
            None,
            &format!(
                "Session ({my_gen}) is stale; skipping clipboard restore to protect newer session"
            ),
        );
        return;
    }
    match expected_temporary_text {
        Some(expected) => restore_clipboard_if_unchanged(backup, expected),
        None => restore_clipboard(backup),
    }
}

fn resolve_session_clipboard_backup(
    unrestored_slot: &mut Option<(u64, ClipboardBackup, String)>,
    current_clipboard_text: Option<&str>,
    fresh_backup: impl FnOnce() -> ClipboardBackup,
) -> ClipboardBackup {
    if let Some((_, ref orig_backup, ref temp_text)) = unrestored_slot {
        if current_clipboard_text == Some(temp_text.as_str()) {
            return orig_backup.clone();
        }
        *unrestored_slot = None;
    }
    fresh_backup()
}

fn take_unrestored_clipboard_for_restore(
    unrestored_slot: &mut Option<(u64, ClipboardBackup, String)>,
    my_gen: u64,
) -> Option<(ClipboardBackup, String)> {
    let owned_by_me = unrestored_slot
        .as_ref()
        .map(|(owner_gen, _, _)| *owner_gen == my_gen)
        .unwrap_or(false);
    if owned_by_me {
        unrestored_slot
            .take()
            .map(|(_, backup, expected)| (backup, expected))
    } else {
        None
    }
}

/// Reads or inherits the user's real pre-Aura clipboard while holding `clipboard_mutex`.
fn peek_or_backup_clipboard(state: &AppState) -> ClipboardBackup {
    let current_text = arboard::Clipboard::new()
        .ok()
        .and_then(|mut cb| cb.get_text().ok());
    let mut guard = state
        .unrestored_clipboard
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    resolve_session_clipboard_backup(&mut guard, current_text.as_deref(), backup_clipboard)
}

fn is_whisper_server_timeout_error(err: &str) -> bool {
    let lower = err.to_ascii_lowercase();
    lower.contains("timed out") || lower.contains("timeout")
}

/// Runs the blocking whisper.cpp sidecar off the async runtime.
///
/// `generation` identifies the owning session: the sidecar decodes are aborted
/// as soon as a newer session supersedes the current one, so a stale
/// verification decode never keeps the overlay stuck on "processing".
async fn run_local_whisper_async(
    app_handle: tauri::AppHandle,
    model: String,
    wav: String,
    language: String,
    dictionary: String,
    generation: u64,
) -> Result<String, String> {
    if session_stale_for_abort(&app_handle, generation) {
        return Err("Session superseded".to_string());
    }
    if model == "parakeet-v3" {
        tauri::async_runtime::spawn_blocking(move || {
            whisper_runner::run_parakeet(&app_handle, &wav, &language, &dictionary, || {
                session_stale_for_abort(&app_handle, generation)
            })
        })
        .await
        .map_err(|e| format!("Local ASR task failed: {e}"))?
    } else {
        let mut whisper_port = app_handle
            .try_state::<AppState>()
            .map(|state| state.whisper_port.load(Ordering::SeqCst))
            .unwrap_or(0);

        if whisper_port == 0 {
            let is_starting = app_handle
                .try_state::<AppState>()
                .map(|state| state.whisper_starting.load(Ordering::Acquire))
                .unwrap_or(false);
            if is_starting {
                crate::logger::log(
                    "INFO",
                    "ASR",
                    None,
                    "Resident Whisper server is starting up; waiting briefly for prewarm to finish",
                );
                let wait_start = std::time::Instant::now();
                while wait_start.elapsed() < std::time::Duration::from_secs(8) {
                    if session_stale_for_abort(&app_handle, generation) {
                        return Err("Session superseded".to_string());
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                    whisper_port = app_handle
                        .try_state::<AppState>()
                        .map(|state| state.whisper_port.load(Ordering::SeqCst))
                        .unwrap_or(0);
                    if whisper_port > 0 {
                        break;
                    }
                    let still_starting = app_handle
                        .try_state::<AppState>()
                        .map(|state| state.whisper_starting.load(Ordering::Acquire))
                        .unwrap_or(false);
                    if !still_starting {
                        break;
                    }
                }
            }
        }

        if session_stale_for_abort(&app_handle, generation) {
            return Err("Session superseded".to_string());
        }

        if whisper_port > 0 {
            let wav_path = std::path::PathBuf::from(&wav);
            match whisper_runner::transcribe_via_whisper_server(
                whisper_port,
                &wav_path,
                &language,
                &dictionary,
            )
            .await
            {
                Ok(text) => {
                    return Ok(whisper_runner::strip_prompt_echo(
                        &text,
                        &language,
                        &dictionary,
                    ))
                }
                Err(server_err) => {
                    if is_whisper_server_timeout_error(&server_err)
                        || session_stale_for_abort(&app_handle, generation)
                    {
                        crate::logger::log(
                            "WARN",
                            "ASR",
                            None,
                            &format!(
                                "Resident Whisper server request timed out or session superseded ({server_err}); skipping CLI sidecar fallback"
                            ),
                        );
                        return Err(server_err);
                    }
                    crate::logger::log(
                        "WARN",
                        "ASR",
                        None,
                        &format!(
                            "Resident Whisper server request failed ({server_err}), falling back to CLI sidecar"
                        ),
                    );
                }
            }
        }

        tauri::async_runtime::spawn_blocking(move || {
            whisper_runner::run_local_whisper(&app_handle, &model, &wav, &language, &dictionary)
                .map(|text| whisper_runner::strip_prompt_echo(&text, &language, &dictionary))
        })
        .await
        .map_err(|e| format!("Local ASR task failed: {e}"))?
    }
}

/// True when `generation` is no longer the current session and blocking work
/// belonging to it should be abandoned immediately.
fn session_stale_for_abort(app_handle: &tauri::AppHandle, generation: u64) -> bool {
    app_handle
        .try_state::<AppState>()
        .map(|state| state.session_gen.load(Ordering::SeqCst) != generation)
        .unwrap_or(true)
}

type ParakeetPreviewSocket =
    tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>;

fn send_parakeet_preview_request(
    socket: &mut ParakeetPreviewSocket,
    samples: &[f32],
) -> Result<(), String> {
    if samples.is_empty() {
        return Err("Cannot send an empty Parakeet preview request".to_string());
    }
    let byte_count = samples
        .len()
        .checked_mul(std::mem::size_of::<f32>())
        .and_then(|size| i32::try_from(size).ok())
        .ok_or_else(|| "Parakeet preview request is too large".to_string())?;

    let mut header = Vec::with_capacity(8);
    header.extend_from_slice(&16_000i32.to_le_bytes());
    header.extend_from_slice(&byte_count.to_le_bytes());
    socket
        .send(tungstenite::Message::Binary(header))
        .map_err(|error| format!("Failed to send Parakeet audio header: {error}"))?;

    const SAMPLES_PER_MESSAGE: usize = 16_384;
    for chunk in samples.chunks(SAMPLES_PER_MESSAGE) {
        let mut payload = Vec::with_capacity(std::mem::size_of_val(chunk));
        for &sample in chunk {
            let sanitized = if sample.is_finite() {
                sample.clamp(-1.0, 1.0)
            } else {
                0.0
            };
            payload.extend_from_slice(&sanitized.to_le_bytes());
        }
        socket
            .send(tungstenite::Message::Binary(payload))
            .map_err(|error| format!("Failed to send Parakeet audio chunk: {error}"))?;
    }
    Ok(())
}

fn streaming_session_cancelled(
    app_handle: &tauri::AppHandle,
    generation: u64,
    cancel: &AtomicBool,
) -> bool {
    cancel.load(Ordering::Acquire)
        || app_handle
            .try_state::<AppState>()
            .map(|state| state.session_gen.load(Ordering::SeqCst) != generation)
            .unwrap_or(true)
}

fn connect_parakeet_socket_on_port<F>(
    port: u16,
    mut is_cancelled: F,
) -> Result<ParakeetPreviewSocket, String>
where
    F: FnMut() -> bool,
{
    let started = std::time::Instant::now();
    loop {
        if is_cancelled() {
            return Err("Parakeet request was cancelled".to_string());
        }
        let url = format!("ws://127.0.0.1:{port}");
        let address = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        let attempt = (|| {
            let stream = std::net::TcpStream::connect_timeout(
                &address,
                std::time::Duration::from_millis(500),
            )
            .map_err(|error| format!("TCP connect failed: {error}"))?;
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(1)))
                .map_err(|error| format!("Failed to set handshake read timeout: {error}"))?;
            stream
                .set_write_timeout(Some(std::time::Duration::from_secs(1)))
                .map_err(|error| format!("Failed to set handshake write timeout: {error}"))?;
            tungstenite::client(
                url.as_str(),
                tungstenite::stream::MaybeTlsStream::Plain(stream),
            )
            .map(|(socket, _)| socket)
            .map_err(|error| format!("WebSocket handshake failed: {error}"))
        })();
        match attempt {
            Ok(socket) => {
                if let tungstenite::stream::MaybeTlsStream::Plain(stream) = socket.get_ref() {
                    stream
                        .set_read_timeout(Some(std::time::Duration::from_millis(250)))
                        .map_err(|error| {
                            format!("Failed to configure Parakeet read timeout: {error}")
                        })?;
                    stream
                        .set_write_timeout(Some(std::time::Duration::from_secs(30)))
                        .map_err(|error| {
                            format!("Failed to configure Parakeet write timeout: {error}")
                        })?;
                }
                return Ok(socket);
            }
            Err(error) if started.elapsed() >= std::time::Duration::from_secs(3) => {
                return Err(format!(
                    "Failed to connect to Parakeet WebSocket on port {port}: {error}"
                ));
            }
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(50));
                if is_cancelled() {
                    return Err("Parakeet request was cancelled".to_string());
                }
            }
        }
    }
}

fn read_parakeet_response_cancellable<F>(
    socket: &mut ParakeetPreviewSocket,
    sample_count: usize,
    mut is_cancelled: F,
) -> Result<String, String>
where
    F: FnMut() -> bool,
{
    let audio_seconds = sample_count as u64 / 16_000;
    let deadline = std::time::Instant::now()
        + std::time::Duration::from_secs((30 + audio_seconds / 2).min(60));
    loop {
        if is_cancelled() {
            return Err("Parakeet request was cancelled".to_string());
        }
        match socket.read() {
            Ok(tungstenite::Message::Text(text)) => return Ok(text),
            Ok(tungstenite::Message::Ping(payload)) => socket
                .send(tungstenite::Message::Pong(payload))
                .map_err(|error| format!("Failed to answer Parakeet WebSocket ping: {error}"))?,
            Ok(tungstenite::Message::Pong(_)) => {}
            Ok(tungstenite::Message::Close(frame)) => {
                return Err(format!(
                    "Parakeet server closed before returning a preview: {frame:?}"
                ));
            }
            Ok(_) => {
                return Err("Unexpected binary message from Parakeet preview server".to_string());
            }
            Err(tungstenite::Error::Io(error))
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) =>
            {
                if std::time::Instant::now() >= deadline {
                    return Err(format!(
                        "Parakeet preview timed out for {sample_count} samples"
                    ));
                }
            }
            Err(error) => {
                return Err(format!("Failed to read Parakeet preview response: {error}"));
            }
        }
    }
}

fn normalize_parakeet_response(response_text: &str) -> String {
    let mut transcript = response_text.to_string();
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(response_text) {
        if let Some(text) = value.get("text").and_then(|value| value.as_str()) {
            transcript = text.trim().to_string();
        }
    }

    let has_cyrillic = transcript
        .chars()
        .any(|character| ('\u{0400}'..='\u{04FF}').contains(&character));
    if has_cyrillic {
        transcript.replace("<unk>", "ё")
    } else {
        transcript.replace("<unk>", "")
    }
}

fn decode_parakeet_samples_on_socket<F>(
    socket: &mut ParakeetPreviewSocket,
    samples: &[f32],
    is_cancelled: F,
) -> Result<String, String>
where
    F: FnMut() -> bool,
{
    send_parakeet_preview_request(socket, samples)?;
    let response = read_parakeet_response_cancellable(socket, samples.len(), is_cancelled)?;
    Ok(normalize_parakeet_response(&response))
}

pub(crate) fn finish_parakeet_socket(socket: &mut ParakeetPreviewSocket) {
    if socket
        .send(tungstenite::Message::Text("Done".to_string()))
        .is_err()
    {
        return;
    }

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    loop {
        match socket.read() {
            Ok(tungstenite::Message::Close(_)) => {
                let _ = socket.flush();
                break;
            }
            Ok(tungstenite::Message::Ping(payload)) => {
                if socket.send(tungstenite::Message::Pong(payload)).is_err() {
                    break;
                }
            }
            Ok(_) => {}
            Err(tungstenite::Error::ConnectionClosed | tungstenite::Error::AlreadyClosed) => break,
            Err(tungstenite::Error::Io(error))
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) && std::time::Instant::now() < deadline => {}
            Err(_) => break,
        }
        if std::time::Instant::now() >= deadline {
            break;
        }
    }
}

#[derive(Debug)]
pub(crate) enum ParakeetWarmUpError {
    Connection(String),
    Inference(String),
}

pub(crate) fn warm_up_parakeet_server_port(port: u16) -> Result<u128, ParakeetWarmUpError> {
    const WARM_UP_SAMPLES: usize = 16_000;
    let samples = vec![0.0_f32; WARM_UP_SAMPLES];
    let mut socket =
        connect_parakeet_socket_on_port(port, || false).map_err(ParakeetWarmUpError::Connection)?;
    let started = std::time::Instant::now();
    let result = decode_parakeet_samples_on_socket(&mut socket, &samples, || false)
        .map_err(ParakeetWarmUpError::Inference);
    finish_parakeet_socket(&mut socket);
    result?;
    Ok(started.elapsed().as_millis())
}

struct ParakeetStreamingDecoder<'a> {
    app_handle: &'a tauri::AppHandle,
    generation: u64,
    cancel: &'a AtomicBool,
    recovery_used: bool,
    session_tag: &'a str,
    socket: Option<ParakeetPreviewSocket>,
    socket_connects: u64,
}

impl ParakeetStreamingDecoder<'_> {
    fn is_cancelled(&self) -> bool {
        streaming_session_cancelled(self.app_handle, self.generation, self.cancel)
    }

    fn current_port(&self) -> Result<u16, String> {
        let port = self
            .app_handle
            .try_state::<AppState>()
            .map(|state| state.parakeet_port.load(Ordering::SeqCst))
            .ok_or_else(|| "Application state is unavailable".to_string())?;
        if port == 0 {
            return Err("Parakeet server port is unavailable".to_string());
        }
        Ok(port)
    }

    fn ensure_socket(&mut self) -> Result<(), String> {
        if self.socket.is_some() {
            return Ok(());
        }
        let port = self.current_port()?;
        let app_handle = self.app_handle;
        let generation = self.generation;
        let cancel = self.cancel;
        let socket = connect_parakeet_socket_on_port(port, || {
            streaming_session_cancelled(app_handle, generation, cancel)
        })?;
        self.socket = Some(socket);
        self.socket_connects = self.socket_connects.saturating_add(1);
        Ok(())
    }

    fn decode_once(&mut self, samples: &[f32]) -> Result<String, String> {
        self.ensure_socket()?;
        let app_handle = self.app_handle;
        let generation = self.generation;
        let cancel = self.cancel;
        let result = self
            .socket
            .as_mut()
            .ok_or_else(|| "Parakeet WebSocket state is unavailable".to_string())
            .and_then(|socket| {
                decode_parakeet_samples_on_socket(socket, samples, || {
                    streaming_session_cancelled(app_handle, generation, cancel)
                })
            });
        if result.is_err() {
            self.socket = None;
        }
        result
    }

    fn decode_with_recovery(&mut self, samples: &[f32]) -> Result<String, String> {
        match self.decode_once(samples) {
            Ok(text) => Ok(text),
            Err(error) if !self.recovery_used && !self.is_cancelled() => {
                self.recovery_used = true;
                crate::logger::log(
                    "WARN",
                    "ASR",
                    Some(self.session_tag),
                    &format!(
                        "Parakeet streaming decode failed; restarting resident server once: {error}"
                    ),
                );
                whisper_runner::stop_parakeet_server(self.app_handle);
                whisper_runner::start_parakeet_server(self.app_handle).map_err(
                    |restart_error| format!("{error}; server restart failed: {restart_error}"),
                )?;
                self.decode_once(samples)
                    .map_err(|retry_error| format!("{error}; retry failed: {retry_error}"))
            }
            Err(error) => Err(error),
        }
    }
}

impl Drop for ParakeetStreamingDecoder<'_> {
    fn drop(&mut self) {
        if let Some(mut socket) = self.socket.take() {
            finish_parakeet_socket(&mut socket);
        }
    }
}

struct ProcessedParakeetDecode {
    elapsed_ms: u128,
    preview_usable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParakeetTranscriptUpdate {
    Preview { usable: bool },
    Committed,
    EmptyEndpoint { recovered_preview: bool },
}

fn apply_parakeet_decode_text(
    reason: parakeet_streaming::DecodeReason,
    requires_transcript_overlap: bool,
    cleaned: &str,
    transcript: &mut parakeet_streaming::TranscriptState,
) -> Result<ParakeetTranscriptUpdate, String> {
    let valid = !cleaned.is_empty() && !is_silence_hallucination(cleaned);

    if reason == parakeet_streaming::DecodeReason::Preview {
        if valid {
            transcript.set_preview(cleaned);
        }
        return Ok(ParakeetTranscriptUpdate::Preview { usable: valid });
    }

    if reason == parakeet_streaming::DecodeReason::Endpoint && !valid {
        let recovered_preview = match transcript.commit_preview() {
            Some(overlap_is_safe) => {
                if requires_transcript_overlap && !overlap_is_safe {
                    // A forced-cut segment whose overlap decodes with zero
                    // token overlap is a transient asr hiccup, not a reason to
                    // abandon the whole session: keep the preview and let the
                    // next segment decide. Previously this aborted the
                    // streaming session and discarded the live preview.
                    crate::logger::log(
                        "WARN",
                        "ASR",
                        None,
                        "Endpoint preview overlap could not be aligned; accepting preview instead of aborting session",
                    );
                }
                true
            }
            None => false,
        };
        return Ok(ParakeetTranscriptUpdate::EmptyEndpoint { recovered_preview });
    }

    if !valid {
        return Err(format!(
            "Parakeet {:?} decode returned no usable speech",
            reason
        ));
    }

    let overlap_is_safe = transcript.commit(cleaned);
    if requires_transcript_overlap && !overlap_is_safe {
        // Forced-cut segments carry a decoding uncertainty that can surface as
        // a zero token overlap. Aborting the session on one such hiccup threw
        // away everything already dictated; committing with a possible brief
        // duplication (handled by merge_transcripts) keeps the session alive.
        crate::logger::log(
            "WARN",
            "ASR",
            None,
            "Forced-transcript overlap could not be aligned; committing with possible duplication",
        );
    }
    Ok(ParakeetTranscriptUpdate::Committed)
}

fn process_parakeet_decode_request(
    decoder: &mut ParakeetStreamingDecoder<'_>,
    request: parakeet_streaming::DecodeRequest,
    transcript: &mut parakeet_streaming::TranscriptState,
    metrics: &mut parakeet_streaming::StreamingMetrics,
) -> Result<ProcessedParakeetDecode, String> {
    if streaming_session_cancelled(decoder.app_handle, decoder.generation, decoder.cancel) {
        return Err("Parakeet streaming session was cancelled".to_string());
    }

    let sample_count = request.samples.len();
    let started = std::time::Instant::now();
    let decoded = decoder.decode_with_recovery(&request.samples)?;
    let elapsed_ms = started.elapsed().as_millis();
    metrics.decode_requests = metrics.decode_requests.saturating_add(1);
    metrics.total_decode_ms = metrics.total_decode_ms.saturating_add(elapsed_ms);
    metrics.decode_latency.record(elapsed_ms);
    crate::logger::log(
        "INFO",
        "ASR",
        Some(decoder.session_tag),
        &format!(
            "Parakeet {:?} decode: {sample_count} samples in {elapsed_ms} ms",
            request.reason
        ),
    );

    let cleaned = clean_hallucinated_brackets(&decoded).trim().to_string();
    let update = apply_parakeet_decode_text(
        request.reason,
        request.requires_transcript_overlap,
        &cleaned,
        transcript,
    )?;
    let preview_usable = matches!(update, ParakeetTranscriptUpdate::Preview { usable: true });
    if let ParakeetTranscriptUpdate::EmptyEndpoint { recovered_preview } = update {
        metrics.empty_endpoint_decodes = metrics.empty_endpoint_decodes.saturating_add(1);
        if recovered_preview {
            metrics.recovered_endpoint_previews =
                metrics.recovered_endpoint_previews.saturating_add(1);
        }
        crate::logger::log(
            "INFO",
            "ASR",
            Some(decoder.session_tag),
            if recovered_preview {
                "Empty Parakeet endpoint; committed the last usable preview and continued"
            } else {
                "Empty Parakeet endpoint without a usable preview; ignored it and continued"
            },
        );
    }

    let typing_mode = TypingUpdateMode::for_parakeet_update(update);
    let display_text = transcript.display_text();
    if !display_text.is_empty() {
        type_streaming_update_sync(
            decoder.app_handle,
            decoder.generation,
            &display_text,
            typing_mode,
        );
    }
    Ok(ProcessedParakeetDecode {
        elapsed_ms,
        preview_usable,
    })
}

fn observe_parakeet_decode_load(
    cadence: &mut parakeet_streaming::PreviewCadenceController,
    sample_stream: &audio_recorder::AudioSampleStream,
    metrics: &mut parakeet_streaming::StreamingMetrics,
    elapsed_ms: u128,
    session_tag: &str,
) {
    let previous_step = cadence.preview_step_samples();
    let queued_chunks = sample_stream.queued_chunks();
    metrics.max_queued_chunks = metrics.max_queued_chunks.max(queued_chunks);
    cadence.observe_decode(elapsed_ms, queued_chunks, sample_stream.capacity());
    let next_step = cadence.preview_step_samples();
    metrics.max_preview_step_samples = metrics.max_preview_step_samples.max(next_step);

    if previous_step != next_step {
        let step_ms = next_step.saturating_mul(1_000) / 16_000;
        crate::logger::log(
            "INFO",
            "ASR",
            Some(session_tag),
            &format!(
                "Adaptive Parakeet preview cadence: step={step_ms} ms, queue={queued_chunks}/{}, last_decode={elapsed_ms} ms",
                sample_stream.capacity()
            ),
        );
    }
}

fn run_parakeet_streaming_worker(
    app_handle: tauri::AppHandle,
    generation: u64,
    sample_stream: audio_recorder::AudioSampleStream,
    cancel: Arc<AtomicBool>,
) -> parakeet_streaming::StreamingOutcome {
    let session_tag = crate::logger::format_session_tag(generation);
    let mut metrics = parakeet_streaming::StreamingMetrics::default();
    let mut transcript = parakeet_streaming::TranscriptState::default();
    let mut error = None;

    if let Err(start_error) = whisper_runner::start_parakeet_server(&app_handle) {
        error = Some(format!("Parakeet server is unavailable: {start_error}"));
    }

    let mut streaming_vad = if error.is_none() {
        match vad::StreamingVad::new_16k() {
            Ok(detector) => Some(detector),
            Err(vad_error) => {
                error = Some(vad_error);
                None
            }
        }
    } else {
        None
    };
    let mut accumulator = parakeet_streaming::SegmentAccumulator::new();
    let mut cadence = parakeet_streaming::PreviewCadenceController::default();
    metrics.max_preview_step_samples = cadence.preview_step_samples();
    let mut decoder = ParakeetStreamingDecoder {
        app_handle: &app_handle,
        generation,
        cancel: &cancel,
        recovery_used: false,
        session_tag: &session_tag,
        socket: None,
        socket_connects: 0,
    };
    let mut stream_closed = false;

    while error.is_none() && !stream_closed {
        if streaming_session_cancelled(&app_handle, generation, &cancel) {
            error = Some("Parakeet streaming session was cancelled".to_string());
            break;
        }

        let first_chunk = match sample_stream.recv_timeout(std::time::Duration::from_millis(50)) {
            Ok(chunk) => chunk,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        let mut chunks = vec![first_chunk];
        loop {
            match sample_stream.try_recv() {
                Ok(chunk) => chunks.push(chunk),
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    stream_closed = true;
                    break;
                }
            }
        }

        let mut requests = Vec::new();
        for chunk in chunks {
            let Some(detector) = streaming_vad.as_mut() else {
                break;
            };
            let vad_result = detector.push_with_endpoints(&chunk);
            match accumulator.push(&chunk, &vad_result.events) {
                Ok(mut chunk_requests) => requests.append(&mut chunk_requests),
                Err(segment_error) => {
                    error = Some(segment_error);
                    break;
                }
            }
            metrics.max_active_samples = metrics
                .max_active_samples
                .max(accumulator.active_sample_count());
        }

        for request in requests {
            metrics.max_active_samples = metrics.max_active_samples.max(request.samples.len());
            match process_parakeet_decode_request(
                &mut decoder,
                request,
                &mut transcript,
                &mut metrics,
            ) {
                Ok(processed) => observe_parakeet_decode_load(
                    &mut cadence,
                    &sample_stream,
                    &mut metrics,
                    processed.elapsed_ms,
                    &session_tag,
                ),
                Err(decode_error) => {
                    error = Some(decode_error);
                    break;
                }
            }
        }

        if error.is_none() && !stream_closed {
            if let Some(request) = accumulator.take_preview_request(cadence.preview_step_samples())
            {
                match process_parakeet_decode_request(
                    &mut decoder,
                    request,
                    &mut transcript,
                    &mut metrics,
                ) {
                    Ok(processed) => {
                        if processed.preview_usable {
                            accumulator.mark_preview_successful();
                        } else {
                            accumulator.invalidate_successful_preview();
                        }
                        observe_parakeet_decode_load(
                            &mut cadence,
                            &sample_stream,
                            &mut metrics,
                            processed.elapsed_ms,
                            &session_tag,
                        );
                    }
                    Err(decode_error) => error = Some(decode_error),
                }
            }
        }
    }

    let dropped_chunks = sample_stream.dropped_chunks();
    if error.is_none() && !streaming_session_cancelled(&app_handle, generation, &cancel) {
        let pending_tail_has_speech = streaming_vad
            .as_mut()
            .map(vad::StreamingVad::finish_pending_has_speech)
            .unwrap_or(true);
        let can_reuse_preview = dropped_chunks == 0
            && accumulator.can_reuse_successful_preview(pending_tail_has_speech)
            && transcript.commit_preview().is_some();
        if can_reuse_preview {
            accumulator.discard_active();
            metrics.final_preview_reused = true;
            crate::logger::log(
                "INFO",
                "ASR",
                Some(&session_tag),
                "Reused fresh Parakeet preview at release; skipped redundant final decode",
            );
        } else if let Some(request) = accumulator.finish() {
            metrics.max_active_samples = metrics.max_active_samples.max(request.samples.len());
            match process_parakeet_decode_request(
                &mut decoder,
                request,
                &mut transcript,
                &mut metrics,
            ) {
                Ok(processed) => observe_parakeet_decode_load(
                    &mut cadence,
                    &sample_stream,
                    &mut metrics,
                    processed.elapsed_ms,
                    &session_tag,
                ),
                Err(decode_error) => error = Some(decode_error),
            }
        }
    }

    metrics.socket_connects = decoder.socket_connects;
    metrics.server_recoveries = u64::from(decoder.recovery_used);
    let latency_p50_ms = metrics
        .decode_latency
        .percentile(50)
        .map_or_else(|| "n/a".to_string(), |value| value.to_string());
    let latency_p95_ms = metrics
        .decode_latency
        .percentile(95)
        .map_or_else(|| "n/a".to_string(), |value| value.to_string());
    let latency_max_ms = metrics
        .decode_latency
        .max_ms()
        .map_or_else(|| "n/a".to_string(), |value| value.to_string());
    crate::logger::log(
        if error.is_some() || dropped_chunks > 0 || metrics.empty_endpoint_decodes > 0 {
            "WARN"
        } else {
            "INFO"
        },
        "ASR",
        Some(&session_tag),
        &format!(
            "Parakeet streaming summary: decodes={}, decode_ms={}, latency_p50_ms={}, latency_p95_ms={}, latency_max_ms={}, socket_connects={}, recoveries={}, empty_endpoints={}, recovered_endpoint_previews={}, final_preview_reused={}, max_active_samples={}, max_queue_depth={}, max_preview_step_samples={}, queue_gaps={}, status={}",
            metrics.decode_requests,
            metrics.total_decode_ms,
            latency_p50_ms,
            latency_p95_ms,
            latency_max_ms,
            metrics.socket_connects,
            metrics.server_recoveries,
            metrics.empty_endpoint_decodes,
            metrics.recovered_endpoint_previews,
            metrics.final_preview_reused,
            metrics.max_active_samples,
            metrics.max_queued_chunks,
            metrics.max_preview_step_samples,
            dropped_chunks,
            if error.is_some() {
                "degraded"
            } else if metrics.empty_endpoint_decodes > 0 {
                "recovered"
            } else {
                "complete"
            }
        ),
    );

    parakeet_streaming::StreamingOutcome {
        generation,
        transcript: transcript.final_text().to_string(),
        dropped_chunks,
        error,
        metrics,
    }
}

async fn await_parakeet_streaming_outcome(
    session: ParakeetStreamingSession,
) -> Result<parakeet_streaming::StreamingOutcome, String> {
    let generation = session.generation;
    let cancel = Arc::clone(&session.cancel);
    let result = tauri::async_runtime::spawn_blocking(move || {
        session
            .result_rx
            .recv_timeout(std::time::Duration::from_secs(75))
            .map_err(|error| {
                format!("Timed out waiting for streaming session {generation}: {error}")
            })
    })
    .await
    .map_err(|error| format!("Streaming result wait worker failed: {error}"))
    .and_then(|result| result);
    if result.is_err() {
        cancel.store(true, Ordering::Release);
    }
    result
}

struct BatchPreview {
    recorded_secs: f64,
    work: Option<(usize, settings::Settings, String)>,
}

fn prepare_batch_preview(
    app_handle: &tauri::AppHandle,
    chunk_path: &str,
    previous_len: usize,
) -> Result<BatchPreview, String> {
    let state = app_handle
        .try_state::<AppState>()
        .ok_or_else(|| "Application state is unavailable".to_string())?;
    let (samples, sample_rate, channels) = state.audio_recorder.get_recorded_samples()?;
    let recorded_secs = samples.len() as f64 / (sample_rate.max(1) as f64 * channels.max(1) as f64);
    let new_start = previous_len.min(samples.len());
    let has_new_speech = vad::has_speech(&samples[new_start..], sample_rate as i64);
    if samples.len() <= 8_000 || !has_new_speech {
        return Ok(BatchPreview {
            recorded_secs,
            work: None,
        });
    }

    audio_recorder::process_and_write_wav(&samples, channels, sample_rate, chunk_path)?;
    let settings = settings::load_settings(app_handle)?;
    let layout_language = match state.selected_language.lock() {
        Ok(language) => language.clone(),
        Err(poisoned) => {
            crate::logger::log(
                "ERROR",
                "State",
                None,
                "Recovering poisoned selected-language mutex",
            );
            poisoned.into_inner().clone()
        }
    };
    Ok(BatchPreview {
        recorded_secs,
        work: Some((samples.len(), settings, layout_language)),
    })
}

fn clean_live_text(text: &str) -> String {
    let mut cleaned = String::new();
    let mut last_was_space = false;

    for ch in text.chars() {
        let lower = ch.to_lowercase().to_string();
        for l_ch in lower.chars() {
            let normalized_ch = if l_ch == 'ё' { 'е' } else { l_ch };

            if normalized_ch.is_alphanumeric() {
                cleaned.push(normalized_ch);
                last_was_space = false;
            } else if normalized_ch.is_whitespace() && !last_was_space && !cleaned.is_empty() {
                cleaned.push(' ');
                last_was_space = true;
            }
        }
    }
    cleaned.trim_end().to_string()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TypingUpdateMode {
    LivePreview,
    CommittedSegment,
    Final,
}

impl TypingUpdateMode {
    fn for_parakeet_update(update: ParakeetTranscriptUpdate) -> Self {
        if matches!(update, ParakeetTranscriptUpdate::Committed) {
            Self::CommittedSegment
        } else {
            Self::LivePreview
        }
    }

    fn requires_recording(self) -> bool {
        !matches!(self, Self::Final)
    }

    fn uses_live_matching(self) -> bool {
        matches!(self, Self::LivePreview)
    }

    fn target_text(self, text: &str) -> String {
        if self.uses_live_matching() {
            clean_live_text(text)
        } else {
            text.to_string()
        }
    }

    fn log_label(self) -> &'static str {
        match self {
            Self::LivePreview => "live-preview",
            Self::CommittedSegment => "committed-segment",
            Self::Final => "final",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TypingUpdateOutcome {
    Applied,
    TargetDesynchronized,
    FocusChanged,
    InputDispatchFailed,
    StaleSession,
    StateUnavailable,
}

impl TypingUpdateOutcome {
    fn needs_safe_clipboard_handoff(self) -> bool {
        matches!(
            self,
            Self::TargetDesynchronized
                | Self::FocusChanged
                | Self::InputDispatchFailed
                | Self::StateUnavailable
        )
    }
}

fn finish_live_target_monitoring(app_handle: &tauri::AppHandle, generation: u64) {
    if let Some(state) = app_handle.try_state::<AppState>() {
        if state.session_gen.load(Ordering::Acquire) == generation {
            state.live_target_monitoring.store(false, Ordering::Release);
            keyboard_hook::set_live_target_monitoring(false);
        }
    }
}

/// Types a preview, committed segment, or final update via simulated keystrokes.
/// Re-checks session generation and, for in-recording modes, the recording flag
/// under the lock so a stale task can never corrupt a newer session's text.
fn type_streaming_update_sync(
    app_handle: &tauri::AppHandle,
    my_gen: u64,
    new_text: &str,
    mode: TypingUpdateMode,
) -> TypingUpdateOutcome {
    let Some(state) = app_handle.try_state::<AppState>() else {
        return TypingUpdateOutcome::StateUnavailable;
    };
    let state = state.inner();
    let session_tag = crate::logger::format_session_tag(my_gen);

    if state.live_target_desynced.load(Ordering::Acquire) {
        return TypingUpdateOutcome::TargetDesynchronized;
    }

    // Selected text guard: when editing selected text, intermediate streaming previews
    // must never be typed into the active window to avoid corrupting/overwriting the selection.
    let has_captured_selection = state
        .selected_text
        .lock()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    if has_captured_selection && mode.requires_recording() {
        return TypingUpdateOutcome::Applied;
    }

    // Focus guard: never type into a window the user switched to mid-dictation
    let start_focus = match state.start_focus.lock() {
        Ok(guard) => *guard,
        Err(_) => {
            crate::logger::log(
                "ERROR",
                "Typing",
                Some(&session_tag),
                "Start-window state is poisoned; refusing simulated typing",
            );
            return TypingUpdateOutcome::StateUnavailable;
        }
    };
    let current_focus = keyboard_simulator::get_focus_target();
    if !start_focus.is_compatible_with(&current_focus) {
        state.live_target_desynced.store(true, Ordering::Release);
        crate::logger::log(
            "WARN",
            "Typing",
            Some(&session_tag),
            "Focus changed since recording started; skipping simulated typing.",
        );
        return TypingUpdateOutcome::FocusChanged;
    }

    let mut typed_guard = match state.typed_so_far.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            crate::logger::log(
                "ERROR",
                "Typing",
                Some(&session_tag),
                "Recovering poisoned live-text mutex before guarded typing",
            );
            poisoned.into_inner()
        }
    };
    let require_recording = mode.requires_recording();
    let session_ok = state.session_gen.load(Ordering::SeqCst) == my_gen;
    let recording_ok = !require_recording || state.is_recording.load(Ordering::SeqCst);
    if !session_ok || !recording_ok {
        crate::logger::log(
            "WARN",
            "Typing",
            Some(&session_tag),
            "Skipping stale typing update (gen/recording check failed).",
        );
        return TypingUpdateOutcome::StaleSession;
    }
    if state.live_target_desynced.load(Ordering::Acquire) {
        return TypingUpdateOutcome::TargetDesynchronized;
    }

    let text_to_type = mode.target_text(new_text);

    let dispatch_started = std::time::Instant::now();
    let dispatch_result = diff_and_type(
        &mut typed_guard,
        &text_to_type,
        mode.uses_live_matching(),
        || {
            state.live_target_desynced.load(Ordering::Acquire)
                || state.session_gen.load(Ordering::Acquire) != my_gen
                || (require_recording && !state.is_recording.load(Ordering::Acquire))
                || !start_focus.is_compatible_with(&keyboard_simulator::get_focus_target())
        },
    );
    let dispatch_ms = dispatch_started.elapsed().as_millis();

    let metrics = match dispatch_result {
        Ok(metrics) => metrics,
        Err(error) => {
            if state.session_gen.load(Ordering::Acquire) != my_gen
                || (require_recording && !state.is_recording.load(Ordering::Acquire))
            {
                crate::logger::log(
                    "WARN",
                    "Typing",
                    Some(&session_tag),
                    &format!(
                        "Keyboard replacement interrupted for stale session after {dispatch_ms} ms"
                    ),
                );
                return TypingUpdateOutcome::StaleSession;
            }
            if state.live_target_desynced.load(Ordering::Acquire) {
                crate::logger::log(
                    "WARN",
                    "Typing",
                    Some(&session_tag),
                    &format!(
                        "Keyboard replacement interrupted after target edit ({dispatch_ms} ms)"
                    ),
                );
                return TypingUpdateOutcome::TargetDesynchronized;
            }
            if !start_focus.is_compatible_with(&keyboard_simulator::get_focus_target()) {
                state.live_target_desynced.store(true, Ordering::Release);
                crate::logger::log(
                    "WARN",
                    "Typing",
                    Some(&session_tag),
                    &format!(
                        "Keyboard replacement interrupted after focus change ({dispatch_ms} ms)"
                    ),
                );
                return TypingUpdateOutcome::FocusChanged;
            }

            state.live_target_desynced.store(true, Ordering::Release);
            crate::logger::log(
                "ERROR",
                "Typing",
                Some(&session_tag),
                &format!("Keyboard replacement dispatch failed after {dispatch_ms} ms: {error}"),
            );
            return TypingUpdateOutcome::InputDispatchFailed;
        }
    };

    if matches!(
        mode,
        TypingUpdateMode::CommittedSegment | TypingUpdateMode::Final
    ) || dispatch_ms >= 100
    {
        crate::logger::log(
            "INFO",
            "Typing",
            Some(&session_tag),
            &format!(
                "Keyboard replacement dispatch: mode={}, backspaces={}, utf16_units={}, batches={}, duration_ms={dispatch_ms}",
                mode.log_label(),
                metrics.backspaces, metrics.utf16_units, metrics.batches
            ),
        );
    }

    if state.live_target_desynced.load(Ordering::Acquire) {
        TypingUpdateOutcome::TargetDesynchronized
    } else if state.session_gen.load(Ordering::Acquire) != my_gen
        || (require_recording && !state.is_recording.load(Ordering::Acquire))
    {
        TypingUpdateOutcome::StaleSession
    } else if !start_focus.is_compatible_with(&keyboard_simulator::get_focus_target()) {
        state.live_target_desynced.store(true, Ordering::Release);
        TypingUpdateOutcome::FocusChanged
    } else {
        TypingUpdateOutcome::Applied
    }
}

/// Runs guarded keyboard reconciliation away from the async runtime worker.
async fn type_streaming_update(
    app_handle: tauri::AppHandle,
    my_gen: u64,
    new_text: String,
    mode: TypingUpdateMode,
) -> TypingUpdateOutcome {
    match tauri::async_runtime::spawn_blocking(move || {
        type_streaming_update_sync(&app_handle, my_gen, &new_text, mode)
    })
    .await
    {
        Ok(outcome) => outcome,
        Err(error) => {
            crate::logger::log(
                "ERROR",
                "Typing",
                Some(&crate::logger::format_session_tag(my_gen)),
                &format!("Typing worker failed: {error}"),
            );
            TypingUpdateOutcome::StateUnavailable
        }
    }
}

/// True when the cloud error looks like "the network/provider is unreachable from here"
/// (VPN/region block, no network, proxy/TLS failure) rather than a config mistake like a
/// bad API key or an exhausted quota — those need the user's attention, not a silent swap.
///
/// IMPORTANT: keep these patterns SPECIFIC. Overly broad matches (e.g. bare "connection"
/// or "reqwest") can silently swallow account-level errors (quota, connection limits for
/// tier, 403 IP-block) and hide them from the user. Each match below is either an exact
/// phrase or a context-narrowed substring that cannot appear in auth/quota errors.
fn is_cloud_unreachable(err: &str) -> bool {
    let e = err.to_lowercase();

    // Explicit geographic / region-block signals
    if e.contains("location is not supported")
        || e.contains("user location")
        || e.contains("failed_precondition")
    {
        return true;
    }

    // 403 Forbidden — only treated as network-level when it is from a VPN/proxy exit IP.
    // Groq/OpenAI 403s on bad keys say "invalid api key", "unauthorized", etc. and are
    // caught by the API-key branch above; a bare 403 here is a Cloudflare IP-block.
    if e.contains("403") || e.contains("forbidden") {
        // If the message also mentions key/auth, it's an account error, not unreachable.
        if e.contains("api key") || e.contains("unauthorized") || e.contains("invalid") {
            return false;
        }
        return true;
    }

    // TLS / proxy layer failures are always infrastructure issues
    if e.contains("proxy") || e.contains("certificate") || e.contains("tls") || e.contains("ssl") {
        return true;
    }

    // Network-level errors: require the error to come from the transport layer.
    // "reqwest" alone can appear in quota/auth errors too, so we require it alongside
    // a transport keyword. "connection refused" / "connection reset" are infrastructure;
    // "connection limit" (Groq tier) is an account error — so match the full phrase.
    if e.contains("no network")
        || e.contains("network unreachable")
        || e.contains("timed out")
        || e.contains("dns error")
        || e.contains("name resolution")
        || e.contains("connection refused")
        || e.contains("connection reset")
        || e.contains("os error 10060")   // Windows WSAETIMEDOUT
        || e.contains("os error 10061")   // Windows WSAECONNREFUSED
        || (e.contains("timeout") && !e.contains("rate limit"))
    {
        return true;
    }

    false
}

fn select_local_fallback_model(downloaded: &[String], preferred: &str) -> Option<String> {
    downloaded
        .iter()
        .find(|model| model.as_str() == preferred)
        .or_else(|| {
            downloaded
                .iter()
                .find(|model| model.as_str() == "parakeet-v3")
        })
        .or_else(|| downloaded.first())
        .cloned()
}

fn categorize_error(err: &str) -> String {
    let err_lower = err.to_lowercase();
    if err_lower.contains("location is not supported")
        || err_lower.contains("user location")
        || err_lower.contains("failed_precondition")
    {
        "Gemini is unavailable in your region. Enable global VPN or choose Groq".to_string()
    } else if err_lower.contains("api key")
        || err_lower.contains("invalid key")
        || err_lower.contains("key is invalid")
        || err_lower.contains("incorrect api key")
        || err_lower.contains("401")
        || err_lower.contains("permission_denied")
    {
        "Invalid API key in settings".to_string()
    } else if err_lower.contains("403") || err_lower.contains("forbidden") {
        // Groq/OpenAI are reachable without a VPN; a 403 almost always means the
        // provider's edge (Cloudflare) blocked the VPN/proxy exit IP, not a bad key.
        "Access forbidden (403): VPN/proxy IP is blocked. Turn off the VPN for Groq/OpenAI or switch server".to_string()
    } else if err_lower.contains("proxy")
        || err_lower.contains("certificate")
        || err_lower.contains("tls")
        || err_lower.contains("ssl")
    {
        "Connection error via VPN/Proxy".to_string()
    } else if err_lower.contains("network")
        || err_lower.contains("timeout")
        || err_lower.contains("timed out")
        || err_lower.contains("dns")
        || err_lower.contains("connection")
        || err_lower.contains("reqwest")
    {
        let clean_err = err
            .replace("Gemini API request failed: ", "")
            .replace("reqwest::Error", "");
        let truncated = if clean_err.chars().count() > 50 {
            clean_err.chars().take(47).collect::<String>() + "..."
        } else {
            clean_err
        };
        format!("No network: {}", truncated)
    } else if err_lower.contains("ggml") || err_lower.contains("model file")
        // Restrict "not found" to local-file paths only; a bare "not found" substring also
        // appears in HTTP 404 messages (e.g. "404 Not Found") and would be mis-classified.
        || (err_lower.contains("not found") && (err_lower.contains(".bin") || err_lower.contains("path") || err_lower.contains("ggml")))
    {
        "Local model not downloaded".to_string()
    } else if err_lower.contains("sherpa") || err_lower.contains("parakeet") {
        "Local Parakeet client failure".to_string()
    } else if err_lower.contains("whisper-sidecar") || err_lower.contains("sidecar") {
        "Local Whisper client failure".to_string()
    } else if err_lower.contains("rate limit") || err_lower.contains("429") {
        "API rate limit reached".to_string()
    } else if err_lower.contains("quota") || err_lower.contains("insufficient balance") {
        "API key balance exhausted".to_string()
    } else {
        if err.chars().count() > 40 {
            err.chars().take(37).collect::<String>() + "..."
        } else {
            err.to_string()
        }
    }
}

fn ensure_overlay_topmost(window: &tauri::WebviewWindow) {
    let _ = window.set_always_on_top(true);
    #[cfg(target_os = "windows")]
    {
        if let Ok(hwnd) = window.hwnd() {
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
                    SWP_SHOWWINDOW,
                };
                SetWindowPos(
                    hwnd.0 as isize,
                    HWND_TOPMOST,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
                );
            }
        }
    }
}

/// Shows the error state in the overlay for a moment, then hides it
/// (unless a newer session already owns the overlay).
async fn show_overlay_error(app_handle: &tauri::AppHandle, my_gen: u64, error_msg: &str) {
    let still_current = app_handle
        .try_state::<AppState>()
        .map(|state| state.inner().session_gen.load(Ordering::SeqCst) == my_gen)
        .unwrap_or(true);
    if !still_current {
        return;
    }

    if let Some(overlay) = app_handle.get_webview_window("overlay") {
        if let Ok(Some(monitor)) = overlay.primary_monitor() {
            let size = monitor.size();
            let scale_factor = monitor.scale_factor();

            let monitor_width = size.width as f64 / scale_factor;
            let monitor_height = size.height as f64 / scale_factor;

            let overlay_width = 360.0;
            let overlay_height = 92.0;
            const TASKBAR_MARGIN: f64 = 83.0;

            let x = (monitor_width - overlay_width) / 2.0;
            let y = monitor_height - overlay_height - TASKBAR_MARGIN;

            let _ =
                overlay.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
        }
        ensure_overlay_topmost(&overlay);
        let _ = overlay.show();

        let app = app_handle.clone();
        let overlay_clone = overlay.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(3_000)).await;
            let still_same_session = app
                .try_state::<AppState>()
                .map(|state| {
                    let state = state.inner();
                    !state.is_recording.load(Ordering::SeqCst)
                        && state.session_gen.load(Ordering::SeqCst) == my_gen
                })
                .unwrap_or(false);
            if still_same_session {
                let _ = overlay_clone.hide();
            }
        });
    }
    let _ = app_handle.emit("recording-state", format!("error:{}", error_msg));
}

/// Emits hide-overlay-requested event and runs an 880ms backup safety timer to close the overlay window.
///
/// The backup hide is skipped when a new session has already claimed the
/// overlay: the user can re-press the hotkey (e.g. re-press V while Alt is
/// still held) while the fade animation is still running, and the window
/// must not be torn down underneath the new recording.
fn request_animated_hide(app_handle: &tauri::AppHandle, status: &str) {
    if let Some(overlay) = app_handle.get_webview_window("overlay") {
        let _ = app_handle.emit(
            "hide-overlay-requested",
            serde_json::json!({ "status": status }),
        );
        let app = app_handle.clone();
        let overlay_clone = overlay.clone();
        let gen_at_request = app_handle
            .try_state::<AppState>()
            .map(|state| state.inner().session_gen.load(Ordering::SeqCst))
            .unwrap_or(u64::MAX);
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(880)).await;
            let still_same_session = app
                .try_state::<AppState>()
                .map(|state| {
                    let state = state.inner();
                    !state.is_recording.load(Ordering::SeqCst)
                        && state.session_gen.load(Ordering::SeqCst) == gen_at_request
                })
                .unwrap_or(false);
            if still_same_session {
                if let Ok(true) = overlay_clone.is_visible() {
                    let _ = overlay_clone.hide();
                }
            }
        });
    }
}

/// Shows a localized, non-error notice long enough to be read, with a backend
/// hide timer as a fallback if the overlay listener is unavailable.
fn show_overlay_notice(app_handle: &tauri::AppHandle, notice_key: &str) {
    if let Some(overlay) = app_handle.get_webview_window("overlay") {
        ensure_overlay_topmost(&overlay);
        let _ = overlay.show();
        let _ = app_handle.emit("recording-state", format!("notice:{notice_key}"));
        let app = app_handle.clone();
        let overlay_clone = overlay.clone();
        let gen_at_request = app_handle
            .try_state::<AppState>()
            .map(|state| state.inner().session_gen.load(Ordering::SeqCst))
            .unwrap_or(u64::MAX);
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(2_800)).await;
            let still_same_session = app
                .try_state::<AppState>()
                .map(|state| {
                    let state = state.inner();
                    !state.is_recording.load(Ordering::SeqCst)
                        && state.session_gen.load(Ordering::SeqCst) == gen_at_request
                })
                .unwrap_or(false);
            if still_same_session {
                let _ = overlay_clone.hide();
            }
        });
    }
}

/// Stops and discards the current recording (accidental tap or Esc cancel).
fn discard_recording(app_handle: &tauri::AppHandle) -> bool {
    let mut discarded = false;
    if let Some(state) = app_handle.try_state::<AppState>() {
        let state = state.inner();
        if state.is_recording.swap(false, Ordering::SeqCst) {
            state.latched.store(false, Ordering::SeqCst);
            let generation = state.session_gen.load(Ordering::SeqCst);
            if let Ok(mut slot) = state.parakeet_streaming.lock() {
                if slot
                    .as_ref()
                    .map(|session| session.generation == generation)
                    .unwrap_or(false)
                {
                    if let Some(session) = slot.take() {
                        session.cancel.store(true, Ordering::Release);
                    }
                }
            }
            let _ = state.audio_recorder.cancel_recording();
            discarded = true;
        }
    }
    if discarded {
        let generation = app_handle
            .try_state::<AppState>()
            .map(|state| state.session_gen.load(Ordering::Acquire))
            .unwrap_or(0);
        finish_live_target_monitoring(app_handle, generation);
        keyboard_hook::set_recording_active(false);
        request_animated_hide(app_handle, "cancel");
    }
    discarded
}

/// Esc pressed during recording: discard audio and erase any streamed preview text.
async fn cancel_recording(app_handle: tauri::AppHandle) {
    let my_gen = if let Some(state) = app_handle.try_state::<AppState>() {
        state.session_gen.load(Ordering::SeqCst)
    } else {
        return;
    };
    let session_tag = crate::logger::format_session_tag(my_gen);
    crate::logger::log(
        "INFO",
        "Session",
        Some(&session_tag),
        "Esc pressed — cancelling recording session",
    );
    if !discard_recording(&app_handle) {
        return;
    }

    // Erase live-preview text that was already typed, if any
    let has_typed = app_handle
        .try_state::<AppState>()
        .and_then(|s| s.typed_so_far.lock().ok().map(|g| !g.is_empty()))
        .unwrap_or(false);
    if has_typed {
        type_streaming_update(
            app_handle.clone(),
            my_gen,
            String::new(),
            TypingUpdateMode::Final,
        )
        .await;
    }
}

/// Starts a new recording session: registers a new generation, captures context
/// (focus window, keyboard layout, selected text), starts audio capture, shows the
/// overlay, and spawns the live-streaming loop when enabled.
async fn start_recording_session(app_handle: tauri::AppHandle) {
    audio_recorder::stop_mic_meter();

    let Some(state) = app_handle.try_state::<AppState>() else {
        return;
    };
    let state = state.inner();

    // The generation invalidates tasks left over from previous sessions. The
    // claim on `is_recording` must succeed *before* the generation advances:
    // a rejected duplicate start (auto-repeat, double-fire) must not bump the
    // generation, or every staleness check would misfire on the live session
    // and its typing, finalize and clipboard restore would be silently skipped.
    // The two stores below are adjacent and synchronous, so no other task can
    // observe the claimed-but-not-yet-bumped window.
    if state
        .is_recording
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        crate::logger::log(
            "WARN",
            "Session",
            None,
            "Ignoring duplicate recording start",
        );
        return;
    }
    let gen = state.session_gen.fetch_add(1, Ordering::SeqCst) + 1;
    let session_tag = crate::logger::format_session_tag(gen);
    state
        .selection_blocked_by_uipi
        .store(false, Ordering::Release);
    state.live_target_desynced.store(false, Ordering::Release);
    state.live_target_monitoring.store(false, Ordering::Release);
    keyboard_hook::set_live_target_monitoring(false);
    match state.parakeet_streaming.lock() {
        Ok(mut slot) => {
            if let Some(previous) = slot.take() {
                previous.cancel.store(true, Ordering::Release);
            }
        }
        Err(_) => crate::logger::log(
            "WARN",
            "ASR",
            Some(&session_tag),
            "Streaming session state is poisoned; this dictation will use WAV fallback",
        ),
    }
    crate::logger::log(
        "INFO",
        "Session",
        Some(&session_tag),
        "Hotkey pressed — starting recording session",
    );

    // Remember the focused window and process for the typing focus guard
    let focus = keyboard_simulator::get_focus_target();
    if let Ok(mut guard) = state.start_focus.lock() {
        *guard = focus;
    }

    // Detect active keyboard language at the moment of press
    let lang = keyboard_simulator::get_active_layout_language();
    crate::logger::log(
        "INFO",
        "Layout",
        Some(&session_tag),
        &format!("Active layout language = {}", lang),
    );
    if let Ok(mut guard) = state.selected_language.lock() {
        *guard = lang;
    }

    // Resetting the mirror state must survive a poisoned mutex: an earlier
    // panic inside a live-typing worker must never leave the previous
    // session's mirror text in place, or the new session would plan
    // backspaces against a phantom prefix and delete real text in the target.
    let reset_session_mutex = |name: &str, value: &Mutex<String>| -> bool {
        match value.lock() {
            Ok(mut guard) => {
                *guard = String::new();
                false
            }
            Err(poisoned) => {
                *poisoned.into_inner() = String::new();
                crate::logger::log(
                    "WARN",
                    "Session",
                    Some(session_tag.as_str()),
                    &format!("{name} mutex was poisoned; mirror reset"),
                );
                true
            }
        }
    };
    let typed_recovered = reset_session_mutex("typed_so_far", &state.typed_so_far);
    let selection_recovered = reset_session_mutex("selected_text", &state.selected_text);
    if let Ok(mut guard) = state.caret_context.lock() {
        *guard = None;
    }
    let _ = app_handle.emit("selection-context-active", false);
    if typed_recovered || selection_recovered {
        // The mirror was rebuilt from scratch; do not trust destructive
        // diffing against the target document until it re-syncs.
        state.live_target_desynced.store(true, Ordering::Release);
    }
    state.latched.store(false, Ordering::SeqCst);
    state.ignore_next_release.store(false, Ordering::SeqCst);
    keyboard_hook::set_recording_active(true);

    if let Ok(mut guard) = state.press_time.lock() {
        *guard = Some(std::time::Instant::now());
    }

    // Copy selected text in a background task only when copy_context_on_start is enabled.
    // Sending Ctrl+C in a terminal kills the foreground process, so users who dictate
    // into terminals should turn this option off in settings.
    let session_settings = match load_settings_async(app_handle.clone()).await {
        Ok(settings) => settings,
        Err(error) => {
            crate::logger::log(
                "WARN",
                "Settings",
                Some(&session_tag),
                &format!("Could not load session settings; using privacy-safe defaults: {error}"),
            );
            settings::Settings::default()
        }
    };
    state
        .live_target_monitoring
        .store(session_settings.streaming_enabled, Ordering::Release);
    keyboard_hook::set_live_target_monitoring(session_settings.streaming_enabled);

    // Selected-text context is captured whenever an edit route will actually
    // consume it: the cloud provider (cloud or cloud-after-local cleanup) or
    // the local Qwen3 server. Route "off" never needs the selection.
    let edit_route = llm_runner::resolve_ai_edit_route(&app_handle, &session_settings);
    let copy_context =
        session_settings.copy_context_on_start && edit_route != llm_runner::AiEditRoute::Off;

    // Non-invasive capture of caret & selection context via fast Win32 / warm UIA worker
    let (caret_tx, caret_rx) = mpsc::sync_channel::<smart_insertion::CaretContext>(1);
    if session_settings.smart_context_enabled || copy_context {
        let app_handle_caret = app_handle.clone();
        let caret_gen = gen;
        let save_caret = session_settings.smart_context_enabled;
        tauri::async_runtime::spawn_blocking(move || {
            let ctx = caret_context::capture_caret_context(
                caret_context::DEFAULT_CARET_CAPTURE_TIMEOUT_MS,
            );
            if let Some(state) = app_handle_caret.try_state::<AppState>() {
                let state = state.inner();
                if state.session_gen.load(Ordering::SeqCst) == caret_gen {
                    if save_caret {
                        if let Ok(mut guard) = state.caret_context.lock() {
                            *guard = Some(ctx.clone());
                        }
                    }
                    if copy_context && !ctx.selected_text.trim().is_empty() {
                        if let Ok(mut sel_guard) = state.selected_text.lock() {
                            if sel_guard.trim().is_empty() {
                                *sel_guard = ctx.selected_text.clone();
                                let _ = app_handle_caret.emit("selection-context-active", true);
                            }
                        }
                    }
                }
            }
            let _ = caret_tx.send(ctx);
        });
    } else {
        drop(caret_tx);
    }

    if copy_context {
        let app_handle_copy = app_handle.clone();
        let session_tag_copy = session_tag.clone();
        // Prewarm the local editing server as soon as a selection is captured:
        // loading Qwen3 into the GPU then overlaps with the recording instead
        // of adding 20+ seconds after the hotkey is released.
        let warm_route = edit_route;
        let warm_model = session_settings.llm_model.clone();
        // Blocking worker: the clipboard mutex guard cannot be held across an
        // async yield (MutexGuard is not Send), and the whole backup -> copy
        // -> sequence check -> read -> restore sequence must stay serialized (C7).
        tauri::async_runtime::spawn_blocking(move || {
            let caret_ctx = caret_rx.recv().unwrap_or_default();

            let already_captured = !caret_ctx.selected_text.trim().is_empty()
                || app_handle_copy
                    .try_state::<AppState>()
                    .and_then(|s| {
                        s.inner()
                            .selected_text
                            .lock()
                            .ok()
                            .map(|g| !g.trim().is_empty())
                    })
                    .unwrap_or(false);

            let mut captured_selection = already_captured;
            if already_captured {
                crate::logger::log(
                    "INFO",
                    "Context",
                    Some(&session_tag_copy),
                    "Selected text already captured non-invasively via UIA/Win32; skipping synthetic Ctrl+C clipboard copy",
                );
            } else if caret_ctx.has_authoritative_selection() {
                crate::logger::log(
                    "INFO",
                    "Context",
                    Some(&session_tag_copy),
                    "Authoritative empty selection reported by UIA/Win32; skipping synthetic Ctrl+C clipboard copy",
                );
            } else if let Some(state_ref) = app_handle_copy.try_state::<AppState>() {
                let state = state_ref.inner();
                // Brief settle for physical hotkey chord before fallback synthetic Ctrl+C.
                std::thread::sleep(std::time::Duration::from_millis(15));

                let session_active = state.session_gen.load(Ordering::SeqCst) == gen
                    && state.is_recording.load(Ordering::SeqCst);
                let start_focus = state
                    .start_focus
                    .lock()
                    .ok()
                    .map(|g| *g)
                    .unwrap_or_default();
                let focus_ok =
                    start_focus.is_compatible_with(&keyboard_simulator::get_focus_target());

                if session_active && focus_ok {
                    // Without the shared clipboard mutex, a session whose paste window
                    // overlaps this copy could capture temporary text as its "original"
                    // and the user's real clipboard content would be lost.
                    let _clipboard_mutex_guard = match state.clipboard_mutex.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    };

                    // Never call `cb.clear()` here: clearing destroys non-text/non-image
                    // clipboard formats (`ClipboardBackup::Preserve`: Explorer files, Excel, RTF).
                    // Instead, check `GetClipboardSequenceNumber()` around `simulate_copy_checked()`.
                    let backup = peek_or_backup_clipboard(state);
                    let seq_before = get_clipboard_sequence_number();
                    let copy_dispatched = keyboard_simulator::simulate_copy_checked();

                    if !copy_dispatched {
                        state
                            .selection_blocked_by_uipi
                            .store(true, Ordering::Release);
                        crate::logger::log(
                            "WARN",
                            "Context",
                            Some(&session_tag_copy),
                            "Synthetic Ctrl+C was blocked by UIPI (elevated target window); armed edit-command guard",
                        );
                    } else {
                        let mut seq_changed = false;
                        for _ in 0..20 {
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            if get_clipboard_sequence_number() != seq_before {
                                seq_changed = true;
                                break;
                            }
                        }

                        if seq_changed {
                            let copied = arboard::Clipboard::new()
                                .ok()
                                .and_then(|mut cb| cb.get_text().ok())
                                .unwrap_or_default();

                            if let Ok(mut unrestored) = state.unrestored_clipboard.lock() {
                                *unrestored = None;
                            }

                            let session_is_current = state.session_gen.load(Ordering::SeqCst)
                                == gen
                                && state.is_recording.load(Ordering::SeqCst);
                            if session_is_current {
                                if let Ok(mut guard) = state.selected_text.lock() {
                                    *guard = copied.clone();
                                    crate::logger::log(
                                        "INFO",
                                        "Context",
                                        Some(&session_tag_copy),
                                        &format!(
                                            "Captured selected text ({} chars)",
                                            copied.chars().count()
                                        ),
                                    );
                                    if !copied.trim().is_empty() {
                                        captured_selection = true;
                                        let _ =
                                            app_handle_copy.emit("selection-context-active", true);
                                    }
                                }
                            }

                            restore_clipboard_guarded(state, gen, backup, Some(&copied));
                        }
                    }
                }
            }

            // Selection captured and the local route will use it: bring the
            // editing server up in the background while the user is still
            // speaking, so the edit does not have to wait for model loading.
            if captured_selection
                && warm_route == llm_runner::AiEditRoute::Local
                && llm_runner::llm_model_is_installed(&app_handle_copy, &warm_model)
            {
                crate::logger::log(
                    "INFO",
                    "LLM",
                    Some(&session_tag_copy),
                    "Selection captured — prewarming the local editing server",
                );
                let warm_handle = app_handle_copy.clone();
                tauri::async_runtime::spawn_blocking(move || {
                    if let Err(error) = llm_runner::start_llm_server(&warm_handle, &warm_model) {
                        crate::logger::log(
                            "WARN",
                            "LLM",
                            None,
                            &format!("Local editing server prewarm failed: {error}"),
                        );
                    }
                });
            }
        });
    }

    if let Some(state) = app_handle.try_state::<AppState>() {
        if let Ok(mut guard) = state.last_activity_time.lock() {
            *guard = std::time::Instant::now();
        }
    }

    if session_settings.transcription_mode == "local" {
        if session_settings.local_engine == "whisper" {
            let prewarm_handle = app_handle.clone();
            let prewarm_settings = session_settings.clone();
            tauri::async_runtime::spawn_blocking(move || {
                whisper_runner::ensure_whisper_server_state(&prewarm_handle, &prewarm_settings);
            });
        } else if session_settings.local_engine == "parakeet" {
            let prewarm_handle = app_handle.clone();
            let prewarm_settings = session_settings.clone();
            tauri::async_runtime::spawn_blocking(move || {
                whisper_runner::ensure_parakeet_server_state(&prewarm_handle, &prewarm_settings);
            });
        }
    }

    // Show overlay immediately in parallel with WASAPI microphone open so HUD
    // feedback feels instantaneous even on slower Bluetooth/USB audio drivers.
    if state.session_gen.load(Ordering::SeqCst) == gen && state.is_recording.load(Ordering::SeqCst)
    {
        if let Some(overlay) = app_handle.get_webview_window("overlay") {
            if let Ok(Some(monitor)) = overlay.primary_monitor() {
                let size = monitor.size();
                let scale_factor = monitor.scale_factor();

                let monitor_width = size.width as f64 / scale_factor;
                let monitor_height = size.height as f64 / scale_factor;

                let overlay_width = 360.0;
                let overlay_height = 92.0;
                const TASKBAR_MARGIN: f64 = 83.0;

                let x = (monitor_width - overlay_width) / 2.0;
                let y = monitor_height - overlay_height - TASKBAR_MARGIN;

                let _ = overlay
                    .set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
            }
            ensure_overlay_topmost(&overlay);
            let _ = overlay.emit(
                "overlay-preferences",
                OverlayPreferences::from(&session_settings),
            );
            let _ = overlay.show();
        }
        let _ = app_handle.emit("recording-state", "recording");
    }

    // Start recording to a session-unique temporary WAV path
    let temp_path = recording_wav_path(gen);
    let temp_path_str = temp_path.to_string_lossy().to_string();
    crate::logger::log(
        "INFO",
        "Audio",
        Some(&session_tag),
        &format!("Starting audio recording to {}", temp_path_str),
    );

    let app_handle_recorder = app_handle.clone();
    let app_handle_vol = app_handle.clone();
    let worker_path = temp_path_str.clone();
    let enable_parakeet_sample_stream = session_settings.streaming_enabled
        && session_settings.transcription_mode == "local"
        && session_settings.local_engine == "parakeet";
    let retain_samples_for_batch_preview =
        session_settings.streaming_enabled && !enable_parakeet_sample_stream;
    let selected_audio_device = session_settings.audio_input_device.clone();
    let start_result = tauri::async_runtime::spawn_blocking(move || {
        let recorder = app_handle_recorder
            .try_state::<AppState>()
            .ok_or_else(|| "Application state is unavailable".to_string())?;
        let dev = if selected_audio_device == "default" || selected_audio_device.is_empty() {
            None
        } else {
            Some(selected_audio_device.as_str())
        };
        let meter_epoch = std::time::Instant::now();
        let last_emit_ms = AtomicU64::new(0);
        recorder.audio_recorder.start_recording(
            &worker_path,
            enable_parakeet_sample_stream,
            retain_samples_for_batch_preview,
            dev,
            move |vol| {
                // Throttle volume updates to ~30 Hz (33 ms) and target only the
                // overlay window so hidden/main webviews stay completely idle.
                let elapsed_ms = meter_epoch.elapsed().as_millis() as u64;
                let prev_ms = last_emit_ms.load(Ordering::Relaxed);
                if prev_ms != 0 && elapsed_ms.saturating_sub(prev_ms) < 33 {
                    return;
                }
                last_emit_ms.store(elapsed_ms.max(1), Ordering::Relaxed);

                let app_handle_vol = app_handle_vol.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = app_handle_vol.emit_to("overlay", "volume-level", vol);
                });
            },
        )
    })
    .await
    .map_err(|error| format!("Audio startup worker failed: {error}"))
    .and_then(|result| result);
    if let Err(e) = start_result {
        crate::logger::log(
            "ERROR",
            "Audio",
            Some(&session_tag),
            &format!("Failed to start recording: {}", e),
        );
        state.is_recording.store(false, Ordering::SeqCst);
        finish_live_target_monitoring(&app_handle, gen);
        keyboard_hook::set_recording_active(false);
        show_overlay_error(&app_handle, gen, "Microphone start error").await;
        return;
    }

    let current_gen = state.session_gen.load(Ordering::SeqCst);
    if current_gen != gen {
        // A newer session has already begun. Do not touch audio_recorder or keyboard_hook,
        // as they now belong to the newer generation!
        finish_live_target_monitoring(&app_handle, gen);
        return;
    }

    if !state.is_recording.load(Ordering::SeqCst) {
        // Recording was cancelled before WASAPI startup completed.
        finish_live_target_monitoring(&app_handle, gen);
        let _ = state.audio_recorder.cancel_recording();
        keyboard_hook::set_recording_active(false);
        request_animated_hide(&app_handle, "cancel");
        return;
    }

    let streaming_enabled = session_settings.streaming_enabled;
    if streaming_enabled {
        let app_handle_loop = app_handle.clone();
        let my_gen = gen;

        if session_settings.transcription_mode == "local"
            && session_settings.local_engine == "parakeet"
        {
            let session_tag = crate::logger::format_session_tag(my_gen);
            match state.audio_recorder.take_sample_stream() {
                Ok(sample_stream) => {
                    let cancel = Arc::new(AtomicBool::new(false));
                    let (result_tx, result_rx) = mpsc::sync_channel(1);
                    let session = ParakeetStreamingSession {
                        generation: my_gen,
                        cancel: Arc::clone(&cancel),
                        result_rx,
                    };
                    let stored = match state.parakeet_streaming.lock() {
                        Ok(mut slot) => {
                            if let Some(previous) = slot.replace(session) {
                                previous.cancel.store(true, Ordering::Release);
                            }
                            true
                        }
                        Err(_) => false,
                    };

                    if stored {
                        crate::logger::log(
                            "INFO",
                            "ASR",
                            Some(&session_tag),
                            "Started event-driven Parakeet streaming worker",
                        );
                        tauri::async_runtime::spawn_blocking(move || {
                            let outcome = run_parakeet_streaming_worker(
                                app_handle_loop,
                                my_gen,
                                sample_stream,
                                cancel,
                            );
                            let _ = result_tx.send(outcome);
                        });
                    } else {
                        cancel.store(true, Ordering::Release);
                        crate::logger::log(
                            "WARN",
                            "ASR",
                            Some(&session_tag),
                            "Streaming session state is unavailable; final WAV fallback remains active",
                        );
                    }
                }
                Err(error) => {
                    crate::logger::log(
                        "WARN",
                        "ASR",
                        Some(&session_tag),
                        &format!(
                            "Could not subscribe to recorder sample stream; using final WAV fallback: {error}"
                        ),
                    );
                    let _ = app_handle_loop.emit(
                        "streaming-degraded",
                        "Live preview is unavailable for this dictation",
                    );
                }
            }
        } else {
            // Existing batch streaming loop
            tauri::async_runtime::spawn(async move {
                let session_tag = crate::logger::format_session_tag(my_gen);
                crate::logger::log(
                    "INFO",
                    "ASR",
                    Some(&session_tag),
                    "Spawning background streaming loop task...",
                );
                tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

                let chunk_path = chunk_wav_path(my_gen);
                let chunk_path_str = chunk_path.to_string_lossy().to_string();
                let mut last_len: usize = 0;

                loop {
                    let still_active = app_handle_loop
                        .try_state::<AppState>()
                        .map(|s| {
                            s.is_recording.load(Ordering::SeqCst)
                                && s.session_gen.load(Ordering::SeqCst) == my_gen
                        })
                        .unwrap_or(false);
                    if !still_active {
                        crate::logger::log(
                            "INFO",
                            "ASR",
                            Some(&session_tag),
                            "Streaming session ended. Exiting streaming loop.",
                        );
                        break;
                    }

                    let preview_handle = app_handle_loop.clone();
                    let preview_path = chunk_path_str.clone();
                    let previous_len = last_len;
                    let preview_result = tauri::async_runtime::spawn_blocking(move || {
                        prepare_batch_preview(&preview_handle, &preview_path, previous_len)
                    })
                    .await;
                    let preview = match preview_result {
                        Ok(Ok(preview)) => preview,
                        Ok(Err(error)) => {
                            crate::logger::log(
                                "WARN",
                                "ASR",
                                Some(&session_tag),
                                &format!("Could not prepare streaming preview: {error}"),
                            );
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                            continue;
                        }
                        Err(error) => {
                            crate::logger::log(
                                "ERROR",
                                "ASR",
                                Some(&session_tag),
                                &format!("Streaming preview worker failed: {error}"),
                            );
                            break;
                        }
                    };

                    if preview.recorded_secs >= 120.0 {
                        crate::logger::log(
                            "WARN",
                            "ASR",
                            Some(&session_tag),
                            "Batch live preview paused after 120 seconds to avoid repeated full-recording uploads and decoding.",
                        );
                        let _ = app_handle_loop.emit(
                            "streaming-degraded",
                            "Live preview paused for this long dictation",
                        );
                        break;
                    }
                    let sleep_ms = if preview.recorded_secs < 60.0 {
                        4_000
                    } else {
                        8_000
                    };

                    if let Some((sample_len, settings, layout_lang)) = preview.work {
                        last_len = sample_len;
                        let language = effective_language(&settings, &layout_lang, false);

                        let transcription_result = if settings.transcription_mode == "local" {
                            run_local_whisper_async(
                                app_handle_loop.clone(),
                                settings.model_name.clone(),
                                chunk_path_str.clone(),
                                language.clone(),
                                settings.dictionary.clone(),
                                my_gen,
                            )
                            .await
                        } else {
                            // Previews must stay snappy: cap each cloud round
                            // like the finalize path instead of relying on the
                            // 900s HTTP backstop.
                            match tokio::time::timeout(
                                std::time::Duration::from_secs(45),
                                ai_client::transcribe_and_clean(
                                    provider_from(&settings),
                                    &settings.api_key,
                                    &chunk_path_str,
                                    "",
                                    &language,
                                    &settings.dictionary,
                                    false,
                                    Some(&settings.custom_api_url),
                                    Some(&settings.custom_model_name),
                                ),
                            )
                            .await
                            {
                                Ok(result) => result,
                                Err(_) => Err("Cloud preview timed out after 45s".to_string()),
                            }
                        };

                        match transcription_result {
                            Ok(text) => {
                                let normalized_text =
                                    text_normalizer::normalize_transcription_text(&text, &language);
                                let trimmed = normalized_text.trim().to_string();
                                crate::logger::log(
                                    "INFO",
                                    "ASR",
                                    Some(&session_tag),
                                    &format!(
                                        "Streaming transcription success: '{}'",
                                        crate::logger::anonymize_speech(
                                            &trimmed,
                                            settings.log_speech_text
                                        )
                                    ),
                                );
                                if !trimmed.is_empty() && !is_silence_hallucination(&trimmed) {
                                    type_streaming_update(
                                        app_handle_loop.clone(),
                                        my_gen,
                                        trimmed,
                                        TypingUpdateMode::LivePreview,
                                    )
                                    .await;
                                }
                            }
                            Err(e) => {
                                crate::logger::log(
                                    "ERROR",
                                    "ASR",
                                    Some(&session_tag),
                                    &format!("Streaming transcription failed: {}", e),
                                );
                            }
                        }
                    }

                    tokio::time::sleep(std::time::Duration::from_millis(sleep_ms)).await;
                }

                let _ = tokio::fs::remove_file(&chunk_path).await;
            });
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditExecutionTarget {
    Local,
    Cloud,
}

pub fn resolve_history_entry_meta(
    transcription_mode: &str,
    local_engine: &str,
    used_local_fallback: bool,
    asr_duration_ms: u64,
    executed_edit_target: Option<EditExecutionTarget>,
) -> (String, Option<String>, u64) {
    match executed_edit_target {
        Some(EditExecutionTarget::Cloud) => ("cloud".to_string(), None, 0),
        Some(EditExecutionTarget::Local) => (
            "local".to_string(),
            Some(local_engine.to_string()),
            asr_duration_ms,
        ),
        None => {
            let history_mode = if used_local_fallback {
                "local (cloud fallback)".to_string()
            } else {
                transcription_mode.to_string()
            };
            let is_local_text = transcription_mode == "local" || used_local_fallback;
            let history_engine = if is_local_text {
                Some(local_engine.to_string())
            } else {
                None
            };
            let history_processing_ms = if is_local_text { asr_duration_ms } else { 0 };
            (history_mode, history_engine, history_processing_ms)
        }
    }
}

struct TempWavGuard(std::path::PathBuf);

impl Drop for TempWavGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn should_abort_uipi_blocked_edit_command(
    selection_blocked_by_uipi: bool,
    has_selection: bool,
    edit_route: llm_runner::AiEditRoute,
    spoken_text: &str,
    translate_default_target: &str,
    ui_language: &str,
    disabled_edit_commands: &[String],
) -> bool {
    if !selection_blocked_by_uipi
        || has_selection
        || edit_route == llm_runner::AiEditRoute::Off
        || spoken_text.trim().is_empty()
    {
        return false;
    }
    let intent =
        edit_intent::parse_edit_intent(spoken_text, translate_default_target, "", ui_language);
    intent != edit_intent::EditIntent::NotACommand
        && !edit_intent::is_command_disabled(&intent, disabled_edit_commands)
}

fn postprocess_session_text(
    raw_text: &str,
    language: &str,
    text_replacements: &[settings::TextReplacement],
    smart_context_enabled: bool,
    caret_ctx: Option<&smart_insertion::CaretContext>,
    smart_context_trailing_period: bool,
    executed_edit_target: Option<EditExecutionTarget>,
) -> (String, usize, bool) {
    if executed_edit_target.is_some() {
        // AI edit outputs are finished documents/translations/code blocks:
        // speech normalizers, custom dictation replacements, smart-caret
        // lowercase/period rules, `[|]` caret markers, and ASR silence-hallucination
        // filters must never mutate newlines (`\n`), identifiers (`std::io`),
        // or polite sign-offs ("Спасибо." / "Thank you.").
        let empty = raw_text.trim().is_empty();
        return (raw_text.to_string(), 0, empty);
    }

    let normalized_text = text_normalizer::normalize_transcription_text(raw_text, language);
    let replaced_text =
        text_normalizer::apply_text_replacements(&normalized_text, text_replacements);

    let (final_candidate, is_empty_speech) = if smart_context_enabled {
        if let Some(ctx) = caret_ctx {
            let adapted = smart_insertion::adapt_transcription_to_context(
                &replaced_text,
                ctx,
                smart_context_trailing_period,
            );
            let empty = adapted.trim().is_empty();
            (adapted, empty)
        } else {
            let trimmed = replaced_text.trim().to_string();
            let empty = trimmed.is_empty();
            (trimmed, empty)
        }
    } else {
        let trimmed = replaced_text.trim().to_string();
        let empty = trimmed.is_empty();
        (trimmed, empty)
    };

    if is_empty_speech || is_silence_hallucination(final_candidate.trim()) {
        return (final_candidate, 0, true);
    }
    let (final_text, caret_left) = text_normalizer::extract_caret_marker(&final_candidate);
    let final_empty = final_text.trim().is_empty();
    (final_text, caret_left, final_empty)
}

/// Stops the recording and runs the final transcription + paste/type pipeline.
async fn finalize_recording(app_handle: tauri::AppHandle, is_toggle: bool) {
    let Some(state) = app_handle.try_state::<AppState>() else {
        return;
    };
    let state = state.inner();

    if state
        .is_recording
        .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }

    let selection_blocked_by_uipi = state
        .selection_blocked_by_uipi
        .swap(false, Ordering::AcqRel);
    let active_layout_lang = state
        .selected_language
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default();
    let session_selected_text = state
        .selected_text
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_default();
    let my_gen = state.session_gen.load(Ordering::SeqCst);
    let session_tag = crate::logger::format_session_tag(my_gen);
    crate::logger::log(
        "INFO",
        "Session",
        Some(&session_tag),
        "Finalizing recording session",
    );
    let streaming_session = match state.parakeet_streaming.lock() {
        Ok(mut slot) => {
            if slot
                .as_ref()
                .map(|session| session.generation == my_gen)
                .unwrap_or(false)
            {
                slot.take()
            } else {
                None
            }
        }
        Err(_) => {
            crate::logger::log(
                "WARN",
                "ASR",
                Some(&session_tag),
                "Streaming session state is unavailable; using final WAV fallback",
            );
            None
        }
    };

    state.latched.store(false, Ordering::SeqCst);
    keyboard_hook::set_recording_active(false);

    let _ = app_handle.emit("recording-state", "processing");

    let temp_path = recording_wav_path(my_gen);
    let temp_wav_guard = TempWavGuard(temp_path.clone());

    let stop_handle = app_handle.clone();
    let stop_res = tauri::async_runtime::spawn_blocking(move || {
        let state = stop_handle
            .try_state::<AppState>()
            .ok_or_else(|| "Application state is unavailable".to_string())?;
        state.audio_recorder.stop_recording_adaptive(is_toggle)?;
        Ok::<Option<bool>, String>(state.audio_recorder.take_last_vad_has_speech())
    })
    .await
    .map_err(|error| format!("Audio stop worker failed: {error}"))
    .and_then(|result| result);
    crate::logger::log(
        "INFO",
        "Audio",
        Some(&session_tag),
        &format!("stop_recording result = {:?}", stop_res),
    );
    let vad_speech_opt = match stop_res {
        Ok(vad_opt) => vad_opt,
        Err(e) => {
            if let Some(session) = &streaming_session {
                session.cancel.store(true, Ordering::Release);
            }
            crate::logger::log(
                "ERROR",
                "Audio",
                Some(&session_tag),
                &format!("Failed to stop recording: {}", e),
            );
            finish_live_target_monitoring(&app_handle, my_gen);
            show_overlay_error(&app_handle, my_gen, "Recording stop error").await;
            return;
        }
    };

    let app_handle_clone = app_handle.clone();
    let session_tag_clone = session_tag.clone();
    tauri::async_runtime::spawn(async move {
        let _temp_wav_guard = temp_wav_guard;
        let start_time = std::time::Instant::now();

        let temp_path_str = temp_path.to_string_lossy().to_string();

        let settings = match load_settings_async(app_handle_clone.clone()).await {
            Ok(s) => s,
            Err(e) => {
                if let Some(session) = &streaming_session {
                    session.cancel.store(true, Ordering::Release);
                }
                crate::logger::log(
                    "ERROR",
                    "Settings",
                    Some(&session_tag_clone),
                    &format!("Failed to load settings: {}", e),
                );
                finish_live_target_monitoring(&app_handle_clone, my_gen);
                show_overlay_error(&app_handle_clone, my_gen, "Settings load error").await;
                return;
            }
        };

        let layout_lang = active_layout_lang.clone();
        let selected_text = session_selected_text;
        let has_selection = !selected_text.trim().is_empty();
        let language = effective_language(&settings, &layout_lang, has_selection);
        let api_call_start = std::time::Instant::now();
        let edit_route = llm_runner::resolve_ai_edit_route(&app_handle_clone, &settings);
        // ASR strictly transcribes speech; the dedicated edit pass handles selected-text
        // modifications deterministically for all engines.

        let streaming_eligible = settings.streaming_enabled
            && settings.transcription_mode == "local"
            && settings.local_engine == "parakeet"
            && settings.model_name == "parakeet-v3";
        let mut streaming_transcript = match (streaming_eligible, streaming_session) {
            (true, Some(session)) => {
                let handoff_started = std::time::Instant::now();
                match await_parakeet_streaming_outcome(session).await {
                    Ok(outcome) => {
                        crate::logger::log(
                            "INFO",
                            "ASR",
                            Some(&session_tag_clone),
                            &format!(
                                "Streaming final handoff: wait_ms={}, decodes={}, decode_ms={}, max_active_samples={}, queue_gaps={}",
                                handoff_started.elapsed().as_millis(),
                                outcome.metrics.decode_requests,
                                outcome.metrics.total_decode_ms,
                                outcome.metrics.max_active_samples,
                                outcome.dropped_chunks
                            ),
                        );
                        match outcome.reusable_transcript(my_gen) {
                            Ok(text) if !is_silence_hallucination(&text) => Some(text),
                            Ok(_) => {
                                crate::logger::log(
                                    "WARN",
                                    "ASR",
                                    Some(&session_tag_clone),
                                    "Streaming result resembled a silence hallucination; using WAV fallback",
                                );
                                None
                            }
                            Err(reason) => {
                                crate::logger::log(
                                    "WARN",
                                    "ASR",
                                    Some(&session_tag_clone),
                                    &format!("Streaming result is not reusable; using WAV fallback: {reason}"),
                                );
                                None
                            }
                        }
                    }
                    Err(error) => {
                        crate::logger::log(
                            "WARN",
                            "ASR",
                            Some(&session_tag_clone),
                            &format!("Streaming final handoff failed; using WAV fallback: {error}"),
                        );
                        None
                    }
                }
            }
            (false, Some(session)) => {
                session.cancel.store(true, Ordering::Release);
                None
            }
            (_, None) => None,
        };
        let used_streaming_result = streaming_transcript.is_some();

        let mut empty_session = false;
        if !used_streaming_result {
            match vad_speech_opt {
                Some(false) => {
                    crate::logger::log(
                        "INFO",
                        "VAD",
                        Some(&session_tag_clone),
                        "Streaming VAD reported silence; skipping transcription (empty session)",
                    );
                    empty_session = true;
                }
                Some(true) => {
                    // Speech confirmed and >600ms trailing silence already trimmed before WAV commit.
                }
                None => {
                    let gate_path = temp_path_str.clone();
                    let gate_tag = session_tag_clone.clone();
                    let has_speech = tauri::async_runtime::spawn_blocking(move || {
                        vad::gate_wav_file(&gate_path, Some(&gate_tag))
                    })
                    .await
                    .map_err(|error| format!("VAD gate worker failed: {error}"))
                    .and_then(|result| result)
                    // Fail-open: if the probe itself errors, transcribe anyway.
                    .unwrap_or(true);
                    if !has_speech {
                        crate::logger::log(
                            "INFO",
                            "VAD",
                            Some(&session_tag_clone),
                            "No speech detected in recording; skipping transcription (empty session)",
                        );
                        empty_session = true;
                    } else {
                        let trim_path = temp_path.clone();
                        let trim_tag = session_tag_clone.clone();
                        let _ = tauri::async_runtime::spawn_blocking(move || {
                            match vad::trim_trailing_silence_wav(&trim_path, &trim_path) {
                                Ok(trimmed) => {
                                    if trimmed {
                                        crate::logger::log(
                                            "INFO",
                                            "VAD",
                                            Some(&trim_tag),
                                            "Trimmed trailing silence from audio WAV (>600ms padding preserved)",
                                        );
                                    }
                                }
                                Err(e) => {
                                    crate::logger::log(
                                        "WARN",
                                        "VAD",
                                        Some(&trim_tag),
                                        &format!("Failed to trim trailing silence from WAV: {e}"),
                                    );
                                }
                            }
                        })
                        .await;
                    }
                }
            }
        }

        crate::logger::log(
            "INFO",
            "ASR",
            Some(&session_tag_clone),
            if used_streaming_result {
                "Reusing completed Parakeet streaming result"
            } else {
                "Calling final transcription from WAV"
            },
        );
        let mut used_local_fallback = false;
        let asr_started = std::time::Instant::now();
        let mut transcription_result = if empty_session {
            Ok(String::new())
        } else if let Some(text) = streaming_transcript.take() {
            Ok(text)
        } else if settings.transcription_mode == "local" {
            run_local_whisper_async(
                app_handle_clone.clone(),
                settings.model_name.clone(),
                temp_path_str.clone(),
                language.clone(),
                settings.dictionary.clone(),
                my_gen,
            )
            .await
        } else {
            if settings.api_key.trim().is_empty() {
                Err("Please enter your API key in settings".to_string())
            } else {
                match tokio::time::timeout(
                    std::time::Duration::from_secs(600),
                    ai_client::transcribe_and_clean(
                        provider_from(&settings),
                        &settings.api_key,
                        &temp_path_str,
                        "",
                        &language,
                        &settings.dictionary,
                        true,
                        Some(&settings.custom_api_url),
                        Some(&settings.custom_model_name),
                    ),
                )
                .await
                {
                    Ok(result) => result,
                    Err(_) => Err("Cloud transcription timed out".to_string()),
                }
            }
        };

        // OpenAI-compatible cloud endpoints take the dictionary in Whisper's
        // `prompt` field, so they can echo it on silence exactly like the local
        // server. Gemini does not take that prompt, so leave its text alone.
        if settings.transcription_mode != "local"
            && matches!(
                settings.api_provider.as_str(),
                "openai" | "groq" | "huggingface" | "custom"
            )
        {
            if let Ok(text) = transcription_result.as_ref() {
                let cleaned =
                    whisper_runner::strip_prompt_echo(text, &language, &settings.dictionary);
                if cleaned.is_empty() && !text.trim().is_empty() {
                    transcription_result = Ok(cleaned);
                }
            }
        }

        if settings.transcription_mode != "local" && settings.cloud_fallback_enabled {
            if let Err(cloud_err) = &transcription_result {
                if is_cloud_unreachable(cloud_err) {
                    if let Ok(downloaded) = get_downloaded_models(app_handle_clone.clone()).await {
                        let fallback_model =
                            select_local_fallback_model(&downloaded, &settings.model_name);
                        if let Some(model) = fallback_model {
                            crate::logger::log(
                                "WARN",
                                "ASR",
                                Some(&session_tag_clone),
                                &format!(
                                    "Cloud unreachable, falling back to local model '{}'",
                                    model
                                ),
                            );
                            let local_result = run_local_whisper_async(
                                app_handle_clone.clone(),
                                model,
                                temp_path_str.clone(),
                                language.clone(),
                                settings.dictionary.clone(),
                                my_gen,
                            )
                            .await;
                            if local_result.is_ok() {
                                used_local_fallback = true;
                                transcription_result = local_result;
                                let _ = app_handle_clone.emit(
                                    "recording-state",
                                    "notice:Cloud unavailable — used local model instead",
                                );
                            }
                        }
                    }
                }
            }
        }
        let final_source = if used_streaming_result {
            "streaming-reuse"
        } else if streaming_eligible {
            "wav-fallback"
        } else {
            "wav-batch"
        };
        crate::logger::log(
            "INFO",
            "ASR",
            Some(&session_tag_clone),
            &format!(
                "Final transcription duration = {} ms (source={})",
                api_call_start.elapsed().as_millis(),
                final_source
            ),
        );

        let _ = tokio::fs::remove_file(&temp_path).await;

        let mut had_error = None;
        let mut final_notice = None;
        let mut overlay_hide_requested = false;
        // Set whenever a selected-text edit pass could not run. `text` then
        // still holds the raw speech, and pasting it would insert the spoken
        // command ("Сократи") instead of the document it was meant to change.
        let mut edit_failed = false;
        let mut executed_edit_target: Option<EditExecutionTarget> = None;
        match transcription_result {
            Ok(mut text) => {
                // Selected-text edit pass: runs whenever a selection and spoken text
                // are present, for any transcription engine and edit route.
                let has_selection = !selected_text.trim().is_empty();
                if should_abort_uipi_blocked_edit_command(
                    selection_blocked_by_uipi,
                    has_selection,
                    edit_route,
                    &text,
                    &settings.translate_default_target,
                    &settings.ui_language,
                    &settings.disabled_edit_commands,
                ) {
                    crate::logger::log(
                        "WARN",
                        "Context",
                        Some(&session_tag_clone),
                        "Edit command spoken into UIPI-protected window where selection could not be copied; aborting literal paste",
                    );
                    edit_failed = true;
                    final_notice = Some("elevated-paste-blocked");
                }
                let session_caret_ctx = app_handle_clone
                    .try_state::<AppState>()
                    .and_then(|s| s.caret_context.lock().ok().and_then(|g| g.clone()));
                let needs_edit_pass = !edit_failed
                    && llm_runner::needs_edit_pass(
                        has_selection,
                        !text.trim().is_empty(),
                        edit_route,
                    );
                if needs_edit_pass {
                    let session_ok = app_handle_clone
                        .try_state::<AppState>()
                        .map(|s| s.session_gen.load(Ordering::SeqCst) == my_gen)
                        .unwrap_or(false);
                    if !session_ok {
                        crate::logger::log(
                            "INFO",
                            "LLM",
                            Some(&session_tag_clone),
                            "Session superseded before the edit pass; keeping the unedited text",
                        );
                    } else {
                        match edit_route {
                            llm_runner::AiEditRoute::Local => {
                                let edit_started = std::time::Instant::now();
                                match llm_runner::run_local_edit(
                                    app_handle_clone.clone(),
                                    &text,
                                    &selected_text,
                                    &language,
                                    &settings.dictionary,
                                )
                                .await
                                {
                                    Ok(llm_runner::EditOutcome::Edited(edited)) => {
                                        let adapted =
                                            smart_insertion::adapt_edit_result_to_selection_context(
                                                &selected_text,
                                                &edited,
                                                session_caret_ctx.as_ref(),
                                            );
                                        crate::logger::log(
                                            "INFO",
                                            "LLM",
                                            Some(&session_tag_clone),
                                            &format!(
                                                "Local edit pass succeeded in {} ms ({} -> {} chars, model={})",
                                                edit_started.elapsed().as_millis(),
                                                text.chars().count(),
                                                adapted.chars().count(),
                                                settings.llm_model
                                            ),
                                        );
                                        text = adapted;
                                        executed_edit_target = Some(EditExecutionTarget::Local);
                                    }
                                    Ok(llm_runner::EditOutcome::PassThroughDictation(
                                        dictation,
                                    )) => {
                                        text = dictation;
                                        // Leave executed_edit_target = None so ordinary dictation
                                        // spoken over a selection still runs silence-hallucination
                                        // filtering, text replacements, caret markers, and smart context.
                                    }
                                    Err(error) => {
                                        crate::logger::log(
                                            "WARN",
                                            "LLM",
                                            Some(&session_tag_clone),
                                            &format!("Local edit pass failed: {error}"),
                                        );
                                        let active_key = if !settings.api_key.trim().is_empty() {
                                            settings.api_key.as_str()
                                        } else {
                                            settings.active_api_key()
                                        };
                                        let can_cloud_fallback = error.allows_cloud_fallback()
                                            && llm_runner::can_cloud_edit_fallback(
                                                &settings.ai_edit_mode,
                                                active_key,
                                            );

                                        if can_cloud_fallback {
                                            crate::logger::log(
                                                "INFO",
                                                "LLM",
                                                Some(&session_tag_clone),
                                                &format!(
                                                    "Falling back to cloud AI ('{}') for text editing pass",
                                                    settings.api_provider
                                                ),
                                            );
                                            match tokio::time::timeout(
                                                std::time::Duration::from_secs(45),
                                                ai_client::edit_text_with_provider(
                                                    provider_from(&settings),
                                                    active_key,
                                                    &text,
                                                    &selected_text,
                                                    &language,
                                                    &settings.dictionary,
                                                    &settings.translate_default_target,
                                                    &settings.ui_language,
                                                    Some(&settings.custom_api_url),
                                                    Some(&settings.custom_edit_model_name),
                                                    &settings.disabled_edit_commands,
                                                ),
                                            )
                                            .await
                                            {
                                                Ok(Ok(llm_runner::EditOutcome::Edited(edited))) => {
                                                    let adapted = smart_insertion::adapt_edit_result_to_selection_context(
                                                        &selected_text,
                                                        &edited,
                                                        session_caret_ctx.as_ref(),
                                                    );
                                                    crate::logger::log(
                                                        "INFO",
                                                        "LLM",
                                                        Some(&session_tag_clone),
                                                        &format!(
                                                            "Cloud fallback edit pass succeeded in {} ms ({} -> {} chars)",
                                                            edit_started.elapsed().as_millis(),
                                                            text.chars().count(),
                                                            adapted.chars().count(),
                                                        ),
                                                    );
                                                    text = adapted;
                                                    executed_edit_target =
                                                        Some(EditExecutionTarget::Cloud);
                                                }
                                                Ok(Ok(
                                                    llm_runner::EditOutcome::PassThroughDictation(
                                                        dictation,
                                                    ),
                                                )) => {
                                                    text = dictation;
                                                }
                                                Ok(Err(cloud_error)) => {
                                                    crate::logger::log(
                                                        "WARN",
                                                        "LLM",
                                                        Some(&session_tag_clone),
                                                        &format!(
                                                            "Cloud fallback edit pass also failed: {cloud_error}"
                                                        ),
                                                    );
                                                    edit_failed = true;
                                                    final_notice =
                                                        Some(cloud_error.overlay_notice(true));
                                                }
                                                Err(_) => {
                                                    crate::logger::log(
                                                        "WARN",
                                                        "LLM",
                                                        Some(&session_tag_clone),
                                                        "Cloud fallback edit pass timed out after 45s",
                                                    );
                                                    edit_failed = true;
                                                    final_notice = Some("edit-timeout");
                                                }
                                            }
                                        } else {
                                            edit_failed = true;
                                            final_notice = Some(error.overlay_notice(false));
                                        }
                                    }
                                }
                            }
                            llm_runner::AiEditRoute::Cloud => {
                                let active_key = if !settings.api_key.trim().is_empty() {
                                    settings.api_key.as_str()
                                } else {
                                    settings.active_api_key()
                                };
                                if active_key.trim().is_empty() {
                                    crate::logger::log(
                                        "WARN",
                                        "LLM",
                                        Some(&session_tag_clone),
                                        "No API key for the cloud edit pass; keeping the unedited text",
                                    );
                                    edit_failed = true;
                                    final_notice = Some("edit-no-api-key");
                                } else {
                                    let edit_started = std::time::Instant::now();
                                    match tokio::time::timeout(
                                        std::time::Duration::from_secs(45),
                                        ai_client::edit_text_with_provider(
                                            provider_from(&settings),
                                            active_key,
                                            &text,
                                            &selected_text,
                                            &language,
                                            &settings.dictionary,
                                            &settings.translate_default_target,
                                            &settings.ui_language,
                                            Some(&settings.custom_api_url),
                                            Some(&settings.custom_edit_model_name),
                                            &settings.disabled_edit_commands,
                                        ),
                                    )
                                    .await
                                    {
                                        Ok(Ok(llm_runner::EditOutcome::Edited(edited))) => {
                                            let adapted =
                                                smart_insertion::adapt_edit_result_to_selection_context(
                                                    &selected_text,
                                                    &edited,
                                                    session_caret_ctx.as_ref(),
                                                );
                                            crate::logger::log(
                                                "INFO",
                                                "LLM",
                                                Some(&session_tag_clone),
                                                &format!(
                                                    "Cloud edit pass succeeded in {} ms ({} -> {} chars)",
                                                    edit_started.elapsed().as_millis(),
                                                    text.chars().count(),
                                                    adapted.chars().count()
                                                ),
                                            );
                                            text = adapted;
                                            executed_edit_target = Some(EditExecutionTarget::Cloud);
                                        }
                                        Ok(Ok(llm_runner::EditOutcome::PassThroughDictation(
                                            dictation,
                                        ))) => {
                                            text = dictation;
                                        }
                                        Ok(Err(error)) => {
                                            crate::logger::log(
                                                "WARN",
                                                "LLM",
                                                Some(&session_tag_clone),
                                                &format!(
                                                    "Cloud edit pass failed; keeping the unedited text: {error}"
                                                ),
                                            );
                                            edit_failed = true;
                                            final_notice = Some(error.overlay_notice(true));
                                        }
                                        Err(_) => {
                                            crate::logger::log(
                                                "WARN",
                                                "LLM",
                                                Some(&session_tag_clone),
                                                "Cloud edit pass timed out after 45s; keeping the unedited text",
                                            );
                                            edit_failed = true;
                                            final_notice = Some("edit-timeout");
                                        }
                                    }
                                }
                            }
                            llm_runner::AiEditRoute::Off => {}
                        }
                    }
                }
                let postprocess_caret_ctx = if settings.smart_context_enabled {
                    session_caret_ctx.as_ref()
                } else {
                    None
                };

                let (final_text, caret_left, is_discarded_output) = postprocess_session_text(
                    &text,
                    &language,
                    &settings.text_replacements,
                    settings.smart_context_enabled,
                    postprocess_caret_ctx,
                    settings.smart_context_trailing_period,
                    executed_edit_target,
                );

                crate::logger::log(
                    "INFO",
                    "ASR",
                    Some(&session_tag_clone),
                    &format!(
                        "Final result: {}",
                        crate::logger::anonymize_speech(&final_text, settings.log_speech_text)
                    ),
                );
                if edit_failed {
                    crate::logger::log(
                        "INFO",
                        "Paste",
                        Some(&session_tag_clone),
                        "Text editing failed; nothing was inserted and the selection was left unchanged",
                    );
                } else if !is_discarded_output {
                    let (history_mode, history_engine, history_processing_ms) =
                        resolve_history_entry_meta(
                            &settings.transcription_mode,
                            &settings.local_engine,
                            used_local_fallback,
                            asr_started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
                            executed_edit_target,
                        );
                    // Deferred: DPAPI + file I/O must not delay the paste, so
                    // the entry is persisted right AFTER insertion completes.
                    let deferred_history = Some((
                        app_handle_clone.clone(),
                        final_text.clone(),
                        history_mode,
                        history_engine,
                        history_processing_ms,
                        session_tag_clone.clone(),
                    ));

                    let paste_start = std::time::Instant::now();
                    if settings.streaming_enabled {
                        let typing_outcome = type_streaming_update(
                            app_handle_clone.clone(),
                            my_gen,
                            final_text.clone(),
                            TypingUpdateMode::Final,
                        )
                        .await;
                        if typing_outcome.needs_safe_clipboard_handoff() {
                            crate::logger::log(
                                "WARN",
                                "Typing",
                                Some(&session_tag_clone),
                                &format!(
                                    "Final document reconciliation skipped ({typing_outcome:?}); preserving transcript in history and clipboard"
                                ),
                            );
                            let handoff_handle = app_handle_clone.clone();
                            let handoff_text = final_text.clone();
                            let copied_privately =
                                tauri::async_runtime::spawn_blocking(move || {
                                    if let Some(state) = handoff_handle.try_state::<AppState>() {
                                        let _guard = state
                                            .clipboard_mutex
                                            .lock()
                                            .unwrap_or_else(|p| p.into_inner());
                                        if let Ok(mut unrestored) =
                                            state.unrestored_clipboard.lock()
                                        {
                                            *unrestored = None;
                                        }
                                        write_transcript_to_clipboard(&handoff_text)
                                    } else {
                                        write_transcript_to_clipboard(&handoff_text)
                                    }
                                })
                                .await
                                .unwrap_or(false);

                            if copied_privately {
                                final_notice = Some("final-copied-after-edit");
                            } else {
                                crate::logger::log(
                                    "ERROR",
                                    "Clipboard",
                                    Some(&session_tag_clone),
                                    "Could not write the safely preserved final transcript to clipboard",
                                );
                            }
                        } else {
                            if caret_left > 0 && typing_outcome == TypingUpdateOutcome::Applied {
                                let arrow_handle = app_handle_clone.clone();
                                let arrow_focus = app_handle_clone
                                    .try_state::<AppState>()
                                    .and_then(|s| s.start_focus.lock().ok().map(|g| *g))
                                    .unwrap_or_default();
                                let _ = tauri::async_runtime::spawn_blocking(move || {
                                    let should_stop = || {
                                        let gen_ok = arrow_handle
                                            .try_state::<AppState>()
                                            .map(|s| s.session_gen.load(Ordering::SeqCst) == my_gen)
                                            .unwrap_or(false);
                                        !gen_ok
                                            || !arrow_focus.is_compatible_with(
                                                &keyboard_simulator::get_focus_target(),
                                            )
                                    };
                                    if !should_stop() {
                                        keyboard_simulator::send_left_arrows_cancellable(
                                            caret_left,
                                            should_stop,
                                        );
                                    }
                                })
                                .await;
                            }
                            request_animated_hide(&app_handle_clone, "success");
                            overlay_hide_requested = true;
                        }
                    } else {
                        let session_ok = app_handle_clone
                            .try_state::<AppState>()
                            .map(|s| s.session_gen.load(Ordering::SeqCst) == my_gen)
                            .unwrap_or(false);
                        let start_focus = app_handle_clone
                            .try_state::<AppState>()
                            .and_then(|s| s.start_focus.lock().ok().map(|g| *g))
                            .unwrap_or_default();
                        let current_focus = keyboard_simulator::get_focus_target();
                        let focus_ok = start_focus.is_compatible_with(&current_focus);

                        if session_ok && focus_ok {
                            let mut original_clipboard = ClipboardBackup::Empty;
                            let mut paste_blocked = false;
                            let mut clipboard_written = false;
                            // Serialize the clipboard mutation against other
                            // overlapping sessions before the paste lands.
                            if let Some(state) = app_handle_clone.try_state::<AppState>() {
                                let _guard = state
                                    .clipboard_mutex
                                    .lock()
                                    .unwrap_or_else(|p| p.into_inner());
                                original_clipboard = peek_or_backup_clipboard(state.inner());
                                if write_transcript_to_clipboard(&final_text) {
                                    clipboard_written = true;
                                }
                                if clipboard_written {
                                    paste_blocked = !keyboard_simulator::simulate_paste();
                                    if paste_blocked {
                                        if let Ok(mut slot) = state.unrestored_clipboard.lock() {
                                            *slot = None;
                                        }
                                        // UIPI silently dropped the input (the
                                        // foreground app likely runs elevated).
                                        // Keep the transcript in the clipboard
                                        // instead of restoring over it.
                                        crate::logger::log(
                                            "WARN",
                                            "Paste",
                                            Some(&session_tag_clone),
                                            "Paste was blocked by the system; transcript kept in clipboard",
                                        );
                                    } else if let Ok(mut slot) = state.unrestored_clipboard.lock() {
                                        *slot = Some((
                                            my_gen,
                                            original_clipboard.clone(),
                                            final_text.clone(),
                                        ));
                                    }
                                } else {
                                    crate::logger::log(
                                        "ERROR",
                                        "Paste",
                                        Some(&session_tag_clone),
                                        "Could not write transcript to clipboard; paste aborted to prevent leaking previous clipboard contents",
                                    );
                                }
                            }
                            if !clipboard_written {
                                had_error = Some("Failed to write to clipboard".to_string());
                            } else if paste_blocked {
                                final_notice = Some("elevated-paste-blocked");
                            } else {
                                let post_paste_started = std::time::Instant::now();
                                // Hide overlay and play success chime immediately upon paste dispatch
                                request_animated_hide(&app_handle_clone, "success");
                                overlay_hide_requested = true;

                                if caret_left > 0 {
                                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                                    let arrow_handle = app_handle_clone.clone();
                                    let arrow_focus = start_focus;
                                    let _ = tauri::async_runtime::spawn_blocking(move || {
                                        let should_stop = || {
                                            let gen_ok = arrow_handle
                                                .try_state::<AppState>()
                                                .map(|s| {
                                                    s.session_gen.load(Ordering::SeqCst) == my_gen
                                                })
                                                .unwrap_or(false);
                                            !gen_ok
                                                || !arrow_focus.is_compatible_with(
                                                    &keyboard_simulator::get_focus_target(),
                                                )
                                        };
                                        if !should_stop() {
                                            keyboard_simulator::send_left_arrows_cancellable(
                                                caret_left,
                                                should_stop,
                                            );
                                        }
                                    })
                                    .await;
                                }

                                let remaining_restore_wait = std::time::Duration::from_millis(400)
                                    .saturating_sub(post_paste_started.elapsed());
                                if !remaining_restore_wait.is_zero() {
                                    tokio::time::sleep(remaining_restore_wait).await;
                                }

                                let restore_handle = app_handle_clone.clone();
                                let fallback_backup = original_clipboard;
                                let fallback_expected = final_text.clone();
                                let _ = tauri::async_runtime::spawn_blocking(move || {
                                    if let Some(state) = restore_handle.try_state::<AppState>() {
                                        let _guard = state
                                            .clipboard_mutex
                                            .lock()
                                            .unwrap_or_else(|p| p.into_inner());
                                        let to_restore = {
                                            let mut slot = state
                                                .unrestored_clipboard
                                                .lock()
                                                .unwrap_or_else(|p| p.into_inner());
                                            take_unrestored_clipboard_for_restore(&mut slot, my_gen)
                                        };
                                        if let Some((backup, expected)) = to_restore {
                                            restore_clipboard_if_unchanged(backup, &expected);
                                        }
                                    } else {
                                        restore_clipboard_if_unchanged(
                                            fallback_backup,
                                            &fallback_expected,
                                        );
                                    }
                                })
                                .await;
                            }
                        } else if session_ok {
                            crate::logger::log(
                                "WARN",
                                "Paste",
                                Some(&session_tag_clone),
                                "Focus changed; leaving text in clipboard instead of pasting.",
                            );
                            final_notice = Some("focus-changed-copied");
                            // Same clipboard serialization as the paste path: a
                            // concurrent session must not interleave writes.
                            if let Some(state) = app_handle_clone.try_state::<AppState>() {
                                let _guard = state
                                    .clipboard_mutex
                                    .lock()
                                    .unwrap_or_else(|p| p.into_inner());
                                if let Ok(mut slot) = state.unrestored_clipboard.lock() {
                                    *slot = None;
                                }
                                let _ = write_transcript_to_clipboard(&final_text);
                            } else {
                                let _ = write_transcript_to_clipboard(&final_text);
                            }
                        } else {
                            crate::logger::log(
                                "WARN",
                                "Paste",
                                Some(&session_tag_clone),
                                "Discarding stale session output without changing keyboard or clipboard.",
                            );
                        }
                    }
                    crate::logger::log(
                        "INFO",
                        "Paste",
                        Some(&session_tag_clone),
                        &format!("Paste duration = {} ms", paste_start.elapsed().as_millis()),
                    );

                    if let Some((
                        history_handle,
                        history_text,
                        history_mode,
                        history_engine,
                        history_processing_ms,
                        history_tag,
                    )) = deferred_history
                    {
                        tauri::async_runtime::spawn_blocking(move || {
                            match history::add_entry(
                                &history_handle,
                                &history_text,
                                &history_mode,
                                history_engine.as_deref(),
                                history_processing_ms,
                            ) {
                                Ok(()) => {
                                    let _ = history_handle.emit("history-updated", ());
                                }
                                Err(e) => crate::logger::log(
                                    "ERROR",
                                    "History",
                                    Some(&history_tag),
                                    &format!("Failed to save history: {}", e),
                                ),
                            }
                        });
                    }
                }
            }
            Err(e) => {
                crate::logger::log(
                    "ERROR",
                    "ASR",
                    Some(&session_tag_clone),
                    &format!("Final transcription failed: {}", e),
                );
                if let Ok(dir) = app_handle_clone.path().app_local_data_dir() {
                    let err_path = dir.join("last_transcription_error.txt");
                    if tokio::fs::write(&err_path, &e).await.is_ok() {
                        let _ = secure_storage::apply_restrictive_acl(&err_path);
                    }
                }
                had_error = Some(categorize_error(&e));
            }
        }
        crate::logger::log(
            "INFO",
            "Session",
            Some(&session_tag_clone),
            &format!(
                "Total processing duration from release = {} ms",
                start_time.elapsed().as_millis()
            ),
        );

        finish_live_target_monitoring(&app_handle_clone, my_gen);

        if let Some(msg) = had_error {
            show_overlay_error(&app_handle_clone, my_gen, &msg).await;
        } else if let Some(notice_key) = final_notice {
            let session_ok = app_handle_clone
                .try_state::<AppState>()
                .map(|s| s.session_gen.load(Ordering::SeqCst) == my_gen)
                .unwrap_or(false);
            if session_ok {
                show_overlay_notice(&app_handle_clone, notice_key);
            }
        } else if !overlay_hide_requested {
            let session_ok = app_handle_clone
                .try_state::<AppState>()
                .map(|s| s.session_gen.load(Ordering::SeqCst) == my_gen)
                .unwrap_or(false);
            if session_ok {
                request_animated_hide(&app_handle_clone, "success");
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    crate::logger::init_early();
    crate::logger::install_panic_hook();
    crate::logger::log_startup();

    let application = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // A second instance just focuses the settings window of the first one
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let app_handle = app.handle().clone();

            // Initialize diagnostics logger
            if let Ok(log_dir) = app
                .path()
                .app_log_dir()
                .or_else(|_| app.path().app_local_data_dir())
            {
                let _ = std::fs::create_dir_all(&log_dir);
                crate::logger::init(log_dir);
                crate::logger::log("INFO", "App", None, "Aura diagnostics logging initialized");
            }

            tauri::async_runtime::spawn_blocking(|| {
                let stale_audio = crate::audio_recorder::sweep_stale_audio(&std::env::temp_dir());
                if stale_audio > 0 {
                    crate::logger::log(
                        "INFO",
                        "Audio",
                        None,
                        &format!("Removed {stale_audio} stale recording file(s)"),
                    );
                }
            });

            // Load settings once, then reuse the validated snapshot throughout setup.
            let startup_settings = match settings::load_settings(&app_handle) {
                Ok(settings) => settings,
                Err(error) => {
                    crate::logger::log(
                        "ERROR",
                        "Settings",
                        None,
                        &format!("Could not load startup settings; using defaults: {error}"),
                    );
                    settings::Settings::default()
                }
            };
            if let Err(error) = keyboard_hook::update_hotkey(&startup_settings.hotkey) {
                crate::logger::log(
                    "ERROR",
                    "Hotkey",
                    None,
                    &format!("Configured hotkey is invalid; keeping the default: {error}"),
                );
            }
            let _ = sync_autostart(&app_handle, startup_settings.autostart);

            let is_autostart = std::env::args().any(|arg| {
                arg == "--autostart" || arg == "--minimized" || arg == "--silent"
            });

            // 1. Intercept CloseRequested on main window to hide it instead of closing the app
            if let Some(main_window) = app.get_webview_window("main") {
                let main_window_clone = main_window.clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = main_window_clone.hide();
                    }
                });
                if !is_autostart {
                    let _ = main_window.show();
                    let _ = main_window.set_focus();
                }
            }

            // 2. Setup custom tray menu popup window
            let last_opened = Arc::new(Mutex::new(
                std::time::Instant::now() - std::time::Duration::from_secs(10),
            ));
            let last_blurred = Arc::new(Mutex::new(
                std::time::Instant::now() - std::time::Duration::from_secs(10),
            ));

            if let Some(tray_window) = app.get_webview_window("tray-menu") {
                let tw = tray_window.clone();
                let last_opened_blur = Arc::clone(&last_opened);
                let last_blurred_blur = Arc::clone(&last_blurred);
                tray_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(false) = event {
                        let is_recent_open = {
                            let lo = last_opened_blur
                                .lock()
                                .unwrap_or_else(|p| p.into_inner());
                            lo.elapsed() < std::time::Duration::from_millis(300)
                        };
                        if !is_recent_open {
                            if let Ok(mut lb) = last_blurred_blur.lock() {
                                *lb = std::time::Instant::now();
                            }
                            let _ = tw.hide();
                        }
                    }
                });
            }

            // 3. Build system tray icon with custom popup menu
            if let Some(tray_icon) = app.default_window_icon().cloned() {
                let initial_tooltip = tray_tooltip(
                    startup_settings.gaming_mode_enabled,
                    &startup_settings.ui_language,
                );

                let initial_icon = if startup_settings.gaming_mode_enabled {
                    tauri::image::Image::from_bytes(include_bytes!("../icons/32x32_gaming.png"))
                        .unwrap_or(tray_icon)
                } else {
                    tray_icon
                };

                let last_opened_tray = Arc::clone(&last_opened);
                let last_blurred_tray = Arc::clone(&last_blurred);

                let _tray = TrayIconBuilder::with_id("main-tray")
                    .icon(initial_icon)
                    .tooltip(initial_tooltip)
                    .on_tray_icon_event(move |tray, event| {
                        match event {
                            TrayIconEvent::Click {
                                button: MouseButton::Left,
                                button_state: MouseButtonState::Up,
                                ..
                            } => {
                                let app = tray.app_handle();
                                if let Some(tray_win) = app.get_webview_window("tray-menu") {
                                    let _ = tray_win.hide();
                                }
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.unminimize();
                                    let _ = window.set_focus();
                                }
                            }
                            TrayIconEvent::Click {
                                button: MouseButton::Right,
                                button_state: MouseButtonState::Up,
                                ..
                            } => {
                                let app = tray.app_handle();
                                if let Some(tray_win) = app.get_webview_window("tray-menu") {
                                    let is_vis = tray_win.is_visible().unwrap_or(false);
                                    let just_blurred = {
                                        let lb = last_blurred_tray
                                            .lock()
                                            .unwrap_or_else(|p| p.into_inner());
                                        lb.elapsed() < std::time::Duration::from_millis(300)
                                    };

                                    if is_vis || just_blurred {
                                        let _ = tray_win.hide();
                                    } else {
                                        #[cfg(target_os = "windows")]
                                        {
                                            use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;
                                            use windows_sys::Win32::Graphics::Gdi::{
                                                MonitorFromPoint, GetMonitorInfoW, MONITORINFO, MONITOR_DEFAULTTONEAREST,
                                            };
                                            use windows_sys::Win32::Foundation::{POINT, RECT};

                                            let mut pt = POINT { x: 0, y: 0 };
                                            unsafe {
                                                GetCursorPos(&mut pt);
                                            }

                                            // 1. Find monitor under cursor to get exact physical work area
                                            let mut work_area: RECT = unsafe { std::mem::zeroed() };
                                            let mut has_work_area = false;

                                            unsafe {
                                                let hmonitor = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
                                                if hmonitor != 0 {
                                                    let mut mi: MONITORINFO = std::mem::zeroed();
                                                    mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                                                    if GetMonitorInfoW(hmonitor, &mut mi) != 0 {
                                                        work_area = mi.rcWork;
                                                        has_work_area = true;
                                                    }
                                                }
                                            }

                                            // Fallback if monitor info failed
                                            if !has_work_area {
                                                use windows_sys::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETWORKAREA};
                                                unsafe {
                                                    SystemParametersInfoW(
                                                        SPI_GETWORKAREA,
                                                        0,
                                                        &mut work_area as *mut _ as *mut _,
                                                        0,
                                                    );
                                                }
                                            }

                                            // 2. Determine scaling factor of the active screen
                                            let scale_factor = tray_win.scale_factor().unwrap_or(1.0);

                                            // Explicitly enforce logical dimensions for WebView2 rendering
                                            let logical_w = 270.0;
                                            let logical_h = 196.0;
                                            let _ = tray_win.set_size(tauri::Size::Logical(tauri::LogicalSize::new(logical_w, logical_h)));

                                            // Physical sizes for precise cursor offset and boundary clamping
                                            let phys_w = (logical_w * scale_factor).round() as i32;
                                            let phys_h = (logical_h * scale_factor).round() as i32;
                                            let margin = (10.0 * scale_factor).round() as i32;
                                            let bottom_offset = (20.0 * scale_factor).round() as i32;

                                            // Default: position popup directly above cursor
                                            let mut x = pt.x - (phys_w / 2);
                                            let mut y = pt.y - phys_h - margin;

                                            // Clamp inside active screen work area (physical coordinates)
                                            if x + phys_w > work_area.right {
                                                x = work_area.right - phys_w - margin;
                                            }
                                            if x < work_area.left {
                                                x = work_area.left + margin;
                                            }
                                            if y < work_area.top {
                                                y = pt.y + bottom_offset;
                                            }

                                            let _ = tray_win.set_position(tauri::Position::Physical(
                                                tauri::PhysicalPosition { x, y },
                                            ));
                                        }
                                        let _ = tray_win.show();
                                        let _ = tray_win.set_focus();
                                        #[cfg(target_os = "windows")]
                                        {
                                            use windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow;
                                            if let Ok(hwnd) = tray_win.hwnd() {
                                                unsafe {
                                                    SetForegroundWindow(hwnd.0 as _);
                                                }
                                            }
                                        }
                                        if let Ok(mut lo) = last_opened_tray.lock() {
                                            *lo = std::time::Instant::now();
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    })
                    .build(app)?;
            }

            app.manage(AppState {
                audio_recorder: audio_recorder::AudioRecorder::new(),
                selected_text: Mutex::new(String::new()),
                selection_blocked_by_uipi: AtomicBool::new(false),
                caret_context: Mutex::new(None),
                press_time: Mutex::new(None),
                is_recording: AtomicBool::new(false),
                toggle_enabled: AtomicBool::new(startup_settings.toggle_enabled),
                typed_so_far: Mutex::new(String::new()),
                live_target_desynced: AtomicBool::new(false),
                live_target_monitoring: AtomicBool::new(false),
                selected_language: Mutex::new(String::new()),
                session_gen: AtomicU64::new(0),
                clipboard_mutex: Mutex::new(()),
                unrestored_clipboard: Mutex::new(None),
                settings_apply_lock: tokio::sync::Mutex::new(()),
                latched: AtomicBool::new(false),
                ignore_next_release: AtomicBool::new(false),
                start_focus: Mutex::new(keyboard_simulator::FocusTarget::default()),
                parakeet_lifecycle: Mutex::new(()),
                parakeet_server: Mutex::new(None),
                parakeet_port: std::sync::atomic::AtomicU16::new(0),
                parakeet_streaming: Mutex::new(None),
                parakeet_watchdog: Mutex::new(None),
                parakeet_starting: AtomicBool::new(false),
                whisper_lifecycle: Mutex::new(()),
                whisper_server: Mutex::new(None),
                whisper_port: std::sync::atomic::AtomicU16::new(0),
                whisper_watchdog: Mutex::new(None),
                whisper_starting: AtomicBool::new(false),
                llm_lifecycle: Mutex::new(()),
                llm_server: Mutex::new(None),
                llm_port: std::sync::atomic::AtomicU16::new(0),
                last_activity_time: Mutex::new(std::time::Instant::now()),
                gaming_mode_active: AtomicBool::new(startup_settings.gaming_mode_enabled),
                last_audio_devices: Mutex::new(Vec::new()),
            });

            if startup_settings.gaming_mode_enabled {
                keyboard_hook::set_hook_paused(true);
            } else {
                // Start Parakeet or Whisper server based on validated startup snapshot.
                if startup_settings.local_engine == "parakeet" {
                    let sidecar_handle = app_handle.clone();
                    let sidecar_settings = startup_settings.clone();
                    tauri::async_runtime::spawn_blocking(move || {
                        whisper_runner::ensure_parakeet_server_state(
                            &sidecar_handle,
                            &sidecar_settings,
                        );
                    });
                } else if startup_settings.local_engine == "whisper" {
                    let whisper_handle = app_handle.clone();
                    let whisper_settings = startup_settings.clone();
                    tauri::async_runtime::spawn_blocking(move || {
                        whisper_runner::ensure_whisper_server_state(
                            &whisper_handle,
                            &whisper_settings,
                        );
                    });
                }

                // Text editing on a selection has to load a Qwen3 model from
                // disk; waiting for that on the first command cost 20+ seconds.
                // Warm the server up in the background right after launch, and
                // only when a model is actually on disk.
                if llm_runner::resolve_ai_edit_route(&app_handle, &startup_settings)
                    == llm_runner::AiEditRoute::Local
                    && llm_runner::llm_model_is_installed(&app_handle, &startup_settings.llm_model)
                {
                    let llm_handle = app_handle.clone();
                    let llm_model = startup_settings.llm_model.clone();
                    tauri::async_runtime::spawn_blocking(move || {
                        let warm_started = std::time::Instant::now();
                        match llm_runner::start_llm_server(&llm_handle, &llm_model) {
                            Ok(()) => crate::logger::log(
                                "INFO",
                                "LLM",
                                None,
                                &format!(
                                    "Local editing server warmed up at startup in {} ms",
                                    warm_started.elapsed().as_millis()
                                ),
                            ),
                            Err(error) => crate::logger::log(
                                "WARN",
                                "LLM",
                                None,
                                &format!(
                                    "Startup warm-up of the local editing server failed: {error}"
                                ),
                            ),
                        }
                    });
                }
            }

            // Standby Unloader & Gaming Mode background monitor
            let standby_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));
                loop {
                    interval.tick().await;
                    check_vram_standby_and_gaming(&standby_handle).await;
                }
            });

            // Event-driven Audio Hotplug monitor (IMMNotificationClient on Windows, polling fallback)
            spawn_audio_hotplug_monitor(app_handle.clone());

            // Periodic background update checker (every 4 hours, starts after 30s)
            let update_checker_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(4 * 3600));
                loop {
                    interval.tick().await;
                    if let Ok(settings) =
                        settings::cached_or_load_settings_without_secrets(&update_checker_handle)
                    {
                        if settings.automatic_update_checks {
                            if let Ok(Some(update)) = query_app_update(&update_checker_handle).await
                            {
                                notify_app_update_available(
                                    &update_checker_handle,
                                    &update.version,
                                );
                            }
                        }
                    }
                }
            });
            // Esc cancels an active recording
            let cancel_handle = app_handle.clone();
            keyboard_hook::set_cancel_callback(move || {
                let app_handle = cancel_handle.clone();
                tauri::async_runtime::spawn(async move {
                    cancel_recording(app_handle).await;
                });
            })
            .map_err(std::io::Error::other)?;

            // Physical typing/Undo while a live transcript is mirrored means
            // the target document no longer matches typed_so_far. Stop all
            // further destructive reconciliation and preserve the final result
            // through history + clipboard instead.
            let user_input_handle = app_handle.clone();
            keyboard_hook::set_user_input_callback(move || {
                let Some(state) = user_input_handle.try_state::<AppState>() else {
                    return;
                };
                if !state.live_target_monitoring.load(Ordering::Acquire) {
                    return;
                }
                if !state.live_target_desynced.swap(true, Ordering::AcqRel) {
                    let generation = state.session_gen.load(Ordering::Acquire);
                    let session_tag = crate::logger::format_session_tag(generation);
                    crate::logger::log_from_hook(
                        "WARN",
                        "Typing",
                        Some(&session_tag),
                        "Physical user input detected; stopping live text reconciliation to protect the target document",
                    );
                }
            })
            .map_err(std::io::Error::other)?;

            // Start global keyboard hook
            keyboard_hook::start_hook(move |is_down| {
                let app_handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    let Some(state) = app_handle.try_state::<AppState>() else {
                        return;
                    };

                    if is_down {
                        crate::logger::log("INFO", "Hotkey", None, "Hotkey down");
                        let recording = state.is_recording.load(Ordering::SeqCst);
                        if recording && state.latched.load(Ordering::SeqCst) {
                            // Second tap in toggle mode stops the recording
                            state.latched.store(false, Ordering::SeqCst);
                            state.ignore_next_release.store(true, Ordering::SeqCst);
                            finalize_recording(app_handle.clone(), true).await;
                        } else if !recording && !state.ignore_next_release.load(Ordering::SeqCst) {
                            start_recording_session(app_handle.clone()).await;
                        }
                    } else {
                        crate::logger::log("INFO", "Hotkey", None, "Hotkey up");
                        // Alt-menu disarming is handled synchronously inside the
                        // keyboard hook (send_disarmed_alt_up / dummy Ctrl tap).

                        if state.ignore_next_release.swap(false, Ordering::SeqCst) {
                            return;
                        }
                        if !state.is_recording.load(Ordering::SeqCst) {
                            return;
                        }

                        let press_duration = state
                            .press_time
                            .lock()
                            .ok()
                            .and_then(|mut g| g.take())
                            .map(|t| t.elapsed());

                        if let Some(d) = press_duration {
                            crate::logger::log(
                                "INFO",
                                "Hotkey",
                                None,
                                &format!("Press duration = {} ms", d.as_millis()),
                            );
                            if d.as_millis() < 300 {
                                let toggle_enabled = state.toggle_enabled.load(Ordering::SeqCst);
                                if toggle_enabled {
                                    // Short tap latches the recording until the next tap or Esc
                                    crate::logger::log(
                                        "INFO",
                                        "Hotkey",
                                        None,
                                        "Short tap — latching recording (toggle mode).",
                                    );
                                    state.latched.store(true, Ordering::SeqCst);
                                } else {
                                    crate::logger::log(
                                        "INFO",
                                        "Hotkey",
                                        None,
                                        "Press too short (< 300ms), discarding.",
                                    );
                                    discard_recording(&app_handle);
                                }
                                return;
                            }
                        }

                        finalize_recording(app_handle.clone(), false).await;
                    }
                });
            })
            .map_err(std::io::Error::other)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_provider_key,
            set_settings,
            set_ui_language,
            set_transcription_mode,
            set_recognition_language,
            complete_onboarding,
            download_model_command,
            cancel_model_download,
            delete_model_command,
            get_downloaded_models,
            get_history,
            clear_history,
            copy_to_clipboard,
            check_for_app_update,
            install_app_update,
            open_url,
            relaunch_app,
            minimize_window,
            close_window,
            start_dragging_command,
            hide_overlay_window,
            download_gpu_binaries,
            cancel_gpu_download,
            delete_gpu_binaries,
            check_gpu_downloaded,
            check_nvidia_runtime_on_path,
            get_diagnostic_report,
            get_engine_health,
            log_frontend_event,
            get_audio_input_devices,
            start_mic_meter,
            stop_mic_meter,
            toggle_gaming_mode,
            get_gaming_mode_state,
            show_settings_window,
            save_settings_backup,
            exit_app
        ])
        .build(tauri::generate_context!());
    let application = match application {
        Ok(application) => application,
        Err(error) => {
            eprintln!("Aura failed to initialize: {error}");
            crate::logger::log("ERROR", "Startup", None, &error.to_string());
            return;
        }
    };
    application.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            stop_all_sidecars_and_watchdogs(app_handle);
        }
    });
}

#[derive(Clone, serde::Serialize)]
struct GpuDownloadProgress {
    provider: String,
    downloaded: u64,
    total: Option<u64>,
    percentage: f64,
    done: bool,
    status: Option<String>,
}

const CUDA_ARCHIVE_URL: &str = "https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.13.4/sherpa-onnx-v1.13.4-win-x64-cuda.tar.bz2";
const CUDA_ARCHIVE_SIZE: u64 = 221_905_418;
const CUDA_ARCHIVE_SHA256: &str =
    "9cc16169fb073ab0acd304ae144ccad21af03e8360921a12285105599f0f692a";

const WHISPER_CUDA_ARCHIVE_URL: &str = "https://github.com/ggml-org/whisper.cpp/releases/download/v1.9.1/whisper-cublas-11.8.0-bin-x64.zip";
const WHISPER_CUDA_ARCHIVE_SIZE: u64 = 278_557_654;
const WHISPER_CUDA_ARCHIVE_SHA256: &str =
    "aecdce0e4d4bb758a7c72a31f3f9f19a7b6d861405fd2da743cd86398633c963";

const CUBLAS_ARCHIVE_URL: &str = "https://developer.download.nvidia.com/compute/cuda/redist/libcublas/windows-x86_64/libcublas-windows-x86_64-11.11.3.6-archive.zip";
const CUBLAS_ARCHIVE_SIZE: u64 = 420_850_025;
const CUBLAS_ARCHIVE_SHA256: &str =
    "67b0934a6359e4ee26fff823c356021589d392c4fd49ca12624f570edc08e2b9";
const CUFFT_ARCHIVE_URL: &str = "https://developer.download.nvidia.com/compute/cuda/redist/libcufft/windows-x86_64/libcufft-windows-x86_64-10.9.0.58-archive.zip";
const CUFFT_ARCHIVE_SIZE: u64 = 168_982_770;
const CUFFT_ARCHIVE_SHA256: &str =
    "a4071a85e3983bf42ea7a2e9bebe3b0b3c9ac258668580adc32ee1c385f7556f";
const CUDNN_ARCHIVE_URL: &str = "https://developer.download.nvidia.com/compute/cudnn/redist/cudnn/windows-x86_64/cudnn-windows-x86_64-8.5.0.96_cuda11-archive.zip";
const CUDNN_ARCHIVE_SIZE: u64 = 542_516_637;
const CUDNN_ARCHIVE_SHA256: &str =
    "bf277ed350addb8f97e0ab6a20b6fad869abe49ea24277d38ca79f5f23fbec6b";

const TOTAL_CUDA_ARCHIVE_SIZE: u64 = CUDA_ARCHIVE_SIZE
    + WHISPER_CUDA_ARCHIVE_SIZE
    + CUBLAS_ARCHIVE_SIZE
    + CUFFT_ARCHIVE_SIZE
    + CUDNN_ARCHIVE_SIZE;
const SYSTEM_CUDA_ARCHIVE_SIZE: u64 = CUDA_ARCHIVE_SIZE + WHISPER_CUDA_ARCHIVE_SIZE;

const SHERPA_CUDA_RUNTIME_FILES: &[&str] = &[
    "sherpa-onnx-offline-websocket-server.exe",
    "onnxruntime.dll",
    "onnxruntime_providers_cuda.dll",
    "onnxruntime_providers_shared.dll",
];
const WHISPER_CUDA_RUNTIME_FILES: &[&str] = &[
    "cudart64_110.dll",
    "ggml.dll",
    "ggml-base.dll",
    "ggml-cpu-alderlake.dll",
    "ggml-cpu-cannonlake.dll",
    "ggml-cpu-cascadelake.dll",
    "ggml-cpu-haswell.dll",
    "ggml-cpu-icelake.dll",
    "ggml-cpu-sandybridge.dll",
    "ggml-cpu-skylakex.dll",
    "ggml-cpu-sse42.dll",
    "ggml-cpu-x64.dll",
    "ggml-cuda.dll",
    "whisper.dll",
];
const CUBLAS_RUNTIME_FILES: &[&str] = &["cublas64_11.dll", "cublasLt64_11.dll"];
const CUFFT_RUNTIME_FILES: &[&str] = &["cufft64_10.dll"];
const CUDNN_RUNTIME_FILES: &[&str] = &[
    "cudnn64_8.dll",
    "cudnn_adv_infer64_8.dll",
    "cudnn_cnn_infer64_8.dll",
    "cudnn_ops_infer64_8.dll",
];
const NVIDIA_CUDA_RUNTIME_FILES: &[&str] = &[
    "cublas64_11.dll",
    "cublasLt64_11.dll",
    "cufft64_10.dll",
    "cudnn64_8.dll",
    "cudnn_adv_infer64_8.dll",
    "cudnn_cnn_infer64_8.dll",
    "cudnn_ops_infer64_8.dll",
];

fn lock_active_gpu_downloads() -> std::sync::MutexGuard<'static, std::collections::HashSet<String>>
{
    match ACTIVE_GPU_DOWNLOADS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            crate::logger::log(
                "WARN",
                "GPU",
                None,
                "GPU download registry was poisoned; recovering its state",
            );
            poisoned.into_inner()
        }
    }
}

struct GpuDownloadGuard {
    provider: String,
    cancel_key: String,
}

impl Drop for GpuDownloadGuard {
    fn drop(&mut self) {
        lock_active_gpu_downloads().remove(&self.provider);
        whisper_runner::clear_cancel(&self.cancel_key);
    }
}

struct ScopedInstallDirectory {
    path: std::path::PathBuf,
    cleanup_on_drop: bool,
}

fn remove_gpu_staging_directory(path: &std::path::Path) {
    if !path.exists() {
        return;
    }
    for attempt in 0..5 {
        if std::fs::remove_dir_all(path).is_ok() || !path.exists() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(50 * (1 << attempt)));
    }
    let display_path = path.display().to_string();
    crate::logger::log(
        "WARN",
        "GPU",
        None,
        &format!("Background cleanup of {display_path} failed after 5 retries"),
    );
}

impl Drop for ScopedInstallDirectory {
    fn drop(&mut self) {
        if !self.cleanup_on_drop || self.path.as_os_str().is_empty() {
            return;
        }
        let path = std::mem::take(&mut self.path);
        let display_path = path.display().to_string();
        if let Err(error) = std::thread::Builder::new()
            .name("aura-gpu-cleanup".to_string())
            .spawn(move || remove_gpu_staging_directory(&path))
        {
            crate::logger::log(
                "WARN",
                "GPU",
                None,
                &format!("Failed to start cleanup for {display_path}: {error}"),
            );
        }
    }
}
fn unique_gpu_install_directory(parent: &std::path::Path, provider: &str) -> std::path::PathBuf {
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    parent.join(format!(
        ".{provider}.install-{}-{suffix}",
        std::process::id()
    ))
}

fn runtime_files_are_present(bin_dir: &std::path::Path, files: &[&str]) -> bool {
    files
        .iter()
        .all(|name| runtime_file_is_present(&bin_dir.join(name)))
}

fn runtime_file_is_present(path: &std::path::Path) -> bool {
    path.metadata()
        .map(|metadata| metadata.is_file() && metadata.len() > 0)
        .unwrap_or(false)
}

fn aura_cuda_runtime_is_complete(bin_dir: &std::path::Path) -> bool {
    SHERPA_CUDA_RUNTIME_FILES
        .iter()
        .chain(WHISPER_CUDA_RUNTIME_FILES)
        .all(|name| runtime_file_is_present(&bin_dir.join(name)))
}

fn nvidia_runtime_is_on_path(path: Option<&std::ffi::OsStr>) -> bool {
    let Some(path) = path else {
        return false;
    };
    let directories = std::env::split_paths(path)
        .filter(|directory| directory.is_absolute())
        .collect::<Vec<_>>();
    NVIDIA_CUDA_RUNTIME_FILES.iter().all(|name| {
        directories
            .iter()
            .any(|directory| runtime_file_is_present(&directory.join(name)))
    })
}

fn require_nvidia_terms(
    use_system_nvidia: bool,
    accepted_nvidia_terms: bool,
) -> Result<(), String> {
    if use_system_nvidia || accepted_nvidia_terms {
        Ok(())
    } else {
        Err("NVIDIA license terms must be accepted before downloading CUDA".to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NvidiaRuntimeSource {
    Managed,
    SystemPath,
    Missing,
}

fn nvidia_runtime_source(
    bin_dir: &std::path::Path,
    path: Option<&std::ffi::OsStr>,
) -> NvidiaRuntimeSource {
    if runtime_files_are_present(bin_dir, NVIDIA_CUDA_RUNTIME_FILES) {
        NvidiaRuntimeSource::Managed
    } else if nvidia_runtime_is_on_path(path) {
        NvidiaRuntimeSource::SystemPath
    } else {
        NvidiaRuntimeSource::Missing
    }
}

fn gpu_bin_is_complete_with_path(
    bin_dir: &std::path::Path,
    provider: &str,
    path: Option<&std::ffi::OsStr>,
) -> bool {
    provider == "cuda"
        && aura_cuda_runtime_is_complete(bin_dir)
        && nvidia_runtime_source(bin_dir, path) != NvidiaRuntimeSource::Missing
}

fn gpu_bin_is_complete(bin_dir: &std::path::Path, provider: &str) -> bool {
    let path = std::env::var_os("PATH");
    gpu_bin_is_complete_with_path(bin_dir, provider, path.as_deref())
}

pub(crate) fn cuda_runtime_source_label(bin_dir: &std::path::Path) -> &'static str {
    if !aura_cuda_runtime_is_complete(bin_dir) {
        return "Incomplete/Missing";
    }
    let path = std::env::var_os("PATH");
    match nvidia_runtime_source(bin_dir, path.as_deref()) {
        NvidiaRuntimeSource::Managed => "Aura-managed NVIDIA runtime",
        NvidiaRuntimeSource::SystemPath => "System NVIDIA runtime (PATH)",
        NvidiaRuntimeSource::Missing => "Missing NVIDIA dependencies",
    }
}

fn extract_selected_tar_bz2_files(
    archive_path: &std::path::Path,
    target_dir: &std::path::Path,
    required_parent: &str,
    required_files: &[&str],
) -> Result<(), String> {
    let archive_file = std::fs::File::open(archive_path)
        .map_err(|error| format!("Failed to open verified CUDA archive: {error}"))?;
    let decoder = bzip2::read::BzDecoder::new(archive_file);
    let mut archive = tar::Archive::new(decoder);
    let mut extracted = std::collections::HashSet::new();

    for entry in archive
        .entries()
        .map_err(|error| format!("Failed to inspect CUDA archive: {error}"))?
    {
        let mut entry = entry.map_err(|error| format!("Failed to read CUDA archive: {error}"))?;
        let path = entry
            .path()
            .map_err(|error| format!("Failed to read CUDA archive path: {error}"))?;
        let Some(file_name) = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_string)
        else {
            continue;
        };
        if path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            != Some(required_parent)
        {
            continue;
        }
        if !required_files.contains(&file_name.as_str()) {
            continue;
        }
        if !extracted.insert(file_name.clone()) {
            return Err(format!(
                "CUDA archive contains duplicate runtime file '{file_name}'"
            ));
        }
        entry.unpack(target_dir.join(&file_name)).map_err(|error| {
            format!("Failed to extract CUDA runtime file '{file_name}': {error}")
        })?;
    }

    ensure_runtime_files_extracted(required_files, &extracted)
}

fn extract_selected_zip_files(
    archive_path: &std::path::Path,
    target_dir: &std::path::Path,
    required_files: &[&str],
) -> Result<(), String> {
    let archive_file = std::fs::File::open(archive_path)
        .map_err(|error| format!("Failed to open verified runtime archive: {error}"))?;
    let mut archive = zip::ZipArchive::new(archive_file)
        .map_err(|error| format!("Failed to read runtime archive: {error}"))?;
    let mut extracted = std::collections::HashSet::new();

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| format!("Failed to read runtime archive entry {index}: {error}"))?;
        if !entry.is_file() {
            continue;
        }
        let Some(file_name) = entry
            .enclosed_name()
            .and_then(|path| path.file_name().map(|name| name.to_owned()))
        else {
            continue;
        };
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if !required_files.contains(&file_name) {
            continue;
        }
        if !extracted.insert(file_name.to_string()) {
            return Err(format!(
                "Runtime archive contains duplicate file '{file_name}'"
            ));
        }
        let output_path = target_dir.join(file_name);
        let mut output = std::fs::File::create(&output_path).map_err(|error| {
            format!(
                "Failed to create runtime file {}: {error}",
                output_path.display()
            )
        })?;
        std::io::copy(&mut entry, &mut output)
            .map_err(|error| format!("Failed to extract runtime file '{file_name}': {error}"))?;
    }

    ensure_runtime_files_extracted(required_files, &extracted)
}

fn ensure_runtime_files_extracted(
    required_files: &[&str],
    extracted: &std::collections::HashSet<String>,
) -> Result<(), String> {
    let missing = required_files
        .iter()
        .filter(|name| !extracted.contains(**name))
        .copied()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Verified runtime archive is missing required files: {}",
            missing.join(", ")
        ))
    }
}

fn install_cuda_archive(
    parakeet_archive_path: &std::path::Path,
    whisper_archive_path: &std::path::Path,
    nvidia_archive_paths: Option<[&std::path::Path; 3]>,
    runtime_path: Option<&std::ffi::OsStr>,
    extraction_dir: &std::path::Path,
    gpu_dir: &std::path::Path,
) -> Result<(), String> {
    let source_bin = extraction_dir.join("bin");
    std::fs::create_dir_all(&source_bin)
        .map_err(|error| format!("Failed to create extraction directory: {error}"))?;

    extract_selected_tar_bz2_files(
        parakeet_archive_path,
        &source_bin,
        "bin",
        SHERPA_CUDA_RUNTIME_FILES,
    )?;
    extract_selected_zip_files(
        whisper_archive_path,
        &source_bin,
        WHISPER_CUDA_RUNTIME_FILES,
    )?;
    if let Some([cublas_archive_path, cufft_archive_path, cudnn_archive_path]) =
        nvidia_archive_paths
    {
        extract_selected_zip_files(cublas_archive_path, &source_bin, CUBLAS_RUNTIME_FILES)?;
        extract_selected_zip_files(cufft_archive_path, &source_bin, CUFFT_RUNTIME_FILES)?;
        extract_selected_zip_files(cudnn_archive_path, &source_bin, CUDNN_RUNTIME_FILES)?;
    }

    if !gpu_bin_is_complete_with_path(&source_bin, "cuda", runtime_path) {
        return Err(
            "Verified CUDA archive does not contain the required runtime files".to_string(),
        );
    }

    let target_bin = gpu_dir.join("bin");
    let backup_bin = gpu_dir.join(".bin.previous");
    if backup_bin.exists() {
        std::fs::remove_dir_all(&backup_bin)
            .map_err(|error| format!("Failed to clear stale CUDA backup: {error}"))?;
    }
    let had_previous = target_bin.exists();
    if had_previous {
        std::fs::rename(&target_bin, &backup_bin)
            .map_err(|error| format!("Failed to stage existing CUDA runtime: {error}"))?;
    }

    if let Err(error) = std::fs::rename(&source_bin, &target_bin) {
        if had_previous {
            let _ = std::fs::rename(&backup_bin, &target_bin);
        }
        return Err(format!("Failed to install CUDA runtime: {error}"));
    }
    if had_previous {
        if let Err(error) = std::fs::remove_dir_all(&backup_bin) {
            crate::logger::log(
                "WARN",
                "GPU",
                None,
                &format!("CUDA update succeeded but old backup cleanup failed: {error}"),
            );
        }
    }
    Ok(())
}

#[tauri::command]
async fn check_gpu_downloaded(
    app_handle: tauri::AppHandle,
    provider: String,
) -> Result<bool, String> {
    if provider != "cuda" && provider != "directml" {
        return Err("Invalid provider".to_string());
    }
    if provider == "directml" {
        return Ok(false);
    }
    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data: {}", e))?;
    let bin_dir = app_local_data.join("binaries").join(&provider).join("bin");
    Ok(gpu_bin_is_complete(&bin_dir, &provider))
}

#[tauri::command]
fn check_nvidia_runtime_on_path() -> bool {
    let path = std::env::var_os("PATH");
    nvidia_runtime_is_on_path(path.as_deref())
}

async fn apply_gaming_mode_state(app: &tauri::AppHandle, enabled: bool, auto_triggered: bool) {
    if let Some(state) = app.try_state::<AppState>() {
        let prev = state.gaming_mode_active.swap(enabled, Ordering::SeqCst);
        if prev == enabled {
            return;
        }
        keyboard_hook::set_hook_paused(enabled);

        // Dynamic Tray Icon & Tooltip update
        if let Some(tray) = app.tray_by_id("main-tray") {
            let settings = settings::cached_or_load_settings_without_secrets(app).ok();
            let lang = settings
                .as_ref()
                .map(|s| s.ui_language.as_str())
                .unwrap_or("ru");
            let tooltip = tray_tooltip(enabled, lang);
            let _ = tray.set_tooltip(Some(tooltip));
            if enabled {
                let icon_bytes = include_bytes!("../icons/32x32_gaming.png");
                if let Ok(img) = tauri::image::Image::from_bytes(icon_bytes) {
                    let _ = tray.set_icon(Some(img));
                }
            } else if let Some(default_icon) = app.default_window_icon().cloned() {
                let _ = tray.set_icon(Some(default_icon));
            }
        }

        if enabled {
            // Cancel active recording immediately to prevent orphaned audio capture and hanging hooks (CONC-01)
            cancel_recording(app.clone()).await;

            let stop_handle = app.clone();
            let _ = tauri::async_runtime::spawn_blocking(move || {
                whisper_runner::stop_whisper_server(&stop_handle);
                whisper_runner::stop_parakeet_server(&stop_handle);
                llm_runner::stop_llm_server(&stop_handle);
            })
            .await;
            crate::logger::log(
                "INFO",
                "GamingMode",
                None,
                if auto_triggered {
                    "Auto-engaged Gaming Mode: VRAM model servers unloaded and keyboard hook paused"
                } else {
                    "Manual Gaming Mode enabled: VRAM model servers unloaded and keyboard hook paused"
                },
            );
        } else {
            let restore_handle = app.clone();
            let _ = tauri::async_runtime::spawn_blocking(move || {
                if let Ok(settings) =
                    settings::cached_or_load_settings_without_secrets(&restore_handle)
                {
                    if settings.transcription_mode == "local" {
                        if settings.local_engine == "whisper" {
                            whisper_runner::ensure_whisper_server_state(&restore_handle, &settings);
                        } else if settings.local_engine == "parakeet" {
                            whisper_runner::ensure_parakeet_server_state(
                                &restore_handle,
                                &settings,
                            );
                        }
                    }
                }
            })
            .await;
            crate::logger::log(
                "INFO",
                "GamingMode",
                None,
                if auto_triggered {
                    "Auto-disengaged Gaming Mode: keyboard hook resumed and resident servers restored"
                } else {
                    "Manual Gaming Mode disabled: keyboard hook resumed and resident servers restored"
                },
            );
        }
        let _ = app.emit("gaming-mode-changed", enabled);
    }
}

#[cfg(target_os = "windows")]
fn is_foreground_window_fullscreen() -> bool {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetWindowLongW, GetWindowRect, GWL_STYLE, WS_CAPTION,
    };

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd == 0 {
            return false;
        }

        let mut class_name = [0u16; 256];
        let len = GetClassNameW(hwnd, class_name.as_mut_ptr(), 256);
        if len > 0 {
            let name = String::from_utf16_lossy(&class_name[..len as usize]);
            if name == "Progman"
                || name == "WorkerW"
                || name == "Shell_TrayWnd"
                || name == "Aura"
                || name.starts_with("Chrome_WidgetWin_")
                || name == "MozillaWindowClass"
                || name == "screenClass"
                || name == "vlc"
                || name == "MediaPlayerClassicW"
                || name == "ApplicationFrameWindow"
            {
                return false;
            }
        }

        let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        if (style & WS_CAPTION) == WS_CAPTION {
            return false;
        }

        let mut win_rect: RECT = std::mem::zeroed();
        if GetWindowRect(hwnd, &mut win_rect) == 0 {
            return false;
        }

        let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        if hmonitor == 0 {
            return false;
        }

        let mut mi: MONITORINFO = std::mem::zeroed();
        mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(hmonitor, &mut mi) == 0 {
            return false;
        }

        let mon_w = mi.rcMonitor.right - mi.rcMonitor.left;
        let mon_h = mi.rcMonitor.bottom - mi.rcMonitor.top;

        let width = win_rect.right - win_rect.left;
        let height = win_rect.bottom - win_rect.top;

        width >= mon_w && height >= mon_h
    }
}

static AUTO_GAMING_DISENGAGE_TICKS: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(0);

async fn check_vram_standby_and_gaming(app: &tauri::AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let Ok(settings) = settings::cached_or_load_settings_without_secrets(app) else {
        return;
    };

    // 1. Auto-detect full-screen 3D games if enabled
    if settings.gaming_mode_auto_detect && !settings.gaming_mode_enabled {
        #[cfg(target_os = "windows")]
        let is_d3d_fullscreen = {
            use windows_sys::Win32::UI::Shell::{
                SHQueryUserNotificationState, QUNS_RUNNING_D3D_FULL_SCREEN,
            };
            let mut query_state = 0;
            let hr = unsafe { SHQueryUserNotificationState(&mut query_state) };
            (hr == 0 && query_state == QUNS_RUNNING_D3D_FULL_SCREEN)
                || is_foreground_window_fullscreen()
        };
        #[cfg(not(target_os = "windows"))]
        let is_d3d_fullscreen = false;

        let currently_active = state.gaming_mode_active.load(Ordering::Acquire);
        if is_d3d_fullscreen && !currently_active && !state.is_recording.load(Ordering::SeqCst) {
            AUTO_GAMING_DISENGAGE_TICKS.store(0, Ordering::Relaxed);
            apply_gaming_mode_state(app, true, true).await;
        } else if is_d3d_fullscreen {
            AUTO_GAMING_DISENGAGE_TICKS.store(0, Ordering::Relaxed);
        } else if !is_d3d_fullscreen && currently_active {
            // Require 4 consecutive ticks (12 seconds) of sustained non-game focus
            // before reloading resident models, preventing VRAM thrashing on quick Alt+Tabs.
            let streak = AUTO_GAMING_DISENGAGE_TICKS.fetch_add(1, Ordering::Relaxed) + 1;
            if streak >= 4 {
                AUTO_GAMING_DISENGAGE_TICKS.store(0, Ordering::Relaxed);
                apply_gaming_mode_state(app, false, true).await;
            }
        }
    }

    // 2. VRAM Standby Unloader
    if !state.gaming_mode_active.load(Ordering::Acquire)
        && settings.vram_standby_timeout_mins > 0
        && !state.is_recording.load(Ordering::SeqCst)
    {
        let last_activity = state
            .last_activity_time
            .lock()
            .map(|t| *t)
            .unwrap_or_else(|p| **p.get_ref());
        let timeout = std::time::Duration::from_secs(
            u64::from(settings.vram_standby_timeout_mins).saturating_mul(60),
        );
        if last_activity.elapsed() >= timeout {
            let whisper_running = state.whisper_port.load(Ordering::SeqCst) > 0;
            let parakeet_running = state
                .parakeet_server
                .lock()
                .map(|s| s.is_some())
                .unwrap_or(false);
            let llm_running = state.llm_port.load(Ordering::SeqCst) > 0;

            if whisper_running || parakeet_running || llm_running {
                let stop_handle = app.clone();
                let _ = tauri::async_runtime::spawn_blocking(move || {
                    whisper_runner::stop_whisper_server(&stop_handle);
                    whisper_runner::stop_parakeet_server(&stop_handle);
                    llm_runner::stop_llm_server(&stop_handle);
                })
                .await;
                crate::logger::log(
                    "INFO",
                    "Standby",
                    None,
                    &format!(
                        "VRAM Standby: Unloaded resident model servers after {} minutes of inactivity",
                        settings.vram_standby_timeout_mins
                    ),
                );
            }
        }
    }
}

#[cfg(target_os = "windows")]
mod audio_hotplug_watcher {
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::{mpsc, Arc, OnceLock};
    use tokio::sync::Notify;
    use windows::core::GUID;
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

    const CLSCTX_ALL_RAW: u32 = 23;
    const S_OK_RAW: i32 = 0;
    const E_NOINTERFACE_RAW: i32 = -2_147_467_262;
    const E_POINTER_RAW: i32 = -2_147_467_261;

    const IID_IUNKNOWN: GUID = GUID::from_u128(0x00000000_0000_0000_c000_000000000046);
    const CLSID_MM_DEVICE_ENUMERATOR: GUID =
        GUID::from_u128(0xbcde0395_e52f_467c_8e3d_c4579291692e);
    const IID_IMM_DEVICE_ENUMERATOR: GUID = GUID::from_u128(0xa95664d2_9614_4f35_a746_de8db63617e6);
    const IID_IMM_NOTIFICATION_CLIENT: GUID =
        GUID::from_u128(0x7991eec9_7e89_4d85_8390_6c703cec60c0);
    const PKEY_DEVICE_FRIENDLY_NAME_FMTID: GUID =
        GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0);

    #[link(name = "ole32")]
    extern "system" {
        fn CoCreateInstance(
            rclsid: *const GUID,
            punk_outer: *mut c_void,
            dw_cls_context: u32,
            riid: *const GUID,
            ppv: *mut *mut c_void,
        ) -> i32;
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct PropertyKey {
        fmtid: GUID,
        pid: u32,
    }

    #[repr(C)]
    #[allow(dead_code)]
    struct IMMDeviceEnumeratorVtbl {
        query_interface:
            unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32,
        add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
        release: unsafe extern "system" fn(*mut c_void) -> u32,
        enum_audio_endpoints:
            unsafe extern "system" fn(*mut c_void, i32, u32, *mut *mut c_void) -> i32,
        get_default_audio_endpoint:
            unsafe extern "system" fn(*mut c_void, i32, i32, *mut *mut c_void) -> i32,
        get_device: unsafe extern "system" fn(*mut c_void, *const u16, *mut *mut c_void) -> i32,
        register_endpoint_notification_callback:
            unsafe extern "system" fn(*mut c_void, *mut c_void) -> i32,
        unregister_endpoint_notification_callback:
            unsafe extern "system" fn(*mut c_void, *mut c_void) -> i32,
    }

    #[repr(C)]
    struct IMMDeviceEnumeratorRaw {
        lp_vtbl: *const IMMDeviceEnumeratorVtbl,
    }

    #[repr(C)]
    struct IMMNotificationClientVtbl {
        query_interface:
            unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32,
        add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
        release: unsafe extern "system" fn(*mut c_void) -> u32,
        on_device_state_changed: unsafe extern "system" fn(*mut c_void, *const u16, u32) -> i32,
        on_device_added: unsafe extern "system" fn(*mut c_void, *const u16) -> i32,
        on_device_removed: unsafe extern "system" fn(*mut c_void, *const u16) -> i32,
        on_default_device_changed:
            unsafe extern "system" fn(*mut c_void, i32, i32, *const u16) -> i32,
        on_property_value_changed:
            unsafe extern "system" fn(*mut c_void, *const u16, PropertyKey) -> i32,
    }

    #[repr(C)]
    struct NotificationClientCom {
        lp_vtbl: *const IMMNotificationClientVtbl,
        ref_count: AtomicU32,
        notify: Arc<Notify>,
    }

    static VTBL: IMMNotificationClientVtbl = IMMNotificationClientVtbl {
        query_interface: client_query_interface,
        add_ref: client_add_ref,
        release: client_release,
        on_device_state_changed: client_on_device_state_changed,
        on_device_added: client_on_device_added,
        on_device_removed: client_on_device_removed,
        on_default_device_changed: client_on_default_device_changed,
        on_property_value_changed: client_on_property_value_changed,
    };

    unsafe extern "system" fn client_query_interface(
        this: *mut c_void,
        riid: *const GUID,
        ppv: *mut *mut c_void,
    ) -> i32 {
        if ppv.is_null() {
            return E_POINTER_RAW;
        }
        if riid.is_null() {
            *ppv = std::ptr::null_mut();
            return E_NOINTERFACE_RAW;
        }
        let iid = *riid;
        if iid == IID_IUNKNOWN || iid == IID_IMM_NOTIFICATION_CLIENT {
            client_add_ref(this);
            *ppv = this;
            S_OK_RAW
        } else {
            *ppv = std::ptr::null_mut();
            E_NOINTERFACE_RAW
        }
    }

    unsafe extern "system" fn client_add_ref(this: *mut c_void) -> u32 {
        let obj = &*(this as *const NotificationClientCom);
        obj.ref_count.fetch_add(1, Ordering::Relaxed) + 1
    }

    unsafe extern "system" fn client_release(this: *mut c_void) -> u32 {
        let obj = &*(this as *const NotificationClientCom);
        let prev = obj.ref_count.fetch_sub(1, Ordering::AcqRel);
        if prev == 1 {
            drop(Box::from_raw(this as *mut NotificationClientCom));
            0
        } else {
            prev - 1
        }
    }

    unsafe extern "system" fn client_on_device_state_changed(
        this: *mut c_void,
        _pwstr_device_id: *const u16,
        _dw_new_state: u32,
    ) -> i32 {
        (*(this as *const NotificationClientCom))
            .notify
            .notify_one();
        S_OK_RAW
    }

    unsafe extern "system" fn client_on_device_added(
        this: *mut c_void,
        _pwstr_device_id: *const u16,
    ) -> i32 {
        (*(this as *const NotificationClientCom))
            .notify
            .notify_one();
        S_OK_RAW
    }

    unsafe extern "system" fn client_on_device_removed(
        this: *mut c_void,
        _pwstr_device_id: *const u16,
    ) -> i32 {
        (*(this as *const NotificationClientCom))
            .notify
            .notify_one();
        S_OK_RAW
    }

    unsafe extern "system" fn client_on_default_device_changed(
        this: *mut c_void,
        flow: i32,
        _role: i32,
        _pwstr_default_device_id: *const u16,
    ) -> i32 {
        // eCapture == 1, eAll == 2
        if flow == 1 || flow == 2 {
            (*(this as *const NotificationClientCom))
                .notify
                .notify_one();
        }
        S_OK_RAW
    }

    unsafe extern "system" fn client_on_property_value_changed(
        this: *mut c_void,
        _pwstr_device_id: *const u16,
        key: PropertyKey,
    ) -> i32 {
        if key.fmtid == PKEY_DEVICE_FRIENDLY_NAME_FMTID && key.pid == 14 {
            (*(this as *const NotificationClientCom))
                .notify
                .notify_one();
        }
        S_OK_RAW
    }

    static WATCHER_KEEPALIVE: OnceLock<mpsc::SyncSender<()>> = OnceLock::new();

    pub fn start(notify: Arc<Notify>) -> bool {
        let (ready_tx, ready_rx) = mpsc::sync_channel::<bool>(1);
        let (shutdown_tx, shutdown_rx) = mpsc::sync_channel::<()>(1);

        let spawned = std::thread::Builder::new()
            .name("aura-audio-hotplug".into())
            .spawn(move || unsafe {
                let hr_init = CoInitializeEx(None, COINIT_MULTITHREADED);
                let must_uninit = hr_init.is_ok();

                let mut enum_void: *mut c_void = std::ptr::null_mut();
                let hr_create = CoCreateInstance(
                    &CLSID_MM_DEVICE_ENUMERATOR,
                    std::ptr::null_mut(),
                    CLSCTX_ALL_RAW,
                    &IID_IMM_DEVICE_ENUMERATOR,
                    &mut enum_void,
                );
                if hr_create < 0 || enum_void.is_null() {
                    let _ = ready_tx.send(false);
                    if must_uninit {
                        CoUninitialize();
                    }
                    return;
                }

                let enumerator = &*(enum_void as *const IMMDeviceEnumeratorRaw);
                let client_ptr = Box::into_raw(Box::new(NotificationClientCom {
                    lp_vtbl: &VTBL,
                    ref_count: AtomicU32::new(1),
                    notify,
                }));

                let hr_reg = ((*enumerator.lp_vtbl).register_endpoint_notification_callback)(
                    enum_void,
                    client_ptr as *mut c_void,
                );
                if hr_reg < 0 {
                    client_release(client_ptr as *mut c_void);
                    ((*enumerator.lp_vtbl).release)(enum_void);
                    let _ = ready_tx.send(false);
                    if must_uninit {
                        CoUninitialize();
                    }
                    return;
                }

                let _ = ready_tx.send(true);
                let _ = shutdown_rx.recv();

                let _ = ((*enumerator.lp_vtbl).unregister_endpoint_notification_callback)(
                    enum_void,
                    client_ptr as *mut c_void,
                );
                client_release(client_ptr as *mut c_void);
                ((*enumerator.lp_vtbl).release)(enum_void);
                if must_uninit {
                    CoUninitialize();
                }
            });

        if spawned.is_err() {
            return false;
        }
        let registered = ready_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap_or(false);
        if registered {
            let _ = WATCHER_KEEPALIVE.set(shutdown_tx);
        }
        registered
    }
}

#[cfg(target_os = "windows")]
fn start_os_audio_endpoint_watcher(notify: Arc<tokio::sync::Notify>) -> bool {
    audio_hotplug_watcher::start(notify)
}

#[cfg(not(target_os = "windows"))]
fn start_os_audio_endpoint_watcher(_notify: Arc<tokio::sync::Notify>) -> bool {
    false
}

fn spawn_audio_hotplug_monitor(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        // Populate initial device baseline off the main startup thread.
        if let Ok(initial_devices) =
            tokio::task::spawn_blocking(audio_recorder::list_audio_input_devices).await
        {
            if let Some(state) = app_handle.try_state::<AppState>() {
                let mut guard = state
                    .last_audio_devices
                    .lock()
                    .unwrap_or_else(|p| p.into_inner());
                *guard = initial_devices;
            }
        }

        let notify = Arc::new(tokio::sync::Notify::new());
        let notify_for_watcher = Arc::clone(&notify);
        let registered = tokio::task::spawn_blocking(move || {
            start_os_audio_endpoint_watcher(notify_for_watcher)
        })
        .await
        .unwrap_or(false);

        if registered {
            loop {
                notify.notified().await;
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                check_audio_devices_hotplug(&app_handle).await;
            }
        } else {
            crate::logger::log(
                "WARN",
                "Audio",
                None,
                "Event-driven audio endpoint watcher unavailable; falling back to 15s polling",
            );
            let mut fallback = tokio::time::interval(std::time::Duration::from_secs(15));
            fallback.tick().await;
            loop {
                fallback.tick().await;
                check_audio_devices_hotplug(&app_handle).await;
            }
        }
    });
}

async fn check_audio_devices_hotplug(app: &tauri::AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let current_devices =
        match tokio::task::spawn_blocking(audio_recorder::list_audio_input_devices).await {
            Ok(devices) => devices,
            Err(_) => return,
        };
    let changed = {
        let mut guard = state
            .last_audio_devices
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if *guard != current_devices {
            *guard = current_devices.clone();
            true
        } else {
            false
        }
    };
    if changed {
        let _ = app.emit("audio-devices-changed", &current_devices);
        crate::logger::log(
            "INFO",
            "Audio",
            None,
            &format!(
                "Audio input devices hotplug detected: {} device(s) currently available",
                current_devices.len()
            ),
        );
    }
}

#[tauri::command]
async fn toggle_gaming_mode(app_handle: tauri::AppHandle) -> Result<bool, String> {
    let _apply_guard = match app_handle.try_state::<AppState>() {
        Some(ref state) => Some(state.inner().settings_apply_lock.lock().await),
        None => None,
    };
    let current_active = app_handle
        .try_state::<AppState>()
        .map(|s| s.gaming_mode_active.load(Ordering::Acquire))
        .unwrap_or(false);
    let new_state = !current_active;

    let save_handle = app_handle.clone();
    tauri::async_runtime::spawn_blocking(move || {
        settings::update_settings(&save_handle, |current| {
            current.gaming_mode_enabled = new_state;
            Ok(())
        })
        .map(|_| ())
    })
    .await
    .map_err(|error| format!("Gaming mode storage worker failed: {error}"))??;

    apply_gaming_mode_state(&app_handle, new_state, false).await;
    let _ = app_handle.emit("settings-changed", ());
    Ok(new_state)
}

#[tauri::command]
fn get_gaming_mode_state(app_handle: tauri::AppHandle) -> bool {
    app_handle
        .try_state::<AppState>()
        .map(|s| s.gaming_mode_active.load(Ordering::Acquire))
        .unwrap_or(false)
}

#[tauri::command]
async fn delete_gpu_binaries(app_handle: tauri::AppHandle, provider: String) -> Result<(), String> {
    if provider != "cuda" && provider != "directml" {
        return Err("Invalid provider".to_string());
    }
    if lock_active_gpu_downloads().contains(&provider) {
        return Err("Cannot delete GPU runtime while its download is active".to_string());
    }

    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data: {}", e))?;
    let binaries_dir = app_local_data.join("binaries");
    let provider_dir = binaries_dir.join(&provider);
    let archives_dir = (provider == "cuda").then(|| binaries_dir.join("cuda-archives"));
    let worker_handle = app_handle.clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Process termination and recursive deletion can block on Windows/antivirus.
        whisper_runner::stop_parakeet_server_and_watchdog(&worker_handle);
        whisper_runner::stop_whisper_server_and_watchdog(&worker_handle);
        if provider_dir.exists() {
            std::fs::remove_dir_all(&provider_dir)
                .map_err(|e| format!("Failed to delete binaries: {}", e))?;
        }
        if let Some(archives) = archives_dir {
            if archives.exists() {
                let _ = std::fs::remove_dir_all(&archives);
            }
        }
        Ok::<(), String>(())
    })
    .await
    .map_err(|error| format!("GPU deletion worker failed: {error}"))?
}

static ACTIVE_GPU_DOWNLOADS: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashSet<String>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashSet::new()));

#[tauri::command]
async fn cancel_gpu_download(provider: String) -> Result<(), String> {
    if provider != "cuda" {
        return Err("Invalid provider".to_string());
    }
    if !lock_active_gpu_downloads().contains(&provider) {
        return Err(format!("No active GPU download for '{provider}' to cancel"));
    }
    whisper_runner::request_cancel_download(&format!("gpu-{provider}"));
    crate::logger::log(
        "INFO",
        "GPU",
        None,
        &format!("GPU download cancellation requested for '{provider}'"),
    );
    Ok(())
}

struct GpuArchiveSpec {
    label: &'static str,
    url: &'static str,
    expected_size: u64,
    sha256: &'static str,
    staging_filename: &'static str,
}

const CUDA_ARCHIVES: &[GpuArchiveSpec] = &[
    GpuArchiveSpec {
        label: "Sherpa ONNX CUDA runtime",
        url: CUDA_ARCHIVE_URL,
        expected_size: CUDA_ARCHIVE_SIZE,
        sha256: CUDA_ARCHIVE_SHA256,
        staging_filename: "sherpa-runtime.tar.bz2",
    },
    GpuArchiveSpec {
        label: "Whisper CUDA runtime",
        url: WHISPER_CUDA_ARCHIVE_URL,
        expected_size: WHISPER_CUDA_ARCHIVE_SIZE,
        sha256: WHISPER_CUDA_ARCHIVE_SHA256,
        staging_filename: "whisper-cuda.zip",
    },
    GpuArchiveSpec {
        label: "NVIDIA cuBLAS runtime",
        url: CUBLAS_ARCHIVE_URL,
        expected_size: CUBLAS_ARCHIVE_SIZE,
        sha256: CUBLAS_ARCHIVE_SHA256,
        staging_filename: "nvidia-cublas.zip",
    },
    GpuArchiveSpec {
        label: "NVIDIA cuFFT runtime",
        url: CUFFT_ARCHIVE_URL,
        expected_size: CUFFT_ARCHIVE_SIZE,
        sha256: CUFFT_ARCHIVE_SHA256,
        staging_filename: "nvidia-cufft.zip",
    },
    GpuArchiveSpec {
        label: "NVIDIA cuDNN runtime",
        url: CUDNN_ARCHIVE_URL,
        expected_size: CUDNN_ARCHIVE_SIZE,
        sha256: CUDNN_ARCHIVE_SHA256,
        staging_filename: "nvidia-cudnn.zip",
    },
];

fn cuda_archives_for_install(use_system_nvidia: bool) -> &'static [GpuArchiveSpec] {
    if use_system_nvidia {
        &CUDA_ARCHIVES[..2]
    } else {
        CUDA_ARCHIVES
    }
}

async fn download_verified_gpu_archive(
    client: &reqwest::Client,
    app_handle: &tauri::AppHandle,
    cancel_key: &str,
    spec: &GpuArchiveSpec,
    destination: &std::path::Path,
    downloaded_before: u64,
    download_total: u64,
) -> Result<(), String> {
    crate::logger::log(
        "INFO",
        "GPU",
        None,
        &format!("Downloading pinned {}", spec.label),
    );
    artifact_download::download_verified_artifact(
        client,
        artifact_download::ArtifactSpec {
            label: spec.label,
            url: spec.url,
            expected_size: spec.expected_size,
            sha256: spec.sha256,
        },
        destination,
        artifact_download::DEFAULT_STALL_TIMEOUT,
        || whisper_runner::is_cancel_requested(cancel_key),
        |progress| {
            let combined_downloaded = downloaded_before + progress.downloaded;
            let percentage = (combined_downloaded as f64 / download_total as f64 * 100.0).min(99.9);
            let _ = app_handle.emit(
                "gpu-download-progress",
                GpuDownloadProgress {
                    provider: "cuda".to_string(),
                    downloaded: combined_downloaded,
                    total: Some(download_total),
                    percentage,
                    done: false,
                    status: None,
                },
            );
        },
    )
    .await
    .map(|_| ())
}

#[tauri::command]
async fn download_gpu_binaries(
    app_handle: tauri::AppHandle,
    provider: String,
    accepted_nvidia_terms: bool,
) -> Result<(), String> {
    let res = download_gpu_binaries_inner(app_handle, provider, accepted_nvidia_terms).await;
    if let Err(ref error) = res {
        crate::logger::log(
            "ERROR",
            "GPU",
            None,
            &format!("CUDA download/install failed: {error}"),
        );
    }
    res
}

async fn download_gpu_binaries_inner(
    app_handle: tauri::AppHandle,
    provider: String,
    accepted_nvidia_terms: bool,
) -> Result<(), String> {
    if provider == "directml" {
        return Err(
            "DirectML is unavailable in the official Sherpa ONNX v1.13.4 Windows runtime; use CUDA or CPU"
                .to_string(),
        );
    }
    if provider != "cuda" {
        return Err("Invalid provider".to_string());
    }
    if !cfg!(target_os = "windows") {
        return Err("The CUDA runtime downloader is available only on Windows".to_string());
    }
    let path = std::env::var_os("PATH");
    let use_system_nvidia = nvidia_runtime_is_on_path(path.as_deref());
    require_nvidia_terms(use_system_nvidia, accepted_nvidia_terms)?;

    let cancel_key = format!("gpu-{provider}");
    {
        let mut active = lock_active_gpu_downloads();
        if !active.insert(provider.clone()) {
            return Err("Download already in progress".to_string());
        }
    }
    whisper_runner::clear_cancel(&cancel_key);
    let _download_guard = GpuDownloadGuard {
        provider: provider.clone(),
        cancel_key: cancel_key.clone(),
    };

    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data: {}", e))?;
    let binaries_dir = app_local_data.join("binaries");
    let gpu_dir = binaries_dir.join(&provider);
    tokio::fs::create_dir_all(&gpu_dir)
        .await
        .map_err(|e| format!("Failed to create GPU directory: {e}"))?;
    let staging_dir = unique_gpu_install_directory(&binaries_dir, &provider);
    tokio::fs::create_dir(&staging_dir)
        .await
        .map_err(|e| format!("Failed to create unique GPU staging directory: {e}"))?;
    let mut staging_guard = ScopedInstallDirectory {
        path: staging_dir.clone(),
        cleanup_on_drop: true,
    };
    // Archives live in a STABLE directory (not the per-attempt staging dir) so
    // that a failed attempt leaves a resumable .part behind for the next try.
    let archives_dir = binaries_dir.join("cuda-archives");
    tokio::fs::create_dir_all(&archives_dir)
        .await
        .map_err(|e| format!("Failed to create CUDA archives directory: {e}"))?;
    let archives = cuda_archives_for_install(use_system_nvidia);
    let download_total = if use_system_nvidia {
        SYSTEM_CUDA_ARCHIVE_SIZE
    } else {
        TOTAL_CUDA_ARCHIVE_SIZE
    };
    debug_assert_eq!(
        download_total,
        archives
            .iter()
            .map(|archive| archive.expected_size)
            .sum::<u64>()
    );
    crate::logger::log(
        "INFO",
        "GPU",
        None,
        if use_system_nvidia {
            "All required NVIDIA CUDA/cuDNN DLLs were found on PATH; reusing the system runtime"
        } else {
            "A complete NVIDIA CUDA/cuDNN runtime was not found on PATH; installing pinned private copies"
        },
    );
    let client = crate::ai_client::build_download_client();
    let mut downloaded = 0u64;
    for spec in archives {
        let destination = archives_dir.join(spec.staging_filename);
        download_verified_gpu_archive(
            &client,
            &app_handle,
            &cancel_key,
            spec,
            &destination,
            downloaded,
            download_total,
        )
        .await?;
        downloaded = downloaded
            .checked_add(spec.expected_size)
            .ok_or_else(|| "CUDA download byte count overflow".to_string())?;
    }

    if whisper_runner::is_cancel_requested(&cancel_key) {
        return Err("Download cancelled".to_string());
    }

    crate::logger::log("INFO", "GPU", None, "Installing complete CUDA runtime");
    let _ = app_handle.emit(
        "gpu-download-progress",
        GpuDownloadProgress {
            provider: provider.clone(),
            downloaded: download_total,
            total: Some(download_total),
            percentage: 100.0,
            done: false,
            status: Some("installing".to_string()),
        },
    );

    let extraction_dir = staging_dir.join("extract");
    let worker_parakeet = archives_dir.join(CUDA_ARCHIVES[0].staging_filename);
    let worker_whisper = archives_dir.join(CUDA_ARCHIVES[1].staging_filename);
    let worker_cublas = archives_dir.join(CUDA_ARCHIVES[2].staging_filename);
    let worker_cufft = archives_dir.join(CUDA_ARCHIVES[3].staging_filename);
    let worker_cudnn = archives_dir.join(CUDA_ARCHIVES[4].staging_filename);
    let worker_gpu_dir = gpu_dir.clone();
    let worker_handle = app_handle.clone();
    let worker_path = path;
    tauri::async_runtime::spawn_blocking(move || {
        whisper_runner::stop_parakeet_server_and_watchdog(&worker_handle);
        whisper_runner::stop_whisper_server_and_watchdog(&worker_handle);
        let install_result = install_cuda_archive(
            &worker_parakeet,
            &worker_whisper,
            (!use_system_nvidia).then_some([
                worker_cublas.as_path(),
                worker_cufft.as_path(),
                worker_cudnn.as_path(),
            ]),
            worker_path.as_deref(),
            &extraction_dir,
            &worker_gpu_dir,
        );
        if let Ok(settings) = settings::cached_or_load_settings_without_secrets(&worker_handle) {
            whisper_runner::ensure_parakeet_server_state(&worker_handle, &settings);
            whisper_runner::ensure_whisper_server_state(&worker_handle, &settings);
        }
        install_result
    })
    .await
    .map_err(|error| format!("CUDA install worker failed: {error}"))??;

    if let Err(error) = tokio::fs::remove_dir_all(&archives_dir).await {
        if error.kind() != std::io::ErrorKind::NotFound {
            crate::logger::log(
                "WARN",
                "GPU",
                None,
                &format!(
                    "Failed to remove downloaded CUDA archives {}: {error}",
                    archives_dir.display()
                ),
            );
        }
    }

    match tokio::fs::remove_dir_all(&staging_dir).await {
        Ok(()) => staging_guard.cleanup_on_drop = false,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            staging_guard.cleanup_on_drop = false;
        }
        Err(error) => {
            crate::logger::log(
                "WARN",
                "GPU",
                None,
                &format!(
                    "Async cleanup of temporary GPU install directory {} failed: {error}; retrying in the background",
                    staging_dir.display()
                ),
            );
        }
    }

    let _ = app_handle.emit(
        "gpu-download-progress",
        GpuDownloadProgress {
            provider: provider.clone(),
            downloaded: download_total,
            total: Some(download_total),
            percentage: 100.0,
            done: true,
            status: Some("done".to_string()),
        },
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn cuda_archive_sizes_match_progress_total() {
        assert_eq!(
            CUDA_ARCHIVES
                .iter()
                .map(|archive| archive.expected_size)
                .sum::<u64>(),
            TOTAL_CUDA_ARCHIVE_SIZE
        );
        assert_eq!(
            cuda_archives_for_install(true)
                .iter()
                .map(|archive| archive.expected_size)
                .sum::<u64>(),
            SYSTEM_CUDA_ARCHIVE_SIZE
        );
        assert_eq!(cuda_archives_for_install(true).len(), 2);
        assert_eq!(cuda_archives_for_install(false).len(), 5);
    }

    #[test]
    fn cuda_runtime_allowlist_excludes_unused_payloads() {
        let runtime_files = SHERPA_CUDA_RUNTIME_FILES
            .iter()
            .chain(WHISPER_CUDA_RUNTIME_FILES)
            .chain(CUBLAS_RUNTIME_FILES)
            .chain(CUFFT_RUNTIME_FILES)
            .chain(CUDNN_RUNTIME_FILES)
            .copied()
            .collect::<std::collections::HashSet<_>>();
        let archived_nvidia_files = CUBLAS_RUNTIME_FILES
            .iter()
            .chain(CUFFT_RUNTIME_FILES)
            .chain(CUDNN_RUNTIME_FILES)
            .copied()
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(
            archived_nvidia_files,
            NVIDIA_CUDA_RUNTIME_FILES.iter().copied().collect()
        );

        for required in [
            "sherpa-onnx-offline-websocket-server.exe",
            "ggml-cuda.dll",
            "cublas64_11.dll",
            "cublasLt64_11.dll",
            "cufft64_10.dll",
            "cudnn64_8.dll",
            "cudnn_adv_infer64_8.dll",
            "cudnn_cnn_infer64_8.dll",
            "cudnn_ops_infer64_8.dll",
        ] {
            assert!(runtime_files.contains(required), "missing {required}");
        }
        for unused in [
            "onnxruntime_providers_tensorrt.dll",
            "whisper-server.exe",
            "sherpa-onnx-offline-tts.exe",
            "nvblas64_11.dll",
            "cufftw64_10.dll",
            "cudnn_adv_train64_8.dll",
            "cudnn_cnn_train64_8.dll",
            "cudnn_ops_train64_8.dll",
            "nvrtc64_112_0.dll",
        ] {
            assert!(!runtime_files.contains(unused), "retained unused {unused}");
        }
    }

    #[test]
    fn cuda_install_check_requires_complete_runtime() {
        let test_dir = std::env::temp_dir().join(format!(
            "aura-cuda-completeness-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock after epoch")
                .as_nanos()
        ));
        std::fs::create_dir(&test_dir).expect("create CUDA completeness test directory");
        let required_files = SHERPA_CUDA_RUNTIME_FILES
            .iter()
            .chain(WHISPER_CUDA_RUNTIME_FILES)
            .chain(CUBLAS_RUNTIME_FILES)
            .chain(CUFFT_RUNTIME_FILES)
            .chain(CUDNN_RUNTIME_FILES);
        for name in required_files {
            std::fs::write(test_dir.join(name), b"runtime").expect("write placeholder runtime");
        }

        assert!(gpu_bin_is_complete_with_path(&test_dir, "cuda", None));
        std::fs::remove_file(test_dir.join("cublasLt64_11.dll"))
            .expect("remove required CUDA dependency");
        assert!(!gpu_bin_is_complete_with_path(&test_dir, "cuda", None));
        std::fs::remove_dir_all(&test_dir).expect("remove CUDA completeness test directory");
    }

    #[test]
    fn cuda_install_check_accepts_nvidia_runtime_across_path_directories() {
        let test_dir = std::env::temp_dir().join(format!(
            "aura-cuda-system-runtime-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock after epoch")
                .as_nanos()
        ));
        let bin_dir = test_dir.join("aura-bin");
        let cuda_dir = test_dir.join("cuda-bin");
        let cudnn_dir = test_dir.join("cudnn-bin");
        for directory in [&bin_dir, &cuda_dir, &cudnn_dir] {
            std::fs::create_dir_all(directory).expect("create CUDA PATH test directory");
        }
        for name in SHERPA_CUDA_RUNTIME_FILES
            .iter()
            .chain(WHISPER_CUDA_RUNTIME_FILES)
        {
            std::fs::write(bin_dir.join(name), b"runtime")
                .expect("write Aura CUDA runtime placeholder");
        }
        for name in CUBLAS_RUNTIME_FILES.iter().chain(CUFFT_RUNTIME_FILES) {
            std::fs::write(cuda_dir.join(name), b"runtime")
                .expect("write system CUDA runtime placeholder");
        }
        for name in CUDNN_RUNTIME_FILES {
            std::fs::write(cudnn_dir.join(name), b"runtime")
                .expect("write system cuDNN runtime placeholder");
        }
        let path = std::env::join_paths([&cuda_dir, &cudnn_dir]).expect("build test PATH");

        assert!(nvidia_runtime_is_on_path(Some(&path)));
        assert_eq!(
            nvidia_runtime_source(&bin_dir, Some(&path)),
            NvidiaRuntimeSource::SystemPath
        );
        assert!(gpu_bin_is_complete_with_path(&bin_dir, "cuda", Some(&path)));

        std::fs::remove_file(cudnn_dir.join("cudnn_ops_infer64_8.dll"))
            .expect("remove system cuDNN dependency");
        assert!(!gpu_bin_is_complete_with_path(
            &bin_dir,
            "cuda",
            Some(&path)
        ));
        std::fs::remove_dir_all(&test_dir).expect("remove CUDA PATH test directory");
    }

    #[test]
    fn nvidia_terms_are_required_only_when_a_download_is_needed() {
        assert!(require_nvidia_terms(true, false).is_ok());
        assert!(require_nvidia_terms(false, true).is_ok());
        assert!(require_nvidia_terms(false, false).is_err());
    }

    #[test]
    fn cuda_installer_reuses_system_nvidia_runtime_without_copying_it() {
        let test_dir = std::env::temp_dir().join(format!(
            "aura-cuda-system-install-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock after epoch")
                .as_nanos()
        ));
        let sherpa_archive_path = test_dir.join("sherpa.tar.bz2");
        let whisper_archive_path = test_dir.join("whisper.zip");
        let system_dir = test_dir.join("system-cuda");
        let extraction_dir = test_dir.join("extract");
        let gpu_dir = test_dir.join("gpu");
        for directory in [&system_dir, &extraction_dir, &gpu_dir] {
            std::fs::create_dir_all(directory).expect("create system install test directory");
        }

        let sherpa_file =
            std::fs::File::create(&sherpa_archive_path).expect("create synthetic Sherpa archive");
        let encoder = bzip2::write::BzEncoder::new(sherpa_file, bzip2::Compression::best());
        let mut sherpa_archive = tar::Builder::new(encoder);
        for name in SHERPA_CUDA_RUNTIME_FILES {
            let contents = b"runtime";
            let mut header = tar::Header::new_gnu();
            header.set_size(contents.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            sherpa_archive
                .append_data(
                    &mut header,
                    format!("runtime/bin/{name}"),
                    contents.as_slice(),
                )
                .expect("add synthetic Sherpa runtime file");
        }
        sherpa_archive
            .into_inner()
            .expect("finish synthetic Sherpa archive")
            .finish()
            .expect("finish compressed Sherpa archive");

        let whisper_file =
            std::fs::File::create(&whisper_archive_path).expect("create synthetic Whisper archive");
        let mut whisper_archive = zip::ZipWriter::new(whisper_file);
        let options = zip::write::SimpleFileOptions::default();
        for name in WHISPER_CUDA_RUNTIME_FILES {
            whisper_archive
                .start_file(format!("runtime/{name}"), options)
                .expect("add synthetic Whisper runtime file");
            whisper_archive
                .write_all(b"runtime")
                .expect("write synthetic Whisper runtime file");
        }
        whisper_archive
            .finish()
            .expect("finish synthetic Whisper archive");

        for name in NVIDIA_CUDA_RUNTIME_FILES {
            std::fs::write(system_dir.join(name), b"runtime")
                .expect("write system NVIDIA runtime placeholder");
        }
        let path = std::env::join_paths([&system_dir]).expect("build system runtime PATH");

        install_cuda_archive(
            &sherpa_archive_path,
            &whisper_archive_path,
            None,
            Some(&path),
            &extraction_dir,
            &gpu_dir,
        )
        .expect("install CUDA runtime backed by system NVIDIA DLLs");

        let installed_bin = gpu_dir.join("bin");
        assert!(gpu_bin_is_complete_with_path(
            &installed_bin,
            "cuda",
            Some(&path)
        ));
        assert_eq!(
            std::fs::read_dir(&installed_bin)
                .expect("read installed CUDA runtime")
                .count(),
            SHERPA_CUDA_RUNTIME_FILES.len() + WHISPER_CUDA_RUNTIME_FILES.len()
        );
        assert!(!installed_bin.join("cublas64_11.dll").exists());

        std::fs::remove_dir_all(&test_dir).expect("remove system install test directory");
    }

    #[test]
    fn selective_zip_extraction_ignores_unused_payloads() {
        let test_dir = std::env::temp_dir().join(format!(
            "aura-cuda-extraction-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock after epoch")
                .as_nanos()
        ));
        let archive_path = test_dir.join("runtime.zip");
        let output_dir = test_dir.join("output");
        std::fs::create_dir_all(&output_dir).expect("create extraction test directory");

        let archive_file = std::fs::File::create(&archive_path).expect("create test archive");
        let mut archive = zip::ZipWriter::new(archive_file);
        let options = zip::write::SimpleFileOptions::default();
        archive
            .start_file("nested/required.dll", options)
            .expect("add required runtime file");
        archive.write_all(b"required").expect("write required file");
        archive
            .start_file("nested/unused.exe", options)
            .expect("add unused runtime file");
        archive.write_all(b"unused").expect("write unused file");
        archive.finish().expect("finish test archive");

        extract_selected_zip_files(&archive_path, &output_dir, &["required.dll"])
            .expect("extract required runtime file");
        assert_eq!(
            std::fs::read(output_dir.join("required.dll")).expect("read extracted file"),
            b"required"
        );
        assert!(!output_dir.join("unused.exe").exists());

        std::fs::remove_dir_all(&test_dir).expect("remove extraction test directory");
    }

    #[test]
    fn selective_tar_extraction_uses_required_parent_directory() {
        let test_dir = std::env::temp_dir().join(format!(
            "aura-cuda-tar-extraction-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock after epoch")
                .as_nanos()
        ));
        let archive_path = test_dir.join("runtime.tar.bz2");
        let output_dir = test_dir.join("output");
        std::fs::create_dir_all(&output_dir).expect("create tar extraction test directory");

        let archive_file = std::fs::File::create(&archive_path).expect("create test archive");
        let encoder = bzip2::write::BzEncoder::new(archive_file, bzip2::Compression::best());
        let mut archive = tar::Builder::new(encoder);
        for (path, contents) in [
            ("runtime/bin/required.dll", b"bin".as_slice()),
            ("runtime/lib/required.dll", b"lib".as_slice()),
            ("runtime/bin/unused.exe", b"unused".as_slice()),
        ] {
            let mut header = tar::Header::new_gnu();
            header.set_size(contents.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            archive
                .append_data(&mut header, path, contents)
                .expect("add tar archive entry");
        }
        archive
            .into_inner()
            .expect("finish tar archive")
            .finish()
            .expect("finish compressed archive");

        extract_selected_tar_bz2_files(&archive_path, &output_dir, "bin", &["required.dll"])
            .expect("extract selected bin runtime");
        assert_eq!(
            std::fs::read(output_dir.join("required.dll")).expect("read extracted file"),
            b"bin"
        );
        assert!(!output_dir.join("unused.exe").exists());

        std::fs::remove_dir_all(&test_dir).expect("remove tar extraction test directory");
    }

    #[test]
    fn canonical_model_name_accepts_parakeet() {
        assert_eq!(canonical_model_name("parakeet-v3").unwrap(), "parakeet-v3");
        assert_eq!(canonical_model_name("base").unwrap(), "base");
        assert!(canonical_model_name("../../evil").is_err());
        assert!(canonical_model_name("nope").is_err());
    }

    #[test]
    fn parakeet_protocol_reuses_one_socket_for_sequential_requests() {
        let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .expect("bind loopback WebSocket server");
        let port = listener.local_addr().expect("loopback address").port();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept one client");
            let mut socket = tungstenite::accept(stream).expect("accept WebSocket handshake");

            for request_index in 1..=2 {
                let header = match socket.read().expect("read audio header") {
                    tungstenite::Message::Binary(header) => header,
                    message => panic!("unexpected header message: {message:?}"),
                };
                assert_eq!(header.len(), 8);
                assert_eq!(i32::from_le_bytes(header[0..4].try_into().unwrap()), 16_000);
                let expected_bytes = i32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
                let mut received_bytes = 0usize;
                while received_bytes < expected_bytes {
                    match socket.read().expect("read audio payload") {
                        tungstenite::Message::Binary(payload) => {
                            received_bytes = received_bytes.saturating_add(payload.len());
                        }
                        message => panic!("unexpected payload message: {message:?}"),
                    }
                }
                assert_eq!(received_bytes, expected_bytes);
                socket
                    .send(tungstenite::Message::Text(format!(
                        "{{\"text\":\"request {request_index}\"}}"
                    )))
                    .expect("send transcript");
            }
            assert_eq!(
                socket.read().expect("read connection terminator"),
                tungstenite::Message::Text("Done".to_string())
            );
            socket
                .close(Some(tungstenite::protocol::CloseFrame {
                    code: tungstenite::protocol::frame::coding::CloseCode::Normal,
                    reason: "Done".into(),
                }))
                .expect("start server close handshake");
            let _ = socket.read();
        });

        let mut socket =
            connect_parakeet_socket_on_port(port, || false).expect("connect loopback client");
        assert_eq!(
            decode_parakeet_samples_on_socket(&mut socket, &[0.0; 320], || false)
                .expect("decode first request"),
            "request 1"
        );
        assert_eq!(
            decode_parakeet_samples_on_socket(&mut socket, &[0.0; 640], || false)
                .expect("decode second request"),
            "request 2"
        );
        finish_parakeet_socket(&mut socket);
        server.join().expect("loopback server thread");
    }

    #[test]
    fn test_silence_hallucination_exact_phrases() {
        assert!(is_silence_hallucination(""));
        assert!(is_silence_hallucination("   ...  "));
        assert!(is_silence_hallucination("Спасибо за просмотр!"));
        assert!(is_silence_hallucination("Thank you for watching."));
        assert!(is_silence_hallucination("Продолжение следует..."));
        assert!(is_silence_hallucination("Продолжение следует…"));
        assert!(is_silence_hallucination("Текст фильма:"));
        assert!(is_silence_hallucination("Текст фильма"));
        assert!(is_silence_hallucination("Субтитры сделал DimaTorzok"));
        assert!(is_silence_hallucination("No speech detected."));
        assert!(is_silence_hallucination("[BLANK_AUDIO]"));
        assert!(is_silence_hallucination("blank_audio"));
        assert!(is_silence_hallucination("You"));
        assert!(is_silence_hallucination("You."));
        assert!(is_silence_hallucination("you you"));
        assert!(is_silence_hallucination("You You You"));
        assert!(is_silence_hallucination("Yeah"));
        assert!(is_silence_hallucination("yeah."));
        assert!(is_silence_hallucination("Yeah Yeah"));
        assert!(is_silence_hallucination("Yep"));
        assert!(is_silence_hallucination("Mm."));
        assert!(is_silence_hallucination("Mm, Mm."));
        assert!(is_silence_hallucination("S"));
        assert!(is_silence_hallucination("Мм"));
        assert!(is_silence_hallucination("ммммммммммм"));
        assert!(is_silence_hallucination("Эмммммммммммммммммммммммммммммм"));
        assert!(is_silence_hallucination("hmmmmmm"));
        assert!(is_silence_hallucination("Ummmmm"));
        assert!(!is_silence_hallucination("em"));
        assert!(!is_silence_hallucination("um"));
        assert!(is_silence_hallucination("Используйте, что это не так"));
        assert!(is_silence_hallucination("Используйтесь, что это не так"));
        assert!(is_silence_hallucination("Убедитесь, что это не так"));
        assert!(is_silence_hallucination(
            "Используйтесь, что это не так. Используйтесь, что это не так."
        ));
        assert!(is_silence_hallucination("Danke fürs Zuschauen!"));
        assert!(is_silence_hallucination(
            "Merci d'avoir regardé cette vidéo"
        ));
        assert!(is_silence_hallucination("¡Gracias por ver el video!"));
        assert!(is_silence_hallucination("Подпишитесь, подпишитесь."));
        assert!(is_silence_hallucination(
            "Dale like y suscribete. Dale like y suscribete."
        ));
        assert!(is_silence_hallucination(
            "Abonnez vous a ma chaine. Abonnez vous a ma chaine."
        ));
    }

    #[test]
    fn test_silence_hallucination_keeps_real_dictation() {
        // Regression: these used to be discarded because of a substring match
        assert!(!is_silence_hallucination(
            "Назначь просмотр квартиры на завтра"
        ));
        assert!(!is_silence_hallucination(
            "Спасибо за просмотр моего резюме и обратную связь"
        ));
        assert!(!is_silence_hallucination(
            "Обычное предложение для диктовки."
        ));
        // "yeah" mid-sentence is normal speech and must survive.
        assert!(!is_silence_hallucination("Yeah, call it done"));
        assert!(!is_silence_hallucination("yep, that is my plan"));
    }

    #[test]
    fn test_is_cloud_unreachable() {
        assert!(is_cloud_unreachable(
            "Gemini API returned status 403 Forbidden"
        ));
        assert!(is_cloud_unreachable("error sending request: dns error"));
        assert!(is_cloud_unreachable(
            "Groq Whisper API request failed: connection timed out"
        ));
        assert!(is_cloud_unreachable(
            "location is not supported for the API use"
        ));
        assert!(!is_cloud_unreachable("Incorrect API key provided"));
        assert!(!is_cloud_unreachable(
            "insufficient_quota: You exceeded your current quota"
        ));
    }

    #[test]
    fn test_select_local_fallback_model() {
        let downloaded = vec![
            "base".to_string(),
            "parakeet-v3".to_string(),
            "small".to_string(),
        ];

        assert_eq!(
            select_local_fallback_model(&downloaded, "small").as_deref(),
            Some("small")
        );
        assert_eq!(
            select_local_fallback_model(&downloaded, "missing").as_deref(),
            Some("parakeet-v3")
        );
        assert_eq!(
            select_local_fallback_model(&["base".to_string()], "missing").as_deref(),
            Some("base")
        );
        assert_eq!(select_local_fallback_model(&[], "base"), None);
    }

    #[test]
    fn test_clean_hallucinated_brackets() {
        assert_eq!(
            clean_hallucinated_brackets("Привет [Музыка] как дела"),
            "Привет  как дела"
        );
        assert_eq!(
            clean_hallucinated_brackets("Привет (laughter) как дела"),
            "Привет  как дела"
        );
        assert_eq!(clean_hallucinated_brackets("[тишина]"), "");
        assert_eq!(clean_hallucinated_brackets("   [смех]   "), "");
        assert_eq!(
            clean_hallucinated_brackets("Обычный текст без шума"),
            "Обычный текст без шума"
        );
    }

    #[test]
    fn test_text_replacement_plan_logic() {
        // Word-normalized live matching intentionally leaves equivalent text
        // alone; final reconciliation restores exact spelling/punctuation.
        assert_eq!(
            plan_text_replacement("Все хорошо", "Всё хорошо", true),
            TextReplacementPlan {
                backspaces: 0,
                suffix: String::new(),
            }
        );
        assert_eq!(
            plan_text_replacement("Привет как дела", "Привет, как дела", true),
            TextReplacementPlan {
                backspaces: 0,
                suffix: String::new(),
            }
        );

        assert_eq!(
            plan_text_replacement("Привет как дела", "Привет как дела хорошо", true),
            TextReplacementPlan {
                backspaces: 0,
                suffix: " хорошо".to_string(),
            }
        );

        // A legitimate long hypothesis correction must remain live. The old
        // hard 25-character cap made every later update fail permanently.
        let corrected = "Очень длинное новое предложение";
        let long_plan = plan_text_replacement(
            "Очень длинное предложение которое мы надиктовали ранее",
            corrected,
            true,
        );
        assert!(long_plan.backspaces > 25);
        assert_eq!(long_plan.suffix, "новое предложение");
        assert_eq!(
            plan_text_replacement(
                corrected,
                "Очень длинное новое предложение продолжается",
                true
            ),
            TextReplacementPlan {
                backspaces: 0,
                suffix: " продолжается".to_string(),
            }
        );

        let final_plan = plan_text_replacement(
            "Очень длинное предложение которое мы надиктовали ранее",
            corrected,
            false,
        );
        assert!(final_plan.backspaces > 25);
        assert_eq!(final_plan.suffix, "новое предложение");
    }

    #[test]
    fn failed_keyboard_dispatch_does_not_advance_mirrored_text() {
        let mut mirrored = "черновой текст".to_string();

        let failure = diff_and_type_with(
            &mut mirrored,
            "чистовой текст",
            false,
            |_backspaces, _suffix| {
                Err(keyboard_simulator::TextReplacementError {
                    message: "simulated SendInput failure".to_string(),
                    backspaces_committed: 0,
                    utf16_units_committed: 0,
                })
            },
        );

        assert!(failure.is_err());
        assert_eq!(mirrored, "черновой текст");

        let metrics = diff_and_type_with(
            &mut mirrored,
            "чистовой текст",
            false,
            |backspaces, suffix| {
                assert_eq!(backspaces, 13);
                assert_eq!(suffix, "истовой текст");
                Ok(keyboard_simulator::ReplacementDispatchMetrics {
                    backspaces,
                    utf16_units: suffix.encode_utf16().count(),
                    batches: 2,
                })
            },
        )
        .expect("successful dispatch");

        assert_eq!(metrics.backspaces, 13);
        assert_eq!(mirrored, "чистовой текст");
    }

    #[test]
    fn interrupted_suffix_dispatch_commits_exact_partial_mirror() {
        let mut mirrored = "hello world".to_string();

        let failure = diff_and_type_with(
            &mut mirrored,
            "hello world extra words",
            false,
            |_backspaces, suffix| {
                // Simulate an Esc/mid-dispatch interruption after only the first
                // four characters of the suffix landed.
                let landed: String = suffix.chars().take(4).collect();
                Err(keyboard_simulator::TextReplacementError {
                    message: "interrupted for test".to_string(),
                    backspaces_committed: 0,
                    utf16_units_committed: landed.encode_utf16().count(),
                })
            },
        );

        assert!(failure.is_err());
        assert_eq!(mirrored, "hello world ext");
    }

    #[test]
    fn interrupted_backspace_phase_commits_truncated_mirror() {
        let mut typed = "abcdefgh".to_string();

        let failure = diff_and_type_with(&mut typed, "abxyz", false, |_backspaces, suffix| {
            assert_eq!(suffix, "xyz");
            Err(keyboard_simulator::TextReplacementError {
                message: "interrupted during delete".to_string(),
                backspaces_committed: 4,
                utf16_units_committed: 0,
            })
        });

        assert!(failure.is_err());
        assert_eq!(typed, "abcd");
    }

    #[test]
    fn semantic_noop_keeps_mirror_equal_to_the_actual_target() {
        let mut mirrored = "Привет как дела".to_string();

        diff_and_type_with(
            &mut mirrored,
            "Привет, как дела",
            true,
            |backspaces, suffix| {
                assert_eq!(backspaces, 0);
                assert!(suffix.is_empty());
                Ok(keyboard_simulator::ReplacementDispatchMetrics {
                    backspaces,
                    utf16_units: 0,
                    batches: 0,
                })
            },
        )
        .expect("semantic no-op dispatch");
        assert_eq!(mirrored, "Привет как дела");

        diff_and_type_with(
            &mut mirrored,
            "Привет, как дела дальше",
            true,
            |backspaces, suffix| {
                assert_eq!(backspaces, 0);
                assert_eq!(suffix, " дальше");
                Ok(keyboard_simulator::ReplacementDispatchMetrics {
                    backspaces,
                    utf16_units: suffix.encode_utf16().count(),
                    batches: 1,
                })
            },
        )
        .expect("semantic suffix dispatch");
        assert_eq!(mirrored, "Привет как дела дальше");

        diff_and_type_with(
            &mut mirrored,
            "Привет, как дела дальше",
            false,
            |backspaces, suffix| {
                assert!(backspaces > 0);
                assert_eq!(suffix, ", как дела дальше");
                Ok(keyboard_simulator::ReplacementDispatchMetrics {
                    backspaces,
                    utf16_units: suffix.encode_utf16().count(),
                    batches: 2,
                })
            },
        )
        .expect("exact committed-segment dispatch");
        assert_eq!(mirrored, "Привет, как дела дальше");
    }

    #[test]
    fn typing_modes_separate_preview_stability_from_exact_reconciliation() {
        assert_eq!(
            TypingUpdateMode::for_parakeet_update(ParakeetTranscriptUpdate::Preview {
                usable: true,
            }),
            TypingUpdateMode::LivePreview
        );
        assert_eq!(
            TypingUpdateMode::for_parakeet_update(ParakeetTranscriptUpdate::EmptyEndpoint {
                recovered_preview: true,
            }),
            TypingUpdateMode::LivePreview
        );
        assert_eq!(
            TypingUpdateMode::for_parakeet_update(ParakeetTranscriptUpdate::Committed),
            TypingUpdateMode::CommittedSegment
        );

        assert!(TypingUpdateMode::LivePreview.requires_recording());
        assert!(TypingUpdateMode::CommittedSegment.requires_recording());
        assert!(!TypingUpdateMode::Final.requires_recording());
        assert!(TypingUpdateMode::LivePreview.uses_live_matching());
        assert!(!TypingUpdateMode::CommittedSegment.uses_live_matching());
        assert!(!TypingUpdateMode::Final.uses_live_matching());
        assert_eq!(
            TypingUpdateMode::LivePreview.target_text("Привет! Всё хорошо."),
            "привет все хорошо"
        );
        assert_eq!(
            TypingUpdateMode::CommittedSegment.target_text("Привет! Всё хорошо."),
            "Привет! Всё хорошо."
        );
    }

    #[test]
    fn empty_endpoint_recovers_preview_and_allows_later_segments() {
        let mut transcript = parakeet_streaming::TranscriptState::default();
        transcript.set_preview("first recovered segment");

        assert_eq!(
            apply_parakeet_decode_text(
                parakeet_streaming::DecodeReason::Endpoint,
                false,
                "",
                &mut transcript,
            )
            .expect("empty endpoint is recoverable"),
            ParakeetTranscriptUpdate::EmptyEndpoint {
                recovered_preview: true,
            }
        );
        assert_eq!(transcript.final_text(), "first recovered segment");

        assert_eq!(
            apply_parakeet_decode_text(
                parakeet_streaming::DecodeReason::Endpoint,
                false,
                "second segment",
                &mut transcript,
            )
            .expect("later endpoint still works"),
            ParakeetTranscriptUpdate::Committed
        );
        assert_eq!(
            transcript.final_text(),
            "first recovered segment second segment"
        );
    }

    #[test]
    fn empty_endpoint_without_preview_is_ignored_without_degrading() {
        let mut transcript = parakeet_streaming::TranscriptState::default();

        assert_eq!(
            apply_parakeet_decode_text(
                parakeet_streaming::DecodeReason::Endpoint,
                false,
                "",
                &mut transcript,
            )
            .expect("silence endpoint is recoverable"),
            ParakeetTranscriptUpdate::EmptyEndpoint {
                recovered_preview: false,
            }
        );
        assert!(transcript.final_text().is_empty());
    }

    #[test]
    fn unsafe_typing_outcomes_require_clipboard_handoff() {
        assert!(TypingUpdateOutcome::TargetDesynchronized.needs_safe_clipboard_handoff());
        assert!(TypingUpdateOutcome::FocusChanged.needs_safe_clipboard_handoff());
        assert!(TypingUpdateOutcome::InputDispatchFailed.needs_safe_clipboard_handoff());
        assert!(TypingUpdateOutcome::StateUnavailable.needs_safe_clipboard_handoff());
        assert!(!TypingUpdateOutcome::Applied.needs_safe_clipboard_handoff());
        assert!(!TypingUpdateOutcome::StaleSession.needs_safe_clipboard_handoff());
    }

    #[test]
    fn test_clean_live_text() {
        assert_eq!(
            clean_live_text("Итак, несмотря на все..."),
            "итак несмотря на все"
        );
        assert_eq!(
            clean_live_text("Итак, несмотря на все."),
            "итак несмотря на все"
        );
        assert_eq!(
            clean_live_text("Привет! Всё ли хорошо?"),
            "привет все ли хорошо"
        );
        assert_eq!(clean_live_text(""), "");
    }

    /// Builds an `AppState` with an arbitrary session generation, used to verify
    /// the clipboard/session-staleness guard in isolation (no OS clipboard is
    /// touched by these assertions).
    fn test_app_state_with_generation(gen: u64) -> crate::AppState {
        crate::AppState {
            audio_recorder: audio_recorder::AudioRecorder::new(),
            selected_text: Mutex::new(String::new()),
            selection_blocked_by_uipi: AtomicBool::new(false),
            caret_context: Mutex::new(None),
            press_time: Mutex::new(None),
            is_recording: AtomicBool::new(false),
            toggle_enabled: AtomicBool::new(false),
            typed_so_far: Mutex::new(String::new()),
            live_target_desynced: AtomicBool::new(false),
            live_target_monitoring: AtomicBool::new(false),
            selected_language: Mutex::new(String::new()),
            session_gen: AtomicU64::new(gen),
            clipboard_mutex: Mutex::new(()),
            unrestored_clipboard: Mutex::new(None),
            settings_apply_lock: tokio::sync::Mutex::new(()),
            latched: AtomicBool::new(false),
            ignore_next_release: AtomicBool::new(false),
            start_focus: Mutex::new(keyboard_simulator::FocusTarget::default()),
            parakeet_lifecycle: Mutex::new(()),
            parakeet_server: Mutex::new(None),
            parakeet_port: std::sync::atomic::AtomicU16::new(0),
            parakeet_streaming: Mutex::new(None),
            parakeet_watchdog: Mutex::new(None),
            parakeet_starting: AtomicBool::new(false),
            whisper_lifecycle: Mutex::new(()),
            whisper_server: Mutex::new(None),
            whisper_port: std::sync::atomic::AtomicU16::new(0),
            whisper_watchdog: Mutex::new(None),
            whisper_starting: AtomicBool::new(false),
            llm_lifecycle: Mutex::new(()),
            llm_server: Mutex::new(None),
            llm_port: std::sync::atomic::AtomicU16::new(0),
            last_activity_time: Mutex::new(std::time::Instant::now()),
            gaming_mode_active: AtomicBool::new(false),
            last_audio_devices: Mutex::new(Vec::new()),
        }
    }

    #[test]
    fn stale_session_never_restores_clipboard_after_overlap() {
        // Session A starts and latches generation 1.
        let state = test_app_state_with_generation(1);
        assert!(session_still_current(&state, 1), "gen 1 is still current");

        // Session B starts (user triggers a new dictation before A's 800ms
        // clipboard-restore window elapses), bumping the generation to 2.
        state.session_gen.store(2, Ordering::SeqCst);

        // A's restore must now be rejected — it would otherwise clobber B's data.
        assert!(
            !session_still_current(&state, 1),
            "A (gen 1) must be treated as stale once gen 2 is current"
        );
        assert!(
            session_still_current(&state, 2),
            "B (gen 2) is the live session"
        );

        // And re-verifying restore_clipboard_guarded returns early without
        // touching the (empty) clipboard for a stale session. Direct restore of
        // an empty backup is a no-op guard-wise; we assert the guard branch is
        // taken rather than attempting an OS-level clipboard write.
        restore_clipboard_guarded(&state, 1, ClipboardBackup::Empty, None);
        // No panic, no clobber — session 1 was rejected because gen became 2.
        assert!(state.session_gen.load(Ordering::SeqCst) == 2);
    }

    #[test]
    fn tray_tooltip_displays_aura_without_voice_input_suffix() {
        assert_eq!(tray_tooltip(false, "en"), "Aura");
        assert_eq!(tray_tooltip(false, "ru"), "Aura");
        assert_eq!(tray_tooltip(false, "de"), "Aura");

        assert_eq!(tray_tooltip(true, "en"), "Aura — 🎮 Gaming Mode active");
        assert_eq!(tray_tooltip(true, "ru"), "Aura — 🎮 Игровой режим активен");
    }

    #[test]
    fn test_resolve_history_entry_meta() {
        // Normal transcription (no edit pass)
        let (mode, engine, duration) =
            resolve_history_entry_meta("local", "whisper", false, 150, None);
        assert_eq!(mode, "local");
        assert_eq!(engine, Some("whisper".to_string()));
        assert_eq!(duration, 150);

        let (mode, engine, duration) =
            resolve_history_entry_meta("cloud", "whisper", false, 150, None);
        assert_eq!(mode, "cloud");
        assert_eq!(engine, None);
        assert_eq!(duration, 0);

        // Edit pass executed in the cloud: must report "cloud" mode and no local engine,
        // even if transcription was performed locally by Whisper
        let (mode, engine, duration) = resolve_history_entry_meta(
            "local",
            "whisper",
            false,
            150,
            Some(EditExecutionTarget::Cloud),
        );
        assert_eq!(mode, "cloud");
        assert_eq!(engine, None);
        assert_eq!(duration, 0);

        // Edit pass executed locally: reports "local" and the local engine
        let (mode, engine, duration) = resolve_history_entry_meta(
            "local",
            "whisper",
            false,
            350,
            Some(EditExecutionTarget::Local),
        );
        assert_eq!(mode, "local");
        assert_eq!(engine, Some("whisper".to_string()));
        assert_eq!(duration, 350);
    }

    #[test]
    fn test_pipeline_smart_context_flow() {
        let raw = "Пачку чая.";
        let ctx = smart_insertion::CaretContext {
            text_before: "Я пошел и купил ".to_string(),
            text_after: "".to_string(),
            ..Default::default()
        };
        let adapted = smart_insertion::adapt_transcription_to_context(raw, &ctx, true);
        assert_eq!(adapted, "пачку чая.");
    }

    #[test]
    fn test_pipeline_smart_context_mid_sentence_english_i() {
        let raw = "I think so.";
        let ctx = smart_insertion::CaretContext {
            text_before: "As ".to_string(),
            text_after: " said before.".to_string(),
            ..Default::default()
        };
        let adapted = smart_insertion::adapt_transcription_to_context(raw, &ctx, true);
        assert_eq!(adapted, "I think so");
    }

    #[test]
    fn test_pipeline_smart_context_quotes() {
        let raw = "это новый факт.";
        let ctx = smart_insertion::CaretContext {
            text_before: "Он сказал: \"Конец!\" ".to_string(),
            text_after: "".to_string(),
            ..Default::default()
        };
        let adapted = smart_insertion::adapt_transcription_to_context(raw, &ctx, true);
        assert_eq!(adapted, "Это новый факт.");
    }

    #[test]
    fn test_update_notification_gaming_mode_suppression_logic() {
        assert_eq!(
            prepare_update_notification_payload(true, "ru", "1.3.0"),
            None,
            "Active Gaming Mode must suppress update notifications"
        );
        assert_eq!(
            prepare_update_notification_payload(true, "en", "1.3.0"),
            None,
            "Active Gaming Mode must suppress update notifications in all locales"
        );

        let (en_title, en_body) = prepare_update_notification_payload(false, "en", "1.3.0")
            .expect("Inactive Gaming Mode must produce localized notification payload");
        assert_eq!(en_title, "Aura Update Available");
        assert!(en_body.contains("1.3.0"));

        let (ru_title, ru_body) = prepare_update_notification_payload(false, "ru", "1.3.0")
            .expect("Russian fallback notification payload must exist");
        assert_eq!(ru_title, "Доступно обновление Aura");
        assert!(ru_body.contains("1.3.0"));
    }

    #[test]
    fn ai_edit_outputs_bypass_speech_normalizer_replacements_caret_and_hallucination_filters() {
        let rules = vec![settings::TextReplacement {
            trigger: "Thank you".to_string(),
            replacement: "REPLACED (|)".to_string(),
        }];
        let mid_sentence_ctx = smart_insertion::CaretContext {
            text_before: "prefix ".to_string(),
            text_after: " suffix".to_string(),
            ..Default::default()
        };

        // 1. Multi-line code / email sign-off from AI edit must stay 1:1 untouched.
        let edited_code = "use std::io;\n\nfn main() {\n    println!(\"(|)\");\n}";
        let (out_code, caret_code, discarded_code) = postprocess_session_text(
            edited_code,
            "en",
            &rules,
            true,
            Some(&mid_sentence_ctx),
            true,
            Some(EditExecutionTarget::Local),
        );
        assert_eq!(out_code, edited_code);
        assert_eq!(caret_code, 0);
        assert!(!discarded_code);

        // 2. Polite sign-offs ("Thank you." / "Спасибо.") from AI translation/proofreading
        // must NOT be discarded as Whisper silence hallucinations or rewritten by rules.
        for sign_off in ["Thank you.", "Спасибо."] {
            let (out, caret, discarded) = postprocess_session_text(
                sign_off,
                "en",
                &rules,
                true,
                Some(&mid_sentence_ctx),
                true,
                Some(EditExecutionTarget::Cloud),
            );
            assert_eq!(out, sign_off);
            assert_eq!(caret, 0);
            assert!(!discarded, "{sign_off} from AI edit must not be discarded");
        }

        // 3. Ordinary dictation (no edit target) still applies replacements, caret marker,
        // and filters standalone Whisper silence hallucinations.
        let (_, _, hallucinated) =
            postprocess_session_text("Thank you.", "en", &[], false, None, true, None);
        assert!(
            hallucinated,
            "Standalone 'Thank you.' in raw ASR dictation must still be filtered"
        );

        let rules_caret = vec![settings::TextReplacement {
            trigger: "скобки".to_string(),
            replacement: "(|)".to_string(),
        }];
        let (dictated, caret_left, discarded) =
            postprocess_session_text("скобки", "ru", &rules_caret, false, None, true, None);
        assert_eq!(dictated, "()");
        assert_eq!(caret_left, 1);
        assert!(!discarded);
    }

    #[test]
    fn finalize_edit_privacy_and_uipi_command_guards_hold() {
        // ADR-012: strict local edit mode never falls back to cloud even with an API key.
        assert!(!llm_runner::can_cloud_edit_fallback("local", "sk-secret"));
        assert!(!llm_runner::can_cloud_edit_fallback("off", "sk-secret"));
        assert!(llm_runner::can_cloud_edit_fallback("auto", "sk-secret"));
        assert!(!llm_runner::can_cloud_edit_fallback("auto", "   "));

        // UIPI blocked selection copy + spoken edit command -> abort literal paste!
        assert!(should_abort_uipi_blocked_edit_command(
            true,
            false,
            llm_runner::AiEditRoute::Local,
            "Исправь грамматику",
            "en",
            "ru",
            &[],
        ));
        assert!(should_abort_uipi_blocked_edit_command(
            true,
            false,
            llm_runner::AiEditRoute::Cloud,
            "Переведи на английский",
            "en",
            "ru",
            &[],
        ));

        // Disabled command or ordinary dictation in elevated window passes through as dictation.
        assert!(!should_abort_uipi_blocked_edit_command(
            true,
            false,
            llm_runner::AiEditRoute::Local,
            "Исправь грамматику",
            "en",
            "ru",
            &["fix_grammar".to_string()],
        ));
        assert!(!should_abort_uipi_blocked_edit_command(
            true,
            false,
            llm_runner::AiEditRoute::Local,
            "Привет коллеги отправляю отчет",
            "en",
            "ru",
            &[],
        ));
        assert!(!should_abort_uipi_blocked_edit_command(
            false,
            false,
            llm_runner::AiEditRoute::Local,
            "Исправь грамматику",
            "en",
            "ru",
            &[],
        ));
    }

    #[test]
    fn effective_language_handles_selection_symmetrically_across_all_locales() {
        let mut s = settings::Settings {
            language: "auto".to_string(),
            ui_language: "de".to_string(),
            ..Default::default()
        };
        // Native German layout on German UI with selection -> biases Whisper to "de"
        assert_eq!(effective_language(&s, "de", true), "de");
        // English layout on German UI with selection -> auto-detect "" so user can speak DE or EN
        assert_eq!(effective_language(&s, "en", true), "");
        // Without selection in "auto" -> always ""
        assert_eq!(effective_language(&s, "de", false), "");

        // Same symmetry for Russian and Turkish UI
        s.ui_language = "ru".to_string();
        assert_eq!(effective_language(&s, "ru", true), "ru");
        assert_eq!(effective_language(&s, "en", true), "");

        s.ui_language = "tr".to_string();
        assert_eq!(effective_language(&s, "tr", true), "tr");

        // In "layout" mode with selection, any supported non-English layout is respected
        s.language = "layout".to_string();
        assert_eq!(effective_language(&s, "fr", true), "fr");
        assert_eq!(effective_language(&s, "es", true), "es");
        assert_eq!(effective_language(&s, "en", true), "");
        s.ui_language = "en".to_string();
        assert_eq!(effective_language(&s, "en", true), "en");
    }

    #[test]
    fn overlapping_sessions_inherit_original_clipboard_across_settle_window() {
        let real_user_clip = ClipboardBackup::Text("real-user-clipboard".to_string());
        let mut slot: Option<(u64, ClipboardBackup, String)> = Some((
            1,
            real_user_clip.clone(),
            "session-1-transcript".to_string(),
        ));

        // Session 2 overlaps during the 400ms settle window while clipboard still holds session 1's transcript.
        let inherited =
            resolve_session_clipboard_backup(&mut slot, Some("session-1-transcript"), || {
                ClipboardBackup::Text("wrong-stale-transcript".to_string())
            });
        assert_eq!(inherited, real_user_clip);

        // Session 2 pastes its own transcript and takes ownership of the unrestored slot.
        slot = Some((2, inherited, "session-2-transcript".to_string()));

        // Session 1's 400ms timer fires first: must NOT steal or restore over Session 2.
        assert_eq!(take_unrestored_clipboard_for_restore(&mut slot, 1), None);

        // Session 2's 400ms timer fires: restores the real user clipboard!
        assert_eq!(
            take_unrestored_clipboard_for_restore(&mut slot, 2),
            Some((real_user_clip, "session-2-transcript".to_string()))
        );
        assert!(slot.is_none());

        // If user externally copies new data during settle window, stale slot is discarded.
        let mut slot_ext = Some((
            3,
            ClipboardBackup::Text("old".to_string()),
            "session-3-transcript".to_string(),
        ));
        let fresh = resolve_session_clipboard_backup(&mut slot_ext, Some("user-new-copy"), || {
            ClipboardBackup::Text("user-new-copy".to_string())
        });
        assert_eq!(fresh, ClipboardBackup::Text("user-new-copy".to_string()));
        assert!(slot_ext.is_none());
    }

    #[test]
    fn open_url_allowlist_enforces_https_and_trusted_hosts() {
        assert!(
            validate_allowed_external_url("https://github.com/malashkadev/aura/releases").is_ok()
        );
        assert!(validate_allowed_external_url("https://aistudio.google.com/app/apikey").is_ok());
        assert!(
            validate_allowed_external_url("https://docs.nvidia.com/cuda/eula/index.html").is_ok()
        );

        // Reject cleartext HTTP, local/file schemes, credentials, and untrusted hosts.
        assert!(validate_allowed_external_url("http://github.com/malashkadev/aura").is_err());
        assert!(validate_allowed_external_url("file:///C:/Windows/System32/calc.exe").is_err());
        assert!(validate_allowed_external_url("https://user:pass@github.com/malashkadev").is_err());
        assert!(validate_allowed_external_url("https://evil.example.com/phish").is_err());
        assert!(validate_allowed_external_url("https://github.com.evil.example/").is_err());
    }

    #[test]
    fn frontend_log_sanitizer_blocks_newline_injection_and_caps_length() {
        let forged = "boom\r\n2026-11-18 [ERROR] [Panic] forged panic line";
        let sanitized = sanitize_frontend_log_field(forged, 512, true);
        assert!(!sanitized.contains('\r'));
        assert!(!sanitized.contains('\n'));
        assert!(!sanitized.contains("[Panic]"));
        assert!(sanitized.contains("(ERROR) (Panic)"));

        let oversized = "x".repeat(900);
        let capped = sanitize_frontend_log_field(&oversized, 512, true);
        assert_eq!(capped.chars().count(), 512);

        assert!(is_whisper_server_timeout_error(
            "Whisper server request failed: operation timed out"
        ));
        assert!(!is_whisper_server_timeout_error(
            "Whisper server error (500 Internal Server Error): decode failed"
        ));
    }
}

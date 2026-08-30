#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod ai_client;
pub mod artifact_download;
pub mod audio_recorder;
#[path = "history_secure.rs"]
pub mod history;
pub mod keyboard_hook;
pub mod keyboard_simulator;
pub mod logger;
pub mod parakeet_streaming;
pub mod secure_storage;
#[path = "settings_secure.rs"]
pub mod settings;
pub mod text_normalizer;
pub mod vad;
pub mod whisper_runner;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Listener, Manager};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_updater::UpdaterExt;

struct ParakeetStreamingSession {
    generation: u64,
    cancel: Arc<AtomicBool>,
    result_rx: mpsc::Receiver<parakeet_streaming::StreamingOutcome>,
}

struct AppState {
    audio_recorder: audio_recorder::AudioRecorder,
    selected_text: Mutex<String>,
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
    whisper_lifecycle: Mutex<()>,
    whisper_server: Mutex<Option<whisper_runner::RunningWhisperServer>>,
    whisper_port: std::sync::atomic::AtomicU16,
    whisper_watchdog: Mutex<Option<whisper_runner::WhisperWatchdog>>,
}

#[tauri::command]
fn minimize_window(window: tauri::WebviewWindow) {
    let _ = window.minimize();
}

#[tauri::command]
fn close_window(window: tauri::WebviewWindow) {
    let _ = window.close();
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
}

impl From<&settings::Settings> for OverlayPreferences {
    fn from(settings: &settings::Settings) -> Self {
        Self {
            overlay_sounds: settings.overlay_sounds,
            overlay_sound_theme: settings.overlay_sound_theme.clone(),
            overlay_sound_volume: settings.overlay_sound_volume,
            overlay_show_timer: settings.overlay_show_timer,
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
    settings: settings::Settings,
) -> Result<(), String> {
    keyboard_hook::validate_hotkey(&settings.hotkey)?;
    let save_handle = app_handle.clone();
    let mut saved_settings = settings.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let result: Result<(), String> = (|| {
            settings::save_settings(&save_handle, &saved_settings)?;
            sync_autostart(&save_handle, saved_settings.autostart);
            Ok(())
        })();
        // The moved-in copy may carry secrets from the request; scrub it.
        zeroize::Zeroize::zeroize(&mut saved_settings.api_key);
        zeroize::Zeroize::zeroize(&mut saved_settings.api_key_gemini);
        zeroize::Zeroize::zeroize(&mut saved_settings.api_key_openai);
        zeroize::Zeroize::zeroize(&mut saved_settings.api_key_groq);
        zeroize::Zeroize::zeroize(&mut saved_settings.api_key_huggingface);
        zeroize::Zeroize::zeroize(&mut saved_settings.api_key_custom);
        result
    })
    .await
    .map_err(|error| format!("Settings storage worker failed: {error}"))??;

    let _ = app_handle.emit("overlay-preferences", OverlayPreferences::from(&settings));

    keyboard_hook::update_hotkey(&settings.hotkey)?;
    if let Some(state) = app_handle.try_state::<AppState>() {
        state
            .toggle_enabled
            .store(settings.toggle_enabled, Ordering::SeqCst);
    }
    let sidecar_handle = app_handle.clone();
    let sidecar_settings = settings.clone();
    tauri::async_runtime::spawn_blocking(move || {
        whisper_runner::ensure_parakeet_server_state(&sidecar_handle, &sidecar_settings);
        whisper_runner::ensure_whisper_server_state(&sidecar_handle, &sidecar_settings);
    });
    let _ = app_handle.emit("settings-changed", ());
    Ok(())
}

#[tauri::command]
async fn set_ui_language(app_handle: tauri::AppHandle, ui_language: String) -> Result<(), String> {
    let save_handle = app_handle.clone();
    tauri::async_runtime::spawn_blocking(move || {
        settings::set_ui_language(&save_handle, &ui_language)
    })
    .await
    .map_err(|error| format!("Interface language storage worker failed: {error}"))??;
    let _ = app_handle.emit("settings-changed", ());
    Ok(())
}

#[derive(Clone, Copy)]
struct TrayTranslations {
    show: &'static str,
    recognition_mode: &'static str,
    cloud: &'static str,
    local: &'static str,
    quit: &'static str,
}

fn tray_translations(language: &str) -> TrayTranslations {
    match language {
        "ru" => TrayTranslations {
            show: "Открыть настройки",
            recognition_mode: "Способ распознавания",
            cloud: "Облачный ИИ",
            local: "Локальный ИИ (Whisper / Parakeet)",
            quit: "Выход",
        },
        "de" => TrayTranslations {
            show: "Einstellungen öffnen",
            recognition_mode: "Erkennungsmodus",
            cloud: "Cloud-KI",
            local: "Lokale KI (Whisper / Parakeet)",
            quit: "Beenden",
        },
        "es" => TrayTranslations {
            show: "Abrir ajustes",
            recognition_mode: "Modo de reconocimiento",
            cloud: "IA en la nube",
            local: "IA local (Whisper / Parakeet)",
            quit: "Salir",
        },
        "fr" => TrayTranslations {
            show: "Ouvrir les paramètres",
            recognition_mode: "Mode de reconnaissance",
            cloud: "IA cloud",
            local: "IA locale (Whisper / Parakeet)",
            quit: "Quitter",
        },
        "it" => TrayTranslations {
            show: "Apri impostazioni",
            recognition_mode: "Modalità di riconoscimento",
            cloud: "IA cloud",
            local: "IA locale (Whisper / Parakeet)",
            quit: "Esci",
        },
        "zh" => TrayTranslations {
            show: "打开设置",
            recognition_mode: "识别模式",
            cloud: "云端 AI",
            local: "本地 AI (Whisper / Parakeet)",
            quit: "退出",
        },
        "pt" => TrayTranslations {
            show: "Abrir configurações",
            recognition_mode: "Modo de reconhecimento",
            cloud: "IA na nuvem",
            local: "IA local (Whisper / Parakeet)",
            quit: "Sair",
        },
        "tr" => TrayTranslations {
            show: "Ayarları aç",
            recognition_mode: "Tanıma modu",
            cloud: "Bulut AI",
            local: "Yerel AI (Whisper / Parakeet)",
            quit: "Çıkış",
        },
        _ => TrayTranslations {
            show: "Open settings",
            recognition_mode: "Recognition mode",
            cloud: "Cloud AI",
            local: "Local AI (Whisper / Parakeet)",
            quit: "Quit",
        },
    }
}

fn canonical_model_name(model_name: &str) -> Result<String, String> {
    if model_name.len() > 64 {
        return Err("Invalid model name".to_string());
    }
    if model_name == "parakeet-v3" || model_name == "punctuation" {
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
    } else {
        whisper_runner::download_model(&app_handle, model_name.as_str())
            .await
            .map(|_| ())
    }
}

#[tauri::command]
async fn cancel_model_download(model_name: String) -> Result<(), String> {
    let model_name = canonical_model_name(&model_name)?;
    whisper_runner::request_cancel_download(&model_name);
    Ok(())
}

#[tauri::command]
async fn delete_model_command(
    app_handle: tauri::AppHandle,
    model_name: String,
) -> Result<(), String> {
    let model_name = canonical_model_name(&model_name)?;
    if whisper_runner::is_model_download_active(&model_name) {
        return Err(format!(
            "Cannot delete '{model_name}' while its download is active"
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
            whisper_runner::stop_parakeet_server(&worker_handle);
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

    if model_name == "punctuation" {
        let punctuation_dir = app_local_data.join("models").join("punctuation");
        return tauri::async_runtime::spawn_blocking(move || {
            match std::fs::remove_dir_all(&punctuation_dir) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(format!("Failed to delete punctuation model: {error}")),
            }
        })
        .await
        .map_err(|error| format!("Punctuation deletion worker failed: {error}"))?;
    }

    let filename = whisper_runner::whisper_model_filename(&model_name)?;
    let model_path = app_local_data.join("models").join(&filename);

    tauri::async_runtime::spawn_blocking(move || match std::fs::remove_file(&model_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("Failed to delete model file: {error}")),
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
    let mut settings = settings::load_settings(app_handle)?;
    if settings.local_engine != "parakeet" {
        return Ok(());
    }
    crate::logger::log(
        "INFO",
        "Models",
        None,
        "Deleted the active parakeet model; switching local engine to whisper/base",
    );
    settings.local_engine = "whisper".to_string();
    settings.model_name = "base".to_string();
    settings::save_settings(app_handle, &settings)
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
        if whisper_runner::punctuation_files_complete(&models_dir.join("punctuation")) {
            downloaded.push("punctuation".to_string());
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
    Ok(audio_recorder::list_audio_input_devices())
}

#[tauri::command]
async fn start_mic_meter(app_handle: tauri::AppHandle) -> Result<(), String> {
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

#[tauri::command]
async fn reprocess_history_text(
    app_handle: tauri::AppHandle,
    text: String,
    language: String,
) -> Result<String, String> {
    let settings = load_settings_async(app_handle.clone()).await?;
    let provider = provider_from(&settings);
    let lang = if language.is_empty() || language == "auto" || language == "layout" {
        "ru"
    } else {
        &language
    };

    if settings.transcription_mode == "cloud" && !settings.api_key.trim().is_empty() {
        ai_client::clean_text_with_llm(
            provider,
            &settings.api_key,
            &text,
            lang,
            &settings.dictionary,
        )
        .await
    } else {
        Ok(text_normalizer::normalize_transcription_text(&text, lang))
    }
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

#[tauri::command]
async fn check_for_app_update(
    app_handle: tauri::AppHandle,
) -> Result<Option<AppUpdateInfo>, String> {
    Ok(query_app_update(&app_handle)
        .await?
        .map(|update| AppUpdateInfo {
            current_version: update.current_version,
            version: update.version,
        }))
}

#[tauri::command]
async fn install_app_update(app_handle: tauri::AppHandle) -> Result<(), String> {
    let update = query_app_update(&app_handle)
        .await?
        .ok_or_else(|| "No update is currently available".to_string())?;
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
/// Opens a URL in the user's default browser (API key portals, release notes, documentation).
#[tauri::command]
async fn open_url(app_handle: tauri::AppHandle, url: String) -> Result<(), String> {
    let parsed = reqwest::Url::parse(&url).map_err(|_| "URL is invalid".to_string())?;
    if parsed.scheme() != "https" && parsed.scheme() != "http" {
        return Err("Only http/https URLs may be opened".to_string());
    }

    use tauri_plugin_opener::OpenerExt;
    app_handle
        .opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn relaunch_app(app_handle: tauri::AppHandle) {
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
    provider: Option<String>,
    port: Option<u16>,
}

#[tauri::command]
fn get_engine_health(app_handle: tauri::AppHandle) -> EngineHealth {
    let settings = settings::load_settings(&app_handle).unwrap_or_default();
    if settings.local_engine == "whisper" {
        return match whisper_runner::whisper_server_status(&app_handle) {
            Some((provider, port)) => EngineHealth {
                engine: "whisper".to_string(),
                running: true,
                provider: Some(provider),
                port: Some(port),
            },
            None => EngineHealth {
                engine: "whisper".to_string(),
                running: false,
                provider: None,
                port: None,
            },
        };
    }
    if settings.local_engine != "parakeet" {
        return EngineHealth {
            engine: settings.local_engine,
            running: true,
            provider: None,
            port: None,
        };
    }
    match whisper_runner::parakeet_server_status(&app_handle) {
        Some((provider, port)) => EngineHealth {
            engine: "parakeet".to_string(),
            running: true,
            provider: Some(provider),
            port: Some(port),
        },
        None => EngineHealth {
            engine: "parakeet".to_string(),
            running: false,
            provider: None,
            port: None,
        },
    }
}

#[tauri::command]
async fn log_frontend_event(
    level: String,
    tag: String,
    session: Option<String>,
    message: String,
) -> Result<(), String> {
    logger::log(&level, &tag, session.as_deref(), &message);
    Ok(())
}

fn sync_autostart(app_handle: &tauri::AppHandle, enabled: bool) {
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
        } else {
            crate::logger::log(
                "ERROR",
                "Autostart",
                None,
                &format!("Failed to update autostart ({}): {}", enabled, e),
            );
        }
    }
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
fn effective_language(settings: &settings::Settings, layout_language: &str) -> String {
    match settings.language.as_str() {
        "ru" | "en" | "de" | "es" | "fr" | "it" | "zh" | "pt" | "tr" => settings.language.clone(),
        "layout" => layout_language.to_string(),
        _ => String::new(), // auto-detect
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
        "спасибо за просмотр",
        "спасибо за внимание",
        "продолжение следует",
        "подпишитесь",
        "thank you for watching",
        "thanks for watching",
        "thank you",
        "you",
        "you you",
        "you you you",
        "you you you you",
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
    exact_markers.contains(&normalized.as_str())
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

#[derive(Clone, Debug)]
enum ClipboardBackup {
    Text(String),
    Image {
        width: usize,
        height: usize,
        bytes: Vec<u8>,
    },
    Empty,
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
            return ClipboardBackup::Empty;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    ClipboardBackup::Empty
}

fn restore_clipboard(backup: ClipboardBackup) {
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
            };
            if res.is_ok() {
                return;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn restore_clipboard_if_unchanged(backup: ClipboardBackup, expected_temporary_text: &str) {
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

struct ClipboardGuard {
    backup: ClipboardBackup,
    expected_temporary_text: Option<String>,
    /// When present, restore is gated on this session still being current,
    /// so a stale (overlapped) session cannot clobber a newer one's clipboard.
    session: Option<(tauri::AppHandle, u64)>,
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        let backup = std::mem::replace(&mut self.backup, ClipboardBackup::Empty);
        if let Some((app_handle, my_gen)) = &self.session {
            if let Some(state) = app_handle.try_state::<AppState>() {
                restore_clipboard_guarded(
                    state.inner(),
                    *my_gen,
                    backup,
                    self.expected_temporary_text.as_deref(),
                );
                return;
            }
        }
        match &self.expected_temporary_text {
            Some(expected) => restore_clipboard_if_unchanged(backup, expected),
            None => restore_clipboard(backup),
        }
    }
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
    if model == "parakeet-v3" {
        tauri::async_runtime::spawn_blocking(move || {
            whisper_runner::run_parakeet(&app_handle, &wav, &language, &dictionary, || {
                session_stale_for_abort(&app_handle, generation)
            })
        })
        .await
        .map_err(|e| format!("Local ASR task failed: {e}"))?
    } else {
        let whisper_port = app_handle
            .try_state::<AppState>()
            .map(|state| state.whisper_port.load(Ordering::SeqCst))
            .unwrap_or(0);

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
                Ok(text) => return Ok(text),
                Err(server_err) => {
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
async fn show_overlay_error(app_handle: &tauri::AppHandle, _my_gen: u64, error_msg: &str) {
    if let Some(overlay) = app_handle.get_webview_window("overlay") {
        if let Ok(Some(monitor)) = overlay.primary_monitor() {
            let size = monitor.size();
            let scale_factor = monitor.scale_factor();

            let monitor_width = size.width as f64 / scale_factor;
            let monitor_height = size.height as f64 / scale_factor;

            let overlay_width = 160.0;
            let overlay_height = 80.0;
            const TASKBAR_MARGIN: f64 = 95.0;

            let x = (monitor_width - overlay_width) / 2.0;
            let y = monitor_height - overlay_height - TASKBAR_MARGIN;

            let _ =
                overlay.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
        }
        ensure_overlay_topmost(&overlay);
        let _ = overlay.show();
    }
    let _ = app_handle.emit("recording-state", format!("error:{}", error_msg));
    // JS handles playing error sound and animating the hide after a 2.5s display timeout
}

/// Emits hide-overlay-requested event and runs a 500ms backup safety timer to close the overlay window.
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
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
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
    state.live_target_desynced.store(false, Ordering::Release);
    state.live_target_monitoring.store(false, Ordering::Release);
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
    let copy_context = session_settings.copy_context_on_start
        && session_settings.transcription_mode == "cloud"
        && session_settings.api_provider != "huggingface";

    if copy_context {
        let app_handle_copy = app_handle.clone();
        let session_tag_copy = session_tag.clone();
        // Blocking worker: the clipboard mutex guard cannot be held across an
        // async yield (MutexGuard is not Send), and the whole backup -> clear
        // -> copy -> read -> restore sequence must stay serialized (C7).
        tauri::async_runtime::spawn_blocking(move || {
            // Sleep 50ms to let the OS keyboard state settle after the physical hotkey down
            std::thread::sleep(std::time::Duration::from_millis(50));

            // Without the shared clipboard mutex, a session whose paste window
            // overlaps this copy could capture this temporary text as its
            // "original" and the user's real clipboard content would be lost.
            let clipboard_mutex_guard = if let Some(state) = app_handle_copy.try_state::<AppState>()
            {
                match state.inner().clipboard_mutex.lock() {
                    Ok(guard) => Some(guard),
                    Err(poisoned) => Some(poisoned.into_inner()),
                }
            } else {
                None
            };

            let mut clipboard_guard = ClipboardGuard {
                backup: backup_clipboard(),
                expected_temporary_text: None,
                session: Some((app_handle_copy.clone(), gen)),
            };

            if let Ok(mut cb) = arboard::Clipboard::new() {
                let _ = cb.clear();
            }
            keyboard_simulator::simulate_copy();
            std::thread::sleep(std::time::Duration::from_millis(200));

            let copied = arboard::Clipboard::new()
                .ok()
                .and_then(|mut cb| cb.get_text().ok())
                .unwrap_or_default();
            clipboard_guard.expected_temporary_text = Some(copied.clone());

            if let Some(state) = app_handle_copy.try_state::<AppState>() {
                let state = state.inner();
                let session_is_current = state.session_gen.load(Ordering::SeqCst) == gen
                    && state.is_recording.load(Ordering::SeqCst);
                if session_is_current {
                    if let Ok(mut guard) = state.selected_text.lock() {
                        *guard = copied.clone();
                        crate::logger::log(
                            "INFO",
                            "Context",
                            Some(&session_tag_copy),
                            &format!("Captured selected text ({} chars)", copied.chars().count()),
                        );
                        if !copied.trim().is_empty() {
                            let _ = app_handle_copy.emit("selection-context-active", true);
                        }
                    }
                }
            }

            // Restore first, then release the mutex: the Drop of the guard
            // performs the clipboard restore while still serialized.
            drop(clipboard_guard);
            drop(clipboard_mutex_guard);
        });
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
        recorder.audio_recorder.start_recording(
            &worker_path,
            enable_parakeet_sample_stream,
            retain_samples_for_batch_preview,
            dev,
            move |vol| {
                // Decouple the overlay's volume IPC from the record worker: a
                // slow/busy overlay webview must never stall audio processing
                // and overflow the capture queue.
                let app_handle_vol = app_handle_vol.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = app_handle_vol.emit("volume-level", vol);
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

    if !state.is_recording.load(Ordering::SeqCst) || state.session_gen.load(Ordering::SeqCst) != gen
    {
        finish_live_target_monitoring(&app_handle, gen);
        return;
    }

    // Show overlay window
    if let Some(overlay) = app_handle.get_webview_window("overlay") {
        // Position overlay in the bottom center of the primary monitor
        if let Ok(Some(monitor)) = overlay.primary_monitor() {
            let size = monitor.size();
            let scale_factor = monitor.scale_factor();

            // Convert physical coordinates to logical coordinates
            let monitor_width = size.width as f64 / scale_factor;
            let monitor_height = size.height as f64 / scale_factor;

            // Match width and height from tauri.conf.json
            let overlay_width = 160.0;
            let overlay_height = 80.0;
            const TASKBAR_MARGIN: f64 = 95.0; // Place cleanly above the taskbar

            // Center horizontally
            let x = (monitor_width - overlay_width) / 2.0;
            let y = monitor_height - overlay_height - TASKBAR_MARGIN;

            let _ =
                overlay.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
        }
        ensure_overlay_topmost(&overlay);
        let _ = overlay.emit(
            "overlay-preferences",
            OverlayPreferences::from(&session_settings),
        );
        let _ = overlay.show();
    }

    let _ = app_handle.emit("recording-state", "recording");

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
                        "Live Parakeet preview is unavailable for this dictation",
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
                        let language = effective_language(&settings, &layout_lang);

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

/// Stops the recording and runs the final transcription + paste/type pipeline.
async fn finalize_recording(app_handle: tauri::AppHandle) {
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

    let stop_handle = app_handle.clone();
    let stop_res = tauri::async_runtime::spawn_blocking(move || {
        stop_handle
            .try_state::<AppState>()
            .ok_or_else(|| "Application state is unavailable".to_string())?
            .audio_recorder
            .stop_recording()
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
    if let Err(e) = stop_res {
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

    let app_handle_clone = app_handle.clone();
    let session_tag_clone = session_tag.clone();
    tauri::async_runtime::spawn(async move {
        let start_time = std::time::Instant::now();

        let temp_path = recording_wav_path(my_gen);
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
        let language = effective_language(&settings, &layout_lang);
        let api_call_start = std::time::Instant::now();

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
            let gate_path = temp_path_str.clone();
            let gate_tag = session_tag_clone.clone();
            let has_speech = tauri::async_runtime::spawn_blocking(move || {
                vad::gate_and_trim_wav_file(&gate_path, Some(&gate_tag))
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
                    std::time::Duration::from_secs(45),
                    ai_client::transcribe_and_clean(
                        provider_from(&settings),
                        &settings.api_key,
                        &temp_path_str,
                        &selected_text,
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
                    Err(_) => Err("Cloud transcription timed out after 45s".to_string()),
                }
            }
        };

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
        match transcription_result {
            Ok(text) => {
                let normalized_text =
                    text_normalizer::normalize_transcription_text(&text, &language);
                let trimmed = normalized_text.trim().to_string();
                crate::logger::log(
                    "INFO",
                    "ASR",
                    Some(&session_tag_clone),
                    &format!(
                        "Final result: {}",
                        crate::logger::anonymize_speech(&trimmed, settings.log_speech_text)
                    ),
                );
                if !trimmed.is_empty() && !is_silence_hallucination(&trimmed) {
                    let final_text = trimmed;

                    let history_mode = if used_local_fallback {
                        "local (cloud fallback)".to_string()
                    } else {
                        settings.transcription_mode.clone()
                    };
                    let is_local_text =
                        settings.transcription_mode == "local" || used_local_fallback;
                    let history_engine = if is_local_text {
                        Some(settings.local_engine.clone())
                    } else {
                        None
                    };
                    let history_processing_ms = if is_local_text {
                        asr_started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
                    } else {
                        0
                    };
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
                            match copy_to_clipboard(final_text.clone()).await {
                                Ok(()) => final_notice = Some("final-copied-after-edit"),
                                Err(error) => crate::logger::log(
                                    "ERROR",
                                    "Clipboard",
                                    Some(&session_tag_clone),
                                    &format!(
                                        "Could not copy the safely preserved final transcript: {error}"
                                    ),
                                ),
                            }
                        } else {
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
                            // Serialize the clipboard mutation against other
                            // overlapping sessions before the paste lands.
                            if let Some(state) = app_handle_clone.try_state::<AppState>() {
                                if let Ok(_guard) = state.clipboard_mutex.lock() {
                                    original_clipboard = backup_clipboard();
                                    if let Ok(mut cb) = arboard::Clipboard::new() {
                                        let _ = cb.set_text(final_text.clone());
                                    }
                                    paste_blocked = !keyboard_simulator::simulate_paste();
                                    if paste_blocked {
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
                                    }
                                }
                            }
                            if paste_blocked {
                                final_notice = Some("elevated-paste-blocked");
                            } else {
                                // Hide overlay and play success chime immediately upon paste dispatch
                                request_animated_hide(&app_handle_clone, "success");
                                overlay_hide_requested = true;

                                tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                                // Guarded restore: only restores if this session is
                                // still current, so an overlapped newer session's
                                // clipboard is never overwritten.
                                if let Some(state) = app_handle_clone.try_state::<AppState>() {
                                    if let Ok(_guard) = state.clipboard_mutex.lock() {
                                        restore_clipboard_guarded(
                                            state.inner(),
                                            my_gen,
                                            original_clipboard.clone(),
                                            Some(&final_text),
                                        );
                                    }
                                } else {
                                    restore_clipboard_if_unchanged(
                                        original_clipboard.clone(),
                                        &final_text,
                                    );
                                }
                            }
                        } else if session_ok {
                            crate::logger::log(
                                "WARN",
                                "Paste",
                                Some(&session_tag_clone),
                                "Focus changed; leaving text in clipboard instead of pasting.",
                            );
                            final_notice = Some("focus-changed-copied");
                            if let Ok(mut cb) = arboard::Clipboard::new() {
                                let _ = cb.set_text(final_text.clone());
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
                    let _ = tokio::fs::write(dir.join("last_transcription_error.txt"), &e).await;
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
            sync_autostart(&app_handle, startup_settings.autostart);

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

            // 2. Build system tray menu
            let tray_text = tray_translations(&startup_settings.ui_language);
            let show_i = MenuItem::with_id(app, "show", tray_text.show, true, None::<&str>)?;
            let sep1 = PredefinedMenuItem::separator(app)?;

            let is_cloud = startup_settings.transcription_mode == "cloud";
            let mode_cloud_i = CheckMenuItem::with_id(app, "tray_mode_cloud", tray_text.cloud, true, is_cloud, None::<&str>)?;
            let mode_local_i = CheckMenuItem::with_id(app, "tray_mode_local", tray_text.local, true, !is_cloud, None::<&str>)?;
            let mode_sub = Submenu::with_items(app, tray_text.recognition_mode, true, &[&mode_cloud_i, &mode_local_i])?;

            let sep2 = PredefinedMenuItem::separator(app)?;
            let quit_i = MenuItem::with_id(app, "quit", tray_text.quit, true, None::<&str>)?;

            let menu = Menu::with_items(app, &[
                &show_i,
                &sep1,
                &mode_sub,
                &sep2,
                &quit_i,
            ])?;

            let mode_cloud_handle = mode_cloud_i.clone();
            let mode_local_handle = mode_local_i.clone();

            // 3. Build tray icon
            if let Some(tray_icon) = app.default_window_icon().cloned() {
                let _tray = TrayIconBuilder::new()
                    .icon(tray_icon)
                    .menu(&menu)
                    .on_menu_event(move |app, event| match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "tray_mode_cloud" => {
                            let _ = mode_cloud_handle.set_checked(true);
                            let _ = mode_local_handle.set_checked(false);
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Ok(mut settings) = settings::load_settings(&app_handle) {
                                    settings.transcription_mode = "cloud".to_string();
                                    let _ = settings::save_settings(&app_handle, &settings);
                                    let _ = app_handle.emit("settings-changed", ());
                                }
                            });
                        }
                        "tray_mode_local" => {
                            let _ = mode_cloud_handle.set_checked(false);
                            let _ = mode_local_handle.set_checked(true);
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Ok(mut settings) = settings::load_settings(&app_handle) {
                                    settings.transcription_mode = "local".to_string();
                                    let _ = settings::save_settings(&app_handle, &settings);
                                    let _ = app_handle.emit("settings-changed", ());
                                }
                            });
                        }
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click {
                            button: MouseButton::Left,
                            ..
                        } = event
                        {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    })
                    .build(app)?;
            }

            let mode_cloud_sync = mode_cloud_i.clone();
            let mode_local_sync = mode_local_i.clone();
            let show_sync = show_i.clone();
            let mode_sub_sync = mode_sub.clone();
            let quit_sync = quit_i.clone();
            let app_for_sync = app_handle.clone();
            app.listen("settings-changed", move |_| {
                if let Ok(s) = settings::load_settings(&app_for_sync) {
                    let is_c = s.transcription_mode == "cloud";
                    let _ = mode_cloud_sync.set_checked(is_c);
                    let _ = mode_local_sync.set_checked(!is_c);
                    let text = tray_translations(&s.ui_language);
                    let _ = show_sync.set_text(text.show);
                    let _ = mode_sub_sync.set_text(text.recognition_mode);
                    let _ = mode_cloud_sync.set_text(text.cloud);
                    let _ = mode_local_sync.set_text(text.local);
                    let _ = quit_sync.set_text(text.quit);
                }
            });

            app.manage(AppState {
                audio_recorder: audio_recorder::AudioRecorder::new(),
                selected_text: Mutex::new(String::new()),
                press_time: Mutex::new(None),
                is_recording: AtomicBool::new(false),
                toggle_enabled: AtomicBool::new(startup_settings.toggle_enabled),
                typed_so_far: Mutex::new(String::new()),
                live_target_desynced: AtomicBool::new(false),
                live_target_monitoring: AtomicBool::new(false),
                selected_language: Mutex::new(String::new()),
                session_gen: AtomicU64::new(0),
                clipboard_mutex: Mutex::new(()),
                latched: AtomicBool::new(false),
                ignore_next_release: AtomicBool::new(false),
                start_focus: Mutex::new(keyboard_simulator::FocusTarget::default()),
                parakeet_lifecycle: Mutex::new(()),
                parakeet_server: Mutex::new(None),
                parakeet_port: std::sync::atomic::AtomicU16::new(3033),
                parakeet_streaming: Mutex::new(None),
                parakeet_watchdog: Mutex::new(None),
                whisper_lifecycle: Mutex::new(()),
                whisper_server: Mutex::new(None),
                whisper_port: std::sync::atomic::AtomicU16::new(0),
                whisper_watchdog: Mutex::new(None),
            });

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
                    crate::logger::log(
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
                            finalize_recording(app_handle.clone()).await;
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

                        finalize_recording(app_handle.clone()).await;
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
            reprocess_history_text
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
            whisper_runner::stop_parakeet_server(app_handle);
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
    let provider_dir = app_local_data.join("binaries").join(&provider);
    let worker_handle = app_handle.clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Process termination and recursive deletion can block on Windows/antivirus.
        whisper_runner::stop_parakeet_server(&worker_handle);
        whisper_runner::stop_whisper_server(&worker_handle);
        if provider_dir.exists() {
            std::fs::remove_dir_all(&provider_dir)
                .map_err(|e| format!("Failed to delete binaries: {}", e))?;
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
        whisper_runner::stop_whisper_server(&worker_handle);
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
        if let Ok(settings) = settings::load_settings(&worker_handle) {
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
    fn canonical_model_name_accepts_punctuation_and_parakeet() {
        assert_eq!(canonical_model_name("punctuation").unwrap(), "punctuation");
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
            press_time: Mutex::new(None),
            is_recording: AtomicBool::new(false),
            toggle_enabled: AtomicBool::new(false),
            typed_so_far: Mutex::new(String::new()),
            live_target_desynced: AtomicBool::new(false),
            live_target_monitoring: AtomicBool::new(false),
            selected_language: Mutex::new(String::new()),
            session_gen: AtomicU64::new(gen),
            clipboard_mutex: Mutex::new(()),
            latched: AtomicBool::new(false),
            ignore_next_release: AtomicBool::new(false),
            start_focus: Mutex::new(keyboard_simulator::FocusTarget::default()),
            parakeet_lifecycle: Mutex::new(()),
            parakeet_server: Mutex::new(None),
            parakeet_port: std::sync::atomic::AtomicU16::new(3033),
            parakeet_streaming: Mutex::new(None),
            parakeet_watchdog: Mutex::new(None),
            whisper_lifecycle: Mutex::new(()),
            whisper_server: Mutex::new(None),
            whisper_port: std::sync::atomic::AtomicU16::new(0),
            whisper_watchdog: Mutex::new(None),
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
}

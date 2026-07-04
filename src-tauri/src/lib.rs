#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod keyboard_hook;
pub mod audio_recorder;
pub mod ai_client;
pub mod whisper_runner;
pub mod settings;
pub mod keyboard_simulator;
pub mod history;

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tauri::{Manager, Emitter};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_autostart::ManagerExt;

/// Streaming chunks whose new audio is quieter than this RMS are skipped (VAD-lite).
const SILENCE_RMS_THRESHOLD: f32 = 0.005;

struct AppState {
    audio_recorder: audio_recorder::AudioRecorder,
    selected_text: Mutex<String>,
    press_time: Mutex<Option<std::time::Instant>>,
    is_recording: AtomicBool,
    typed_so_far: Mutex<String>,
    selected_language: Mutex<String>,
    /// Increments on every new recording session; stale async tasks compare
    /// against it before touching the keyboard or clipboard.
    session_gen: AtomicU64,
    /// Toggle mode: a short tap latched the recording until the next tap / Esc.
    latched: AtomicBool,
    /// Set when a toggle-stopping tap already finalized; its key release is a no-op.
    ignore_next_release: AtomicBool,
    /// Window that had focus when the recording started (focus guard for typing).
    start_hwnd: Mutex<isize>,
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

#[tauri::command]
async fn get_settings(app_handle: tauri::AppHandle) -> Result<settings::Settings, String> {
    settings::load_settings(&app_handle)
}

#[tauri::command]
async fn set_settings(app_handle: tauri::AppHandle, settings: settings::Settings) -> Result<(), String> {
    settings::save_settings(&app_handle, &settings)?;
    keyboard_hook::update_hotkey(&settings.hotkey);
    sync_autostart(&app_handle, settings.autostart);
    Ok(())
}

#[tauri::command]
async fn download_model_command(app_handle: tauri::AppHandle, model_name: String) -> Result<(), String> {
    whisper_runner::download_model(&app_handle, &model_name).await.map(|_| ())
}

#[tauri::command]
async fn delete_model_command(app_handle: tauri::AppHandle, model_name: String) -> Result<(), String> {
    let filename = format!("ggml-{}.bin", model_name.strip_prefix("ggml-").unwrap_or(&model_name).strip_suffix(".bin").unwrap_or(&model_name));
    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {}", e))?;
    let model_path = app_local_data.join("models").join(&filename);
    
    if model_path.exists() {
        std::fs::remove_file(&model_path)
            .map_err(|e| format!("Failed to delete model file: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
async fn get_downloaded_models(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {}", e))?;
    let models_dir = app_local_data.join("models");

    let mut downloaded = Vec::new();
    if models_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(models_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                        if filename.starts_with("ggml-") && filename.ends_with(".bin") {
                            // Extract model name between "ggml-" and ".bin"
                            let name = &filename[5..filename.len() - 4];
                            downloaded.push(name.to_string());
                        }
                    }
                }
            }
        }
    }
    Ok(downloaded)
}

#[tauri::command]
async fn get_history(app_handle: tauri::AppHandle) -> Result<Vec<history::HistoryEntry>, String> {
    history::load_history(&app_handle)
}

#[tauri::command]
async fn clear_history(app_handle: tauri::AppHandle) -> Result<(), String> {
    history::clear_history(&app_handle)
}

#[tauri::command]
async fn copy_to_clipboard(text: String) -> Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(text).map_err(|e| e.to_string())
}

fn sync_autostart(app_handle: &tauri::AppHandle, enabled: bool) {
    let manager = app_handle.autolaunch();
    let result = if enabled { manager.enable() } else { manager.disable() };
    if let Err(e) = result {
        let err_str = e.to_string();
        let is_not_found = err_str.contains("os error 2")
            || err_str.contains("not find")
            || err_str.contains("не удается найти")
            || err_str.contains("not found");

        if !enabled && is_not_found {
            eprintln!("Aura Dev Log: Autostart was already disabled.");
        } else {
            eprintln!("Aura Dev Log ERROR: Failed to update autostart ({}): {}", enabled, e);
        }
    }
}

fn recording_wav_path(gen: u64) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("aura-rec-{}-{}.wav", std::process::id(), gen))
}

fn chunk_wav_path(gen: u64) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("aura-chunk-{}-{}.wav", std::process::id(), gen))
}

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
}

fn provider_from(settings: &settings::Settings) -> ai_client::ApiProvider {
    match settings.api_provider.as_str() {
        "openai" => ai_client::ApiProvider::OpenAi,
        "groq" => ai_client::ApiProvider::Groq,
        _ => ai_client::ApiProvider::Gemini,
    }
}

/// Resolves the language to send to the recognizer from settings + detected layout.
fn effective_language(settings: &settings::Settings, layout_language: &str) -> String {
    match settings.language.as_str() {
        "ru" | "en" => settings.language.clone(),
        "layout" => layout_language.to_string(),
        _ => String::new(), // auto-detect
    }
}

fn diff_and_type(typed_so_far: &mut String, new_text: &str) {
    let typed_chars: Vec<char> = typed_so_far.chars().collect();
    let new_chars: Vec<char> = new_text.chars().collect();

    let mut common_prefix_len = 0;
    for (c1, c2) in typed_chars.iter().zip(new_chars.iter()) {
        if c1 == c2 {
            common_prefix_len += 1;
        } else {
            break;
        }
    }

    let chars_to_delete = typed_chars.len() - common_prefix_len;
    let suffix: String = new_chars[common_prefix_len..].iter().collect();

    if chars_to_delete > 0 || !suffix.is_empty() {
        keyboard_simulator::replace_text(chars_to_delete, &suffix);
    }

    *typed_so_far = new_text.to_string();
}

fn is_silence_hallucination(text: &str) -> bool {
    let t = text.trim().to_lowercase();
    if t.is_empty() {
        return true;
    }
    if t.chars().all(|c| c.is_ascii_punctuation() || c.is_whitespace()) {
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
    ];
    exact_markers.contains(&normalized.as_str())
}

/// Converts spoken punctuation commands ("запятая", "новая строка") into symbols.
/// Opt-in via settings; mainly useful for the local/raw transcription mode.
fn apply_voice_punctuation(text: &str) -> String {
    // Longer patterns first so "точка с запятой" wins over "точка".
    const RULES: &[(&[&str], &str)] = &[
        (&["с", "новой", "строки"], "\n"),
        (&["новая", "строка"], "\n"),
        (&["с", "нового", "абзаца"], "\n\n"),
        (&["новый", "абзац"], "\n\n"),
        (&["вопросительный", "знак"], "?"),
        (&["восклицательный", "знак"], "!"),
        (&["точка", "с", "запятой"], ";"),
        (&["двоеточие"], ":"),
        (&["запятая"], ","),
        (&["точка"], "."),
        (&["тире"], "—"),
        (&["открыть", "скобку"], "("),
        (&["закрыть", "скобку"], ")"),
    ];

    fn norm(token: &str) -> String {
        token
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase()
    }

    let tokens: Vec<&str> = text.split_whitespace().collect();
    let mut items: Vec<String> = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        let mut matched = false;
        for (pattern, replacement) in RULES {
            if i + pattern.len() <= tokens.len()
                && pattern.iter().enumerate().all(|(k, w)| norm(tokens[i + k]) == *w)
            {
                items.push(replacement.to_string());
                i += pattern.len();
                matched = true;
                break;
            }
        }
        if !matched {
            items.push(tokens[i].to_string());
            i += 1;
        }
    }

    let mut result = String::new();
    let mut capitalize_next = false;
    let mut glue_next = false;
    for item in &items {
        match item.as_str() {
            "," | "." | "?" | "!" | ":" | ";" | ")" => {
                result.push_str(item);
                if matches!(item.as_str(), "." | "?" | "!") {
                    capitalize_next = true;
                }
            }
            "\n" | "\n\n" => {
                result.push_str(item);
                capitalize_next = true;
            }
            "(" => {
                if !result.is_empty() && !result.ends_with('\n') {
                    result.push(' ');
                }
                result.push('(');
                glue_next = true;
            }
            "—" => {
                if !result.is_empty() && !result.ends_with('\n') {
                    result.push(' ');
                }
                result.push('—');
            }
            word => {
                if !result.is_empty() && !result.ends_with('\n') && !glue_next {
                    result.push(' ');
                }
                glue_next = false;
                if capitalize_next {
                    let mut chars = word.chars();
                    if let Some(first) = chars.next() {
                        result.extend(first.to_uppercase());
                        result.push_str(chars.as_str());
                    }
                    capitalize_next = false;
                } else {
                    result.push_str(word);
                }
            }
        }
    }
    result
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
    }
    ClipboardBackup::Empty
}

fn restore_clipboard(backup: ClipboardBackup) {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        match backup {
            ClipboardBackup::Text(text) => {
                let _ = cb.set_text(text);
            }
            ClipboardBackup::Image { width, height, bytes } => {
                let img = arboard::ImageData {
                    width,
                    height,
                    bytes: std::borrow::Cow::Owned(bytes),
                };
                let _ = cb.set_image(img);
            }
            ClipboardBackup::Empty => {
                let _ = cb.clear();
            }
        }
    }
}

struct ClipboardGuard {
    backup: ClipboardBackup,
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        let backup = std::mem::replace(&mut self.backup, ClipboardBackup::Empty);
        restore_clipboard(backup);
    }
}

/// Runs the blocking whisper.cpp sidecar off the async runtime.
async fn run_local_whisper_async(
    app_handle: tauri::AppHandle,
    model: String,
    wav: String,
    language: String,
    dictionary: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        whisper_runner::run_local_whisper(&app_handle, &model, &wav, &language, &dictionary)
    })
    .await
    .map_err(|e| format!("Local whisper task failed: {e}"))?
}

/// Types a streaming/final update via simulated keystrokes on a blocking thread.
/// Re-checks session generation (and, for live previews, the recording flag) under
/// the lock so a stale task can never corrupt a newer session's text.
async fn type_streaming_update(
    app_handle: tauri::AppHandle,
    my_gen: u64,
    new_text: String,
    require_recording: bool,
) {
    let _ = tauri::async_runtime::spawn_blocking(move || {
        let Some(state) = app_handle.try_state::<AppState>() else { return };
        let state = state.inner();

        // Focus guard: never type into a window the user switched to mid-dictation
        let start_hwnd = state.start_hwnd.lock().map(|g| *g).unwrap_or(0);
        if start_hwnd != 0 && keyboard_simulator::get_foreground_window() != start_hwnd {
            eprintln!("Aura Dev Log: Focus changed since recording started; skipping simulated typing.");
            return;
        }

        if let Ok(mut typed_guard) = state.typed_so_far.lock() {
            let session_ok = state.session_gen.load(Ordering::SeqCst) == my_gen;
            let recording_ok = !require_recording || state.is_recording.load(Ordering::SeqCst);
            if session_ok && recording_ok {
                diff_and_type(&mut typed_guard, &new_text);
            } else {
                eprintln!("Aura Dev Log: Skipping stale typing update (gen/recording check failed).");
            }
        }
    })
    .await;
}

fn categorize_error(err: &str) -> String {
    let err_lower = err.to_lowercase();
    if err_lower.contains("location is not supported")
        || err_lower.contains("user location")
        || err_lower.contains("failed_precondition")
    {
        "Gemini недоступен в вашем регионе. Включите VPN для всех приложений или выберите Groq".to_string()
    } else if err_lower.contains("api key") || err_lower.contains("invalid key") || err_lower.contains("key is invalid") || err_lower.contains("incorrect api key") || err_lower.contains("401") || err_lower.contains("permission_denied") {
        "Неверный API-ключ в настройках".to_string()
    } else if err_lower.contains("proxy") || err_lower.contains("certificate") || err_lower.contains("tls") || err_lower.contains("ssl") {
        "Ошибка соединения через VPN/прокси".to_string()
    } else if err_lower.contains("network") || err_lower.contains("timeout") || err_lower.contains("timed out") || err_lower.contains("dns") || err_lower.contains("connection") || err_lower.contains("reqwest") {
        let clean_err = err.replace("Gemini API request failed: ", "").replace("reqwest::Error", "");
        let truncated = if clean_err.chars().count() > 50 {
            clean_err.chars().take(47).collect::<String>() + "..."
        } else {
            clean_err
        };
        format!("Нет сети: {}", truncated)
    } else if err_lower.contains("ggml") || err_lower.contains("model file") || err_lower.contains("not found") {
        "Локальная модель не скачана".to_string()
    } else if err_lower.contains("whisper-sidecar") || err_lower.contains("sidecar") {
        "Сбой локального Whisper-клиента".to_string()
    } else if err_lower.contains("rate limit") || err_lower.contains("429") {
        "Лимит запросов API исчерпан".to_string()
    } else if err_lower.contains("quota") || err_lower.contains("insufficient balance") {
        "Баланс API ключа исчерпан".to_string()
    } else {
        if err.chars().count() > 40 {
            err.chars().take(37).collect::<String>() + "..."
        } else {
            err.to_string()
        }
    }
}

/// Shows the error state in the overlay for a moment, then hides it
/// (unless a newer session already owns the overlay).
async fn show_overlay_error(app_handle: &tauri::AppHandle, _my_gen: u64, error_msg: &str) {
    let _ = app_handle.emit("recording-state", format!("error:{}", error_msg));
    // JS handles playing error sound and animating the hide after a 2.5s display timeout
}

/// Emits hide-overlay-requested event and runs a 500ms backup safety timer to close the overlay window.
fn request_animated_hide(app_handle: &tauri::AppHandle, status: &str) {
    if let Some(overlay) = app_handle.get_webview_window("overlay") {
        let _ = app_handle.emit("hide-overlay-requested", serde_json::json!({ "status": status }));
        let overlay_clone = overlay.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            if let Ok(true) = overlay_clone.is_visible() {
                let _ = overlay_clone.hide();
            }
        });
    }
}

/// Stops and discards the current recording (accidental tap or Esc cancel).
fn discard_recording(app_handle: &tauri::AppHandle) {
    if let Some(state) = app_handle.try_state::<AppState>() {
        let state = state.inner();
        state.is_recording.store(false, Ordering::SeqCst);
        state.latched.store(false, Ordering::SeqCst);
        let _ = state.audio_recorder.cancel_recording();
    }
    keyboard_hook::set_recording_active(false);
    request_animated_hide(app_handle, "cancel");
}

/// Esc pressed during recording: discard audio and erase any streamed preview text.
async fn cancel_recording(app_handle: tauri::AppHandle) {
    eprintln!("Aura Dev Log: Esc pressed — cancelling recording session");
    let my_gen = if let Some(state) = app_handle.try_state::<AppState>() {
        state.session_gen.load(Ordering::SeqCst)
    } else {
        return;
    };
    discard_recording(&app_handle);

    // Erase live-preview text that was already typed, if any
    let has_typed = app_handle
        .try_state::<AppState>()
        .and_then(|s| s.typed_so_far.lock().ok().map(|g| !g.is_empty()))
        .unwrap_or(false);
    if has_typed {
        type_streaming_update(app_handle.clone(), my_gen, String::new(), false).await;
    }
}

/// Starts a new recording session: registers a new generation, captures context
/// (focus window, keyboard layout, selected text), starts audio capture, shows the
/// overlay, and spawns the live-streaming loop when enabled.
async fn start_recording_session(app_handle: tauri::AppHandle) {
    eprintln!("Aura Dev Log: Hotkey pressed — starting recording session");
    let Some(state) = app_handle.try_state::<AppState>() else { return };
    let state = state.inner();

    // A new generation invalidates every task left over from previous sessions
    let gen = state.session_gen.fetch_add(1, Ordering::SeqCst) + 1;

    // Remember the focused window for the typing focus guard
    let hwnd = keyboard_simulator::get_foreground_window();
    if let Ok(mut guard) = state.start_hwnd.lock() {
        *guard = hwnd;
    }

    // Detect active keyboard language at the moment of press
    let lang = keyboard_simulator::get_active_layout_language();
    eprintln!("Aura Dev Log: Active layout language = {}", lang);
    if let Ok(mut guard) = state.selected_language.lock() {
        *guard = lang;
    }

    if let Ok(mut guard) = state.typed_so_far.lock() {
        *guard = String::new();
    }
    if let Ok(mut guard) = state.selected_text.lock() {
        *guard = String::new();
    }
    state.latched.store(false, Ordering::SeqCst);
    state.ignore_next_release.store(false, Ordering::SeqCst);
    state.is_recording.store(true, Ordering::SeqCst);
    keyboard_hook::set_recording_active(true);

    if let Ok(mut guard) = state.press_time.lock() {
        *guard = Some(std::time::Instant::now());
    }

    // Copy selected text in a background task. The clipboard is cleared first so
    // stale clipboard contents are never mistaken for a selection (and never leak
    // into API prompts).
    let app_handle_copy = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let _guard = ClipboardGuard {
            backup: backup_clipboard(),
        };

        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.clear();
        }
        keyboard_simulator::simulate_copy();
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        let copied = arboard::Clipboard::new()
            .ok()
            .and_then(|mut cb| cb.get_text().ok())
            .unwrap_or_default();

        if let Some(state) = app_handle_copy.try_state::<AppState>() {
            if let Ok(mut guard) = state.inner().selected_text.lock() {
                *guard = copied.clone();
                eprintln!("Aura Dev Log: Captured selected text ({} chars)", copied.chars().count());
            }
        }
    });

    // Start recording to a session-unique temporary WAV path
    let temp_path = recording_wav_path(gen);
    let temp_path_str = temp_path.to_string_lossy().to_string();
    eprintln!("Aura Dev Log: Starting audio recording to {}", temp_path_str);

    let app_handle_vol = app_handle.clone();
    if let Err(e) = state.audio_recorder.start_recording(&temp_path_str, move |vol| {
        let _ = app_handle_vol.emit("volume-level", vol);
    }) {
        eprintln!("Aura Dev Log ERROR: Failed to start recording: {}", e);
        state.is_recording.store(false, Ordering::SeqCst);
        keyboard_hook::set_recording_active(false);
        show_overlay_error(&app_handle, gen, "Ошибка запуска микрофона").await;
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

            let _ = overlay.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
        }
        let _ = overlay.set_always_on_top(true);
        let _ = overlay.show();
    }

    let _ = app_handle.emit("recording-state", "recording");

    // Spawn background streaming loop if enabled in settings
    let streaming_enabled = settings::load_settings(&app_handle)
        .map(|s| s.streaming_enabled)
        .unwrap_or(false);
    if streaming_enabled {
        let app_handle_loop = app_handle.clone();
        let my_gen = gen;
        tauri::async_runtime::spawn(async move {
            eprintln!("Aura Dev Log: Spawning background streaming loop task...");
            // Wait to gather initial audio and stay under Groq rate limits
            tokio::time::sleep(std::time::Duration::from_millis(4000)).await;

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
                    eprintln!("Aura Dev Log: Streaming session ended. Exiting streaming loop.");
                    break;
                }

                let mut sleep_ms: u64 = 4000;
                let Some(state) = app_handle_loop.try_state::<AppState>() else { break };
                let state = state.inner();
                if let Ok((samples, sample_rate, channels)) = state.audio_recorder.get_recorded_samples() {
                    // VAD-lite: skip the API call when nothing new was said in this chunk
                    let new_start = last_len.min(samples.len());
                    let has_new_speech = rms(&samples[new_start..]) > SILENCE_RMS_THRESHOLD;

                    // Ensure we have at least 0.5s of audio (8000 samples at 16kHz)
                    if samples.len() > 8000 && has_new_speech {
                        last_len = samples.len();
                        let write_res = audio_recorder::process_and_write_wav(
                            &samples, channels, sample_rate, &chunk_path_str,
                        );
                        if write_res.is_ok() {
                            if let Ok(settings) = settings::load_settings(&app_handle_loop) {
                                let layout_lang = state
                                    .selected_language
                                    .lock()
                                    .map(|g| g.clone())
                                    .unwrap_or_default();
                                let language = effective_language(&settings, &layout_lang);

                                // Fast verbatim preview (clean=false); the final pass does the cleanup
                                let transcription_result = if settings.transcription_mode == "local" {
                                    run_local_whisper_async(
                                        app_handle_loop.clone(),
                                        settings.model_name.clone(),
                                        chunk_path_str.clone(),
                                        language.clone(),
                                        settings.dictionary.clone(),
                                    )
                                    .await
                                } else {
                                    ai_client::transcribe_and_clean(
                                        provider_from(&settings),
                                        &settings.api_key,
                                        &chunk_path_str,
                                        "", // No selected text during live streaming
                                        &language,
                                        &settings.dictionary,
                                        false,
                                    )
                                    .await
                                };

                                match transcription_result {
                                    Ok(text) => {
                                        let trimmed = text.trim().to_string();
                                        eprintln!("Aura Dev Log: Streaming transcription success: '{}'", trimmed);
                                        if !trimmed.is_empty() && !is_silence_hallucination(&trimmed) {
                                            type_streaming_update(
                                                app_handle_loop.clone(),
                                                my_gen,
                                                trimmed,
                                                true,
                                            )
                                            .await;
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Aura Dev Log ERROR: Streaming transcription failed: {}", e);
                                    }
                                }
                            }
                        }
                    }

                    // Adaptive interval: the full recording is re-sent each chunk, so
                    // slow down as it grows to keep traffic and rate limits sane.
                    let recorded_secs =
                        samples.len() as f64 / (sample_rate.max(1) as f64 * channels.max(1) as f64);
                    sleep_ms = if recorded_secs < 60.0 {
                        4000
                    } else if recorded_secs < 150.0 {
                        8000
                    } else {
                        12000
                    };
                }

                tokio::time::sleep(std::time::Duration::from_millis(sleep_ms)).await;
            }

            let _ = std::fs::remove_file(&chunk_path);
        });
    }
}

/// Stops the recording and runs the final transcription + paste/type pipeline.
async fn finalize_recording(app_handle: tauri::AppHandle) {
    let Some(state) = app_handle.try_state::<AppState>() else { return };
    let state = state.inner();

    let my_gen = state.session_gen.load(Ordering::SeqCst);
    state.is_recording.store(false, Ordering::SeqCst);
    state.latched.store(false, Ordering::SeqCst);
    keyboard_hook::set_recording_active(false);

    let _ = app_handle.emit("recording-state", "processing");

    let stop_res = state.audio_recorder.stop_recording();
    eprintln!("Aura Dev Log: stop_recording result = {:?}", stop_res);
    if let Err(e) = stop_res {
        eprintln!("Aura Dev Log ERROR: Failed to stop recording: {}", e);
        show_overlay_error(&app_handle, my_gen, "Ошибка остановки записи").await;
        return;
    }

    // Perform final transcription in a background task
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let start_time = std::time::Instant::now();

        let temp_path = recording_wav_path(my_gen);
        let temp_path_str = temp_path.to_string_lossy().to_string();

        // Load settings
        let settings = match settings::load_settings(&app_handle_clone) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Aura Dev Log ERROR: Failed to load settings: {}", e);
                show_overlay_error(&app_handle_clone, my_gen, "Ошибка загрузки настроек").await;
                return;
            }
        };

        let (layout_lang, selected_text) = if let Some(state) = app_handle_clone.try_state::<AppState>() {
            let state = state.inner();
            let lang = state.selected_language.lock().map(|g| g.clone()).unwrap_or_default();
            let text = state.selected_text.lock().map(|g| g.clone()).unwrap_or_default();
            (lang, text)
        } else {
            (String::new(), String::new())
        };
        let language = effective_language(&settings, &layout_lang);

        eprintln!("Aura Dev Log: Calling final transcription...");
        let api_call_start = std::time::Instant::now();
        let transcription_result = if settings.transcription_mode == "local" {
            run_local_whisper_async(
                app_handle_clone.clone(),
                settings.model_name.clone(),
                temp_path_str.clone(),
                language.clone(),
                settings.dictionary.clone(),
            )
            .await
        } else {
            if settings.api_key.trim().is_empty() {
                Err("Введите API-ключ в настройках программы".to_string())
            } else {
                ai_client::transcribe_and_clean(
                    provider_from(&settings),
                    &settings.api_key,
                    &temp_path_str,
                    &selected_text,
                    &language,
                    &settings.dictionary,
                    true,
                )
                .await
            }
        };
        eprintln!("Aura Dev Log: Transcription call duration = {} ms", api_call_start.elapsed().as_millis());

        // The temporary WAV is no longer needed
        let _ = std::fs::remove_file(&temp_path);

        let mut had_error = None;
        match transcription_result {
            Ok(text) => {
                let trimmed = text.trim().to_string();
                eprintln!("Aura Dev Log: Final transcription success: '{}'", trimmed);
                if !trimmed.is_empty() && !is_silence_hallucination(&trimmed) {
                    let final_text = if settings.voice_punctuation {
                        apply_voice_punctuation(&trimmed)
                    } else {
                        trimmed
                    };

                    // Save to history and notify the settings UI
                    match history::add_entry(&app_handle_clone, &final_text, &settings.transcription_mode) {
                        Ok(()) => {
                            let _ = app_handle_clone.emit("history-updated", ());
                        }
                        Err(e) => eprintln!("Aura Dev Log ERROR: Failed to save history: {}", e),
                    }

                    let paste_start = std::time::Instant::now();
                    if settings.streaming_enabled {
                        // Smart diff replacement of the live preview with the final text
                        type_streaming_update(app_handle_clone.clone(), my_gen, final_text, false).await;
                    } else {
                        // Classic mode: instantaneous clipboard paste
                        let session_ok = app_handle_clone
                            .try_state::<AppState>()
                            .map(|s| s.session_gen.load(Ordering::SeqCst) == my_gen)
                            .unwrap_or(false);
                        let start_hwnd = app_handle_clone
                            .try_state::<AppState>()
                            .and_then(|s| s.start_hwnd.lock().ok().map(|g| *g))
                            .unwrap_or(0);
                        let focus_ok = start_hwnd == 0
                            || keyboard_simulator::get_foreground_window() == start_hwnd;

                        if session_ok && focus_ok {
                            let original_clipboard = backup_clipboard();
                            if let Ok(mut cb) = arboard::Clipboard::new() {
                                let _ = cb.set_text(final_text.clone());
                            }
                            keyboard_simulator::simulate_paste();
                            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                            restore_clipboard(original_clipboard);
                        } else {
                            // Don't paste into the wrong window/session; leave the text in
                            // the clipboard so the user can paste it manually (also in history).
                            eprintln!("Aura Dev Log: Focus/session changed; leaving text in clipboard instead of pasting.");
                            if let Ok(mut cb) = arboard::Clipboard::new() {
                                let _ = cb.set_text(final_text.clone());
                            }
                        }
                    }
                    eprintln!("Aura Dev Log: Paste duration = {} ms", paste_start.elapsed().as_millis());
                }
            }
            Err(e) => {
                eprintln!("Aura Dev Log ERROR: Final transcription failed: {}", e);
                had_error = Some(categorize_error(&e));
            }
        }
        eprintln!("Aura Dev Log: Total processing duration from release = {} ms", start_time.elapsed().as_millis());

        if let Some(msg) = had_error {
            show_overlay_error(&app_handle_clone, my_gen, &msg).await;
        } else {
            // Hide overlay window via animated request
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
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // A second instance just focuses the settings window of the first one
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Load settings, apply configured hotkey and autostart state on startup
            if let Ok(settings) = settings::load_settings(&app_handle) {
                keyboard_hook::update_hotkey(&settings.hotkey);
                sync_autostart(&app_handle, settings.autostart);
            }

            // 1. Intercept CloseRequested on main window to hide it instead of closing the app
            if let Some(main_window) = app.get_webview_window("main") {
                let main_window_clone = main_window.clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = main_window_clone.hide();
                    }
                });
            }

            // 2. Build system tray menu
            let quit_i = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Открыть настройки", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            // 3. Build tray icon
            if let Some(tray_icon) = app.default_window_icon().cloned() {
                let _tray = TrayIconBuilder::new()
                    .icon(tray_icon)
                    .menu(&menu)
                    .on_menu_event(|app, event| {
                        match event.id.as_ref() {
                            "quit" => {
                                app.exit(0);
                            }
                            "show" => {
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                            _ => {}
                        }
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                            }
                        }
                    })
                    .build(app)?;
            }

            app.manage(AppState {
                audio_recorder: audio_recorder::AudioRecorder::new(),
                selected_text: Mutex::new(String::new()),
                press_time: Mutex::new(None),
                is_recording: AtomicBool::new(false),
                typed_so_far: Mutex::new(String::new()),
                selected_language: Mutex::new(String::new()),
                session_gen: AtomicU64::new(0),
                latched: AtomicBool::new(false),
                ignore_next_release: AtomicBool::new(false),
                start_hwnd: Mutex::new(0),
            });

            // Esc cancels an active recording
            let cancel_handle = app_handle.clone();
            keyboard_hook::set_cancel_callback(move || {
                let app_handle = cancel_handle.clone();
                tauri::async_runtime::spawn(async move {
                    cancel_recording(app_handle).await;
                });
            })
            .expect("Failed to set cancel callback");

            // Start global keyboard hook
            keyboard_hook::start_hook(move |is_down| {
                let app_handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    let Some(state) = app_handle.try_state::<AppState>() else { return };

                    if is_down {
                        eprintln!("Aura Dev Log: Hotkey down");
                        let recording = state.is_recording.load(Ordering::SeqCst);
                        if recording && state.latched.load(Ordering::SeqCst) {
                            // Second tap in toggle mode stops the recording
                            state.latched.store(false, Ordering::SeqCst);
                            state.ignore_next_release.store(true, Ordering::SeqCst);
                            finalize_recording(app_handle.clone()).await;
                        } else if !recording {
                            start_recording_session(app_handle.clone()).await;
                        }
                    } else {
                        eprintln!("Aura Dev Log: Hotkey up");
                        keyboard_simulator::send_dummy_key();

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
                            eprintln!("Aura Dev Log: Press duration = {} ms", d.as_millis());
                            if d.as_millis() < 300 {
                                let toggle_enabled = settings::load_settings(&app_handle)
                                    .map(|s| s.toggle_enabled)
                                    .unwrap_or(false);
                                if toggle_enabled {
                                    // Short tap latches the recording until the next tap or Esc
                                    eprintln!("Aura Dev Log: Short tap — latching recording (toggle mode).");
                                    state.latched.store(true, Ordering::SeqCst);
                                } else {
                                    eprintln!("Aura Dev Log: Press too short (< 300ms), discarding.");
                                    discard_recording(&app_handle);
                                }
                                return;
                            }
                        }

                        finalize_recording(app_handle.clone()).await;
                    }
                });
            }).expect("Failed to start global Win32 keyboard hook");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            download_model_command,
            delete_model_command,
            get_downloaded_models,
            get_history,
            clear_history,
            copy_to_clipboard,
            minimize_window,
            close_window,
            start_dragging_command,
            hide_overlay_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silence_hallucination_exact_phrases() {
        assert!(is_silence_hallucination(""));
        assert!(is_silence_hallucination("   ...  "));
        assert!(is_silence_hallucination("Спасибо за просмотр!"));
        assert!(is_silence_hallucination("Thank you for watching."));
        assert!(is_silence_hallucination("Продолжение следует..."));
        assert!(is_silence_hallucination("Субтитры сделал DimaTorzok"));
        assert!(is_silence_hallucination("No speech detected."));
        assert!(is_silence_hallucination("[BLANK_AUDIO]"));
        assert!(is_silence_hallucination("blank_audio"));
        assert!(is_silence_hallucination("You"));
        assert!(is_silence_hallucination("You."));
        assert!(is_silence_hallucination("you you"));
        assert!(is_silence_hallucination("You You You"));
    }

    #[test]
    fn test_silence_hallucination_keeps_real_dictation() {
        // Regression: these used to be discarded because of a substring match
        assert!(!is_silence_hallucination("Назначь просмотр квартиры на завтра"));
        assert!(!is_silence_hallucination("Спасибо за просмотр моего резюме и обратную связь"));
        assert!(!is_silence_hallucination("Обычное предложение для диктовки."));
    }

    #[test]
    fn test_voice_punctuation_basic() {
        assert_eq!(
            apply_voice_punctuation("привет запятая как дела вопросительный знак"),
            "привет, как дела?"
        );
        assert_eq!(
            apply_voice_punctuation("это тест точка новое предложение"),
            "это тест. Новое предложение"
        );
        assert_eq!(
            apply_voice_punctuation("первая строка новая строка вторая строка"),
            "первая строка\nВторая строка"
        );
    }

    #[test]
    fn test_voice_punctuation_no_commands() {
        assert_eq!(
            apply_voice_punctuation("просто обычный текст без команд"),
            "просто обычный текст без команд"
        );
        // Words that merely contain command stems must not trigger
        assert_eq!(
            apply_voice_punctuation("мы дошли до этой точки маршрута"),
            "мы дошли до этой точки маршрута"
        );
    }

    #[test]
    fn test_voice_punctuation_multiword_priority() {
        assert_eq!(
            apply_voice_punctuation("список точка с запятой продолжение"),
            "список; продолжение"
        );
    }

    #[test]
    fn test_rms() {
        assert_eq!(rms(&[]), 0.0);
        assert!(rms(&[0.0, 0.0, 0.0]) < f32::EPSILON);
        assert!((rms(&[0.5, -0.5, 0.5, -0.5]) - 0.5).abs() < 1e-6);
    }
}

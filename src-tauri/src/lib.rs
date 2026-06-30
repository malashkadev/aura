pub mod keyboard_hook;
pub mod audio_recorder;
pub mod ai_client;
pub mod whisper_runner;
pub mod settings;
pub mod keyboard_simulator;

use std::sync::Mutex;
use tauri::{Manager, Emitter};

struct AppState {
    audio_recorder: audio_recorder::AudioRecorder,
    selected_text: Mutex<String>,
    press_time: Mutex<Option<std::time::Instant>>,
}

#[tauri::command]
async fn get_settings(app_handle: tauri::AppHandle) -> Result<settings::Settings, String> {
    settings::load_settings(&app_handle)
}

#[tauri::command]
async fn set_settings(app_handle: tauri::AppHandle, settings: settings::Settings) -> Result<(), String> {
    settings::save_settings(&app_handle, &settings)?;
    keyboard_hook::update_hotkey(&settings.hotkey);
    Ok(())
}

#[tauri::command]
async fn download_model_command(app_handle: tauri::AppHandle, model_name: String) -> Result<(), String> {
    whisper_runner::download_model(&app_handle, &model_name).await.map(|_| ())
}

fn is_silence_hallucination(text: &str) -> bool {
    let t = text.trim().to_lowercase();
    if t.is_empty() {
        return true;
    }
    let markers = [
        "no audio to transcribe",
        "no speech",
        "no audio detected",
        "no speech detected",
        "there is no audio",
        "thank you for watching",
        "thanks for watching",
        "subtitles by",
        "amara.org",
        "подпишитесь",
        "спасибо за просмотр",
        "просмотр",
    ];
    for marker in &markers {
        if t.contains(marker) {
            return true;
        }
    }
    if t.chars().all(|c| c.is_ascii_punctuation() || c.is_whitespace()) {
        return true;
    }
    false
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Load settings and apply configured hotkey on startup
            if let Ok(settings) = settings::load_settings(&app_handle) {
                keyboard_hook::update_hotkey(&settings.hotkey);
            }

            // Manage global state for audio recording and context text
            app.manage(AppState {
                audio_recorder: audio_recorder::AudioRecorder::new(),
                selected_text: Mutex::new(String::new()),
                press_time: Mutex::new(None),
            });

            // Start global keyboard hook
            keyboard_hook::start_hook(move |is_down| {
                let app_handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    if is_down {
                        // Record press time
                        if let Some(state) = app_handle.try_state::<AppState>() {
                            let state = state.inner();
                            if let Ok(mut guard) = state.press_time.lock() {
                                *guard = Some(std::time::Instant::now());
                            }
                        }

                        // 1. Read clipboard BEFORE copy to compare later
                        let original_clip = if let Ok(mut cb) = arboard::Clipboard::new() {
                            cb.get_text().unwrap_or_default()
                        } else {
                            String::new()
                        };

                        // 2. Simulate Copy to capture selection
                        keyboard_simulator::simulate_copy();

                        // 3. Sleep 120ms to allow system clipboard update
                        tokio::time::sleep(std::time::Duration::from_millis(120)).await;

                        // 4. Read clipboard AFTER copy
                        let new_clip = if let Ok(mut cb) = arboard::Clipboard::new() {
                            cb.get_text().unwrap_or_default()
                        } else {
                            String::new()
                        };

                        // If clipboard did not change, assume no text was selected
                        let selected_text = if new_clip == original_clip {
                            String::new()
                        } else {
                            new_clip
                        };

                        // Store selected text in state
                        if let Some(state) = app_handle.try_state::<AppState>() {
                            let state = state.inner();
                            if let Ok(mut guard) = state.selected_text.lock() {
                                *guard = selected_text;
                            }
                        }

                        // 5. Start recording to temporary WAV path
                        let temp_path = std::env::temp_dir().join("aura-temp.wav");
                        let temp_path_str = temp_path.to_string_lossy().to_string();

                        let app_handle_clone = app_handle.clone();
                        if let Some(state) = app_handle.try_state::<AppState>() {
                            let state = state.inner();
                            if let Err(e) = state.audio_recorder.start_recording(&temp_path_str, move |vol| {
                                let _ = app_handle_clone.emit("volume-level", vol);
                            }) {
                                eprintln!("Failed to start recording: {}", e);
                            }
                        }

                        // 6. Show overlay window
                        if let Some(overlay) = app_handle.get_webview_window("overlay") {
                            // Position overlay in the bottom center of the primary monitor
                            if let Ok(Some(monitor)) = overlay.primary_monitor() {
                                let size = monitor.size();
                                let scale_factor = monitor.scale_factor();
                                
                                // Convert physical coordinates to logical coordinates
                                let monitor_width = size.width as f64 / scale_factor;
                                let monitor_height = size.height as f64 / scale_factor;
                                
                                // Match width and height from tauri.conf.json
                                let overlay_width = 130.0;
                                let overlay_height = 50.0;
                                
                                // Center horizontally, place ~60px above the bottom (just above the taskbar)
                                let x = (monitor_width - overlay_width) / 2.0;
                                let y = monitor_height - overlay_height - 60.0;
                                
                                let _ = overlay.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
                            }
                            let _ = overlay.show();
                        }

                        // 7. Emit event
                        let _ = app_handle.emit("recording-state", "recording");
                    } else {
                        // Check duration of hotkey press
                        let press_duration = if let Some(state) = app_handle.try_state::<AppState>() {
                            let state = state.inner();
                            if let Ok(mut guard) = state.press_time.lock() {
                                guard.take().map(|t| t.elapsed())
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        if let Some(d) = press_duration {
                            if d.as_millis() < 300 {
                                // Ignore quick taps (accidental press or empty duration)
                                if let Some(state) = app_handle.try_state::<AppState>() {
                                    let state = state.inner();
                                    let _ = state.audio_recorder.stop_recording();
                                }
                                if let Some(overlay) = app_handle.get_webview_window("overlay") {
                                    let _ = overlay.hide();
                                }
                                return;
                            }
                        }

                        // 1. Emit event "processing"
                        let _ = app_handle.emit("recording-state", "processing");

                        // 2. Stop recording
                        if let Some(state) = app_handle.try_state::<AppState>() {
                            let state = state.inner();
                            if let Err(e) = state.audio_recorder.stop_recording() {
                                eprintln!("Failed to stop recording: {}", e);
                            }
                        }

                        // 3. Perform transcription in a background task
                        let app_handle_clone = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            let temp_path = std::env::temp_dir().join("aura-temp.wav");
                            let temp_path_str = temp_path.to_string_lossy().to_string();

                            // Load settings
                            let settings = match settings::load_settings(&app_handle_clone) {
                                Ok(s) => s,
                                Err(e) => {
                                    eprintln!("Failed to load settings: {}", e);
                                    if let Some(overlay) = app_handle_clone.get_webview_window("overlay") {
                                        let _ = overlay.hide();
                                    }
                                    return;
                                }
                            };

                            let selected_text = if let Some(state) = app_handle_clone.try_state::<AppState>() {
                                let state = state.inner();
                                if let Ok(guard) = state.selected_text.lock() {
                                    guard.clone()
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            };

                            // Perform transcription
                            let transcription_result = if settings.transcription_mode == "local" {
                                whisper_runner::run_local_whisper(&app_handle_clone, &settings.model_name, &temp_path_str)
                            } else {
                                let provider = match settings.api_provider.as_str() {
                                    "openai" => ai_client::ApiProvider::OpenAi,
                                    "groq" => ai_client::ApiProvider::Groq,
                                    _ => ai_client::ApiProvider::Gemini,
                                };
                                ai_client::transcribe_and_clean(
                                    provider,
                                    &settings.api_key,
                                    &temp_path_str,
                                    &selected_text,
                                ).await
                            };

                            match transcription_result {
                                Ok(text) => {
                                    let trimmed = text.trim();
                                    if !trimmed.is_empty() && !is_silence_hallucination(trimmed) {
                                        // Save original clipboard
                                        let original_clipboard = if let Ok(mut cb) = arboard::Clipboard::new() {
                                            cb.get_text().ok()
                                        } else {
                                            None
                                        };

                                        // Set clipboard to transcription result
                                        if let Ok(mut cb) = arboard::Clipboard::new() {
                                            let _ = cb.set_text(trimmed.to_string());
                                        }

                                        // Paste text
                                        keyboard_simulator::simulate_paste();

                                        // Sleep 150ms to allow paste to register
                                        tokio::time::sleep(std::time::Duration::from_millis(150)).await;

                                        // Restore original clipboard
                                        if let Some(orig) = original_clipboard {
                                            if let Ok(mut cb) = arboard::Clipboard::new() {
                                                let _ = cb.set_text(orig);
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Transcription error: {}", e);
                                }
                            }

                            // Hide overlay window
                            if let Some(overlay) = app_handle_clone.get_webview_window("overlay") {
                                let _ = overlay.hide();
                            }
                        });
                    }
                });
            }).expect("Failed to start global Win32 keyboard hook");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            download_model_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}



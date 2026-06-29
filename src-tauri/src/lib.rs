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
            });

            // Start global keyboard hook
            keyboard_hook::start_hook(move |is_down| {
                let app_handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    if is_down {
                        // 1. Simulate Copy to capture selection
                        keyboard_simulator::simulate_copy();

                        // 2. Sleep 120ms to allow system clipboard update
                        tokio::time::sleep(std::time::Duration::from_millis(120)).await;

                        // 3. Read clipboard
                        let selected_text = if let Ok(mut cb) = arboard::Clipboard::new() {
                            cb.get_text().unwrap_or_default()
                        } else {
                            String::new()
                        };

                        // Store selected text in state
                        if let Some(state) = app_handle.try_state::<AppState>() {
                            let state = state.inner();
                            if let Ok(mut guard) = state.selected_text.lock() {
                                *guard = selected_text;
                            }
                        }

                        // 4. Start recording to temporary WAV path
                        let temp_path = std::env::temp_dir().join("aura-temp.wav");
                        let temp_path_str = temp_path.to_string_lossy().to_string();

                        if let Some(state) = app_handle.try_state::<AppState>() {
                            let state = state.inner();
                            if let Err(e) = state.audio_recorder.start_recording(&temp_path_str) {
                                eprintln!("Failed to start recording: {}", e);
                            }
                        }

                        // 5. Show overlay window
                        if let Some(overlay) = app_handle.get_webview_window("overlay") {
                            let _ = overlay.show();
                        }

                        // 6. Emit event
                        let _ = app_handle.emit("recording-state", "recording");
                    } else {
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

                            // Detect active layout language
                            let target_lang = keyboard_simulator::get_active_layout_language();

                            // Perform transcription
                            let transcription_result = if settings.transcription_mode == "local" {
                                whisper_runner::run_local_whisper(&app_handle_clone, &settings.model_name, &temp_path_str, &target_lang)
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
                                    &target_lang,
                                ).await
                            };

                            match transcription_result {
                                Ok(text) => {
                                    if !text.is_empty() {
                                        // Save original clipboard
                                        let original_clipboard = if let Ok(mut cb) = arboard::Clipboard::new() {
                                            cb.get_text().ok()
                                        } else {
                                            None
                                        };

                                        // Set clipboard to transcription result
                                        if let Ok(mut cb) = arboard::Clipboard::new() {
                                            let _ = cb.set_text(text);
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



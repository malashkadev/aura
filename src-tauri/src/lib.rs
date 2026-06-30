pub mod keyboard_hook;
pub mod audio_recorder;
pub mod ai_client;
pub mod whisper_runner;
pub mod settings;
pub mod keyboard_simulator;

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Manager, Emitter};

struct AppState {
    audio_recorder: audio_recorder::AudioRecorder,
    selected_text: Mutex<String>,
    press_time: Mutex<Option<std::time::Instant>>,
    is_recording: AtomicBool,
    typed_so_far: Mutex<String>,
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
    if chars_to_delete > 0 {
        keyboard_simulator::type_backspaces(chars_to_delete);
    }
    
    let suffix: String = new_chars[common_prefix_len..].iter().collect();
    if !suffix.is_empty() {
        keyboard_simulator::type_string(&suffix);
    }
    
    *typed_so_far = new_text.to_string();
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

            app.manage(AppState {
                audio_recorder: audio_recorder::AudioRecorder::new(),
                selected_text: Mutex::new(String::new()),
                press_time: Mutex::new(None),
                is_recording: AtomicBool::new(false),
                typed_so_far: Mutex::new(String::new()),
            });

            // Start global keyboard hook
            keyboard_hook::start_hook(move |is_down| {
                let app_handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    if is_down {
                        // Clear typed so far and set is_recording to true
                        if let Some(state) = app_handle.try_state::<AppState>() {
                            let state = state.inner();
                            if let Ok(mut guard) = state.typed_so_far.lock() {
                                *guard = String::new();
                            }
                            state.is_recording.store(true, Ordering::Relaxed);
                            
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
                                let overlay_width = 160.0;
                                let overlay_height = 60.0;
                                
                                // Center horizontally, place ~95px above the bottom (just above the taskbar)
                                let x = (monitor_width - overlay_width) / 2.0;
                                let y = monitor_height - overlay_height - 95.0;
                                
                                let _ = overlay.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
                            }
                            let _ = overlay.show();
                        }

                        // 7. Emit event
                        let _ = app_handle.emit("recording-state", "recording");

                        // 8. Spawn background streaming loop
                        let app_handle_loop = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            // Wait initial 1.5 seconds to gather initial audio
                            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

                            let chunk_path = std::env::temp_dir().join("aura-chunk.wav");
                            let chunk_path_str = chunk_path.to_string_lossy().to_string();

                            loop {
                                let is_recording = if let Some(state) = app_handle_loop.try_state::<AppState>() {
                                    state.is_recording.load(Ordering::Relaxed)
                                } else {
                                    false
                                };

                                if !is_recording {
                                    break;
                                }

                                if let Some(state) = app_handle_loop.try_state::<AppState>() {
                                    let state = state.inner();
                                    if let Ok((samples, sample_rate, channels)) = state.audio_recorder.get_recorded_samples() {
                                        // Ensure we have at least 0.5s of audio (8000 samples)
                                        if samples.len() > 8000 {
                                            if audio_recorder::process_and_write_wav(&samples, channels, sample_rate, &chunk_path_str).is_ok() {
                                                if let Ok(settings) = settings::load_settings(&app_handle_loop) {
                                                    let selected_text = if let Ok(guard) = state.selected_text.lock() {
                                                        guard.clone()
                                                    } else {
                                                        String::new()
                                                    };

                                                    let transcription_result = if settings.transcription_mode == "local" {
                                                        whisper_runner::run_local_whisper(&app_handle_loop, &settings.model_name, &chunk_path_str)
                                                    } else {
                                                        let provider = match settings.api_provider.as_str() {
                                                            "openai" => ai_client::ApiProvider::OpenAi,
                                                            "groq" => ai_client::ApiProvider::Groq,
                                                            _ => ai_client::ApiProvider::Gemini,
                                                        };
                                                        ai_client::transcribe_and_clean(
                                                            provider,
                                                            &settings.api_key,
                                                            &chunk_path_str,
                                                            &selected_text,
                                                        ).await
                                                    };

                                                    if let Ok(text) = transcription_result {
                                                        let trimmed = text.trim();
                                                        if !trimmed.is_empty() && !is_silence_hallucination(trimmed) {
                                                            if let Ok(mut typed_guard) = state.typed_so_far.lock() {
                                                                diff_and_type(&mut *typed_guard, trimmed);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Wait another 1.5 seconds for next chunk
                                tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                            }
                        });
                    } else {
                        // Check duration of hotkey press
                        let press_duration = if let Some(state) = app_handle.try_state::<AppState>() {
                            let state = state.inner();
                            state.is_recording.store(false, Ordering::Relaxed);
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

                        // 3. Perform final transcription in a background task
                        let app_handle_clone = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            // Wait a short moment to ensure background loop is stopped and doesn't conflict
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

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

                            // Perform transcription on the full audio file
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

                            if let Ok(text) = transcription_result {
                                let trimmed = text.trim();
                                if !trimmed.is_empty() && !is_silence_hallucination(trimmed) {
                                    if let Some(state) = app_handle_clone.try_state::<AppState>() {
                                        let state = state.inner();
                                        if let Ok(mut typed_guard) = state.typed_so_far.lock() {
                                            diff_and_type(&mut *typed_guard, trimmed);
                                        }
                                    }
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



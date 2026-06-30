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
    selected_language: Mutex<String>,
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
                selected_language: Mutex::new(String::new()),
            });

            // Start global keyboard hook
            keyboard_hook::start_hook(move |is_down| {
                let app_handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    if is_down {
                        eprintln!("Aura Dev Log: Alt+V pressed (is_down = true)");
                        // Clear typed so far, detect active language, and set is_recording to true
                        if let Some(state) = app_handle.try_state::<AppState>() {
                            let state = state.inner();
                            
                            // Detect active keyboard language at the moment of press
                            let lang = keyboard_simulator::get_active_layout_language();
                            eprintln!("Aura Dev Log: Active layout language = {}", lang);
                            if let Ok(mut guard) = state.selected_language.lock() {
                                *guard = lang;
                            }

                            if let Ok(mut guard) = state.typed_so_far.lock() {
                                *guard = String::new();
                            }
                            state.is_recording.store(true, Ordering::Relaxed);
                            
                            if let Ok(mut guard) = state.press_time.lock() {
                                *guard = Some(std::time::Instant::now());
                            }
                        }

                        // Start recording to temporary WAV path
                        let temp_path = std::env::temp_dir().join("aura-temp.wav");
                        let temp_path_str = temp_path.to_string_lossy().to_string();
                        eprintln!("Aura Dev Log: Starting audio recording to {}", temp_path_str);

                        let app_handle_clone = app_handle.clone();
                        if let Some(state) = app_handle.try_state::<AppState>() {
                            let state = state.inner();
                            if let Err(e) = state.audio_recorder.start_recording(&temp_path_str, move |vol| {
                                let _ = app_handle_clone.emit("volume-level", vol);
                            }) {
                                eprintln!("Aura Dev Log ERROR: Failed to start recording: {}", e);
                            }
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
                                let overlay_height = 60.0;
                                
                                // Center horizontally, place ~95px above the bottom (just above the taskbar)
                                let x = (monitor_width - overlay_width) / 2.0;
                                let y = monitor_height - overlay_height - 95.0;
                                
                                let _ = overlay.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
                            }
                            let _ = overlay.show();
                        }

                        // Emit event
                        let _ = app_handle.emit("recording-state", "recording");

                        // Spawn background streaming loop if enabled in settings
                        if let Ok(settings) = settings::load_settings(&app_handle) {
                            if settings.streaming_enabled {
                                let app_handle_loop = app_handle.clone();
                                tauri::async_runtime::spawn(async move {
                                    eprintln!("Aura Dev Log: Spawning background streaming loop task...");
                                    // Wait initial 4.0 seconds to gather initial audio to stay under Groq 20 RPM limit
                                    tokio::time::sleep(std::time::Duration::from_millis(4000)).await;

                                    let chunk_path = std::env::temp_dir().join("aura-chunk.wav");
                                    let chunk_path_str = chunk_path.to_string_lossy().to_string();

                                    loop {
                                        let is_recording = if let Some(state) = app_handle_loop.try_state::<AppState>() {
                                            state.is_recording.load(Ordering::Relaxed)
                                        } else {
                                            false
                                        };

                                        if !is_recording {
                                            eprintln!("Aura Dev Log: is_recording is false. Exiting streaming loop.");
                                            break;
                                        }

                                        eprintln!("Aura Dev Log: Loop tick - reading samples...");
                                        if let Some(state) = app_handle_loop.try_state::<AppState>() {
                                            let state = state.inner();
                                            if let Ok((samples, sample_rate, channels)) = state.audio_recorder.get_recorded_samples() {
                                                eprintln!("Aura Dev Log: Retrieved {} samples (rate={}, channels={})", samples.len(), sample_rate, channels);
                                                // Ensure we have at least 0.5s of audio (8000 samples)
                                                if samples.len() > 8000 {
                                                    let write_res = audio_recorder::process_and_write_wav(&samples, channels, sample_rate, &chunk_path_str);
                                                    eprintln!("Aura Dev Log: process_and_write_wav result = {:?}", write_res);
                                                    if write_res.is_ok() {
                                                        if let Ok(settings) = settings::load_settings(&app_handle_loop) {
                                                            let selected_language = if let Ok(guard) = state.selected_language.lock() {
                                                                guard.clone()
                                                            } else {
                                                                "ru".to_string()
                                                            };

                                                            eprintln!("Aura Dev Log: Calling streaming transcribe_and_clean...");
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
                                                                    "", // No selected text during live streaming
                                                                    &selected_language,
                                                                    true,
                                                                ).await
                                                            };

                                                            match transcription_result {
                                                                Ok(text) => {
                                                                    let trimmed = text.trim();
                                                                    eprintln!("Aura Dev Log: Streaming transcription success: '{}'", trimmed);
                                                                    if !trimmed.is_empty() && !is_silence_hallucination(trimmed) {
                                                                        if let Ok(mut typed_guard) = state.typed_so_far.lock() {
                                                                            diff_and_type(&mut *typed_guard, trimmed);
                                                                        }
                                                                    }
                                                                }
                                                                Err(e) => {
                                                                    eprintln!("Aura Dev Log ERROR: Streaming transcription failed: {}", e);
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    eprintln!("Aura Dev Log: Not enough samples ({} <= 8000), skipping transcription.", samples.len());
                                                }
                                            }
                                        }

                                        // Wait another 4.0 seconds for next chunk to stay under Groq 20 RPM limit
                                        tokio::time::sleep(std::time::Duration::from_millis(4000)).await;
                                    }
                                });
                            }
                        }
                    } else {
                        eprintln!("Aura Dev Log: Alt+V released (is_down = false)");
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
                            eprintln!("Aura Dev Log: Press duration = {} ms", d.as_millis());
                            if d.as_millis() < 300 {
                                eprintln!("Aura Dev Log: Press too short (< 300ms), discarding.");
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

                        // Emit event "processing"
                        let _ = app_handle.emit("recording-state", "processing");

                        // Stop recording
                        if let Some(state) = app_handle.try_state::<AppState>() {
                            let state = state.inner();
                            let stop_res = state.audio_recorder.stop_recording();
                            eprintln!("Aura Dev Log: stop_recording result = {:?}", stop_res);
                        }

                        let streaming_enabled = if let Ok(settings) = settings::load_settings(&app_handle) {
                            settings.streaming_enabled
                        } else {
                            false
                        };

                        if !streaming_enabled {
                            if let Some(overlay) = app_handle.get_webview_window("overlay") {
                                let _ = overlay.hide();
                            }
                        }

                        // Perform final transcription in a background task
                        let app_handle_clone = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            // Only wait if streaming was enabled to let the loop exit cleanly
                            if streaming_enabled {
                                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                            }

                            let temp_path = std::env::temp_dir().join("aura-temp.wav");
                            let temp_path_str = temp_path.to_string_lossy().to_string();

                            // Load settings
                            let settings = match settings::load_settings(&app_handle_clone) {
                                Ok(s) => s,
                                Err(e) => {
                                    eprintln!("Aura Dev Log ERROR: Failed to load settings: {}", e);
                                    if let Some(overlay) = app_handle_clone.get_webview_window("overlay") {
                                        let _ = overlay.hide();
                                    }
                                    return;
                                }
                            };

                            let (selected_language, selected_text) = if let Some(state) = app_handle_clone.try_state::<AppState>() {
                                let state = state.inner();
                                let lang = if let Ok(guard) = state.selected_language.lock() {
                                    guard.clone()
                                } else {
                                    "ru".to_string()
                                };
                                let text = if let Ok(guard) = state.selected_text.lock() {
                                    guard.clone()
                                } else {
                                    String::new()
                                };
                                (lang, text)
                            } else {
                                ("ru".to_string(), String::new())
                            };

                            eprintln!("Aura Dev Log: Calling final transcribe_and_clean...");
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
                                    &selected_language,
                                    true,
                                ).await
                            };

                            match transcription_result {
                                Ok(text) => {
                                    let trimmed = text.trim();
                                    eprintln!("Aura Dev Log: Final transcription success: '{}'", trimmed);
                                    if !trimmed.is_empty() && !is_silence_hallucination(trimmed) {
                                        if let Some(state) = app_handle_clone.try_state::<AppState>() {
                                            let state = state.inner();
                                            let streaming_enabled = if let Ok(settings) = settings::load_settings(&app_handle_clone) {
                                                settings.streaming_enabled
                                            } else {
                                                false
                                            };

                                            if streaming_enabled {
                                                if let Ok(mut typed_guard) = state.typed_so_far.lock() {
                                                    // Perform a smart diff replacement to only touch changed suffixes and avoid erasing the whole line
                                                    diff_and_type(&mut *typed_guard, trimmed);
                                                }
                                            } else {
                                                // Perform instantaneous clipboard paste for classic stable mode (v1.0 behavior)
                                                let original_clipboard = if let Ok(mut cb) = arboard::Clipboard::new() {
                                                    cb.get_text().ok()
                                                } else {
                                                    None
                                                };

                                                if let Ok(mut cb) = arboard::Clipboard::new() {
                                                    let _ = cb.set_text(trimmed.to_string());
                                                }

                                                keyboard_simulator::simulate_paste();

                                                tokio::time::sleep(std::time::Duration::from_millis(150)).await;

                                                if let Some(orig) = original_clipboard {
                                                    if let Ok(mut cb) = arboard::Clipboard::new() {
                                                        let _ = cb.set_text(orig);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Aura Dev Log ERROR: Final transcription failed: {}", e);
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



use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::sync::Mutex;
use std::thread;
use std::time::SystemTime;
use tauri::Manager;

pub const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024; // 10 MB
pub const LOG_FILE_NAME: &str = "aura_diagnostics.log";
const LOGGER_QUEUE_CAPACITY: usize = 2_048;

static LOGGER_TX: Mutex<Option<SyncSender<String>>> = Mutex::new(None);
static DROPPED_LOG_LINES: AtomicU64 = AtomicU64::new(0);
static LOGGER_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn anonymize_speech(text: &str, log_speech_text: bool) -> String {
    if log_speech_text {
        text.to_string()
    } else {
        let word_count = text.split_whitespace().count();
        let char_count = text.chars().count();
        format!("[REDACTED: {} words, {} chars]", word_count, char_count)
    }
}

pub fn format_session_tag(session_id: u64) -> String {
    format!("#session-{}", session_id)
}

pub fn init(log_dir: PathBuf) {
    if let Err(e) = fs::create_dir_all(&log_dir) {
        eprintln!("[LOGGER ERROR] Failed to create log directory: {}", e);
        return;
    }

    let (tx, rx) = mpsc::sync_channel::<String>(LOGGER_QUEUE_CAPACITY);
    let dir = log_dir.clone();

    thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            if let Err(e) = rotate_logs_if_needed(&dir, MAX_LOG_SIZE) {
                eprintln!("[LOGGER ERROR] Failed log rotation: {}", e);
            }
            let log_path = dir.join(LOG_FILE_NAME);
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) {
                let _ = writeln!(file, "{}", msg);
            }
        }
    });

    if let Ok(mut lock) = LOGGER_TX.lock() {
        *lock = Some(tx);
    }
    if let Ok(mut lock) = LOGGER_DIR.lock() {
        *lock = Some(log_dir);
    }
}

pub fn log(level: &str, tag: &str, session: Option<&str>, message: &str) {
    let timestamp = format_timestamp(SystemTime::now());
    let level_str = level.to_uppercase();
    let session_str = match session {
        Some(s) if s.starts_with('[') => format!(" {}", s),
        Some(s) => format!(" [{}]", s),
        None => String::new(),
    };

    let formatted_line = format!(
        "{} [{}] [{}]{} {}",
        timestamp, level_str, tag, session_str, message
    );
    eprintln!("{}", formatted_line);

    let lock = match LOGGER_TX.lock() {
        Ok(lock) => lock,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some(tx) = lock.as_ref() {
        match tx.try_send(formatted_line) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                let dropped = DROPPED_LOG_LINES.fetch_add(1, Ordering::Relaxed) + 1;
                if dropped == 1 || dropped.is_multiple_of(1_000) {
                    eprintln!(
                        "[LOGGER WARN] Dropped {dropped} log lines because the disk writer is behind"
                    );
                }
            }
            Err(TrySendError::Disconnected(_)) => {
                eprintln!("[LOGGER ERROR] Background log writer is unavailable");
            }
        }
    }
}

pub fn rotate_logs_if_needed(log_dir: &Path, max_bytes: u64) -> std::io::Result<()> {
    let log_path = log_dir.join(LOG_FILE_NAME);
    if log_path.exists() {
        if let Ok(meta) = fs::metadata(&log_path) {
            if meta.len() >= max_bytes {
                let log_1_path = log_dir.join(format!("{}.1", LOG_FILE_NAME));
                let log_2_path = log_dir.join(format!("{}.2", LOG_FILE_NAME));

                if log_2_path.exists() {
                    let _ = fs::remove_file(&log_2_path);
                }
                if log_1_path.exists() {
                    let _ = fs::rename(&log_1_path, &log_2_path);
                }
                let _ = fs::rename(&log_path, &log_1_path);
                File::create(&log_path)?;
            }
        }
    }
    Ok(())
}

pub fn get_recent_logs(max_lines: usize) -> Vec<String> {
    let log_dir = match LOGGER_DIR.lock() {
        Ok(lock) => match lock.as_ref() {
            Some(dir) => dir.clone(),
            None => return Vec::new(),
        },
        Err(_) => return Vec::new(),
    };

    let log_path = log_dir.join(LOG_FILE_NAME);
    if !log_path.exists() {
        return Vec::new();
    }

    let file = match File::open(&log_path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();

    if lines.len() > max_lines {
        lines[lines.len() - max_lines..].to_vec()
    } else {
        lines
    }
}

fn format_timestamp(now: SystemTime) -> String {
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    let days = (secs / 86400) as i64;
    let rem_secs = (secs % 86400) as u32;
    let hours = rem_secs / 3600;
    let mins = (rem_secs % 3600) / 60;
    let seconds = rem_secs % 60;

    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}Z",
        year, m, d, hours, mins, seconds, millis
    )
}

pub fn generate_diagnostic_report<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Result<String, String> {
    let version = app_handle.package_info().version.to_string();
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let settings = crate::settings::load_settings(app_handle).unwrap_or_default();

    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {}", e))?;

    let parakeet_dir = app_local_data.join("models").join("parakeet-v3");
    let parakeet_encoder = parakeet_dir.join("encoder.onnx").exists();
    let parakeet_decoder = parakeet_dir.join("decoder.onnx").exists();
    let parakeet_joiner = parakeet_dir.join("joiner.onnx").exists();
    let parakeet_tokens = parakeet_dir.join("tokens.txt").exists();
    let parakeet_complete =
        parakeet_encoder && parakeet_decoder && parakeet_joiner && parakeet_tokens;

    let punc_dir = app_local_data.join("models").join("punctuation");
    let punc_model = punc_dir.join("model.int8.onnx").exists();

    let cuda_bin_dir = app_local_data.join("binaries").join("cuda").join("bin");
    let cuda_dll = cuda_bin_dir.join("onnxruntime_providers_cuda.dll").exists();
    let cuda_exe = cuda_bin_dir
        .join("sherpa-onnx-offline-websocket-server.exe")
        .exists();
    let cuda_runtime_source = crate::cuda_runtime_source_label(&cuda_bin_dir);

    let mut whisper_models = Vec::new();
    let models_dir = app_local_data.join("models");
    if models_dir.exists() {
        if let Ok(entries) = fs::read_dir(&models_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                        if filename.starts_with("ggml-") && filename.ends_with(".bin") {
                            let name = &filename[5..filename.len() - 4];
                            whisper_models.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    let raw_logs = get_recent_logs(50);
    let logs = sanitize_logs_for_report(&raw_logs, settings.log_speech_text);

    Ok(format_diagnostic_report(
        &version,
        os,
        arch,
        &settings.transcription_mode,
        &settings.local_engine,
        &settings.model_name,
        &settings.local_acceleration,
        cuda_runtime_source,
        settings.streaming_enabled,
        settings.voice_punctuation,
        settings.cloud_fallback_enabled,
        parakeet_complete,
        parakeet_encoder,
        parakeet_decoder,
        parakeet_joiner,
        parakeet_tokens,
        punc_model,
        cuda_dll,
        cuda_exe,
        &whisper_models,
        &logs,
        &engine_status(app_handle, settings.local_engine.as_str()),
    ))
}

/// Live engine state for the diagnostic report: the actual sidecar process for
/// Parakeet, or the in-process nature of Whisper.
fn engine_status<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
    local_engine: &str,
) -> String {
    if local_engine == "whisper" {
        return match crate::whisper_runner::whisper_server_status(app_handle) {
            Some((provider, port)) => {
                format!("Resident Server (provider: {provider}, port: {port})")
            }
            None => "In-process (whisper)".to_string(),
        };
    }
    if local_engine != "parakeet" {
        return format!("In-process ({local_engine})");
    }
    match crate::whisper_runner::parakeet_server_status(app_handle) {
        Some((provider, port)) => format!("Running (provider: {provider}, port: {port})"),
        None => "Not started".to_string(),
    }
}

pub fn sanitize_logs_for_report(logs: &[String], log_speech_text: bool) -> Vec<String> {
    if log_speech_text {
        return logs.to_vec();
    }

    logs.iter()
        .map(|line| {
            if let Some(pos) = line.find("Final result: ") {
                let prefix = &line[..pos + "Final result: ".len()];
                let speech = &line[pos + "Final result: ".len()..];
                if !speech.starts_with("[REDACTED:") {
                    let redacted = anonymize_speech(speech, false);
                    format!("{}{}", prefix, redacted)
                } else {
                    line.clone()
                }
            } else {
                line.clone()
            }
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub fn format_diagnostic_report(
    version: &str,
    os: &str,
    arch: &str,
    transcription_mode: &str,
    local_engine: &str,
    model_name: &str,
    acceleration: &str,
    cuda_runtime_source: &str,
    streaming_enabled: bool,
    voice_punctuation: bool,
    cloud_fallback: bool,
    parakeet_complete: bool,
    parakeet_encoder: bool,
    parakeet_decoder: bool,
    parakeet_joiner: bool,
    parakeet_tokens: bool,
    punctuation_model: bool,
    cuda_dll: bool,
    cuda_exe: bool,
    whisper_models: &[String],
    logs: &[String],
    engine_status: &str,
) -> String {
    let whisper_models_str = if whisper_models.is_empty() {
        "None".to_string()
    } else {
        whisper_models.join(", ")
    };

    let log_content = if logs.is_empty() {
        "(No logs recorded)".to_string()
    } else {
        logs.join("\n")
    };

    format!(
        "# Aura Diagnostic Report\n\n\
        ## System & Config Specs\n\
        - **App Version**: {}\n\
        - **OS**: {}\n\
        - **Architecture**: {}\n\
        - **Transcription Mode**: {}\n\
- **Local Engine**: {}\n\
        - **Engine Status**: {}\n\
        - **Selected Model**: {}\n\
        - **Acceleration**: {}\n\
        - **CUDA Runtime Source**: {}\n\
        - **Real-time Streaming**: {}\n\
        - **Voice Punctuation**: {}\n\
        - **Cloud Fallback**: {}\n\n\
        ## Component Verification\n\
        - **Parakeet V3 Model**: {}\n  \
          - encoder.onnx: {}\n  \
          - decoder.onnx: {}\n  \
          - joiner.onnx: {}\n  \
          - tokens.txt: {}\n\
        - **Punctuation Model**: {}\n\
        - **CUDA Binaries**: {}\n  \
          - onnxruntime_providers_cuda.dll: {}\n  \
          - sherpa-onnx-offline-websocket-server.exe: {}\n\
        - **Whisper GGML Models**: {}\n\n\
        ## Unified Log (Last 50 Lines)\n\
        ```\n\
        {}\n\
        ```\n",
        version,
        os,
        arch,
        transcription_mode,
        local_engine,
        engine_status,
        model_name,
        acceleration,
        cuda_runtime_source,
        streaming_enabled,
        voice_punctuation,
        cloud_fallback,
        if parakeet_complete {
            "Installed"
        } else {
            "Incomplete/Missing"
        },
        if parakeet_encoder {
            "Present"
        } else {
            "Missing"
        },
        if parakeet_decoder {
            "Present"
        } else {
            "Missing"
        },
        if parakeet_joiner {
            "Present"
        } else {
            "Missing"
        },
        if parakeet_tokens {
            "Present"
        } else {
            "Missing"
        },
        if punctuation_model {
            "Present"
        } else {
            "Missing"
        },
        if cuda_dll || cuda_exe {
            "Present"
        } else {
            "Missing"
        },
        if cuda_dll { "Present" } else { "Missing" },
        if cuda_exe { "Present" } else { "Missing" },
        whisper_models_str,
        log_content
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_anonymize_speech() {
        assert_eq!(
            anonymize_speech("hello world from aura", true),
            "hello world from aura"
        );
        assert_eq!(
            anonymize_speech("hello world from aura", false),
            "[REDACTED: 4 words, 21 chars]"
        );
        assert_eq!(anonymize_speech("", false), "[REDACTED: 0 words, 0 chars]");
    }

    #[test]
    fn test_format_session_tag() {
        assert_eq!(format_session_tag(123456), "#session-123456");
        assert_eq!(format_session_tag(0), "#session-0");
    }

    #[test]
    fn test_log_rotation() {
        let unique_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("aura_log_test_{}", unique_id));
        fs::create_dir_all(&temp_dir).unwrap();

        let log_file = temp_dir.join(LOG_FILE_NAME);
        fs::write(&log_file, "A".repeat(100)).unwrap();

        // 1st rotation: log file size 100 >= threshold 50 -> rotated to log.1
        rotate_logs_if_needed(&temp_dir, 50).unwrap();
        assert!(temp_dir.join(LOG_FILE_NAME).exists());
        assert!(temp_dir.join(format!("{}.1", LOG_FILE_NAME)).exists());
        assert_eq!(
            fs::read_to_string(temp_dir.join(format!("{}.1", LOG_FILE_NAME))).unwrap(),
            "A".repeat(100)
        );

        // Fill current log file again
        fs::write(&log_file, "B".repeat(100)).unwrap();

        // 2nd rotation: log file size 100 >= threshold 50 -> log.1 becomes log.2, current becomes log.1
        rotate_logs_if_needed(&temp_dir, 50).unwrap();
        assert!(temp_dir.join(LOG_FILE_NAME).exists());
        assert!(temp_dir.join(format!("{}.1", LOG_FILE_NAME)).exists());
        assert!(temp_dir.join(format!("{}.2", LOG_FILE_NAME)).exists());
        assert_eq!(
            fs::read_to_string(temp_dir.join(format!("{}.1", LOG_FILE_NAME))).unwrap(),
            "B".repeat(100)
        );
        assert_eq!(
            fs::read_to_string(temp_dir.join(format!("{}.2", LOG_FILE_NAME))).unwrap(),
            "A".repeat(100)
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_format_diagnostic_report() {
        let logs = vec![
            "2026-07-20 00:00:00.000Z [INFO] [System] Test log line 1".to_string(),
            "2026-07-20 00:00:01.000Z [WARN] [Audio] Test log line 2".to_string(),
        ];
        let whisper_models = vec!["small".to_string(), "base".to_string()];

        let report = format_diagnostic_report(
            "1.0.8",
            "windows",
            "x86_64",
            "cloud",
            "parakeet",
            "parakeet-v3",
            "cuda",
            "System NVIDIA runtime (PATH)",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            &whisper_models,
            &logs,
            "Running (provider: cuda, port: 3033)",
        );

        assert!(report.contains("# Aura Diagnostic Report"));
        assert!(report.contains("## System & Config Specs"));
        assert!(report.contains("## Component Verification"));
        assert!(report.contains("## Unified Log (Last 50 Lines)"));
        assert!(report.contains("- **App Version**: 1.0.8"));
        assert!(report.contains("- **Engine Status**: Running (provider: cuda, port: 3033)"));
        assert!(report.contains("- **Selected Model**: parakeet-v3"));
        assert!(report.contains("- **CUDA Runtime Source**: System NVIDIA runtime (PATH)"));
        assert!(report.contains("- **Real-time Streaming**: true"));
        assert!(report.contains("Test log line 1"));
        assert!(report.contains("Test log line 2"));
    }

    #[test]
    fn test_sanitize_logs_for_report() {
        let logs = vec![
            "2026-07-20 00:58:11.597Z [INFO] [ASR] [#session-1] Final result: Hello world test speech".to_string(),
            "2026-07-20 00:58:35.784Z [INFO] [ASR] [#session-2] Final result: [REDACTED: 3 words, 12 chars]".to_string(),
        ];

        let sanitized_off = sanitize_logs_for_report(&logs, false);
        assert_eq!(sanitized_off[0], "2026-07-20 00:58:11.597Z [INFO] [ASR] [#session-1] Final result: [REDACTED: 4 words, 23 chars]");
        assert_eq!(sanitized_off[1], "2026-07-20 00:58:35.784Z [INFO] [ASR] [#session-2] Final result: [REDACTED: 3 words, 12 chars]");

        let sanitized_on = sanitize_logs_for_report(&logs, true);
        assert_eq!(sanitized_on[0], logs[0]);
        assert_eq!(sanitized_on[1], logs[1]);
    }
}
